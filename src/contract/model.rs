use std::fmt;

use super::limits::LimitViolation;
use super::protocol::{RevisionAdvertisementSummary, SupportedRevision};
use super::redaction::RedactedValue;

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum CheckId {
    TransportStdio,
    ProtocolEnvelope,
    ProtocolRevision,
    DiscoveryCatalogs,
    SchemaContracts,
    RuntimeTools,
}

impl CheckId {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::TransportStdio => "transport.stdio",
            Self::ProtocolRevision => "protocol.revision",
            Self::ProtocolEnvelope => "protocol.envelope",
            Self::DiscoveryCatalogs => "discovery.catalogs",
            Self::SchemaContracts => "schema.contracts",
            Self::RuntimeTools => "runtime.tools",
        }
    }
}

impl fmt::Display for CheckId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Requirement {
    Required,
    Optional,
}

impl Requirement {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Optional => "optional",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum SkipReason {
    NotAuthorized,
    NotAdvertised,
    UnsupportedRevision,
    PrerequisiteFailed,
    LimitReached,
    NotApplicable,
}

impl SkipReason {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::NotAuthorized => "not_authorized",
            Self::NotAdvertised => "not_advertised",
            Self::UnsupportedRevision => "unsupported_revision",
            Self::PrerequisiteFailed => "prerequisite_failed",
            Self::LimitReached => "limit_reached",
            Self::NotApplicable => "not_applicable",
        }
    }

    pub(super) const fn description(self) -> &'static str {
        match self {
            Self::NotAuthorized => "active behavior was not authorized",
            Self::NotAdvertised => "the capability was not advertised",
            Self::UnsupportedRevision => "the protocol revision is unsupported",
            Self::PrerequisiteFailed => "a required prerequisite did not pass",
            Self::LimitReached => "a safety limit prevented the check",
            Self::NotApplicable => "the check does not apply to this target",
        }
    }

    pub(super) const fn is_causal(self) -> bool {
        matches!(
            self,
            Self::UnsupportedRevision | Self::PrerequisiteFailed | Self::LimitReached
        )
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

impl Severity {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }

    pub(super) const fn is_failure(self) -> bool {
        matches!(self, Self::Error | Self::Critical)
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum FindingCode {
    ProcessStartFailed,
    StdioIoFailed,
    InvalidStdioMessage,
    ServerExitedEarly,
    ProtocolRevisionConfirmed,
    UnsupportedProtocolRevision,
    InvalidProtocolRevisionValue,
    DeprecatedProtocolFeature,
    LimitExceeded,
    CleanupFailed,
    CatalogContractInvalid,
    DuplicateCatalogIdentifier,
    PaginationCursorRepeated,
    SchemaContractInvalid,
    UnsupportedSchemaDialect,
    ExternalSchemaReferenceBlocked,
}

impl FindingCode {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ProcessStartFailed => "MCP-TRANSPORT-001",
            Self::StdioIoFailed => "MCP-TRANSPORT-002",
            Self::InvalidStdioMessage => "MCP-TRANSPORT-003",
            Self::ServerExitedEarly => "MCP-TRANSPORT-004",
            Self::ProtocolRevisionConfirmed => "MCP-PROTOCOL-001",
            Self::UnsupportedProtocolRevision => "MCP-PROTOCOL-002",
            Self::InvalidProtocolRevisionValue => "MCP-PROTOCOL-003",
            Self::DeprecatedProtocolFeature => "MCP-PROTOCOL-004",
            Self::LimitExceeded => "MCP-LIMIT-001",
            Self::CleanupFailed => "MCP-SAFETY-001",
            Self::CatalogContractInvalid => "MCP-CATALOG-001",
            Self::DuplicateCatalogIdentifier => "MCP-CATALOG-002",
            Self::PaginationCursorRepeated => "MCP-CATALOG-003",
            Self::SchemaContractInvalid => "MCP-SCHEMA-001",
            Self::UnsupportedSchemaDialect => "MCP-SCHEMA-002",
            Self::ExternalSchemaReferenceBlocked => "MCP-SCHEMA-003",
        }
    }

    pub(super) const fn severity(self) -> Severity {
        match self {
            Self::ProtocolRevisionConfirmed => Severity::Info,
            Self::DeprecatedProtocolFeature => Severity::Warning,
            Self::ProcessStartFailed
            | Self::StdioIoFailed
            | Self::InvalidStdioMessage
            | Self::ServerExitedEarly
            | Self::UnsupportedProtocolRevision
            | Self::InvalidProtocolRevisionValue
            | Self::LimitExceeded
            | Self::CatalogContractInvalid
            | Self::DuplicateCatalogIdentifier
            | Self::PaginationCursorRepeated
            | Self::SchemaContractInvalid
            | Self::UnsupportedSchemaDialect
            | Self::ExternalSchemaReferenceBlocked => Severity::Error,
            Self::CleanupFailed => Severity::Critical,
        }
    }

    pub(super) const fn title(self) -> &'static str {
        match self {
            Self::ProcessStartFailed => "The MCP server process could not be started.",
            Self::StdioIoFailed => "The STDIO channel failed before diagnosis completed.",
            Self::InvalidStdioMessage => "The server wrote an invalid STDIO message.",
            Self::ServerExitedEarly => "The server process exited before returning a response.",
            Self::ProtocolRevisionConfirmed => "The requested protocol revision is supported.",
            Self::UnsupportedProtocolRevision => {
                "The server does not advertise the required protocol revision."
            }
            Self::InvalidProtocolRevisionValue => {
                "The protocol revision value is missing or has the wrong JSON type."
            }
            Self::DeprecatedProtocolFeature => {
                "The server advertises a feature deprecated by this protocol revision."
            }
            Self::LimitExceeded => "A configured diagnostic safety limit was exceeded.",
            Self::CleanupFailed => "The managed target could not be fully cleaned up.",
            Self::CatalogContractInvalid => {
                "An advertised MCP catalog does not match its protocol contract."
            }
            Self::DuplicateCatalogIdentifier => {
                "An advertised catalog contains a duplicate identifier."
            }
            Self::PaginationCursorRepeated => {
                "A catalog repeated a pagination cursor and inspection stopped."
            }
            Self::SchemaContractInvalid => "An advertised JSON Schema contract is invalid.",
            Self::UnsupportedSchemaDialect => {
                "An advertised schema uses an unsupported JSON Schema dialect."
            }
            Self::ExternalSchemaReferenceBlocked => {
                "An advertised schema requires external reference retrieval."
            }
        }
    }

    pub(super) const fn impact(self) -> &'static str {
        match self {
            Self::ProcessStartFailed => {
                "No protocol or contract check can run until the target starts."
            }
            Self::StdioIoFailed => {
                "A broken channel prevents the passive inspection from completing reliably."
            }
            Self::InvalidStdioMessage => {
                "MCP clients cannot interpret this output as a JSON-RPC response."
            }
            Self::ServerExitedEarly => {
                "The pending passive request cannot complete after the server exits."
            }
            Self::ProtocolRevisionConfirmed => {
                "The advertised revision determines which protocol rules can be applied."
            }
            Self::UnsupportedProtocolRevision => {
                "Applying 2026-07-28 rules to another revision could produce a false diagnosis."
            }
            Self::InvalidProtocolRevisionValue => {
                "A valid protocol rule set cannot be selected from this advertisement."
            }
            Self::DeprecatedProtocolFeature => {
                "Deprecated behavior may stop working in a future protocol revision."
            }
            Self::LimitExceeded => {
                "Continuing past this bound could make inspection unsafe or unbounded."
            }
            Self::CleanupFailed => {
                "A surviving process can keep consuming resources or running after inspection."
            }
            Self::CatalogContractInvalid => {
                "Clients cannot reliably discover or use a capability with this structure."
            }
            Self::DuplicateCatalogIdentifier => {
                "Clients cannot reliably choose between items with the same identifier."
            }
            Self::PaginationCursorRepeated => {
                "The repeated cursor would prevent discovery from reaching a complete result."
            }
            Self::SchemaContractInvalid => {
                "Clients cannot safely construct or validate values from this schema."
            }
            Self::UnsupportedSchemaDialect => {
                "Interpreting another dialect as Draft 2020-12 could produce incorrect results."
            }
            Self::ExternalSchemaReferenceBlocked => {
                "Resolving this contract would require network or file access that was not authorized."
            }
        }
    }

    pub(super) const fn expectation(self) -> &'static str {
        match self {
            Self::ProcessStartFailed => "The target must be a directly executable local program.",
            Self::StdioIoFailed => {
                "The target must keep its STDIO pipes available through the passive inspection."
            }
            Self::InvalidStdioMessage => {
                "Each STDOUT frame must be one valid JSON-RPC 2.0 message terminated by a newline."
            }
            Self::ServerExitedEarly => {
                "The server must remain alive until it returns the pending passive response."
            }
            Self::ProtocolRevisionConfirmed => {
                "The server advertises MCP protocol revision 2026-07-28."
            }
            Self::UnsupportedProtocolRevision => {
                "server/discover must advertise MCP protocol revision 2026-07-28."
            }
            Self::InvalidProtocolRevisionValue => {
                "supportedVersions must be an array of protocol revision strings."
            }
            Self::DeprecatedProtocolFeature => {
                "Servers should avoid features deprecated by MCP 2026-07-28."
            }
            Self::LimitExceeded => {
                "Passive inspection must remain within every configured safety limit."
            }
            Self::CleanupFailed => {
                "The managed process tree must terminate and be reaped before inspection returns."
            }
            Self::CatalogContractInvalid => {
                "Each advertised catalog response and item must match MCP 2026-07-28."
            }
            Self::DuplicateCatalogIdentifier => {
                "Identifiers must be unique within their advertised catalog scope."
            }
            Self::PaginationCursorRepeated => {
                "Each nextCursor must advance the catalog or end pagination."
            }
            Self::SchemaContractInvalid => {
                "Tool schemas must be valid local JSON Schema Draft 2020-12 objects."
            }
            Self::UnsupportedSchemaDialect => {
                "Tool schemas must omit $schema or declare JSON Schema Draft 2020-12."
            }
            Self::ExternalSchemaReferenceBlocked => {
                "Passive inspection accepts only references contained in the advertised schema."
            }
        }
    }

    pub(super) const fn remediation(self) -> &'static str {
        match self {
            Self::ProcessStartFailed => {
                "Check the executable path and permissions, then rerun inspect."
            }
            Self::StdioIoFailed => "Fix the server's STDIO lifecycle and rerun inspect.",
            Self::InvalidStdioMessage => {
                "Write only newline-delimited JSON-RPC messages to STDOUT; send logs to STDERR."
            }
            Self::ServerExitedEarly => "Keep the server alive long enough to answer the request.",
            Self::ProtocolRevisionConfirmed => "No correction is needed.",
            Self::UnsupportedProtocolRevision => {
                "Add MCP 2026-07-28 support and advertise it from server/discover."
            }
            Self::InvalidProtocolRevisionValue => {
                "Return supportedVersions as an array containing string revision identifiers."
            }
            Self::DeprecatedProtocolFeature => "Remove or replace the deprecated capability.",
            Self::LimitExceeded => {
                "Reduce the advertised data or work below the reported maximum, then rerun inspect."
            }
            Self::CleanupFailed => {
                "Make the server and descendants exit when STDIN closes or termination is requested."
            }
            Self::CatalogContractInvalid => {
                "Correct the value at the reported structural location, then rerun inspect."
            }
            Self::DuplicateCatalogIdentifier => {
                "Rename or remove the later duplicate so each identifier is unique."
            }
            Self::PaginationCursorRepeated => {
                "Return a new cursor for the next page or omit nextCursor on the final page."
            }
            Self::SchemaContractInvalid => {
                "Correct the schema at the reported structural location and validate it as Draft 2020-12."
            }
            Self::UnsupportedSchemaDialect => {
                "Remove $schema to use the MCP default or declare the Draft 2020-12 schema URI."
            }
            Self::ExternalSchemaReferenceBlocked => {
                "Inline the referenced schema or move it into local $defs and use a fragment reference."
            }
        }
    }

    pub(super) const fn reference(self) -> &'static str {
        match self {
            Self::ProcessStartFailed
            | Self::StdioIoFailed
            | Self::InvalidStdioMessage
            | Self::ServerExitedEarly
            | Self::LimitExceeded
            | Self::CleanupFailed => "mcp-doctor M1 passive STDIO safety contract",
            Self::ProtocolRevisionConfirmed
            | Self::UnsupportedProtocolRevision
            | Self::InvalidProtocolRevisionValue
            | Self::DeprecatedProtocolFeature => "MCP 2026-07-28 server/discover contract",
            Self::CatalogContractInvalid
            | Self::DuplicateCatalogIdentifier
            | Self::PaginationCursorRepeated => "MCP 2026-07-28 catalog contracts",
            Self::SchemaContractInvalid
            | Self::UnsupportedSchemaDialect
            | Self::ExternalSchemaReferenceBlocked => {
                "MCP 2026-07-28 Tool contract and JSON Schema Draft 2020-12"
            }
        }
    }

    pub(super) const fn is_independent_safety(self) -> bool {
        matches!(self, Self::CleanupFailed)
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum LocationField {
    Request,
    Meta,
    ProtocolVersion,
    Server,
    SupportedVersions,
    Tools,
    Prompts,
    Resources,
    ResourceTemplates,
    Capabilities,
    Result,
    ResultType,
    TtlMs,
    CacheScope,
    NextCursor,
    Name,
    Arguments,
    ListChanged,
    Subscribe,
    Uri,
    UriTemplate,
    InputSchema,
    OutputSchema,
    Schema,
    Type,
    Properties,
    Defs,
    Ref,
    DynamicRef,
    Items,
    PrefixItems,
    AllOf,
    AnyOf,
    OneOf,
    Not,
    If,
    Then,
    Else,
    Required,
    Process,
    Stdin,
    Stdout,
    Stderr,
    Message,
}

impl LocationField {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::Meta => "_meta",
            Self::ProtocolVersion => "io.modelcontextprotocol/protocolVersion",
            Self::Server => "server",
            Self::SupportedVersions => "supportedVersions",
            Self::Tools => "tools",
            Self::Prompts => "prompts",
            Self::Resources => "resources",
            Self::ResourceTemplates => "resourceTemplates",
            Self::Capabilities => "capabilities",
            Self::Result => "result",
            Self::ResultType => "resultType",
            Self::TtlMs => "ttlMs",
            Self::CacheScope => "cacheScope",
            Self::NextCursor => "nextCursor",
            Self::Name => "name",
            Self::Arguments => "arguments",
            Self::ListChanged => "listChanged",
            Self::Subscribe => "subscribe",
            Self::Uri => "uri",
            Self::UriTemplate => "uriTemplate",
            Self::InputSchema => "inputSchema",
            Self::OutputSchema => "outputSchema",
            Self::Schema => "$schema",
            Self::Type => "type",
            Self::Properties => "properties",
            Self::Defs => "$defs",
            Self::Ref => "$ref",
            Self::DynamicRef => "$dynamicRef",
            Self::Items => "items",
            Self::PrefixItems => "prefixItems",
            Self::AllOf => "allOf",
            Self::AnyOf => "anyOf",
            Self::OneOf => "oneOf",
            Self::Not => "not",
            Self::If => "if",
            Self::Then => "then",
            Self::Else => "else",
            Self::Required => "required",
            Self::Process => "process",
            Self::Stdin => "stdin",
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
            Self::Message => "message",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum LocationSegment {
    Field(LocationField),
    Index(usize),
    Wildcard,
}

/// A location made only from trusted field identifiers and numeric indices.
/// It cannot contain paths, payloads, server-provided names, or credentials.
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct Location {
    segments: Vec<LocationSegment>,
}

impl Location {
    pub(super) fn root(field: LocationField) -> Self {
        Self {
            segments: vec![LocationSegment::Field(field)],
        }
    }

    pub(super) fn field(mut self, field: LocationField) -> Self {
        self.segments.push(LocationSegment::Field(field));
        self
    }

    pub(super) fn index(mut self, index: usize) -> Self {
        self.segments.push(LocationSegment::Index(index));
        self
    }

    pub(super) fn wildcard(mut self) -> Self {
        self.segments.push(LocationSegment::Wildcard);
        self
    }
}

impl fmt::Display for Location {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, segment) in self.segments.iter().enumerate() {
            match segment {
                LocationSegment::Field(field) => {
                    if index > 0 {
                        formatter.write_str(".")?;
                    }
                    formatter.write_str(field.as_str())?;
                }
                LocationSegment::Index(value) => write!(formatter, "[{value}]")?,
                LocationSegment::Wildcard => formatter.write_str("[*]")?,
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum JsonKind {
    Missing,
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
}

impl JsonKind {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Null => "null",
            Self::Boolean => "boolean",
            Self::Number => "number",
            Self::String => "string",
            Self::Array => "array",
            Self::Object => "object",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ExpectedShape {
    Object,
    Array,
    String,
    Boolean,
    NonNegativeNumber,
}

impl ExpectedShape {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Object => "object",
            Self::Array => "array",
            Self::String => "string",
            Self::Boolean => "boolean",
            Self::NonNegativeNumber => "non-negative number",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum RuleViolation {
    ExpectedShape {
        expected: ExpectedShape,
        observed: JsonKind,
    },
    ExpectedCompleteResult {
        observed: JsonKind,
    },
    ExpectedCacheScope {
        observed: JsonKind,
    },
    ExpectedCurrentRevision,
    ExpectedInputSchemaRootObject {
        observed: JsonKind,
    },
    ServerErrorResponse,
    DuplicateIdentifier,
    RepeatedCursor,
    UnsupportedSchemaDialect {
        observed: JsonKind,
    },
    ExternalSchemaReference,
    UnresolvedLocalReference,
    InvalidDraft202012 {
        error_count: u64,
    },
}

impl RuleViolation {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ExpectedShape { .. } => "expected_shape",
            Self::ExpectedCompleteResult { .. } => "expected_complete_result",
            Self::ExpectedCacheScope { .. } => "expected_cache_scope",
            Self::ExpectedCurrentRevision => "expected_current_revision",
            Self::ExpectedInputSchemaRootObject { .. } => "expected_input_schema_root_object",
            Self::ServerErrorResponse => "server_error_response",
            Self::DuplicateIdentifier => "duplicate_identifier",
            Self::RepeatedCursor => "repeated_cursor",
            Self::UnsupportedSchemaDialect { .. } => "unsupported_schema_dialect",
            Self::ExternalSchemaReference => "external_schema_reference",
            Self::UnresolvedLocalReference => "unresolved_local_reference",
            Self::InvalidDraft202012 { .. } => "invalid_draft_2020_12",
        }
    }

    pub(super) const fn observed(self) -> Option<JsonKind> {
        match self {
            Self::ExpectedShape { observed, .. }
            | Self::ExpectedCompleteResult { observed }
            | Self::ExpectedCacheScope { observed }
            | Self::ExpectedInputSchemaRootObject { observed }
            | Self::UnsupportedSchemaDialect { observed } => Some(observed),
            Self::ExpectedCurrentRevision
            | Self::ServerErrorResponse
            | Self::DuplicateIdentifier
            | Self::RepeatedCursor
            | Self::ExternalSchemaReference
            | Self::UnresolvedLocalReference
            | Self::InvalidDraft202012 { .. } => None,
        }
    }

    pub(super) const fn expected_shape(self) -> Option<ExpectedShape> {
        match self {
            Self::ExpectedShape { expected, .. } => Some(expected),
            _ => None,
        }
    }

    pub(super) const fn error_count(self) -> Option<u64> {
        match self {
            Self::InvalidDraft202012 { error_count } => Some(error_count),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum FindingEvidence {
    None,
    RevisionAdvertisement(RevisionAdvertisementSummary),
    RedactedObservation(RedactedValue),
    LimitViolation(LimitViolation),
    RuleViolation(RuleViolation),
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct Finding {
    code: FindingCode,
    revision: SupportedRevision,
    location: Location,
    evidence: FindingEvidence,
}

impl Finding {
    pub(super) fn process_start_failed(revision: SupportedRevision, location: Location) -> Self {
        Self::new(
            FindingCode::ProcessStartFailed,
            revision,
            location,
            FindingEvidence::None,
        )
    }

    pub(super) fn stdio_io_failed(revision: SupportedRevision, location: Location) -> Self {
        Self::new(
            FindingCode::StdioIoFailed,
            revision,
            location,
            FindingEvidence::None,
        )
    }

    pub(super) fn invalid_stdio_message(
        revision: SupportedRevision,
        location: Location,
        observation: RedactedValue,
    ) -> Self {
        Self::new(
            FindingCode::InvalidStdioMessage,
            revision,
            location,
            FindingEvidence::RedactedObservation(observation),
        )
    }

    pub(super) fn server_exited_early(revision: SupportedRevision, location: Location) -> Self {
        Self::new(
            FindingCode::ServerExitedEarly,
            revision,
            location,
            FindingEvidence::None,
        )
    }

    pub(super) fn revision_confirmed(revision: SupportedRevision, location: Location) -> Self {
        Self::new(
            FindingCode::ProtocolRevisionConfirmed,
            revision,
            location,
            FindingEvidence::None,
        )
    }

    pub(super) fn unsupported_revision(
        revision: SupportedRevision,
        location: Location,
        advertisement: RevisionAdvertisementSummary,
    ) -> Self {
        Self::new(
            FindingCode::UnsupportedProtocolRevision,
            revision,
            location,
            FindingEvidence::RevisionAdvertisement(advertisement),
        )
    }

    pub(super) fn invalid_revision_value(
        revision: SupportedRevision,
        location: Location,
        observation: RedactedValue,
    ) -> Self {
        Self::new(
            FindingCode::InvalidProtocolRevisionValue,
            revision,
            location,
            FindingEvidence::RedactedObservation(observation),
        )
    }

    pub(super) fn deprecated_protocol_feature(
        revision: SupportedRevision,
        location: Location,
    ) -> Self {
        Self::new(
            FindingCode::DeprecatedProtocolFeature,
            revision,
            location,
            FindingEvidence::None,
        )
    }

    pub(super) fn limit_exceeded(
        revision: SupportedRevision,
        location: Location,
        violation: LimitViolation,
    ) -> Self {
        Self::new(
            FindingCode::LimitExceeded,
            revision,
            location,
            FindingEvidence::LimitViolation(violation),
        )
    }

    pub(super) fn cleanup_failed(revision: SupportedRevision, location: Location) -> Self {
        Self::new(
            FindingCode::CleanupFailed,
            revision,
            location,
            FindingEvidence::None,
        )
    }

    pub(super) fn catalog_contract_invalid(
        revision: SupportedRevision,
        location: Location,
        violation: RuleViolation,
    ) -> Self {
        Self::new(
            FindingCode::CatalogContractInvalid,
            revision,
            location,
            FindingEvidence::RuleViolation(violation),
        )
    }

    pub(super) fn duplicate_catalog_identifier(
        revision: SupportedRevision,
        location: Location,
    ) -> Self {
        Self::new(
            FindingCode::DuplicateCatalogIdentifier,
            revision,
            location,
            FindingEvidence::RuleViolation(RuleViolation::DuplicateIdentifier),
        )
    }

    pub(super) fn pagination_cursor_repeated(
        revision: SupportedRevision,
        location: Location,
    ) -> Self {
        Self::new(
            FindingCode::PaginationCursorRepeated,
            revision,
            location,
            FindingEvidence::RuleViolation(RuleViolation::RepeatedCursor),
        )
    }

    pub(super) fn schema_contract_invalid(
        revision: SupportedRevision,
        location: Location,
        violation: RuleViolation,
    ) -> Self {
        Self::new(
            FindingCode::SchemaContractInvalid,
            revision,
            location,
            FindingEvidence::RuleViolation(violation),
        )
    }

    pub(super) fn unsupported_schema_dialect(
        revision: SupportedRevision,
        location: Location,
        observed: JsonKind,
    ) -> Self {
        Self::new(
            FindingCode::UnsupportedSchemaDialect,
            revision,
            location,
            FindingEvidence::RuleViolation(RuleViolation::UnsupportedSchemaDialect { observed }),
        )
    }

    pub(super) fn external_schema_reference_blocked(
        revision: SupportedRevision,
        location: Location,
    ) -> Self {
        Self::new(
            FindingCode::ExternalSchemaReferenceBlocked,
            revision,
            location,
            FindingEvidence::RuleViolation(RuleViolation::ExternalSchemaReference),
        )
    }

    fn new(
        code: FindingCode,
        revision: SupportedRevision,
        location: Location,
        evidence: FindingEvidence,
    ) -> Self {
        Self {
            code,
            revision,
            location,
            evidence,
        }
    }

    pub(super) const fn code(&self) -> FindingCode {
        self.code
    }

    pub(super) const fn severity(&self) -> Severity {
        self.code.severity()
    }

    pub(super) const fn revision(&self) -> SupportedRevision {
        self.revision
    }

    pub(super) fn location(&self) -> &Location {
        &self.location
    }

    pub(super) fn evidence(&self) -> &FindingEvidence {
        &self.evidence
    }

    pub(super) const fn expectation(&self) -> &'static str {
        self.code.expectation()
    }

    pub(super) const fn impact(&self) -> &'static str {
        self.code.impact()
    }

    pub(super) const fn remediation(&self) -> &'static str {
        self.code.remediation()
    }

    pub(super) const fn reference(&self) -> &'static str {
        self.code.reference()
    }

    pub(super) const fn is_independent_safety(&self) -> bool {
        self.code.is_independent_safety()
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum CheckOutcome {
    Passed,
    Warning,
    Failed,
}

impl CheckOutcome {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Failed => "failed",
        }
    }

    pub(super) const fn human_label(self) -> &'static str {
        match self {
            Self::Passed => "PASS",
            Self::Warning => "WARN",
            Self::Failed => "FAIL",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum CheckState {
    Performed { findings: Vec<Finding> },
    Skipped { reason: SkipReason },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct CheckResult {
    id: CheckId,
    requirement: Requirement,
    state: CheckState,
}

impl CheckResult {
    pub(super) fn performed(
        id: CheckId,
        requirement: Requirement,
        mut findings: Vec<Finding>,
    ) -> Self {
        findings.sort();
        findings.dedup();
        Self {
            id,
            requirement,
            state: CheckState::Performed { findings },
        }
    }

    pub(super) const fn skipped(id: CheckId, requirement: Requirement, reason: SkipReason) -> Self {
        Self {
            id,
            requirement,
            state: CheckState::Skipped { reason },
        }
    }

    pub(super) const fn id(&self) -> CheckId {
        self.id
    }

    pub(super) const fn requirement(&self) -> Requirement {
        self.requirement
    }

    pub(super) fn findings(&self) -> Option<&[Finding]> {
        match &self.state {
            CheckState::Performed { findings } => Some(findings),
            CheckState::Skipped { .. } => None,
        }
    }

    pub(super) const fn skip_reason(&self) -> Option<SkipReason> {
        match self.state {
            CheckState::Performed { .. } => None,
            CheckState::Skipped { reason } => Some(reason),
        }
    }

    pub(super) fn outcome(&self) -> Option<CheckOutcome> {
        let findings = self.findings()?;
        if findings
            .iter()
            .any(|finding| finding.severity().is_failure())
        {
            Some(CheckOutcome::Failed)
        } else if findings
            .iter()
            .any(|finding| finding.severity() == Severity::Warning)
        {
            Some(CheckOutcome::Warning)
        } else {
            Some(CheckOutcome::Passed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CheckId, CheckOutcome, CheckResult, Finding, FindingCode, Location, LocationField,
        Requirement, Severity, SkipReason,
    };
    use crate::contract::protocol::SupportedRevision;

    #[test]
    fn finding_codes_own_their_stable_severity_policy() {
        let cases = [
            (
                FindingCode::ProcessStartFailed,
                "MCP-TRANSPORT-001",
                Severity::Error,
            ),
            (
                FindingCode::StdioIoFailed,
                "MCP-TRANSPORT-002",
                Severity::Error,
            ),
            (
                FindingCode::InvalidStdioMessage,
                "MCP-TRANSPORT-003",
                Severity::Error,
            ),
            (
                FindingCode::ServerExitedEarly,
                "MCP-TRANSPORT-004",
                Severity::Error,
            ),
            (
                FindingCode::ProtocolRevisionConfirmed,
                "MCP-PROTOCOL-001",
                Severity::Info,
            ),
            (
                FindingCode::UnsupportedProtocolRevision,
                "MCP-PROTOCOL-002",
                Severity::Error,
            ),
            (
                FindingCode::InvalidProtocolRevisionValue,
                "MCP-PROTOCOL-003",
                Severity::Error,
            ),
            (
                FindingCode::DeprecatedProtocolFeature,
                "MCP-PROTOCOL-004",
                Severity::Warning,
            ),
            (FindingCode::LimitExceeded, "MCP-LIMIT-001", Severity::Error),
            (
                FindingCode::CleanupFailed,
                "MCP-SAFETY-001",
                Severity::Critical,
            ),
            (
                FindingCode::CatalogContractInvalid,
                "MCP-CATALOG-001",
                Severity::Error,
            ),
            (
                FindingCode::DuplicateCatalogIdentifier,
                "MCP-CATALOG-002",
                Severity::Error,
            ),
            (
                FindingCode::PaginationCursorRepeated,
                "MCP-CATALOG-003",
                Severity::Error,
            ),
            (
                FindingCode::SchemaContractInvalid,
                "MCP-SCHEMA-001",
                Severity::Error,
            ),
            (
                FindingCode::UnsupportedSchemaDialect,
                "MCP-SCHEMA-002",
                Severity::Error,
            ),
            (
                FindingCode::ExternalSchemaReferenceBlocked,
                "MCP-SCHEMA-003",
                Severity::Error,
            ),
        ];

        for (code, stable_code, severity) in cases {
            assert_eq!(code.as_str(), stable_code);
            assert_eq!(code.severity(), severity);
            assert!(!code.title().is_empty());
            assert!(!code.impact().is_empty());
            assert!(!code.expectation().is_empty());
            assert!(!code.remediation().is_empty());
            assert!(!code.reference().is_empty());
        }
    }

    #[test]
    fn check_ids_and_skip_reasons_have_stable_report_values() {
        let check_ids = [
            (CheckId::TransportStdio, "transport.stdio"),
            (CheckId::ProtocolEnvelope, "protocol.envelope"),
            (CheckId::ProtocolRevision, "protocol.revision"),
            (CheckId::DiscoveryCatalogs, "discovery.catalogs"),
            (CheckId::SchemaContracts, "schema.contracts"),
            (CheckId::RuntimeTools, "runtime.tools"),
        ];
        let skip_reasons = [
            (SkipReason::NotAuthorized, "not_authorized"),
            (SkipReason::NotAdvertised, "not_advertised"),
            (SkipReason::UnsupportedRevision, "unsupported_revision"),
            (SkipReason::PrerequisiteFailed, "prerequisite_failed"),
            (SkipReason::LimitReached, "limit_reached"),
            (SkipReason::NotApplicable, "not_applicable"),
        ];

        for (check, expected) in check_ids {
            assert_eq!(check.as_str(), expected);
        }
        for (reason, expected) in skip_reasons {
            assert_eq!(reason.as_str(), expected);
            assert!(!reason.description().is_empty());
        }
        assert!(SkipReason::PrerequisiteFailed.is_causal());
        assert!(SkipReason::UnsupportedRevision.is_causal());
        assert!(SkipReason::LimitReached.is_causal());
        assert!(!SkipReason::NotAuthorized.is_causal());
    }

    #[test]
    fn locations_contain_only_trusted_structure() {
        let location = Location::root(LocationField::Tools)
            .index(3)
            .field(LocationField::InputSchema)
            .field(LocationField::Required);

        assert_eq!(location.to_string(), "tools[3].inputSchema.required");
    }

    #[test]
    fn performed_warning_and_skipped_checks_remain_distinct() {
        let warning = Finding::deprecated_protocol_feature(
            SupportedRevision::CURRENT,
            Location::root(LocationField::Server),
        );
        let performed = CheckResult::performed(
            CheckId::DiscoveryCatalogs,
            Requirement::Required,
            vec![warning],
        );
        let skipped = CheckResult::skipped(
            CheckId::RuntimeTools,
            Requirement::Optional,
            SkipReason::NotAuthorized,
        );

        assert_eq!(performed.outcome(), Some(CheckOutcome::Warning));
        assert!(performed.skip_reason().is_none());
        assert!(skipped.outcome().is_none());
        assert_eq!(skipped.skip_reason(), Some(SkipReason::NotAuthorized));
    }
}
