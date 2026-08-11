use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Number, Value};

use super::catalog::{InstanceValidationIssue, LocalValidator, resolve_local_reference};
use super::limits::{DiagnosticLimits, LimitKind, LimitViolation};
use super::model::{GeneratedCaseReproduction, JsonKind, StructuralInput};

pub(super) const GENERATOR_VERSION: &str = "mcp-doctor.generator/v1";

pub(super) struct GeneratedInput {
    pub(super) arguments: Value,
    pub(super) reproduction: GeneratedCaseReproduction,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum GenerationFailure {
    Limit(LimitViolation),
    Unavailable,
}

pub(super) fn generate_inputs(
    schema: &Value,
    validator: &LocalValidator,
    base_seed: u64,
    case_count: usize,
) -> Result<Vec<GeneratedInput>, GenerationFailure> {
    let limits = DiagnosticLimits::M1_DEFAULTS.values();
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
        let encoded = serde_json::to_vec(&candidate).map_err(|_| GenerationFailure::Unavailable)?;
        let observed = u64::try_from(encoded.len()).unwrap_or(u64::MAX);
        if observed > limits.instance_bytes {
            return Err(GenerationFailure::Limit(
                LimitViolation::new(LimitKind::InstanceBytes, observed, limits.instance_bytes)
                    .expect("an oversized generated input exceeds its maximum"),
            ));
        }
        if !identities.insert(candidate_identity(&encoded)) {
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

    let mut aggregate_input_bytes = 0_u64;
    let mut generated = Vec::with_capacity(case_count);
    for index in 0..case_count {
        let case_seed = base_seed.wrapping_add(u64::try_from(index).unwrap_or(u64::MAX));
        let candidate_index = bounded_index(stable_mix(case_seed), candidates.len());
        let (arguments, byte_count) = &candidates[candidate_index];
        aggregate_input_bytes = aggregate_input_bytes.saturating_add(*byte_count);
        if aggregate_input_bytes > limits.aggregate_output_bytes {
            return Err(GenerationFailure::Limit(
                LimitViolation::new(
                    LimitKind::ActiveInputBytes,
                    aggregate_input_bytes,
                    limits.aggregate_output_bytes,
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

struct Synthesizer<'root, 'budget> {
    root: &'root Value,
    random: StableRandom,
    steps: &'budget mut u64,
    active_references: BTreeSet<String>,
}

impl<'root, 'budget> Synthesizer<'root, 'budget> {
    fn new(root: &'root Value, seed: u64, steps: &'budget mut u64) -> Self {
        Self {
            root,
            random: StableRandom(seed),
            steps,
            active_references: BTreeSet::new(),
        }
    }

    fn value(
        &mut self,
        schema: &'root Value,
        depth: u64,
    ) -> Result<Option<Value>, GenerationFailure> {
        self.tick()?;
        let limits = DiagnosticLimits::M1_DEFAULTS.values();
        if depth > limits.schema_depth {
            return Err(GenerationFailure::Limit(
                LimitViolation::new(LimitKind::SchemaDepth, depth, limits.schema_depth)
                    .expect("generated input depth exceeds its maximum"),
            ));
        }

        match schema {
            Value::Bool(false) => return Ok(None),
            Value::Bool(true) => return Ok(Some(self.generic_value())),
            Value::Object(object) => {
                if let Some(value) = object.get("const") {
                    return Ok(Some(value.clone()));
                }
                if let Some(values) = object.get("enum").and_then(Value::as_array) {
                    if values.is_empty() {
                        return Ok(None);
                    }
                    return Ok(Some(values[self.choose(values.len())].clone()));
                }
                if self.choose(4) == 0
                    && let Some(value) = declared_example(object, self.next_u64())
                {
                    return Ok(Some(value.clone()));
                }

                if let Some(reference) = object
                    .get("$ref")
                    .or_else(|| object.get("$dynamicRef"))
                    .and_then(Value::as_str)
                    && self.active_references.insert(reference.to_owned())
                {
                    let resolved = resolve_local_reference(self.root, reference);
                    let generated = match resolved {
                        Some(target) => self.value(target, depth.saturating_add(1))?,
                        None => None,
                    };
                    self.active_references.remove(reference);
                    if generated.is_some() {
                        return Ok(generated);
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
                    ValueKind::Null => Ok(Some(Value::Null)),
                    ValueKind::Boolean => Ok(Some(Value::Bool(self.choose(2) == 1))),
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
                selected.insert(name.clone());
            }
        }
        for (trigger, dependents) in &plan.dependent_required {
            if selected.contains(trigger) {
                selected.extend(dependents.iter().cloned());
            }
        }

        let minimum = usize::try_from(plan.minimum_properties).unwrap_or(usize::MAX);
        for name in plan.properties.keys() {
            if selected.len() >= minimum {
                break;
            }
            selected.insert(name.clone());
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
            let generated = if let Some(schemas) = plan.properties.get(&name) {
                let selected_schema = schemas[self.choose(schemas.len())];
                self.value(selected_schema, depth.saturating_add(1))?
            } else if let Some(schema) = plan.additional_schema {
                self.value(schema, depth.saturating_add(1))?
            } else {
                Some(self.generic_value())
            };
            if let Some(value) = generated {
                output.insert(name, value);
            }
        }

        let mut generated_index = 0_u64;
        while output.len() < minimum {
            self.tick()?;
            if plan.additional_forbidden {
                break;
            }
            let name = format!("mcp_doctor_generated_{generated_index}");
            generated_index = generated_index.saturating_add(1);
            if output.contains_key(&name) || plan.properties.contains_key(&name) {
                continue;
            }
            let value = if let Some(schema) = plan.additional_schema {
                self.value(schema, depth.saturating_add(1))?
                    .unwrap_or_else(|| self.generic_value())
            } else {
                self.generic_value()
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
        let limits = DiagnosticLimits::M1_DEFAULTS.values();
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
        let mut output = Vec::with_capacity(capacity);
        for index in 0..capacity {
            self.tick()?;
            let schema = prefix.get(index).or(item_schema);
            let value = match schema {
                Some(schema) => self
                    .value(schema, depth.saturating_add(1))?
                    .unwrap_or_else(|| self.generic_value()),
                None => self.generic_value(),
            };
            output.push(value);
        }
        Ok(output)
    }

    fn string(&mut self, object: &'root Map<String, Value>) -> Result<String, GenerationFailure> {
        let limits = DiagnosticLimits::M1_DEFAULTS.values();
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
        Ok(Value::Number(selected))
    }

    fn generic_value(&mut self) -> Value {
        match self.choose(7) {
            0 => Value::Null,
            1 => Value::Bool(false),
            2 => Value::Number(Number::from(0)),
            3 => Value::String(String::new()),
            4 => Value::Array(Vec::new()),
            5 => Value::Object(Map::new()),
            _ => Value::String("synthetic-boundary".to_owned()),
        }
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
    properties: BTreeMap<String, Vec<&'root Value>>,
    required: BTreeSet<String>,
    dependent_required: BTreeMap<String, BTreeSet<String>>,
    minimum_properties: u64,
    maximum_properties: Option<u64>,
    additional_forbidden: bool,
    additional_schema: Option<&'root Value>,
}

fn collect_object_plan<'root>(
    root: &'root Value,
    schema: &'root Value,
    plan: &mut ObjectPlan<'root>,
    active_references: &mut BTreeSet<String>,
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
                .entry(name.clone())
                .or_default()
                .push(schema);
        }
    }
    if let Some(required) = object.get("required").and_then(Value::as_array) {
        plan.required.extend(
            required
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned),
        );
    }
    if let Some(dependencies) = object.get("dependentRequired").and_then(Value::as_object) {
        for (trigger, values) in dependencies {
            let target = plan.dependent_required.entry(trigger.clone()).or_default();
            target.extend(
                values
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned),
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
        && active_references.insert(reference.to_owned())
    {
        if let Some(target) = resolve_local_reference(root, reference) {
            collect_object_plan(root, target, plan, active_references, random, steps)?;
        }
        active_references.remove(reference);
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

fn tick(steps: &mut u64) -> Result<(), GenerationFailure> {
    let maximum = DiagnosticLimits::M1_DEFAULTS.values().generation_steps;
    *steps = steps.saturating_add(1);
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

    use super::{GenerationFailure, generate_inputs};
    use crate::contract::catalog::LocalValidator;
    use crate::contract::limits::LimitKind;

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
}
