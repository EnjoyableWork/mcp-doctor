use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;

use jsonschema::paths::LocationSegment as SchemaLocationSegment;
use jsonschema::{Retrieve, Uri};
use serde_json::{Map, Value, json};

use super::http_headers::validate_annotations;
use super::limits::{DiagnosticLimits, LimitKind, LimitViolation};
use super::model::{
    CheckId, CheckResult, ExpectedShape, Finding, JsonKind, Location, LocationField, Requirement,
    RuleViolation, SkipReason,
};
use super::protocol::{RevisionSelection, SupportedRevision, select_server_revision};
use crate::transport::{Conversation, ProbeRequest, ProbeResponse};

const PROTOCOL_REVISION: &str = "2026-07-28";
const DRAFT_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";

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
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum RequestKind {
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

/// Drives only capability-gated list requests. It never constructs `tools/call`,
/// `prompts/get`, `resources/read`, or an initialization handshake.
pub(crate) struct PassiveCatalogConversation {
    started: bool,
    stopped: bool,
    next_id: i64,
    queue: VecDeque<CatalogKind>,
    records: Vec<RequestRecord>,
    pages: BTreeMap<CatalogKind, usize>,
    seen_cursors: BTreeMap<CatalogKind, BTreeSet<String>>,
    observed_items: u64,
    maximum_items: u64,
    validate_http_headers: bool,
}

impl PassiveCatalogConversation {
    pub(crate) fn new() -> Self {
        Self::with_catalog_limit(DiagnosticLimits::M1_DEFAULTS.values().catalog_items)
    }

    fn with_catalog_limit(maximum_items: u64) -> Self {
        Self {
            started: false,
            stopped: false,
            next_id: 1,
            queue: VecDeque::new(),
            records: Vec::new(),
            pages: BTreeMap::new(),
            seen_cursors: BTreeMap::new(),
            observed_items: 0,
            maximum_items,
            validate_http_headers: false,
        }
    }

    pub(crate) fn new_http() -> Self {
        let mut conversation = Self::new();
        conversation.validate_http_headers = true;
        conversation
    }

    fn record_request(&mut self, kind: RequestKind, cursor: Option<String>) -> ProbeRequest {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("the bounded message count keeps request ids representable");
        let page = match kind {
            RequestKind::Discover => 0,
            RequestKind::Catalog(catalog) => {
                let page = self.pages.entry(catalog).or_default();
                let current = *page;
                *page = page.saturating_add(1);
                current
            }
        };
        let bytes = encode_request(id, kind, cursor.as_deref());
        self.records.push(RequestRecord {
            id,
            kind,
            page,
            cursor,
        });
        ProbeRequest::new(id, bytes)
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
            RequestKind::Discover => {
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
            return Some(self.record_request(RequestKind::Discover, None));
        }

        let response = previous.expect("each later request follows a matching response");
        self.advance_after(response)
            .map(|(kind, cursor)| self.record_request(kind, cursor))
    }
}

fn encode_request(id: i64, kind: RequestKind, cursor: Option<&str>) -> Vec<u8> {
    let method = match kind {
        RequestKind::Discover => "server/discover",
        RequestKind::Catalog(kind) => kind.method(),
    };
    let mut params = Map::new();
    params.insert("_meta".to_owned(), request_meta());
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
    Schema,
}

struct Analyzer {
    limits: DiagnosticLimits,
    revision: Vec<Finding>,
    envelope: Vec<Finding>,
    catalog: Vec<Finding>,
    schema: Vec<Finding>,
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
}

impl Analyzer {
    fn new(reserved_findings: usize, validate_http_headers: bool) -> Self {
        let limits = DiagnosticLimits::M1_DEFAULTS;
        let maximum = usize::try_from(limits.values().report_findings).unwrap_or(usize::MAX);
        Self {
            limits,
            revision: Vec::new(),
            envelope: Vec::new(),
            catalog: Vec::new(),
            schema: Vec::new(),
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
        }
    }

    fn push(&mut self, bucket: FindingBucket, finding: Finding) {
        let duplicate = match bucket {
            FindingBucket::Revision => self.revision.contains(&finding),
            FindingBucket::Envelope => self.envelope.contains(&finding),
            FindingBucket::Catalog => self.catalog.contains(&finding),
            FindingBucket::Schema => self.schema.contains(&finding),
        };
        if duplicate {
            return;
        }
        if self.stored_findings < self.finding_capacity {
            match bucket {
                FindingBucket::Revision => self.revision.push(finding),
                FindingBucket::Envelope => self.envelope.push(finding),
                FindingBucket::Catalog => self.catalog.push(finding),
                FindingBucket::Schema => self.schema.push(finding),
            }
            self.stored_findings += 1;
        } else {
            self.finding_overflow = true;
        }
    }

    fn finish_overflow(&mut self) {
        if !self.finding_overflow || self.finding_capacity == 0 {
            return;
        }
        let removed = self
            .schema
            .pop()
            .or_else(|| self.catalog.pop())
            .or_else(|| self.envelope.pop())
            .or_else(|| self.revision.pop());
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
        if object.contains_key("error") {
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

        match select_server_revision(
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
        if object.contains_key("error") {
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
        if input_object.get("type").and_then(Value::as_str) != Some("object") {
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
        self.analyze_schema(input_schema, input_location);
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
                self.analyze_schema(output_schema, output_location);
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

    fn analyze_schema(&mut self, schema: &Value, base: Location) {
        let values = self.limits.values();
        let bytes = u64::try_from(serialized_len(schema)).unwrap_or(u64::MAX);
        if bytes > values.schema_bytes {
            self.schema_limit(base, LimitKind::SchemaBytes, bytes, values.schema_bytes);
            return;
        }

        let Some(object) = schema.as_object() else {
            unreachable!("tool schema object shape was checked before schema analysis")
        };
        if let Some(dialect) = object.get("$schema")
            && dialect.as_str() != Some(DRAFT_2020_12)
        {
            self.push(
                FindingBucket::Schema,
                Finding::unsupported_schema_dialect(
                    SupportedRevision::CURRENT,
                    base.clone().field(LocationField::Schema),
                    json_kind(Some(dialect)),
                ),
            );
            return;
        }

        let mut stack = vec![(schema, 0_u64, base.clone())];
        let mut nodes = 0_u64;
        let mut work = 0_u64;
        let mut references = Vec::new();
        let mut external_reference = false;
        while let Some((value, depth, location)) = stack.pop() {
            nodes = nodes.saturating_add(1);
            work = work.saturating_add(1);
            if nodes > values.schema_nodes {
                self.schema_limit(location, LimitKind::SchemaNodes, nodes, values.schema_nodes);
                return;
            }
            if work > values.schema_evaluation_steps {
                self.schema_limit(
                    location,
                    LimitKind::SchemaEvaluationSteps,
                    work,
                    values.schema_evaluation_steps,
                );
                return;
            }
            if depth > values.schema_depth {
                self.schema_limit(location, LimitKind::SchemaDepth, depth, values.schema_depth);
                return;
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
                        let child_location = schema_child_location(location.clone(), key);
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
                return;
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
            return;
        }

        if external_reference || unresolved_reference {
            return;
        }

        let maximum_errors = values.validation_errors;
        let meta_validator = jsonschema::draft202012::meta::validator();
        let mut errors = meta_validator.iter_errors(schema).take(
            usize::try_from(maximum_errors)
                .unwrap_or(usize::MAX)
                .saturating_add(1),
        );
        let first_error = errors.next();
        let error_location = first_error.as_ref().map_or_else(
            || base.clone(),
            |error| append_schema_error_location(base.clone(), error.instance_path()),
        );
        let error_count = if first_error.is_some() {
            u64::try_from(errors.count())
                .unwrap_or(u64::MAX)
                .saturating_add(1)
        } else {
            0
        };
        if error_count > 0 {
            self.push(
                FindingBucket::Schema,
                Finding::schema_contract_invalid(
                    SupportedRevision::CURRENT,
                    error_location,
                    RuleViolation::InvalidDraft202012 { error_count },
                ),
            );
            if error_count > maximum_errors {
                self.schema_limit(
                    base,
                    LimitKind::ValidationErrors,
                    error_count,
                    maximum_errors,
                );
            }
            return;
        }

        if let Err(error) = jsonschema::draft202012::options()
            .with_retriever(NoExternalRetrieval)
            .build(schema)
        {
            self.push(
                FindingBucket::Schema,
                Finding::schema_contract_invalid(
                    SupportedRevision::CURRENT,
                    append_schema_error_location(base, error.instance_path()),
                    RuleViolation::InvalidDraft202012 { error_count: 1 },
                ),
            );
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

    fn into_checks(mut self) -> Vec<CheckResult> {
        self.finish_overflow();
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

pub(super) fn validate_local_schema(schema: &Value, base: Location) -> Vec<Finding> {
    let mut analyzer = Analyzer::new(0, false);
    analyzer.analyze_schema(schema, base.clone());
    if analyzer.finding_overflow {
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
    analyzer.schema
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum InstanceValidationIssue {
    Mismatch { error_count: u64 },
    Limit(LimitViolation),
    InvalidSchema,
}

pub(super) struct LocalValidator {
    validator: jsonschema::Validator,
}

impl LocalValidator {
    pub(super) fn compile(schema: &Value) -> Result<Self, InstanceValidationIssue> {
        let validator = jsonschema::draft202012::options()
            .with_retriever(NoExternalRetrieval)
            .build(schema)
            .map_err(|_| InstanceValidationIssue::InvalidSchema)?;
        Ok(Self { validator })
    }

    pub(super) fn validate(&self, instance: &Value) -> Result<(), InstanceValidationIssue> {
        let values = DiagnosticLimits::M1_DEFAULTS.values();
        let bytes = u64::try_from(serialized_len(instance)).unwrap_or(u64::MAX);
        if bytes > values.instance_bytes {
            return Err(InstanceValidationIssue::Limit(
                LimitViolation::new(LimitKind::InstanceBytes, bytes, values.instance_bytes)
                    .expect("the instance byte count exceeds its checked maximum"),
            ));
        }

        let mut stack = vec![(instance, 0_u64)];
        let mut work = 0_u64;
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
                Value::Array(values) => {
                    stack.extend(
                        values
                            .iter()
                            .rev()
                            .map(|value| (value, depth.saturating_add(1))),
                    );
                }
                Value::Object(values) => {
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

        let maximum_errors = values.validation_errors;
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
        if error_count > maximum_errors {
            return Err(InstanceValidationIssue::Limit(
                LimitViolation::new(LimitKind::ValidationErrors, error_count, maximum_errors)
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
    let mut analyzer = Analyzer::new(reserved_findings, conversation.validate_http_headers);
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

fn schema_child_location(location: Location, key: &str) -> Location {
    let field = match key {
        "$schema" => Some(LocationField::Schema),
        "type" => Some(LocationField::Type),
        "properties" => Some(LocationField::Properties),
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

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{
        CatalogKind, DRAFT_2020_12, PassiveCatalogConversation, RequestKind, encode_request,
        percent_decode, resolve_local_reference, schema_child_location,
    };
    use crate::contract::model::{Location, LocationField};

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
            let bytes = encode_request(i64::try_from(index + 1).unwrap(), kind, None);
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
    fn conversation_defaults_to_the_m1_catalog_limit() {
        let conversation = PassiveCatalogConversation::new();
        assert_eq!(conversation.maximum_items, 10_000);
    }
}
