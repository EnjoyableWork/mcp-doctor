use std::borrow::Cow;
use std::cell::RefCell;
use std::cmp::Ordering as CmpOrdering;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use jsonschema::error::ValidationErrorKind;
use jsonschema::json::{Array, Json, JsonNumber, Node, NodeIdentity, Object};
use jsonschema::paths::Location as SchemaLocation;
use jsonschema::{Draft, Keyword, PatternOptions, Retrieve, Uri, ValidationError, Validator};
use regex_syntax::hir::{Class, Hir, HirKind};
use serde_json::{Map, Number, Value};

use super::limits::{LimitKind, LimitViolation};

// Inject one always-true custom validator at every schema location. The pinned
// engine runs custom validators after its built-ins, so the conservative
// evaluation-width admission check accounts for locations that short-circuit
// before reaching this marker; accepted built-in work is separately metered by
// the custom JSON representation below.
const WORK_MARKER_KEYWORD: &str = "";
// The pinned compiler performs several whole-document operations around its
// per-location keyword callbacks: instrumentation, resource preparation and
// indexing, cache sizing, and tree construction. Charge a conservative fixed
// number of equivalent linear passes in addition to the separately measured
// meta-validation and reference-fan-out work. A jsonschema upgrade must
// re-audit this constant against the new compiler path.
const COMPILE_DOCUMENT_PASSES: u64 = 8;
const REGEX_SIZE_LIMIT: usize = 100_000;
const REGEX_DFA_SIZE_LIMIT: usize = 100_000;
const PATTERN_ANALYSIS_PASSES: u64 = 2;
const PATTERN_TRANSLATION_PASSES: u64 = 2;
const UNICODE_CLASS_EXPANSION_WORK: u64 = 10_000;
// `patternProperties`, `additionalProperties`, and
// `unevaluatedProperties` can cause the same compiled pattern to be consulted
// through separate validator paths. Charge three searches per reachable
// pattern and text byte before any one of them starts.
const PATTERN_MATCH_PASSES: u64 = 3;

#[derive(Debug)]
pub(super) enum SchemaWorkIssue {
    Limit(LimitViolation),
    Invalid {
        location: SchemaLocation,
        error_count: u64,
    },
    UnsupportedPattern {
        location: SchemaLocation,
    },
}

#[derive(Debug)]
pub(super) struct SchemaWorkBudget {
    maximum: u64,
    observed: AtomicU64,
    #[cfg(test)]
    attempts: AtomicU64,
}

impl SchemaWorkBudget {
    pub(super) fn new(maximum: u64) -> Arc<Self> {
        Self::with_observed(maximum, 0)
    }

    pub(super) fn with_observed(maximum: u64, observed: u64) -> Arc<Self> {
        Arc::new(Self {
            maximum,
            observed: AtomicU64::new(observed.min(maximum.saturating_add(1))),
            #[cfg(test)]
            attempts: AtomicU64::new(0),
        })
    }

    pub(super) fn observe(&self, cost: u64) -> bool {
        #[cfg(test)]
        self.attempts.fetch_add(cost, Ordering::Relaxed);
        if cost == 0 {
            return !self.exhausted();
        }
        let ceiling = self.maximum.saturating_add(1);
        let mut current = self.observed.load(Ordering::Relaxed);
        loop {
            if current > self.maximum {
                return false;
            }
            let next = current.saturating_add(cost).min(ceiling);
            match self.observed.compare_exchange_weak(
                current,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return next <= self.maximum,
                Err(observed) => current = observed,
            }
        }
    }

    pub(super) fn exhausted(&self) -> bool {
        self.observed.load(Ordering::Relaxed) > self.maximum
    }

    fn deny(&self) {
        self.observed
            .store(self.maximum.saturating_add(1), Ordering::Relaxed);
    }

    fn remaining(&self) -> u64 {
        self.maximum
            .saturating_sub(self.observed.load(Ordering::Relaxed))
    }

    pub(super) fn violation(&self) -> LimitViolation {
        LimitViolation::new(
            LimitKind::SchemaEvaluationSteps,
            self.observed.load(Ordering::Relaxed),
            self.maximum,
        )
        .expect("an exhausted schema work budget exceeds its maximum")
    }

    fn observed(&self) -> u64 {
        self.observed.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    fn attempts(&self) -> u64 {
        self.attempts.load(Ordering::Relaxed)
    }
}

thread_local! {
    static ACTIVE_BUDGETS: RefCell<Vec<Arc<SchemaWorkBudget>>> = const { RefCell::new(Vec::new()) };
}

struct ActiveBudgetGuard {
    budget: Arc<SchemaWorkBudget>,
}

impl ActiveBudgetGuard {
    fn enter(budget: Arc<SchemaWorkBudget>) -> Self {
        ACTIVE_BUDGETS.with(|budgets| budgets.borrow_mut().push(Arc::clone(&budget)));
        Self { budget }
    }
}

impl Drop for ActiveBudgetGuard {
    fn drop(&mut self) {
        ACTIVE_BUDGETS.with(|budgets| {
            let popped = budgets.borrow_mut().pop();
            debug_assert!(
                popped
                    .as_ref()
                    .is_some_and(|budget| Arc::ptr_eq(budget, &self.budget)),
                "schema work budgets must leave in stack order"
            );
        });
    }
}

fn observe_active(cost: u64) -> bool {
    ACTIVE_BUDGETS.with(|budgets| {
        budgets
            .borrow()
            .last()
            .is_none_or(|budget| budget.observe(cost))
    })
}

fn active_budget_exhausted() -> bool {
    ACTIVE_BUDGETS.with(|budgets| {
        budgets
            .borrow()
            .last()
            .is_some_and(|budget| budget.exhausted())
    })
}

fn value_work(value: &Value, mut observe: impl FnMut(u64) -> bool) -> Option<u64> {
    let mut work = 0_u64;
    let mut stack = vec![value];
    while let Some(value) = stack.pop() {
        work = work.saturating_add(1);
        if !observe(1) {
            return None;
        }
        match value {
            Value::String(value) => {
                let bytes = u64::try_from(value.len()).unwrap_or(u64::MAX);
                work = work.saturating_add(bytes);
                if !observe(bytes) {
                    return None;
                }
            }
            Value::Array(values) => stack.extend(values),
            Value::Object(values) => {
                for (key, value) in values {
                    let bytes = u64::try_from(key.len()).unwrap_or(u64::MAX);
                    work = work.saturating_add(bytes);
                    if !observe(bytes) {
                        return None;
                    }
                    stack.push(value);
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
        }
    }
    Some(work)
}

fn charge_active_value(value: &Value) -> Option<u64> {
    value_work(value, observe_active)
}

fn budgeted_equal(left: &Value, right: &Value) -> bool {
    let Some(left_work) = charge_active_value(left) else {
        return false;
    };
    let Some(right_work) = charge_active_value(right) else {
        return false;
    };
    if !observe_active(left_work.saturating_add(right_work)) {
        return false;
    }
    jsonschema::json::cmp::equal(left, right)
}

struct BudgetedJson;

impl Json for BudgetedJson {
    type Node<'a> = &'a Value;
    type PreparedKey = String;
    type StringBuffer = Value;

    fn prepare_key(key: &str) -> Self::PreparedKey {
        key.to_owned()
    }

    fn with_string_node<T>(
        buffer: &mut Self::StringBuffer,
        string: &str,
        f: impl FnOnce(Self::Node<'_>) -> T,
    ) -> T {
        let cost = u64::try_from(string.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        let selected = if observe_active(cost) { string } else { "" };
        if let Value::String(existing) = buffer {
            existing.clear();
            existing.push_str(selected);
        } else {
            *buffer = Value::String(selected.to_owned());
        }
        f(buffer)
    }
}

#[derive(Clone, Copy)]
struct BudgetedNumber<'a>(&'a Number);

impl JsonNumber for BudgetedNumber<'_> {
    fn as_u64(&self) -> Option<u64> {
        let _ = observe_active(1);
        self.0.as_u64()
    }

    fn as_i64(&self) -> Option<i64> {
        let _ = observe_active(1);
        self.0.as_i64()
    }

    fn as_f64(&self) -> Option<f64> {
        let _ = observe_active(1);
        self.0.as_f64()
    }

    fn as_str(&self) -> Cow<'_, str> {
        let rendered = self.0.to_string();
        let cost = u64::try_from(rendered.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        let _ = observe_active(cost);
        Cow::Owned(rendered)
    }

    fn to_number(&self) -> Cow<'_, Number> {
        let _ = observe_active(1);
        Cow::Borrowed(self.0)
    }
}

impl<'a> Node<'a, BudgetedJson> for &'a Value {
    type Object = BudgetedObject<'a>;
    type Array = BudgetedArray<'a>;
    type Number = BudgetedNumber<'a>;

    fn as_object(&self) -> Option<Self::Object> {
        let _ = observe_active(1);
        Value::as_object(self).map(BudgetedObject)
    }

    fn as_array(&self) -> Option<Self::Array> {
        let _ = observe_active(1);
        Value::as_array(self).map(|values| BudgetedArray(values.as_slice()))
    }

    fn as_string(&self) -> Option<Cow<'a, str>> {
        let Value::String(value) = self else {
            let _ = observe_active(1);
            return None;
        };
        let cost = u64::try_from(value.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        Some(if observe_active(cost) {
            Cow::Borrowed(value)
        } else {
            Cow::Borrowed("")
        })
    }

    fn as_number(&self) -> Option<Self::Number> {
        let _ = observe_active(1);
        Value::as_number(self).map(BudgetedNumber)
    }

    fn as_boolean(&self) -> Option<bool> {
        let _ = observe_active(1);
        self.as_bool()
    }

    fn is_null(&self) -> bool {
        let _ = observe_active(1);
        matches!(self, Value::Null)
    }

    fn json_type(&self) -> jsonschema::types::JsonType {
        let _ = observe_active(1);
        match self {
            Value::Null => jsonschema::types::JsonType::Null,
            Value::Bool(_) => jsonschema::types::JsonType::Boolean,
            Value::Number(_) => jsonschema::types::JsonType::Number,
            Value::String(_) => jsonschema::types::JsonType::String,
            Value::Array(_) => jsonschema::types::JsonType::Array,
            Value::Object(_) => jsonschema::types::JsonType::Object,
        }
    }

    fn string_length(&self) -> Option<u64> {
        let Value::String(value) = self else {
            let _ = observe_active(1);
            return None;
        };
        let cost = u64::try_from(value.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        if observe_active(cost) {
            Some(u64::try_from(value.chars().count()).unwrap_or(u64::MAX))
        } else {
            Some(0)
        }
    }

    fn equals_value(&self, expected: &Value) -> bool {
        budgeted_equal(self, expected)
    }

    fn to_value(&self) -> Cow<'a, Value> {
        if charge_active_value(self).is_some() {
            Cow::Borrowed(self)
        } else {
            Cow::Owned(Value::Null)
        }
    }

    fn identity(&self) -> Option<NodeIdentity> {
        let _ = observe_active(1);
        Some(NodeIdentity::new(std::ptr::from_ref::<Value>(self) as usize))
    }
}

#[derive(Clone, Copy)]
struct BudgetedObject<'a>(&'a Map<String, Value>);

#[derive(Clone, Copy)]
struct BudgetedMemberName<'a>(&'a str);

impl AsRef<str> for BudgetedMemberName<'_> {
    fn as_ref(&self) -> &str {
        let cost = u64::try_from(self.0.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        if observe_active(cost) { self.0 } else { "" }
    }
}

impl<'a> From<BudgetedMemberName<'a>> for Cow<'a, str> {
    fn from(value: BudgetedMemberName<'a>) -> Self {
        let cost = u64::try_from(value.0.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        if observe_active(cost) {
            Cow::Borrowed(value.0)
        } else {
            Cow::Borrowed("")
        }
    }
}

struct BudgetedMembers<'a> {
    inner: serde_json::map::Iter<'a>,
    stopped: bool,
}

impl<'a> Iterator for BudgetedMembers<'a> {
    type Item = (BudgetedMemberName<'a>, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        if self.stopped {
            return None;
        }
        let (key, value) = self.inner.next()?;
        if !observe_active(1) {
            self.stopped = true;
            return None;
        }
        Some((BudgetedMemberName(key.as_str()), value))
    }
}

impl<'a> Object<'a, BudgetedJson> for BudgetedObject<'a> {
    type Node = &'a Value;
    type MemberName = BudgetedMemberName<'a>;
    type MembersIter = BudgetedMembers<'a>;

    fn len(&self) -> usize {
        if observe_active(1) { self.0.len() } else { 0 }
    }

    fn get(&self, key: &String) -> Option<Self::Node> {
        // `serde_json::Map` is ordered without its optional preserve-order
        // feature. Account for every possible key comparison in its logarithmic
        // lookup path, including the bytes compared at each step.
        let lookup_steps = usize::BITS
            .saturating_sub(self.0.len().saturating_add(1).leading_zeros())
            .max(1);
        let cost = u64::try_from(key.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1)
            .saturating_mul(u64::from(lookup_steps).saturating_mul(16));
        observe_active(cost).then(|| self.0.get(key)).flatten()
    }

    fn members(&self) -> Self::MembersIter {
        BudgetedMembers {
            inner: self.0.iter(),
            stopped: active_budget_exhausted(),
        }
    }
}

#[derive(Clone, Copy)]
struct BudgetedArray<'a>(&'a [Value]);

struct BudgetedElements<'a> {
    inner: std::slice::Iter<'a, Value>,
    stopped: bool,
}

impl<'a> Iterator for BudgetedElements<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stopped || !observe_active(1) {
            self.stopped = true;
            return None;
        }
        self.inner.next()
    }
}

fn budgeted_unique_strings(values: &[Value]) -> Option<bool> {
    for value in values {
        if !observe_active(1) {
            return Some(false);
        }
        if !matches!(value, Value::String(_)) {
            return None;
        }
    }

    let allocation_work = u64::try_from(values.len()).unwrap_or(u64::MAX);
    if !observe_active(allocation_work) {
        return Some(false);
    }
    let mut sorted = Vec::with_capacity(values.len());
    for value in values {
        let Value::String(candidate) = value else {
            unreachable!("the complete array was classified as strings")
        };
        let candidate_work = u64::try_from(candidate.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        if !observe_active(candidate_work) {
            return Some(false);
        }

        let mut lower = 0_usize;
        let mut upper = sorted.len();
        while lower < upper {
            let middle = lower.saturating_add(upper.saturating_sub(lower) / 2);
            let existing: &&str = &sorted[middle];
            let comparison_work = u64::try_from(candidate.len().min(existing.len()))
                .unwrap_or(u64::MAX)
                .saturating_add(1);
            if !observe_active(comparison_work) {
                return Some(false);
            }
            match candidate.as_str().cmp(existing) {
                CmpOrdering::Less => upper = middle,
                CmpOrdering::Greater => lower = middle.saturating_add(1),
                CmpOrdering::Equal => return Some(false),
            }
        }

        // `Vec::insert` moves every retained pointer after the insertion
        // point. Charge those moves before performing them so an exhausted
        // budget never enters unmetered quadratic work.
        let insertion_work = u64::try_from(sorted.len().saturating_sub(lower))
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        if !observe_active(insertion_work) {
            return Some(false);
        }
        sorted.insert(lower, candidate.as_str());
    }
    Some(true)
}

impl<'a> Array<'a, BudgetedJson> for BudgetedArray<'a> {
    type Node = &'a Value;
    type ElementsIter = BudgetedElements<'a>;

    fn len(&self) -> usize {
        if observe_active(1) { self.0.len() } else { 0 }
    }

    fn elements(&self) -> Self::ElementsIter {
        BudgetedElements {
            inner: self.0.iter(),
            stopped: active_budget_exhausted(),
        }
    }

    fn is_unique(&self) -> bool {
        if !observe_active(1) {
            return false;
        }
        if let Some(unique) = budgeted_unique_strings(self.0) {
            return unique;
        }
        for left in 0..self.0.len() {
            for right in left.saturating_add(1)..self.0.len() {
                if budgeted_equal(&self.0[left], &self.0[right]) {
                    return false;
                }
                if active_budget_exhausted() {
                    return false;
                }
            }
        }
        true
    }
}

struct WorkMarker;

impl<'i> Keyword<'i, BudgetedJson> for WorkMarker {
    fn validate(&self, _instance: &'i Value) -> Result<(), ValidationError<'i>> {
        if observe_active(1) {
            Ok(())
        } else {
            Err(ValidationError::custom("schema work limit exceeded"))
        }
    }

    fn is_valid(&self, _instance: &'i Value) -> bool {
        observe_active(1)
    }
}

#[derive(Debug)]
struct RetrievalDisabled;

impl fmt::Display for RetrievalDisabled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("external JSON Schema retrieval is disabled")
    }
}

impl Error for RetrievalDisabled {}

#[derive(Debug)]
struct NoExternalRetrieval;

impl Retrieve for NoExternalRetrieval {
    fn retrieve(&self, _uri: &Uri<String>) -> Result<Value, Box<dyn Error + Send + Sync>> {
        Err(Box::new(RetrievalDisabled))
    }
}

static META_VALIDATOR: LazyLock<Validator<BudgetedJson>> = LazyLock::new(|| {
    jsonschema::options_for::<BudgetedJson>()
        .with_draft(Draft::Draft202012)
        .with_registry(&referencing::SPECIFICATIONS)
        .with_retriever(NoExternalRetrieval)
        .with_pattern_options(
            PatternOptions::regex()
                .size_limit(REGEX_SIZE_LIMIT)
                .dfa_size_limit(REGEX_DFA_SIZE_LIMIT),
        )
        .build(&referencing::meta::DRAFT202012)
        .expect("the pinned embedded Draft 2020-12 meta-schema must compile")
});

pub(super) fn validate_meta_schema(
    schema: &Value,
    budget: Arc<SchemaWorkBudget>,
    maximum_errors: u64,
) -> Result<(), SchemaWorkIssue> {
    // Construct the trusted, pinned meta-validator outside the untrusted
    // document's budget. Lazy initialization must not make the first schema in
    // a process consume a different amount of work from every later schema.
    let validator = LazyLock::force(&META_VALIDATOR);
    let guard = ActiveBudgetGuard::enter(Arc::clone(&budget));
    let mut errors = validator.iter_errors(schema).take(
        usize::try_from(maximum_errors)
            .unwrap_or(usize::MAX)
            .saturating_add(1),
    );
    let first = errors.next();
    let location = first.as_ref().map(|error| error.instance_path().clone());
    let error_count = u64::try_from(errors.count())
        .unwrap_or(u64::MAX)
        .saturating_add(u64::from(first.is_some()));
    drop(guard);
    if budget.exhausted() {
        return Err(SchemaWorkIssue::Limit(budget.violation()));
    }
    if let Some(location) = location {
        Err(SchemaWorkIssue::Invalid {
            location,
            error_count,
        })
    } else {
        Ok(())
    }
}

pub(super) struct BudgetedValidator {
    validator: Validator<BudgetedJson>,
    compile_work: u64,
    schema_work: u64,
    evaluation_width: u64,
    pattern_work: u64,
}

impl BudgetedValidator {
    pub(super) fn compile(schema: &Value, maximum: u64) -> Result<Self, SchemaWorkIssue> {
        let budget = SchemaWorkBudget::new(maximum);
        Self::compile_with_budget(schema, budget)
    }

    pub(super) fn compile_with_budget(
        schema: &Value,
        budget: Arc<SchemaWorkBudget>,
    ) -> Result<Self, SchemaWorkIssue> {
        let Some(document_work) = value_work(schema, |cost| budget.observe(cost)) else {
            return Err(SchemaWorkIssue::Limit(budget.violation()));
        };
        let prepaid_passes = COMPILE_DOCUMENT_PASSES.saturating_sub(1);
        if !budget.observe(document_work.saturating_mul(prepaid_passes)) {
            return Err(SchemaWorkIssue::Limit(budget.violation()));
        }
        let mut pattern_cache = BTreeMap::new();
        charge_pattern_compilation(schema, &mut pattern_cache, &budget, budget.maximum)
            .ok_or_else(|| SchemaWorkIssue::Limit(budget.violation()))?;
        let estimate = estimate_evaluation_work(schema, &mut pattern_cache, Arc::clone(&budget))?;
        let instrumented = instrument_schema(schema);
        let before_meta = budget.observed();
        validate_meta_schema(&instrumented, Arc::clone(&budget), 1)?;
        let meta_work = budget.observed().saturating_sub(before_meta);
        // The public jsonschema build API repeats meta-validation internally and
        // does not expose its crate-private skip switch. Prepay the measured work
        // of the same pinned validator and document before entering that call.
        if !budget.observe(meta_work) {
            return Err(SchemaWorkIssue::Limit(budget.violation()));
        }
        let compile_budget = Arc::clone(&budget);
        let result = jsonschema::options_for::<BudgetedJson>()
            .with_draft(Draft::Draft202012)
            .with_registry(&referencing::SPECIFICATIONS)
            .with_retriever(NoExternalRetrieval)
            .with_pattern_options(
                PatternOptions::regex()
                    .size_limit(REGEX_SIZE_LIMIT)
                    .dfa_size_limit(REGEX_DFA_SIZE_LIMIT),
            )
            .with_keyword(WORK_MARKER_KEYWORD, move |_, _, _| {
                if compile_budget.observe(1) {
                    Ok(Box::new(WorkMarker)
                        as Box<dyn for<'instance> Keyword<'instance, BudgetedJson>>)
                } else {
                    Err(ValidationError::custom("schema work limit exceeded"))
                }
            })
            .build(&instrumented);
        if budget.exhausted() {
            return Err(SchemaWorkIssue::Limit(budget.violation()));
        }
        result.map_or_else(
            |error| {
                let location = error.schema_path().clone();
                if matches!(
                    error.kind(),
                    ValidationErrorKind::Format { format } if format == "regex"
                ) {
                    Err(SchemaWorkIssue::UnsupportedPattern { location })
                } else {
                    Err(SchemaWorkIssue::Invalid {
                        location,
                        error_count: 1,
                    })
                }
            },
            |validator| {
                Ok(Self {
                    validator,
                    compile_work: budget.observed(),
                    schema_work: document_work,
                    evaluation_width: estimate.width,
                    pattern_work: estimate.pattern_work,
                })
            },
        )
    }

    pub(super) fn error_count(
        &self,
        instance: &Value,
        maximum: u64,
        instance_nodes: u64,
        instance_text_work: u64,
        maximum_errors: u64,
    ) -> Result<u64, LimitViolation> {
        let budget = SchemaWorkBudget::with_observed(
            maximum,
            self.compile_work.saturating_add(instance_nodes),
        );
        if budget.exhausted() {
            return Err(budget.violation());
        }
        self.error_count_with_budget(
            instance,
            budget,
            instance_nodes,
            instance_text_work,
            maximum_errors,
        )
    }

    fn error_count_with_budget(
        &self,
        instance: &Value,
        budget: Arc<SchemaWorkBudget>,
        instance_nodes: u64,
        instance_text_work: u64,
        maximum_errors: u64,
    ) -> Result<u64, LimitViolation> {
        let pattern_pass_work = self.pattern_work.saturating_mul(instance_text_work);
        let conservative_pass_work = self
            .evaluation_width
            .max(1)
            .saturating_mul(instance_nodes.max(1))
            .saturating_add(pattern_pass_work);
        if conservative_pass_work > budget.remaining() {
            budget.deny();
            return Err(budget.violation());
        }
        if !budget.observe(pattern_pass_work) {
            return Err(budget.violation());
        }
        let before = budget.observed();
        let guard = ActiveBudgetGuard::enter(Arc::clone(&budget));
        let valid = self.validator.is_valid(instance);
        drop(guard);
        if budget.exhausted() {
            return Err(budget.violation());
        }
        if valid {
            return Ok(0);
        }

        let first_pass_work = budget.observed().saturating_sub(before);
        let conservative_error_work = self
            .schema_work
            .max(self.evaluation_width)
            .saturating_mul(instance_nodes.max(1))
            .saturating_add(first_pass_work)
            .saturating_add(pattern_pass_work);
        if conservative_error_work > budget.remaining() {
            budget.deny();
            return Err(budget.violation());
        }
        if !budget.observe(pattern_pass_work) {
            return Err(budget.violation());
        }

        let guard = ActiveBudgetGuard::enter(Arc::clone(&budget));
        let error_count = u64::try_from(
            self.validator
                .iter_errors(instance)
                .take(
                    usize::try_from(maximum_errors)
                        .unwrap_or(usize::MAX)
                        .saturating_add(1),
                )
                .count(),
        )
        .unwrap_or(u64::MAX);
        drop(guard);
        if budget.exhausted() {
            Err(budget.violation())
        } else {
            Ok(error_count)
        }
    }
}

#[derive(Clone, Copy)]
struct EvaluationEstimate {
    width: u64,
    pattern_work: u64,
}

fn estimate_evaluation_work(
    schema: &Value,
    pattern_cache: &mut BTreeMap<(usize, usize), u64>,
    budget: Arc<SchemaWorkBudget>,
) -> Result<EvaluationEstimate, SchemaWorkIssue> {
    let mut active_references = BTreeSet::new();
    let estimate = estimate_schema_node(
        schema,
        schema,
        &mut active_references,
        pattern_cache,
        &budget,
        budget.maximum,
    )
    .ok_or_else(|| SchemaWorkIssue::Limit(budget.violation()))?;
    if budget.exhausted() {
        Err(SchemaWorkIssue::Limit(budget.violation()))
    } else {
        Ok(estimate)
    }
}

fn estimate_schema_node(
    root: &Value,
    schema: &Value,
    active_references: &mut BTreeSet<String>,
    pattern_cache: &mut BTreeMap<(usize, usize), u64>,
    budget: &SchemaWorkBudget,
    maximum: u64,
) -> Option<EvaluationEstimate> {
    if !budget.observe(1) {
        return None;
    }
    let Value::Object(object) = schema else {
        return Some(EvaluationEstimate {
            width: 1,
            pattern_work: 0,
        });
    };
    let mut estimate = EvaluationEstimate {
        width: 1,
        pattern_work: 0,
    };

    if let Some(pattern) = object.get("pattern").and_then(Value::as_str) {
        add_pattern_work(&mut estimate, pattern, pattern_cache, budget, maximum)?;
    }
    if let Some(Value::Object(patterns)) = object.get("patternProperties") {
        for pattern in patterns.keys() {
            add_pattern_work(&mut estimate, pattern, pattern_cache, budget, maximum)?;
        }
    }

    for keyword in ["$ref", "$dynamicRef"] {
        let Some(reference) = object.get(keyword).and_then(Value::as_str) else {
            continue;
        };
        if !reference.starts_with('#') || !active_references.insert(reference.to_owned()) {
            continue;
        }
        if !budget.observe(u64::try_from(reference.len()).unwrap_or(u64::MAX)) {
            return None;
        }
        if let Some(target) = resolve_local_reference(root, reference, budget) {
            estimate = checked_estimate_add(
                estimate,
                estimate_schema_node(
                    root,
                    target,
                    active_references,
                    pattern_cache,
                    budget,
                    maximum,
                )?,
                maximum,
                budget,
            )?;
        }
        active_references.remove(reference);
    }

    for keyword in [
        "additionalProperties",
        "contains",
        "contentSchema",
        "else",
        "if",
        "items",
        "not",
        "propertyNames",
        "then",
        "unevaluatedItems",
        "unevaluatedProperties",
    ] {
        if let Some(child @ (Value::Bool(_) | Value::Object(_))) = object.get(keyword) {
            estimate = checked_estimate_add(
                estimate,
                estimate_schema_node(
                    root,
                    child,
                    active_references,
                    pattern_cache,
                    budget,
                    maximum,
                )?,
                maximum,
                budget,
            )?;
        }
    }
    for keyword in ["allOf", "anyOf", "oneOf", "prefixItems"] {
        if let Some(Value::Array(children)) = object.get(keyword) {
            for child in children {
                if matches!(child, Value::Bool(_) | Value::Object(_)) {
                    estimate = checked_estimate_add(
                        estimate,
                        estimate_schema_node(
                            root,
                            child,
                            active_references,
                            pattern_cache,
                            budget,
                            maximum,
                        )?,
                        maximum,
                        budget,
                    )?;
                }
            }
        }
    }
    for keyword in ["dependentSchemas", "patternProperties", "properties"] {
        if let Some(Value::Object(children)) = object.get(keyword) {
            for child in children.values() {
                if matches!(child, Value::Bool(_) | Value::Object(_)) {
                    estimate = checked_estimate_add(
                        estimate,
                        estimate_schema_node(
                            root,
                            child,
                            active_references,
                            pattern_cache,
                            budget,
                            maximum,
                        )?,
                        maximum,
                        budget,
                    )?;
                }
            }
        }
    }
    Some(estimate)
}

fn add_pattern_work(
    estimate: &mut EvaluationEstimate,
    pattern: &str,
    cache: &mut BTreeMap<(usize, usize), u64>,
    budget: &SchemaWorkBudget,
    maximum: u64,
) -> Option<()> {
    let base_work = cached_pattern_complexity(pattern, cache, budget, maximum)?;
    let work = base_work
        .saturating_mul(PATTERN_MATCH_PASSES)
        .min(maximum.saturating_add(1));
    estimate.pattern_work = checked_work_add(estimate.pattern_work, work, maximum, budget)?;
    Some(())
}

fn cached_pattern_complexity(
    pattern: &str,
    cache: &mut BTreeMap<(usize, usize), u64>,
    budget: &SchemaWorkBudget,
    maximum: u64,
) -> Option<u64> {
    let cache_work = u64::from(
        usize::BITS
            .saturating_sub(cache.len().saturating_add(1).leading_zeros())
            .max(1),
    );
    if !budget.observe(cache_work) {
        return None;
    }
    let key = (pattern.as_ptr() as usize, pattern.len());
    if let Some(work) = cache.get(&key) {
        return Some(*work);
    }
    let source_bytes = u64::try_from(pattern.len())
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    let source_searches = u64::from(
        usize::BITS
            .saturating_sub(pattern.len().saturating_add(1).leading_zeros())
            .max(1),
    );
    let analysis_work = source_bytes
        .saturating_mul(source_searches)
        .saturating_mul(PATTERN_ANALYSIS_PASSES);
    if analysis_work > budget.remaining() || !budget.observe(analysis_work) {
        budget.deny();
        return None;
    }
    let work = pattern_complexity(pattern, budget, maximum)?;
    if !budget.observe(work) {
        return None;
    }
    cache.insert(key, work);
    Some(work)
}

fn charge_pattern_compilation(
    schema: &Value,
    cache: &mut BTreeMap<(usize, usize), u64>,
    budget: &SchemaWorkBudget,
    maximum: u64,
) -> Option<()> {
    let mut stack = vec![schema];
    while let Some(schema) = stack.pop() {
        if !budget.observe(1) {
            return None;
        }
        let Value::Object(object) = schema else {
            continue;
        };
        if let Some(pattern) = object.get("pattern").and_then(Value::as_str) {
            cached_pattern_complexity(pattern, cache, budget, maximum)?;
        }
        if let Some(Value::Object(patterns)) = object.get("patternProperties") {
            for pattern in patterns.keys() {
                cached_pattern_complexity(pattern, cache, budget, maximum)?;
            }
        }

        for keyword in [
            "additionalProperties",
            "contains",
            "contentSchema",
            "else",
            "if",
            "items",
            "not",
            "propertyNames",
            "then",
            "unevaluatedItems",
            "unevaluatedProperties",
        ] {
            if let Some(child @ (Value::Bool(_) | Value::Object(_))) = object.get(keyword) {
                stack.push(child);
            }
        }
        for keyword in ["allOf", "anyOf", "oneOf", "prefixItems"] {
            if let Some(Value::Array(children)) = object.get(keyword) {
                stack.extend(
                    children
                        .iter()
                        .filter(|child| matches!(child, Value::Bool(_) | Value::Object(_))),
                );
            }
        }
        for keyword in [
            "$defs",
            "definitions",
            "dependentSchemas",
            "patternProperties",
            "properties",
        ] {
            if let Some(Value::Object(children)) = object.get(keyword) {
                stack.extend(
                    children
                        .values()
                        .filter(|child| matches!(child, Value::Bool(_) | Value::Object(_))),
                );
            }
        }
    }
    Some(())
}

fn pattern_complexity(pattern: &str, budget: &SchemaWorkBudget, maximum: u64) -> Option<u64> {
    match jsonschema_regex::analyze_pattern(pattern) {
        Some(jsonschema_regex::PatternAnalysis::Prefix(_))
        | Some(jsonschema_regex::PatternAnalysis::Exact(_))
        | Some(jsonschema_regex::PatternAnalysis::NoWhitespace) => return Some(1),
        Some(jsonschema_regex::PatternAnalysis::Alternation(alternatives)) => {
            return Some(
                u64::try_from(alternatives.len())
                    .unwrap_or(u64::MAX)
                    .max(1)
                    .min(maximum.saturating_add(1)),
            );
        }
        None => {}
    }

    let translation_work = pattern_translation_work(pattern)
        .saturating_mul(PATTERN_TRANSLATION_PASSES)
        .min(maximum.saturating_add(1));
    if translation_work > budget.remaining() || !budget.observe(translation_work) {
        budget.deny();
        return None;
    }
    let Some(hir) = jsonschema_regex::to_rust_regex(pattern)
        .ok()
        .and_then(|translated| regex_syntax::Parser::new().parse(&translated).ok())
    else {
        // The configured jsonschema compiler will reject this pattern before
        // retaining a validator. Keep enough budget for that deterministic
        // typed diagnostic without attempting to duplicate its error logic.
        return Some(
            u64::try_from(pattern.len())
                .unwrap_or(u64::MAX)
                .saturating_add(1)
                .min(maximum.saturating_add(1)),
        );
    };
    Some(hir_complexity(&hir, maximum))
}

fn pattern_translation_work(pattern: &str) -> u64 {
    let bytes = pattern.as_bytes();
    let mut rewrites = 0_u64;
    let mut unicode_classes = 0_u64;
    for pair in bytes.windows(2) {
        if pair[0] != b'\\' {
            continue;
        }
        if matches!(pair[1], b'c' | b'd' | b'D' | b'w' | b'W' | b's' | b'S') {
            rewrites = rewrites.saturating_add(1);
        }
        if matches!(pair[1], b'p' | b'P') {
            unicode_classes = unicode_classes.saturating_add(1);
        }
    }
    let source_bytes = u64::try_from(bytes.len())
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    let expanded_bytes = source_bytes.saturating_add(rewrites.saturating_mul(64));
    expanded_bytes
        .saturating_mul(rewrites.saturating_add(2))
        .saturating_add(unicode_classes.saturating_mul(UNICODE_CLASS_EXPANSION_WORK))
}

fn hir_complexity(hir: &Hir, maximum: u64) -> u64 {
    let ceiling = maximum.saturating_add(1);
    match hir.kind() {
        HirKind::Empty | HirKind::Look(_) => 1,
        HirKind::Literal(literal) => u64::try_from(literal.0.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1)
            .min(ceiling),
        HirKind::Class(class) => {
            let ranges = match class {
                Class::Unicode(class) => class.ranges().len(),
                Class::Bytes(class) => class.ranges().len(),
            };
            u64::try_from(ranges)
                .unwrap_or(u64::MAX)
                .saturating_mul(4)
                .saturating_add(1)
                .min(ceiling)
        }
        HirKind::Repetition(repetition) => {
            let copies = u64::from(
                repetition
                    .max
                    .unwrap_or_else(|| repetition.min.saturating_add(1))
                    .max(1),
            );
            hir_complexity(&repetition.sub, maximum)
                .saturating_mul(copies)
                .saturating_add(1)
                .min(ceiling)
        }
        HirKind::Capture(capture) => hir_complexity(&capture.sub, maximum)
            .saturating_add(2)
            .min(ceiling),
        HirKind::Concat(children) | HirKind::Alternation(children) => {
            children.iter().fold(1_u64, |work, child| {
                work.saturating_add(hir_complexity(child, maximum))
                    .min(ceiling)
            })
        }
    }
}

fn checked_estimate_add(
    left: EvaluationEstimate,
    right: EvaluationEstimate,
    maximum: u64,
    budget: &SchemaWorkBudget,
) -> Option<EvaluationEstimate> {
    Some(EvaluationEstimate {
        width: checked_work_add(left.width, right.width, maximum, budget)?,
        pattern_work: checked_work_add(left.pattern_work, right.pattern_work, maximum, budget)?,
    })
}

fn checked_work_add(left: u64, right: u64, maximum: u64, budget: &SchemaWorkBudget) -> Option<u64> {
    let work = left.saturating_add(right);
    if work > maximum {
        budget.deny();
        None
    } else {
        Some(work)
    }
}

fn resolve_local_reference<'a>(
    schema: &'a Value,
    reference: &str,
    budget: &SchemaWorkBudget,
) -> Option<&'a Value> {
    let fragment = reference.strip_prefix('#')?;
    if fragment.is_empty() {
        return Some(schema);
    }
    if fragment.starts_with('/') {
        let decoded = percent_decode(fragment)?;
        return schema.pointer(&decoded);
    }

    let anchor = percent_decode(fragment)?;
    let mut stack = vec![schema];
    while let Some(value) = stack.pop() {
        if !budget.observe(1) {
            return None;
        }
        match value {
            Value::Object(object) => {
                if object.get("$anchor").and_then(Value::as_str) == Some(anchor.as_str())
                    || object.get("$dynamicAnchor").and_then(Value::as_str) == Some(anchor.as_str())
                {
                    return Some(value);
                }
                stack.extend(object.values());
            }
            Value::Array(values) => stack.extend(values),
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
    }
    None
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = *bytes.get(index.saturating_add(1))?;
            let low = *bytes.get(index.saturating_add(2))?;
            decoded.push(
                hex_value(high)?
                    .checked_mul(16)?
                    .checked_add(hex_value(low)?)?,
            );
            index = index.saturating_add(3);
        } else {
            decoded.push(bytes[index]);
            index = index.saturating_add(1);
        }
    }
    String::from_utf8(decoded).ok()
}

const fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn instrument_schema(schema: &Value) -> Value {
    match schema {
        Value::Bool(true) => Value::Object(marker_object()),
        Value::Bool(false) => {
            let mut object = marker_object();
            object.insert("not".to_owned(), Value::Object(marker_object()));
            Value::Object(object)
        }
        Value::Object(source) => {
            // Build the instrumented tree in one pass. Cloning the complete
            // object before recursively replacing each subschema would copy a
            // deeply nested document once per ancestor.
            let mut object = marker_object();
            for (keyword, value) in source {
                if keyword == WORK_MARKER_KEYWORD {
                    // The empty string is a valid unknown JSON Schema keyword,
                    // and a local JSON Pointer may use its value as a schema.
                    // Keep that target intact while the registered custom
                    // keyword still supplies this location's work marker.
                    let value = if matches!(value, Value::Bool(_) | Value::Object(_)) {
                        instrument_schema(value)
                    } else {
                        value.clone()
                    };
                    object.insert(keyword.clone(), value);
                    continue;
                }
                let value = match keyword.as_str() {
                    "additionalProperties"
                    | "contains"
                    | "contentSchema"
                    | "else"
                    | "if"
                    | "items"
                    | "not"
                    | "propertyNames"
                    | "then"
                    | "unevaluatedItems"
                    | "unevaluatedProperties"
                        if matches!(value, Value::Bool(_) | Value::Object(_)) =>
                    {
                        instrument_schema(value)
                    }
                    "allOf" | "anyOf" | "oneOf" | "prefixItems" => instrument_schema_array(value),
                    "$defs" | "dependentSchemas" | "patternProperties" | "properties" => {
                        instrument_schema_map(value)
                    }
                    _ => value.clone(),
                };
                object.insert(keyword.clone(), value);
            }
            Value::Object(object)
        }
        Value::Null | Value::Number(_) | Value::String(_) | Value::Array(_) => schema.clone(),
    }
}

fn instrument_schema_array(value: &Value) -> Value {
    let Value::Array(values) = value else {
        return value.clone();
    };
    Value::Array(
        values
            .iter()
            .map(|value| {
                if matches!(value, Value::Bool(_) | Value::Object(_)) {
                    instrument_schema(value)
                } else {
                    value.clone()
                }
            })
            .collect(),
    )
}

fn instrument_schema_map(value: &Value) -> Value {
    let Value::Object(values) = value else {
        return value.clone();
    };
    Value::Object(
        values
            .iter()
            .map(|(key, value)| {
                let value = if matches!(value, Value::Bool(_) | Value::Object(_)) {
                    instrument_schema(value)
                } else {
                    value.clone()
                };
                (key.clone(), value)
            })
            .collect(),
    )
}

fn marker_object() -> Map<String, Value> {
    [(WORK_MARKER_KEYWORD.to_owned(), Value::Null)]
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::{Map, Value, json};

    use super::{
        BudgetedValidator, NoExternalRetrieval, SchemaWorkBudget, SchemaWorkIssue,
        validate_meta_schema,
    };

    fn instance_work(instance: &Value) -> (u64, u64) {
        let mut nodes = 0_u64;
        let mut text = 0_u64;
        let mut stack = vec![instance];
        while let Some(value) = stack.pop() {
            nodes = nodes.saturating_add(1);
            match value {
                Value::String(value) => {
                    text = text.saturating_add(
                        u64::try_from(value.len())
                            .unwrap_or(u64::MAX)
                            .saturating_add(1),
                    );
                }
                Value::Array(values) => stack.extend(values),
                Value::Object(values) => {
                    for (key, value) in values {
                        text = text.saturating_add(
                            u64::try_from(key.len())
                                .unwrap_or(u64::MAX)
                                .saturating_add(1),
                        );
                        stack.push(value);
                    }
                }
                Value::Null | Value::Bool(_) | Value::Number(_) => {}
            }
        }
        (nodes, text)
    }

    #[test]
    fn budget_records_only_one_over_maximum() {
        let budget = SchemaWorkBudget::new(3);
        assert!(budget.observe(2));
        assert!(!budget.observe(2));
        assert_eq!(budget.observed(), 4);
        assert!(!budget.observe(1));
        assert_eq!(budget.observed(), 4);
    }

    #[test]
    fn meta_validation_is_budgeted_and_preserves_invalidity() {
        let ordinary = json!({"type": "object", "properties": {"ok": {"type": "boolean"}}});
        validate_meta_schema(&ordinary, SchemaWorkBudget::new(10_000), 100).unwrap();

        let invalid = json!({"allOf": []});
        assert!(matches!(
            validate_meta_schema(&invalid, SchemaWorkBudget::new(10_000), 100),
            Err(SchemaWorkIssue::Invalid { .. })
        ));

        assert!(matches!(
            validate_meta_schema(&ordinary, SchemaWorkBudget::new(1), 100),
            Err(SchemaWorkIssue::Limit(_))
        ));
    }

    #[test]
    fn long_required_lists_remain_valid_under_the_declared_work_budget() {
        let required = (0..101)
            .map(|index| format!("private-field-{index:03}"))
            .collect::<Vec<_>>();
        BudgetedValidator::compile(&json!({"type": "object", "required": required}), 100_000)
            .unwrap();

        let duplicated = (0..101).map(|_| "private-field").collect::<Vec<_>>();
        assert!(matches!(
            BudgetedValidator::compile(&json!({"type": "object", "required": duplicated}), 100_000,),
            Err(SchemaWorkIssue::Invalid { .. })
        ));
    }

    #[test]
    fn ordinary_validation_keeps_boolean_object_and_reference_semantics() {
        let schema = json!({
            "$defs": {"flag": {"type": "boolean"}},
            "type": "object",
            "properties": {"ok": {"$ref": "#/$defs/flag"}},
            "required": ["ok"],
            "additionalProperties": false
        });
        let validator = BudgetedValidator::compile(&schema, 100_000).unwrap();
        assert_eq!(
            validator.error_count(&json!({"ok": true}), 100_000, 2, 0, 100,),
            Ok(0)
        );
        assert_eq!(
            validator.error_count(&json!({"ok": 1}), 100_000, 2, 0, 100,),
            Ok(1)
        );

        let composed = BudgetedValidator::compile(
            &json!({
                "allOf": [
                    {"anyOf": [{"type": "integer"}, {"type": "string"}]},
                    {"oneOf": [{"minimum": 0}, {"type": "string"}]}
                ]
            }),
            100_000,
        )
        .unwrap();
        assert_eq!(composed.error_count(&json!(7), 100_000, 1, 0, 100), Ok(0));
        assert!(matches!(
            composed.error_count(&json!(false), 100_000, 1, 0, 100),
            Ok(count) if count > 0
        ));
    }

    #[test]
    fn instrumentation_preserves_an_empty_keyword_reference_target() {
        let validator = BudgetedValidator::compile(
            &json!({
                "": {"type": "integer"},
                "$ref": "#/"
            }),
            100_000,
        )
        .unwrap();
        assert_eq!(validator.error_count(&json!(7), 100_000, 1, 0, 100), Ok(0));
        assert!(matches!(
            validator.error_count(&json!("private"), 100_000, 1, 8, 100),
            Ok(count) if count > 0
        ));
    }

    #[test]
    fn bounded_validation_preserves_ordinary_draft_2020_12_semantics() {
        let fixtures = vec![
            (Value::Bool(false), vec![json!(null), json!({"ok": true})]),
            (
                json!({
                    "": {"type": "integer"},
                    "$ref": "#/"
                }),
                vec![json!(7), json!("seven")],
            ),
            (
                json!({
                    "type": "object",
                    "properties": {"ok": {"type": "boolean"}},
                    "required": ["ok"],
                    "additionalProperties": false
                }),
                vec![
                    json!({"ok": true}),
                    json!({"ok": 1}),
                    json!({"extra": true}),
                ],
            ),
            (
                json!({
                    "type": "array",
                    "prefixItems": [{"type": "integer"}],
                    "items": {"type": "string"},
                    "contains": {"const": "x"},
                    "minContains": 1,
                    "uniqueItems": true
                }),
                vec![json!([1, "x"]), json!([1, "y"]), json!([1, "x", "x"])],
            ),
            (
                json!({
                    "type": "object",
                    "dependentSchemas": {"a": {"required": ["b"]}}
                }),
                vec![json!({"a": 1, "b": 2}), json!({"a": 1}), json!({"b": 2})],
            ),
            (
                json!({
                    "type": "object",
                    "patternProperties": {"^x[0-9]+$": {"type": "integer"}},
                    "propertyNames": {"pattern": "^[a-z0-9]+$"},
                    "additionalProperties": false
                }),
                vec![json!({"x1": 1}), json!({"x1": "one"}), json!({"other": 1})],
            ),
            (
                json!({
                    "allOf": [{"properties": {"known": {"type": "integer"}}}],
                    "unevaluatedProperties": false
                }),
                vec![json!({"known": 1}), json!({"known": 1, "extra": 2})],
            ),
            (
                json!({
                    "anyOf": [
                        {"const": {"kind": "a", "value": 1}},
                        {"enum": [1, 2.0, "three"]}
                    ]
                }),
                vec![json!({"kind": "a", "value": 1}), json!(2), json!(false)],
            ),
        ];

        for (fixture_index, (schema, instances)) in fixtures.into_iter().enumerate() {
            let baseline = jsonschema::draft202012::options()
                .with_retriever(NoExternalRetrieval)
                .build(&schema)
                .expect("the ordinary comparison schema should compile");
            let bounded = BudgetedValidator::compile(&schema, 100_000)
                .expect("the bounded ordinary comparison schema should compile");
            for (instance_index, instance) in instances.iter().enumerate() {
                let expected = baseline.is_valid(instance);
                let (nodes, text) = instance_work(instance);
                let actual = bounded
                    .error_count(instance, 100_000, nodes, text, 100)
                    .expect("ordinary validation should stay within its work budget")
                    == 0;
                assert_eq!(
                    actual, expected,
                    "ordinary schema fixture {fixture_index} instance {instance_index} changed validity"
                );
            }
        }
    }

    #[test]
    fn compilation_and_instance_walk_share_one_operation_budget() {
        let validator = BudgetedValidator::compile(
            &json!({
                "type": "object",
                "properties": {"ok": {"type": "boolean"}},
                "required": ["ok"]
            }),
            100_000,
        )
        .unwrap();
        assert!(validator.compile_work > 0);
        let maximum = validator.compile_work.saturating_add(1);
        let violation = validator
            .error_count(&json!({"ok": true}), maximum, 2, 0, 100)
            .expect_err("compile work plus the instance walk must exhaust one shared budget");
        assert_eq!(violation.kind(), super::LimitKind::SchemaEvaluationSteps);
        assert_eq!(violation.observed(), maximum.saturating_add(1));
        assert_eq!(violation.maximum(), maximum);
    }

    #[test]
    fn cross_product_and_unique_work_stop_at_the_exact_budget_boundary() {
        let branches = (0..64)
            .map(|value| json!({"const": value}))
            .collect::<Vec<_>>();
        let schema = json!({"type": "array", "items": {"allOf": branches}});
        let validator = BudgetedValidator::compile(&schema, 100_000).unwrap();
        let instance = Value::Array((0..64).map(|_| json!(999)).collect());
        let budget = SchemaWorkBudget::new(1_000);
        assert!(
            validator
                .error_count_with_budget(&instance, Arc::clone(&budget), 65, 0, 100)
                .is_err()
        );
        assert_eq!(budget.observed(), 1_001);
        assert_eq!(budget.attempts(), 0);

        let unique = BudgetedValidator::compile(&json!({"uniqueItems": true}), 100_000).unwrap();
        let instance = Value::Array((0..64).map(|value| json!([value])).collect());
        let budget = SchemaWorkBudget::new(500);
        assert!(
            unique
                .error_count_with_budget(&instance, Arc::clone(&budget), 129, 0, 100)
                .is_err()
        );
        assert_eq!(budget.observed(), 501);
        assert!(budget.attempts() <= 550);
    }

    #[test]
    fn combinator_reference_pattern_and_equality_work_are_bounded() {
        for keyword in ["anyOf", "oneOf"] {
            let branches = (0..128)
                .map(|value| json!({"const": value}))
                .collect::<Vec<_>>();
            let schema = Value::Object(
                [(keyword.to_owned(), Value::Array(branches))]
                    .into_iter()
                    .collect(),
            );
            let validator = BudgetedValidator::compile(&schema, 100_000).unwrap();
            let budget = SchemaWorkBudget::new(100);
            assert!(
                validator
                    .error_count_with_budget(&json!(999), Arc::clone(&budget), 1, 0, 100)
                    .is_err()
            );
            assert_eq!(budget.observed(), 101);
            assert_eq!(budget.attempts(), 0);
        }

        let fanout = reference_fanout_schema(10);
        let validator = BudgetedValidator::compile(&fanout, 100_000).unwrap();
        let budget = SchemaWorkBudget::new(1_000);
        assert!(
            validator
                .error_count_with_budget(&json!(999), Arc::clone(&budget), 1, 0, 100)
                .is_err()
        );
        assert_eq!(budget.observed(), 1_001);
        assert_eq!(budget.attempts(), 0);

        assert!(matches!(
            BudgetedValidator::compile(&reference_fanout_schema(14), 10_000),
            Err(SchemaWorkIssue::Limit(_))
        ));

        let pattern = BudgetedValidator::compile(&json!({"pattern": "^a+$"}), 100_000).unwrap();
        let pattern_budget = SchemaWorkBudget::new(100);
        assert!(
            pattern
                .error_count_with_budget(
                    &json!("a".repeat(1_000)),
                    Arc::clone(&pattern_budget),
                    1,
                    1_001,
                    100,
                )
                .is_err()
        );
        assert_eq!(pattern_budget.observed(), 101);
        assert_eq!(pattern_budget.attempts(), 0);

        let ordinary_pattern_budget = SchemaWorkBudget::new(100_000);
        assert_eq!(
            pattern.error_count_with_budget(
                &json!("a".repeat(1_000)),
                Arc::clone(&ordinary_pattern_budget),
                1,
                1_001,
                100,
            ),
            Ok(0)
        );
        assert!(ordinary_pattern_budget.observed() < 100_000);

        let expanded =
            BudgetedValidator::compile(&json!({"pattern": "^(a{1000})+$"}), 100_000).unwrap();
        let expanded_budget = SchemaWorkBudget::new(100_000);
        assert!(
            expanded
                .error_count_with_budget(
                    &json!("a".repeat(100)),
                    Arc::clone(&expanded_budget),
                    1,
                    101,
                    100,
                )
                .is_err()
        );
        assert_eq!(expanded_budget.observed(), 100_001);
        assert_eq!(expanded_budget.attempts(), 0);

        let equality =
            BudgetedValidator::compile(&json!({"const": "a".repeat(1_000)}), 100_000).unwrap();
        let equality_budget = SchemaWorkBudget::new(100);
        assert!(
            equality
                .error_count_with_budget(
                    &json!("b".repeat(1_000)),
                    Arc::clone(&equality_budget),
                    1,
                    0,
                    100,
                )
                .is_err()
        );
        assert_eq!(equality_budget.observed(), 101);
        assert!(equality_budget.attempts() >= 1_000);

        assert!(matches!(
            BudgetedValidator::compile(&json!({"pattern": "^(?!private)"}), 100_000),
            Err(SchemaWorkIssue::UnsupportedPattern { .. })
        ));
        assert!(matches!(
            BudgetedValidator::compile(&json!({"pattern": r"\s".repeat(50)}), 100_000),
            Err(SchemaWorkIssue::Limit(_))
        ));
        BudgetedValidator::compile(&json!({"pattern": r"\s+"}), 100_000).unwrap();
        assert!(matches!(
            BudgetedValidator::compile(
                &json!({"patternProperties": {"^(?!private)": {"type": "string"}}}),
                100_000,
            ),
            Err(SchemaWorkIssue::UnsupportedPattern { .. })
        ));
        assert!(matches!(
            BudgetedValidator::compile(
                &json!({
                    "properties": {
                        "pattern": {"$ref": "https://synthetic.invalid/schema"}
                    }
                }),
                100_000,
            ),
            Err(SchemaWorkIssue::Invalid { .. })
        ));
    }

    #[test]
    fn pattern_property_cross_products_are_admitted_before_matching() {
        let patterns = (0..32)
            .map(|index| {
                (
                    format!("^field-{index:02}-[a-z]+$"),
                    json!({"type": "string"}),
                )
            })
            .collect::<Map<_, _>>();
        let validator = BudgetedValidator::compile(
            &json!({"type": "object", "patternProperties": patterns}),
            100_000,
        )
        .unwrap();
        let instance = Value::Object(
            (0..32)
                .map(|index| (format!("field-{index:02}-{}", "a".repeat(48)), json!("ok")))
                .collect(),
        );
        let text_work = instance
            .as_object()
            .unwrap()
            .keys()
            .fold(0_u64, |work, key| {
                work.saturating_add(u64::try_from(key.len()).unwrap().saturating_add(1))
            })
            + 32 * 3;
        let budget = SchemaWorkBudget::new(100_000);
        assert!(
            validator
                .error_count_with_budget(&instance, Arc::clone(&budget), 33, text_work, 100,)
                .is_err()
        );
        assert_eq!(budget.observed(), 100_001);
        assert_eq!(budget.attempts(), 0);

        let ordinary = BudgetedValidator::compile(
            &json!({
                "type": "object",
                "patternProperties": {"^field-[0-9]+$": {"type": "string"}}
            }),
            100_000,
        )
        .unwrap();
        let instance = json!({"field-1": "ok"});
        assert_eq!(ordinary.error_count(&instance, 100_000, 2, 11, 100,), Ok(0));
    }

    fn reference_fanout_schema(depth: usize) -> Value {
        let mut definitions = Map::new();
        for index in (0..=depth).rev() {
            let value = if index == depth {
                json!({"const": 0})
            } else {
                let reference = format!("#/$defs/node{}", index + 1);
                json!({"allOf": [{"$ref": reference}, {"$ref": reference}]})
            };
            definitions.insert(format!("node{index}"), value);
        }
        json!({"$defs": definitions, "$ref": "#/$defs/node0"})
    }
}
