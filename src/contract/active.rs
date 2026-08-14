use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;

use reqwest::Url;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Number, Value};

use super::active_protocol::{
    ActiveProtocolAdapter, ActiveStartKind, ActiveTaskSupport, ActiveToolResultKind,
};
use super::catalog::{
    InstanceValidationIssue, LocalValidator, validate_cacheable_result,
    validate_discovery_capabilities, validate_legacy_capabilities, validate_local_schema,
    validate_local_schema_with_policy,
};
use super::generate::{
    GenerationFailure, INVALID_MUTATION_KINDS, generate_inputs, generate_invalid_inputs,
};
use super::http_headers::{HeaderAnnotation, validate_annotations};
use super::limits::{DiagnosticLimits, LimitKind, LimitViolation};
use super::model::{
    CheckId, CheckResult, ExpectedShape, Finding, FindingCode, FindingEvidence,
    GeneratedCaseReproduction, JsonKind, Location, LocationField, Requirement, RuleViolation,
    SkipReason,
};
use super::protocol::{
    ActiveProtocolRevision, KnownRevision, RevisionSelection, SupportedRevision,
    select_server_revision,
};
use super::redaction::RedactedValue;
use super::report::{DiagnosticReport, ExitStatus};
use super::{
    Diagnostic, HttpDiagnostic, ReportTransport, StdioDiagnostic, http_checks_for_revision,
    stdio_findings_for_revision,
};
use crate::transport::{Conversation, ProbeRequest, ProbeResponse};

pub(crate) const SCENARIO_SCHEMA_VERSION: &str = "mcp-doctor.scenario/v1alpha1";
pub(crate) const WORKFLOW_SCHEMA_VERSION: &str = "mcp-doctor.scenario/v2alpha1";
pub(crate) const MAX_SCENARIO_BYTES: u64 = 1_048_576;
pub(crate) const REJECTION_CASE_COUNT: usize = INVALID_MUTATION_KINDS.len();
const MAX_WORKFLOW_NAME_CHARS: usize = 1_024;
const MAX_WORKFLOW_POINTER_CHARS: usize = 8_192;

pub(crate) struct ScenarioFailure {
    findings: Vec<Finding>,
}

impl ScenarioFailure {
    fn one(finding: Finding) -> Self {
        Self {
            findings: vec![finding],
        }
    }

    pub(crate) fn file_limit(observed: u64) -> Self {
        let violation = LimitViolation::new(LimitKind::ScenarioBytes, observed, MAX_SCENARIO_BYTES)
            .expect("an oversized scenario exceeds its checked maximum");
        Self::one(Finding::limit_exceeded(
            SupportedRevision::CURRENT,
            scenario_location(),
            violation,
        ))
    }

    pub(crate) fn unreadable() -> Self {
        Self::one(Finding::scenario_invalid(
            scenario_location(),
            RuleViolation::InvalidScenarioShape,
        ))
    }
}

pub(crate) struct ActiveScenario {
    tools: BTreeSet<String>,
    side_effecting: bool,
    target_env: Vec<String>,
    cases: Vec<ScenarioCase>,
    source: CaseSource,
    resolved_input_bytes: u64,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ScenarioEffects {
    ReadOnly,
    SideEffecting,
}

struct ScenarioCase {
    tool: String,
    arguments: Value,
    omit_arguments: bool,
    applicable: bool,
    secret_refs: BTreeMap<String, String>,
    argument_refs: BTreeMap<String, String>,
    captures: BTreeMap<String, String>,
    cleanup: bool,
    expected: ExpectedResult,
    output_validator: Option<LocalValidator>,
    reproduction: Option<GeneratedCaseReproduction>,
}

#[derive(Clone, Copy)]
enum CaseSource {
    ReviewedV1,
    Workflow { first_cleanup: usize },
    Generated { seed: u64, requested_cases: usize },
    Rejection { seed: u64 },
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ExpectedResult {
    Success,
    ToolError,
    InvalidArgumentsRejection,
}

enum ArgumentReferenceFailure {
    Unavailable,
    Limit(LimitViolation),
}

enum CaptureFailure {
    Missing,
    Limit(LimitViolation),
}

impl ActiveScenario {
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, ScenarioFailure> {
        let UniqueValue(value) = serde_json::from_slice::<UniqueValue>(bytes).map_err(|_| {
            ScenarioFailure::one(Finding::scenario_invalid(
                scenario_location(),
                RuleViolation::InvalidScenarioShape,
            ))
        })?;
        let root = value.as_object().ok_or_else(|| {
            shape_failure(
                scenario_location(),
                ExpectedShape::Object,
                json_kind(Some(&value)),
            )
        })?;
        let schema_version = root.get("schema_version").and_then(Value::as_str);
        if schema_version == Some(WORKFLOW_SCHEMA_VERSION) {
            return Self::parse_workflow(root);
        }
        if schema_version != Some(SCENARIO_SCHEMA_VERSION) {
            return Err(ScenarioFailure::one(Finding::scenario_invalid(
                scenario_location().field(LocationField::SchemaVersion),
                RuleViolation::UnsupportedScenarioVersion,
            )));
        }
        ensure_fields(
            root,
            &["schema_version", "tool", "safety", "target_env", "cases"],
            &["schema_version", "tool", "safety", "cases"],
            scenario_location(),
        )?;

        let tool = required_nonempty_string(
            root.get("tool"),
            scenario_location().field(LocationField::Tools),
        )?
        .to_owned();

        let safety_location = scenario_location().field(LocationField::Safety);
        let safety = required_object(root.get("safety"), safety_location.clone())?;
        ensure_fields(safety, &["effects"], &["effects"], safety_location.clone())?;
        let effects = match safety.get("effects").and_then(Value::as_str) {
            Some("read_only") => ScenarioEffects::ReadOnly,
            Some("side_effecting") => ScenarioEffects::SideEffecting,
            _ => {
                return Err(ScenarioFailure::one(Finding::scenario_invalid(
                    safety_location.field(LocationField::Effects),
                    RuleViolation::InvalidScenarioShape,
                )));
            }
        };

        let target_env = parse_target_environment(root)?;

        let cases_value = root.get("cases").expect("required fields were checked");
        let cases_array = cases_value.as_array().ok_or_else(|| {
            shape_failure(
                scenario_location().field(LocationField::Cases),
                ExpectedShape::Array,
                json_kind(Some(cases_value)),
            )
        })?;
        let maximum_cases = DiagnosticLimits::M1_DEFAULTS.values().active_cases;
        if cases_array.is_empty() {
            return Err(ScenarioFailure::one(Finding::scenario_invalid(
                scenario_location().field(LocationField::Cases),
                RuleViolation::InvalidScenarioShape,
            )));
        }
        let observed_cases = u64::try_from(cases_array.len()).unwrap_or(u64::MAX);
        if observed_cases > maximum_cases {
            return Err(ScenarioFailure::one(Finding::limit_exceeded(
                SupportedRevision::CURRENT,
                scenario_location().field(LocationField::Cases),
                LimitViolation::new(LimitKind::ActiveCases, observed_cases, maximum_cases)
                    .expect("the scenario case count exceeds its checked maximum"),
            )));
        }

        let mut case_ids = BTreeSet::new();
        let mut cases = Vec::with_capacity(cases_array.len());
        for (index, value) in cases_array.iter().enumerate() {
            let base = case_location(index);
            let object = required_object(Some(value), base.clone())?;
            ensure_fields(
                object,
                &["id", "arguments", "secret_refs", "expect"],
                &["id", "arguments", "expect"],
                base.clone(),
            )?;
            let id =
                required_nonempty_string(object.get("id"), base.clone().field(LocationField::Id))?
                    .to_owned();
            if !case_ids.insert(id.clone()) {
                return Err(ScenarioFailure::one(Finding::scenario_invalid(
                    base.clone().field(LocationField::Id),
                    RuleViolation::DuplicateCaseId,
                )));
            }

            let arguments = object
                .get("arguments")
                .expect("required fields were checked");
            if !arguments.is_object() {
                return Err(shape_failure(
                    base.clone().field(LocationField::Arguments),
                    ExpectedShape::Object,
                    json_kind(Some(arguments)),
                ));
            }
            check_instance_bytes(arguments, base.clone().field(LocationField::Arguments))?;

            let mut secret_refs = BTreeMap::new();
            if let Some(value) = object.get("secret_refs") {
                let references = value.as_object().ok_or_else(|| {
                    shape_failure(
                        base.clone().field(LocationField::SecretRefs),
                        ExpectedShape::Object,
                        json_kind(Some(value)),
                    )
                })?;
                for (pointer, source) in references {
                    let location = base.clone().field(LocationField::SecretRefs).wildcard();
                    let Some(source) = source.as_str() else {
                        return Err(shape_failure(
                            location,
                            ExpectedShape::String,
                            json_kind(Some(source)),
                        ));
                    };
                    if !valid_json_pointer(pointer)
                        || !valid_environment_name(source)
                        || arguments.pointer(pointer) != Some(&Value::Null)
                    {
                        return Err(ScenarioFailure::one(Finding::secret_reference_invalid(
                            location,
                            RuleViolation::InvalidEnvironmentReference,
                        )));
                    }
                    secret_refs.insert(pointer.clone(), source.to_owned());
                }
            }

            let expect_location = base.clone().field(LocationField::Expect);
            let expect = required_object(object.get("expect"), expect_location.clone())?;
            ensure_fields(
                expect,
                &["result", "structured_output_schema"],
                &["result"],
                expect_location.clone(),
            )?;
            let expected = match expect.get("result").and_then(Value::as_str) {
                Some("success") => ExpectedResult::Success,
                Some("tool_error") => ExpectedResult::ToolError,
                _ => {
                    return Err(ScenarioFailure::one(Finding::scenario_invalid(
                        expect_location.clone().field(LocationField::Result),
                        RuleViolation::InvalidScenarioShape,
                    )));
                }
            };
            let output_validator = if let Some(schema) = expect.get("structured_output_schema") {
                let location = expect_location.field(LocationField::StructuredOutputSchema);
                if !schema.is_object() {
                    return Err(shape_failure(
                        location,
                        ExpectedShape::Object,
                        json_kind(Some(schema)),
                    ));
                }
                let findings = scenario_schema_findings(validate_local_schema(schema, location));
                if !findings.is_empty() {
                    return Err(ScenarioFailure { findings });
                }
                Some(
                    LocalValidator::compile(schema)
                        .expect("a validated scenario schema compiles without retrieval"),
                )
            } else {
                None
            };

            cases.push(ScenarioCase {
                tool: tool.clone(),
                arguments: arguments.clone(),
                omit_arguments: false,
                applicable: true,
                secret_refs,
                argument_refs: BTreeMap::new(),
                captures: BTreeMap::new(),
                cleanup: false,
                expected,
                output_validator,
                reproduction: None,
            });
        }

        Ok(Self {
            tools: BTreeSet::from([tool]),
            side_effecting: effects == ScenarioEffects::SideEffecting,
            target_env,
            cases,
            source: CaseSource::ReviewedV1,
            resolved_input_bytes: 0,
        })
    }

    fn parse_workflow(root: &Map<String, Value>) -> Result<Self, ScenarioFailure> {
        ensure_fields(
            root,
            &["schema_version", "target_env", "steps"],
            &["schema_version", "steps"],
            scenario_location(),
        )?;
        let target_env = parse_target_environment(root)?;
        let steps_value = root.get("steps").expect("required fields were checked");
        let steps = steps_value.as_array().ok_or_else(|| {
            shape_failure(
                scenario_location().field(LocationField::Steps),
                ExpectedShape::Array,
                json_kind(Some(steps_value)),
            )
        })?;
        let maximum_steps = DiagnosticLimits::M1_DEFAULTS.values().active_cases;
        let maximum_items = usize::try_from(maximum_steps).unwrap_or(usize::MAX);
        if target_env.len() > maximum_items
            || target_env
                .iter()
                .any(|name| name.chars().count() > MAX_WORKFLOW_NAME_CHARS)
        {
            return Err(ScenarioFailure::one(Finding::scenario_invalid(
                scenario_location().field(LocationField::TargetEnv),
                RuleViolation::InvalidScenarioShape,
            )));
        }
        if steps.is_empty() {
            return Err(ScenarioFailure::one(Finding::scenario_invalid(
                scenario_location().field(LocationField::Steps),
                RuleViolation::InvalidScenarioShape,
            )));
        }
        let observed_steps = u64::try_from(steps.len()).unwrap_or(u64::MAX);
        if observed_steps > maximum_steps {
            return Err(ScenarioFailure::one(Finding::limit_exceeded(
                SupportedRevision::CURRENT,
                scenario_location().field(LocationField::Steps),
                LimitViolation::new(LimitKind::ActiveCases, observed_steps, maximum_steps)
                    .expect("the workflow step count exceeds its checked maximum"),
            )));
        }

        let mut step_ids = BTreeSet::new();
        let mut tools = BTreeSet::new();
        let mut declared_captures = BTreeSet::new();
        let mut total_captures = 0_u64;
        let mut first_cleanup = None;
        let mut side_effecting = false;
        let mut cases = Vec::with_capacity(steps.len());
        for (index, value) in steps.iter().enumerate() {
            let base = workflow_step_location(index);
            let object = required_object(Some(value), base.clone())?;
            ensure_fields(
                object,
                &[
                    "id",
                    "tool",
                    "safety",
                    "cleanup",
                    "arguments",
                    "secret_refs",
                    "argument_refs",
                    "captures",
                    "expect",
                ],
                &["id", "tool", "safety", "arguments", "expect"],
                base.clone(),
            )?;
            let id =
                required_nonempty_string(object.get("id"), base.clone().field(LocationField::Id))?;
            if id.chars().count() > MAX_WORKFLOW_NAME_CHARS {
                return Err(ScenarioFailure::one(Finding::scenario_invalid(
                    base.clone().field(LocationField::Id),
                    RuleViolation::InvalidScenarioShape,
                )));
            }
            if !step_ids.insert(id.to_owned()) {
                return Err(ScenarioFailure::one(Finding::scenario_invalid(
                    base.clone().field(LocationField::Id),
                    RuleViolation::DuplicateCaseId,
                )));
            }
            let tool = required_nonempty_string(
                object.get("tool"),
                base.clone().field(LocationField::Tools),
            )?
            .to_owned();
            if tool.chars().count() > MAX_WORKFLOW_NAME_CHARS {
                return Err(ScenarioFailure::one(Finding::scenario_invalid(
                    base.clone().field(LocationField::Tools),
                    RuleViolation::InvalidScenarioShape,
                )));
            }
            tools.insert(tool.clone());

            let safety_location = base.clone().field(LocationField::Safety);
            let safety = required_object(object.get("safety"), safety_location.clone())?;
            ensure_fields(safety, &["effects"], &["effects"], safety_location.clone())?;
            let effects = match safety.get("effects").and_then(Value::as_str) {
                Some("read_only") => ScenarioEffects::ReadOnly,
                Some("side_effecting") => ScenarioEffects::SideEffecting,
                _ => {
                    return Err(ScenarioFailure::one(Finding::scenario_invalid(
                        safety_location.field(LocationField::Effects),
                        RuleViolation::InvalidScenarioShape,
                    )));
                }
            };
            side_effecting |= effects == ScenarioEffects::SideEffecting;

            let cleanup = match object.get("cleanup") {
                None | Some(Value::Bool(false)) => false,
                Some(Value::Bool(true)) => true,
                Some(value) => {
                    return Err(shape_failure(
                        base.clone().field(LocationField::Cleanup),
                        ExpectedShape::Boolean,
                        json_kind(Some(value)),
                    ));
                }
            };
            if cleanup {
                first_cleanup.get_or_insert(index);
            } else if first_cleanup.is_some() {
                return Err(ScenarioFailure::one(Finding::scenario_invalid(
                    base.clone().field(LocationField::Cleanup),
                    RuleViolation::InvalidScenarioShape,
                )));
            }

            let arguments = object
                .get("arguments")
                .expect("required fields were checked");
            if !arguments.is_object() {
                return Err(shape_failure(
                    base.clone().field(LocationField::Arguments),
                    ExpectedShape::Object,
                    json_kind(Some(arguments)),
                ));
            }
            check_instance_bytes(arguments, base.clone().field(LocationField::Arguments))?;

            let mut destination_pointers: Vec<String> = Vec::new();
            let mut secret_refs = BTreeMap::new();
            if let Some(value) = object.get("secret_refs") {
                let references = value.as_object().ok_or_else(|| {
                    shape_failure(
                        base.clone().field(LocationField::SecretRefs),
                        ExpectedShape::Object,
                        json_kind(Some(value)),
                    )
                })?;
                if references.len() > maximum_items {
                    return Err(ScenarioFailure::one(Finding::scenario_invalid(
                        base.clone().field(LocationField::SecretRefs),
                        RuleViolation::InvalidScenarioShape,
                    )));
                }
                for (pointer, source) in references {
                    let location = base.clone().field(LocationField::SecretRefs).wildcard();
                    let Some(source) = source.as_str() else {
                        return Err(shape_failure(
                            location,
                            ExpectedShape::String,
                            json_kind(Some(source)),
                        ));
                    };
                    if !valid_workflow_pointer(pointer)
                        || !valid_workflow_name(source)
                        || arguments.pointer(pointer) != Some(&Value::Null)
                        || destination_pointers
                            .iter()
                            .any(|existing| pointers_overlap(existing, pointer))
                    {
                        return Err(ScenarioFailure::one(Finding::secret_reference_invalid(
                            location,
                            RuleViolation::InvalidEnvironmentReference,
                        )));
                    }
                    destination_pointers.push(pointer.clone());
                    secret_refs.insert(pointer.clone(), source.to_owned());
                }
            }

            let mut argument_refs = BTreeMap::new();
            if let Some(value) = object.get("argument_refs") {
                let references = value.as_object().ok_or_else(|| {
                    shape_failure(
                        base.clone().field(LocationField::ArgumentRefs),
                        ExpectedShape::Object,
                        json_kind(Some(value)),
                    )
                })?;
                if references.len() > maximum_items {
                    return Err(ScenarioFailure::one(Finding::scenario_invalid(
                        base.clone().field(LocationField::ArgumentRefs),
                        RuleViolation::InvalidScenarioShape,
                    )));
                }
                for (pointer, capture) in references {
                    let location = base.clone().field(LocationField::ArgumentRefs).wildcard();
                    let Some(capture) = capture.as_str() else {
                        return Err(shape_failure(
                            location,
                            ExpectedShape::String,
                            json_kind(Some(capture)),
                        ));
                    };
                    if !valid_workflow_pointer(pointer)
                        || !valid_workflow_name(capture)
                        || !declared_captures.contains(capture)
                        || arguments.pointer(pointer) != Some(&Value::Null)
                        || destination_pointers
                            .iter()
                            .any(|existing| pointers_overlap(existing, pointer))
                    {
                        return Err(ScenarioFailure::one(Finding::scenario_invalid(
                            location,
                            RuleViolation::InvalidScenarioShape,
                        )));
                    }
                    destination_pointers.push(pointer.clone());
                    argument_refs.insert(pointer.clone(), capture.to_owned());
                }
            }

            let mut captures = BTreeMap::new();
            if let Some(value) = object.get("captures") {
                let declared = value.as_object().ok_or_else(|| {
                    shape_failure(
                        base.clone().field(LocationField::Captures),
                        ExpectedShape::Object,
                        json_kind(Some(value)),
                    )
                })?;
                if declared.len() > maximum_items {
                    return Err(ScenarioFailure::one(Finding::scenario_invalid(
                        base.clone().field(LocationField::Captures),
                        RuleViolation::InvalidScenarioShape,
                    )));
                }
                for (name, pointer) in declared {
                    let location = base.clone().field(LocationField::Captures).wildcard();
                    let Some(pointer) = pointer.as_str() else {
                        return Err(shape_failure(
                            location,
                            ExpectedShape::String,
                            json_kind(Some(pointer)),
                        ));
                    };
                    if cleanup
                        || !valid_workflow_name(name)
                        || !valid_workflow_pointer(pointer)
                        || !declared_captures.insert(name.clone())
                    {
                        return Err(ScenarioFailure::one(Finding::scenario_invalid(
                            location,
                            RuleViolation::InvalidScenarioShape,
                        )));
                    }
                    total_captures = total_captures.saturating_add(1);
                    if total_captures > maximum_steps {
                        return Err(ScenarioFailure::one(Finding::limit_exceeded(
                            SupportedRevision::CURRENT,
                            scenario_location().field(LocationField::Steps),
                            LimitViolation::new(
                                LimitKind::ActiveCases,
                                total_captures,
                                maximum_steps,
                            )
                            .expect("the workflow capture count exceeds its checked maximum"),
                        )));
                    }
                    captures.insert(name.clone(), pointer.to_owned());
                }
            }

            let expect_location = base.clone().field(LocationField::Expect);
            let expect = required_object(object.get("expect"), expect_location.clone())?;
            ensure_fields(
                expect,
                &["result", "structured_output_schema"],
                &["result"],
                expect_location.clone(),
            )?;
            let expected = match expect.get("result").and_then(Value::as_str) {
                Some("success") => ExpectedResult::Success,
                Some("tool_error") if !cleanup && captures.is_empty() => ExpectedResult::ToolError,
                _ => {
                    return Err(ScenarioFailure::one(Finding::scenario_invalid(
                        expect_location.clone().field(LocationField::Result),
                        RuleViolation::InvalidScenarioShape,
                    )));
                }
            };
            let output_validator = if let Some(schema) = expect.get("structured_output_schema") {
                let location = expect_location.field(LocationField::StructuredOutputSchema);
                if !schema.is_object() {
                    return Err(shape_failure(
                        location,
                        ExpectedShape::Object,
                        json_kind(Some(schema)),
                    ));
                }
                let findings = scenario_schema_findings(validate_local_schema(schema, location));
                if !findings.is_empty() {
                    return Err(ScenarioFailure { findings });
                }
                Some(
                    LocalValidator::compile(schema)
                        .expect("a validated workflow schema compiles without retrieval"),
                )
            } else {
                None
            };

            cases.push(ScenarioCase {
                tool,
                arguments: arguments.clone(),
                omit_arguments: false,
                applicable: true,
                secret_refs,
                argument_refs,
                captures,
                cleanup,
                expected,
                output_validator,
                reproduction: None,
            });
        }
        let first_cleanup = first_cleanup.unwrap_or(cases.len());
        if first_cleanup == 0 {
            return Err(ScenarioFailure::one(Finding::scenario_invalid(
                scenario_location().field(LocationField::Steps),
                RuleViolation::InvalidScenarioShape,
            )));
        }

        Ok(Self {
            tools,
            side_effecting,
            target_env,
            cases,
            source: CaseSource::Workflow { first_cleanup },
            resolved_input_bytes: 0,
        })
    }

    pub(crate) fn generated(
        tool: String,
        side_effecting: bool,
        requested_cases: usize,
        seed: u64,
    ) -> Result<Self, ScenarioFailure> {
        if tool.is_empty() || requested_cases == 0 {
            return Err(ScenarioFailure::one(Finding::case_generation_failed(
                generation_location(),
                RuleViolation::InvalidGenerationConfiguration,
            )));
        }
        let observed = u64::try_from(requested_cases).unwrap_or(u64::MAX);
        let maximum = DiagnosticLimits::M1_DEFAULTS.values().active_cases;
        if observed > maximum {
            return Err(ScenarioFailure::one(Finding::limit_exceeded(
                SupportedRevision::CURRENT,
                generation_location().field(LocationField::Cases),
                LimitViolation::new(LimitKind::ActiveCases, observed, maximum)
                    .expect("the generated case count exceeds its checked maximum"),
            )));
        }
        Ok(Self {
            tools: BTreeSet::from([tool]),
            side_effecting,
            target_env: Vec::new(),
            cases: Vec::new(),
            source: CaseSource::Generated {
                seed,
                requested_cases,
            },
            resolved_input_bytes: 0,
        })
    }

    pub(crate) fn rejection(
        tool: String,
        side_effecting: bool,
        seed: u64,
    ) -> Result<Self, ScenarioFailure> {
        if tool.is_empty() {
            return Err(ScenarioFailure::one(Finding::case_generation_failed(
                generation_location(),
                RuleViolation::InvalidGenerationConfiguration,
            )));
        }
        Ok(Self {
            tools: BTreeSet::from([tool]),
            side_effecting,
            target_env: Vec::new(),
            cases: Vec::new(),
            source: CaseSource::Rejection { seed },
            resolved_input_bytes: 0,
        })
    }

    pub(crate) fn authorize(
        &self,
        allowed_tool: &str,
        allow_side_effects: bool,
    ) -> Result<(), ScenarioFailure> {
        self.authorize_tools(std::iter::once(allowed_tool), allow_side_effects)
    }

    pub(crate) fn authorize_tools<'a>(
        &self,
        allowed_tools: impl IntoIterator<Item = &'a str>,
        allow_side_effects: bool,
    ) -> Result<(), ScenarioFailure> {
        let mut authorized = BTreeSet::new();
        let mut observed = 0_usize;
        for tool in allowed_tools {
            observed = observed.saturating_add(1);
            authorized.insert(tool);
        }
        if observed != authorized.len()
            || authorized.len() != self.tools.len()
            || !self
                .tools
                .iter()
                .all(|tool| authorized.contains(tool.as_str()))
        {
            return Err(ScenarioFailure::one(Finding::tool_authorization_missing(
                Location::root(LocationField::Authorization).field(LocationField::Tools),
            )));
        }
        if self.side_effecting && !allow_side_effects {
            return Err(ScenarioFailure::one(Finding::side_effects_not_authorized(
                Location::root(LocationField::Authorization)
                    .field(LocationField::Safety)
                    .field(LocationField::Effects),
            )));
        }
        Ok(())
    }

    pub(crate) fn validate_revision(
        &self,
        revision: ActiveProtocolRevision,
    ) -> Result<(), ScenarioFailure> {
        if matches!(self.source, CaseSource::Workflow { .. })
            && revision.as_supported() != SupportedRevision::CURRENT
        {
            return Err(ScenarioFailure::one(Finding::scenario_invalid(
                scenario_location().field(LocationField::SchemaVersion),
                RuleViolation::UnsupportedScenarioRevision,
            )));
        }
        Ok(())
    }

    pub(crate) fn target_environment_names(&self) -> impl Iterator<Item = &str> {
        self.target_env.iter().map(String::as_str)
    }

    pub(crate) fn resolve_argument_secrets<F>(
        &mut self,
        mut lookup: F,
    ) -> Result<(), ScenarioFailure>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let mut aggregate_bytes = 0_u64;
        let workflow = self.is_workflow();
        let maximum_aggregate = DiagnosticLimits::M1_DEFAULTS
            .values()
            .aggregate_output_bytes;
        for (case_index, case) in self.cases.iter_mut().enumerate() {
            let case_location = if workflow {
                workflow_step_location(case_index)
            } else {
                case_location(case_index)
            };
            let mut replacements = Vec::with_capacity(case.secret_refs.len());
            for (pointer, source) in &case.secret_refs {
                let Some(value) = lookup(source) else {
                    return Err(ScenarioFailure::one(Finding::secret_reference_invalid(
                        case_location
                            .clone()
                            .field(LocationField::SecretRefs)
                            .wildcard(),
                        RuleViolation::MissingEnvironmentValue,
                    )));
                };
                replacements.push((pointer.clone(), value));
            }
            for (pointer, value) in replacements {
                let destination = case
                    .arguments
                    .pointer_mut(&pointer)
                    .expect("validated secret pointers remain present before replacement");
                debug_assert!(destination.is_null());
                *destination = Value::String(value);
            }
            case.secret_refs.clear();
            check_instance_bytes(
                &case.arguments,
                case_location.clone().field(LocationField::Arguments),
            )?;
            aggregate_bytes = aggregate_bytes
                .saturating_add(u64::try_from(serialized_len(&case.arguments)).unwrap_or(u64::MAX));
            if aggregate_bytes > maximum_aggregate {
                return Err(ScenarioFailure::one(Finding::limit_exceeded(
                    SupportedRevision::CURRENT,
                    scenario_location().field(if workflow {
                        LocationField::Steps
                    } else {
                        LocationField::Cases
                    }),
                    LimitViolation::new(
                        LimitKind::ActiveInputBytes,
                        aggregate_bytes,
                        maximum_aggregate,
                    )
                    .expect("resolved active inputs exceed their aggregate maximum"),
                )));
            }
        }
        self.resolved_input_bytes = aggregate_bytes;
        Ok(())
    }

    pub(crate) fn discard_target_environment_names(&mut self) {
        self.target_env.clear();
    }

    pub(crate) fn reject_remote_target_environment(&self) -> Result<(), ScenarioFailure> {
        if self.target_env.is_empty() {
            Ok(())
        } else {
            Err(ScenarioFailure::one(Finding::secret_reference_invalid(
                scenario_location()
                    .field(LocationField::TargetEnv)
                    .wildcard(),
                RuleViolation::InvalidEnvironmentReference,
            )))
        }
    }

    pub(crate) fn case_count(&self) -> usize {
        match self.source {
            CaseSource::ReviewedV1 | CaseSource::Workflow { .. } => self.cases.len(),
            CaseSource::Generated {
                requested_cases, ..
            } => requested_cases,
            CaseSource::Rejection { .. } => REJECTION_CASE_COUNT,
        }
    }

    fn configuration_check(&self) -> CheckId {
        match self.source {
            CaseSource::ReviewedV1 | CaseSource::Workflow { .. } => CheckId::ScenarioConfiguration,
            CaseSource::Generated { .. } | CaseSource::Rejection { .. } => {
                CheckId::GenerationConfiguration
            }
        }
    }

    fn generates_cases(&self) -> bool {
        matches!(
            self.source,
            CaseSource::Generated { .. } | CaseSource::Rejection { .. }
        )
    }

    fn expects_invalid_arguments_rejection(&self) -> bool {
        matches!(self.source, CaseSource::Rejection { .. })
    }

    fn case_requirement(&self) -> Requirement {
        if self.expects_invalid_arguments_rejection() {
            Requirement::Optional
        } else {
            Requirement::Required
        }
    }

    fn is_workflow(&self) -> bool {
        matches!(self.source, CaseSource::Workflow { .. })
    }

    fn first_cleanup(&self) -> usize {
        match self.source {
            CaseSource::Workflow { first_cleanup } => first_cleanup,
            _ => self.cases.len(),
        }
    }

    fn case_is_cleanup(&self, index: usize) -> bool {
        self.cases.get(index).is_some_and(|case| case.cleanup)
    }

    fn case_check_id(&self, index: usize) -> CheckId {
        match self.source {
            CaseSource::Workflow { .. } if self.case_is_cleanup(index) => {
                CheckId::RuntimeWorkflowCleanup(index)
            }
            CaseSource::Workflow { .. } => CheckId::RuntimeWorkflowStep(index),
            _ => CheckId::RuntimeToolCase(index),
        }
    }

    fn case_location(&self, index: usize) -> Location {
        match self.source {
            CaseSource::ReviewedV1 => case_location(index),
            CaseSource::Workflow { .. } => workflow_step_location(index),
            CaseSource::Generated { .. } | CaseSource::Rejection { .. } => generation_location()
                .field(LocationField::Cases)
                .index(index),
        }
    }

    fn generate_cases(
        &mut self,
        schema: &Value,
        validator: &LocalValidator,
    ) -> Result<(), GenerationFailure> {
        let tool = self
            .tools
            .iter()
            .next()
            .expect("generated scenarios declare one tool")
            .clone();
        match self.source {
            CaseSource::ReviewedV1 | CaseSource::Workflow { .. } => {}
            CaseSource::Generated {
                seed,
                requested_cases,
            } => {
                let generated = generate_inputs(schema, validator, seed, requested_cases)?;
                self.cases = generated
                    .into_iter()
                    .map(|generated| ScenarioCase {
                        tool: tool.clone(),
                        arguments: generated.arguments,
                        omit_arguments: false,
                        applicable: true,
                        secret_refs: BTreeMap::new(),
                        argument_refs: BTreeMap::new(),
                        captures: BTreeMap::new(),
                        cleanup: false,
                        expected: ExpectedResult::Success,
                        output_validator: None,
                        reproduction: Some(generated.reproduction),
                    })
                    .collect();
                debug_assert_eq!(self.cases.len(), requested_cases);
            }
            CaseSource::Rejection { seed } => {
                let generated = generate_invalid_inputs(schema, validator, seed)?;
                self.cases = generated
                    .into_iter()
                    .map(|generated| match generated {
                        Some(generated) => ScenarioCase {
                            tool: tool.clone(),
                            arguments: generated.arguments,
                            omit_arguments: generated.omit_arguments,
                            applicable: true,
                            secret_refs: BTreeMap::new(),
                            argument_refs: BTreeMap::new(),
                            captures: BTreeMap::new(),
                            cleanup: false,
                            expected: ExpectedResult::InvalidArgumentsRejection,
                            output_validator: None,
                            reproduction: Some(generated.reproduction),
                        },
                        None => ScenarioCase {
                            tool: tool.clone(),
                            arguments: Value::Null,
                            omit_arguments: false,
                            applicable: false,
                            secret_refs: BTreeMap::new(),
                            argument_refs: BTreeMap::new(),
                            captures: BTreeMap::new(),
                            cleanup: false,
                            expected: ExpectedResult::InvalidArgumentsRejection,
                            output_validator: None,
                            reproduction: None,
                        },
                    })
                    .collect();
                debug_assert_eq!(self.cases.len(), REJECTION_CASE_COUNT);
            }
        }
        Ok(())
    }
}

pub(crate) fn render_scenario_failure_for_revision(
    failure: ScenarioFailure,
    transport: ReportTransport,
    revision: ActiveProtocolRevision,
) -> Diagnostic {
    render_prestart_failure(
        failure,
        CheckId::ScenarioConfiguration,
        None,
        false,
        false,
        transport,
        revision.as_supported(),
    )
}

pub(crate) fn render_resolved_scenario_failure_for_revision(
    scenario: &ActiveScenario,
    failure: ScenarioFailure,
    transport: ReportTransport,
    revision: ActiveProtocolRevision,
) -> Diagnostic {
    render_prestart_failure(
        failure,
        scenario.configuration_check(),
        Some(
            (0..scenario.case_count())
                .map(|index| scenario.case_check_id(index))
                .collect(),
        ),
        true,
        scenario.generates_cases(),
        transport,
        revision.as_supported(),
    )
}

pub(crate) fn render_generation_configuration_failure_for_revision(
    failure: ScenarioFailure,
    requested_cases: usize,
    transport: ReportTransport,
    revision: ActiveProtocolRevision,
) -> Diagnostic {
    let maximum =
        usize::try_from(DiagnosticLimits::M1_DEFAULTS.values().active_cases).unwrap_or(usize::MAX);
    let case_checks = (requested_cases > 0 && requested_cases <= maximum)
        .then(|| (0..requested_cases).map(CheckId::RuntimeToolCase).collect());
    render_prestart_failure(
        failure,
        CheckId::GenerationConfiguration,
        case_checks,
        false,
        true,
        transport,
        revision.as_supported(),
    )
}

fn render_prestart_failure(
    failure: ScenarioFailure,
    configuration_check: CheckId,
    case_checks: Option<Vec<CheckId>>,
    authorization_passed: bool,
    generated: bool,
    transport: ReportTransport,
    revision: SupportedRevision,
) -> Diagnostic {
    let mut checks = vec![CheckResult::performed(
        configuration_check,
        Requirement::Required,
        failure
            .findings
            .into_iter()
            .map(|finding| finding.with_revision(revision))
            .collect(),
    )];
    checks.push(if authorization_passed {
        CheckResult::performed(
            CheckId::ActiveAuthorization,
            Requirement::Required,
            Vec::new(),
        )
    } else {
        CheckResult::skipped(
            CheckId::ActiveAuthorization,
            Requirement::Required,
            SkipReason::PrerequisiteFailed,
        )
    });
    checks.extend(prestart_transport_checks(
        transport,
        SkipReason::PrerequisiteFailed,
    ));
    checks.extend([
        CheckResult::skipped(
            CheckId::ProtocolEnvelope,
            Requirement::Required,
            SkipReason::PrerequisiteFailed,
        ),
        CheckResult::skipped(
            CheckId::ProtocolRevision,
            Requirement::Required,
            SkipReason::PrerequisiteFailed,
        ),
        CheckResult::skipped(
            CheckId::DiscoveryCatalogs,
            Requirement::Required,
            SkipReason::PrerequisiteFailed,
        ),
        CheckResult::skipped(
            CheckId::SchemaContracts,
            Requirement::Required,
            SkipReason::PrerequisiteFailed,
        ),
    ]);
    if generated {
        checks.push(CheckResult::skipped(
            CheckId::CaseGeneration,
            Requirement::Required,
            SkipReason::PrerequisiteFailed,
        ));
    }
    if let Some(case_checks) = case_checks {
        checks.extend(case_checks.into_iter().map(|id| {
            CheckResult::skipped(id, Requirement::Required, SkipReason::PrerequisiteFailed)
        }));
    } else {
        checks.push(CheckResult::skipped(
            CheckId::RuntimeTools,
            Requirement::Required,
            SkipReason::PrerequisiteFailed,
        ));
    }
    let report = DiagnosticReport::new(revision, DiagnosticLimits::M1_DEFAULTS, checks)
        .expect("a scenario configuration failure is a valid report")
        .with_exit_status(ExitStatus::InvocationError);
    Diagnostic::from_report(report)
}

pub(crate) fn render_authorization_failure_for_revision(
    scenario: &ActiveScenario,
    failure: ScenarioFailure,
    transport: ReportTransport,
    revision: ActiveProtocolRevision,
) -> Diagnostic {
    let revision = revision.as_supported();
    let mut checks = vec![
        CheckResult::performed(
            scenario.configuration_check(),
            Requirement::Required,
            Vec::new(),
        ),
        CheckResult::performed(
            CheckId::ActiveAuthorization,
            Requirement::Required,
            failure
                .findings
                .into_iter()
                .map(|finding| finding.with_revision(revision))
                .collect(),
        ),
    ];
    checks.extend(prestart_transport_checks(
        transport,
        SkipReason::AuthorizationFailed,
    ));
    checks.extend([
        CheckResult::skipped(
            CheckId::ProtocolEnvelope,
            Requirement::Required,
            SkipReason::AuthorizationFailed,
        ),
        CheckResult::skipped(
            CheckId::ProtocolRevision,
            Requirement::Required,
            SkipReason::AuthorizationFailed,
        ),
        CheckResult::skipped(
            CheckId::DiscoveryCatalogs,
            Requirement::Required,
            SkipReason::AuthorizationFailed,
        ),
        CheckResult::skipped(
            CheckId::SchemaContracts,
            Requirement::Required,
            SkipReason::AuthorizationFailed,
        ),
    ]);
    if scenario.generates_cases() {
        checks.push(CheckResult::skipped(
            CheckId::CaseGeneration,
            Requirement::Required,
            SkipReason::AuthorizationFailed,
        ));
    }
    checks.extend((0..scenario.case_count()).map(|index| {
        CheckResult::skipped(
            scenario.case_check_id(index),
            Requirement::Required,
            SkipReason::AuthorizationFailed,
        )
    }));
    let report = DiagnosticReport::new(revision, DiagnosticLimits::M1_DEFAULTS, checks)
        .expect("an active authorization failure is a valid report")
        .with_exit_status(ExitStatus::InvocationError);
    Diagnostic::from_report(report)
}

fn prestart_transport_checks(transport: ReportTransport, reason: SkipReason) -> Vec<CheckResult> {
    match transport {
        ReportTransport::Stdio => vec![CheckResult::skipped(
            CheckId::TransportStdio,
            Requirement::Required,
            reason,
        )],
        ReportTransport::Http => vec![
            CheckResult::skipped(CheckId::NetworkTarget, Requirement::Required, reason),
            CheckResult::skipped(CheckId::NetworkResolution, Requirement::Required, reason),
            CheckResult::skipped(CheckId::TransportTls, Requirement::Required, reason),
            CheckResult::skipped(CheckId::TransportHttp, Requirement::Required, reason),
        ],
    }
}

pub(crate) struct ActiveConversation {
    adapter: ActiveProtocolAdapter,
    scenario: ActiveScenario,
    stage: Stage,
    pending: Option<PendingRequest>,
    next_id: i64,
    envelope: PhaseState,
    revision: PhaseState,
    discovery: PhaseState,
    schemas: PhaseState,
    generation: Option<PhaseState>,
    case_states: Vec<CaseState>,
    seen_names: BTreeSet<String>,
    seen_cursors: BTreeSet<String>,
    observed_items: u64,
    selected_names: BTreeSet<String>,
    selected_tools: BTreeMap<String, ToolContract>,
    tool_validators: BTreeMap<String, ToolValidators>,
    selected_schema_findings: Vec<Finding>,
    captured_values: BTreeMap<String, Value>,
    captured_bytes: u64,
    next_case: usize,
    validate_http_headers: bool,
    negotiated_revision: Option<KnownRevision>,
}

enum Stage {
    Start,
    Initialized { list_tools: bool },
    Tools(Option<String>),
    Cases,
    Done,
}

#[derive(Clone, Copy)]
enum PendingRequest {
    Start,
    Tools,
    Call(usize),
}

enum PhaseState {
    Pending,
    Performed(Vec<Finding>),
    Skipped(SkipReason),
}

enum CaseState {
    Pending,
    Performed(Vec<Finding>),
    Incomplete,
    Skipped(SkipReason),
}

struct ActiveFindingCollector {
    findings: BTreeSet<Finding>,
    overflow: bool,
}

impl ActiveFindingCollector {
    fn new() -> Self {
        Self {
            findings: BTreeSet::new(),
            overflow: false,
        }
    }

    fn push(&mut self, finding: Finding) {
        if self.findings.contains(&finding) {
            return;
        }
        if self.findings.len() >= report_finding_capacity() {
            self.overflow = true;
            return;
        }
        self.findings.insert(finding);
    }

    fn is_empty(&self) -> bool {
        self.findings.is_empty()
    }

    fn finish(self, location: Location, revision: SupportedRevision) -> Vec<Finding> {
        cap_active_phase_findings(
            self.findings.into_iter().collect(),
            self.overflow,
            location,
            revision,
        )
    }
}

struct ToolContract {
    input_schema: Value,
    output_schema: Option<Value>,
    header_annotations: Vec<HeaderAnnotation>,
}

struct ToolValidators {
    input: LocalValidator,
    output: Option<LocalValidator>,
    header_annotations: Vec<HeaderAnnotation>,
}

impl ActiveConversation {
    pub(crate) fn for_revision(scenario: ActiveScenario, revision: ActiveProtocolRevision) -> Self {
        let adapter = ActiveProtocolAdapter::new(revision);
        let generation = scenario.generates_cases().then_some(PhaseState::Pending);
        let case_states = (0..scenario.case_count())
            .map(|_| CaseState::Pending)
            .collect();
        Self {
            adapter,
            scenario,
            stage: Stage::Start,
            pending: None,
            next_id: 1,
            envelope: PhaseState::Pending,
            revision: PhaseState::Pending,
            discovery: PhaseState::Pending,
            schemas: PhaseState::Pending,
            generation,
            case_states,
            seen_names: BTreeSet::new(),
            seen_cursors: BTreeSet::new(),
            observed_items: 0,
            selected_names: BTreeSet::new(),
            selected_tools: BTreeMap::new(),
            tool_validators: BTreeMap::new(),
            selected_schema_findings: Vec::new(),
            captured_values: BTreeMap::new(),
            captured_bytes: 0,
            next_case: 0,
            validate_http_headers: false,
            negotiated_revision: None,
        }
    }

    pub(crate) fn new_http_for_revision(
        scenario: ActiveScenario,
        revision: ActiveProtocolRevision,
    ) -> Self {
        let mut conversation = Self::for_revision(scenario, revision);
        conversation.validate_http_headers = conversation.adapter.permits_http_mappings();
        conversation
    }

    fn revision(&self) -> SupportedRevision {
        self.adapter.revision()
    }

    fn begin_request(&mut self, pending: PendingRequest) -> i64 {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("bounded requests keep active request ids representable");
        self.pending = Some(pending);
        id
    }

    fn resolve_argument_refs(&mut self, index: usize) -> Result<(), ArgumentReferenceFailure> {
        if self.scenario.cases[index].argument_refs.is_empty() {
            return Ok(());
        }
        let replacements = self.scenario.cases[index]
            .argument_refs
            .iter()
            .map(|(pointer, capture)| {
                self.captured_values
                    .get(capture)
                    .map(|value| (pointer.clone(), capture.clone(), serialized_len(value)))
                    .ok_or(ArgumentReferenceFailure::Unavailable)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let previous = u64::try_from(serialized_len(&self.scenario.cases[index].arguments))
            .unwrap_or(u64::MAX);
        let observed = replacements.iter().fold(previous, |total, (_, _, bytes)| {
            total
                .saturating_sub(u64::try_from(serialized_len(&Value::Null)).unwrap_or(u64::MAX))
                .saturating_add(u64::try_from(*bytes).unwrap_or(u64::MAX))
        });
        let values = DiagnosticLimits::M1_DEFAULTS.values();
        if observed > values.instance_bytes {
            return Err(ArgumentReferenceFailure::Limit(
                LimitViolation::new(LimitKind::InstanceBytes, observed, values.instance_bytes)
                    .expect("resolved workflow arguments exceed their checked maximum"),
            ));
        }
        self.scenario.resolved_input_bytes = self
            .scenario
            .resolved_input_bytes
            .saturating_sub(previous)
            .saturating_add(observed);
        if self.scenario.resolved_input_bytes > values.aggregate_output_bytes {
            return Err(ArgumentReferenceFailure::Limit(
                LimitViolation::new(
                    LimitKind::ActiveInputBytes,
                    self.scenario.resolved_input_bytes,
                    values.aggregate_output_bytes,
                )
                .expect("resolved workflow inputs exceed their aggregate maximum"),
            ));
        }
        for (pointer, capture, _) in replacements {
            let value = self
                .captured_values
                .get(&capture)
                .expect("validated workflow captures remain available")
                .clone();
            let destination = self.scenario.cases[index]
                .arguments
                .pointer_mut(&pointer)
                .expect("validated workflow reference pointers remain present");
            debug_assert!(destination.is_null());
            *destination = value;
        }
        self.scenario.cases[index].argument_refs.clear();
        debug_assert_eq!(
            u64::try_from(serialized_len(&self.scenario.cases[index].arguments))
                .unwrap_or(u64::MAX),
            observed
        );
        Ok(())
    }

    fn capture_workflow_values(
        &mut self,
        index: usize,
        structured: Option<&Value>,
    ) -> Result<(), CaptureFailure> {
        if self.scenario.cases[index].captures.is_empty() {
            return Ok(());
        }
        let Some(structured) = structured else {
            return Err(CaptureFailure::Missing);
        };
        let captures = self.scenario.cases[index]
            .captures
            .iter()
            .map(|(name, pointer)| {
                structured
                    .pointer(pointer)
                    .map(|value| (name.clone(), value))
                    .ok_or(CaptureFailure::Missing)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let values = DiagnosticLimits::M1_DEFAULTS.values();
        let mut added_bytes = 0_u64;
        for (_, value) in &captures {
            let observed = u64::try_from(serialized_len(value)).unwrap_or(u64::MAX);
            if observed > values.instance_bytes {
                return Err(CaptureFailure::Limit(
                    LimitViolation::new(LimitKind::InstanceBytes, observed, values.instance_bytes)
                        .expect("a workflow capture exceeds its checked maximum"),
                ));
            }
            added_bytes = added_bytes.saturating_add(observed);
        }
        let aggregate = self.captured_bytes.saturating_add(added_bytes);
        if aggregate > values.aggregate_output_bytes {
            return Err(CaptureFailure::Limit(
                LimitViolation::new(
                    LimitKind::AggregateOutputBytes,
                    aggregate,
                    values.aggregate_output_bytes,
                )
                .expect("retained workflow captures exceed their aggregate maximum"),
            ));
        }
        for (name, value) in captures {
            self.captured_values.insert(name, value.clone());
        }
        self.captured_bytes = aggregate;
        self.scenario.cases[index].captures.clear();
        Ok(())
    }

    fn findings_failed(findings: &[Finding]) -> bool {
        findings
            .iter()
            .any(|finding| finding.severity().is_failure())
    }

    fn add_cleanup_failure(&self, index: usize, findings: &mut Vec<Finding>) {
        if self.scenario.case_is_cleanup(index)
            && !findings
                .iter()
                .any(|finding| finding.code() == FindingCode::WorkflowCleanupFailed)
        {
            findings.push(
                Finding::workflow_cleanup_failed(
                    self.scenario
                        .case_location(index)
                        .field(LocationField::Cleanup),
                )
                .with_revision(self.revision()),
            );
        }
    }

    fn stop_case(&mut self, index: usize, mut findings: Vec<Finding>, reason: SkipReason) {
        self.add_cleanup_failure(index, &mut findings);
        self.case_states[index] = CaseState::Performed(findings);
        self.next_case = index.saturating_add(1);
        self.stop_cases(reason);
    }

    fn complete_case(&mut self, index: usize, mut findings: Vec<Finding>) {
        let failed = Self::findings_failed(&findings);
        let cleanup = self.scenario.case_is_cleanup(index);
        if failed {
            self.add_cleanup_failure(index, &mut findings);
        }
        self.case_states[index] = CaseState::Performed(findings);
        self.next_case = index.saturating_add(1);
        if cleanup && failed {
            self.stop_cases(SkipReason::PrerequisiteFailed);
        } else if self.scenario.is_workflow() && failed {
            self.start_workflow_cleanup(SkipReason::PrerequisiteFailed);
        } else {
            self.stage = Stage::Cases;
        }
    }

    fn mark_incomplete(&mut self, index: usize, continue_v1: bool) {
        if self.scenario.case_is_cleanup(index) {
            self.complete_case(
                index,
                vec![
                    Finding::workflow_cleanup_failed(
                        self.scenario
                            .case_location(index)
                            .field(LocationField::Cleanup),
                    )
                    .with_revision(self.revision()),
                ],
            );
            return;
        }
        self.case_states[index] = CaseState::Incomplete;
        self.next_case = index.saturating_add(1);
        if self.scenario.is_workflow() {
            self.start_workflow_cleanup(SkipReason::InputRequired);
        } else if !continue_v1 {
            self.stop_cases(SkipReason::InputRequired);
        } else {
            self.stage = Stage::Cases;
        }
    }

    fn start_workflow_cleanup(&mut self, reason: SkipReason) {
        let first_cleanup = self.scenario.first_cleanup();
        for state in &mut self.case_states[self.next_case..first_cleanup] {
            if matches!(state, CaseState::Pending) {
                *state = CaseState::Skipped(reason);
            }
        }
        if first_cleanup < self.case_states.len() {
            self.next_case = first_cleanup;
            self.stage = Stage::Cases;
        } else {
            self.stage = Stage::Done;
        }
    }

    fn next_outbound(&mut self) -> Option<ProbeRequest> {
        loop {
            match &self.stage {
                Stage::Start => {
                    let id = self.begin_request(PendingRequest::Start);
                    return Some(self.adapter.start_request(id));
                }
                Stage::Initialized { list_tools } => {
                    self.stage = if *list_tools {
                        Stage::Tools(None)
                    } else {
                        Stage::Done
                    };
                    return self.adapter.initialized_notification();
                }
                Stage::Tools(cursor) => {
                    let cursor = cursor.clone();
                    let id = self.begin_request(PendingRequest::Tools);
                    return Some(self.adapter.tools_request(id, cursor.as_deref()));
                }
                Stage::Cases => {
                    if self.next_case >= self.scenario.cases.len() {
                        self.stage = Stage::Done;
                        continue;
                    }
                    let index = self.next_case;
                    if !self.scenario.cases[index].applicable {
                        self.case_states[index] = CaseState::Skipped(SkipReason::NotApplicable);
                        self.next_case += 1;
                        continue;
                    }
                    match self.resolve_argument_refs(index) {
                        Ok(()) => {}
                        Err(ArgumentReferenceFailure::Unavailable) => {
                            if self.scenario.case_is_cleanup(index) {
                                self.stop_case(
                                    index,
                                    vec![
                                        Finding::workflow_cleanup_failed(
                                            self.scenario
                                                .case_location(index)
                                                .field(LocationField::Cleanup),
                                        )
                                        .with_revision(self.revision()),
                                    ],
                                    SkipReason::PrerequisiteFailed,
                                );
                            } else {
                                self.case_states[index] =
                                    CaseState::Skipped(SkipReason::PrerequisiteFailed);
                                self.next_case = index.saturating_add(1);
                                self.stop_cases(SkipReason::PrerequisiteFailed);
                            }
                            continue;
                        }
                        Err(ArgumentReferenceFailure::Limit(violation)) => {
                            self.scenario.cases[index].arguments = Value::Null;
                            self.stop_case(
                                index,
                                vec![Finding::limit_exceeded(
                                    self.revision(),
                                    self.scenario
                                        .case_location(index)
                                        .field(LocationField::Arguments),
                                    violation,
                                )],
                                SkipReason::LimitReached,
                            );
                            continue;
                        }
                    }
                    let expects_rejection = self.scenario.cases[index].expected
                        == ExpectedResult::InvalidArgumentsRejection;
                    let validation = {
                        let validators = self
                            .tool_validators
                            .get(&self.scenario.cases[index].tool)
                            .expect("case replay starts only after selecting its tool contract");
                        validators
                            .input
                            .validate(&self.scenario.cases[index].arguments)
                    };
                    match (expects_rejection, validation) {
                        (false, Ok(()))
                        | (true, Err(InstanceValidationIssue::Mismatch { error_count: 1 })) => {}
                        (false, Err(InstanceValidationIssue::Mismatch { error_count })) => {
                            self.scenario.cases[index].arguments = Value::Null;
                            self.complete_case(
                                index,
                                vec![
                                    Finding::tool_arguments_mismatch(
                                        self.scenario
                                            .case_location(index)
                                            .field(LocationField::Arguments),
                                        error_count,
                                    )
                                    .with_revision(self.revision()),
                                ],
                            );
                            continue;
                        }
                        (true, Ok(())) | (true, Err(InstanceValidationIssue::Mismatch { .. })) => {
                            self.scenario.cases[index].arguments = Value::Null;
                            self.case_states[index] = CaseState::Performed(vec![
                                Finding::case_generation_failed(
                                    self.scenario.case_location(index),
                                    RuleViolation::NoValidBoundaryInput,
                                )
                                .with_revision(self.revision()),
                            ]);
                            self.next_case += 1;
                            self.stop_cases(SkipReason::PrerequisiteFailed);
                            continue;
                        }
                        (_, Err(InstanceValidationIssue::Limit(violation))) => {
                            self.scenario.cases[index].arguments = Value::Null;
                            self.stop_case(
                                index,
                                vec![Finding::limit_exceeded(
                                    self.revision(),
                                    self.scenario
                                        .case_location(index)
                                        .field(LocationField::Arguments),
                                    violation,
                                )],
                                SkipReason::LimitReached,
                            );
                            continue;
                        }
                        (_, Err(InstanceValidationIssue::InvalidSchema)) => {
                            self.scenario.cases[index].arguments = Value::Null;
                            self.stop_case(
                                index,
                                vec![
                                    Finding::tool_result_invalid(
                                        self.scenario
                                            .case_location(index)
                                            .field(LocationField::InputSchema),
                                    )
                                    .with_revision(self.revision()),
                                ],
                                SkipReason::PrerequisiteFailed,
                            );
                            continue;
                        }
                    }
                    let mirrored_fields = {
                        let validators = self
                            .tool_validators
                            .get(&self.scenario.cases[index].tool)
                            .expect("case replay starts only after selecting its tool contract");
                        validators
                            .header_annotations
                            .iter()
                            .map(|annotation| {
                                annotation.extract(&self.scenario.cases[index].arguments)
                            })
                            .collect::<Result<Vec<_>, ()>>()
                            .map(|fields| fields.into_iter().flatten().collect::<Vec<_>>())
                    };
                    let mirrored_fields = match mirrored_fields {
                        Ok(fields) => fields,
                        Err(()) if expects_rejection => {
                            self.scenario.cases[index].arguments = Value::Null;
                            self.case_states[index] = CaseState::Skipped(SkipReason::NotApplicable);
                            self.next_case += 1;
                            continue;
                        }
                        Err(()) => {
                            self.scenario.cases[index].arguments = Value::Null;
                            self.complete_case(
                                index,
                                vec![Finding::http_header_mapping_invalid(
                                    self.scenario
                                        .case_location(index)
                                        .field(LocationField::Arguments),
                                    RuleViolation::InvalidMirroredHeaderValue,
                                )],
                            );
                            continue;
                        }
                    };
                    let arguments = std::mem::take(&mut self.scenario.cases[index].arguments);
                    let arguments =
                        (!self.scenario.cases[index].omit_arguments).then_some(arguments);
                    let tool = self.scenario.cases[index].tool.clone();
                    let id = self.begin_request(PendingRequest::Call(index));
                    return Some(self.adapter.tool_call_request(
                        id,
                        tool,
                        arguments,
                        mirrored_fields,
                    ));
                }
                Stage::Done => return None,
            }
        }
    }

    fn process_response(&mut self, response: &ProbeResponse) {
        let pending = self
            .pending
            .take()
            .expect("every accepted response matches one active request");
        match pending {
            PendingRequest::Start => self.process_start(response),
            PendingRequest::Tools => self.process_tools(response),
            PendingRequest::Call(index) => self.process_call(index, response),
        }
    }

    fn process_start(&mut self, response: &ProbeResponse) {
        match self.adapter.start_kind() {
            ActiveStartKind::Discover => self.process_discovery(response),
            ActiveStartKind::Initialize => self.process_initialize(response),
        }
    }

    fn process_initialize(&mut self, response: &ProbeResponse) {
        let value: Value = serde_json::from_slice(response.as_bytes())
            .expect("the transport accepted this JSON response");
        let object = value
            .as_object()
            .expect("the transport accepted only JSON-RPC objects");
        let revision = self.revision();
        let server_location = Location::root(LocationField::Server);
        if object.contains_key("error") {
            self.envelope = PhaseState::Performed(vec![Finding::catalog_contract_invalid(
                revision,
                server_location,
                RuleViolation::ServerErrorResponse,
            )]);
            self.stop_before_cases(SkipReason::PrerequisiteFailed);
            return;
        }
        let Some(result) = object.get("result").and_then(Value::as_object) else {
            self.envelope = PhaseState::Performed(vec![Finding::catalog_contract_invalid(
                revision,
                server_location.field(LocationField::Result),
                RuleViolation::ExpectedShape {
                    expected: ExpectedShape::Object,
                    observed: json_kind(object.get("result")),
                },
            )]);
            self.stop_before_cases(SkipReason::PrerequisiteFailed);
            return;
        };

        let result_location = server_location.clone().field(LocationField::Result);
        let revision_location = result_location
            .clone()
            .field(LocationField::NegotiatedProtocolVersion);
        let negotiated = result.get("protocolVersion").and_then(Value::as_str);
        self.negotiated_revision = negotiated.and_then(KnownRevision::parse);
        let revision_valid = match negotiated {
            Some(value) if value == revision.as_str() => {
                self.revision = PhaseState::Performed(vec![Finding::revision_confirmed(
                    revision,
                    revision_location,
                )]);
                true
            }
            Some(_) => {
                self.revision = PhaseState::Performed(vec![Finding::revision_mismatch(
                    revision,
                    revision_location,
                )]);
                false
            }
            None => {
                self.revision = PhaseState::Performed(vec![Finding::invalid_revision_value(
                    revision,
                    revision_location,
                    RedactedValue::new(
                        result
                            .get("protocolVersion")
                            .map(serialized_len)
                            .unwrap_or_default(),
                    ),
                )]);
                false
            }
        };

        let (mut envelope_findings, tools_advertised) =
            validate_legacy_capabilities(result, result_location.clone(), revision);
        let server_info_location = result_location.clone().field(LocationField::ServerInfo);
        match result.get("serverInfo").and_then(Value::as_object) {
            Some(server_info) => {
                for (name, field) in [
                    ("name", LocationField::Name),
                    ("version", LocationField::Version),
                ] {
                    if !server_info.get(name).is_some_and(Value::is_string) {
                        envelope_findings.push(Finding::catalog_contract_invalid(
                            revision,
                            server_info_location.clone().field(field),
                            RuleViolation::ExpectedShape {
                                expected: ExpectedShape::String,
                                observed: json_kind(server_info.get(name)),
                            },
                        ));
                    }
                }
            }
            None => envelope_findings.push(Finding::catalog_contract_invalid(
                revision,
                server_info_location,
                RuleViolation::ExpectedShape {
                    expected: ExpectedShape::Object,
                    observed: json_kind(result.get("serverInfo")),
                },
            )),
        }
        if let Some(instructions) = result.get("instructions")
            && !instructions.is_string()
        {
            envelope_findings.push(Finding::catalog_contract_invalid(
                revision,
                result_location.field(LocationField::Instructions),
                RuleViolation::ExpectedShape {
                    expected: ExpectedShape::String,
                    observed: json_kind(Some(instructions)),
                },
            ));
        }
        self.envelope = PhaseState::Performed(cap_active_phase_findings(
            envelope_findings,
            false,
            server_location,
            revision,
        ));
        if !revision_valid {
            self.discovery = PhaseState::Skipped(SkipReason::UnsupportedRevision);
            self.schemas = PhaseState::Skipped(SkipReason::UnsupportedRevision);
            self.stop_cases(SkipReason::UnsupportedRevision);
            return;
        }
        if phase_failed(&self.envelope) {
            self.stop_before_cases(SkipReason::PrerequisiteFailed);
            return;
        }
        if !tools_advertised {
            self.discovery = PhaseState::Performed(vec![
                Finding::tool_not_found(Location::root(LocationField::Tools))
                    .with_revision(revision),
            ]);
            self.schemas = PhaseState::Skipped(SkipReason::PrerequisiteFailed);
            self.stop_cases(SkipReason::PrerequisiteFailed);
            return;
        }
        self.stage = Stage::Initialized { list_tools: true };
    }

    fn process_discovery(&mut self, response: &ProbeResponse) {
        let value: Value = serde_json::from_slice(response.as_bytes())
            .expect("the transport accepted this JSON response");
        let object = value
            .as_object()
            .expect("the transport accepted only JSON-RPC objects");
        if object.contains_key("error") {
            self.envelope = PhaseState::Performed(vec![Finding::catalog_contract_invalid(
                SupportedRevision::CURRENT,
                Location::root(LocationField::Server),
                RuleViolation::ServerErrorResponse,
            )]);
            self.stop_before_cases(SkipReason::PrerequisiteFailed);
            return;
        }
        let Some(result) = object.get("result").and_then(Value::as_object) else {
            self.envelope = PhaseState::Performed(vec![Finding::catalog_contract_invalid(
                SupportedRevision::CURRENT,
                Location::root(LocationField::Server).field(LocationField::Result),
                RuleViolation::ExpectedShape {
                    expected: ExpectedShape::Object,
                    observed: json_kind(object.get("result")),
                },
            )]);
            self.stop_before_cases(SkipReason::PrerequisiteFailed);
            return;
        };
        let server_location = Location::root(LocationField::Server);
        let common_findings = validate_cacheable_result(result, server_location.clone());
        if !common_findings.is_empty() {
            self.envelope = PhaseState::Performed(common_findings);
            self.stop_before_cases(SkipReason::PrerequisiteFailed);
            return;
        }
        self.envelope = PhaseState::Performed(Vec::new());

        let Some(versions) = result.get("supportedVersions").and_then(Value::as_array) else {
            self.revision = PhaseState::Performed(vec![Finding::invalid_revision_value(
                SupportedRevision::CURRENT,
                Location::root(LocationField::Server).field(LocationField::SupportedVersions),
                RedactedValue::new(
                    result
                        .get("supportedVersions")
                        .map(serialized_len)
                        .unwrap_or_default(),
                ),
            )]);
            self.stop_before_cases(SkipReason::PrerequisiteFailed);
            return;
        };
        let maximum_revisions = DiagnosticLimits::M1_DEFAULTS.values().protocol_revisions;
        let observed_revisions = u64::try_from(versions.len()).unwrap_or(u64::MAX);
        if observed_revisions > maximum_revisions {
            self.revision = PhaseState::Performed(vec![Finding::limit_exceeded(
                SupportedRevision::CURRENT,
                Location::root(LocationField::Server).field(LocationField::SupportedVersions),
                LimitViolation::new(
                    LimitKind::ProtocolRevisions,
                    observed_revisions,
                    maximum_revisions,
                )
                .expect("the revision advertisement exceeds its checked maximum"),
            )]);
            self.stop_before_cases(SkipReason::LimitReached);
            return;
        }
        let mut revision_findings = ActiveFindingCollector::new();
        let mut revision_values = Vec::with_capacity(versions.len());
        for (index, version) in versions.iter().enumerate() {
            if let Some(version) = version.as_str() {
                revision_values.push(version);
            } else {
                revision_findings.push(Finding::invalid_revision_value(
                    SupportedRevision::CURRENT,
                    Location::root(LocationField::Server)
                        .field(LocationField::SupportedVersions)
                        .index(index),
                    RedactedValue::new(serialized_len(version)),
                ));
            }
        }
        if !revision_findings.is_empty() {
            self.revision = PhaseState::Performed(revision_findings.finish(
                Location::root(LocationField::Server).field(LocationField::SupportedVersions),
                self.revision(),
            ));
            self.stop_before_cases(SkipReason::PrerequisiteFailed);
            return;
        }
        match select_server_revision(revision_values, maximum_revisions) {
            RevisionSelection::Selected(revision) => {
                self.revision = PhaseState::Performed(vec![Finding::revision_confirmed(
                    revision,
                    Location::root(LocationField::Server).field(LocationField::SupportedVersions),
                )]);
            }
            RevisionSelection::Unsupported(summary) => {
                self.revision = PhaseState::Performed(vec![Finding::unsupported_revision(
                    SupportedRevision::CURRENT,
                    Location::root(LocationField::Server).field(LocationField::SupportedVersions),
                    summary,
                )]);
                self.discovery = PhaseState::Skipped(SkipReason::UnsupportedRevision);
                self.schemas = PhaseState::Skipped(SkipReason::UnsupportedRevision);
                self.stop_cases(SkipReason::UnsupportedRevision);
                return;
            }
            RevisionSelection::LimitExceeded(violation) => {
                self.revision = PhaseState::Performed(vec![Finding::limit_exceeded(
                    SupportedRevision::CURRENT,
                    Location::root(LocationField::Server).field(LocationField::SupportedVersions),
                    violation,
                )]);
                self.stop_before_cases(SkipReason::LimitReached);
                return;
            }
        }

        let (capability_findings, tools_advertised) =
            validate_discovery_capabilities(result, server_location);
        if !capability_findings.is_empty() {
            self.envelope = PhaseState::Performed(capability_findings);
            self.stop_before_cases(SkipReason::PrerequisiteFailed);
            return;
        }
        if !tools_advertised {
            self.discovery = PhaseState::Performed(vec![Finding::tool_not_found(Location::root(
                LocationField::Tools,
            ))]);
            self.schemas = PhaseState::Skipped(SkipReason::PrerequisiteFailed);
            self.stop_cases(SkipReason::PrerequisiteFailed);
            return;
        }
        self.stage = Stage::Tools(None);
    }

    fn process_tools(&mut self, response: &ProbeResponse) {
        let revision = self.revision();
        let value: Value = serde_json::from_slice(response.as_bytes())
            .expect("the transport accepted this JSON response");
        let object = value
            .as_object()
            .expect("the transport accepted only JSON-RPC objects");
        let mut findings = ActiveFindingCollector::new();
        if object.contains_key("error") {
            findings.push(Finding::catalog_contract_invalid(
                revision,
                Location::root(LocationField::Tools),
                RuleViolation::ServerErrorResponse,
            ));
            self.fail_discovery(
                findings.finish(Location::root(LocationField::Tools), revision),
                SkipReason::PrerequisiteFailed,
            );
            return;
        }
        let Some(result) = object.get("result").and_then(Value::as_object) else {
            findings.push(Finding::catalog_contract_invalid(
                revision,
                Location::root(LocationField::Tools).field(LocationField::Result),
                RuleViolation::ExpectedShape {
                    expected: ExpectedShape::Object,
                    observed: json_kind(object.get("result")),
                },
            ));
            self.fail_discovery(
                findings.finish(Location::root(LocationField::Tools), revision),
                SkipReason::PrerequisiteFailed,
            );
            return;
        };
        let common_findings = match self.adapter.tool_result_kind() {
            ActiveToolResultKind::Modern => {
                validate_cacheable_result(result, Location::root(LocationField::Tools))
            }
            ActiveToolResultKind::Legacy => Vec::new(),
        };
        if !common_findings.is_empty() {
            self.fail_discovery(common_findings, SkipReason::PrerequisiteFailed);
            return;
        }
        let Some(tools) = result.get("tools").and_then(Value::as_array) else {
            findings.push(Finding::catalog_contract_invalid(
                revision,
                Location::root(LocationField::Tools),
                RuleViolation::ExpectedShape {
                    expected: ExpectedShape::Array,
                    observed: json_kind(result.get("tools")),
                },
            ));
            self.fail_discovery(
                findings.finish(Location::root(LocationField::Tools), revision),
                SkipReason::PrerequisiteFailed,
            );
            return;
        };
        let previously_observed = self.observed_items;
        let page_offset = usize::try_from(previously_observed).unwrap_or(usize::MAX);
        self.observed_items = self
            .observed_items
            .saturating_add(u64::try_from(tools.len()).unwrap_or(u64::MAX));
        let maximum_items = DiagnosticLimits::M1_DEFAULTS.values().catalog_items;
        if self.observed_items > maximum_items {
            findings.push(Finding::limit_exceeded(
                revision,
                Location::root(LocationField::Tools),
                LimitViolation::new(LimitKind::CatalogItems, self.observed_items, maximum_items)
                    .expect("the active catalog exceeds its checked maximum"),
            ));
        }
        let remaining = usize::try_from(maximum_items.saturating_sub(previously_observed))
            .unwrap_or(usize::MAX);

        for (page_index, tool) in tools.iter().take(remaining).enumerate() {
            let index = page_offset.saturating_add(page_index);
            let location = Location::root(LocationField::Tools).index(index);
            let Some(tool) = tool.as_object() else {
                findings.push(Finding::catalog_contract_invalid(
                    revision,
                    location,
                    RuleViolation::ExpectedShape {
                        expected: ExpectedShape::Object,
                        observed: json_kind(Some(tool)),
                    },
                ));
                continue;
            };
            let Some(name) = tool.get("name").and_then(Value::as_str) else {
                findings.push(Finding::catalog_contract_invalid(
                    revision,
                    location.clone().field(LocationField::Name),
                    RuleViolation::ExpectedShape {
                        expected: ExpectedShape::String,
                        observed: json_kind(tool.get("name")),
                    },
                ));
                continue;
            };
            if !self.seen_names.insert(name.to_owned()) {
                findings.push(Finding::duplicate_catalog_identifier(
                    revision,
                    location.clone().field(LocationField::Name),
                ));
                continue;
            }
            if !self.scenario.tools.contains(name) {
                continue;
            }
            self.selected_names.insert(name.to_owned());
            match self.adapter.task_support(tool) {
                ActiveTaskSupport::Immediate => {}
                ActiveTaskSupport::Required => {
                    self.selected_schema_findings
                        .push(Finding::tool_task_required(
                            revision,
                            location
                                .clone()
                                .field(LocationField::Execution)
                                .field(LocationField::TaskSupport),
                        ));
                    continue;
                }
                ActiveTaskSupport::InvalidExecution => {
                    self.selected_schema_findings
                        .push(Finding::catalog_contract_invalid(
                            revision,
                            location.clone().field(LocationField::Execution),
                            RuleViolation::ExpectedShape {
                                expected: ExpectedShape::Object,
                                observed: json_kind(tool.get("execution")),
                            },
                        ));
                    continue;
                }
                ActiveTaskSupport::InvalidTaskSupport => {
                    let execution = tool.get("execution").and_then(Value::as_object);
                    self.selected_schema_findings
                        .push(Finding::catalog_contract_invalid(
                            revision,
                            location
                                .clone()
                                .field(LocationField::Execution)
                                .field(LocationField::TaskSupport),
                            RuleViolation::ExpectedTaskSupport {
                                observed: json_kind(
                                    execution.and_then(|execution| execution.get("taskSupport")),
                                ),
                            },
                        ));
                    continue;
                }
            }
            let Some(input_schema) = tool.get("inputSchema") else {
                self.selected_schema_findings
                    .push(Finding::schema_contract_invalid(
                        revision,
                        location.clone().field(LocationField::InputSchema),
                        RuleViolation::ExpectedShape {
                            expected: ExpectedShape::Object,
                            observed: JsonKind::Missing,
                        },
                    ));
                continue;
            };
            if !input_schema.is_object() {
                self.selected_schema_findings
                    .push(Finding::schema_contract_invalid(
                        revision,
                        location.clone().field(LocationField::InputSchema),
                        RuleViolation::ExpectedShape {
                            expected: ExpectedShape::Object,
                            observed: json_kind(Some(input_schema)),
                        },
                    ));
                continue;
            }
            let output_schema = tool.get("outputSchema").cloned();
            if output_schema
                .as_ref()
                .is_some_and(|schema| !schema.is_object())
            {
                self.selected_schema_findings
                    .push(Finding::schema_contract_invalid(
                        revision,
                        location.field(LocationField::OutputSchema),
                        RuleViolation::ExpectedShape {
                            expected: ExpectedShape::Object,
                            observed: json_kind(output_schema.as_ref()),
                        },
                    ));
                continue;
            }
            let header_annotations = if self.validate_http_headers {
                match validate_annotations(
                    input_schema,
                    location.clone().field(LocationField::InputSchema),
                ) {
                    Ok(annotations) => annotations,
                    Err(finding) => {
                        self.selected_schema_findings.push(finding);
                        continue;
                    }
                }
            } else {
                Vec::new()
            };
            self.selected_tools.insert(
                name.to_owned(),
                ToolContract {
                    input_schema: input_schema.clone(),
                    output_schema,
                    header_annotations,
                },
            );
        }
        if !findings.is_empty() {
            self.fail_discovery(
                findings.finish(Location::root(LocationField::Tools), revision),
                SkipReason::PrerequisiteFailed,
            );
            return;
        }

        let next_cursor = match result.get("nextCursor") {
            None => None,
            Some(Value::String(cursor)) => Some(cursor.clone()),
            Some(value) => {
                self.fail_discovery(
                    vec![Finding::catalog_contract_invalid(
                        revision,
                        Location::root(LocationField::Tools).field(LocationField::NextCursor),
                        RuleViolation::ExpectedShape {
                            expected: ExpectedShape::String,
                            observed: json_kind(Some(value)),
                        },
                    )],
                    SkipReason::PrerequisiteFailed,
                );
                return;
            }
        };
        if let Some(cursor) = next_cursor {
            if !self.seen_cursors.insert(cursor.clone()) {
                self.fail_discovery(
                    vec![Finding::pagination_cursor_repeated(
                        revision,
                        Location::root(LocationField::Tools).field(LocationField::NextCursor),
                    )],
                    SkipReason::PrerequisiteFailed,
                );
                return;
            }
            self.stage = Stage::Tools(Some(cursor));
            return;
        }

        if self.selected_names.len() != self.scenario.tools.len() {
            self.fail_discovery(
                vec![
                    Finding::tool_not_found(Location::root(LocationField::Tools))
                        .with_revision(revision),
                ],
                SkipReason::PrerequisiteFailed,
            );
            return;
        }
        self.discovery = PhaseState::Performed(Vec::new());
        let mut schema_findings = std::mem::take(&mut self.selected_schema_findings);
        if self.selected_tools.len() != self.scenario.tools.len() {
            debug_assert!(!schema_findings.is_empty());
            self.schemas = PhaseState::Performed(cap_active_phase_findings(
                schema_findings,
                false,
                Location::root(LocationField::Tools),
                revision,
            ));
            self.stop_cases(SkipReason::PrerequisiteFailed);
            return;
        }
        for contract in self.selected_tools.values() {
            let input_location = Location::root(LocationField::Tools)
                .wildcard()
                .field(LocationField::InputSchema);
            if contract.input_schema.get("type").and_then(Value::as_str) != Some("object") {
                schema_findings.push(Finding::schema_contract_invalid(
                    revision,
                    input_location.clone().field(LocationField::Type),
                    RuleViolation::ExpectedInputSchemaRootObject {
                        observed: json_kind(contract.input_schema.get("type")),
                    },
                ));
            }
            schema_findings.extend(
                validate_local_schema_with_policy(
                    &contract.input_schema,
                    input_location,
                    self.adapter.schema_dialect_policy(),
                )
                .into_iter()
                .map(|finding| finding.with_revision(revision)),
            );
            if let Some(output_schema) = &contract.output_schema {
                schema_findings.extend(
                    validate_local_schema_with_policy(
                        output_schema,
                        Location::root(LocationField::Tools)
                            .wildcard()
                            .field(LocationField::OutputSchema),
                        self.adapter.schema_dialect_policy(),
                    )
                    .into_iter()
                    .map(|finding| finding.with_revision(revision)),
                );
            }
        }
        if schema_findings.is_empty() {
            self.tool_validators = self
                .selected_tools
                .iter()
                .map(|(name, contract)| {
                    (
                        name.clone(),
                        ToolValidators {
                            input: LocalValidator::compile(&contract.input_schema).expect(
                                "a validated advertised input schema compiles without retrieval",
                            ),
                            output: contract.output_schema.as_ref().map(|schema| {
                                LocalValidator::compile(schema).expect(
                                    "a validated advertised output schema compiles without retrieval",
                                )
                            }),
                            header_annotations: contract.header_annotations.clone(),
                        },
                    )
                })
                .collect();
            if self.scenario.generates_cases() {
                let tool = self
                    .scenario
                    .tools
                    .iter()
                    .next()
                    .expect("generated scenarios declare one tool");
                let input_schema = self
                    .selected_tools
                    .get(tool)
                    .expect("the generated tool contract was selected")
                    .input_schema
                    .clone();
                let input_validator = &self
                    .tool_validators
                    .get(tool)
                    .expect("the generated tool validator was compiled")
                    .input;
                match self.scenario.generate_cases(&input_schema, input_validator) {
                    Ok(()) => {
                        self.generation = Some(PhaseState::Performed(Vec::new()));
                    }
                    Err(GenerationFailure::Limit(violation)) => {
                        self.schemas = PhaseState::Performed(Vec::new());
                        self.generation =
                            Some(PhaseState::Performed(vec![Finding::limit_exceeded(
                                revision,
                                generation_location().field(LocationField::Cases),
                                violation,
                            )]));
                        self.stop_cases(SkipReason::LimitReached);
                        return;
                    }
                    Err(GenerationFailure::Unavailable) => {
                        self.schemas = PhaseState::Performed(Vec::new());
                        self.generation = Some(PhaseState::Performed(vec![
                            Finding::case_generation_failed(
                                generation_location().field(LocationField::Cases),
                                RuleViolation::NoValidBoundaryInput,
                            )
                            .with_revision(revision),
                        ]));
                        self.stop_cases(SkipReason::PrerequisiteFailed);
                        return;
                    }
                }
            }
            self.selected_tools.clear();
            self.schemas = PhaseState::Performed(Vec::new());
            self.stage = Stage::Cases;
        } else {
            self.schemas = PhaseState::Performed(cap_active_phase_findings(
                schema_findings,
                false,
                Location::root(LocationField::Tools),
                revision,
            ));
            self.stop_cases(SkipReason::PrerequisiteFailed);
        }
    }

    fn process_call(&mut self, index: usize, response: &ProbeResponse) {
        let value: Value = serde_json::from_slice(response.as_bytes())
            .expect("the transport accepted this JSON response");
        let object = value
            .as_object()
            .expect("the transport accepted only JSON-RPC objects");
        if object.contains_key("method") {
            if self.adapter.tool_result_kind() == ActiveToolResultKind::Legacy
                && server_request_requires_input(object)
            {
                self.mark_incomplete(index, false);
            } else {
                self.invalid_tool_result(index, LocationField::Request);
            }
            return;
        }
        if self.scenario.cases[index].expected == ExpectedResult::InvalidArgumentsRejection {
            self.process_invalid_arguments_rejection(index, object);
            return;
        }
        if object.contains_key("error") {
            if self.adapter.tool_result_kind() == ActiveToolResultKind::Legacy
                && is_url_elicitation_required(object)
            {
                self.mark_incomplete(index, true);
                return;
            }
            self.complete_case(
                index,
                vec![
                    Finding::tool_call_rejected(
                        self.scenario
                            .case_location(index)
                            .field(LocationField::Result),
                    )
                    .with_revision(self.revision()),
                ],
            );
            return;
        }
        let Some(result) = object.get("result").and_then(Value::as_object) else {
            self.invalid_tool_result(index, LocationField::Result);
            return;
        };
        if self.adapter.tool_result_kind() == ActiveToolResultKind::Modern {
            match result.get("resultType").and_then(Value::as_str) {
                Some("input_required") => {
                    self.mark_incomplete(index, true);
                    return;
                }
                Some("complete") => {}
                _ => {
                    self.invalid_tool_result(index, LocationField::ResultType);
                    return;
                }
            }
        }
        if !result.get("content").is_some_and(Value::is_array) {
            self.invalid_tool_result(index, LocationField::Content);
            return;
        }
        let is_error = match result.get("isError") {
            None => false,
            Some(Value::Bool(value)) => *value,
            Some(_) => {
                self.invalid_tool_result(index, LocationField::IsError);
                return;
            }
        };
        let structured = match result.get("structuredContent") {
            None => None,
            Some(value) if value.is_object() => Some(value),
            Some(_) => {
                self.invalid_tool_result(index, LocationField::StructuredContent);
                return;
            }
        };

        let mut findings = Vec::new();
        let case = &self.scenario.cases[index];
        let classification_matches = matches!(
            (case.expected, is_error),
            (ExpectedResult::Success, false) | (ExpectedResult::ToolError, true)
        );
        if !classification_matches {
            findings.push(
                Finding::tool_result_mismatch(
                    self.scenario
                        .case_location(index)
                        .field(LocationField::IsError),
                    if case.expected == ExpectedResult::Success {
                        RuleViolation::ExpectedSuccess
                    } else {
                        RuleViolation::ExpectedToolError
                    },
                )
                .with_revision(self.revision()),
            );
        }

        let advertised = self
            .tool_validators
            .get(&case.tool)
            .and_then(|validators| validators.output.as_ref());
        let scenario_validator = case.output_validator.as_ref();
        let mut advertised_mismatch = None;
        let mut scenario_mismatch = None;
        if let Some(validator) = advertised {
            advertised_mismatch = validate_optional_output(validator, structured);
        }
        if let Some(validator) = scenario_validator {
            scenario_mismatch = validate_optional_output(validator, structured);
        }
        let limit = advertised_mismatch
            .as_ref()
            .and_then(validation_limit)
            .or_else(|| scenario_mismatch.as_ref().and_then(validation_limit));
        if let Some(violation) = limit {
            findings.push(Finding::limit_exceeded(
                self.revision(),
                self.scenario
                    .case_location(index)
                    .field(LocationField::StructuredContent),
                violation,
            ));
            self.stop_case(index, findings, SkipReason::LimitReached);
            return;
        }
        if advertised_mismatch
            .as_ref()
            .is_some_and(validation_invalid_schema)
            || scenario_mismatch
                .as_ref()
                .is_some_and(validation_invalid_schema)
        {
            findings.push(
                Finding::tool_result_invalid(
                    self.scenario
                        .case_location(index)
                        .field(LocationField::StructuredContent),
                )
                .with_revision(self.revision()),
            );
            self.stop_case(index, findings, SkipReason::PrerequisiteFailed);
            return;
        }

        let advertised_errors = mismatch_count(advertised_mismatch.as_ref());
        let scenario_errors = mismatch_count(scenario_mismatch.as_ref());
        if advertised_errors > 0 || scenario_errors > 0 {
            let violation = match (advertised_errors > 0, scenario_errors > 0) {
                (true, true) => RuleViolation::AdvertisedAndScenarioOutputMismatch {
                    error_count: advertised_errors.saturating_add(scenario_errors),
                },
                (true, false) => RuleViolation::AdvertisedOutputMismatch {
                    error_count: advertised_errors,
                },
                (false, true) => RuleViolation::ScenarioOutputMismatch {
                    error_count: scenario_errors,
                },
                (false, false) => unreachable!(),
            };
            findings.push(
                Finding::tool_output_mismatch(
                    self.scenario
                        .case_location(index)
                        .field(LocationField::StructuredContent),
                    violation,
                )
                .with_revision(self.revision()),
            );
        }

        if findings.is_empty() && self.scenario.is_workflow() {
            match self.capture_workflow_values(index, structured) {
                Ok(()) => {}
                Err(CaptureFailure::Missing) => findings.push(
                    Finding::workflow_capture_missing(
                        self.scenario
                            .case_location(index)
                            .field(LocationField::Captures)
                            .wildcard(),
                    )
                    .with_revision(self.revision()),
                ),
                Err(CaptureFailure::Limit(violation)) => {
                    findings.push(Finding::limit_exceeded(
                        self.revision(),
                        self.scenario
                            .case_location(index)
                            .field(LocationField::Captures),
                        violation,
                    ));
                    self.stop_case(index, findings, SkipReason::LimitReached);
                    return;
                }
            }
        }

        self.complete_case(index, findings);
    }

    fn process_invalid_arguments_rejection(&mut self, index: usize, response: &Map<String, Value>) {
        if let Some(error) = response.get("error").and_then(Value::as_object) {
            let accepted = error.get("code").and_then(Value::as_i64) == Some(-32602)
                && error.get("message").is_some_and(Value::is_string);
            if accepted {
                self.case_states[index] = CaseState::Performed(Vec::new());
                self.next_case = index.saturating_add(1);
                self.stage = Stage::Cases;
            } else {
                self.invalid_tool_result(index, LocationField::Result);
            }
            return;
        }
        if response.contains_key("error") {
            self.invalid_tool_result(index, LocationField::Result);
            return;
        }
        self.case_states[index] = CaseState::Performed(vec![
            Finding::schema_invalid_arguments_accepted(
                self.scenario
                    .case_location(index)
                    .field(LocationField::Result),
            )
            .with_revision(self.revision()),
        ]);
        self.next_case = index.saturating_add(1);
        self.stop_cases(SkipReason::PrerequisiteFailed);
    }

    fn invalid_tool_result(&mut self, index: usize, field: LocationField) {
        let findings = vec![
            Finding::tool_result_invalid(self.scenario.case_location(index).field(field))
                .with_revision(self.revision()),
        ];
        self.stop_case(index, findings, SkipReason::PrerequisiteFailed);
    }

    fn fail_discovery(&mut self, findings: Vec<Finding>, reason: SkipReason) {
        self.discovery = PhaseState::Performed(findings);
        self.schemas = PhaseState::Skipped(reason);
        self.stop_cases(reason);
    }

    fn stop_before_cases(&mut self, reason: SkipReason) {
        if matches!(self.discovery, PhaseState::Pending) {
            self.discovery = PhaseState::Skipped(reason);
        }
        if matches!(self.schemas, PhaseState::Pending) {
            self.schemas = PhaseState::Skipped(reason);
        }
        self.stop_cases(reason);
    }

    fn stop_cases(&mut self, reason: SkipReason) {
        if let Some(state @ PhaseState::Pending) = self.generation.as_mut() {
            *state = PhaseState::Skipped(reason);
        }
        for state in &mut self.case_states[self.next_case..] {
            if matches!(state, CaseState::Pending) {
                *state = CaseState::Skipped(reason);
            }
        }
        self.stage = Stage::Done;
    }

    pub(crate) fn into_diagnostic(self, stdio: StdioDiagnostic) -> Diagnostic {
        let revision = self.revision();
        let failed = stdio.primary.is_some();
        let transport_findings = stdio_findings_for_revision(stdio, revision);
        self.into_transport_diagnostic(
            vec![CheckResult::performed(
                CheckId::TransportStdio,
                Requirement::Required,
                transport_findings,
            )],
            failed,
        )
    }

    pub(crate) fn into_http_diagnostic(self, http: HttpDiagnostic) -> Diagnostic {
        if http.unsupported_protocol_version() {
            return self.into_protocol_version_rejection(http);
        }
        let failed = http.failed();
        let revision = self.revision();
        self.into_transport_diagnostic(http_checks_for_revision(http, revision), failed)
    }

    fn into_protocol_version_rejection(mut self, http: HttpDiagnostic) -> Diagnostic {
        self.pending = None;
        self.envelope = PhaseState::Performed(Vec::new());
        self.revision = PhaseState::Performed(vec![Finding::unsupported_protocol_version(
            self.revision(),
            Location::root(LocationField::Http).field(LocationField::Body),
        )]);
        self.discovery = PhaseState::Skipped(SkipReason::UnsupportedRevision);
        self.schemas = PhaseState::Skipped(SkipReason::UnsupportedRevision);
        self.stop_cases(SkipReason::UnsupportedRevision);
        let revision = self.revision();
        self.into_transport_diagnostic(
            http_checks_for_revision(http.without_primary_failure(), revision),
            false,
        )
    }

    fn into_transport_diagnostic(
        mut self,
        transport_checks: Vec<CheckResult>,
        transport_failed: bool,
    ) -> Diagnostic {
        let report_revision = self.revision();
        if transport_failed {
            if let Some(PendingRequest::Call(index)) = self.pending.take() {
                self.case_states[index] = if self.scenario.case_is_cleanup(index) {
                    CaseState::Performed(vec![
                        Finding::workflow_cleanup_failed(
                            self.scenario
                                .case_location(index)
                                .field(LocationField::Cleanup),
                        )
                        .with_revision(report_revision),
                    ])
                } else {
                    CaseState::Skipped(SkipReason::PrerequisiteFailed)
                };
                self.next_case = index.saturating_add(1);
            }
            self.stop_before_cases(SkipReason::PrerequisiteFailed);
        }
        let transport_finding_count = transport_checks
            .iter()
            .filter_map(CheckResult::findings)
            .map(<[Finding]>::len)
            .sum();
        self.fit_report_finding_budget(transport_finding_count);
        let configuration_check = self.scenario.configuration_check();
        let case_requirement = self.scenario.case_requirement();
        let case_ids = (0..self.case_states.len())
            .map(|index| self.scenario.case_check_id(index))
            .collect::<Vec<_>>();
        self.captured_values.clear();
        self.captured_bytes = 0;
        let mut reproductions = (0..self.case_states.len())
            .map(|index| {
                self.scenario
                    .cases
                    .get_mut(index)
                    .and_then(|case| case.reproduction.take())
            })
            .collect::<Vec<_>>()
            .into_iter();
        let mut checks = vec![
            CheckResult::performed(configuration_check, Requirement::Required, Vec::new()),
            CheckResult::performed(
                CheckId::ActiveAuthorization,
                Requirement::Required,
                Vec::new(),
            ),
        ];
        checks.extend(transport_checks);
        checks.extend([
            phase_check(CheckId::ProtocolEnvelope, self.envelope),
            phase_check(CheckId::ProtocolRevision, self.revision),
            phase_check(CheckId::DiscoveryCatalogs, self.discovery),
            phase_check(CheckId::SchemaContracts, self.schemas),
        ]);
        if let Some(generation) = self.generation {
            checks.push(phase_check(CheckId::CaseGeneration, generation));
        }
        checks.extend(
            self.case_states
                .into_iter()
                .zip(case_ids)
                .map(|(state, check_id)| {
                    let check = match state {
                        CaseState::Performed(findings) => {
                            CheckResult::performed(check_id, case_requirement, findings)
                        }
                        CaseState::Incomplete => CheckResult::skipped(
                            check_id,
                            case_requirement,
                            SkipReason::InputRequired,
                        ),
                        CaseState::Skipped(reason) => {
                            CheckResult::skipped(check_id, case_requirement, reason)
                        }
                        CaseState::Pending => CheckResult::skipped(
                            check_id,
                            case_requirement,
                            SkipReason::PrerequisiteFailed,
                        ),
                    };
                    match reproductions.next().flatten() {
                        Some(reproduction) => check.with_reproduction(reproduction),
                        None => check,
                    }
                }),
        );
        let mut report =
            DiagnosticReport::new(report_revision, DiagnosticLimits::M1_DEFAULTS, checks)
                .expect("the active application must construct a valid diagnostic report");
        if let Some(negotiated_revision) = self.negotiated_revision {
            report = report.with_negotiated_revision(negotiated_revision);
        }
        Diagnostic::from_report(report)
    }

    fn fit_report_finding_budget(&mut self, transport_findings: usize) {
        let revision = self.revision();
        let maximum = usize::try_from(DiagnosticLimits::M1_DEFAULTS.values().report_findings)
            .unwrap_or(usize::MAX);
        let total = transport_findings
            .saturating_add(phase_finding_count(&self.envelope))
            .saturating_add(phase_finding_count(&self.revision))
            .saturating_add(phase_finding_count(&self.discovery))
            .saturating_add(phase_finding_count(&self.schemas))
            .saturating_add(self.generation.as_ref().map_or(0, phase_finding_count))
            .saturating_add(
                self.case_states
                    .iter()
                    .map(case_finding_count)
                    .sum::<usize>(),
            );
        if total <= maximum {
            return;
        }

        let observed = u64::try_from(total).unwrap_or(u64::MAX);
        let target = match &mut self.discovery {
            PhaseState::Performed(findings) if !findings.is_empty() => {
                Some((findings, Location::root(LocationField::Tools)))
            }
            _ => match &mut self.schemas {
                PhaseState::Performed(findings) if !findings.is_empty() => {
                    Some((findings, Location::root(LocationField::Tools)))
                }
                _ => match &mut self.revision {
                    PhaseState::Performed(findings) if !findings.is_empty() => Some((
                        findings,
                        Location::root(LocationField::Server)
                            .field(LocationField::SupportedVersions),
                    )),
                    _ => None,
                },
            },
        }
        .expect("only bounded catalog or schema findings can exhaust the active report budget");
        let target_count = target.0.len();
        let capacity = maximum.saturating_sub(total.saturating_sub(target_count));
        assert!(
            capacity > 0,
            "independent active findings fit the report budget"
        );
        *target.0 = cap_active_phase_to_budget(
            std::mem::take(target.0),
            capacity,
            observed,
            target.1,
            revision,
        );
    }
}

impl Conversation for ActiveConversation {
    fn next_request(&mut self, previous: Option<&ProbeResponse>) -> Option<ProbeRequest> {
        if let Some(response) = previous {
            if self.pending.is_some() {
                self.process_response(response);
            }
        } else {
            assert!(
                self.pending.is_none(),
                "the adapter start message is the first active request"
            );
        }
        self.next_outbound()
    }
}

fn phase_check(id: CheckId, state: PhaseState) -> CheckResult {
    match state {
        PhaseState::Performed(findings) => {
            CheckResult::performed(id, Requirement::Required, findings)
        }
        PhaseState::Skipped(reason) => CheckResult::skipped(id, Requirement::Required, reason),
        PhaseState::Pending => {
            CheckResult::skipped(id, Requirement::Required, SkipReason::PrerequisiteFailed)
        }
    }
}

fn phase_finding_count(state: &PhaseState) -> usize {
    match state {
        PhaseState::Performed(findings) => findings.len(),
        PhaseState::Pending | PhaseState::Skipped(_) => 0,
    }
}

fn phase_failed(state: &PhaseState) -> bool {
    matches!(
        state,
        PhaseState::Performed(findings)
            if findings.iter().any(|finding| finding.severity().is_failure())
    )
}

fn case_finding_count(state: &CaseState) -> usize {
    match state {
        CaseState::Performed(findings) => findings.len(),
        CaseState::Pending | CaseState::Incomplete | CaseState::Skipped(_) => 0,
    }
}

fn cap_active_phase_findings(
    mut findings: Vec<Finding>,
    overflow: bool,
    location: Location,
    revision: SupportedRevision,
) -> Vec<Finding> {
    findings.sort();
    findings.dedup();
    let capacity = report_finding_capacity();
    if !overflow && findings.len() <= capacity {
        return findings;
    }

    cap_active_phase_to_budget(
        findings,
        capacity,
        DiagnosticLimits::M1_DEFAULTS
            .values()
            .report_findings
            .saturating_add(1),
        location,
        revision,
    )
}

fn cap_active_phase_to_budget(
    mut findings: Vec<Finding>,
    capacity: usize,
    observed: u64,
    location: Location,
    revision: SupportedRevision,
) -> Vec<Finding> {
    let maximum = DiagnosticLimits::M1_DEFAULTS.values().report_findings;
    findings.retain(|finding| {
        !matches!(
            finding.evidence(),
            FindingEvidence::LimitViolation(violation)
                if violation.kind() == LimitKind::ReportFindings
        )
    });
    findings.sort();
    findings.dedup();
    findings.truncate(capacity.saturating_sub(1));
    findings.push(Finding::limit_exceeded(
        revision,
        location,
        LimitViolation::new(
            LimitKind::ReportFindings,
            observed.max(maximum.saturating_add(1)),
            maximum,
        )
        .expect("the active report overflow observation exceeds its maximum"),
    ));
    findings
}

fn report_finding_capacity() -> usize {
    usize::try_from(DiagnosticLimits::M1_DEFAULTS.values().report_findings).unwrap_or(usize::MAX)
}

fn validate_optional_output(
    validator: &LocalValidator,
    structured: Option<&Value>,
) -> Option<InstanceValidationIssue> {
    match structured {
        Some(value) => validator.validate(value).err(),
        None => Some(InstanceValidationIssue::Mismatch { error_count: 1 }),
    }
}

fn is_url_elicitation_required(response: &Map<String, Value>) -> bool {
    let Some(error) = response.get("error").and_then(Value::as_object) else {
        return false;
    };
    if error.get("code").and_then(Value::as_i64) != Some(-32042)
        || !error.get("message").is_some_and(Value::is_string)
    {
        return false;
    }
    let Some(elicitations) = error
        .get("data")
        .and_then(Value::as_object)
        .and_then(|data| data.get("elicitations"))
        .and_then(Value::as_array)
    else {
        return false;
    };
    let maximum =
        usize::try_from(DiagnosticLimits::M1_DEFAULTS.values().active_cases).unwrap_or(usize::MAX);
    elicitations.len() <= maximum
        && elicitations.iter().all(|elicitation| {
            let Some(elicitation) = elicitation.as_object() else {
                return false;
            };
            elicitation.get("mode").and_then(Value::as_str) == Some("url")
                && elicitation
                    .get("elicitationId")
                    .and_then(Value::as_str)
                    .is_some()
                && elicitation.get("message").and_then(Value::as_str).is_some()
                && elicitation
                    .get("url")
                    .and_then(Value::as_str)
                    .is_some_and(|url| Url::parse(url).is_ok())
                && elicitation.get("_meta").is_none_or(valid_elicitation_meta)
                && elicitation.get("task").is_none_or(valid_task_metadata)
        })
}

fn valid_elicitation_meta(value: &Value) -> bool {
    value.as_object().is_some_and(|meta| {
        meta.get("progressToken")
            .is_none_or(|token| token.is_string() || json_integer(token))
    })
}

fn valid_task_metadata(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|task| task.get("ttl").is_none_or(json_integer))
}

fn json_integer(value: &Value) -> bool {
    value.as_number().is_some_and(|number| {
        number.is_i64()
            || number.is_u64()
            || number.as_f64().is_some_and(|number| number.fract() == 0.0)
    })
}

fn server_request_requires_input(request: &Map<String, Value>) -> bool {
    let method = request.get("method").and_then(Value::as_str);
    let params = request.get("params").and_then(Value::as_object);
    match method {
        Some("elicitation/create") => params.is_some_and(valid_elicitation_request),
        Some("sampling/createMessage") => params.is_some_and(|params| {
            params.get("maxTokens").is_some_and(json_integer)
                && params.get("messages").is_some_and(Value::is_array)
                && params.get("_meta").is_none_or(valid_elicitation_meta)
                && params.get("task").is_none_or(valid_task_metadata)
        }),
        Some("roots/list") => request.get("params").is_none_or(|params| {
            params
                .as_object()
                .is_some_and(|params| params.get("_meta").is_none_or(valid_elicitation_meta))
        }),
        Some(_) | None => false,
    }
}

fn valid_elicitation_request(params: &Map<String, Value>) -> bool {
    if !params.get("message").is_some_and(Value::is_string)
        || !params.get("_meta").is_none_or(valid_elicitation_meta)
        || !params.get("task").is_none_or(valid_task_metadata)
    {
        return false;
    }
    match params.get("mode").and_then(Value::as_str) {
        None | Some("form") => params
            .get("requestedSchema")
            .and_then(Value::as_object)
            .is_some_and(|schema| {
                schema.get("type").and_then(Value::as_str) == Some("object")
                    && schema.get("properties").is_some_and(Value::is_object)
                    && schema.get("$schema").is_none_or(Value::is_string)
                    && schema.get("required").is_none_or(|required| {
                        required
                            .as_array()
                            .is_some_and(|required| required.iter().all(Value::is_string))
                    })
            }),
        Some("url") => {
            params.get("elicitationId").is_some_and(Value::is_string)
                && params
                    .get("url")
                    .and_then(Value::as_str)
                    .is_some_and(|url| Url::parse(url).is_ok())
        }
        Some(_) => false,
    }
}

fn scenario_schema_findings(findings: Vec<Finding>) -> Vec<Finding> {
    findings
        .into_iter()
        .map(|finding| {
            if finding.code() == FindingCode::SchemaContractInvalid
                && let FindingEvidence::RuleViolation(violation) = finding.evidence()
            {
                return Finding::scenario_schema_invalid(finding.location().clone(), *violation);
            }
            finding
        })
        .collect()
}

fn validation_limit(issue: &InstanceValidationIssue) -> Option<LimitViolation> {
    match issue {
        InstanceValidationIssue::Limit(violation) => Some(*violation),
        _ => None,
    }
}

fn validation_invalid_schema(issue: &InstanceValidationIssue) -> bool {
    matches!(issue, InstanceValidationIssue::InvalidSchema)
}

fn mismatch_count(issue: Option<&InstanceValidationIssue>) -> u64 {
    match issue {
        Some(InstanceValidationIssue::Mismatch { error_count }) => *error_count,
        _ => 0,
    }
}

fn scenario_location() -> Location {
    Location::root(LocationField::Scenario)
}

fn generation_location() -> Location {
    Location::root(LocationField::Generation)
}

fn case_location(index: usize) -> Location {
    scenario_location().field(LocationField::Cases).index(index)
}

fn workflow_step_location(index: usize) -> Location {
    scenario_location().field(LocationField::Steps).index(index)
}

fn shape_failure(
    location: Location,
    expected: ExpectedShape,
    observed: JsonKind,
) -> ScenarioFailure {
    ScenarioFailure::one(Finding::scenario_invalid(
        location,
        RuleViolation::ExpectedShape { expected, observed },
    ))
}

fn required_object(
    value: Option<&Value>,
    location: Location,
) -> Result<&Map<String, Value>, ScenarioFailure> {
    value
        .and_then(Value::as_object)
        .ok_or_else(|| shape_failure(location, ExpectedShape::Object, json_kind(value)))
}

fn required_nonempty_string(
    value: Option<&Value>,
    location: Location,
) -> Result<&str, ScenarioFailure> {
    match value.and_then(Value::as_str) {
        Some(value) if !value.is_empty() => Ok(value),
        _ => Err(shape_failure(
            location,
            ExpectedShape::String,
            json_kind(value),
        )),
    }
}

fn ensure_fields(
    object: &Map<String, Value>,
    allowed: &[&str],
    required: &[&str],
    location: Location,
) -> Result<(), ScenarioFailure> {
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(ScenarioFailure::one(Finding::scenario_invalid(
            location.wildcard(),
            RuleViolation::InvalidScenarioShape,
        )));
    }
    if required.iter().any(|field| !object.contains_key(*field)) {
        return Err(ScenarioFailure::one(Finding::scenario_invalid(
            location,
            RuleViolation::InvalidScenarioShape,
        )));
    }
    Ok(())
}

fn parse_target_environment(root: &Map<String, Value>) -> Result<Vec<String>, ScenarioFailure> {
    let mut target_env = Vec::new();
    let mut target_names = BTreeSet::new();
    if let Some(value) = root.get("target_env") {
        let values = value.as_array().ok_or_else(|| {
            shape_failure(
                scenario_location().field(LocationField::TargetEnv),
                ExpectedShape::Array,
                json_kind(Some(value)),
            )
        })?;
        for (index, value) in values.iter().enumerate() {
            let location = scenario_location()
                .field(LocationField::TargetEnv)
                .index(index);
            let Some(name) = value.as_str() else {
                return Err(shape_failure(
                    location,
                    ExpectedShape::String,
                    json_kind(Some(value)),
                ));
            };
            let identity = environment_identity(name);
            if !valid_environment_name(name) || !target_names.insert(identity) {
                return Err(ScenarioFailure::one(Finding::secret_reference_invalid(
                    location,
                    RuleViolation::InvalidEnvironmentReference,
                )));
            }
            target_env.push(name.to_owned());
        }
    }
    Ok(target_env)
}

fn check_instance_bytes(value: &Value, location: Location) -> Result<(), ScenarioFailure> {
    let observed = u64::try_from(serialized_len(value)).unwrap_or(u64::MAX);
    let maximum = DiagnosticLimits::M1_DEFAULTS.values().instance_bytes;
    if observed > maximum {
        return Err(ScenarioFailure::one(Finding::limit_exceeded(
            SupportedRevision::CURRENT,
            location,
            LimitViolation::new(LimitKind::InstanceBytes, observed, maximum)
                .expect("the scenario instance exceeds its checked maximum"),
        )));
    }
    Ok(())
}

fn valid_json_pointer(pointer: &str) -> bool {
    if pointer.is_empty() || !pointer.starts_with('/') {
        return false;
    }
    let bytes = pointer.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'~' {
            if !matches!(bytes.get(index + 1), Some(b'0' | b'1')) {
                return false;
            }
            index += 2;
        } else {
            index += 1;
        }
    }
    true
}

fn valid_workflow_pointer(pointer: &str) -> bool {
    pointer.chars().count() <= MAX_WORKFLOW_POINTER_CHARS
        && (pointer.is_empty() || valid_json_pointer(pointer))
}

fn pointers_overlap(left: &str, right: &str) -> bool {
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|suffix| suffix.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn valid_environment_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first == b'_' || first.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

fn valid_workflow_name(name: &str) -> bool {
    name.chars().count() <= MAX_WORKFLOW_NAME_CHARS && valid_environment_name(name)
}

fn environment_identity(name: &str) -> String {
    #[cfg(windows)]
    {
        name.to_ascii_lowercase()
    }
    #[cfg(not(windows))]
    {
        name.to_owned()
    }
}

fn serialized_len(value: &Value) -> usize {
    serde_json::to_vec(value)
        .map(|bytes| bytes.len())
        .unwrap_or(usize::MAX)
}

fn json_kind(value: Option<&Value>) -> JsonKind {
    match value {
        None => JsonKind::Missing,
        Some(Value::Null) => JsonKind::Null,
        Some(Value::Bool(_)) => JsonKind::Boolean,
        Some(Value::Number(_)) => JsonKind::Number,
        Some(Value::String(_)) => JsonKind::String,
        Some(Value::Array(_)) => JsonKind::Array,
        Some(Value::Object(_)) => JsonKind::Object,
    }
}

struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueValueVisitor)
    }
}

struct UniqueValueVisitor;

impl<'de> Visitor<'de> for UniqueValueVisitor {
    type Value = UniqueValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("one JSON value without duplicate object members")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(Number::from(value))))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(Number::from(value))))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .map(UniqueValue)
            .ok_or_else(|| E::custom("invalid JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value)))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(UniqueValue(value)) = sequence.next_element()? {
            values.push(value);
        }
        Ok(UniqueValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("duplicate JSON object member"));
            }
            let UniqueValue(value) = map.next_value()?;
            values.insert(key, value);
        }
        Ok(UniqueValue(Value::Object(values)))
    }
}

pub(crate) fn resolve_target_environment<F>(
    scenario: &ActiveScenario,
    mut lookup: F,
) -> Result<Vec<(OsString, OsString)>, ScenarioFailure>
where
    F: FnMut(&str) -> Option<OsString>,
{
    let mut resolved = Vec::new();
    let mut aggregate_bytes = 0_u64;
    let maximum_bytes = DiagnosticLimits::M1_DEFAULTS.values().instance_bytes;
    for (index, name) in scenario.target_environment_names().enumerate() {
        let Some(value) = lookup(name) else {
            return Err(ScenarioFailure::one(Finding::secret_reference_invalid(
                scenario_location()
                    .field(LocationField::TargetEnv)
                    .index(index),
                RuleViolation::MissingEnvironmentValue,
            )));
        };
        let entry_bytes = name
            .len()
            .saturating_add(value.as_os_str().as_encoded_bytes().len());
        aggregate_bytes =
            aggregate_bytes.saturating_add(u64::try_from(entry_bytes).unwrap_or(u64::MAX));
        if aggregate_bytes > maximum_bytes {
            return Err(ScenarioFailure::one(Finding::limit_exceeded(
                SupportedRevision::CURRENT,
                scenario_location().field(LocationField::TargetEnv),
                LimitViolation::new(LimitKind::EnvironmentBytes, aggregate_bytes, maximum_bytes)
                    .expect("the explicit target environment exceeds its byte maximum"),
            )));
        }
        resolved.push((OsString::from(name), value));
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value, json};

    use super::{
        ActiveConversation, ActiveProtocolRevision, ActiveScenario, ArgumentReferenceFailure,
        CaptureFailure, LimitKind, is_url_elicitation_required, server_request_requires_input,
    };

    fn object(value: &Value) -> &Map<String, Value> {
        value.as_object().expect("the fixture should be an object")
    }

    #[test]
    fn url_elicitation_requires_the_exact_bounded_legacy_error_shape() {
        let valid = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "error": {
                "code": -32042,
                "message": "synthetic",
                "data": {
                    "elicitations": [{
                        "mode": "url",
                        "elicitationId": "",
                        "url": "https://synthetic.invalid/continue",
                        "message": "",
                        "_meta": {"progressToken": 7},
                        "task": {"ttl": 1000}
                    }]
                }
            }
        });
        assert!(is_url_elicitation_required(object(&valid)));
        assert!(is_url_elicitation_required(object(&json!({
            "error": {
                "code": -32042,
                "message": "synthetic",
                "data": {"elicitations": []}
            }
        }))));

        for invalid in [
            json!({"error": {"code": -32043, "message": "synthetic", "data": {"elicitations": []}}}),
            json!({"error": {"code": -32042, "message": "synthetic", "data": {}}}),
            json!({"error": {"code": -32042, "message": "synthetic", "data": {"elicitations": [{"mode": "form", "elicitationId": "id", "url": "https://synthetic.invalid", "message": "synthetic"}]}}}),
            json!({"error": {"code": -32042, "message": "synthetic", "data": {"elicitations": [{"mode": "url", "elicitationId": "id", "url": "not a URL", "message": "synthetic"}]}}}),
            json!({"error": {"code": -32042, "message": "synthetic", "data": {"elicitations": [{"mode": "url", "elicitationId": "id", "url": "https://synthetic.invalid", "message": "synthetic", "_meta": []}]}}}),
            json!({"error": {"code": -32042, "message": "synthetic", "data": {"elicitations": [{"mode": "url", "elicitationId": "id", "url": "https://synthetic.invalid", "message": "synthetic", "task": {"ttl": "soon"}}]}}}),
        ] {
            assert!(!is_url_elicitation_required(object(&invalid)));
        }
    }

    #[test]
    fn only_structurally_recognized_additional_input_requests_are_incomplete() {
        for request in [
            json!({
                "method": "elicitation/create",
                "params": {
                    "mode": "url",
                    "message": "synthetic",
                    "elicitationId": "synthetic-id",
                    "url": "https://synthetic.invalid/continue"
                }
            }),
            json!({
                "method": "elicitation/create",
                "params": {
                    "mode": "form",
                    "message": "synthetic",
                    "requestedSchema": {"type": "object", "properties": {}}
                }
            }),
            json!({"method": "sampling/createMessage", "params": {"maxTokens": 1, "messages": []}}),
            json!({"method": "roots/list"}),
        ] {
            assert!(server_request_requires_input(object(&request)));
        }

        for request in [
            json!({"method": "elicitation/create", "params": {"mode": "url"}}),
            json!({"method": "elicitation/create", "params": {"mode": "form", "message": "synthetic", "requestedSchema": {"type": "object"}}}),
            json!({"method": "sampling/createMessage", "params": {"messages": []}}),
            json!({"method": "sampling/createMessage", "params": []}),
            json!({"method": "roots/list", "params": []}),
            json!({"method": "ping", "params": {}}),
        ] {
            assert!(!server_request_requires_input(object(&request)));
        }
    }

    #[test]
    fn workflow_capture_memory_is_bounded_before_retention() {
        let document = json!({
            "schema_version": "mcp-doctor.scenario/v2alpha1",
            "steps": [{
                "id": "capture",
                "tool": "synthetic.capture",
                "safety": {"effects": "read_only"},
                "arguments": {},
                "captures": {"value": "/value"},
                "expect": {"result": "success"}
            }]
        });
        let bytes = serde_json::to_vec(&document).expect("the workflow should serialize");
        let scenario = ActiveScenario::parse(&bytes)
            .unwrap_or_else(|_| panic!("the bounded workflow should parse"));
        let mut conversation =
            ActiveConversation::for_revision(scenario, ActiveProtocolRevision::CURRENT);
        let structured = json!({"value": "x".repeat(1_048_576)});

        let failure = conversation
            .capture_workflow_values(0, Some(&structured))
            .expect_err("the oversized capture must fail before retention");
        assert!(matches!(
            failure,
            CaptureFailure::Limit(violation) if violation.kind() == LimitKind::InstanceBytes
        ));
        assert!(conversation.captured_values.is_empty());
        assert_eq!(conversation.captured_bytes, 0);

        let captures = (0..9)
            .map(|index| (format!("value_{index}"), Value::String("/value".to_owned())))
            .collect::<Map<_, _>>();
        let aggregate_document = json!({
            "schema_version": "mcp-doctor.scenario/v2alpha1",
            "steps": [{
                "id": "capture",
                "tool": "synthetic.capture",
                "safety": {"effects": "read_only"},
                "arguments": {},
                "captures": captures,
                "expect": {"result": "success"}
            }]
        });
        let bytes = serde_json::to_vec(&aggregate_document)
            .expect("the aggregate workflow should serialize");
        let scenario = ActiveScenario::parse(&bytes)
            .unwrap_or_else(|_| panic!("the aggregate workflow should parse"));
        let mut conversation =
            ActiveConversation::for_revision(scenario, ActiveProtocolRevision::CURRENT);
        let structured = json!({"value": "x".repeat(1_000_000)});

        let failure = conversation
            .capture_workflow_values(0, Some(&structured))
            .expect_err("aggregate capture size must fail before retention");
        assert!(matches!(
            failure,
            CaptureFailure::Limit(violation)
                if violation.kind() == LimitKind::AggregateOutputBytes
        ));
        assert!(conversation.captured_values.is_empty());
        assert_eq!(conversation.captured_bytes, 0);
    }

    #[test]
    fn workflow_reference_size_is_proved_before_argument_replacement() {
        let document = json!({
            "schema_version": "mcp-doctor.scenario/v2alpha1",
            "steps": [{
                "id": "capture",
                "tool": "synthetic.capture",
                "safety": {"effects": "read_only"},
                "arguments": {},
                "captures": {"value": "/value"},
                "expect": {"result": "success"}
            }, {
                "id": "consume",
                "tool": "synthetic.consume",
                "safety": {"effects": "read_only"},
                "arguments": {"value": null, "padding": "x".repeat(600_000)},
                "argument_refs": {"/value": "value"},
                "expect": {"result": "success"}
            }]
        });
        let bytes = serde_json::to_vec(&document).expect("the workflow should serialize");
        let mut scenario = ActiveScenario::parse(&bytes)
            .unwrap_or_else(|_| panic!("the bounded workflow should parse"));
        scenario
            .resolve_argument_secrets(|_| None)
            .unwrap_or_else(|_| panic!("the initial workflow arguments should fit"));
        let mut conversation =
            ActiveConversation::for_revision(scenario, ActiveProtocolRevision::CURRENT);
        conversation
            .captured_values
            .insert("value".to_owned(), Value::String("y".repeat(600_000)));

        let failure = conversation
            .resolve_argument_refs(1)
            .expect_err("the resolved arguments must remain within one instance bound");
        assert!(matches!(
            failure,
            ArgumentReferenceFailure::Limit(violation)
                if violation.kind() == LimitKind::InstanceBytes
        ));
        assert_eq!(
            conversation.scenario.cases[1].arguments.pointer("/value"),
            Some(&Value::Null)
        );
        assert!(!conversation.scenario.cases[1].argument_refs.is_empty());
    }

    #[test]
    fn workflow_parser_rejects_future_data_flow_and_non_suffix_cleanup() {
        let future_reference = json!({
            "schema_version": "mcp-doctor.scenario/v2alpha1",
            "steps": [{
                "id": "future",
                "tool": "synthetic.read",
                "safety": {"effects": "read_only"},
                "arguments": {"id": null},
                "argument_refs": {"/id": "later"},
                "expect": {"result": "success"}
            }, {
                "id": "producer",
                "tool": "synthetic.lookup",
                "safety": {"effects": "read_only"},
                "arguments": {},
                "captures": {"later": "/id"},
                "expect": {"result": "success"}
            }]
        });
        let non_suffix_cleanup = json!({
            "schema_version": "mcp-doctor.scenario/v2alpha1",
            "steps": [{
                "id": "main",
                "tool": "synthetic.main",
                "safety": {"effects": "read_only"},
                "arguments": {},
                "expect": {"result": "success"}
            }, {
                "id": "cleanup",
                "tool": "synthetic.cleanup",
                "safety": {"effects": "side_effecting"},
                "cleanup": true,
                "arguments": {},
                "expect": {"result": "success"}
            }, {
                "id": "continued-main",
                "tool": "synthetic.main",
                "safety": {"effects": "read_only"},
                "arguments": {},
                "expect": {"result": "success"}
            }]
        });

        for document in [future_reference, non_suffix_cleanup] {
            let bytes = serde_json::to_vec(&document).expect("the workflow should serialize");
            assert!(ActiveScenario::parse(&bytes).is_err());
        }
    }
}
