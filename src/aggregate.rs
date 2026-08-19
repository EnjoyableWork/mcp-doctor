use std::collections::BTreeSet;
use std::fmt::{self, Write as _};
use std::fs::{self, File};
use std::io::{self, Read as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::bound_file::{BoundFile, BoundFileErrorKind};

const REPORT_SCHEMA: &str = include_str!("../schemas/mcp-doctor.report.v1.schema.json");
pub(crate) const AGGREGATE_SCHEMA_VERSION: &str = "mcp-doctor.aggregate/v1";
const MAXIMUM_INPUTS: usize = 32;
const MAXIMUM_INPUT_BYTES: u64 = 4 * 1024 * 1024;
const MAXIMUM_TOTAL_INPUT_BYTES: u64 = 16 * 1024 * 1024;
const MAXIMUM_JSON_DEPTH: u64 = 64;
const MAXIMUM_JSON_NODES: u64 = 1_000_000;
const MAXIMUM_VALIDATION_WORK: u64 = 1_000_000;
const MAXIMUM_CHECKS: u64 = 4_096;
const MAXIMUM_FINDINGS: u64 = 2_048;
const MAXIMUM_RENDERED_BYTES: u64 = 8 * 1024 * 1024;
const MAXIMUM_OPERATION_TIME: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum AggregateFormat {
    Human,
    Json,
}

#[derive(Debug)]
pub(crate) struct RenderedAggregate {
    pub(crate) stdout: String,
    pub(crate) artifact: String,
    pub(crate) exit: u8,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum AggregateErrorKind {
    InputCount,
    InputUnavailable,
    InputNotRegular,
    InputAlias,
    InputLimit,
    InputMalformed,
    InputSchema,
    InputSemantic,
    OperationLimit,
    EmbeddedSchema,
    Render,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct AggregateError {
    kind: AggregateErrorKind,
    ordinal: Option<usize>,
}

impl AggregateError {
    const fn invocation(kind: AggregateErrorKind) -> Self {
        Self {
            kind,
            ordinal: None,
        }
    }

    const fn input(kind: AggregateErrorKind, ordinal: usize) -> Self {
        Self {
            kind,
            ordinal: Some(ordinal),
        }
    }

    pub(crate) const fn exit_code(self) -> u8 {
        match self.kind {
            AggregateErrorKind::EmbeddedSchema | AggregateErrorKind::Render => 4,
            _ => 2,
        }
    }
}

impl fmt::Display for AggregateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let input = |formatter: &mut fmt::Formatter<'_>, message: &str| {
            write!(
                formatter,
                "aggregate input [{}] {message}",
                self.ordinal
                    .expect("an input error always retains its value-free ordinal")
            )
        };
        match self.kind {
            AggregateErrorKind::InputCount => {
                formatter.write_str("aggregate requires from one through 32 explicit reports")
            }
            AggregateErrorKind::InputUnavailable => input(formatter, "could not be opened safely"),
            AggregateErrorKind::InputNotRegular => {
                input(formatter, "is not a non-symbolic regular file")
            }
            AggregateErrorKind::InputAlias => {
                input(formatter, "aliases an earlier aggregate input")
            }
            AggregateErrorKind::InputLimit => input(formatter, "exceeded an aggregate input limit"),
            AggregateErrorKind::InputMalformed => {
                input(formatter, "is not one bounded JSON document")
            }
            AggregateErrorKind::InputSchema => input(
                formatter,
                "does not satisfy the stable mcp-doctor.report/v1 schema",
            ),
            AggregateErrorKind::InputSemantic => {
                input(formatter, "contains inconsistent diagnostic claims")
            }
            AggregateErrorKind::OperationLimit => formatter.write_str(
                "aggregate exceeded its bounded validation work or ten-second operation limit",
            ),
            AggregateErrorKind::EmbeddedSchema => formatter
                .write_str("the embedded stable diagnostic report schema could not be prepared"),
            AggregateErrorKind::Render => {
                formatter.write_str("the aggregate report could not be rendered within its limit")
            }
        }
    }
}

impl std::error::Error for AggregateError {}

trait Clock {
    fn elapsed(&self) -> Duration;
}

pub(crate) struct AggregateDeadline {
    started: Instant,
}

impl AggregateDeadline {
    pub(crate) fn start() -> Self {
        Self {
            started: Instant::now(),
        }
    }

    pub(crate) fn within_limit(&self) -> bool {
        self.elapsed() <= MAXIMUM_OPERATION_TIME
    }
}

impl Clock for AggregateDeadline {
    fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
}

#[derive(Debug)]
struct OpenedInput {
    ordinal: usize,
    file: File,
}

#[derive(Debug, Default)]
struct WorkBudget {
    nodes: u64,
    work: u64,
    checks: u64,
    findings: u64,
}

impl WorkBudget {
    fn observe_nodes(&mut self, observed: u64) -> Result<(), AggregateError> {
        self.nodes = self
            .nodes
            .checked_add(observed)
            .ok_or_else(operation_limit)?;
        self.observe_work(observed)?;
        if self.nodes > MAXIMUM_JSON_NODES {
            return Err(operation_limit());
        }
        Ok(())
    }

    fn observe_work(&mut self, observed: u64) -> Result<(), AggregateError> {
        self.work = self
            .work
            .checked_add(observed)
            .ok_or_else(operation_limit)?;
        if self.work > MAXIMUM_VALIDATION_WORK {
            return Err(operation_limit());
        }
        Ok(())
    }

    fn observe_report(&mut self, report: &StableReport) -> Result<(), AggregateError> {
        let checks = u64::try_from(report.checks.len()).map_err(|_| operation_limit())?;
        let findings = report.checks.iter().try_fold(0_u64, |total, check| {
            total
                .checked_add(u64::try_from(check.findings.len()).map_err(|_| operation_limit())?)
                .ok_or_else(operation_limit)
        })?;
        self.checks = self
            .checks
            .checked_add(checks)
            .ok_or_else(operation_limit)?;
        self.findings = self
            .findings
            .checked_add(findings)
            .ok_or_else(operation_limit)?;
        self.observe_work(checks.saturating_add(findings))?;
        if self.checks > MAXIMUM_CHECKS || self.findings > MAXIMUM_FINDINGS {
            return Err(operation_limit());
        }
        Ok(())
    }
}

fn operation_limit() -> AggregateError {
    AggregateError::invocation(AggregateErrorKind::OperationLimit)
}

pub(crate) fn run(
    paths: &[PathBuf],
    format: AggregateFormat,
    deadline: &AggregateDeadline,
) -> Result<RenderedAggregate, AggregateError> {
    run_with_clock(paths, format, deadline)
}

fn run_with_clock(
    paths: &[PathBuf],
    format: AggregateFormat,
    clock: &dyn Clock,
) -> Result<RenderedAggregate, AggregateError> {
    if paths.is_empty() || paths.len() > MAXIMUM_INPUTS {
        return Err(AggregateError::invocation(AggregateErrorKind::InputCount));
    }
    check_time(clock)?;

    let mut opened = open_inputs(paths, clock)?;
    let schema: serde_json::Value = serde_json::from_str(REPORT_SCHEMA)
        .map_err(|_| AggregateError::invocation(AggregateErrorKind::EmbeddedSchema))?;
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .map_err(|_| AggregateError::invocation(AggregateErrorKind::EmbeddedSchema))?;
    let mut budget = WorkBudget::default();
    let mut total_bytes = 0_u64;
    let mut reports = Vec::with_capacity(opened.len());

    for input in &mut opened {
        check_time(clock)?;
        let mut bytes = Vec::new();
        input
            .file
            .by_ref()
            .take(MAXIMUM_INPUT_BYTES.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|_| {
                AggregateError::input(AggregateErrorKind::InputUnavailable, input.ordinal)
            })?;
        let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        total_bytes = total_bytes
            .checked_add(length)
            .ok_or_else(|| AggregateError::input(AggregateErrorKind::InputLimit, input.ordinal))?;
        if length > MAXIMUM_INPUT_BYTES || total_bytes > MAXIMUM_TOTAL_INPUT_BYTES {
            return Err(AggregateError::input(
                AggregateErrorKind::InputLimit,
                input.ordinal,
            ));
        }
        scan_json_depth(&bytes, input.ordinal)?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| {
            AggregateError::input(AggregateErrorKind::InputMalformed, input.ordinal)
        })?;
        let nodes = count_json_nodes(&value, input.ordinal)?;
        budget.observe_nodes(nodes).map_err(|error| {
            if error.kind == AggregateErrorKind::OperationLimit {
                AggregateError::input(AggregateErrorKind::InputLimit, input.ordinal)
            } else {
                error
            }
        })?;
        check_time(clock)?;
        if !validator.is_valid(&value) {
            return Err(AggregateError::input(
                AggregateErrorKind::InputSchema,
                input.ordinal,
            ));
        }
        budget
            .observe_work(nodes)
            .map_err(|_| AggregateError::input(AggregateErrorKind::InputLimit, input.ordinal))?;
        let report: StableReport = serde_json::from_value(value)
            .map_err(|_| AggregateError::input(AggregateErrorKind::InputSchema, input.ordinal))?;
        budget
            .observe_report(&report)
            .map_err(|_| AggregateError::input(AggregateErrorKind::InputLimit, input.ordinal))?;
        validate_report(&report, &mut budget).map_err(|error| {
            if error.kind == AggregateErrorKind::OperationLimit {
                AggregateError::input(AggregateErrorKind::InputLimit, input.ordinal)
            } else {
                AggregateError::input(AggregateErrorKind::InputSemantic, input.ordinal)
            }
        })?;
        reports.push(report);
    }
    check_time(clock)?;

    let aggregate = AggregateReport::new(reports);
    let artifact = render_json(&aggregate)?;
    let stdout = match format {
        AggregateFormat::Human => render_human(&aggregate)?,
        AggregateFormat::Json => artifact.clone(),
    };
    let rendered_bytes = u64::try_from(artifact.len())
        .unwrap_or(u64::MAX)
        .checked_add(u64::try_from(stdout.len()).unwrap_or(u64::MAX))
        .ok_or_else(|| AggregateError::invocation(AggregateErrorKind::Render))?;
    if rendered_bytes > MAXIMUM_RENDERED_BYTES {
        return Err(AggregateError::invocation(AggregateErrorKind::Render));
    }
    check_time(clock)?;

    Ok(RenderedAggregate {
        stdout,
        artifact,
        exit: aggregate.exit_code,
    })
}

fn check_time(clock: &dyn Clock) -> Result<(), AggregateError> {
    if clock.elapsed() > MAXIMUM_OPERATION_TIME {
        Err(operation_limit())
    } else {
        Ok(())
    }
}

fn open_inputs(paths: &[PathBuf], clock: &dyn Clock) -> Result<Vec<OpenedInput>, AggregateError> {
    let mut opened = Vec::with_capacity(paths.len());
    let mut identities = BTreeSet::new();
    let mut canonical_paths = BTreeSet::new();
    let mut declared_total = 0_u64;

    for (ordinal, path) in paths.iter().enumerate() {
        check_time(clock)?;
        let bound = BoundFile::open(path).map_err(|error| {
            let kind = match error.kind() {
                BoundFileErrorKind::NotRegular => AggregateErrorKind::InputNotRegular,
                BoundFileErrorKind::Unavailable | BoundFileErrorKind::IdentityChanged => {
                    AggregateErrorKind::InputUnavailable
                }
            };
            AggregateError::input(kind, ordinal)
        })?;
        let canonical = fs::canonicalize(path)
            .map_err(|_| AggregateError::input(AggregateErrorKind::InputUnavailable, ordinal))?;
        if bound.metadata().len() > MAXIMUM_INPUT_BYTES {
            return Err(AggregateError::input(
                AggregateErrorKind::InputLimit,
                ordinal,
            ));
        }
        declared_total = declared_total
            .checked_add(bound.metadata().len())
            .ok_or_else(|| AggregateError::input(AggregateErrorKind::InputLimit, ordinal))?;
        if declared_total > MAXIMUM_TOTAL_INPUT_BYTES {
            return Err(AggregateError::input(
                AggregateErrorKind::InputLimit,
                ordinal,
            ));
        }

        let identity = bound.identity().clone();
        let canonical_key = canonical_path_key(&canonical);
        if !identities.insert(identity) || !canonical_paths.insert(canonical_key) {
            return Err(AggregateError::input(
                AggregateErrorKind::InputAlias,
                ordinal,
            ));
        }
        opened.push(OpenedInput {
            ordinal,
            file: bound.into_file(),
        });
    }
    Ok(opened)
}

#[cfg(windows)]
fn canonical_path_key(path: &Path) -> PathBuf {
    PathBuf::from(path.to_string_lossy().to_lowercase())
}

#[cfg(not(windows))]
fn canonical_path_key(path: &Path) -> PathBuf {
    path.to_path_buf()
}

fn scan_json_depth(bytes: &[u8], ordinal: usize) -> Result<(), AggregateError> {
    let mut depth = 0_u64;
    let mut in_string = false;
    let mut escaped = false;
    for byte in bytes {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match *byte {
            b'"' => in_string = true,
            b'{' | b'[' => {
                depth = depth.saturating_add(1);
                if depth > MAXIMUM_JSON_DEPTH {
                    return Err(AggregateError::input(
                        AggregateErrorKind::InputLimit,
                        ordinal,
                    ));
                }
            }
            b'}' | b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    Ok(())
}

fn count_json_nodes(value: &serde_json::Value, ordinal: usize) -> Result<u64, AggregateError> {
    let mut nodes = 0_u64;
    let mut stack = vec![value];
    while let Some(value) = stack.pop() {
        nodes = nodes.saturating_add(1);
        if nodes > MAXIMUM_JSON_NODES {
            return Err(AggregateError::input(
                AggregateErrorKind::InputLimit,
                ordinal,
            ));
        }
        match value {
            serde_json::Value::Array(values) => stack.extend(values),
            serde_json::Value::Object(values) => stack.extend(values.values()),
            _ => {}
        }
    }
    Ok(nodes)
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum ReportOutcome {
    Passed,
    Failed,
    Incomplete,
}

impl ReportOutcome {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Incomplete => "incomplete",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Requirement {
    Required,
    Optional,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum CheckState {
    Performed,
    Skipped,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum CheckOutcome {
    Passed,
    Warning,
    Failed,
}

impl CheckOutcome {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

impl Severity {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }

    const fn is_failure(self) -> bool {
        matches!(self, Self::Error | Self::Critical)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SkipReason {
    NotAuthorized,
    AuthorizationFailed,
    NotAdvertised,
    InputRequired,
    UnsupportedRevision,
    PrerequisiteFailed,
    LimitReached,
    NotApplicable,
}

impl SkipReason {
    const fn as_str(self) -> &'static str {
        match self {
            Self::NotAuthorized => "not_authorized",
            Self::AuthorizationFailed => "authorization_failed",
            Self::NotAdvertised => "not_advertised",
            Self::InputRequired => "input_required",
            Self::UnsupportedRevision => "unsupported_revision",
            Self::PrerequisiteFailed => "prerequisite_failed",
            Self::LimitReached => "limit_reached",
            Self::NotApplicable => "not_applicable",
        }
    }

    const fn is_causal(self) -> bool {
        matches!(
            self,
            Self::AuthorizationFailed
                | Self::UnsupportedRevision
                | Self::PrerequisiteFailed
                | Self::LimitReached
        )
    }
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
struct StableReport {
    schema_version: String,
    schema_stability: String,
    protocol_revision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    negotiated_protocol_revision: Option<String>,
    primary_diagnosis: Option<Diagnosis>,
    independent_findings: Vec<FindingReference>,
    outcome: ReportOutcome,
    exit_code: u8,
    limits: ReportLimits,
    summary: ReportSummary,
    checks: Vec<StableCheck>,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
struct Diagnosis {
    check_id: String,
    findings: Vec<DiagnosisFinding>,
}

#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct DiagnosisFinding {
    code: String,
    location: String,
}

#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct FindingReference {
    check_id: String,
    code: String,
    location: String,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
struct ReportLimits {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    profile: Option<String>,
    startup_ms: u64,
    discovery_ms: u64,
    request_ms: u64,
    response_ms: u64,
    shutdown_grace_ms: u64,
    total_ms: u64,
    message_bytes: u64,
    stdout_bytes: u64,
    stderr_bytes: u64,
    aggregate_output_bytes: u64,
    endpoint_bytes: u64,
    resolution_addresses: u64,
    resolution_count: u64,
    trust_bytes: u64,
    trust_certificates: u64,
    request_fields: u64,
    request_field_name_bytes: u64,
    request_field_value_bytes: u64,
    request_fields_bytes: u64,
    response_fields: u64,
    response_field_name_bytes: u64,
    response_field_value_bytes: u64,
    response_fields_bytes: u64,
    message_count: u64,
    protocol_revisions: u64,
    catalog_items: u64,
    schema_bytes: u64,
    instance_bytes: u64,
    schema_nodes: u64,
    schema_depth: u64,
    schema_ref_depth: u64,
    schema_evaluation_steps: u64,
    validation_errors: u64,
    report_findings: u64,
    report_bytes: u64,
    active_cases: u64,
    generation_attempts: u64,
    generation_candidates: u64,
    generation_steps: u64,
    redirects: u64,
    retries: u64,
    concurrency: u64,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Eq, PartialEq, Serialize)]
struct SeverityCounts {
    info: u64,
    warning: u64,
    error: u64,
    critical: u64,
}

impl SeverityCounts {
    fn observe(&mut self, severity: Severity) {
        match severity {
            Severity::Info => self.info = self.info.saturating_add(1),
            Severity::Warning => self.warning = self.warning.saturating_add(1),
            Severity::Error => self.error = self.error.saturating_add(1),
            Severity::Critical => self.critical = self.critical.saturating_add(1),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Eq, PartialEq, Serialize)]
struct ReportSummary {
    checks: u64,
    required: u64,
    optional: u64,
    performed: u64,
    skipped: u64,
    passed: u64,
    warned: u64,
    failed: u64,
    required_skipped: u64,
    findings: SeverityCounts,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
struct StableCheck {
    id: String,
    requirement: Requirement,
    state: CheckState,
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome: Option<CheckOutcome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skip_reason: Option<SkipReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocked_by: Option<Diagnosis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reproduction: Option<Reproduction>,
    findings: Vec<StableFinding>,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
struct Reproduction {
    generator: String,
    seed: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    mutation_kind: Option<String>,
    input: StructuralInput,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
struct StructuralInput {
    root: String,
    byte_count: u64,
    node_count: u64,
    maximum_depth: u64,
    nulls: u64,
    booleans: u64,
    numbers: u64,
    strings: u64,
    arrays: u64,
    array_items: u64,
    objects: u64,
    object_members: u64,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
struct StableFinding {
    code: String,
    severity: Severity,
    protocol_revision: String,
    location: String,
    message: String,
    impact: String,
    expectation: String,
    remediation: String,
    reference: String,
    evidence: Evidence,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Evidence {
    None,
    RevisionAdvertisement {
        required: String,
        offered: u64,
        recognized_legacy: u64,
        unknown_date: u64,
        opaque: u64,
    },
    RedactedObservation {
        marker: String,
        byte_count: u64,
    },
    LimitViolation {
        limit: String,
        unit: String,
        observed: u64,
        maximum: u64,
    },
    RuleViolation {
        rule: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        expected: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        observed: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_count: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        http_status: Option<u16>,
    },
    JsonRpcError {
        error_kind: StableJsonRpcErrorKind,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<i64>,
    },
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum StableJsonRpcErrorKind {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    InternalError,
    Other,
}

impl StableJsonRpcErrorKind {
    const fn standard_code(self) -> Option<i64> {
        match self {
            Self::ParseError => Some(-32700),
            Self::InvalidRequest => Some(-32600),
            Self::MethodNotFound => Some(-32601),
            Self::InvalidParams => Some(-32602),
            Self::InternalError => Some(-32603),
            Self::Other => None,
        }
    }
}

fn validate_report(report: &StableReport, budget: &mut WorkBudget) -> Result<(), AggregateError> {
    let mut check_ids = BTreeSet::new();
    let mut summary = ReportSummary {
        checks: u64::try_from(report.checks.len()).map_err(|_| operation_limit())?,
        ..ReportSummary::default()
    };

    for check in &report.checks {
        budget.observe_work(1)?;
        if !check_ids.insert(check.id.as_str()) {
            return Err(AggregateError::invocation(
                AggregateErrorKind::InputSemantic,
            ));
        }
        match check.requirement {
            Requirement::Required => summary.required = summary.required.saturating_add(1),
            Requirement::Optional => summary.optional = summary.optional.saturating_add(1),
        }
        match check.state {
            CheckState::Performed => {
                summary.performed = summary.performed.saturating_add(1);
                let expected = outcome_for_findings(&check.findings);
                if check.outcome != Some(expected)
                    || check.skip_reason.is_some()
                    || check.blocked_by.is_some()
                {
                    return Err(AggregateError::invocation(
                        AggregateErrorKind::InputSemantic,
                    ));
                }
                match expected {
                    CheckOutcome::Passed => summary.passed = summary.passed.saturating_add(1),
                    CheckOutcome::Warning => summary.warned = summary.warned.saturating_add(1),
                    CheckOutcome::Failed => summary.failed = summary.failed.saturating_add(1),
                }
            }
            CheckState::Skipped => {
                summary.skipped = summary.skipped.saturating_add(1);
                if check.requirement == Requirement::Required {
                    summary.required_skipped = summary.required_skipped.saturating_add(1);
                }
                let Some(reason) = check.skip_reason else {
                    return Err(AggregateError::invocation(
                        AggregateErrorKind::InputSemantic,
                    ));
                };
                if check.outcome.is_some() || !check.findings.is_empty() {
                    return Err(AggregateError::invocation(
                        AggregateErrorKind::InputSemantic,
                    ));
                }
                if reason.is_causal() {
                    if check.blocked_by.as_ref() != report.primary_diagnosis.as_ref() {
                        return Err(AggregateError::invocation(
                            AggregateErrorKind::InputSemantic,
                        ));
                    }
                } else if check.blocked_by.is_some() {
                    return Err(AggregateError::invocation(
                        AggregateErrorKind::InputSemantic,
                    ));
                }
            }
        }

        for finding in &check.findings {
            budget.observe_work(1)?;
            if finding.protocol_revision != report.protocol_revision {
                return Err(AggregateError::invocation(
                    AggregateErrorKind::InputSemantic,
                ));
            }
            if let Evidence::RevisionAdvertisement { required, .. } = &finding.evidence
                && required != &report.protocol_revision
            {
                return Err(AggregateError::invocation(
                    AggregateErrorKind::InputSemantic,
                ));
            }
            if let Evidence::JsonRpcError { error_kind, code } = &finding.evidence
                && *code != error_kind.standard_code()
            {
                return Err(AggregateError::invocation(
                    AggregateErrorKind::InputSemantic,
                ));
            }
            summary.findings.observe(finding.severity);
        }
    }

    if summary != report.summary {
        return Err(AggregateError::invocation(
            AggregateErrorKind::InputSemantic,
        ));
    }
    let expected_outcome = if summary.failed > 0 {
        ReportOutcome::Failed
    } else if summary.required == 0 || summary.performed == 0 || summary.required_skipped > 0 {
        ReportOutcome::Incomplete
    } else {
        ReportOutcome::Passed
    };
    if expected_outcome != report.outcome
        || !matches!(
            (report.outcome, report.exit_code),
            (ReportOutcome::Passed, 0)
                | (ReportOutcome::Failed, 1 | 2)
                | (ReportOutcome::Incomplete, 3)
        )
    {
        return Err(AggregateError::invocation(
            AggregateErrorKind::InputSemantic,
        ));
    }

    match (&report.primary_diagnosis, report.outcome) {
        (Some(diagnosis), ReportOutcome::Failed) => {
            for finding in &diagnosis.findings {
                budget.observe_work(1)?;
                if !reference_resolves(
                    report,
                    &diagnosis.check_id,
                    &finding.code,
                    &finding.location,
                    true,
                ) {
                    return Err(AggregateError::invocation(
                        AggregateErrorKind::InputSemantic,
                    ));
                }
            }
        }
        (None, ReportOutcome::Passed | ReportOutcome::Incomplete) => {}
        _ => {
            return Err(AggregateError::invocation(
                AggregateErrorKind::InputSemantic,
            ));
        }
    }

    if report
        .negotiated_protocol_revision
        .as_ref()
        .is_some_and(|revision| revision != &report.protocol_revision)
        && !report.primary_diagnosis.as_ref().is_some_and(|diagnosis| {
            diagnosis.check_id == "protocol.revision"
                && diagnosis
                    .findings
                    .iter()
                    .any(|finding| finding.code == "MCP-PROTOCOL-005")
        })
    {
        return Err(AggregateError::invocation(
            AggregateErrorKind::InputSemantic,
        ));
    }

    for finding in &report.independent_findings {
        budget.observe_work(1)?;
        if !reference_resolves(
            report,
            &finding.check_id,
            &finding.code,
            &finding.location,
            true,
        ) {
            return Err(AggregateError::invocation(
                AggregateErrorKind::InputSemantic,
            ));
        }
    }
    Ok(())
}

fn outcome_for_findings(findings: &[StableFinding]) -> CheckOutcome {
    if findings.iter().any(|finding| finding.severity.is_failure()) {
        CheckOutcome::Failed
    } else if findings
        .iter()
        .any(|finding| finding.severity == Severity::Warning)
    {
        CheckOutcome::Warning
    } else {
        CheckOutcome::Passed
    }
}

fn reference_resolves(
    report: &StableReport,
    check_id: &str,
    code: &str,
    location: &str,
    require_failure: bool,
) -> bool {
    report
        .checks
        .iter()
        .find(|check| check.id == check_id)
        .is_some_and(|check| {
            check.findings.iter().any(|finding| {
                finding.code == code
                    && finding.location == location
                    && (!require_failure || finding.severity.is_failure())
            })
        })
}

#[derive(Debug, Serialize)]
struct AggregateReport {
    schema_version: &'static str,
    schema_stability: &'static str,
    outcome: ReportOutcome,
    exit_code: u8,
    limits: AggregateLimits,
    summary: AggregateSummary,
    members: Vec<AggregateMember>,
}

impl AggregateReport {
    fn new(reports: Vec<StableReport>) -> Self {
        let mut summary = AggregateSummary {
            members: u64::try_from(reports.len()).expect("at most 32 members fit u64"),
            ..AggregateSummary::default()
        };
        for report in &reports {
            match report.outcome {
                ReportOutcome::Passed => summary.passed += 1,
                ReportOutcome::Failed => summary.failed += 1,
                ReportOutcome::Incomplete => summary.incomplete += 1,
            }
        }
        let outcome = if summary.failed > 0 {
            ReportOutcome::Failed
        } else if summary.incomplete > 0 {
            ReportOutcome::Incomplete
        } else {
            ReportOutcome::Passed
        };
        let exit_code = match outcome {
            ReportOutcome::Passed => 0,
            ReportOutcome::Failed => 1,
            ReportOutcome::Incomplete => 3,
        };
        Self {
            schema_version: AGGREGATE_SCHEMA_VERSION,
            schema_stability: "stable",
            outcome,
            exit_code,
            limits: AggregateLimits::contract(),
            summary,
            members: reports
                .into_iter()
                .enumerate()
                .map(|(ordinal, report)| AggregateMember { ordinal, report })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct AggregateMember {
    ordinal: usize,
    report: StableReport,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct AggregateLimits {
    input_files: u64,
    input_file_bytes: u64,
    input_bytes: u64,
    json_depth: u64,
    json_nodes: u64,
    validation_work: u64,
    checks: u64,
    findings: u64,
    rendered_bytes: u64,
    total_ms: u64,
    retries: u64,
    concurrency: u64,
}

impl AggregateLimits {
    const fn contract() -> Self {
        Self {
            input_files: MAXIMUM_INPUTS as u64,
            input_file_bytes: MAXIMUM_INPUT_BYTES,
            input_bytes: MAXIMUM_TOTAL_INPUT_BYTES,
            json_depth: MAXIMUM_JSON_DEPTH,
            json_nodes: MAXIMUM_JSON_NODES,
            validation_work: MAXIMUM_VALIDATION_WORK,
            checks: MAXIMUM_CHECKS,
            findings: MAXIMUM_FINDINGS,
            rendered_bytes: MAXIMUM_RENDERED_BYTES,
            total_ms: MAXIMUM_OPERATION_TIME.as_millis() as u64,
            retries: 0,
            concurrency: 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
struct AggregateSummary {
    members: u64,
    passed: u64,
    failed: u64,
    incomplete: u64,
}

struct BoundedOutput {
    bytes: Vec<u8>,
    maximum: usize,
    exceeded: bool,
}

impl BoundedOutput {
    fn new(maximum: u64) -> Self {
        Self {
            bytes: Vec::new(),
            maximum: usize::try_from(maximum).unwrap_or(usize::MAX),
            exceeded: false,
        }
    }

    fn append(&mut self, bytes: &[u8]) {
        let Some(length) = self.bytes.len().checked_add(bytes.len()) else {
            self.exceeded = true;
            return;
        };
        if length > self.maximum {
            self.exceeded = true;
            return;
        }
        self.bytes.extend_from_slice(bytes);
    }

    fn finish(self) -> Result<String, AggregateError> {
        if self.exceeded {
            return Err(AggregateError::invocation(AggregateErrorKind::Render));
        }
        String::from_utf8(self.bytes)
            .map_err(|_| AggregateError::invocation(AggregateErrorKind::Render))
    }
}

impl io::Write for BoundedOutput {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.append(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl fmt::Write for BoundedOutput {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.append(value.as_bytes());
        Ok(())
    }
}

fn render_json(report: &AggregateReport) -> Result<String, AggregateError> {
    let mut output = BoundedOutput::new(MAXIMUM_RENDERED_BYTES);
    serde_json::to_writer_pretty(&mut output, report)
        .map_err(|_| AggregateError::invocation(AggregateErrorKind::Render))?;
    output.append(b"\n");
    output.finish()
}

fn render_human(report: &AggregateReport) -> Result<String, AggregateError> {
    let mut output = BoundedOutput::new(MAXIMUM_RENDERED_BYTES);
    writeln!(output, "mcp-doctor aggregate · {AGGREGATE_SCHEMA_VERSION}")
        .expect("the bounded aggregate writer records failures");
    writeln!(
        output,
        "{} members · conservative fail > incomplete > pass",
        report.summary.members
    )
    .expect("the bounded aggregate writer records failures");

    for member in &report.members {
        let member_report = &member.report;
        writeln!(
            output,
            "\nMEMBER [{}] · MCP {} · outcome {} · exit {}",
            member.ordinal,
            member_report.protocol_revision,
            member_report.outcome.as_str(),
            member_report.exit_code
        )
        .expect("the bounded aggregate writer records failures");
        if let Some(negotiated) = &member_report.negotiated_protocol_revision {
            writeln!(output, "  negotiated revision · {negotiated}")
                .expect("the bounded aggregate writer records failures");
        }
        if let Some(primary) = &member_report.primary_diagnosis {
            writeln!(output, "  PRIMARY DIAGNOSIS · {}", primary.check_id)
                .expect("the bounded aggregate writer records failures");
            for finding in &primary.findings {
                writeln!(output, "    {} · {}", finding.code, finding.location)
                    .expect("the bounded aggregate writer records failures");
            }
        } else {
            writeln!(output, "  PRIMARY DIAGNOSIS · none")
                .expect("the bounded aggregate writer records failures");
        }
        if !member_report.independent_findings.is_empty() {
            writeln!(
                output,
                "  INDEPENDENT SAFETY FINDINGS · {}",
                member_report.independent_findings.len()
            )
            .expect("the bounded aggregate writer records failures");
            for finding in &member_report.independent_findings {
                writeln!(
                    output,
                    "    {} · {} · {}",
                    finding.code, finding.check_id, finding.location
                )
                .expect("the bounded aggregate writer records failures");
            }
        }
        for check in &member_report.checks {
            match check.state {
                CheckState::Performed => {
                    writeln!(
                        output,
                        "  {} · {} · {}",
                        check.id,
                        check
                            .outcome
                            .expect("a validated performed check has an outcome")
                            .as_str(),
                        match check.requirement {
                            Requirement::Required => "required",
                            Requirement::Optional => "optional",
                        }
                    )
                    .expect("the bounded aggregate writer records failures");
                    for finding in &check.findings {
                        writeln!(
                            output,
                            "    {} · {} · {} · fix: {}",
                            finding.code,
                            finding.severity.as_str(),
                            finding.location,
                            finding.remediation
                        )
                        .expect("the bounded aggregate writer records failures");
                    }
                }
                CheckState::Skipped => {
                    write!(
                        output,
                        "  {} · skipped · {}",
                        check.id,
                        check
                            .skip_reason
                            .expect("a validated skipped check has a reason")
                            .as_str()
                    )
                    .expect("the bounded aggregate writer records failures");
                    if let Some(blocked) = &check.blocked_by {
                        write!(output, " · blocked by {}", blocked.check_id)
                            .expect("the bounded aggregate writer records failures");
                    }
                    writeln!(output).expect("the bounded aggregate writer records failures");
                }
            }
        }
    }

    writeln!(
        output,
        "\n{} failed · {} incomplete · {} passed · outcome {} · exit {}",
        report.summary.failed,
        report.summary.incomplete,
        report.summary.passed,
        report.outcome.as_str(),
        report.exit_code
    )
    .expect("the bounded aggregate writer records failures");
    output.finish()
}

#[cfg(test)]
mod tests {
    use super::open_inputs;
    use super::{
        AggregateErrorKind, AggregateFormat, Clock, MAXIMUM_OPERATION_TIME, run_with_clock,
    };
    use std::cell::Cell;
    use std::io::Read as _;
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::TempDir;

    struct AdvancingClock {
        reads: Cell<u64>,
        fail_after: u64,
    }

    impl Clock for AdvancingClock {
        fn elapsed(&self) -> Duration {
            let reads = self.reads.get().saturating_add(1);
            self.reads.set(reads);
            if reads > self.fail_after {
                MAXIMUM_OPERATION_TIME.saturating_add(Duration::from_nanos(1))
            } else {
                Duration::ZERO
            }
        }
    }

    #[test]
    fn injected_clock_fails_without_sleeping_or_retrying() {
        let clock = AdvancingClock {
            reads: Cell::new(0),
            fail_after: 0,
        };
        let error = run_with_clock(
            &[PathBuf::from("synthetic-input-must-not-be-opened")],
            AggregateFormat::Human,
            &clock,
        )
        .expect_err("the injected deadline should fail before input activity");
        assert_eq!(error.kind, AggregateErrorKind::OperationLimit);
        assert_eq!(clock.reads.get(), 1);
    }

    #[test]
    fn injected_clock_stops_completed_validation_before_rendering() {
        let root = TempDir::new().expect("a disposable root should be created");
        let path = root.path().join("report.json");
        std::fs::write(
            &path,
            include_bytes!("../tests/fixtures/aggregates/passed-report.json"),
        )
        .unwrap();
        let clock = AdvancingClock {
            reads: Cell::new(0),
            fail_after: 4,
        };
        let error = run_with_clock(&[path], AggregateFormat::Human, &clock)
            .expect_err("the injected deadline should fail before rendering");

        assert_eq!(error.kind, AggregateErrorKind::OperationLimit);
        assert_eq!(clock.reads.get(), 5);
    }

    #[test]
    fn input_reads_remain_bound_to_the_identity_opened_during_preflight() {
        let root = TempDir::new().expect("a disposable root should be created");
        let path = root.path().join("input.json");
        let moved = root.path().join("opened.json");
        std::fs::write(&path, b"opened identity").unwrap();
        let clock = AdvancingClock {
            reads: Cell::new(0),
            fail_after: u64::MAX,
        };
        let mut inputs = open_inputs(std::slice::from_ref(&path), &clock)
            .expect("the original regular file should open");

        std::fs::rename(&path, moved).unwrap();
        std::fs::write(&path, b"replacement path").unwrap();
        let mut retained = String::new();
        inputs[0].file.read_to_string(&mut retained).unwrap();

        assert_eq!(retained, "opened identity");
    }
}
