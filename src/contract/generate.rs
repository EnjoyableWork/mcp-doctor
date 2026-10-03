use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};

use serde::ser::{Serialize, SerializeMap, Serializer};
use serde_json::{Map, Number, Value};

use super::catalog::{InstanceValidationIssue, LocalValidator, resolve_local_reference_with_work};
use super::limits::{DiagnosticLimits, LimitKind, LimitViolation};
use super::model::{GeneratedCaseReproduction, JsonKind, StructuralInput};

pub(crate) const GENERATOR_VERSION: &str = "mcp-doctor.generator/v1";

pub(super) const INVALID_MUTATION_KINDS: [InvalidMutationKind; 7] = [
    InvalidMutationKind::MissingArguments,
    InvalidMutationKind::WrongRootType,
    InvalidMutationKind::OmittedRequiredProperty,
    InvalidMutationKind::WrongPropertyType,
    InvalidMutationKind::ForbiddenNull,
    InvalidMutationKind::InvalidEnum,
    InvalidMutationKind::UnexpectedProperty,
];

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum InvalidMutationKind {
    MissingArguments,
    WrongRootType,
    OmittedRequiredProperty,
    WrongPropertyType,
    ForbiddenNull,
    InvalidEnum,
    UnexpectedProperty,
}

impl InvalidMutationKind {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::MissingArguments => "missing_arguments",
            Self::WrongRootType => "wrong_root_type",
            Self::OmittedRequiredProperty => "omitted_required_property",
            Self::WrongPropertyType => "wrong_property_type",
            Self::ForbiddenNull => "forbidden_null",
            Self::InvalidEnum => "invalid_enum",
            Self::UnexpectedProperty => "unexpected_property",
        }
    }
}

pub(super) struct GeneratedInput {
    pub(super) arguments: Value,
    pub(super) reproduction: GeneratedCaseReproduction,
}

pub(super) struct GeneratedInvalidInput {
    pub(super) arguments: Value,
    pub(super) omit_arguments: bool,
    pub(super) reproduction: GeneratedCaseReproduction,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum GenerationFailure {
    Limit(LimitViolation),
    Unavailable,
}

struct InputByteBudget {
    used: u64,
    maximum: u64,
}

impl InputByteBudget {
    const fn new(maximum: u64) -> Self {
        Self { used: 0, maximum }
    }

    fn check(&self, additional: u64) -> Result<u64, GenerationFailure> {
        let observed = self.used.saturating_add(additional);
        if observed > self.maximum {
            return Err(instance_byte_limit(observed, self.maximum));
        }
        Ok(observed)
    }

    fn reserve(&mut self, additional: u64) -> Result<(), GenerationFailure> {
        self.used = self.check(additional)?;
        Ok(())
    }

    fn reserve_json<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), GenerationFailure> {
        let remaining = self.maximum.saturating_sub(self.used);
        let measured = measure_json(value, remaining, false).map_err(|failure| match failure {
            GenerationFailure::Limit(violation) => {
                instance_byte_limit(self.used.saturating_add(violation.observed()), self.maximum)
            }
            failure => failure,
        })?;
        self.reserve(measured.bytes)
    }

    fn clone_value(&mut self, value: &Value) -> Result<Value, GenerationFailure> {
        // Schema-owned const, enum, and example values can be large aggregates.
        // Count them without allocation before cloning any member.
        self.reserve_json(value)?;
        Ok(value.clone())
    }

    fn reserve_member(&mut self, name: &str, has_members: bool) -> Result<(), GenerationFailure> {
        self.reserve_json(name)?;
        self.reserve(1 + u64::from(has_members))
    }
}

fn instance_byte_limit(observed: u64, maximum: u64) -> GenerationFailure {
    GenerationFailure::Limit(
        LimitViolation::new(LimitKind::InstanceBytes, observed, maximum)
            .expect("generated input bytes exceed their maximum"),
    )
}

struct JsonMeasurement {
    bytes: u64,
    maximum: u64,
    violation: Option<LimitViolation>,
    hash: Option<(u64, u64)>,
}

impl JsonMeasurement {
    fn identity(&self) -> (u64, u64) {
        let (first, second) = self.hash.expect("identity measurement enables hashing");
        (first ^ self.bytes, second ^ self.bytes.rotate_left(32))
    }
}

impl Write for JsonMeasurement {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let next = self
            .bytes
            .saturating_add(u64::try_from(bytes.len()).unwrap_or(u64::MAX));
        if next > self.maximum {
            self.violation = Some(
                LimitViolation::new(LimitKind::InstanceBytes, next, self.maximum)
                    .expect("serialized generated input exceeds its maximum"),
            );
            return Err(io::Error::other("generated input byte allowance exhausted"));
        }
        if let Some((first, second)) = &mut self.hash {
            for byte in bytes {
                *first ^= u64::from(*byte);
                *first = first.wrapping_mul(0x0000_0100_0000_01b3);
                *second = stable_mix(*second ^ u64::from(*byte));
            }
        }
        self.bytes = next;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn measure_json<T: Serialize + ?Sized>(
    value: &T,
    maximum: u64,
    identity: bool,
) -> Result<JsonMeasurement, GenerationFailure> {
    let mut measurement = JsonMeasurement {
        bytes: 0,
        maximum,
        violation: None,
        hash: identity.then_some((0xcbf2_9ce4_8422_2325, 0x6a09_e667_f3bc_c909)),
    };
    if serde_json::to_writer(&mut measurement, value).is_err() {
        return Err(measurement
            .violation
            .map_or(GenerationFailure::Unavailable, GenerationFailure::Limit));
    }
    Ok(measurement)
}

struct MutatedObject<'a> {
    object: &'a Map<String, Value>,
    name: &'a str,
    replacement: Option<&'a Value>,
}

impl Serialize for MutatedObject<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        for (name, value) in self.object {
            if name != self.name {
                map.serialize_entry(name, value)?;
            }
        }
        if let Some(value) = self.replacement {
            map.serialize_entry(self.name, value)?;
        }
        map.end()
    }
}

fn mutated_object(
    base: &Value,
    name: &str,
    replacement: Option<&Value>,
    maximum: u64,
) -> Result<Value, GenerationFailure> {
    let object = base.as_object().ok_or(GenerationFailure::Unavailable)?;
    // Measure the final mutation through borrowed members before cloning any
    // retained payload, property name, or replacement. JSON member order does
    // not affect this exact size check; replaced payloads are never cloned.
    measure_json(
        &MutatedObject {
            object,
            name,
            replacement,
        },
        maximum,
        false,
    )?;
    let mut output = Map::new();
    for (key, value) in object {
        if key != name {
            output.insert(key.clone(), value.clone());
        }
    }
    if let Some(value) = replacement {
        output.insert(name.to_owned(), value.clone());
    }
    Ok(Value::Object(output))
}

pub(super) fn generate_inputs(
    schema: &Value,
    validator: &LocalValidator,
    base_seed: u64,
    case_count: usize,
) -> Result<Vec<GeneratedInput>, GenerationFailure> {
    let limits = DiagnosticLimits::DEFAULTS.values();
    if case_count == 0 {
        return Err(GenerationFailure::Unavailable);
    }
    let observed_cases = u64::try_from(case_count).unwrap_or(u64::MAX);
    if observed_cases > limits.active_cases {
        return Err(GenerationFailure::Limit(
            LimitViolation::new(LimitKind::ActiveCases, observed_cases, limits.active_cases)
                .expect("the generated case count exceeds its maximum"),
        ));
    }
    let maximum_attempts = usize::try_from(limits.generation_attempts).unwrap_or(usize::MAX);
    let maximum_candidates = usize::try_from(limits.generation_candidates).unwrap_or(usize::MAX);
    let mut generation_steps = 0_u64;
    let mut candidates = Vec::new();
    let mut identities = BTreeSet::new();
    let mut candidate_bytes = 0_u64;

    // Candidate construction is independent of the run seed so a reported
    // case seed can reproduce that case as a one-case run.
    for attempt in 0..maximum_attempts {
        if candidates.len() >= maximum_candidates {
            break;
        }
        let attempt = u64::try_from(attempt).unwrap_or(u64::MAX);
        let mut synthesizer = Synthesizer::new(
            schema,
            stable_mix(attempt ^ 0xd1b5_4a32_d192_ed03),
            &mut generation_steps,
        );
        let Some(candidate) = synthesizer.value(schema, 0)? else {
            continue;
        };
        if !candidate.is_object() {
            continue;
        }
        // Stream the bounded candidate into its fixed-size identity. Retaining
        // a second complete serialization would spend the instance allowance
        // again solely for deduplication.
        let measured = measure_json(&candidate, limits.instance_bytes, true)?;
        let observed = measured.bytes;
        debug_assert_eq!(observed, synthesizer.bytes.used);
        if !identities.insert(measured.identity()) {
            continue;
        }
        match validator.validate(&candidate) {
            Ok(()) => {}
            Err(InstanceValidationIssue::Mismatch { .. }) => continue,
            Err(InstanceValidationIssue::Limit(violation)) => {
                return Err(GenerationFailure::Limit(violation));
            }
            Err(InstanceValidationIssue::InvalidSchema) => {
                return Err(GenerationFailure::Unavailable);
            }
        }
        let next_bytes = candidate_bytes.saturating_add(observed);
        if next_bytes > limits.aggregate_output_bytes {
            break;
        }
        candidate_bytes = next_bytes;
        candidates.push((candidate, observed));
    }

    if candidates.is_empty() {
        return Err(GenerationFailure::Unavailable);
    }

    select_generated_inputs(
        &candidates,
        base_seed,
        case_count,
        limits.aggregate_output_bytes,
    )
}

fn select_generated_inputs(
    candidates: &[(Value, u64)],
    base_seed: u64,
    case_count: usize,
    maximum_input_bytes: u64,
) -> Result<Vec<GeneratedInput>, GenerationFailure> {
    let mut aggregate_input_bytes = 0_u64;
    let mut generated = Vec::with_capacity(case_count);
    for index in 0..case_count {
        let case_seed = base_seed.wrapping_add(u64::try_from(index).unwrap_or(u64::MAX));
        let candidate_index = bounded_index(stable_mix(case_seed), candidates.len());
        let (arguments, byte_count) = &candidates[candidate_index];
        aggregate_input_bytes = aggregate_input_bytes.saturating_add(*byte_count);
        if aggregate_input_bytes > maximum_input_bytes {
            return Err(GenerationFailure::Limit(
                LimitViolation::new(
                    LimitKind::ActiveInputBytes,
                    aggregate_input_bytes,
                    maximum_input_bytes,
                )
                .expect("aggregate generated inputs exceed their maximum"),
            ));
        }
        generated.push(GeneratedInput {
            arguments: arguments.clone(),
            reproduction: GeneratedCaseReproduction::new(
                GENERATOR_VERSION,
                case_seed,
                structural_input(arguments, *byte_count),
            ),
        });
    }
    Ok(generated)
}

pub(super) fn generate_invalid_inputs(
    schema: &Value,
    validator: &LocalValidator,
    seed: u64,
) -> Result<Vec<Option<GeneratedInvalidInput>>, GenerationFailure> {
    // A rejection probe starts from evidence that the advertised schema has at
    // least one bounded, locally valid object instance. Without that baseline,
    // a rejection could be caused by an unsatisfiable schema rather than the
    // one mutation named in the report.
    let pool = generate_inputs(schema, validator, seed, INVALID_MUTATION_KINDS.len())?;
    let mut aggregate_bytes = 0_u64;
    let mut generated = Vec::with_capacity(INVALID_MUTATION_KINDS.len());
    for kind in INVALID_MUTATION_KINDS {
        let candidate = invalid_candidate(schema, validator, &pool, kind)?;
        let Some((arguments, omit_arguments)) = candidate else {
            generated.push(None);
            continue;
        };
        let limits = DiagnosticLimits::DEFAULTS.values();
        let bytes = measure_json(&arguments, limits.instance_bytes, false)?.bytes;
        aggregate_bytes = aggregate_bytes.saturating_add(bytes);
        if aggregate_bytes > limits.aggregate_output_bytes {
            return Err(GenerationFailure::Limit(
                LimitViolation::new(
                    LimitKind::ActiveInputBytes,
                    aggregate_bytes,
                    limits.aggregate_output_bytes,
                )
                .expect("aggregate invalid inputs exceed their maximum"),
            ));
        }
        generated.push(Some(GeneratedInvalidInput {
            reproduction: GeneratedCaseReproduction::new(
                GENERATOR_VERSION,
                seed,
                structural_input(&arguments, bytes),
            )
            .with_mutation_kind(kind.as_str()),
            arguments,
            omit_arguments,
        }));
    }
    if generated.iter().all(Option::is_none) {
        return Err(GenerationFailure::Unavailable);
    }
    Ok(generated)
}

fn invalid_candidate(
    schema: &Value,
    validator: &LocalValidator,
    pool: &[GeneratedInput],
    kind: InvalidMutationKind,
) -> Result<Option<(Value, bool)>, GenerationFailure> {
    match kind {
        InvalidMutationKind::MissingArguments => {
            let arguments = Value::Object(Map::new());
            exact_mismatch(validator, &arguments).map(|valid| valid.then_some((arguments, true)))
        }
        InvalidMutationKind::WrongRootType => {
            let arguments = Value::Array(Vec::new());
            exact_mismatch(validator, &arguments).map(|valid| valid.then_some((arguments, false)))
        }
        InvalidMutationKind::OmittedRequiredProperty => {
            omitted_required_property(schema, validator, pool)
        }
        InvalidMutationKind::WrongPropertyType => wrong_property_type(schema, validator, pool),
        InvalidMutationKind::ForbiddenNull => forbidden_null(schema, validator, pool),
        InvalidMutationKind::InvalidEnum => invalid_enum(schema, validator, pool),
        InvalidMutationKind::UnexpectedProperty => unexpected_property(schema, validator, pool),
    }
}

fn omitted_required_property(
    schema: &Value,
    validator: &LocalValidator,
    pool: &[GeneratedInput],
) -> Result<Option<(Value, bool)>, GenerationFailure> {
    let Some(required) = schema
        .as_object()
        .and_then(|object| object.get("required"))
        .and_then(Value::as_array)
    else {
        return Ok(None);
    };
    for name in required.iter().filter_map(Value::as_str) {
        for base in pool {
            let Some(object) = base.arguments.as_object() else {
                continue;
            };
            if !object.contains_key(name) {
                continue;
            }
            let arguments = mutated_object(
                &base.arguments,
                name,
                None,
                DiagnosticLimits::DEFAULTS.values().instance_bytes,
            )?;
            if exact_mismatch(validator, &arguments)? {
                return Ok(Some((arguments, false)));
            }
        }
    }
    Ok(None)
}

fn wrong_property_type(
    schema: &Value,
    validator: &LocalValidator,
    pool: &[GeneratedInput],
) -> Result<Option<(Value, bool)>, GenerationFailure> {
    let Some(properties) = schema
        .as_object()
        .and_then(|object| object.get("properties"))
        .and_then(Value::as_object)
    else {
        return Ok(None);
    };
    let replacements = [
        Value::Bool(false),
        Value::Number(Number::from(0)),
        Value::String(String::new()),
        Value::Array(Vec::new()),
        Value::Object(Map::new()),
    ];
    for base in pool {
        let Some(object) = base.arguments.as_object() else {
            continue;
        };
        for (name, current) in object {
            let Some(property_schema) = properties.get(name) else {
                continue;
            };
            for replacement in &replacements {
                if json_kind(current) == json_kind(replacement) {
                    continue;
                }
                if declared_type_allows(property_schema, replacement) != Some(false) {
                    continue;
                }
                let arguments = mutated_object(
                    &base.arguments,
                    name,
                    Some(replacement),
                    DiagnosticLimits::DEFAULTS.values().instance_bytes,
                )?;
                if exact_mismatch(validator, &arguments)? {
                    return Ok(Some((arguments, false)));
                }
            }
        }
    }
    Ok(None)
}

fn forbidden_null(
    schema: &Value,
    validator: &LocalValidator,
    pool: &[GeneratedInput],
) -> Result<Option<(Value, bool)>, GenerationFailure> {
    let Some(properties) = schema
        .as_object()
        .and_then(|object| object.get("properties"))
        .and_then(Value::as_object)
    else {
        return Ok(None);
    };
    for base in pool {
        let Some(object) = base.arguments.as_object() else {
            continue;
        };
        for (name, current) in object {
            if current.is_null()
                || properties
                    .get(name)
                    .is_none_or(|schema| declared_type_allows(schema, &Value::Null) != Some(false))
            {
                continue;
            }
            let arguments = mutated_object(
                &base.arguments,
                name,
                Some(&Value::Null),
                DiagnosticLimits::DEFAULTS.values().instance_bytes,
            )?;
            if exact_mismatch(validator, &arguments)? {
                return Ok(Some((arguments, false)));
            }
        }
    }
    Ok(None)
}

fn invalid_enum(
    schema: &Value,
    validator: &LocalValidator,
    pool: &[GeneratedInput],
) -> Result<Option<(Value, bool)>, GenerationFailure> {
    let Some(properties) = schema
        .as_object()
        .and_then(|object| object.get("properties"))
        .and_then(Value::as_object)
    else {
        return Ok(None);
    };
    let alternatives = [
        Value::String("mcp-doctor-invalid-enum".to_owned()),
        Value::Number(Number::from(0)),
        Value::Number(Number::from(1)),
        Value::Bool(false),
        Value::Bool(true),
        Value::Null,
        Value::Array(Vec::new()),
        Value::Array(vec![Value::Null]),
        Value::Object(Map::new()),
        Value::Object(Map::from_iter([(
            "mcp_doctor_enum".to_owned(),
            Value::Bool(false),
        )])),
    ];
    for (name, property_schema) in properties {
        let Some(values) = property_schema
            .as_object()
            .and_then(|object| object.get("enum"))
            .and_then(Value::as_array)
        else {
            continue;
        };
        for base in pool {
            if !base
                .arguments
                .as_object()
                .is_some_and(|object| object.contains_key(name))
            {
                continue;
            }
            for example in values {
                for replacement in alternatives.iter().filter(|value| {
                    json_kind(value) == json_kind(example) && !values.contains(value)
                }) {
                    let arguments = mutated_object(
                        &base.arguments,
                        name,
                        Some(replacement),
                        DiagnosticLimits::DEFAULTS.values().instance_bytes,
                    )?;
                    if exact_mismatch(validator, &arguments)? {
                        return Ok(Some((arguments, false)));
                    }
                }
            }
        }
    }
    Ok(None)
}

fn unexpected_property(
    schema: &Value,
    validator: &LocalValidator,
    pool: &[GeneratedInput],
) -> Result<Option<(Value, bool)>, GenerationFailure> {
    if schema
        .as_object()
        .and_then(|object| object.get("additionalProperties"))
        != Some(&Value::Bool(false))
    {
        return Ok(None);
    }
    let declared = schema
        .as_object()
        .and_then(|object| object.get("properties"))
        .and_then(Value::as_object);
    for base in pool {
        let Some(object) = base.arguments.as_object() else {
            continue;
        };
        let mut index = 0_u64;
        let name = loop {
            let name = format!("mcp_doctor_unexpected_{index}");
            if !object.contains_key(&name)
                && !declared.is_some_and(|properties| properties.contains_key(&name))
            {
                break name;
            }
            index = index.saturating_add(1);
            if index > DiagnosticLimits::DEFAULTS.values().generation_attempts {
                return Ok(None);
            }
        };
        let arguments = mutated_object(
            &base.arguments,
            &name,
            Some(&Value::Bool(false)),
            DiagnosticLimits::DEFAULTS.values().instance_bytes,
        )?;
        if exact_mismatch(validator, &arguments)? {
            return Ok(Some((arguments, false)));
        }
    }
    Ok(None)
}

fn declared_type_allows(schema: &Value, value: &Value) -> Option<bool> {
    let declared = schema.as_object()?.get("type")?;
    let allows = |name: &str| match (name, value) {
        ("null", Value::Null)
        | ("boolean", Value::Bool(_))
        | ("string", Value::String(_))
        | ("array", Value::Array(_))
        | ("object", Value::Object(_))
        | ("number", Value::Number(_)) => true,
        ("integer", Value::Number(number)) => {
            number.as_i64().is_some() || number.as_u64().is_some()
        }
        _ => false,
    };
    match declared {
        Value::String(name) => Some(allows(name)),
        Value::Array(names) => Some(names.iter().filter_map(Value::as_str).any(allows)),
        _ => None,
    }
}

fn exact_mismatch(validator: &LocalValidator, value: &Value) -> Result<bool, GenerationFailure> {
    match validator.validate(value) {
        Ok(()) => Ok(false),
        Err(InstanceValidationIssue::Mismatch { error_count }) => Ok(error_count == 1),
        Err(InstanceValidationIssue::Limit(violation)) => Err(GenerationFailure::Limit(violation)),
        Err(InstanceValidationIssue::InvalidSchema) => Err(GenerationFailure::Unavailable),
    }
}

struct Synthesizer<'root, 'budget> {
    root: &'root Value,
    random: StableRandom,
    steps: &'budget mut u64,
    active_references: BTreeSet<&'root str>,
    bytes: InputByteBudget,
}

impl<'root, 'budget> Synthesizer<'root, 'budget> {
    fn new(root: &'root Value, seed: u64, steps: &'budget mut u64) -> Self {
        Self {
            root,
            random: StableRandom(seed),
            steps,
            active_references: BTreeSet::new(),
            bytes: InputByteBudget::new(DiagnosticLimits::DEFAULTS.values().instance_bytes),
        }
    }

    fn value(
        &mut self,
        schema: &'root Value,
        depth: u64,
    ) -> Result<Option<Value>, GenerationFailure> {
        self.tick()?;
        let limits = DiagnosticLimits::DEFAULTS.values();
        if depth > limits.schema_depth {
            return Err(GenerationFailure::Limit(
                LimitViolation::new(LimitKind::SchemaDepth, depth, limits.schema_depth)
                    .expect("generated input depth exceeds its maximum"),
            ));
        }

        match schema {
            Value::Bool(false) => return Ok(None),
            Value::Bool(true) => return self.generic_value().map(Some),
            Value::Object(object) => {
                if let Some(value) = object.get("const") {
                    return self.bytes.clone_value(value).map(Some);
                }
                if let Some(values) = object.get("enum").and_then(Value::as_array) {
                    if values.is_empty() {
                        return Ok(None);
                    }
                    let selected = &values[self.choose(values.len())];
                    return self.bytes.clone_value(selected).map(Some);
                }
                if self.choose(4) == 0
                    && let Some(value) = declared_example(object, self.next_u64())
                {
                    return self.bytes.clone_value(value).map(Some);
                }

                if let Some(reference) = object
                    .get("$ref")
                    .or_else(|| object.get("$dynamicRef"))
                    .and_then(Value::as_str)
                {
                    charge_reference_set(reference, self.active_references.len(), self.steps)?;
                    if self.active_references.insert(reference) {
                        let resolved =
                            resolve_generation_reference(self.root, reference, self.steps)?;
                        let generated = match resolved {
                            Some(target) => self.value(target, depth.saturating_add(1))?,
                            None => None,
                        };
                        charge_reference_set(reference, self.active_references.len(), self.steps)?;
                        self.active_references.remove(reference);
                        if generated.is_some() {
                            return Ok(generated);
                        }
                    }
                }

                if self.choose(3) == 0
                    && let Some(branch) = selected_branch(object, &mut self.random)
                    && let Some(value) = self.value(branch, depth.saturating_add(1))?
                {
                    return Ok(Some(value));
                }

                let kinds = schema_kinds(object);
                let kind = kinds
                    .get(self.choose(kinds.len()))
                    .copied()
                    .unwrap_or(ValueKind::Object);
                return match kind {
                    ValueKind::Null => {
                        self.bytes.reserve(4)?;
                        Ok(Some(Value::Null))
                    }
                    ValueKind::Boolean => {
                        let value = self.choose(2) == 1;
                        self.bytes.reserve(if value { 4 } else { 5 })?;
                        Ok(Some(Value::Bool(value)))
                    }
                    ValueKind::Integer => self.number(object, true).map(Some),
                    ValueKind::Number => self.number(object, false).map(Some),
                    ValueKind::String => {
                        self.string(object).map(|value| Some(Value::String(value)))
                    }
                    ValueKind::Array => self
                        .array(object, depth)
                        .map(|value| Some(Value::Array(value))),
                    ValueKind::Object => self
                        .object(schema, depth)
                        .map(|value| Some(Value::Object(value))),
                };
            }
            _ => {}
        }
        Ok(None)
    }

    fn object(
        &mut self,
        schema: &'root Value,
        depth: u64,
    ) -> Result<Map<String, Value>, GenerationFailure> {
        self.bytes.reserve(2)?;
        let mut plan = ObjectPlan::default();
        let mut references = BTreeSet::new();
        collect_object_plan(
            self.root,
            schema,
            &mut plan,
            &mut references,
            &mut self.random,
            self.steps,
        )?;

        let include_mode = self.choose(3);
        let mut selected = plan.required.clone();
        for name in plan.properties.keys() {
            if selected.contains(name) {
                continue;
            }
            if include_mode == 1 || (include_mode == 2 && self.choose(2) == 1) {
                selected.insert(*name);
            }
        }
        for (trigger, dependents) in &plan.dependent_required {
            if selected.contains(trigger) {
                selected.extend(dependents.iter().copied());
            }
        }

        let minimum = usize::try_from(plan.minimum_properties).unwrap_or(usize::MAX);
        for name in plan.properties.keys() {
            if selected.len() >= minimum {
                break;
            }
            selected.insert(*name);
        }
        if let Some(maximum) = plan.maximum_properties {
            let maximum = usize::try_from(maximum).unwrap_or(usize::MAX);
            if selected.len() > maximum {
                let required = &plan.required;
                selected.retain(|name| required.contains(name));
            }
        }

        let mut output = Map::new();
        for name in selected {
            self.tick()?;
            let before_member = self.bytes.used;
            self.bytes.reserve_member(name, !output.is_empty())?;
            let generated = if let Some(schemas) = plan.properties.get(name) {
                let selected_schema = schemas[self.choose(schemas.len())];
                self.value(selected_schema, depth.saturating_add(1))?
            } else if let Some(schema) = plan.additional_schema {
                self.value(schema, depth.saturating_add(1))?
            } else {
                Some(self.generic_value()?)
            };
            if let Some(value) = generated {
                output.insert(name.to_owned(), value);
            } else {
                self.bytes.used = before_member;
            }
        }

        let mut generated_index = 0_u64;
        while output.len() < minimum {
            self.tick()?;
            if plan.additional_forbidden {
                break;
            }
            let before_member = self.bytes.used;
            // This ASCII name has no escaping expansion. Reserve its quotes,
            // colon, separator, and digits before formatting it.
            let name_bytes = u64::try_from("mcp_doctor_generated_".len()).unwrap()
                + decimal_digits(generated_index);
            self.bytes
                .reserve(name_bytes + 3 + u64::from(!output.is_empty()))?;
            let name = format!("mcp_doctor_generated_{generated_index}");
            generated_index = generated_index.saturating_add(1);
            if output.contains_key(&name) || plan.properties.contains_key(name.as_str()) {
                self.bytes.used = before_member;
                continue;
            }
            let value = if let Some(schema) = plan.additional_schema {
                match self.value(schema, depth.saturating_add(1))? {
                    Some(value) => value,
                    None => self.generic_value()?,
                }
            } else {
                self.generic_value()?
            };
            output.insert(name, value);
        }
        Ok(output)
    }

    fn array(
        &mut self,
        object: &'root Map<String, Value>,
        depth: u64,
    ) -> Result<Vec<Value>, GenerationFailure> {
        let limits = DiagnosticLimits::DEFAULTS.values();
        let minimum = integer_keyword(object, "minItems").unwrap_or(0);
        let maximum = integer_keyword(object, "maxItems");
        if minimum > limits.generation_steps {
            return Err(GenerationFailure::Limit(
                LimitViolation::new(LimitKind::GenerationSteps, minimum, limits.generation_steps)
                    .expect("the required generated collection work exceeds its maximum"),
            ));
        }
        let mut lengths = vec![minimum, minimum.saturating_add(1), 0, 1, 2];
        if let Some(maximum) = maximum {
            lengths.push(maximum);
            lengths.push(maximum.saturating_sub(1));
        }
        lengths.sort_unstable();
        lengths.dedup();
        lengths.retain(|length| *length >= minimum && maximum.is_none_or(|max| *length <= max));
        let length = lengths
            .get(self.choose(lengths.len()))
            .copied()
            .unwrap_or(minimum);
        if length > limits.generation_steps {
            return Err(GenerationFailure::Limit(
                LimitViolation::new(LimitKind::GenerationSteps, length, limits.generation_steps)
                    .expect("the generated collection work exceeds its maximum"),
            ));
        }

        let prefix = object
            .get("prefixItems")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let item_schema = object.get("items");
        let capacity = usize::try_from(length).unwrap_or(usize::MAX);
        // Every JSON item takes at least one byte, plus a separator after the
        // first. Reject an impossible collection before allocating its vector,
        // then reserve each actual member before construction and insertion.
        self.bytes.check(2 + length + length.saturating_sub(1))?;
        self.bytes.reserve(2)?;
        let mut output = Vec::new();
        for index in 0..capacity {
            self.tick()?;
            if index > 0 {
                self.bytes.reserve(1)?;
            }
            let schema = prefix.get(index).or(item_schema);
            let value = match schema {
                Some(schema) => match self.value(schema, depth.saturating_add(1))? {
                    Some(value) => value,
                    None => self.generic_value()?,
                },
                None => self.generic_value()?,
            };
            output.push(value);
        }
        Ok(output)
    }

    fn string(&mut self, object: &'root Map<String, Value>) -> Result<String, GenerationFailure> {
        let limits = DiagnosticLimits::DEFAULTS.values();
        let minimum = integer_keyword(object, "minLength").unwrap_or(0);
        let maximum = integer_keyword(object, "maxLength");
        if minimum.saturating_add(2) > limits.instance_bytes {
            return Err(GenerationFailure::Limit(
                LimitViolation::new(
                    LimitKind::InstanceBytes,
                    minimum.saturating_add(2),
                    limits.instance_bytes,
                )
                .expect("the required generated string exceeds the instance maximum"),
            ));
        }

        let examples = [
            "",
            "a",
            "A",
            "0",
            "test",
            "synthetic-boundary",
            "00000000-0000-4000-8000-000000000000",
            "test@example.invalid",
            "https://example.invalid/",
        ];
        let mut lengths = vec![minimum, minimum.saturating_add(1), 0, 1, 2, 8, 32, 255];
        if let Some(maximum) = maximum {
            lengths.push(maximum);
            lengths.push(maximum.saturating_sub(1));
        }
        lengths.sort_unstable();
        lengths.dedup();
        lengths.retain(|length| {
            *length >= minimum
                && maximum.is_none_or(|max| *length <= max)
                && length.saturating_add(2) <= limits.instance_bytes
        });
        let length = usize::try_from(
            lengths
                .get(self.choose(lengths.len()))
                .copied()
                .unwrap_or(minimum),
        )
        .unwrap_or(usize::MAX);
        let example = examples[self.choose(examples.len())];
        // Every generated character is unescaped ASCII, so this is the exact
        // JSON string length. Charge the shared candidate allowance first.
        self.bytes
            .reserve(u64::try_from(length).unwrap_or(u64::MAX).saturating_add(2))?;
        let mut value = String::with_capacity(length);
        value.extend(example.chars().take(length));
        let remaining = length.saturating_sub(value.chars().count());
        value.extend(std::iter::repeat_n('a', remaining));
        Ok(value)
    }

    fn number(
        &mut self,
        object: &'root Map<String, Value>,
        integer: bool,
    ) -> Result<Value, GenerationFailure> {
        let mut values = vec![
            Number::from(0),
            Number::from(1),
            Number::from(-1),
            Number::from(i32::MAX),
            Number::from(i32::MIN),
        ];
        for key in ["minimum", "maximum", "exclusiveMinimum", "exclusiveMaximum"] {
            if let Some(number) = object.get(key).and_then(Value::as_number) {
                values.push(number.clone());
                if let Some(value) = number.as_i64() {
                    if let Some(value) = value.checked_add(1) {
                        values.push(Number::from(value));
                    }
                    if let Some(value) = value.checked_sub(1) {
                        values.push(Number::from(value));
                    }
                } else if !integer && let Some(value) = number.as_f64() {
                    for adjacent in [value + f64::EPSILON, value - f64::EPSILON] {
                        if let Some(number) = Number::from_f64(adjacent) {
                            values.push(number);
                        }
                    }
                }
            }
        }
        if let Some(multiple) = object.get("multipleOf").and_then(Value::as_number) {
            values.push(multiple.clone());
            if let Some(value) = multiple.as_f64()
                && let Some(value) = Number::from_f64(value * 2.0)
            {
                values.push(value);
            }
        }
        values.sort_by_key(ToString::to_string);
        values.dedup();
        let selected = values
            .get(self.choose(values.len()))
            .cloned()
            .ok_or(GenerationFailure::Unavailable)?;
        self.bytes.reserve_json(&selected)?;
        Ok(Value::Number(selected))
    }

    fn generic_value(&mut self) -> Result<Value, GenerationFailure> {
        let selected = self.choose(7);
        let bytes = match selected {
            0 => 4,
            1 => 5,
            2 => 1,
            3..=5 => 2,
            _ => 20,
        };
        self.bytes.reserve(bytes)?;
        Ok(match selected {
            0 => Value::Null,
            1 => Value::Bool(false),
            2 => Value::Number(Number::from(0)),
            3 => Value::String(String::new()),
            4 => Value::Array(Vec::new()),
            5 => Value::Object(Map::new()),
            _ => Value::String("synthetic-boundary".to_owned()),
        })
    }

    fn tick(&mut self) -> Result<(), GenerationFailure> {
        tick(self.steps)
    }

    fn next_u64(&mut self) -> u64 {
        self.random.next()
    }

    fn choose(&mut self, length: usize) -> usize {
        self.random.choose(length)
    }
}

#[derive(Default)]
struct ObjectPlan<'root> {
    properties: BTreeMap<&'root str, Vec<&'root Value>>,
    required: BTreeSet<&'root str>,
    dependent_required: BTreeMap<&'root str, BTreeSet<&'root str>>,
    minimum_properties: u64,
    maximum_properties: Option<u64>,
    additional_forbidden: bool,
    additional_schema: Option<&'root Value>,
}

fn collect_object_plan<'root>(
    root: &'root Value,
    schema: &'root Value,
    plan: &mut ObjectPlan<'root>,
    active_references: &mut BTreeSet<&'root str>,
    random: &mut StableRandom,
    steps: &mut u64,
) -> Result<(), GenerationFailure> {
    tick(steps)?;
    let Some(object) = schema.as_object() else {
        return Ok(());
    };
    if let Some(properties) = object.get("properties").and_then(Value::as_object) {
        for (name, schema) in properties {
            tick(steps)?;
            plan.properties
                .entry(name.as_str())
                .or_default()
                .push(schema);
        }
    }
    if let Some(required) = object.get("required").and_then(Value::as_array) {
        plan.required
            .extend(required.iter().filter_map(Value::as_str));
    }
    if let Some(dependencies) = object.get("dependentRequired").and_then(Value::as_object) {
        for (trigger, values) in dependencies {
            let target = plan.dependent_required.entry(trigger.as_str()).or_default();
            target.extend(
                values
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str),
            );
        }
    }
    plan.minimum_properties = plan
        .minimum_properties
        .max(integer_keyword(object, "minProperties").unwrap_or(0));
    if let Some(maximum) = integer_keyword(object, "maxProperties") {
        plan.maximum_properties = Some(
            plan.maximum_properties
                .map_or(maximum, |current| current.min(maximum)),
        );
    }
    match object.get("additionalProperties") {
        Some(Value::Bool(false)) => plan.additional_forbidden = true,
        Some(schema @ (Value::Bool(true) | Value::Object(_))) => {
            plan.additional_schema.get_or_insert(schema);
        }
        _ => {}
    }

    if let Some(reference) = object
        .get("$ref")
        .or_else(|| object.get("$dynamicRef"))
        .and_then(Value::as_str)
    {
        charge_reference_set(reference, active_references.len(), steps)?;
        if active_references.insert(reference) {
            if let Some(target) = resolve_generation_reference(root, reference, steps)? {
                collect_object_plan(root, target, plan, active_references, random, steps)?;
            }
            charge_reference_set(reference, active_references.len(), steps)?;
            active_references.remove(reference);
        }
    }
    if let Some(branches) = object.get("allOf").and_then(Value::as_array) {
        for branch in branches {
            collect_object_plan(root, branch, plan, active_references, random, steps)?;
        }
    }
    for keyword in ["anyOf", "oneOf"] {
        if let Some(branches) = object.get(keyword).and_then(Value::as_array)
            && !branches.is_empty()
        {
            let branch = &branches[random.choose(branches.len())];
            collect_object_plan(root, branch, plan, active_references, random, steps)?;
        }
    }
    if let Some(branch) = if random.choose(2) == 0 {
        object.get("then")
    } else {
        object.get("else")
    } {
        collect_object_plan(root, branch, plan, active_references, random, steps)?;
    }
    Ok(())
}

fn selected_branch<'a>(
    object: &'a Map<String, Value>,
    random: &mut StableRandom,
) -> Option<&'a Value> {
    for keyword in ["anyOf", "oneOf", "allOf"] {
        if let Some(branches) = object.get(keyword).and_then(Value::as_array)
            && !branches.is_empty()
        {
            return branches.get(random.choose(branches.len()));
        }
    }
    if random.choose(2) == 0 {
        object.get("then")
    } else {
        object.get("else")
    }
}

fn declared_example(object: &Map<String, Value>, selector: u64) -> Option<&Value> {
    if selector.is_multiple_of(2)
        && let Some(default) = object.get("default")
    {
        return Some(default);
    }
    let examples = object.get("examples")?.as_array()?;
    (!examples.is_empty()).then(|| &examples[bounded_index(selector, examples.len())])
}

#[derive(Clone, Copy)]
enum ValueKind {
    Null,
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
}

fn schema_kinds(object: &Map<String, Value>) -> Vec<ValueKind> {
    let mut kinds = Vec::new();
    match object.get("type") {
        Some(Value::String(value)) => push_kind(&mut kinds, value),
        Some(Value::Array(values)) => {
            for value in values.iter().filter_map(Value::as_str) {
                push_kind(&mut kinds, value);
            }
        }
        _ => {}
    }
    if !kinds.is_empty() {
        return kinds;
    }
    if object.keys().any(|key| {
        matches!(
            key.as_str(),
            "properties" | "required" | "additionalProperties" | "minProperties" | "maxProperties"
        )
    }) {
        return vec![ValueKind::Object];
    }
    if object.keys().any(|key| {
        matches!(
            key.as_str(),
            "items" | "prefixItems" | "minItems" | "maxItems" | "contains"
        )
    }) {
        return vec![ValueKind::Array];
    }
    if object.keys().any(|key| {
        matches!(
            key.as_str(),
            "minLength" | "maxLength" | "pattern" | "format"
        )
    }) {
        return vec![ValueKind::String];
    }
    if object.keys().any(|key| {
        matches!(
            key.as_str(),
            "minimum" | "maximum" | "exclusiveMinimum" | "exclusiveMaximum" | "multipleOf"
        )
    }) {
        return vec![ValueKind::Number];
    }
    vec![
        ValueKind::Null,
        ValueKind::Boolean,
        ValueKind::Integer,
        ValueKind::Number,
        ValueKind::String,
        ValueKind::Array,
        ValueKind::Object,
    ]
}

fn push_kind(kinds: &mut Vec<ValueKind>, value: &str) {
    let kind = match value {
        "null" => ValueKind::Null,
        "boolean" => ValueKind::Boolean,
        "integer" => ValueKind::Integer,
        "number" => ValueKind::Number,
        "string" => ValueKind::String,
        "array" => ValueKind::Array,
        "object" => ValueKind::Object,
        _ => return,
    };
    kinds.push(kind);
}

fn integer_keyword(object: &Map<String, Value>, keyword: &str) -> Option<u64> {
    object.get(keyword).and_then(Value::as_u64)
}

fn resolve_generation_reference<'a>(
    root: &'a Value,
    reference: &str,
    steps: &mut u64,
) -> Result<Option<&'a Value>, GenerationFailure> {
    let maximum = DiagnosticLimits::DEFAULTS.values().generation_steps;
    resolve_local_reference_with_work(root, reference, steps, maximum).map_err(|violation| {
        GenerationFailure::Limit(
            LimitViolation::new(LimitKind::GenerationSteps, violation.observed(), maximum)
                .expect("local reference resolution exhausted the generation work allowance"),
        )
    })
}

const fn decimal_digits(mut value: u64) -> u64 {
    let mut digits = 1;
    while value >= 10 {
        digits += 1;
        value /= 10;
    }
    digits
}

fn tick(steps: &mut u64) -> Result<(), GenerationFailure> {
    charge_generation_steps(steps, 1)
}

fn charge_reference_set(
    reference: &str,
    members: usize,
    steps: &mut u64,
) -> Result<(), GenerationFailure> {
    let comparison_bytes = u64::try_from(reference.len()).unwrap_or(u64::MAX);
    let comparisons = u64::try_from(members).unwrap_or(u64::MAX).saturating_add(1);
    charge_generation_steps(steps, comparison_bytes.saturating_mul(comparisons))
}

fn charge_generation_steps(steps: &mut u64, additional: u64) -> Result<(), GenerationFailure> {
    let maximum = DiagnosticLimits::DEFAULTS.values().generation_steps;
    *steps = steps.saturating_add(additional);
    if *steps > maximum {
        return Err(GenerationFailure::Limit(
            LimitViolation::new(LimitKind::GenerationSteps, *steps, maximum)
                .expect("generation work exceeds its maximum"),
        ));
    }
    Ok(())
}

fn structural_input(value: &Value, byte_count: u64) -> StructuralInput {
    let mut nodes = 0_u64;
    let mut maximum_depth = 0_u64;
    let mut nulls = 0_u64;
    let mut booleans = 0_u64;
    let mut numbers = 0_u64;
    let mut strings = 0_u64;
    let mut arrays = 0_u64;
    let mut array_items = 0_u64;
    let mut objects = 0_u64;
    let mut object_members = 0_u64;
    let mut stack = vec![(value, 0_u64)];
    while let Some((value, depth)) = stack.pop() {
        nodes = nodes.saturating_add(1);
        maximum_depth = maximum_depth.max(depth);
        match value {
            Value::Null => nulls = nulls.saturating_add(1),
            Value::Bool(_) => booleans = booleans.saturating_add(1),
            Value::Number(_) => numbers = numbers.saturating_add(1),
            Value::String(_) => strings = strings.saturating_add(1),
            Value::Array(values) => {
                arrays = arrays.saturating_add(1);
                array_items =
                    array_items.saturating_add(u64::try_from(values.len()).unwrap_or(u64::MAX));
                stack.extend(
                    values
                        .iter()
                        .rev()
                        .map(|value| (value, depth.saturating_add(1))),
                );
            }
            Value::Object(values) => {
                objects = objects.saturating_add(1);
                object_members =
                    object_members.saturating_add(u64::try_from(values.len()).unwrap_or(u64::MAX));
                stack.extend(
                    values
                        .values()
                        .rev()
                        .map(|value| (value, depth.saturating_add(1))),
                );
            }
        }
    }
    StructuralInput::new(
        json_kind(value),
        byte_count,
        nodes,
        maximum_depth,
        nulls,
        booleans,
        numbers,
        strings,
        arrays,
        array_items,
        objects,
        object_members,
    )
}

const fn json_kind(value: &Value) -> JsonKind {
    match value {
        Value::Null => JsonKind::Null,
        Value::Bool(_) => JsonKind::Boolean,
        Value::Number(_) => JsonKind::Number,
        Value::String(_) => JsonKind::String,
        Value::Array(_) => JsonKind::Array,
        Value::Object(_) => JsonKind::Object,
    }
}

struct StableRandom(u64);

impl StableRandom {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        stable_mix(self.0)
    }

    fn choose(&mut self, length: usize) -> usize {
        if length == 0 {
            return 0;
        }
        bounded_index(self.next(), length)
    }
}

fn bounded_index(value: u64, length: usize) -> usize {
    debug_assert!(length > 0);
    let length = u64::try_from(length).unwrap_or(u64::MAX);
    usize::try_from(value % length).unwrap_or(0)
}

#[cfg(test)]
fn candidate_identity(bytes: &[u8]) -> (u64, u64) {
    // The fixed-size identity bounds deduplication memory. A collision can
    // reduce coverage, but can never admit an unvalidated candidate.
    let mut first = 0xcbf2_9ce4_8422_2325_u64;
    let mut second = 0x6a09_e667_f3bc_c909_u64;
    for byte in bytes {
        first ^= u64::from(*byte);
        first = first.wrapping_mul(0x0000_0100_0000_01b3);
        second = stable_mix(second ^ u64::from(*byte));
    }
    let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    (first ^ length, second ^ length.rotate_left(32))
}

const fn stable_mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        GenerationFailure, InputByteBudget, StableRandom, Synthesizer, candidate_identity,
        declared_example, generate_inputs, generate_invalid_inputs, measure_json, mutated_object,
        resolve_generation_reference, select_generated_inputs,
    };
    use crate::contract::catalog::{InstanceValidationIssue, LocalValidator};
    use crate::contract::limits::LimitKind;

    #[test]
    fn reference_resolution_stops_at_the_existing_generation_work_ceiling() {
        let schema = json!({"$anchor": "node", "type": "object"});
        let maximum = crate::contract::limits::DiagnosticLimits::DEFAULTS
            .values()
            .generation_steps;
        let mut steps = maximum - 10;
        assert!(matches!(
            resolve_generation_reference(&schema, "#node", &mut steps),
            Err(GenerationFailure::Limit(violation))
                if violation.kind() == LimitKind::GenerationSteps
                    && violation.observed() == maximum + 1
                    && violation.maximum() == maximum
        ));
        assert_eq!(steps, maximum + 1);
    }

    #[test]
    fn aggregate_candidate_bytes_stop_before_another_string_is_constructed() {
        let schema = json!({
            "type": "array",
            "minItems": 3,
            "maxItems": 3,
            "items": {"type": "string", "minLength": 8, "maxLength": 8}
        });
        let mut steps = 0;
        let mut synthesizer = Synthesizer::new(&schema, 7, &mut steps);
        synthesizer.bytes = InputByteBudget::new(25);
        let result = synthesizer.value(&schema, 0);
        assert!(matches!(
            result,
            Err(GenerationFailure::Limit(violation))
                if violation.kind() == LimitKind::InstanceBytes
                    && violation.observed() == 34
                    && violation.maximum() == 25
        ));
        // Two eight-byte strings, quotes, the array delimiters, and both
        // separators fit. The third string's reservation fails before its
        // String allocation; traversal stops at that member.
        assert_eq!(synthesizer.bytes.used, 24);
        assert_eq!(steps, 7);
    }

    #[test]
    fn required_collection_lower_bound_stops_before_vector_allocation() {
        let schema = json!({
            "type": "array",
            "minItems": 20,
            "maxItems": 20,
            "items": {"type": "null"}
        });
        let mut steps = 0;
        let mut synthesizer = Synthesizer::new(&schema, 7, &mut steps);
        synthesizer.bytes = InputByteBudget::new(32);
        assert!(matches!(
            synthesizer.value(&schema, 0),
            Err(GenerationFailure::Limit(violation))
                if violation.kind() == LimitKind::InstanceBytes
                    && violation.observed() == 41
        ));
        assert_eq!(synthesizer.bytes.used, 0);
        assert_eq!(steps, 1);
    }

    #[test]
    fn borrowed_values_are_counted_exactly_before_const_enum_and_example_clones() {
        let value = json!({"escaped\"key\n": ["quote\" slash\\ line\n", "é🦀", {"nested": null}]});
        let encoded = serde_json::to_vec(&value).unwrap();
        let exact = u64::try_from(encoded.len()).unwrap();
        let schemas = [
            json!({"const": value}),
            json!({"enum": [value]}),
            json!({"examples": [value]}),
            json!({"default": value}),
        ];
        for schema in &schemas {
            let seed = if schema.get("const").is_some() || schema.get("enum").is_some() {
                7
            } else {
                (0..128)
                    .find(|seed| {
                        let mut random = StableRandom(*seed);
                        random.choose(4) == 0
                            && declared_example(schema.as_object().unwrap(), random.next())
                                .is_some()
                    })
                    .expect("one fixed bounded seed should select the declared example")
            };
            let mut steps = 0;
            let mut exact_synthesizer = Synthesizer::new(schema, seed, &mut steps);
            exact_synthesizer.bytes = InputByteBudget::new(exact);
            let generated = exact_synthesizer.value(schema, 0).unwrap().unwrap();
            assert_eq!(generated, value);
            assert_eq!(exact_synthesizer.bytes.used, exact);
            let mut steps = 0;
            let mut limited = Synthesizer::new(schema, seed, &mut steps);
            limited.bytes = InputByteBudget::new(exact - 1);
            assert!(matches!(
                limited.value(schema, 0),
                Err(GenerationFailure::Limit(violation))
                    if violation.kind() == LimitKind::InstanceBytes
                        && violation.maximum() == exact - 1
            ));
            assert_eq!(limited.bytes.used, 0);
        }
    }

    #[test]
    fn object_keys_and_nested_members_share_one_exact_byte_allowance() {
        let key = "escaped\"key\n\\é";
        let schema = json!({
            "type": "object",
            "properties": {key: {"const": ["é", {"nested": true}]}},
            "required": [key],
            "additionalProperties": false
        });
        let expected = json!({key: ["é", {"nested": true}]});
        let exact = u64::try_from(serde_json::to_vec(&expected).unwrap().len()).unwrap();
        let mut steps = 0;
        let mut synthesizer = Synthesizer::new(&schema, 7, &mut steps);
        synthesizer.bytes = InputByteBudget::new(exact);
        assert_eq!(synthesizer.value(&schema, 0).unwrap().unwrap(), expected);
        assert_eq!(synthesizer.bytes.used, exact);
        let mut steps = 0;
        let mut limited = Synthesizer::new(&schema, 7, &mut steps);
        limited.bytes = InputByteBudget::new(exact - 1);
        assert!(matches!(
            limited.value(&schema, 0),
            Err(GenerationFailure::Limit(_))
        ));
        assert!(limited.bytes.used < exact);
    }

    #[test]
    fn generated_string_and_budget_zero_and_overflow_boundaries_are_checked_first() {
        let schema = json!({"type": "string", "minLength": 30, "maxLength": 30});
        let mut steps = 0;
        let mut synthesizer = Synthesizer::new(&schema, 7, &mut steps);
        synthesizer.bytes = InputByteBudget::new(32);
        assert_eq!(
            synthesizer
                .value(&schema, 0)
                .unwrap()
                .unwrap()
                .as_str()
                .unwrap()
                .len(),
            30
        );
        assert_eq!(synthesizer.bytes.used, 32);

        for (maximum, additional) in [(0, 1), (32, u64::MAX)] {
            let mut budget = InputByteBudget::new(maximum);
            assert!(matches!(
                budget.reserve(additional),
                Err(GenerationFailure::Limit(_))
            ));
            assert_eq!(budget.used, 0);
        }
        let mut budget = InputByteBudget::new(32);
        budget.reserve(1).unwrap();
        assert!(matches!(
            budget.reserve(u64::MAX),
            Err(GenerationFailure::Limit(_))
        ));
        assert_eq!(budget.used, 1);
        assert!(matches!(
            measure_json(&json!(null), 0, false),
            Err(GenerationFailure::Limit(_))
        ));
    }

    #[test]
    fn mutations_preflight_final_members_without_cloning_replaced_payloads() {
        let base = json!({"payload": "a".repeat(512), "keep": true});
        let replacement = json!("quote\"\n");
        let expected = json!({"payload": replacement, "keep": true});
        let exact = u64::try_from(serde_json::to_vec(&expected).unwrap().len()).unwrap();
        let mutated = mutated_object(&base, "payload", Some(&replacement), exact).unwrap();
        assert_eq!(mutated, expected);
        assert!(matches!(
            mutated_object(&base, "payload", Some(&replacement), exact - 1),
            Err(GenerationFailure::Limit(violation)) if violation.maximum() == exact - 1
        ));
        assert_eq!(
            mutated_object(&base, "payload", None, 13).unwrap(),
            json!({"keep": true})
        );
        assert!(matches!(
            mutated_object(&json!({}), "escaped\"key", Some(&replacement), 8),
            Err(GenerationFailure::Limit(_))
        ));
    }

    #[test]
    fn streaming_candidate_identity_matches_the_existing_generator_identity() {
        for value in [
            json!({}),
            json!({"escaped\"key\n": ["é🦀", true, 3.5, null]}),
        ] {
            let encoded = serde_json::to_vec(&value).unwrap();
            let exact = u64::try_from(encoded.len()).unwrap();
            let measured = measure_json(&value, exact, true).unwrap();
            assert_eq!(measured.bytes, exact);
            assert_eq!(measured.identity(), candidate_identity(&encoded));
            assert!(matches!(
                measure_json(&value, exact - 1, true),
                Err(GenerationFailure::Limit(_))
            ));
        }
    }

    #[test]
    fn generated_inputs_are_deterministic_schema_valid_and_value_free_in_reproduction() {
        let schema = json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "minLength": 1, "maxLength": 8},
                "limit": {"type": "integer", "minimum": 1, "maximum": 5},
                "flags": {
                    "type": "array",
                    "items": {"type": "boolean"},
                    "minItems": 1,
                    "maxItems": 2
                }
            },
            "required": ["query", "limit"],
            "additionalProperties": false
        });
        let validator = LocalValidator::compile(&schema).expect("the schema should compile");

        let first = generate_inputs(&schema, &validator, 4242, 12)
            .expect("the common object schema should generate");
        let second = generate_inputs(&schema, &validator, 4242, 12)
            .expect("the same seed should generate again");

        assert_eq!(first.len(), 12);
        assert!(
            first
                .iter()
                .all(|case| validator.validate(&case.arguments).is_ok())
        );
        for (left, right) in first.iter().zip(&second) {
            assert_eq!(left.arguments, right.arguments);
            assert_eq!(left.reproduction, right.reproduction);
            assert_eq!(left.reproduction.input().root().as_str(), "object");
        }
        assert_eq!(
            first[0].arguments,
            json!({"flags": [false, false], "limit": 4, "query": "ht"}),
            "generator/v1 seed 4242 changed without a version change"
        );
        assert_eq!(
            first[11].arguments,
            json!({"flags": [true, true], "limit": 2, "query": "testaaaa"}),
            "generator/v1 seed 4253 changed without a version change"
        );
        let safe_debug = format!("{:?}", first[0].reproduction);
        assert!(!safe_debug.contains("query"));
        assert!(!safe_debug.contains("limit"));
    }

    #[test]
    fn invalid_inputs_are_deterministic_and_each_has_one_proven_structural_mismatch() {
        let schema = json!({
            "type": "object",
            "properties": {
                "mode": {"type": "string", "enum": ["safe", "strict"]},
                "count": {"type": "integer", "minimum": 1, "maximum": 5}
            },
            "required": ["count"],
            "additionalProperties": false
        });
        let validator = LocalValidator::compile(&schema).expect("the schema should compile");
        let first = generate_invalid_inputs(&schema, &validator, 4242)
            .expect("every fixed mutation should apply");
        let second = generate_invalid_inputs(&schema, &validator, 4242)
            .expect("the same rejection seed should generate again");

        assert_eq!(first.len(), 7);
        assert!(first.iter().all(Option::is_some));
        for (left, right) in first.iter().zip(&second) {
            let left = left.as_ref().unwrap();
            let right = right.as_ref().unwrap();
            assert_eq!(left.arguments, right.arguments);
            assert_eq!(left.omit_arguments, right.omit_arguments);
            assert_eq!(left.reproduction, right.reproduction);
            assert!(matches!(
                validator.validate(&left.arguments),
                Err(InstanceValidationIssue::Mismatch { error_count: 1 })
            ));
            assert!(left.reproduction.mutation_kind().is_some());
        }
        assert!(first[0].as_ref().unwrap().omit_arguments);
        assert!(
            first[1..]
                .iter()
                .all(|case| !case.as_ref().unwrap().omit_arguments)
        );
    }

    #[test]
    fn fixed_mutation_names_require_the_corresponding_advertised_schema_rule() {
        let schema = json!({
            "type": "object",
            "properties": {
                "mode": {"enum": ["safe", "strict"]}
            },
            "required": ["mode"]
        });
        let validator = LocalValidator::compile(&schema).expect("the schema should compile");
        let generated = generate_invalid_inputs(&schema, &validator, 99)
            .expect("the schema still admits several exact invalid mutations");

        assert!(generated[0].is_some());
        assert!(generated[1].is_some());
        assert!(generated[2].is_some());
        assert!(generated[3].is_none(), "no property type was advertised");
        assert!(generated[4].is_none(), "null was not forbidden by a type");
        assert!(generated[5].is_some());
        assert!(
            generated[6].is_none(),
            "additional properties were not forbidden"
        );
    }

    #[test]
    fn every_valid_root_object_contract_has_at_least_one_rejection_case() {
        let schema = json!({"type": "object"});
        let validator = LocalValidator::compile(&schema).expect("the schema should compile");
        let generated = generate_invalid_inputs(&schema, &validator, 101)
            .expect("a root object contract always admits a wrong-root mutation");

        assert_eq!(generated.len(), 7);
        let wrong_root = generated[1]
            .as_ref()
            .expect("the wrong-root mutation must remain applicable");
        assert!(wrong_root.arguments.is_array());
        assert!(matches!(
            validator.validate(&wrong_root.arguments),
            Err(InstanceValidationIssue::Mismatch { error_count: 1 })
        ));
        assert!(generated.iter().any(Option::is_some));
    }

    #[test]
    fn const_enum_defaults_and_local_references_supply_constrained_boundaries() {
        let schema = json!({
            "type": "object",
            "$defs": {
                "mode": {"enum": ["safe", "strict"]}
            },
            "properties": {
                "mode": {"$ref": "#/$defs/mode"},
                "enabled": {"const": true},
                "count": {"type": "integer", "default": 3, "minimum": 3, "maximum": 3}
            },
            "required": ["mode", "enabled", "count"],
            "additionalProperties": false
        });
        let validator = LocalValidator::compile(&schema).expect("the schema should compile");
        let generated = generate_inputs(&schema, &validator, 7, 8)
            .expect("the constrained schema should generate");
        assert!(
            generated
                .iter()
                .all(|case| validator.validate(&case.arguments).is_ok())
        );
    }

    #[test]
    fn unsatisfiable_or_oversized_schemas_fail_without_an_input() {
        let impossible = json!({"type": "object", "not": {}});
        let validator = LocalValidator::compile(&impossible).expect("the schema should compile");
        assert!(matches!(
            generate_inputs(&impossible, &validator, 1, 1),
            Err(GenerationFailure::Unavailable)
        ));
        assert!(matches!(
            generate_invalid_inputs(&impossible, &validator, 1),
            Err(GenerationFailure::Unavailable)
        ));

        let oversized = json!({
            "type": "object",
            "properties": {"value": {"type": "string", "minLength": 1_048_576}},
            "required": ["value"]
        });
        let validator = LocalValidator::compile(&oversized).expect("the schema should compile");
        assert!(matches!(
            generate_inputs(&oversized, &validator, 1, 1),
            Err(GenerationFailure::Limit(violation))
                if violation.kind() == LimitKind::InstanceBytes
        ));

        let ordinary = json!({"type": "object"});
        let validator = LocalValidator::compile(&ordinary).expect("the schema should compile");
        assert!(matches!(
            generate_inputs(&ordinary, &validator, 1, 101),
            Err(GenerationFailure::Limit(violation))
                if violation.kind() == LimitKind::ActiveCases
        ));
    }

    #[test]
    fn aggregate_generated_input_bytes_stop_before_case_retention() {
        let maximum = 8_388_608_u64;
        let case_count = 100_usize;
        let exact_case_bytes = maximum / u64::try_from(case_count).unwrap();
        let candidates = vec![(json!({}), exact_case_bytes)];
        assert_eq!(
            select_generated_inputs(&candidates, 7, case_count, maximum)
                .expect("the exact aggregate boundary should remain valid")
                .len(),
            case_count
        );

        let oversized = vec![(json!({}), exact_case_bytes.saturating_add(1))];
        assert!(matches!(
            select_generated_inputs(&oversized, 7, case_count, maximum),
            Err(GenerationFailure::Limit(violation))
                if violation.kind() == LimitKind::ActiveInputBytes
                    && violation.observed() > maximum
                    && violation.maximum() == maximum
        ));
    }
}
