use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::sync::Arc;

use jsonschema::paths::LocationSegment as SchemaLocationSegment;
use serde_json::{Map, Value, json};

use super::http_headers::validate_annotations;
use super::limits::{DiagnosticLimits, LimitKind, LimitViolation};
use super::model::{
    CheckId, CheckResult, CredentialSchemaKeyword, ExpectedShape, Finding, JsonKind,
    JsonRpcErrorKind, Location, LocationField, Requirement, RuleViolation, SchemaValidationPhase,
    SkipReason,
};
use super::protocol::{RevisionSelection, SupportedRevision, select_current_modern_revision};
use super::schema_budget::{
    BudgetedValidator, SchemaWorkBudget, SchemaWorkIssue, validate_meta_schema,
};
use crate::transport::{Conversation, ProbeRequest, ProbeResponse};

const PROTOCOL_REVISION: &str = "2026-07-28";
pub(super) const DRAFT_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";
const DRAFT_2020_12_VOCABULARIES: &[&str] = &[
    "https://json-schema.org/draft/2020-12/vocab/core",
    "https://json-schema.org/draft/2020-12/vocab/applicator",
    "https://json-schema.org/draft/2020-12/vocab/unevaluated",
    "https://json-schema.org/draft/2020-12/vocab/validation",
    "https://json-schema.org/draft/2020-12/vocab/meta-data",
    "https://json-schema.org/draft/2020-12/vocab/format-annotation",
    "https://json-schema.org/draft/2020-12/vocab/content",
];

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum CatalogKind {
    Tools,
    Prompts,
    Resources,
    ResourceTemplates,
}

impl CatalogKind {
    const ALL: [Self; 4] = [
        Self::Tools,
        Self::Prompts,
        Self::Resources,
        Self::ResourceTemplates,
    ];

    const fn method(self) -> &'static str {
        match self {
            Self::Tools => "tools/list",
            Self::Prompts => "prompts/list",
            Self::Resources => "resources/list",
            Self::ResourceTemplates => "resources/templates/list",
        }
    }

    const fn result_field(self) -> &'static str {
        match self {
            Self::Tools => "tools",
            Self::Prompts => "prompts",
            Self::Resources => "resources",
            Self::ResourceTemplates => "resourceTemplates",
        }
    }

    const fn capability(self) -> &'static str {
        match self {
            Self::Tools => "tools",
            Self::Prompts => "prompts",
            Self::Resources | Self::ResourceTemplates => "resources",
        }
    }

    const fn location_field(self) -> LocationField {
        match self {
            Self::Tools => LocationField::Tools,
            Self::Prompts => LocationField::Prompts,
            Self::Resources => LocationField::Resources,
            Self::ResourceTemplates => LocationField::ResourceTemplates,
        }
    }

    fn location(self) -> Location {
        Location::root(self.location_field())
    }

    fn response_location(self) -> Location {
        let method = match self {
            Self::Tools => LocationField::ToolsList,
            Self::Prompts => LocationField::PromptsList,
            Self::Resources => LocationField::ResourcesList,
            Self::ResourceTemplates => LocationField::ResourceTemplatesList,
        };
        Location::root(method).field(LocationField::Response)
    }
}

fn classify_json_rpc_error(object: &Map<String, Value>) -> Option<JsonRpcErrorKind> {
    let error = object.get("error")?.as_object()?;
    let code = error.get("code")?.as_number()?;
    if !code.is_i64() && !code.is_u64() {
        return None;
    }
    error.get("message").filter(|value| value.is_string())?;
    Some(
        code.as_i64()
            .map(JsonRpcErrorKind::from_code)
            .unwrap_or(JsonRpcErrorKind::Other),
    )
}

fn is_unsupported_protocol_error(object: &Map<String, Value>) -> bool {
    let Some(error) = object.get("error").and_then(Value::as_object) else {
        return false;
    };
    let Some(data) = error.get("data").and_then(Value::as_object) else {
        return false;
    };
    let Some(supported) = data.get("supported").and_then(Value::as_array) else {
        return false;
    };
    error.get("code").and_then(Value::as_i64) == Some(-32022)
        && error.get("message").is_some_and(Value::is_string)
        && data.get("requested").and_then(Value::as_str) == Some(PROTOCOL_REVISION)
        && u64::try_from(supported.len()).unwrap_or(u64::MAX)
            <= DiagnosticLimits::DEFAULTS.values().protocol_revisions
        && supported.iter().all(Value::is_string)
        && !supported
            .iter()
            .any(|revision| revision.as_str() == Some(PROTOCOL_REVISION))
}

fn unsupported_protocol_revision_limit(object: &Map<String, Value>) -> Option<LimitViolation> {
    let error = object.get("error")?.as_object()?;
    let data = error.get("data")?.as_object()?;
    let supported = data.get("supported")?.as_array()?;
    if error.get("code").and_then(Value::as_i64) != Some(-32022)
        || !error.get("message").is_some_and(Value::is_string)
        || data.get("requested").and_then(Value::as_str) != Some(PROTOCOL_REVISION)
    {
        return None;
    }
    LimitViolation::new(
        LimitKind::ProtocolRevisions,
        u64::try_from(supported.len()).unwrap_or(u64::MAX),
        DiagnosticLimits::DEFAULTS.values().protocol_revisions,
    )
    .ok()
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum RequestKind {
    Initialize,
    Discover,
    Catalog(CatalogKind),
}

#[derive(Clone, Eq, PartialEq)]
struct RequestRecord {
    id: i64,
    kind: RequestKind,
    page: usize,
    cursor: Option<String>,
}

impl fmt::Debug for RequestRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestRecord")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("page", &self.page)
            .field("cursor", &self.cursor.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

/// Drives the selected lifecycle and only capability-gated list requests. It
/// never constructs `tools/call`, `prompts/get`, or `resources/read`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum AutoDiscoveryOutcome {
    NotApplicable,
    Pending,
    Modern,
    LegacySignal,
    Terminal,
}

pub(crate) struct PassiveCatalogConversation {
    revision: SupportedRevision,
    started: bool,
    stopped: bool,
    initialized_sent: bool,
    negotiated_revision: Option<super::protocol::KnownRevision>,
    next_id: i64,
    queue: VecDeque<CatalogKind>,
    records: Vec<RequestRecord>,
    pages: BTreeMap<CatalogKind, usize>,
    seen_cursors: BTreeMap<CatalogKind, BTreeSet<String>>,
    observed_items: u64,
    maximum_items: u64,
    validate_http_headers: bool,
    auto_discovery: bool,
    allow_legacy_counteroffer: bool,
    auto_discovery_outcome: AutoDiscoveryOutcome,
    auto_selected_revision: Option<SupportedRevision>,
}

impl PassiveCatalogConversation {
    pub(crate) fn new() -> Self {
        Self::for_revision(SupportedRevision::CURRENT)
    }

    pub(crate) fn for_revision(revision: SupportedRevision) -> Self {
        Self::with_catalog_limit(revision, DiagnosticLimits::DEFAULTS.values().catalog_items)
    }

    fn with_catalog_limit(revision: SupportedRevision, maximum_items: u64) -> Self {
        Self::with_options(revision, maximum_items, false, false)
    }

    fn with_options(
        revision: SupportedRevision,
        maximum_items: u64,
        auto_discovery: bool,
        allow_legacy_counteroffer: bool,
    ) -> Self {
        Self {
            revision,
            started: false,
            stopped: false,
            initialized_sent: false,
            negotiated_revision: None,
            next_id: 1,
            queue: VecDeque::new(),
            records: Vec::new(),
            pages: BTreeMap::new(),
            seen_cursors: BTreeMap::new(),
            observed_items: 0,
            maximum_items,
            validate_http_headers: false,
            auto_discovery,
            allow_legacy_counteroffer,
            auto_discovery_outcome: if auto_discovery {
                AutoDiscoveryOutcome::Pending
            } else {
                AutoDiscoveryOutcome::NotApplicable
            },
            auto_selected_revision: None,
        }
    }

    pub(crate) fn for_auto_modern() -> Self {
        Self::with_options(
            SupportedRevision::CURRENT,
            DiagnosticLimits::DEFAULTS.values().catalog_items,
            true,
            false,
        )
    }

    pub(crate) fn for_auto_legacy() -> Self {
        Self::with_options(
            SupportedRevision::V2025_11_25,
            DiagnosticLimits::DEFAULTS.values().catalog_items,
            false,
            true,
        )
    }

    pub(crate) fn new_http() -> Self {
        let mut conversation = Self::new();
        conversation.validate_http_headers = true;
        conversation
    }

    pub(crate) fn new_http_for_revision(revision: SupportedRevision) -> Self {
        let mut conversation = Self::for_revision(revision);
        conversation.validate_http_headers = !revision.uses_initialize();
        conversation
    }

    pub(crate) fn new_http_for_auto_modern() -> Self {
        let mut conversation = Self::for_auto_modern();
        conversation.validate_http_headers = true;
        conversation
    }

    pub(crate) fn new_http_for_auto_legacy() -> Self {
        Self::for_auto_legacy()
    }

    pub(crate) const fn revision(&self) -> SupportedRevision {
        self.revision
    }

    pub(crate) const fn negotiated_revision(&self) -> Option<super::protocol::KnownRevision> {
        self.negotiated_revision
    }

    pub(crate) const fn auto_discovery_outcome(&self) -> AutoDiscoveryOutcome {
        self.auto_discovery_outcome
    }

    pub(crate) const fn auto_selected_revision(&self) -> Option<SupportedRevision> {
        self.auto_selected_revision
    }

    fn record_request(&mut self, kind: RequestKind, cursor: Option<String>) -> ProbeRequest {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("the bounded message count keeps request ids representable");
        let page = match kind {
            RequestKind::Initialize | RequestKind::Discover => 0,
            RequestKind::Catalog(catalog) => {
                let page = self.pages.entry(catalog).or_default();
                let current = *page;
                *page = page.saturating_add(1);
                current
            }
        };
        let bytes = encode_request(id, kind, cursor.as_deref(), self.revision);
        self.records.push(RequestRecord {
            id,
            kind,
            page,
            cursor,
        });
        ProbeRequest::new(id, bytes).with_protocol_revision(self.revision.as_str())
    }

    fn advance_after(&mut self, response: &ProbeResponse) -> Option<(RequestKind, Option<String>)> {
        let record = self
            .records
            .last()
            .expect("a previous response has a matching local request record");
        assert_eq!(
            response.request_id(),
            record.id,
            "the transport already matched the response id"
        );

        match record.kind {
            RequestKind::Initialize => unreachable!("initialize advances through its notification"),
            RequestKind::Discover => {
                if self.auto_discovery {
                    self.auto_discovery_outcome = classify_auto_discovery(response);
                    if self.auto_discovery_outcome != AutoDiscoveryOutcome::Modern {
                        self.stopped = true;
                        return None;
                    }
                    self.auto_selected_revision = selected_modern_revision(response);
                    if self.auto_selected_revision.is_none() {
                        self.stopped = true;
                        return None;
                    }
                }
                self.queue = advertised_catalogs(response).into();
                self.queue
                    .pop_front()
                    .map(|kind| (RequestKind::Catalog(kind), None))
            }
            RequestKind::Catalog(kind) => {
                let (items, next_cursor) = catalog_page_summary(response, kind);
                self.observed_items = self.observed_items.saturating_add(items);
                if self.observed_items > self.maximum_items {
                    self.stopped = true;
                    return None;
                }

                if let Some(cursor) = next_cursor {
                    let cursors = self.seen_cursors.entry(kind).or_default();
                    if cursors.insert(cursor.clone()) {
                        return Some((RequestKind::Catalog(kind), Some(cursor)));
                    }
                }

                self.queue
                    .pop_front()
                    .map(|next| (RequestKind::Catalog(next), None))
            }
        }
    }
}

impl Default for PassiveCatalogConversation {
    fn default() -> Self {
        Self::new()
    }
}

impl Conversation for PassiveCatalogConversation {
    fn next_request(&mut self, previous: Option<&ProbeResponse>) -> Option<ProbeRequest> {
        if self.stopped {
            return None;
        }
        if !self.started {
            assert!(previous.is_none(), "discovery is always the first exchange");
            self.started = true;
            let kind = if self.revision.uses_initialize() {
                RequestKind::Initialize
            } else {
                RequestKind::Discover
            };
            return Some(self.record_request(kind, None));
        }

        let response = previous.expect("each later request follows a matching response");
        if self.revision.uses_initialize()
            && self
                .records
                .last()
                .is_some_and(|record| record.kind == RequestKind::Initialize)
        {
            if !self.initialized_sent {
                self.negotiated_revision = negotiated_revision(response);
                if self.allow_legacy_counteroffer
                    && let Some(selected) = self
                        .negotiated_revision
                        .and_then(super::protocol::KnownRevision::supported)
                    && selected.uses_initialize()
                {
                    self.revision = selected;
                }
                let Some(catalogs) = legacy_advertised_catalogs(response, self.revision) else {
                    self.stopped = true;
                    return None;
                };
                self.queue = catalogs.into();
                self.initialized_sent = true;
                return Some(initialized_notification(self.revision));
            }
            return self
                .queue
                .pop_front()
                .map(|kind| self.record_request(RequestKind::Catalog(kind), None));
        }
        self.advance_after(response)
            .map(|(kind, cursor)| self.record_request(kind, cursor))
    }
}

fn encode_request(
    id: i64,
    kind: RequestKind,
    cursor: Option<&str>,
    revision: SupportedRevision,
) -> Vec<u8> {
    if kind == RequestKind::Initialize {
        return serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "initialize",
            "params": {
                "protocolVersion": revision.as_str(),
                "capabilities": {},
                "clientInfo": {
                    "name": "mcp-doctor",
                    "version": env!("CARGO_PKG_VERSION"),
                },
            },
        }))
        .expect("the typed initialize request must serialize");
    }
    let method = match kind {
        RequestKind::Initialize => unreachable!("initialize was encoded above"),
        RequestKind::Discover => "server/discover",
        RequestKind::Catalog(kind) => kind.method(),
    };
    let mut params = Map::new();
    if !revision.uses_initialize() {
        params.insert("_meta".to_owned(), request_meta());
    }
    if let Some(cursor) = cursor {
        params.insert("cursor".to_owned(), Value::String(cursor.to_owned()));
    }

    serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    }))
    .expect("the typed passive request must serialize")
}

fn initialized_notification(revision: SupportedRevision) -> ProbeRequest {
    let bytes = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
    }))
    .expect("the typed initialized notification must serialize");
    ProbeRequest::notification(bytes).with_protocol_revision(revision.as_str())
}

fn classify_auto_discovery(response: &ProbeResponse) -> AutoDiscoveryOutcome {
    let Ok(value) = serde_json::from_slice::<Value>(response.as_bytes()) else {
        return AutoDiscoveryOutcome::Terminal;
    };
    let Some(object) = value.as_object() else {
        return AutoDiscoveryOutcome::Terminal;
    };
    if let Some(error) = object.get("error") {
        let Some(error) = error.as_object() else {
            return AutoDiscoveryOutcome::Terminal;
        };
        let Some(code) = error.get("code").and_then(Value::as_i64) else {
            return AutoDiscoveryOutcome::Terminal;
        };
        if !error.get("message").is_some_and(Value::is_string) {
            return AutoDiscoveryOutcome::Terminal;
        }
        return if matches!(code, -32022..=-32020) {
            AutoDiscoveryOutcome::Terminal
        } else {
            AutoDiscoveryOutcome::LegacySignal
        };
    }
    AutoDiscoveryOutcome::Modern
}

fn selected_modern_revision(response: &ProbeResponse) -> Option<SupportedRevision> {
    let value: Value = serde_json::from_slice(response.as_bytes()).ok()?;
    let result = value.get("result")?.as_object()?;
    let base = Location::root(LocationField::Server);
    if !validate_cacheable_result(result, base.clone()).is_empty()
        || !validate_discovery_capabilities(result, base).0.is_empty()
    {
        return None;
    }
    let supported = result.get("supportedVersions")?.as_array()?;
    (u64::try_from(supported.len()).unwrap_or(u64::MAX)
        <= DiagnosticLimits::DEFAULTS.values().protocol_revisions
        && supported.iter().all(Value::is_string)
        && supported
            .iter()
            .any(|revision| revision.as_str() == Some(PROTOCOL_REVISION)))
    .then_some(SupportedRevision::CURRENT)
}

fn negotiated_revision(response: &ProbeResponse) -> Option<super::protocol::KnownRevision> {
    let value: Value = serde_json::from_slice(response.as_bytes()).ok()?;
    let revision = value.get("result")?.get("protocolVersion")?.as_str()?;
    super::protocol::KnownRevision::parse(revision)
}

fn legacy_advertised_catalogs(
    response: &ProbeResponse,
    revision: SupportedRevision,
) -> Option<Vec<CatalogKind>> {
    let value: Value = serde_json::from_slice(response.as_bytes()).ok()?;
    let result = value.get("result")?.as_object()?;
    if result.get("protocolVersion")?.as_str()? != revision.as_str() {
        return None;
    }
    let capabilities = result.get("capabilities")?.as_object()?;
    let server_info = result.get("serverInfo")?.as_object()?;
    if !server_info.get("name").is_some_and(Value::is_string)
        || !server_info.get("version").is_some_and(Value::is_string)
        || result
            .get("instructions")
            .is_some_and(|instructions| !instructions.is_string())
    {
        return None;
    }
    let (capability_findings, _) = validate_legacy_capabilities(
        result,
        Location::root(LocationField::Server).field(LocationField::Result),
        revision,
    );
    if capability_findings
        .iter()
        .any(|finding| finding.severity().is_failure())
    {
        return None;
    }
    Some(
        CatalogKind::ALL
            .into_iter()
            .filter(|kind| {
                capabilities
                    .get(kind.capability())
                    .is_some_and(Value::is_object)
            })
            .collect(),
    )
}

pub(super) fn request_meta() -> Value {
    json!({
        "io.modelcontextprotocol/clientCapabilities": {},
        "io.modelcontextprotocol/clientInfo": {
            "name": "mcp-doctor",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "io.modelcontextprotocol/protocolVersion": PROTOCOL_REVISION,
    })
}

fn advertised_catalogs(response: &ProbeResponse) -> Vec<CatalogKind> {
    let Ok(value) = serde_json::from_slice::<Value>(response.as_bytes()) else {
        return Vec::new();
    };
    let Some(result) = value.get("result").and_then(Value::as_object) else {
        return Vec::new();
    };
    if result.get("resultType").and_then(Value::as_str) != Some("complete") {
        return Vec::new();
    }
    let Some(versions) = result.get("supportedVersions").and_then(Value::as_array) else {
        return Vec::new();
    };
    if !versions
        .iter()
        .any(|version| version.as_str() == Some(PROTOCOL_REVISION))
    {
        return Vec::new();
    }
    let Some(capabilities) = result.get("capabilities").and_then(Value::as_object) else {
        return Vec::new();
    };

    CatalogKind::ALL
        .into_iter()
        .filter(|kind| {
            capabilities
                .get(kind.capability())
                .is_some_and(Value::is_object)
        })
        .collect()
}

fn catalog_page_summary(response: &ProbeResponse, kind: CatalogKind) -> (u64, Option<String>) {
    let Ok(value) = serde_json::from_slice::<Value>(response.as_bytes()) else {
        return (0, None);
    };
    let Some(result) = value.get("result").and_then(Value::as_object) else {
        return (0, None);
    };
    let items = result
        .get(kind.result_field())
        .and_then(Value::as_array)
        .map_or(0, |items| u64::try_from(items.len()).unwrap_or(u64::MAX));
    let next_cursor = result
        .get("nextCursor")
        .and_then(Value::as_str)
        .map(str::to_owned);
    (items, next_cursor)
}

#[derive(Debug, Clone, Copy)]
enum FindingBucket {
    Revision,
    Envelope,
    Catalog,
    Quality,
    Schema,
    Security,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ToolDescriptionDiagnosis {
    Usable,
    MissingOrBlank(Finding),
    PlaceholderOrNameOnly(Finding),
    Invalid(Finding),
}

fn diagnose_tool_description(
    revision: SupportedRevision,
    tool_location: Location,
    tool: &Map<String, Value>,
) -> ToolDescriptionDiagnosis {
    let location = tool_location.field(LocationField::Description);
    match tool.get("description") {
        None => ToolDescriptionDiagnosis::MissingOrBlank(
            Finding::tool_description_missing_or_blank(revision, location),
        ),
        Some(Value::String(description)) if is_a1_v1_blank(description) => {
            ToolDescriptionDiagnosis::MissingOrBlank(Finding::tool_description_missing_or_blank(
                revision, location,
            ))
        }
        Some(Value::String(description))
            if is_a1_v1_placeholder_or_name_only(
                tool.get("name").and_then(Value::as_str),
                description,
            ) =>
        {
            ToolDescriptionDiagnosis::PlaceholderOrNameOnly(
                Finding::tool_description_placeholder_or_name_only(revision, location),
            )
        }
        Some(Value::String(_)) => ToolDescriptionDiagnosis::Usable,
        Some(description) => ToolDescriptionDiagnosis::Invalid(Finding::catalog_contract_invalid(
            revision,
            location,
            RuleViolation::ExpectedShape {
                expected: ExpectedShape::String,
                observed: json_kind(Some(description)),
            },
        )),
    }
}

fn is_a1_v1_blank(description: &str) -> bool {
    description.chars().all(is_a1_v1_whitespace)
}

const fn is_a1_v1_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'..='\u{000D}'
            | '\u{0020}'
            | '\u{0085}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

const A1_V1_PLACEHOLDERS: [&str; 5] = ["todo", "tbd", "tool", "description", "placeholder"];

fn is_a1_v1_placeholder_or_name_only(name: Option<&str>, description: &str) -> bool {
    A1_V1_PLACEHOLDERS
        .into_iter()
        .any(|placeholder| a1_v1_normalized_eq(description, placeholder))
        || name.is_some_and(|name| a1_v1_normalized_eq(description, name))
}

fn a1_v1_normalized_eq(left: &str, right: &str) -> bool {
    A1V1Normalized::new(left).eq(A1V1Normalized::new(right))
}

/// A value-free iterator for the fixed A1 comparison. ASCII whitespace is
/// trimmed at the boundaries and collapsed between retained characters,
/// ASCII punctuation is omitted, and only ASCII letters are case-folded.
/// Non-ASCII scalars are compared exactly and are never transliterated.
struct A1V1Normalized<'a> {
    characters: std::str::Chars<'a>,
    pending_space: bool,
    emitted: bool,
    buffered: Option<char>,
}

impl<'a> A1V1Normalized<'a> {
    fn new(value: &'a str) -> Self {
        Self {
            characters: value.chars(),
            pending_space: false,
            emitted: false,
            buffered: None,
        }
    }
}

impl Iterator for A1V1Normalized<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(character) = self.buffered.take() {
            self.emitted = true;
            return Some(character);
        }
        for character in self.characters.by_ref() {
            if character.is_ascii_punctuation() {
                continue;
            }
            if character.is_ascii_whitespace() {
                if self.emitted {
                    self.pending_space = true;
                }
                continue;
            }
            let character = character.to_ascii_lowercase();
            if self.pending_space {
                self.pending_space = false;
                self.buffered = Some(character);
                return Some(' ');
            }
            self.emitted = true;
            return Some(character);
        }
        None
    }
}

const CREDENTIAL_IDENTIFIER_SEGMENTS: [&str; 6] = [
    "password",
    "passwd",
    "secret",
    "token",
    "apikey",
    "credential",
];
const CREDENTIAL_IDENTIFIER_PAIRS: [(&str, &str); 3] =
    [("api", "key"), ("access", "token"), ("private", "key")];

fn has_credential_identifier_segment(identifier: &str) -> bool {
    let normalized = normalize_credential_identifier(identifier);
    let segments = normalized.split('-').collect::<Vec<_>>();
    segments
        .iter()
        .any(|segment| CREDENTIAL_IDENTIFIER_SEGMENTS.contains(segment))
        || segments.windows(2).any(|pair| {
            CREDENTIAL_IDENTIFIER_PAIRS
                .iter()
                .any(|expected| pair == [expected.0, expected.1])
        })
}

/// Normalizes only ASCII identifier structure: punctuation and non-ASCII
/// scalars delimit segments, ASCII letters are case-folded, and camel-case or
/// acronym-to-word transitions add a delimiter. No locale or semantic
/// inference participates in the credential rule.
fn normalize_credential_identifier(identifier: &str) -> String {
    let mut normalized = String::with_capacity(identifier.len());
    let mut characters = identifier.chars().peekable();
    let mut previous: Option<char> = None;
    let mut pending_boundary = false;

    while let Some(character) = characters.next() {
        if !character.is_ascii_alphanumeric() {
            pending_boundary = !normalized.is_empty();
            previous = None;
            continue;
        }

        let camel_boundary = character.is_ascii_uppercase()
            && previous.is_some_and(|previous| {
                previous.is_ascii_lowercase()
                    || previous.is_ascii_digit()
                    || (previous.is_ascii_uppercase()
                        && characters.peek().is_some_and(char::is_ascii_lowercase))
            });
        if (pending_boundary || camel_boundary) && !normalized.ends_with('-') {
            normalized.push('-');
        }
        normalized.push(character.to_ascii_lowercase());
        pending_boundary = false;
        previous = Some(character);
    }

    normalized
}

#[derive(Default)]
struct CredentialLiteralScan {
    findings: Vec<Finding>,
    limit: Option<(Location, LimitViolation)>,
}

fn scan_credential_literals(
    revision: SupportedRevision,
    schema: &Value,
    base: Location,
    budget: &SchemaWorkBudget,
) -> CredentialLiteralScan {
    let Some(properties) = schema
        .as_object()
        .and_then(|schema| schema.get("properties"))
        .and_then(Value::as_object)
    else {
        return CredentialLiteralScan::default();
    };
    let mut scan = CredentialLiteralScan::default();

    for (property_index, (identifier, property_schema)) in properties.iter().enumerate() {
        let property_location = base
            .clone()
            .field(LocationField::Properties)
            .index(property_index);
        let identifier_work = u64::try_from(identifier.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        if !budget.observe(identifier_work) {
            scan.limit = Some((property_location, budget.violation()));
            return scan;
        }
        if !has_credential_identifier_segment(identifier) {
            continue;
        }
        let Some(property_schema) = property_schema.as_object() else {
            continue;
        };

        for keyword in CredentialSchemaKeyword::ALL {
            let keyword_location = property_location.clone().field(keyword.location_field());
            if !budget.observe(1) {
                scan.limit = Some((keyword_location, budget.violation()));
                return scan;
            }
            let Some(value) = property_schema.get(keyword.as_str()) else {
                continue;
            };
            let literal_count = match keyword {
                CredentialSchemaKeyword::Default | CredentialSchemaKeyword::Const => {
                    u64::from(value.as_str().is_some_and(|literal| !literal.is_empty()))
                }
                CredentialSchemaKeyword::Examples | CredentialSchemaKeyword::Enum => {
                    let mut count = 0_u64;
                    for candidate in value.as_array().into_iter().flatten() {
                        if !budget.observe(1) {
                            scan.limit = Some((keyword_location, budget.violation()));
                            return scan;
                        }
                        if candidate
                            .as_str()
                            .is_some_and(|literal| !literal.is_empty())
                        {
                            count = count.saturating_add(1);
                        }
                    }
                    count
                }
            };
            if literal_count > 0 {
                scan.findings.push(Finding::credential_literal_exposed(
                    revision,
                    keyword_location,
                    keyword,
                    literal_count,
                ));
            }
        }
    }

    scan
}

struct Analyzer {
    limits: DiagnosticLimits,
    revision: Vec<Finding>,
    envelope: Vec<Finding>,
    catalog: Vec<Finding>,
    quality: Vec<Finding>,
    schema: Vec<Finding>,
    security: Vec<Finding>,
    stored_findings: usize,
    finding_capacity: usize,
    finding_overflow: bool,
    validate_http_headers: bool,
    discovery_valid: bool,
    revision_checked: bool,
    revision_supported: bool,
    revision_block: Option<SkipReason>,
    catalog_limit_reached: bool,
    tools_catalog_valid: bool,
    tools_advertised: bool,
    item_offsets: BTreeMap<CatalogKind, usize>,
    observed_catalog_items: u64,
    identifiers: BTreeMap<CatalogKind, BTreeMap<String, usize>>,
    secondary_identifiers: BTreeMap<CatalogKind, BTreeMap<String, usize>>,
    returned_cursors: BTreeMap<CatalogKind, BTreeSet<String>>,
    auto_discovery: bool,
}

impl Analyzer {
    fn new(reserved_findings: usize, validate_http_headers: bool, auto_discovery: bool) -> Self {
        let limits = DiagnosticLimits::DEFAULTS;
        let maximum = usize::try_from(limits.values().report_findings).unwrap_or(usize::MAX);
        Self {
            limits,
            revision: Vec::new(),
            envelope: Vec::new(),
            catalog: Vec::new(),
            quality: Vec::new(),
            schema: Vec::new(),
            security: Vec::new(),
            stored_findings: 0,
            finding_capacity: maximum.saturating_sub(reserved_findings),
            finding_overflow: false,
            validate_http_headers,
            discovery_valid: false,
            revision_checked: false,
            revision_supported: false,
            revision_block: None,
            catalog_limit_reached: false,
            tools_catalog_valid: true,
            tools_advertised: false,
            item_offsets: BTreeMap::new(),
            observed_catalog_items: 0,
            identifiers: BTreeMap::new(),
            secondary_identifiers: BTreeMap::new(),
            returned_cursors: BTreeMap::new(),
            auto_discovery,
        }
    }

    fn push(&mut self, bucket: FindingBucket, finding: Finding) {
        let duplicate = match bucket {
            FindingBucket::Revision => self.revision.contains(&finding),
            FindingBucket::Envelope => self.envelope.contains(&finding),
            FindingBucket::Catalog => self.catalog.contains(&finding),
            FindingBucket::Quality => self.quality.contains(&finding),
            FindingBucket::Schema => self.schema.contains(&finding),
            FindingBucket::Security => self.security.contains(&finding),
        };
        if duplicate {
            return;
        }
        if self.stored_findings < self.finding_capacity {
            match bucket {
                FindingBucket::Revision => self.revision.push(finding),
                FindingBucket::Envelope => self.envelope.push(finding),
                FindingBucket::Catalog => self.catalog.push(finding),
                FindingBucket::Quality => self.quality.push(finding),
                FindingBucket::Schema => self.schema.push(finding),
                FindingBucket::Security => self.security.push(finding),
            }
            self.stored_findings += 1;
        } else if matches!(bucket, FindingBucket::Security) {
            let displaced = self
                .quality
                .pop()
                .or_else(|| self.schema.pop())
                .or_else(|| self.catalog.pop());
            if displaced.is_some() {
                self.security.push(finding);
            }
            self.finding_overflow = true;
        } else if !matches!(bucket, FindingBucket::Quality) && self.quality.pop().is_some() {
            match bucket {
                FindingBucket::Revision => self.revision.push(finding),
                FindingBucket::Envelope => self.envelope.push(finding),
                FindingBucket::Catalog => self.catalog.push(finding),
                FindingBucket::Quality => unreachable!("quality findings do not displace findings"),
                FindingBucket::Schema => self.schema.push(finding),
                FindingBucket::Security => unreachable!("security findings use priority storage"),
            }
            self.finding_overflow = true;
        } else {
            self.finding_overflow = true;
        }
    }

    fn finish_overflow(&mut self) {
        if !self.finding_overflow || self.finding_capacity == 0 {
            return;
        }
        let removed = self
            .quality
            .pop()
            .or_else(|| self.schema.pop())
            .or_else(|| self.catalog.pop())
            .or_else(|| self.envelope.pop())
            .or_else(|| self.revision.pop())
            .or_else(|| self.security.pop());
        if removed.is_some() {
            self.stored_findings = self.stored_findings.saturating_sub(1);
        }
        let maximum = self.limits.values().report_findings;
        let violation = LimitViolation::new(
            LimitKind::ReportFindings,
            maximum.saturating_add(1),
            maximum,
        )
        .expect("the synthetic overflow observation exceeds the report maximum");
        self.catalog.push(Finding::limit_exceeded(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Server),
            violation,
        ));
        self.stored_findings += 1;
    }

    fn analyze_discovery(&mut self, response: &ProbeResponse) {
        let value: Value = serde_json::from_slice(response.as_bytes())
            .expect("the transport accepted this JSON response");
        let Some(object) = value.as_object() else {
            unreachable!("the transport accepted only JSON-RPC objects")
        };
        if let Some(error) = classify_json_rpc_error(object) {
            self.revision_checked = true;
            if self.auto_discovery
                && let Some(violation) = unsupported_protocol_revision_limit(object)
            {
                self.revision_block = Some(SkipReason::LimitReached);
                self.push(
                    FindingBucket::Revision,
                    Finding::limit_exceeded(
                        SupportedRevision::CURRENT,
                        Location::root(LocationField::ServerDiscover)
                            .field(LocationField::Response),
                        violation,
                    ),
                );
            } else if self.auto_discovery && is_unsupported_protocol_error(object) {
                self.revision_block = Some(SkipReason::UnsupportedRevision);
                self.push(
                    FindingBucket::Revision,
                    Finding::unsupported_protocol_version(
                        SupportedRevision::CURRENT,
                        Location::root(LocationField::ServerDiscover)
                            .field(LocationField::Response),
                    ),
                );
            } else {
                self.revision_block = Some(SkipReason::PrerequisiteFailed);
                self.push(
                    FindingBucket::Revision,
                    Finding::lifecycle_method_rejected(
                        SupportedRevision::CURRENT,
                        Location::root(LocationField::ServerDiscover)
                            .field(LocationField::Response),
                        error,
                    ),
                );
            }
            return;
        } else if object.contains_key("error") {
            self.push(
                FindingBucket::Envelope,
                Finding::catalog_contract_invalid(
                    SupportedRevision::CURRENT,
                    Location::root(LocationField::Server),
                    RuleViolation::ServerErrorResponse,
                ),
            );
            return;
        }
        let Some(result) = object.get("result").and_then(Value::as_object) else {
            self.expected_shape(
                FindingBucket::Envelope,
                Location::root(LocationField::Server).field(LocationField::Result),
                ExpectedShape::Object,
                object.get("result"),
            );
            return;
        };

        let base = Location::root(LocationField::Server);
        let common_valid =
            self.analyze_cacheable_result(result, base.clone(), FindingBucket::Envelope);
        let revision_valid = self.analyze_supported_versions(result, base.clone());
        let capabilities_valid = self.analyze_capabilities(result, base);
        self.discovery_valid = common_valid && revision_valid && capabilities_valid;
    }

    fn analyze_supported_versions(&mut self, result: &Map<String, Value>, base: Location) -> bool {
        self.revision_checked = true;
        let location = base.field(LocationField::SupportedVersions);
        let Some(versions) = result.get("supportedVersions").and_then(Value::as_array) else {
            let observed = result.get("supportedVersions").map_or(0, serialized_len);
            self.push(
                FindingBucket::Revision,
                Finding::invalid_revision_value(
                    SupportedRevision::CURRENT,
                    location,
                    super::redaction::RedactedValue::new(observed),
                ),
            );
            return false;
        };
        let mut strings = Vec::with_capacity(versions.len());
        for (index, version) in versions.iter().enumerate() {
            let Some(version) = version.as_str() else {
                self.push(
                    FindingBucket::Revision,
                    Finding::invalid_revision_value(
                        SupportedRevision::CURRENT,
                        location.clone().index(index),
                        super::redaction::RedactedValue::new(serialized_len(version)),
                    ),
                );
                return false;
            };
            strings.push(version);
        }

        match select_current_modern_revision(
            strings.iter().copied(),
            self.limits.values().protocol_revisions,
        ) {
            RevisionSelection::Selected(revision) => {
                self.revision_supported = true;
                self.push(
                    FindingBucket::Revision,
                    Finding::revision_confirmed(revision, location),
                );
                true
            }
            RevisionSelection::Unsupported(summary) => {
                self.revision_block = Some(SkipReason::UnsupportedRevision);
                self.push(
                    FindingBucket::Revision,
                    Finding::unsupported_revision(SupportedRevision::CURRENT, location, summary),
                );
                false
            }
            RevisionSelection::LimitExceeded(violation) => {
                self.revision_block = Some(SkipReason::LimitReached);
                self.push(
                    FindingBucket::Revision,
                    Finding::limit_exceeded(SupportedRevision::CURRENT, location, violation),
                );
                false
            }
        }
    }

    fn analyze_capabilities(&mut self, result: &Map<String, Value>, base: Location) -> bool {
        let (findings, tools_advertised) = validate_discovery_capabilities(result, base);
        let valid = findings.is_empty();
        self.tools_advertised = tools_advertised;
        for finding in findings {
            self.push(FindingBucket::Envelope, finding);
        }
        valid
    }

    fn analyze_cacheable_result(
        &mut self,
        result: &Map<String, Value>,
        base: Location,
        bucket: FindingBucket,
    ) -> bool {
        let findings = validate_cacheable_result(result, base);
        let valid = findings.is_empty();
        for finding in findings {
            self.push(bucket, finding);
        }
        valid
    }

    fn expected_shape(
        &mut self,
        bucket: FindingBucket,
        location: Location,
        expected: ExpectedShape,
        observed: Option<&Value>,
    ) {
        self.push(
            bucket,
            Finding::catalog_contract_invalid(
                SupportedRevision::CURRENT,
                location,
                RuleViolation::ExpectedShape {
                    expected,
                    observed: json_kind(observed),
                },
            ),
        );
    }

    fn analyze_catalog_page(&mut self, record: &RequestRecord, response: &ProbeResponse) {
        let RequestKind::Catalog(kind) = record.kind else {
            return;
        };
        let value: Value = serde_json::from_slice(response.as_bytes())
            .expect("the transport accepted this JSON response");
        let object = value
            .as_object()
            .expect("the transport accepted a JSON-RPC object");
        let base = kind.location();
        if let Some(error) = classify_json_rpc_error(object) {
            self.push(
                FindingBucket::Catalog,
                Finding::catalog_method_rejected(
                    SupportedRevision::CURRENT,
                    kind.response_location(),
                    error,
                ),
            );
            if kind == CatalogKind::Tools {
                self.tools_catalog_valid = false;
            }
            return;
        } else if object.contains_key("error") {
            self.push(
                FindingBucket::Catalog,
                Finding::catalog_contract_invalid(
                    SupportedRevision::CURRENT,
                    base,
                    RuleViolation::ServerErrorResponse,
                ),
            );
            if kind == CatalogKind::Tools {
                self.tools_catalog_valid = false;
            }
            return;
        }
        let Some(result) = object.get("result").and_then(Value::as_object) else {
            self.expected_shape(
                FindingBucket::Catalog,
                base.clone().field(LocationField::Result),
                ExpectedShape::Object,
                object.get("result"),
            );
            if kind == CatalogKind::Tools {
                self.tools_catalog_valid = false;
            }
            return;
        };

        let mut page_valid =
            self.analyze_cacheable_result(result, base.clone(), FindingBucket::Catalog);
        if let Some(cursor) = result.get("nextCursor") {
            if let Some(cursor) = cursor.as_str() {
                let cursors = self.returned_cursors.entry(kind).or_default();
                if !cursors.insert(cursor.to_owned()) {
                    self.push(
                        FindingBucket::Catalog,
                        Finding::pagination_cursor_repeated(
                            SupportedRevision::CURRENT,
                            base.clone().field(LocationField::NextCursor),
                        ),
                    );
                    page_valid = false;
                }
            } else {
                self.expected_shape(
                    FindingBucket::Catalog,
                    base.clone().field(LocationField::NextCursor),
                    ExpectedShape::String,
                    Some(cursor),
                );
                page_valid = false;
            }
        }

        let Some(items) = result.get(kind.result_field()).and_then(Value::as_array) else {
            self.expected_shape(
                FindingBucket::Catalog,
                base,
                ExpectedShape::Array,
                result.get(kind.result_field()),
            );
            if kind == CatalogKind::Tools {
                self.tools_catalog_valid = false;
            }
            return;
        };

        let offset = *self.item_offsets.get(&kind).unwrap_or(&0);
        let page_items = u64::try_from(items.len()).unwrap_or(u64::MAX);
        self.observed_catalog_items = self.observed_catalog_items.saturating_add(page_items);
        let observed = self.observed_catalog_items;
        let maximum = self.limits.values().catalog_items;
        if observed > maximum {
            let violation = LimitViolation::new(LimitKind::CatalogItems, observed, maximum)
                .expect("the observed catalog count exceeds its maximum");
            self.push(
                FindingBucket::Catalog,
                Finding::limit_exceeded(SupportedRevision::CURRENT, kind.location(), violation),
            );
            self.catalog_limit_reached = true;
            if kind == CatalogKind::Tools {
                self.tools_catalog_valid = false;
            }
        }

        let previously_observed = observed.saturating_sub(page_items);
        let remaining =
            usize::try_from(maximum.saturating_sub(previously_observed)).unwrap_or(usize::MAX);
        for (page_index, item) in items.iter().take(remaining).enumerate() {
            let index = offset.saturating_add(page_index);
            self.analyze_item(kind, index, item);
        }
        self.item_offsets
            .insert(kind, offset.saturating_add(items.len()));
        if kind == CatalogKind::Tools && !page_valid {
            self.tools_catalog_valid = false;
        }
    }

    fn analyze_item(&mut self, kind: CatalogKind, index: usize, item: &Value) {
        let location = kind.location().index(index);
        let Some(object) = item.as_object() else {
            self.expected_shape(
                FindingBucket::Catalog,
                location,
                ExpectedShape::Object,
                Some(item),
            );
            if kind == CatalogKind::Tools {
                self.tools_catalog_valid = false;
            }
            return;
        };

        self.analyze_name(kind, index, object);
        match kind {
            CatalogKind::Tools => self.analyze_tool(index, object),
            CatalogKind::Prompts => self.analyze_prompt(index, object),
            CatalogKind::Resources => self.analyze_resource(index, object),
            CatalogKind::ResourceTemplates => self.analyze_resource_template(index, object),
        }
    }

    fn analyze_name(&mut self, kind: CatalogKind, index: usize, object: &Map<String, Value>) {
        let location = kind.location().index(index).field(LocationField::Name);
        let Some(name) = object.get("name").and_then(Value::as_str) else {
            self.expected_shape(
                FindingBucket::Catalog,
                location,
                ExpectedShape::String,
                object.get("name"),
            );
            return;
        };
        let identifiers = self.identifiers.entry(kind).or_default();
        if identifiers.insert(name.to_owned(), index).is_some() {
            self.push(
                FindingBucket::Catalog,
                Finding::duplicate_catalog_identifier(SupportedRevision::CURRENT, location),
            );
        }
    }

    fn analyze_tool(&mut self, index: usize, tool: &Map<String, Value>) {
        let location = CatalogKind::Tools.location().index(index);
        match diagnose_tool_description(SupportedRevision::CURRENT, location.clone(), tool) {
            ToolDescriptionDiagnosis::Usable => {}
            ToolDescriptionDiagnosis::MissingOrBlank(finding)
            | ToolDescriptionDiagnosis::PlaceholderOrNameOnly(finding) => {
                self.push(FindingBucket::Quality, finding);
            }
            ToolDescriptionDiagnosis::Invalid(finding) => {
                self.push(FindingBucket::Catalog, finding);
                self.tools_catalog_valid = false;
            }
        }
        let Some(input_schema) = tool.get("inputSchema") else {
            self.push(
                FindingBucket::Schema,
                Finding::schema_contract_invalid(
                    SupportedRevision::CURRENT,
                    location.clone().field(LocationField::InputSchema),
                    RuleViolation::ExpectedShape {
                        expected: ExpectedShape::Object,
                        observed: JsonKind::Missing,
                    },
                ),
            );
            return;
        };
        let input_location = location.clone().field(LocationField::InputSchema);
        let Some(input_object) = input_schema.as_object() else {
            self.push(
                FindingBucket::Schema,
                Finding::schema_contract_invalid(
                    SupportedRevision::CURRENT,
                    input_location,
                    RuleViolation::ExpectedShape {
                        expected: ExpectedShape::Object,
                        observed: json_kind(Some(input_schema)),
                    },
                ),
            );
            return;
        };
        let input_root_valid = input_object.get("type").and_then(Value::as_str) == Some("object");
        if !input_root_valid {
            self.push(
                FindingBucket::Schema,
                Finding::schema_contract_invalid(
                    SupportedRevision::CURRENT,
                    input_location.clone().field(LocationField::Type),
                    RuleViolation::ExpectedInputSchemaRootObject {
                        observed: json_kind(input_object.get("type")),
                    },
                ),
            );
        }
        let schema_budget = self.analyze_schema(input_schema, input_location.clone());
        if input_root_valid && let Some(budget) = schema_budget {
            let scan = scan_credential_literals(
                SupportedRevision::CURRENT,
                input_schema,
                input_location,
                &budget,
            );
            for finding in scan.findings {
                self.push(FindingBucket::Security, finding);
            }
            if let Some((location, violation)) = scan.limit {
                self.push(
                    FindingBucket::Schema,
                    Finding::limit_exceeded(SupportedRevision::CURRENT, location, violation),
                );
            }
        }
        if self.validate_http_headers
            && let Err(finding) = validate_annotations(
                input_schema,
                location.clone().field(LocationField::InputSchema),
            )
        {
            self.push(FindingBucket::Schema, finding);
        }

        if let Some(output_schema) = tool.get("outputSchema") {
            let output_location = location.field(LocationField::OutputSchema);
            if output_schema.is_object() {
                let _ = self.analyze_schema(output_schema, output_location);
            } else {
                self.push(
                    FindingBucket::Schema,
                    Finding::schema_contract_invalid(
                        SupportedRevision::CURRENT,
                        output_location,
                        RuleViolation::ExpectedShape {
                            expected: ExpectedShape::Object,
                            observed: json_kind(Some(output_schema)),
                        },
                    ),
                );
            }
        }
    }

    fn analyze_prompt(&mut self, prompt_index: usize, prompt: &Map<String, Value>) {
        let Some(arguments) = prompt.get("arguments") else {
            return;
        };
        let base = CatalogKind::Prompts
            .location()
            .index(prompt_index)
            .field(LocationField::Arguments);
        let Some(arguments) = arguments.as_array() else {
            self.expected_shape(
                FindingBucket::Catalog,
                base,
                ExpectedShape::Array,
                Some(arguments),
            );
            return;
        };
        let mut names = BTreeSet::new();
        for (argument_index, argument) in arguments.iter().enumerate() {
            let location = base.clone().index(argument_index);
            let Some(argument) = argument.as_object() else {
                self.expected_shape(
                    FindingBucket::Catalog,
                    location,
                    ExpectedShape::Object,
                    Some(argument),
                );
                continue;
            };
            match argument.get("name").and_then(Value::as_str) {
                Some(name) if !names.insert(name.to_owned()) => self.push(
                    FindingBucket::Catalog,
                    Finding::duplicate_catalog_identifier(
                        SupportedRevision::CURRENT,
                        location.clone().field(LocationField::Name),
                    ),
                ),
                Some(_) => {}
                None => self.expected_shape(
                    FindingBucket::Catalog,
                    location.clone().field(LocationField::Name),
                    ExpectedShape::String,
                    argument.get("name"),
                ),
            }
            if let Some(required) = argument.get("required")
                && !required.is_boolean()
            {
                self.expected_shape(
                    FindingBucket::Catalog,
                    location.field(LocationField::Required),
                    ExpectedShape::Boolean,
                    Some(required),
                );
            }
        }
    }

    fn analyze_resource(&mut self, index: usize, resource: &Map<String, Value>) {
        let location = CatalogKind::Resources
            .location()
            .index(index)
            .field(LocationField::Uri);
        let Some(uri) = resource.get("uri").and_then(Value::as_str) else {
            self.expected_shape(
                FindingBucket::Catalog,
                location,
                ExpectedShape::String,
                resource.get("uri"),
            );
            return;
        };
        self.analyze_secondary_identifier(CatalogKind::Resources, uri, index, location);
    }

    fn analyze_resource_template(&mut self, index: usize, template: &Map<String, Value>) {
        let location = CatalogKind::ResourceTemplates
            .location()
            .index(index)
            .field(LocationField::UriTemplate);
        let Some(uri_template) = template.get("uriTemplate").and_then(Value::as_str) else {
            self.expected_shape(
                FindingBucket::Catalog,
                location,
                ExpectedShape::String,
                template.get("uriTemplate"),
            );
            return;
        };
        self.analyze_secondary_identifier(
            CatalogKind::ResourceTemplates,
            uri_template,
            index,
            location,
        );
    }

    fn analyze_secondary_identifier(
        &mut self,
        kind: CatalogKind,
        identifier: &str,
        index: usize,
        location: Location,
    ) {
        // Resource URI identity is independent from display-name identity. The
        // values remain transient and never enter a finding or report.
        let key = identifier.to_owned();
        let identifiers = self.secondary_identifiers.entry(kind).or_default();
        if identifiers.insert(key, index).is_some() {
            self.push(
                FindingBucket::Catalog,
                Finding::duplicate_catalog_identifier(SupportedRevision::CURRENT, location),
            );
        }
    }

    fn analyze_schema(&mut self, schema: &Value, base: Location) -> Option<Arc<SchemaWorkBudget>> {
        self.analyze_schema_with_policy(
            schema,
            base,
            LocalSchemaDialectPolicy::RevisionDefaultDraft202012,
        )
    }

    fn analyze_schema_with_policy(
        &mut self,
        schema: &Value,
        base: Location,
        policy: LocalSchemaDialectPolicy,
    ) -> Option<Arc<SchemaWorkBudget>> {
        let values = self.limits.values();
        let bytes = u64::try_from(serialized_len(schema)).unwrap_or(u64::MAX);
        if bytes > values.schema_bytes {
            self.schema_limit(base, LimitKind::SchemaBytes, bytes, values.schema_bytes);
            return None;
        }

        let Some(object) = schema.as_object() else {
            unreachable!("tool schema object shape was checked before schema analysis")
        };
        match object.get("$schema") {
            Some(dialect) if dialect.as_str() != Some(DRAFT_2020_12) => {
                self.push(
                    FindingBucket::Schema,
                    Finding::unsupported_schema_dialect(
                        SupportedRevision::CURRENT,
                        base.clone().field(LocationField::Schema),
                        json_kind(Some(dialect)),
                    ),
                );
                return None;
            }
            None if policy == LocalSchemaDialectPolicy::RequireExactDraft202012 => {
                self.push(
                    FindingBucket::Schema,
                    Finding::unsupported_schema_dialect(
                        SupportedRevision::CURRENT,
                        base.clone().field(LocationField::Schema),
                        json_kind(None),
                    ),
                );
                return None;
            }
            Some(_) | None => {}
        }

        let mut stack = vec![(schema, 0_u64, base.clone())];
        let mut nodes = 0_u64;
        let mut work = 0_u64;
        let mut references = Vec::new();
        let mut external_reference = false;
        let mut unsupported_dialect = false;
        let mut unsupported_vocabulary = false;
        while let Some((value, depth, location)) = stack.pop() {
            nodes = nodes.saturating_add(1);
            work = work.saturating_add(1);
            if nodes > values.schema_nodes {
                self.schema_limit(location, LimitKind::SchemaNodes, nodes, values.schema_nodes);
                return None;
            }
            if work > values.schema_evaluation_steps {
                self.schema_limit(
                    location,
                    LimitKind::SchemaEvaluationSteps,
                    work,
                    values.schema_evaluation_steps,
                );
                return None;
            }
            if depth > values.schema_depth {
                self.schema_limit(location, LimitKind::SchemaDepth, depth, values.schema_depth);
                return None;
            }

            match value {
                Value::Array(values) => {
                    for (index, value) in values.iter().enumerate().rev() {
                        stack.push((
                            value,
                            depth.saturating_add(1),
                            location.clone().index(index),
                        ));
                    }
                }
                Value::Object(values) => {
                    for (key, value) in values.iter().rev() {
                        let child_location = if policy
                            == LocalSchemaDialectPolicy::RequireExactDraft202012
                            && key == "$vocabulary"
                        {
                            location.clone().field(LocationField::Vocabulary)
                        } else {
                            schema_child_location(location.clone(), key)
                        };
                        if policy == LocalSchemaDialectPolicy::RequireExactDraft202012
                            && key == "$schema"
                            && value.as_str() != Some(DRAFT_2020_12)
                        {
                            unsupported_dialect = true;
                            self.push(
                                FindingBucket::Schema,
                                Finding::unsupported_schema_dialect(
                                    SupportedRevision::CURRENT,
                                    child_location.clone(),
                                    json_kind(Some(value)),
                                ),
                            );
                        }
                        if policy == LocalSchemaDialectPolicy::RequireExactDraft202012
                            && key == "$vocabulary"
                            && has_unsupported_vocabulary(value)
                        {
                            unsupported_vocabulary = true;
                            self.push(
                                FindingBucket::Schema,
                                Finding::schema_contract_invalid(
                                    SupportedRevision::CURRENT,
                                    child_location.clone(),
                                    RuleViolation::UnsupportedSchemaVocabulary,
                                ),
                            );
                        }
                        if matches!(key.as_str(), "$ref" | "$dynamicRef")
                            && let Some(reference) = value.as_str()
                        {
                            if is_local_reference(reference) {
                                references.push((reference.to_owned(), child_location.clone()));
                            } else {
                                external_reference = true;
                                self.push(
                                    FindingBucket::Schema,
                                    Finding::external_schema_reference_blocked(
                                        SupportedRevision::CURRENT,
                                        child_location.clone(),
                                    ),
                                );
                            }
                        }
                        stack.push((value, depth.saturating_add(1), child_location));
                    }
                }
                _ => {}
            }
        }

        let mut unresolved_reference = false;
        for (reference, location) in &references {
            work = work.saturating_add(1);
            if work > values.schema_evaluation_steps {
                self.schema_limit(
                    location.clone(),
                    LimitKind::SchemaEvaluationSteps,
                    work,
                    values.schema_evaluation_steps,
                );
                return None;
            }
            if resolve_local_reference(schema, reference).is_none() {
                unresolved_reference = true;
                self.push(
                    FindingBucket::Schema,
                    Finding::schema_contract_invalid(
                        SupportedRevision::CURRENT,
                        location.clone(),
                        RuleViolation::UnresolvedLocalReference,
                    ),
                );
            }
        }
        if let Some((observed, location)) = reference_depth_violation(
            schema,
            &references,
            values.schema_ref_depth,
            &mut work,
            values.schema_evaluation_steps,
        ) {
            let kind = if work > values.schema_evaluation_steps {
                LimitKind::SchemaEvaluationSteps
            } else {
                LimitKind::SchemaRefDepth
            };
            let maximum = if kind == LimitKind::SchemaRefDepth {
                values.schema_ref_depth
            } else {
                values.schema_evaluation_steps
            };
            self.schema_limit(location, kind, observed, maximum);
            return None;
        }

        if external_reference
            || unresolved_reference
            || unsupported_dialect
            || unsupported_vocabulary
        {
            return None;
        }

        let budget = SchemaWorkBudget::with_observed(values.schema_evaluation_steps, work);
        match validate_meta_schema(schema, budget.clone(), values.validation_errors) {
            Ok(()) => {}
            Err(SchemaWorkIssue::Limit(limit)) => {
                self.schema_validation_incomplete(
                    base,
                    SchemaValidationPhase::MetaValidation,
                    limit,
                );
                return None;
            }
            Err(SchemaWorkIssue::Invalid {
                location,
                error_count,
            }) => {
                self.push(
                    FindingBucket::Schema,
                    Finding::schema_contract_invalid(
                        SupportedRevision::CURRENT,
                        append_schema_error_location(base.clone(), &location),
                        RuleViolation::InvalidDraft202012 { error_count },
                    ),
                );
                if error_count > values.validation_errors {
                    self.schema_limit(
                        base,
                        LimitKind::ValidationErrors,
                        error_count,
                        values.validation_errors,
                    );
                }
                return None;
            }
            Err(SchemaWorkIssue::UnsupportedPattern { location }) => {
                self.push(
                    FindingBucket::Schema,
                    Finding::schema_contract_invalid(
                        SupportedRevision::CURRENT,
                        append_schema_error_location(base, &location),
                        RuleViolation::UnsupportedLinearPattern,
                    ),
                );
                return None;
            }
        }

        match BudgetedValidator::compile_with_budget(schema, Arc::clone(&budget)) {
            Ok(_) => Some(budget),
            Err(issue) => {
                match issue {
                    SchemaWorkIssue::Limit(limit) => {
                        self.schema_validation_incomplete(
                            base,
                            SchemaValidationPhase::CompileConstruction,
                            limit,
                        );
                    }
                    SchemaWorkIssue::Invalid { location, .. } => {
                        self.push(
                            FindingBucket::Schema,
                            Finding::schema_contract_invalid(
                                SupportedRevision::CURRENT,
                                append_schema_error_location(base, &location),
                                RuleViolation::InvalidDraft202012 { error_count: 1 },
                            ),
                        );
                    }
                    SchemaWorkIssue::UnsupportedPattern { location } => {
                        self.push(
                            FindingBucket::Schema,
                            Finding::schema_contract_invalid(
                                SupportedRevision::CURRENT,
                                append_schema_error_location(base, &location),
                                RuleViolation::UnsupportedLinearPattern,
                            ),
                        );
                    }
                }
                None
            }
        }
    }

    fn schema_limit(&mut self, location: Location, kind: LimitKind, observed: u64, maximum: u64) {
        let violation = LimitViolation::new(kind, observed, maximum)
            .expect("schema limit evidence exceeds its maximum");
        self.push(
            FindingBucket::Schema,
            Finding::limit_exceeded(SupportedRevision::CURRENT, location, violation),
        );
    }

    fn schema_validation_incomplete(
        &mut self,
        location: Location,
        phase: SchemaValidationPhase,
        violation: LimitViolation,
    ) {
        self.push(
            FindingBucket::Schema,
            Finding::schema_validation_incomplete(
                SupportedRevision::CURRENT,
                location,
                phase,
                violation,
            ),
        );
    }

    fn into_checks(mut self) -> Vec<CheckResult> {
        self.finish_overflow();
        self.catalog.append(&mut self.quality);
        self.schema.append(&mut self.security);
        let downstream_skip_reason = if self
            .envelope
            .iter()
            .any(|finding| finding.severity().is_failure())
        {
            SkipReason::PrerequisiteFailed
        } else {
            self.revision_block
                .unwrap_or(SkipReason::PrerequisiteFailed)
        };
        let revision_check = if self.revision_checked {
            CheckResult::performed(
                CheckId::ProtocolRevision,
                Requirement::Required,
                self.revision,
            )
        } else {
            CheckResult::skipped(
                CheckId::ProtocolRevision,
                Requirement::Required,
                SkipReason::PrerequisiteFailed,
            )
        };
        let envelope_check = CheckResult::performed(
            CheckId::ProtocolEnvelope,
            Requirement::Required,
            self.envelope,
        );
        let catalog_check = if self.discovery_valid && self.revision_supported {
            CheckResult::performed(
                CheckId::DiscoveryCatalogs,
                Requirement::Required,
                self.catalog,
            )
        } else {
            CheckResult::skipped(
                CheckId::DiscoveryCatalogs,
                Requirement::Required,
                downstream_skip_reason,
            )
        };
        let schema_check = if !self.discovery_valid || !self.revision_supported {
            CheckResult::skipped(
                CheckId::SchemaContracts,
                Requirement::Required,
                downstream_skip_reason,
            )
        } else if self.catalog_limit_reached {
            CheckResult::skipped(
                CheckId::SchemaContracts,
                Requirement::Required,
                SkipReason::LimitReached,
            )
        } else if self.tools_advertised && !self.tools_catalog_valid {
            CheckResult::skipped(
                CheckId::SchemaContracts,
                Requirement::Required,
                SkipReason::PrerequisiteFailed,
            )
        } else {
            CheckResult::performed(CheckId::SchemaContracts, Requirement::Required, self.schema)
        };

        vec![
            revision_check,
            envelope_check,
            catalog_check,
            schema_check,
            CheckResult::skipped(
                CheckId::RuntimeTools,
                Requirement::Optional,
                SkipReason::NotAuthorized,
            ),
        ]
    }
}

pub(super) fn validate_cacheable_result(
    result: &Map<String, Value>,
    base: Location,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let result_type = result.get("resultType");
    if result_type.and_then(Value::as_str) != Some("complete") {
        findings.push(Finding::catalog_contract_invalid(
            SupportedRevision::CURRENT,
            base.clone().field(LocationField::ResultType),
            RuleViolation::ExpectedCompleteResult {
                observed: json_kind(result_type),
            },
        ));
    }

    let ttl = result.get("ttlMs");
    if !ttl.is_some_and(is_non_negative_number) {
        findings.push(Finding::catalog_contract_invalid(
            SupportedRevision::CURRENT,
            base.clone().field(LocationField::TtlMs),
            RuleViolation::ExpectedShape {
                expected: ExpectedShape::NonNegativeNumber,
                observed: json_kind(ttl),
            },
        ));
    }

    let cache_scope = result.get("cacheScope");
    if !matches!(
        cache_scope.and_then(Value::as_str),
        Some("public" | "private")
    ) {
        findings.push(Finding::catalog_contract_invalid(
            SupportedRevision::CURRENT,
            base.field(LocationField::CacheScope),
            RuleViolation::ExpectedCacheScope {
                observed: json_kind(cache_scope),
            },
        ));
    }
    findings
}

pub(super) fn validate_discovery_capabilities(
    result: &Map<String, Value>,
    base: Location,
) -> (Vec<Finding>, bool) {
    let location = base.field(LocationField::Capabilities);
    let Some(capabilities) = result.get("capabilities").and_then(Value::as_object) else {
        return (
            vec![Finding::catalog_contract_invalid(
                SupportedRevision::CURRENT,
                location,
                RuleViolation::ExpectedShape {
                    expected: ExpectedShape::Object,
                    observed: json_kind(result.get("capabilities")),
                },
            )],
            false,
        );
    };

    let mut findings = Vec::new();
    for kind in [
        CatalogKind::Tools,
        CatalogKind::Prompts,
        CatalogKind::Resources,
    ] {
        let Some(value) = capabilities.get(kind.capability()) else {
            continue;
        };
        let Some(capability) = value.as_object() else {
            findings.push(Finding::catalog_contract_invalid(
                SupportedRevision::CURRENT,
                location.clone().field(kind.location_field()),
                RuleViolation::ExpectedShape {
                    expected: ExpectedShape::Object,
                    observed: json_kind(Some(value)),
                },
            ));
            continue;
        };
        for (field_name, field) in match kind {
            CatalogKind::Resources => [
                ("listChanged", LocationField::ListChanged),
                ("subscribe", LocationField::Subscribe),
            ]
            .as_slice(),
            CatalogKind::Tools | CatalogKind::Prompts => {
                [("listChanged", LocationField::ListChanged)].as_slice()
            }
            CatalogKind::ResourceTemplates => unreachable!("not a discovery capability"),
        } {
            if let Some(setting) = capability.get(*field_name)
                && !setting.is_boolean()
            {
                findings.push(Finding::catalog_contract_invalid(
                    SupportedRevision::CURRENT,
                    location.clone().field(kind.location_field()).field(*field),
                    RuleViolation::ExpectedShape {
                        expected: ExpectedShape::Boolean,
                        observed: json_kind(Some(setting)),
                    },
                ));
            }
        }
    }
    (
        findings,
        capabilities.get("tools").is_some_and(Value::is_object),
    )
}

pub(super) fn validate_legacy_capabilities(
    result: &Map<String, Value>,
    base: Location,
    revision: SupportedRevision,
) -> (Vec<Finding>, bool) {
    let (findings, tools_advertised) = validate_discovery_capabilities(result, base.clone());
    let mut findings = findings
        .into_iter()
        .map(|finding| finding.with_revision(revision))
        .collect();
    let Some(capabilities) = result.get("capabilities").and_then(Value::as_object) else {
        return (findings, false);
    };
    let location = base.field(LocationField::Capabilities);
    for (name, field) in [
        ("logging", LocationField::Logging),
        ("completions", LocationField::Completions),
    ] {
        optional_capability_object(
            &mut findings,
            revision,
            capabilities.get(name),
            location.clone().field(field),
        );
    }
    if let Some(experimental) = optional_capability_object(
        &mut findings,
        revision,
        capabilities.get("experimental"),
        location.clone().field(LocationField::Experimental),
    ) && let Some(invalid) = experimental.values().find(|value| !value.is_object())
    {
        findings.push(Finding::catalog_contract_invalid(
            revision,
            location
                .clone()
                .field(LocationField::Experimental)
                .wildcard(),
            RuleViolation::ExpectedShape {
                expected: ExpectedShape::Object,
                observed: json_kind(Some(invalid)),
            },
        ));
    }
    if revision == SupportedRevision::V2025_11_25
        && let Some(tasks) = optional_capability_object(
            &mut findings,
            revision,
            capabilities.get("tasks"),
            location.clone().field(LocationField::Tasks),
        )
    {
        for (name, field) in [
            ("list", LocationField::List),
            ("cancel", LocationField::Cancel),
        ] {
            optional_capability_object(
                &mut findings,
                revision,
                tasks.get(name),
                location.clone().field(LocationField::Tasks).field(field),
            );
        }
        if let Some(requests) = optional_capability_object(
            &mut findings,
            revision,
            tasks.get("requests"),
            location
                .clone()
                .field(LocationField::Tasks)
                .field(LocationField::Requests),
        ) && let Some(tools) = optional_capability_object(
            &mut findings,
            revision,
            requests.get("tools"),
            location
                .clone()
                .field(LocationField::Tasks)
                .field(LocationField::Requests)
                .field(LocationField::Tools),
        ) {
            optional_capability_object(
                &mut findings,
                revision,
                tools.get("call"),
                location
                    .field(LocationField::Tasks)
                    .field(LocationField::Requests)
                    .field(LocationField::Tools)
                    .field(LocationField::Call),
            );
        }
    }
    (findings, tools_advertised)
}

fn optional_capability_object<'a>(
    findings: &mut Vec<Finding>,
    revision: SupportedRevision,
    value: Option<&'a Value>,
    location: Location,
) -> Option<&'a Map<String, Value>> {
    let value = value?;
    if let Some(object) = value.as_object() {
        Some(object)
    } else {
        findings.push(Finding::catalog_contract_invalid(
            revision,
            location,
            RuleViolation::ExpectedShape {
                expected: ExpectedShape::Object,
                observed: json_kind(Some(value)),
            },
        ));
        None
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum LocalSchemaDialectPolicy {
    RevisionDefaultDraft202012,
    RequireExactDraft202012,
}

pub(super) fn validate_local_schema(schema: &Value, base: Location) -> Vec<Finding> {
    validate_local_schema_with_policy(
        schema,
        base,
        LocalSchemaDialectPolicy::RevisionDefaultDraft202012,
    )
}

pub(super) fn validate_local_schema_with_policy(
    schema: &Value,
    base: Location,
    policy: LocalSchemaDialectPolicy,
) -> Vec<Finding> {
    validate_local_schema_with_budget(schema, base, policy).0
}

fn validate_local_schema_with_budget(
    schema: &Value,
    base: Location,
    policy: LocalSchemaDialectPolicy,
) -> (Vec<Finding>, Option<Arc<SchemaWorkBudget>>) {
    let mut analyzer = Analyzer::new(0, false, false);
    let mut budget = analyzer.analyze_schema_with_policy(schema, base.clone(), policy);
    if analyzer.finding_overflow {
        budget = None;
        analyzer.schema.pop();
        let maximum = analyzer.limits.values().report_findings;
        analyzer.schema.push(Finding::limit_exceeded(
            SupportedRevision::CURRENT,
            base,
            LimitViolation::new(
                LimitKind::ReportFindings,
                maximum.saturating_add(1),
                maximum,
            )
            .expect("the local schema finding overflow exceeds the report maximum"),
        ));
    }
    (analyzer.schema, budget)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum InstanceValidationIssue {
    Mismatch { error_count: u64 },
    Limit(LimitViolation),
    InvalidSchema,
}

pub(super) struct LocalValidator {
    validator: BudgetedValidator,
}

impl LocalValidator {
    pub(super) fn compile(schema: &Value) -> Result<Self, InstanceValidationIssue> {
        let maximum = DiagnosticLimits::DEFAULTS.values().schema_evaluation_steps;
        let validator =
            BudgetedValidator::compile(schema, maximum).map_err(|issue| match issue {
                SchemaWorkIssue::Limit(limit) => InstanceValidationIssue::Limit(limit),
                SchemaWorkIssue::Invalid { .. } | SchemaWorkIssue::UnsupportedPattern { .. } => {
                    InstanceValidationIssue::InvalidSchema
                }
            })?;
        Ok(Self { validator })
    }

    pub(super) fn validate(&self, instance: &Value) -> Result<(), InstanceValidationIssue> {
        let values = DiagnosticLimits::DEFAULTS.values();
        let bytes = u64::try_from(serialized_len(instance)).unwrap_or(u64::MAX);
        if bytes > values.instance_bytes {
            return Err(InstanceValidationIssue::Limit(
                LimitViolation::new(LimitKind::InstanceBytes, bytes, values.instance_bytes)
                    .expect("the instance byte count exceeds its checked maximum"),
            ));
        }

        let mut stack = vec![(instance, 0_u64)];
        let mut work = 0_u64;
        let mut text_work = 0_u64;
        while let Some((value, depth)) = stack.pop() {
            work = work.saturating_add(1);
            if work > values.schema_evaluation_steps {
                return Err(InstanceValidationIssue::Limit(
                    LimitViolation::new(
                        LimitKind::SchemaEvaluationSteps,
                        work,
                        values.schema_evaluation_steps,
                    )
                    .expect("the instance traversal work exceeds its checked maximum"),
                ));
            }
            if depth > values.schema_depth {
                return Err(InstanceValidationIssue::Limit(
                    LimitViolation::new(LimitKind::SchemaDepth, depth, values.schema_depth)
                        .expect("the instance depth exceeds its checked maximum"),
                ));
            }
            match value {
                Value::String(value) => {
                    text_work = text_work.saturating_add(
                        u64::try_from(value.len())
                            .unwrap_or(u64::MAX)
                            .saturating_add(1),
                    );
                }
                Value::Array(values) => {
                    stack.extend(
                        values
                            .iter()
                            .rev()
                            .map(|value| (value, depth.saturating_add(1))),
                    );
                }
                Value::Object(values) => {
                    for key in values.keys() {
                        text_work = text_work.saturating_add(
                            u64::try_from(key.len())
                                .unwrap_or(u64::MAX)
                                .saturating_add(1),
                        );
                    }
                    stack.extend(
                        values
                            .values()
                            .rev()
                            .map(|value| (value, depth.saturating_add(1))),
                    );
                }
                _ => {}
            }
        }

        let error_count = self
            .validator
            .error_count(
                instance,
                values.schema_evaluation_steps,
                work,
                text_work,
                values.validation_errors,
            )
            .map_err(InstanceValidationIssue::Limit)?;
        if error_count > values.validation_errors {
            return Err(InstanceValidationIssue::Limit(
                LimitViolation::new(
                    LimitKind::ValidationErrors,
                    error_count,
                    values.validation_errors,
                )
                .expect("the validation error count exceeds its checked maximum"),
            ));
        }
        if error_count > 0 {
            return Err(InstanceValidationIssue::Mismatch { error_count });
        }
        Ok(())
    }
}

pub(super) fn diagnose(
    conversation: &PassiveCatalogConversation,
    responses: &[ProbeResponse],
    reserved_findings: usize,
) -> Vec<CheckResult> {
    if conversation.revision.uses_initialize() {
        return diagnose_legacy(conversation, responses, reserved_findings);
    }
    let mut analyzer = Analyzer::new(
        reserved_findings,
        conversation.validate_http_headers,
        conversation.auto_discovery,
    );
    let Some(discovery) = responses.first() else {
        return analyzer.into_checks();
    };
    analyzer.analyze_discovery(discovery);

    for (record, response) in conversation
        .records
        .iter()
        .skip(1)
        .zip(responses.iter().skip(1))
    {
        debug_assert_eq!(record.id, response.request_id());
        analyzer.analyze_catalog_page(record, response);
    }
    analyzer.into_checks()
}

struct LegacyAnalyzer {
    revision_kind: SupportedRevision,
    limits: DiagnosticLimits,
    revision: Vec<Finding>,
    envelope: Vec<Finding>,
    catalog: Vec<Finding>,
    quality: Vec<Finding>,
    schema: Vec<Finding>,
    security: Vec<Finding>,
    capacity: usize,
    stored: usize,
    overflow: bool,
    initialize_valid: bool,
    revision_checked: bool,
    revision_supported: bool,
    revision_block: Option<SkipReason>,
    tools_advertised: bool,
    tools_catalog_valid: bool,
    catalog_limit_reached: bool,
    observed_items: u64,
    item_offsets: BTreeMap<CatalogKind, usize>,
    identifiers: BTreeMap<CatalogKind, BTreeMap<String, usize>>,
    secondary_identifiers: BTreeMap<CatalogKind, BTreeMap<String, usize>>,
    cursors: BTreeMap<CatalogKind, BTreeSet<String>>,
}

impl LegacyAnalyzer {
    fn new(revision: SupportedRevision, reserved_findings: usize) -> Self {
        let limits = DiagnosticLimits::DEFAULTS;
        let maximum = usize::try_from(limits.values().report_findings).unwrap_or(usize::MAX);
        Self {
            revision_kind: revision,
            limits,
            revision: Vec::new(),
            envelope: Vec::new(),
            catalog: Vec::new(),
            quality: Vec::new(),
            schema: Vec::new(),
            security: Vec::new(),
            capacity: maximum.saturating_sub(reserved_findings),
            stored: 0,
            overflow: false,
            initialize_valid: false,
            revision_checked: false,
            revision_supported: false,
            revision_block: None,
            tools_advertised: false,
            tools_catalog_valid: true,
            catalog_limit_reached: false,
            observed_items: 0,
            item_offsets: BTreeMap::new(),
            identifiers: BTreeMap::new(),
            secondary_identifiers: BTreeMap::new(),
            cursors: BTreeMap::new(),
        }
    }

    fn push(&mut self, bucket: FindingBucket, finding: Finding) {
        let destination = match bucket {
            FindingBucket::Revision => &mut self.revision,
            FindingBucket::Envelope => &mut self.envelope,
            FindingBucket::Catalog => &mut self.catalog,
            FindingBucket::Quality => &mut self.quality,
            FindingBucket::Schema => &mut self.schema,
            FindingBucket::Security => &mut self.security,
        };
        if destination.contains(&finding) {
            return;
        }
        if self.stored < self.capacity {
            destination.push(finding);
            self.stored += 1;
        } else if matches!(bucket, FindingBucket::Security) {
            let displaced = self
                .quality
                .pop()
                .or_else(|| self.schema.pop())
                .or_else(|| self.catalog.pop());
            if displaced.is_some() {
                self.security.push(finding);
            }
            self.overflow = true;
        } else if !matches!(bucket, FindingBucket::Quality) && self.quality.pop().is_some() {
            let destination = match bucket {
                FindingBucket::Revision => &mut self.revision,
                FindingBucket::Envelope => &mut self.envelope,
                FindingBucket::Catalog => &mut self.catalog,
                FindingBucket::Quality => unreachable!("quality findings do not displace findings"),
                FindingBucket::Schema => &mut self.schema,
                FindingBucket::Security => unreachable!("security findings use priority storage"),
            };
            destination.push(finding);
            self.overflow = true;
        } else {
            self.overflow = true;
        }
    }

    fn expected_shape(
        &mut self,
        bucket: FindingBucket,
        location: Location,
        expected: ExpectedShape,
        observed: Option<&Value>,
    ) {
        let violation = RuleViolation::ExpectedShape {
            expected,
            observed: json_kind(observed),
        };
        let finding = if matches!(bucket, FindingBucket::Schema) {
            Finding::schema_contract_invalid(self.revision_kind, location, violation)
        } else {
            Finding::catalog_contract_invalid(self.revision_kind, location, violation)
        };
        self.push(bucket, finding);
    }

    fn analyze_initialize(&mut self, response: &ProbeResponse) {
        let value: Value = serde_json::from_slice(response.as_bytes())
            .expect("the transport accepted this JSON response");
        let object = value
            .as_object()
            .expect("the transport accepted a JSON-RPC object");
        let base = Location::root(LocationField::Server);
        if let Some(error) = classify_json_rpc_error(object) {
            self.revision_checked = true;
            self.revision_block = Some(SkipReason::PrerequisiteFailed);
            self.push(
                FindingBucket::Revision,
                Finding::lifecycle_method_rejected(
                    self.revision_kind,
                    Location::root(LocationField::Initialize).field(LocationField::Response),
                    error,
                ),
            );
            return;
        } else if object.contains_key("error") {
            self.push(
                FindingBucket::Envelope,
                Finding::catalog_contract_invalid(
                    self.revision_kind,
                    base,
                    RuleViolation::ServerErrorResponse,
                ),
            );
            return;
        }
        let Some(result) = object.get("result").and_then(Value::as_object) else {
            self.expected_shape(
                FindingBucket::Envelope,
                base.field(LocationField::Result),
                ExpectedShape::Object,
                object.get("result"),
            );
            return;
        };

        self.revision_checked = true;
        let revision_location = base
            .clone()
            .field(LocationField::Result)
            .field(LocationField::NegotiatedProtocolVersion);
        match result.get("protocolVersion").and_then(Value::as_str) {
            Some(revision) if revision == self.revision_kind.as_str() => {
                self.revision_supported = true;
                self.push(
                    FindingBucket::Revision,
                    Finding::revision_confirmed(self.revision_kind, revision_location),
                );
            }
            Some(_) => self.push(
                FindingBucket::Revision,
                Finding::revision_mismatch(self.revision_kind, revision_location),
            ),
            None => self.push(
                FindingBucket::Revision,
                Finding::invalid_revision_value(
                    self.revision_kind,
                    revision_location,
                    super::redaction::RedactedValue::new(
                        result.get("protocolVersion").map_or(0, serialized_len),
                    ),
                ),
            ),
        }

        let (capability_findings, tools_advertised) = validate_legacy_capabilities(
            result,
            base.clone().field(LocationField::Result),
            self.revision_kind,
        );
        self.tools_advertised = tools_advertised;
        for finding in capability_findings {
            self.push(FindingBucket::Envelope, finding);
        }

        let server_info_location = base
            .clone()
            .field(LocationField::Result)
            .field(LocationField::ServerInfo);
        let Some(server_info) = result.get("serverInfo").and_then(Value::as_object) else {
            self.expected_shape(
                FindingBucket::Envelope,
                server_info_location,
                ExpectedShape::Object,
                result.get("serverInfo"),
            );
            return;
        };
        for (name, field) in [
            ("name", LocationField::Name),
            ("version", LocationField::Version),
        ] {
            if !server_info.get(name).is_some_and(Value::is_string) {
                self.expected_shape(
                    FindingBucket::Envelope,
                    server_info_location.clone().field(field),
                    ExpectedShape::String,
                    server_info.get(name),
                );
            }
        }
        if let Some(instructions) = result.get("instructions")
            && !instructions.is_string()
        {
            self.expected_shape(
                FindingBucket::Envelope,
                base.field(LocationField::Result)
                    .field(LocationField::Instructions),
                ExpectedShape::String,
                Some(instructions),
            );
        }
        self.initialize_valid = self.revision_supported
            && !self
                .envelope
                .iter()
                .any(|finding| finding.severity().is_failure());
    }

    fn analyze_catalog_page(&mut self, record: &RequestRecord, response: &ProbeResponse) {
        let RequestKind::Catalog(kind) = record.kind else {
            return;
        };
        let value: Value = serde_json::from_slice(response.as_bytes())
            .expect("the transport accepted this JSON response");
        let object = value
            .as_object()
            .expect("the transport accepted a JSON-RPC object");
        let base = kind.location();
        if let Some(error) = classify_json_rpc_error(object) {
            self.push(
                FindingBucket::Catalog,
                Finding::catalog_method_rejected(
                    self.revision_kind,
                    kind.response_location(),
                    error,
                ),
            );
            if kind == CatalogKind::Tools {
                self.tools_catalog_valid = false;
            }
            return;
        } else if object.contains_key("error") {
            self.push(
                FindingBucket::Catalog,
                Finding::catalog_contract_invalid(
                    self.revision_kind,
                    base,
                    RuleViolation::ServerErrorResponse,
                ),
            );
            if kind == CatalogKind::Tools {
                self.tools_catalog_valid = false;
            }
            return;
        }
        let Some(result) = object.get("result").and_then(Value::as_object) else {
            self.expected_shape(
                FindingBucket::Catalog,
                base.clone().field(LocationField::Result),
                ExpectedShape::Object,
                object.get("result"),
            );
            if kind == CatalogKind::Tools {
                self.tools_catalog_valid = false;
            }
            return;
        };
        if let Some(cursor) = result.get("nextCursor") {
            if let Some(cursor) = cursor.as_str() {
                if !self
                    .cursors
                    .entry(kind)
                    .or_default()
                    .insert(cursor.to_owned())
                {
                    self.push(
                        FindingBucket::Catalog,
                        Finding::pagination_cursor_repeated(
                            self.revision_kind,
                            base.clone().field(LocationField::NextCursor),
                        ),
                    );
                }
            } else {
                self.expected_shape(
                    FindingBucket::Catalog,
                    base.clone().field(LocationField::NextCursor),
                    ExpectedShape::String,
                    Some(cursor),
                );
            }
        }
        let Some(items) = result.get(kind.result_field()).and_then(Value::as_array) else {
            self.expected_shape(
                FindingBucket::Catalog,
                base,
                ExpectedShape::Array,
                result.get(kind.result_field()),
            );
            if kind == CatalogKind::Tools {
                self.tools_catalog_valid = false;
            }
            return;
        };
        let offset = *self.item_offsets.get(&kind).unwrap_or(&0);
        let page_items = u64::try_from(items.len()).unwrap_or(u64::MAX);
        let previous = self.observed_items;
        self.observed_items = self.observed_items.saturating_add(page_items);
        let maximum = self.limits.values().catalog_items;
        if self.observed_items > maximum {
            self.catalog_limit_reached = true;
            self.push(
                FindingBucket::Catalog,
                Finding::limit_exceeded(
                    self.revision_kind,
                    kind.location(),
                    LimitViolation::new(LimitKind::CatalogItems, self.observed_items, maximum)
                        .expect("the observed catalog count exceeds its maximum"),
                ),
            );
        }
        let remaining = usize::try_from(maximum.saturating_sub(previous)).unwrap_or(usize::MAX);
        for (page_index, item) in items.iter().take(remaining).enumerate() {
            self.analyze_item(kind, offset.saturating_add(page_index), item);
        }
        self.item_offsets
            .insert(kind, offset.saturating_add(items.len()));
    }

    fn analyze_item(&mut self, kind: CatalogKind, index: usize, item: &Value) {
        let base = kind.location().index(index);
        let Some(object) = item.as_object() else {
            self.expected_shape(
                FindingBucket::Catalog,
                base,
                ExpectedShape::Object,
                Some(item),
            );
            if kind == CatalogKind::Tools {
                self.tools_catalog_valid = false;
            }
            return;
        };
        match object.get("name").and_then(Value::as_str) {
            Some(name) => {
                if self
                    .identifiers
                    .entry(kind)
                    .or_default()
                    .insert(name.to_owned(), index)
                    .is_some()
                {
                    self.push(
                        FindingBucket::Catalog,
                        Finding::duplicate_catalog_identifier(
                            self.revision_kind,
                            base.clone().field(LocationField::Name),
                        ),
                    );
                }
            }
            None => self.expected_shape(
                FindingBucket::Catalog,
                base.clone().field(LocationField::Name),
                ExpectedShape::String,
                object.get("name"),
            ),
        }
        match kind {
            CatalogKind::Tools => self.analyze_tool(base, object),
            CatalogKind::Prompts => self.analyze_prompt(base, object),
            CatalogKind::Resources => {
                self.analyze_secondary_identifier(kind, base, object, "uri", LocationField::Uri)
            }
            CatalogKind::ResourceTemplates => self.analyze_secondary_identifier(
                kind,
                base,
                object,
                "uriTemplate",
                LocationField::UriTemplate,
            ),
        }
    }

    fn analyze_tool(&mut self, base: Location, tool: &Map<String, Value>) {
        match diagnose_tool_description(self.revision_kind, base.clone(), tool) {
            ToolDescriptionDiagnosis::Usable => {}
            ToolDescriptionDiagnosis::MissingOrBlank(finding)
            | ToolDescriptionDiagnosis::PlaceholderOrNameOnly(finding) => {
                self.push(FindingBucket::Quality, finding);
            }
            ToolDescriptionDiagnosis::Invalid(finding) => {
                self.push(FindingBucket::Catalog, finding);
                self.tools_catalog_valid = false;
            }
        }
        let Some(input_schema) = tool.get("inputSchema") else {
            self.expected_shape(
                FindingBucket::Schema,
                base.field(LocationField::InputSchema),
                ExpectedShape::Object,
                None,
            );
            self.tools_catalog_valid = false;
            return;
        };
        let input_location = base.clone().field(LocationField::InputSchema);
        let Some(input_object) = input_schema.as_object() else {
            self.expected_shape(
                FindingBucket::Schema,
                input_location,
                ExpectedShape::Object,
                Some(input_schema),
            );
            self.tools_catalog_valid = false;
            return;
        };
        let input_root_valid = input_object.get("type").and_then(Value::as_str) == Some("object");
        if !input_root_valid {
            self.push(
                FindingBucket::Schema,
                Finding::schema_contract_invalid(
                    self.revision_kind,
                    input_location.clone().field(LocationField::Type),
                    RuleViolation::ExpectedToolSchemaRootObject {
                        observed: json_kind(input_object.get("type")),
                    },
                ),
            );
        }
        let schema_budget = self.analyze_legacy_schema(input_schema, input_location.clone());
        if input_root_valid && let Some(budget) = schema_budget {
            let scan =
                scan_credential_literals(self.revision_kind, input_schema, input_location, &budget);
            for finding in scan.findings {
                self.push(FindingBucket::Security, finding);
            }
            if let Some((location, violation)) = scan.limit {
                self.push(
                    FindingBucket::Schema,
                    Finding::limit_exceeded(self.revision_kind, location, violation),
                );
            }
        }
        if let Some(output_schema) = tool.get("outputSchema") {
            let output_location = base.field(LocationField::OutputSchema);
            let Some(output_object) = output_schema.as_object() else {
                self.expected_shape(
                    FindingBucket::Schema,
                    output_location,
                    ExpectedShape::Object,
                    Some(output_schema),
                );
                return;
            };
            if output_object.get("type").and_then(Value::as_str) != Some("object") {
                self.push(
                    FindingBucket::Schema,
                    Finding::schema_contract_invalid(
                        self.revision_kind,
                        output_location.clone().field(LocationField::Type),
                        RuleViolation::ExpectedToolSchemaRootObject {
                            observed: json_kind(output_object.get("type")),
                        },
                    ),
                );
            }
            let _ = self.analyze_legacy_schema(output_schema, output_location);
        }
    }

    fn analyze_legacy_schema(
        &mut self,
        schema: &Value,
        location: Location,
    ) -> Option<Arc<SchemaWorkBudget>> {
        if self.revision_kind == SupportedRevision::V2025_06_18
            && schema
                .as_object()
                .is_some_and(|object| !object.contains_key("$schema"))
        {
            self.push(
                FindingBucket::Schema,
                Finding::ambiguous_schema_dialect(
                    self.revision_kind,
                    location.clone().field(LocationField::Schema),
                ),
            );
            self.analyze_legacy_schema_structure(schema, location);
            return None;
        }
        let (findings, budget) = validate_local_schema_with_budget(
            schema,
            location,
            LocalSchemaDialectPolicy::RevisionDefaultDraft202012,
        );
        let valid = findings.is_empty();
        for finding in findings {
            self.push(
                FindingBucket::Schema,
                finding.with_revision(self.revision_kind),
            );
        }
        valid.then_some(budget).flatten()
    }

    fn analyze_legacy_schema_structure(&mut self, schema: &Value, base: Location) {
        let values = self.limits.values();
        let bytes = u64::try_from(serialized_len(schema)).unwrap_or(u64::MAX);
        if bytes > values.schema_bytes {
            self.push(
                FindingBucket::Schema,
                Finding::limit_exceeded(
                    self.revision_kind,
                    base,
                    LimitViolation::new(LimitKind::SchemaBytes, bytes, values.schema_bytes)
                        .expect("the legacy schema byte count exceeds its maximum"),
                ),
            );
            return;
        }
        let object = schema
            .as_object()
            .expect("the legacy tool schema object shape was checked");
        if object.get("type").and_then(Value::as_str) != Some("object") {
            self.push(
                FindingBucket::Schema,
                Finding::schema_contract_invalid(
                    self.revision_kind,
                    base.clone().field(LocationField::Type),
                    RuleViolation::ExpectedToolSchemaRootObject {
                        observed: json_kind(object.get("type")),
                    },
                ),
            );
        }
        if let Some(properties) = object.get("properties") {
            if let Some(properties) = properties.as_object() {
                if let Some(invalid) = properties.values().find(|value| !value.is_object()) {
                    self.expected_shape(
                        FindingBucket::Schema,
                        base.clone().field(LocationField::Properties).wildcard(),
                        ExpectedShape::Object,
                        Some(invalid),
                    );
                }
            } else {
                self.expected_shape(
                    FindingBucket::Schema,
                    base.clone().field(LocationField::Properties),
                    ExpectedShape::Object,
                    Some(properties),
                );
            }
        }
        if let Some(required) = object.get("required") {
            if let Some(required) = required.as_array() {
                for (index, value) in required.iter().enumerate() {
                    if !value.is_string() {
                        self.expected_shape(
                            FindingBucket::Schema,
                            base.clone().field(LocationField::Required).index(index),
                            ExpectedShape::String,
                            Some(value),
                        );
                    }
                }
            } else {
                self.expected_shape(
                    FindingBucket::Schema,
                    base.clone().field(LocationField::Required),
                    ExpectedShape::Array,
                    Some(required),
                );
            }
        }

        let mut stack = vec![(schema, 0_u64, base)];
        let mut nodes = 0_u64;
        let mut references = Vec::new();
        while let Some((value, depth, location)) = stack.pop() {
            nodes = nodes.saturating_add(1);
            if nodes > values.schema_nodes {
                self.push(
                    FindingBucket::Schema,
                    Finding::limit_exceeded(
                        self.revision_kind,
                        location,
                        LimitViolation::new(LimitKind::SchemaNodes, nodes, values.schema_nodes)
                            .expect("the legacy schema node count exceeds its maximum"),
                    ),
                );
                return;
            }
            if nodes > values.schema_evaluation_steps {
                self.push(
                    FindingBucket::Schema,
                    Finding::limit_exceeded(
                        self.revision_kind,
                        location,
                        LimitViolation::new(
                            LimitKind::SchemaEvaluationSteps,
                            nodes,
                            values.schema_evaluation_steps,
                        )
                        .expect("the legacy schema traversal exceeds its maximum"),
                    ),
                );
                return;
            }
            if depth > values.schema_depth {
                self.push(
                    FindingBucket::Schema,
                    Finding::limit_exceeded(
                        self.revision_kind,
                        location,
                        LimitViolation::new(LimitKind::SchemaDepth, depth, values.schema_depth)
                            .expect("the legacy schema depth exceeds its maximum"),
                    ),
                );
                return;
            }
            match value {
                Value::Object(object) => {
                    for (key, child) in object {
                        let child_location = schema_child_location(location.clone(), key);
                        if matches!(key.as_str(), "$ref" | "$dynamicRef") {
                            match child.as_str() {
                                Some(reference) if !reference.starts_with('#') => self.push(
                                    FindingBucket::Schema,
                                    Finding::external_schema_reference_blocked(
                                        self.revision_kind,
                                        child_location.clone(),
                                    ),
                                ),
                                Some(reference) => {
                                    references.push((reference.to_owned(), child_location.clone()));
                                }
                                None => self.expected_shape(
                                    FindingBucket::Schema,
                                    child_location.clone(),
                                    ExpectedShape::String,
                                    Some(child),
                                ),
                            }
                        }
                        stack.push((child, depth.saturating_add(1), child_location));
                    }
                }
                Value::Array(items) => {
                    for (index, child) in items.iter().enumerate() {
                        stack.push((
                            child,
                            depth.saturating_add(1),
                            location.clone().index(index),
                        ));
                    }
                }
                _ => {}
            }
        }
        let mut work = nodes;
        for (reference, location) in &references {
            work = work.saturating_add(1);
            if work > values.schema_evaluation_steps {
                self.push(
                    FindingBucket::Schema,
                    Finding::limit_exceeded(
                        self.revision_kind,
                        location.clone(),
                        LimitViolation::new(
                            LimitKind::SchemaEvaluationSteps,
                            work,
                            values.schema_evaluation_steps,
                        )
                        .expect("the legacy schema reference work exceeds its maximum"),
                    ),
                );
                return;
            }
            if resolve_local_reference(schema, reference).is_none() {
                self.push(
                    FindingBucket::Schema,
                    Finding::schema_contract_invalid(
                        self.revision_kind,
                        location.clone(),
                        RuleViolation::UnresolvedLocalReference,
                    ),
                );
            }
        }
        if let Some((observed, location)) = reference_depth_violation(
            schema,
            &references,
            values.schema_ref_depth,
            &mut work,
            values.schema_evaluation_steps,
        ) {
            let (kind, maximum) = if work > values.schema_evaluation_steps {
                (
                    LimitKind::SchemaEvaluationSteps,
                    values.schema_evaluation_steps,
                )
            } else {
                (LimitKind::SchemaRefDepth, values.schema_ref_depth)
            };
            self.push(
                FindingBucket::Schema,
                Finding::limit_exceeded(
                    self.revision_kind,
                    location,
                    LimitViolation::new(kind, observed, maximum)
                        .expect("the legacy schema reference bound was exceeded"),
                ),
            );
        }
    }

    fn analyze_prompt(&mut self, base: Location, prompt: &Map<String, Value>) {
        let Some(arguments) = prompt.get("arguments") else {
            return;
        };
        let arguments_base = base.field(LocationField::Arguments);
        let Some(arguments) = arguments.as_array() else {
            self.expected_shape(
                FindingBucket::Catalog,
                arguments_base,
                ExpectedShape::Array,
                Some(arguments),
            );
            return;
        };
        let mut names = BTreeSet::new();
        for (index, argument) in arguments.iter().enumerate() {
            let location = arguments_base.clone().index(index);
            let Some(argument) = argument.as_object() else {
                self.expected_shape(
                    FindingBucket::Catalog,
                    location,
                    ExpectedShape::Object,
                    Some(argument),
                );
                continue;
            };
            match argument.get("name").and_then(Value::as_str) {
                Some(name) if !names.insert(name.to_owned()) => self.push(
                    FindingBucket::Catalog,
                    Finding::duplicate_catalog_identifier(
                        self.revision_kind,
                        location.clone().field(LocationField::Name),
                    ),
                ),
                Some(_) => {}
                None => self.expected_shape(
                    FindingBucket::Catalog,
                    location.clone().field(LocationField::Name),
                    ExpectedShape::String,
                    argument.get("name"),
                ),
            }
            if let Some(required) = argument.get("required")
                && !required.is_boolean()
            {
                self.expected_shape(
                    FindingBucket::Catalog,
                    location.field(LocationField::Required),
                    ExpectedShape::Boolean,
                    Some(required),
                );
            }
        }
    }

    fn analyze_secondary_identifier(
        &mut self,
        kind: CatalogKind,
        base: Location,
        object: &Map<String, Value>,
        name: &str,
        field: LocationField,
    ) {
        let location = base.field(field);
        let Some(value) = object.get(name).and_then(Value::as_str) else {
            self.expected_shape(
                FindingBucket::Catalog,
                location,
                ExpectedShape::String,
                object.get(name),
            );
            return;
        };
        if self
            .secondary_identifiers
            .entry(kind)
            .or_default()
            .insert(value.to_owned(), 0)
            .is_some()
        {
            self.push(
                FindingBucket::Catalog,
                Finding::duplicate_catalog_identifier(self.revision_kind, location),
            );
        }
    }

    fn finish(mut self) -> Vec<CheckResult> {
        if self.overflow && self.capacity > 0 {
            let removed = self
                .quality
                .pop()
                .or_else(|| self.schema.pop())
                .or_else(|| self.catalog.pop())
                .or_else(|| self.envelope.pop())
                .or_else(|| self.revision.pop())
                .or_else(|| self.security.pop());
            if removed.is_some() {
                self.catalog.push(Finding::limit_exceeded(
                    self.revision_kind,
                    Location::root(LocationField::Server),
                    LimitViolation::new(
                        LimitKind::ReportFindings,
                        self.limits.values().report_findings.saturating_add(1),
                        self.limits.values().report_findings,
                    )
                    .expect("the finding count exceeds its maximum"),
                ));
            }
        }
        self.catalog.append(&mut self.quality);
        self.schema.append(&mut self.security);
        let envelope_failed = self
            .envelope
            .iter()
            .any(|finding| finding.severity().is_failure());
        let revision_failed = self
            .revision
            .iter()
            .any(|finding| finding.severity().is_failure());
        let downstream = if revision_failed {
            self.revision_block
                .unwrap_or(SkipReason::UnsupportedRevision)
        } else {
            SkipReason::PrerequisiteFailed
        };
        let revision = if self.revision_checked {
            CheckResult::performed(
                CheckId::ProtocolRevision,
                Requirement::Required,
                self.revision,
            )
        } else {
            CheckResult::skipped(
                CheckId::ProtocolRevision,
                Requirement::Required,
                SkipReason::PrerequisiteFailed,
            )
        };
        let catalogs = if self.initialize_valid && !envelope_failed && !revision_failed {
            CheckResult::performed(
                CheckId::DiscoveryCatalogs,
                Requirement::Required,
                self.catalog,
            )
        } else {
            CheckResult::skipped(
                CheckId::DiscoveryCatalogs,
                Requirement::Required,
                downstream,
            )
        };
        let schemas = if !self.initialize_valid || envelope_failed || revision_failed {
            CheckResult::skipped(CheckId::SchemaContracts, Requirement::Required, downstream)
        } else if self.catalog_limit_reached {
            CheckResult::skipped(
                CheckId::SchemaContracts,
                Requirement::Required,
                SkipReason::LimitReached,
            )
        } else if self.tools_advertised && !self.tools_catalog_valid {
            CheckResult::skipped(
                CheckId::SchemaContracts,
                Requirement::Required,
                SkipReason::PrerequisiteFailed,
            )
        } else {
            CheckResult::performed(CheckId::SchemaContracts, Requirement::Required, self.schema)
        };
        vec![
            revision,
            CheckResult::performed(
                CheckId::ProtocolEnvelope,
                Requirement::Required,
                self.envelope,
            ),
            catalogs,
            schemas,
            CheckResult::skipped(
                CheckId::RuntimeTools,
                Requirement::Optional,
                SkipReason::NotAuthorized,
            ),
        ]
    }
}

fn diagnose_legacy(
    conversation: &PassiveCatalogConversation,
    responses: &[ProbeResponse],
    reserved_findings: usize,
) -> Vec<CheckResult> {
    let mut analyzer = LegacyAnalyzer::new(conversation.revision, reserved_findings);
    let Some(initialize) = responses.first() else {
        return analyzer.finish();
    };
    analyzer.analyze_initialize(initialize);
    for (record, response) in conversation
        .records
        .iter()
        .skip(1)
        .zip(responses.iter().skip(1))
    {
        debug_assert_eq!(record.id, response.request_id());
        analyzer.analyze_catalog_page(record, response);
    }
    analyzer.finish()
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

fn is_non_negative_number(value: &Value) -> bool {
    value.as_f64().is_some_and(|value| value >= 0.0)
}

fn has_unsupported_vocabulary(value: &Value) -> bool {
    value.as_object().is_some_and(|vocabularies| {
        vocabularies
            .keys()
            .any(|vocabulary| !DRAFT_2020_12_VOCABULARIES.contains(&vocabulary.as_str()))
    })
}

fn schema_child_location(location: Location, key: &str) -> Location {
    let field = match key {
        "$schema" => Some(LocationField::Schema),
        "type" => Some(LocationField::Type),
        "pattern" => Some(LocationField::Pattern),
        "properties" => Some(LocationField::Properties),
        "default" => Some(LocationField::Default),
        "const" => Some(LocationField::Const),
        "examples" => Some(LocationField::Examples),
        "enum" => Some(LocationField::Enum),
        "$defs" => Some(LocationField::Defs),
        "$ref" => Some(LocationField::Ref),
        "$dynamicRef" => Some(LocationField::DynamicRef),
        "items" => Some(LocationField::Items),
        "prefixItems" => Some(LocationField::PrefixItems),
        "allOf" => Some(LocationField::AllOf),
        "anyOf" => Some(LocationField::AnyOf),
        "oneOf" => Some(LocationField::OneOf),
        "not" => Some(LocationField::Not),
        "if" => Some(LocationField::If),
        "then" => Some(LocationField::Then),
        "else" => Some(LocationField::Else),
        "required" => Some(LocationField::Required),
        _ => None,
    };
    match field {
        Some(field) => location.field(field),
        None => location.wildcard(),
    }
}

fn append_schema_error_location(
    mut location: Location,
    path: &jsonschema::paths::Location,
) -> Location {
    let mut untrusted_name = false;
    for segment in path {
        match segment {
            SchemaLocationSegment::Index(index) => {
                location = location.index(index);
                untrusted_name = false;
            }
            SchemaLocationSegment::Property(_) if untrusted_name => {
                location = location.wildcard();
                untrusted_name = false;
            }
            SchemaLocationSegment::Property(property) => {
                untrusted_name = matches!(
                    property.as_ref(),
                    "properties"
                        | "patternProperties"
                        | "$defs"
                        | "definitions"
                        | "dependentSchemas"
                );
                location = schema_child_location(location, &property);
            }
        }
    }
    location
}

fn is_local_reference(reference: &str) -> bool {
    reference.is_empty() || reference.starts_with('#')
}

pub(super) fn resolve_local_reference<'a>(schema: &'a Value, reference: &str) -> Option<&'a Value> {
    if matches!(reference, "" | "#") {
        return Some(schema);
    }
    let fragment = reference.strip_prefix('#')?;
    if fragment.starts_with('/') {
        let decoded = percent_decode(fragment)?;
        return schema.pointer(&decoded);
    }

    let anchor = percent_decode(fragment)?;
    let mut stack = vec![schema];
    while let Some(value) = stack.pop() {
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
            _ => {}
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
            let high = *bytes.get(index + 1)?;
            let low = *bytes.get(index + 2)?;
            decoded.push(
                hex_value(high)?
                    .checked_mul(16)?
                    .checked_add(hex_value(low)?)?,
            );
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
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

fn reference_depth_violation(
    schema: &Value,
    references: &[(String, Location)],
    maximum_depth: u64,
    work: &mut u64,
    maximum_work: u64,
) -> Option<(u64, Location)> {
    for (reference, location) in references {
        let mut active = BTreeSet::new();
        if let Some(observed) = follow_reference_depth(
            schema,
            reference,
            1,
            maximum_depth,
            &mut active,
            work,
            maximum_work,
        ) {
            return Some((observed, location.clone()));
        }
    }
    None
}

fn follow_reference_depth(
    schema: &Value,
    reference: &str,
    depth: u64,
    maximum_depth: u64,
    active: &mut BTreeSet<String>,
    work: &mut u64,
    maximum_work: u64,
) -> Option<u64> {
    *work = work.saturating_add(1);
    if *work > maximum_work {
        return Some(*work);
    }
    if depth > maximum_depth {
        return Some(depth);
    }
    if !active.insert(reference.to_owned()) {
        return None;
    }
    let target = resolve_local_reference(schema, reference)?;
    let mut nested = Vec::new();
    collect_local_references(target, &mut nested, work, maximum_work);
    for nested_reference in nested {
        if let Some(observed) = follow_reference_depth(
            schema,
            &nested_reference,
            depth.saturating_add(1),
            maximum_depth,
            active,
            work,
            maximum_work,
        ) {
            return Some(observed);
        }
    }
    active.remove(reference);
    None
}

fn collect_local_references(
    value: &Value,
    references: &mut Vec<String>,
    work: &mut u64,
    maximum_work: u64,
) {
    let mut stack = vec![value];
    while let Some(value) = stack.pop() {
        *work = work.saturating_add(1);
        if *work > maximum_work {
            return;
        }
        match value {
            Value::Object(object) => {
                for (key, value) in object {
                    if matches!(key.as_str(), "$ref" | "$dynamicRef")
                        && let Some(reference) = value.as_str()
                        && is_local_reference(reference)
                    {
                        references.push(reference.to_owned());
                    }
                    // Definitions are reached through their references. Skipping their
                    // containers here avoids treating unrelated definitions as nested hops.
                    if !matches!(key.as_str(), "$defs" | "definitions") {
                        stack.push(value);
                    }
                }
            }
            Value::Array(values) => stack.extend(values),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value, json};

    use super::{
        CatalogKind, DRAFT_2020_12, LocalSchemaDialectPolicy, PassiveCatalogConversation,
        RequestKind, ToolDescriptionDiagnosis, classify_json_rpc_error, diagnose_tool_description,
        encode_request, has_credential_identifier_segment, normalize_credential_identifier,
        percent_decode, resolve_local_reference, scan_credential_literals, schema_child_location,
        validate_local_schema_with_policy,
    };
    use crate::contract::model::{
        FindingCode, FindingEvidence, JsonRpcErrorKind, Location, LocationField, Severity,
    };
    use crate::contract::protocol::SupportedRevision;
    use crate::contract::schema_budget::SchemaWorkBudget;

    #[test]
    fn every_passive_request_has_modern_metadata_and_no_active_method() {
        for (index, kind) in [
            RequestKind::Discover,
            RequestKind::Catalog(CatalogKind::Tools),
            RequestKind::Catalog(CatalogKind::Prompts),
            RequestKind::Catalog(CatalogKind::Resources),
            RequestKind::Catalog(CatalogKind::ResourceTemplates),
        ]
        .into_iter()
        .enumerate()
        {
            let bytes = encode_request(
                i64::try_from(index + 1).unwrap(),
                kind,
                None,
                SupportedRevision::CURRENT,
            );
            let value: Value = serde_json::from_slice(&bytes).expect("request should be JSON");
            assert_eq!(value["jsonrpc"], "2.0");
            assert_eq!(
                value["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"],
                "2026-07-28"
            );
            let text = String::from_utf8(bytes).expect("request should be UTF-8");
            assert!(!text.contains("tools/call"));
            assert!(!text.contains("prompts/get"));
            assert!(!text.contains("resources/read"));
            assert!(!text.contains("initialize"));
        }
    }

    #[test]
    fn credential_identifier_normalization_uses_only_the_fixed_exact_segments() {
        for identifier in [
            "password",
            "userPassword",
            "user_passwd",
            "client.secret",
            "auth-token",
            "apikey",
            "api_key",
            "APIKey",
            "accessToken",
            "private key",
            "serviceCredentialValue",
        ] {
            assert!(
                has_credential_identifier_segment(identifier),
                "{identifier} should contain one fixed credential segment"
            );
        }
        for identifier in [
            "tokenizer",
            "secretary",
            "passwordless",
            "key",
            "api",
            "private",
            "credentialsProvider",
            "tøken",
            "region",
        ] {
            assert!(
                !has_credential_identifier_segment(identifier),
                "{identifier} must not be inferred as a credential identifier"
            );
        }
        assert_eq!(
            normalize_credential_identifier("APIKey_Value"),
            "api-key-value"
        );
        assert_eq!(
            normalize_credential_identifier("private.Key/value"),
            "private-key-value"
        );
    }

    #[test]
    fn credential_scan_charges_the_existing_schema_work_budget() {
        let schema = json!({
            "$schema": DRAFT_2020_12,
            "type": "object",
            "properties": {
                "password": {"type": "string", "default": "never-retain-this-value"}
            }
        });
        let budget = SchemaWorkBudget::new(0);
        let scan = scan_credential_literals(
            SupportedRevision::CURRENT,
            &schema,
            Location::root(LocationField::Tools)
                .index(0)
                .field(LocationField::InputSchema),
            &budget,
        );

        assert!(scan.findings.is_empty());
        let (location, violation) = scan.limit.expect("the zero budget must stop the scan");
        assert_eq!(location.to_string(), "tools[0].inputSchema.properties[0]");
        assert_eq!(violation.kind().as_str(), "schema_evaluation_steps");
        assert_eq!(violation.maximum(), 0);
    }

    #[test]
    fn a1_v1_tool_description_normalization_is_exact_and_value_free() {
        let revisions = [
            SupportedRevision::V2025_06_18,
            SupportedRevision::V2025_11_25,
            SupportedRevision::V2026_07_28,
        ];
        let whitespace_scalars = [
            0x0009, 0x000A, 0x000B, 0x000C, 0x000D, 0x0020, 0x0085, 0x00A0, 0x1680, 0x2000, 0x2001,
            0x2002, 0x2003, 0x2004, 0x2005, 0x2006, 0x2007, 0x2008, 0x2009, 0x200A, 0x2028, 0x2029,
            0x202F, 0x205F, 0x3000,
        ];

        for revision in revisions {
            for description in std::iter::once(String::new())
                .chain(whitespace_scalars.map(|scalar| {
                    char::from_u32(scalar)
                        .expect("the A1 blank set contains Unicode scalar values")
                        .to_string()
                }))
                .chain(std::iter::once(
                    whitespace_scalars
                        .into_iter()
                        .map(|scalar| {
                            char::from_u32(scalar)
                                .expect("the A1 blank set contains Unicode scalar values")
                        })
                        .collect(),
                ))
            {
                let mut tool = Map::new();
                tool.insert("description".to_owned(), Value::String(description));
                let ToolDescriptionDiagnosis::MissingOrBlank(finding) = diagnose_tool_description(
                    revision,
                    Location::root(LocationField::Tools).index(7),
                    &tool,
                ) else {
                    panic!("every A1 v1 blank string should receive the quality diagnosis");
                };
                assert_eq!(finding.code(), FindingCode::ToolDescriptionMissingOrBlank);
                assert_eq!(finding.severity(), Severity::Warning);
                assert_eq!(finding.revision(), revision);
                assert_eq!(finding.location().to_string(), "tools[7].description");
                assert_eq!(finding.evidence(), &FindingEvidence::None);
            }

            let ToolDescriptionDiagnosis::MissingOrBlank(finding) = diagnose_tool_description(
                revision,
                Location::root(LocationField::Tools).index(8),
                &Map::new(),
            ) else {
                panic!("an absent description should receive the quality diagnosis");
            };
            assert_eq!(finding.code(), FindingCode::ToolDescriptionMissingOrBlank);
            assert_eq!(finding.location().to_string(), "tools[8].description");
            assert_eq!(finding.evidence(), &FindingEvidence::None);
        }

        for description in [
            "A synthetic tool description.",
            " \tselect this synthetic tool",
            "\u{200B}",
            "\u{FEFF}",
        ] {
            let tool = json!({"description": description});
            assert_eq!(
                diagnose_tool_description(
                    SupportedRevision::CURRENT,
                    Location::root(LocationField::Tools).index(0),
                    tool.as_object().expect("the fixture should be an object"),
                ),
                ToolDescriptionDiagnosis::Usable,
                "characters outside the fixed A1 v1 set must not be trimmed"
            );
        }

        let invalid = json!({"description": 42});
        let ToolDescriptionDiagnosis::Invalid(finding) = diagnose_tool_description(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Tools).index(9),
            invalid
                .as_object()
                .expect("the fixture should be an object"),
        ) else {
            panic!("a non-string description should remain a catalog-contract error");
        };
        assert_eq!(finding.code(), FindingCode::CatalogContractInvalid);
        assert_eq!(finding.location().to_string(), "tools[9].description");
    }

    #[test]
    fn a1_v1_placeholder_and_name_only_comparison_is_exact_and_value_free() {
        let positive = [
            ("synthetic-selector", "todo"),
            ("synthetic-selector", " T.B.D. "),
            ("synthetic-selector", "TOOL!!!"),
            ("synthetic-selector", "\tdEsCrIpTiOn\r\n"),
            ("synthetic-selector", "PLACEHOLDER"),
            ("synthetic-tool", "SYNTHETIC_TOOL"),
            ("synthetic  selector", " synthetic\tselector "),
            ("synthetic.tool", "SYNTHETIC-TOOL"),
            ("todo", "T.O.D.O."),
            ("工具", "工具"),
        ];
        for (index, (name, description)) in positive.into_iter().enumerate() {
            let tool = json!({"name": name, "description": description});
            let ToolDescriptionDiagnosis::PlaceholderOrNameOnly(finding) =
                diagnose_tool_description(
                    SupportedRevision::CURRENT,
                    Location::root(LocationField::Tools).index(index),
                    tool.as_object().expect("the fixture should be an object"),
                )
            else {
                panic!("positive A1 comparison case {index} should produce one finding");
            };
            assert_eq!(
                finding.code(),
                FindingCode::ToolDescriptionPlaceholderOrNameOnly
            );
            assert_eq!(finding.severity(), Severity::Warning);
            assert_eq!(finding.revision(), SupportedRevision::CURRENT);
            assert_eq!(
                finding.location().to_string(),
                format!("tools[{index}].description")
            );
            assert_eq!(finding.evidence(), &FindingEvidence::None);
        }

        let close_non_matches = [
            ("synthetic-selector", "todo item"),
            ("synthetic-selector", "tooling"),
            ("synthetic-selector", "description text"),
            ("synthetic-selector", "place holder"),
            ("synthetic-selector", "tbd2"),
            ("synthetic-tool", "Use synthetic tool"),
            ("café", "CAFE"),
            ("synthetic-selector", "tödö"),
        ];
        for (index, (name, description)) in close_non_matches.into_iter().enumerate() {
            let tool = json!({"name": name, "description": description});
            assert!(
                matches!(
                    diagnose_tool_description(
                        SupportedRevision::CURRENT,
                        Location::root(LocationField::Tools).index(index),
                        tool.as_object().expect("the fixture should be an object"),
                    ),
                    ToolDescriptionDiagnosis::Usable
                ),
                "close non-match case {index} must not be guessed"
            );
        }

        for description in ["", " \t\r\n"] {
            let tool = json!({"name": "todo", "description": description});
            assert!(matches!(
                diagnose_tool_description(
                    SupportedRevision::CURRENT,
                    Location::root(LocationField::Tools).index(0),
                    tool.as_object().expect("the fixture should be an object"),
                ),
                ToolDescriptionDiagnosis::MissingOrBlank(_)
            ));
        }
    }

    #[test]
    fn local_reference_resolution_supports_pointers_anchors_and_cycles() {
        let schema = json!({
            "$schema": DRAFT_2020_12,
            "$defs": {
                "space value": {"$anchor": "node", "type": "string"}
            }
        });
        assert!(resolve_local_reference(&schema, "#").is_some());
        assert!(resolve_local_reference(&schema, "#/$defs/space%20value").is_some());
        assert!(resolve_local_reference(&schema, "#node").is_some());
        assert!(resolve_local_reference(&schema, "#missing").is_none());
        assert_eq!(
            percent_decode("space%20value").as_deref(),
            Some("space value")
        );
    }

    #[test]
    fn exact_draft_policy_rejects_missing_nested_and_vocabulary_ambiguity() {
        let base = Location::root(LocationField::Tools)
            .wildcard()
            .field(LocationField::InputSchema);
        let missing = validate_local_schema_with_policy(
            &json!({"type": "object"}),
            base.clone(),
            LocalSchemaDialectPolicy::RequireExactDraft202012,
        );
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].code(), FindingCode::UnsupportedSchemaDialect);
        assert_eq!(
            missing[0].location().to_string(),
            "tools[*].inputSchema.$schema"
        );

        let exact = validate_local_schema_with_policy(
            &json!({"$schema": DRAFT_2020_12, "type": "object"}),
            base.clone(),
            LocalSchemaDialectPolicy::RequireExactDraft202012,
        );
        assert!(exact.is_empty(), "{exact:?}");

        let standard_vocabularies = validate_local_schema_with_policy(
            &json!({
                "$schema": DRAFT_2020_12,
                "$vocabulary": {
                    "https://json-schema.org/draft/2020-12/vocab/core": true,
                    "https://json-schema.org/draft/2020-12/vocab/applicator": true,
                    "https://json-schema.org/draft/2020-12/vocab/validation": true
                },
                "type": "object"
            }),
            base.clone(),
            LocalSchemaDialectPolicy::RequireExactDraft202012,
        );
        assert!(
            standard_vocabularies.is_empty(),
            "{standard_vocabularies:?}"
        );

        let nested = validate_local_schema_with_policy(
            &json!({
                "$schema": DRAFT_2020_12,
                "type": "object",
                "$defs": {
                    "private": {
                        "$schema": "https://synthetic.invalid/unsupported-dialect",
                        "type": "string"
                    }
                }
            }),
            base.clone(),
            LocalSchemaDialectPolicy::RequireExactDraft202012,
        );
        assert!(
            nested
                .iter()
                .any(|finding| finding.code() == FindingCode::UnsupportedSchemaDialect)
        );

        let vocabulary = validate_local_schema_with_policy(
            &json!({
                "$schema": DRAFT_2020_12,
                "$vocabulary": {"https://synthetic.invalid/private-vocabulary": true},
                "type": "object"
            }),
            base,
            LocalSchemaDialectPolicy::RequireExactDraft202012,
        );
        assert_eq!(vocabulary.len(), 1);
        assert_eq!(vocabulary[0].code(), FindingCode::SchemaContractInvalid);
    }

    #[test]
    fn untrusted_schema_property_names_become_wildcards() {
        let location = schema_child_location(
            Location::root(LocationField::Tools)
                .index(0)
                .field(LocationField::InputSchema)
                .field(LocationField::Properties),
            "synthetic-private-property-7f2c",
        );
        assert_eq!(location.to_string(), "tools[0].inputSchema.properties[*]");
        assert!(!location.to_string().contains("synthetic-private"));
    }

    #[test]
    fn conversation_uses_the_default_catalog_limit() {
        let conversation = PassiveCatalogConversation::new();
        assert_eq!(conversation.maximum_items, 10_000);
    }

    #[test]
    fn json_rpc_error_classification_requires_structure_and_discards_values() {
        for (code, expected) in [
            (-32700, JsonRpcErrorKind::ParseError),
            (-32600, JsonRpcErrorKind::InvalidRequest),
            (-32601, JsonRpcErrorKind::MethodNotFound),
            (-32602, JsonRpcErrorKind::InvalidParams),
            (-32603, JsonRpcErrorKind::InternalError),
            (-31999, JsonRpcErrorKind::Other),
        ] {
            let value = json!({
                "error": {
                    "code": code,
                    "message": "synthetic-private-message-never-retain",
                    "data": {"secret": "synthetic-private-data-never-retain"}
                }
            });
            assert_eq!(
                classify_json_rpc_error(value.as_object().unwrap()),
                Some(expected)
            );
        }

        for value in [
            json!({"error": null}),
            json!({"error": {"code": -32601}}),
            json!({"error": {"code": -32601, "message": 7}}),
            json!({"error": {"code": -32601.5, "message": "invalid"}}),
        ] {
            assert_eq!(classify_json_rpc_error(value.as_object().unwrap()), None);
        }
    }
}
