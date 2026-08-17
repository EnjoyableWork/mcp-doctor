use std::fmt;

use super::limits::LimitViolation;
use super::protocol::{RevisionAdvertisementSummary, SupportedRevision};
use super::redaction::RedactedValue;

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum CheckId {
    ScenarioConfiguration,
    GenerationConfiguration,
    ActiveAuthorization,
    NetworkTarget,
    NetworkResolution,
    TransportTls,
    TransportHttp,
    TransportStdio,
    ProtocolEnvelope,
    ProtocolRevision,
    DiscoveryCatalogs,
    SchemaContracts,
    CaseGeneration,
    RuntimeTools,
    RuntimeToolCase(usize),
    RuntimeWorkflowStep(usize),
    RuntimeWorkflowCleanup(usize),
}

impl CheckId {
    pub(super) fn as_str(self) -> String {
        match self {
            Self::ScenarioConfiguration => "scenario.configuration".to_owned(),
            Self::GenerationConfiguration => "generation.configuration".to_owned(),
            Self::ActiveAuthorization => "authorization.active".to_owned(),
            Self::NetworkTarget => "network.target".to_owned(),
            Self::NetworkResolution => "network.resolution".to_owned(),
            Self::TransportTls => "transport.tls".to_owned(),
            Self::TransportHttp => "transport.http".to_owned(),
            Self::TransportStdio => "transport.stdio".to_owned(),
            Self::ProtocolRevision => "protocol.revision".to_owned(),
            Self::ProtocolEnvelope => "protocol.envelope".to_owned(),
            Self::DiscoveryCatalogs => "discovery.catalogs".to_owned(),
            Self::SchemaContracts => "schema.contracts".to_owned(),
            Self::CaseGeneration => "generation.cases".to_owned(),
            Self::RuntimeTools => "runtime.tools".to_owned(),
            Self::RuntimeToolCase(index) => format!("runtime.tools.case[{index}]"),
            Self::RuntimeWorkflowStep(index) => format!("runtime.workflow.step[{index}]"),
            Self::RuntimeWorkflowCleanup(index) => {
                format!("runtime.workflow.cleanup[{index}]")
            }
        }
    }
}

impl fmt::Display for CheckId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.as_str())
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
    AuthorizationFailed,
    NotAdvertised,
    InputRequired,
    UnsupportedRevision,
    PrerequisiteFailed,
    LimitReached,
    NotApplicable,
}

impl SkipReason {
    pub(super) const fn as_str(self) -> &'static str {
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

    pub(super) const fn description(self) -> &'static str {
        match self {
            Self::NotAuthorized => "active behavior was not authorized",
            Self::AuthorizationFailed => "an explicit active authorization gate failed",
            Self::NotAdvertised => "the capability was not advertised",
            Self::InputRequired => {
                "the server requires input that mcp-doctor is not authorized to provide"
            }
            Self::UnsupportedRevision => "the protocol revision is unsupported",
            Self::PrerequisiteFailed => "a required prerequisite did not pass",
            Self::LimitReached => "a safety limit prevented the check",
            Self::NotApplicable => "the check does not apply to this target",
        }
    }

    pub(super) const fn is_causal(self) -> bool {
        matches!(
            self,
            Self::AuthorizationFailed
                | Self::UnsupportedRevision
                | Self::PrerequisiteFailed
                | Self::LimitReached
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
    RemoteTargetInvalid,
    NetworkAuthorizationMissing,
    ResolutionFailed,
    AddressPolicyBlocked,
    PeerAddressMismatch,
    TlsVerificationFailed,
    HttpExchangeFailed,
    HttpResponseInvalid,
    RemoteAuthenticationRejected,
    HttpHeaderMappingInvalid,
    ProtocolRevisionConfirmed,
    UnsupportedProtocolRevision,
    InvalidProtocolRevisionValue,
    ProtocolRevisionMismatch,
    DeprecatedProtocolFeature,
    LimitExceeded,
    CleanupFailed,
    SessionCleanupFailed,
    CatalogContractInvalid,
    DuplicateCatalogIdentifier,
    PaginationCursorRepeated,
    SchemaContractInvalid,
    UnsupportedSchemaDialect,
    AmbiguousSchemaDialect,
    ExternalSchemaReferenceBlocked,
    ScenarioInvalid,
    SecretReferenceInvalid,
    ScenarioSchemaInvalid,
    CaseGenerationFailed,
    ToolAuthorizationMissing,
    SideEffectsNotAuthorized,
    ToolNotFound,
    ToolArgumentsMismatch,
    ToolCallRejected,
    ToolResultMismatch,
    ToolOutputMismatch,
    ToolResultInvalid,
    ToolTaskRequired,
    SchemaInvalidArgumentsAccepted,
    WorkflowCaptureMissing,
    WorkflowCleanupFailed,
}

impl FindingCode {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ProcessStartFailed => "MCP-TRANSPORT-001",
            Self::StdioIoFailed => "MCP-TRANSPORT-002",
            Self::InvalidStdioMessage => "MCP-TRANSPORT-003",
            Self::ServerExitedEarly => "MCP-TRANSPORT-004",
            Self::RemoteTargetInvalid => "MCP-TARGET-001",
            Self::NetworkAuthorizationMissing => "MCP-TARGET-002",
            Self::ResolutionFailed => "MCP-NETWORK-001",
            Self::AddressPolicyBlocked => "MCP-NETWORK-002",
            Self::PeerAddressMismatch => "MCP-NETWORK-003",
            Self::TlsVerificationFailed => "MCP-TLS-001",
            Self::HttpExchangeFailed => "MCP-HTTP-001",
            Self::HttpResponseInvalid => "MCP-HTTP-002",
            Self::RemoteAuthenticationRejected => "MCP-HTTP-AUTH-001",
            Self::HttpHeaderMappingInvalid => "MCP-HTTP-HEADER-001",
            Self::ProtocolRevisionConfirmed => "MCP-PROTOCOL-001",
            Self::UnsupportedProtocolRevision => "MCP-PROTOCOL-002",
            Self::InvalidProtocolRevisionValue => "MCP-PROTOCOL-003",
            Self::DeprecatedProtocolFeature => "MCP-PROTOCOL-004",
            Self::ProtocolRevisionMismatch => "MCP-PROTOCOL-005",
            Self::LimitExceeded => "MCP-LIMIT-001",
            Self::CleanupFailed => "MCP-SAFETY-001",
            Self::SessionCleanupFailed => "MCP-SAFETY-002",
            Self::CatalogContractInvalid => "MCP-CATALOG-001",
            Self::DuplicateCatalogIdentifier => "MCP-CATALOG-002",
            Self::PaginationCursorRepeated => "MCP-CATALOG-003",
            Self::SchemaContractInvalid => "MCP-SCHEMA-001",
            Self::UnsupportedSchemaDialect => "MCP-SCHEMA-002",
            Self::ExternalSchemaReferenceBlocked => "MCP-SCHEMA-003",
            Self::AmbiguousSchemaDialect => "MCP-SCHEMA-004",
            Self::ScenarioInvalid => "MCP-SCENARIO-001",
            Self::SecretReferenceInvalid => "MCP-SCENARIO-002",
            Self::ScenarioSchemaInvalid => "MCP-SCENARIO-003",
            Self::CaseGenerationFailed => "MCP-GENERATION-001",
            Self::ToolAuthorizationMissing => "MCP-AUTH-001",
            Self::SideEffectsNotAuthorized => "MCP-AUTH-002",
            Self::ToolNotFound => "MCP-ACTIVE-001",
            Self::ToolArgumentsMismatch => "MCP-ACTIVE-002",
            Self::ToolCallRejected => "MCP-ACTIVE-003",
            Self::ToolResultMismatch => "MCP-ACTIVE-004",
            Self::ToolOutputMismatch => "MCP-ACTIVE-005",
            Self::ToolResultInvalid => "MCP-ACTIVE-006",
            Self::ToolTaskRequired => "MCP-ACTIVE-007",
            Self::SchemaInvalidArgumentsAccepted => "MCP-ACTIVE-008",
            Self::WorkflowCaptureMissing => "MCP-WORKFLOW-001",
            Self::WorkflowCleanupFailed => "MCP-SAFETY-003",
        }
    }

    pub(super) const fn severity(self) -> Severity {
        match self {
            Self::ProtocolRevisionConfirmed => Severity::Info,
            Self::DeprecatedProtocolFeature | Self::AmbiguousSchemaDialect => Severity::Warning,
            Self::ProcessStartFailed
            | Self::StdioIoFailed
            | Self::InvalidStdioMessage
            | Self::ServerExitedEarly
            | Self::RemoteTargetInvalid
            | Self::NetworkAuthorizationMissing
            | Self::ResolutionFailed
            | Self::AddressPolicyBlocked
            | Self::PeerAddressMismatch
            | Self::TlsVerificationFailed
            | Self::HttpExchangeFailed
            | Self::HttpResponseInvalid
            | Self::RemoteAuthenticationRejected
            | Self::HttpHeaderMappingInvalid
            | Self::UnsupportedProtocolRevision
            | Self::InvalidProtocolRevisionValue
            | Self::ProtocolRevisionMismatch
            | Self::LimitExceeded
            | Self::CatalogContractInvalid
            | Self::DuplicateCatalogIdentifier
            | Self::PaginationCursorRepeated
            | Self::SchemaContractInvalid
            | Self::UnsupportedSchemaDialect
            | Self::ExternalSchemaReferenceBlocked
            | Self::ScenarioInvalid
            | Self::SecretReferenceInvalid
            | Self::ScenarioSchemaInvalid
            | Self::CaseGenerationFailed
            | Self::ToolAuthorizationMissing
            | Self::SideEffectsNotAuthorized
            | Self::ToolNotFound
            | Self::ToolArgumentsMismatch
            | Self::ToolCallRejected
            | Self::ToolResultMismatch
            | Self::ToolOutputMismatch
            | Self::ToolResultInvalid
            | Self::ToolTaskRequired
            | Self::WorkflowCaptureMissing => Severity::Error,
            Self::CleanupFailed
            | Self::SessionCleanupFailed
            | Self::SchemaInvalidArgumentsAccepted
            | Self::WorkflowCleanupFailed => Severity::Critical,
        }
    }

    pub(super) const fn title(self) -> &'static str {
        match self {
            Self::ProcessStartFailed => "The MCP server process could not be started.",
            Self::StdioIoFailed => "The STDIO channel failed before diagnosis completed.",
            Self::InvalidStdioMessage => "The server wrote an invalid STDIO message.",
            Self::ServerExitedEarly => "The server process exited before returning a response.",
            Self::RemoteTargetInvalid => "The remote MCP endpoint is not safe to use.",
            Self::NetworkAuthorizationMissing => {
                "The invocation does not authorize this remote network activity."
            }
            Self::ResolutionFailed => "The remote endpoint could not be resolved safely.",
            Self::AddressPolicyBlocked => {
                "A resolved address is outside the permitted destination classes."
            }
            Self::PeerAddressMismatch => {
                "The connected peer is outside the one validated pinned address set."
            }
            Self::TlsVerificationFailed => "The remote endpoint did not complete verified TLS.",
            Self::HttpExchangeFailed => "The bounded HTTP exchange did not complete.",
            Self::HttpResponseInvalid => {
                "The HTTP response does not match the bounded Streamable HTTP contract."
            }
            Self::RemoteAuthenticationRejected => {
                "The remote endpoint rejected the pre-provisioned authentication."
            }
            Self::HttpHeaderMappingInvalid => {
                "A remote request header mapping is invalid or unsafe."
            }
            Self::ProtocolRevisionConfirmed => "The requested protocol revision is supported.",
            Self::UnsupportedProtocolRevision => {
                "The server does not support the required protocol revision."
            }
            Self::InvalidProtocolRevisionValue => {
                "The protocol revision value is missing or has the wrong JSON type."
            }
            Self::ProtocolRevisionMismatch => {
                "The server negotiated a different protocol revision than the explicit selection."
            }
            Self::DeprecatedProtocolFeature => {
                "The server advertises a feature deprecated by this protocol revision."
            }
            Self::LimitExceeded => "A configured diagnostic safety limit was exceeded.",
            Self::CleanupFailed => "The managed target could not be fully cleaned up.",
            Self::SessionCleanupFailed => {
                "The remote MCP session could not be terminated within its cleanup bound."
            }
            Self::CatalogContractInvalid => {
                "An advertised MCP catalog does not match its protocol contract."
            }
            Self::DuplicateCatalogIdentifier => {
                "An advertised catalog contains a duplicate identifier."
            }
            Self::PaginationCursorRepeated => {
                "A catalog repeated a pagination cursor and inspection stopped."
            }
            Self::SchemaContractInvalid => "A local JSON Schema contract is invalid.",
            Self::UnsupportedSchemaDialect => {
                "A local schema uses an unsupported JSON Schema dialect."
            }
            Self::AmbiguousSchemaDialect => {
                "The selected revision does not define a default JSON Schema dialect."
            }
            Self::ExternalSchemaReferenceBlocked => {
                "A schema requires prohibited external reference retrieval."
            }
            Self::ScenarioInvalid => "The check scenario does not match its versioned contract.",
            Self::SecretReferenceInvalid => {
                "A scenario environment reference could not be resolved safely."
            }
            Self::ScenarioSchemaInvalid => {
                "A scenario-provided output schema is not a valid bounded local contract."
            }
            Self::CaseGenerationFailed => {
                "Bounded boundary inputs could not be generated from the selected tool schema."
            }
            Self::ToolAuthorizationMissing => {
                "The invocation does not authorize the selected exact tool."
            }
            Self::SideEffectsNotAuthorized => {
                "The invocation does not authorize this side-effecting active run."
            }
            Self::ToolNotFound => "The exactly authorized tool was not advertised uniquely.",
            Self::ToolArgumentsMismatch => {
                "The active case arguments do not match the advertised input schema."
            }
            Self::ToolCallRejected => "The server rejected the active tool request.",
            Self::ToolResultMismatch => {
                "The completed tool result does not match the active case expectation."
            }
            Self::ToolOutputMismatch => {
                "The structured tool output does not match its required local schema contract."
            }
            Self::ToolResultInvalid => {
                "The tool response does not match the selected MCP revision's result contract."
            }
            Self::ToolTaskRequired => {
                "The selected tool requires task execution that this active run does not perform."
            }
            Self::SchemaInvalidArgumentsAccepted => {
                "The server accepted schema-invalid tool arguments instead of rejecting them."
            }
            Self::WorkflowCaptureMissing => {
                "A workflow step did not produce one declared structural capture."
            }
            Self::WorkflowCleanupFailed => {
                "An explicitly declared workflow cleanup step did not complete successfully."
            }
        }
    }

    pub(super) const fn impact(self) -> &'static str {
        match self {
            Self::ProcessStartFailed => {
                "No protocol or contract check can run until the target starts."
            }
            Self::StdioIoFailed => {
                "A broken channel prevents the diagnosis from completing reliably."
            }
            Self::InvalidStdioMessage => {
                "MCP clients cannot interpret this output as a JSON-RPC response."
            }
            Self::ServerExitedEarly => {
                "The pending request cannot complete after the server exits."
            }
            Self::RemoteTargetInvalid
            | Self::NetworkAuthorizationMissing
            | Self::ResolutionFailed
            | Self::AddressPolicyBlocked
            | Self::PeerAddressMismatch => {
                "Continuing could connect to a destination the invocation did not safely authorize."
            }
            Self::TlsVerificationFailed => {
                "The server identity and protected channel cannot be trusted."
            }
            Self::HttpExchangeFailed | Self::HttpResponseInvalid => {
                "No reliable MCP response can be diagnosed from this exchange."
            }
            Self::RemoteAuthenticationRejected => {
                "The requested remote operation cannot proceed with the supplied credentials."
            }
            Self::HttpHeaderMappingInvalid => {
                "Sending the mapping could create ambiguous routing or expose an unbounded value."
            }
            Self::ProtocolRevisionConfirmed => {
                "The advertised revision determines which protocol rules can be applied."
            }
            Self::UnsupportedProtocolRevision => {
                "Applying rules for a different revision could produce a false diagnosis."
            }
            Self::InvalidProtocolRevisionValue => {
                "A valid protocol rule set cannot be selected from this advertisement."
            }
            Self::ProtocolRevisionMismatch => {
                "Continuing would silently downgrade, upgrade, or misapply protocol rules."
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
            Self::SessionCleanupFailed => {
                "A retained remote session can keep server-side state or resources alive after inspection."
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
            Self::AmbiguousSchemaDialect => {
                "Dialect-specific keywords cannot be validated without guessing."
            }
            Self::ExternalSchemaReferenceBlocked => {
                "Resolving this contract would require network or file access that was not authorized."
            }
            Self::ScenarioInvalid => {
                "The declared cases cannot be replayed deterministically or within their safety boundary."
            }
            Self::SecretReferenceInvalid => {
                "Starting the target could omit, misplace, or disclose a required secret."
            }
            Self::ScenarioSchemaInvalid => {
                "The expected structured output cannot be checked locally and deterministically."
            }
            Self::CaseGenerationFailed => {
                "No generated call is safe until a schema-valid input can be constructed within every generation and active-input bound."
            }
            Self::ToolAuthorizationMissing | Self::SideEffectsNotAuthorized => {
                "Calling the tool without every redundant authorization gate could cause unexpected activity."
            }
            Self::ToolNotFound => {
                "mcp-doctor cannot safely choose the one tool authorized by the scenario and invocation."
            }
            Self::ToolArgumentsMismatch => {
                "Sending invalid arguments would turn a local scenario defect into target activity."
            }
            Self::ToolCallRejected => {
                "The active case did not produce a completed tool result to validate."
            }
            Self::ToolResultMismatch => {
                "The tool behaved differently from the case's declared or generated expectation."
            }
            Self::ToolOutputMismatch => {
                "Consumers cannot rely on the structured result shape promised for this case."
            }
            Self::ToolResultInvalid => {
                "Continuing after an invalid result envelope could make later conclusions unreliable."
            }
            Self::ToolTaskRequired => {
                "Calling this tool immediately would violate its advertised execution contract."
            }
            Self::SchemaInvalidArgumentsAccepted => {
                "The defective server may have executed a call that its advertised input schema forbids."
            }
            Self::WorkflowCaptureMissing => {
                "A later exact step cannot receive its reviewed argument without guessing or reflecting the result."
            }
            Self::WorkflowCleanupFailed => {
                "State or resources created by the reviewed workflow may remain after diagnosis."
            }
        }
    }

    pub(super) const fn expectation(self) -> &'static str {
        match self {
            Self::ProcessStartFailed => "The target must be a directly executable local program.",
            Self::StdioIoFailed => {
                "The target must keep its STDIO pipes available through the diagnosis."
            }
            Self::InvalidStdioMessage => {
                "Each STDOUT frame must be one valid JSON-RPC 2.0 message terminated by a newline."
            }
            Self::ServerExitedEarly => {
                "The server must remain alive until it returns the pending passive response."
            }
            Self::RemoteTargetInvalid => {
                "Use one strict absolute HTTPS endpoint, or an exactly gated loopback HTTP endpoint."
            }
            Self::NetworkAuthorizationMissing => {
                "Private, cleartext, and credential use each require their exact matching endpoint gate."
            }
            Self::ResolutionFailed => {
                "The endpoint must resolve once to at most 16 unique addresses within the startup bound."
            }
            Self::AddressPolicyBlocked => {
                "Every answer must be public or belong to one consistently and exactly authorized private class."
            }
            Self::PeerAddressMismatch => {
                "Every connection peer must belong to the sorted address set validated for this run."
            }
            Self::TlsVerificationFailed => {
                "HTTPS must verify TLS 1.2 or 1.3, the complete chain, validity, and canonical service identity."
            }
            Self::HttpExchangeFailed => {
                "Each planned MCP operation must complete once, directly, within its request and response deadlines."
            }
            Self::HttpResponseInvalid => {
                "The endpoint must follow the selected revision's bounded JSON, request-scoped SSE, header, and session contract."
            }
            Self::RemoteAuthenticationRejected => {
                "A pre-provisioned credential must be accepted without redirect, replay, or automatic OAuth discovery."
            }
            Self::HttpHeaderMappingInvalid => {
                "Custom and Mcp-Param fields must satisfy the current-revision token, type, encoding, uniqueness, and size rules."
            }
            Self::ProtocolRevisionConfirmed => {
                "The server confirms the exact MCP revision selected for this diagnostic."
            }
            Self::UnsupportedProtocolRevision => {
                "The server must support MCP protocol revision 2026-07-28 for this diagnosis."
            }
            Self::InvalidProtocolRevisionValue => {
                "The selected lifecycle's revision field must have its required string or string-array shape."
            }
            Self::ProtocolRevisionMismatch => {
                "InitializeResult.protocolVersion must exactly equal the explicitly selected revision."
            }
            Self::DeprecatedProtocolFeature => {
                "Servers should avoid features deprecated by MCP 2026-07-28."
            }
            Self::LimitExceeded => {
                "Every diagnostic path must remain within its reported safety limit."
            }
            Self::CleanupFailed => {
                "The managed process tree must terminate and be reaped before mcp-doctor returns."
            }
            Self::SessionCleanupFailed => {
                "A stateful legacy HTTP diagnostic must attempt one bounded DELETE and receive a successful, unsupported, or already-absent response."
            }
            Self::CatalogContractInvalid => {
                "Each advertised catalog response and item must match the selected MCP revision."
            }
            Self::DuplicateCatalogIdentifier => {
                "Identifiers must be unique within their advertised catalog scope."
            }
            Self::PaginationCursorRepeated => {
                "Each nextCursor must advance the catalog or end pagination."
            }
            Self::SchemaContractInvalid => {
                "Advertised and scenario-provided schemas must be valid local JSON Schema Draft 2020-12 objects whose patterns use the supported linear-time subset."
            }
            Self::UnsupportedSchemaDialect => {
                "Local schemas must resolve to JSON Schema Draft 2020-12 through the selected revision's default or an exact $schema declaration."
            }
            Self::AmbiguousSchemaDialect => {
                "MCP 2025-06-18 schemas without $schema receive bounded structural checks only; no dialect-specific semantics are assumed."
            }
            Self::ExternalSchemaReferenceBlocked => {
                "mcp-doctor accepts only references contained in the local schema being checked."
            }
            Self::ScenarioInvalid => {
                "The file must be one strict supported mcp-doctor scenario JSON document within its finite case or step bounds."
            }
            Self::SecretReferenceInvalid => {
                "Every reference must name an existing invoking-process environment value and every argument pointer must target an existing null placeholder."
            }
            Self::ScenarioSchemaInvalid => {
                "Scenario output schemas must be bounded local JSON Schema Draft 2020-12 objects whose patterns use the supported linear-time subset."
            }
            Self::CaseGenerationFailed => {
                "The selected input schema must admit at least one bounded object that mcp-doctor.generator/v1 can reproduce."
            }
            Self::ToolAuthorizationMissing => {
                "--allow-tool must match the scenario or generated selection and discovered tool byte for byte."
            }
            Self::SideEffectsNotAuthorized => {
                "A side_effecting active run also requires --allow-side-effects."
            }
            Self::ToolNotFound => {
                "The server must advertise exactly one tool matching the authorized selection."
            }
            Self::ToolArgumentsMismatch => {
                "Each case must pass the selected tool's advertised input schema before it is called."
            }
            Self::ToolCallRejected => {
                "The server must return a valid selected-revision tools/call result or supported incomplete-input signal."
            }
            Self::ToolResultMismatch => {
                "The isError classification must match the case's success or tool_error expectation."
            }
            Self::ToolOutputMismatch => {
                "structuredContent must match the advertised output schema and the scenario schema when present."
            }
            Self::ToolResultInvalid => {
                "A tools/call response must match the result envelope defined by the selected revision."
            }
            Self::ToolTaskRequired => {
                "Active calls must use immediate execution; a tool requiring task augmentation is not called."
            }
            Self::SchemaInvalidArgumentsAccepted => {
                "MCP 2026-07-28 schema-invalid tool arguments must receive a matching JSON-RPC -32602 error before execution."
            }
            Self::WorkflowCaptureMissing => {
                "Every declared capture pointer must resolve within a successful, validated structuredContent object."
            }
            Self::WorkflowCleanupFailed => {
                "Every declared cleanup step must return its expected successful result within the remaining safety bounds."
            }
        }
    }

    pub(super) const fn remediation(self) -> &'static str {
        match self {
            Self::ProcessStartFailed => {
                "Check the executable path and permissions, then rerun the same command."
            }
            Self::StdioIoFailed => "Fix the server's STDIO lifecycle and rerun the same command.",
            Self::InvalidStdioMessage => {
                "Write only newline-delimited JSON-RPC messages to STDOUT; send logs to STDERR."
            }
            Self::ServerExitedEarly => "Keep the server alive long enough to answer the request.",
            Self::RemoteTargetInvalid => {
                "Correct the endpoint structure and rerun with the same intended destination."
            }
            Self::NetworkAuthorizationMissing => {
                "Review the destination and add only the required exact endpoint gate."
            }
            Self::ResolutionFailed => {
                "Correct DNS or reduce the answer set, then rerun without widening destination authority."
            }
            Self::AddressPolicyBlocked => {
                "Use an eligible destination; prohibited special-purpose addresses cannot be authorized."
            }
            Self::PeerAddressMismatch => {
                "Correct the resolver or route so the peer matches the validated pinned set."
            }
            Self::TlsVerificationFailed => {
                "Correct the certificate chain and identity, or provide the intended CA with --tls-ca-file."
            }
            Self::HttpExchangeFailed => {
                "Correct the direct endpoint or server lifecycle, then rerun the same single operation."
            }
            Self::HttpResponseInvalid => {
                "Return the selected revision's bounded identity-encoded JSON, request-scoped SSE, header, and session behavior."
            }
            Self::RemoteAuthenticationRejected => {
                "Provision the intended credential and rerun; mcp-doctor will not start an OAuth flow."
            }
            Self::HttpHeaderMappingInvalid => {
                "Correct or remove the unsafe header mapping before replaying the tool."
            }
            Self::ProtocolRevisionConfirmed => "No correction is needed.",
            Self::UnsupportedProtocolRevision => {
                "Add MCP 2026-07-28 support, then rerun the same diagnosis without falling back."
            }
            Self::InvalidProtocolRevisionValue => {
                "Correct the revision field at the reported structural location and rerun the same explicit selection."
            }
            Self::ProtocolRevisionMismatch => {
                "Return the selected revision exactly, or rerun with an explicit supported selection; mcp-doctor will not fall back."
            }
            Self::DeprecatedProtocolFeature => "Remove or replace the deprecated capability.",
            Self::LimitExceeded => {
                "Reduce the reported data or work below the maximum, then rerun the same command."
            }
            Self::CleanupFailed => {
                "Make the server and descendants exit when STDIN closes or termination is requested."
            }
            Self::SessionCleanupFailed => {
                "Make session DELETE complete within the bound, or return 405 when termination is unsupported."
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
                "Correct the schema at the reported structural location and validate it as Draft 2020-12; use the supported linear-time subset for patterns."
            }
            Self::UnsupportedSchemaDialect => {
                "For MCP 2025-06-18, declare the Draft 2020-12 URI; otherwise remove an unsupported declaration only when the selected revision defines that default, or declare Draft 2020-12 explicitly."
            }
            Self::AmbiguousSchemaDialect => {
                "Declare the exact Draft 2020-12 $schema URI to enable full local semantic validation."
            }
            Self::ExternalSchemaReferenceBlocked => {
                "Inline the referenced schema or move it into local $defs and use a fragment reference."
            }
            Self::ScenarioInvalid => "Correct the reported scenario structure and rerun check.",
            Self::SecretReferenceInvalid => {
                "Correct the environment reference or null placeholder, provide the value, and rerun check."
            }
            Self::ScenarioSchemaInvalid => {
                "Correct or bound the local output schema, including any unsupported pattern, then rerun check."
            }
            Self::CaseGenerationFailed => {
                "Expose a bounded object schema with usable const, enum, default, or structural boundaries, or replay reviewed arguments with check."
            }
            Self::ToolAuthorizationMissing => {
                "Pass the selected exact tool name independently through --allow-tool."
            }
            Self::SideEffectsNotAuthorized => {
                "Use a disposable target and add --allow-side-effects only after reviewing the exact tool, seed, and case bound."
            }
            Self::ToolNotFound => {
                "Advertise one exact matching tool or correct the selection and authorization together."
            }
            Self::ToolArgumentsMismatch => {
                "Correct the case arguments or the advertised input schema; the case was not called."
            }
            Self::ToolCallRejected => {
                "Correct the server-side rejection and rerun the same case seed or reviewed case."
            }
            Self::ToolResultMismatch => {
                "Correct the tool behavior, then rerun the same generated seed or reviewed scenario."
            }
            Self::ToolOutputMismatch => {
                "Correct structuredContent or the applicable local output schema, then rerun the active command."
            }
            Self::ToolResultInvalid => {
                "Return a valid result for the selected MCP revision before running later active cases."
            }
            Self::ToolTaskRequired => {
                "Advertise optional, forbidden, or omitted task support for this run, or use a task-capable client."
            }
            Self::SchemaInvalidArgumentsAccepted => {
                "Validate arguments against the advertised input schema before invoking tool logic, return JSON-RPC -32602, and rerun the same seed."
            }
            Self::WorkflowCaptureMissing => {
                "Correct the producing tool or capture pointer, then rerun the same reviewed workflow."
            }
            Self::WorkflowCleanupFailed => {
                "Inspect the disposable target, correct the cleanup tool behavior, and rerun the same reviewed workflow."
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
            | Self::CleanupFailed => "mcp-doctor bounded local STDIO safety contract",
            Self::SessionCleanupFailed => {
                "selected MCP revision Streamable HTTP session lifecycle and mcp-doctor DEC-044"
            }
            Self::RemoteTargetInvalid
            | Self::NetworkAuthorizationMissing
            | Self::ResolutionFailed
            | Self::AddressPolicyBlocked
            | Self::PeerAddressMismatch
            | Self::TlsVerificationFailed
            | Self::HttpExchangeFailed
            | Self::HttpResponseInvalid
            | Self::RemoteAuthenticationRejected => {
                "selected MCP revision Streamable HTTP and mcp-doctor DEC-030/DEC-044"
            }
            Self::HttpHeaderMappingInvalid => {
                "MCP 2026-07-28 Streamable HTTP header mapping and mcp-doctor DEC-030"
            }
            Self::ProtocolRevisionConfirmed
            | Self::UnsupportedProtocolRevision
            | Self::InvalidProtocolRevisionValue
            | Self::ProtocolRevisionMismatch
            | Self::DeprecatedProtocolFeature => "selected MCP revision lifecycle contract",
            Self::CatalogContractInvalid
            | Self::DuplicateCatalogIdentifier
            | Self::PaginationCursorRepeated => "selected MCP revision catalog contracts",
            Self::SchemaContractInvalid
            | Self::UnsupportedSchemaDialect
            | Self::ExternalSchemaReferenceBlocked => {
                "selected MCP revision Tool contract and JSON Schema Draft 2020-12"
            }
            Self::AmbiguousSchemaDialect => {
                "MCP 2025-06-18 Tool schema contract and mcp-doctor DEC-044"
            }
            Self::ScenarioInvalid | Self::SecretReferenceInvalid | Self::ScenarioSchemaInvalid => {
                "selected mcp-doctor versioned scenario contract"
            }
            Self::CaseGenerationFailed => "mcp-doctor MCPD-011 bounded generation contract",
            Self::ToolAuthorizationMissing | Self::SideEffectsNotAuthorized => {
                "mcp-doctor MCPD-009 and MCPD-011 active-authorization contract"
            }
            Self::ToolNotFound
            | Self::ToolArgumentsMismatch
            | Self::ToolCallRejected
            | Self::ToolResultMismatch
            | Self::ToolOutputMismatch
            | Self::ToolResultInvalid
            | Self::ToolTaskRequired
            | Self::SchemaInvalidArgumentsAccepted => {
                "selected MCP revision tools contract and mcp-doctor MCPD-009/MCPD-011/MCPD-027/MCPD-029 active contract"
            }
            Self::WorkflowCaptureMissing | Self::WorkflowCleanupFailed => {
                "mcp-doctor DEC-056 bounded workflow contract"
            }
        }
    }

    pub(super) const fn is_independent_safety(self) -> bool {
        matches!(
            self,
            Self::CleanupFailed | Self::SessionCleanupFailed | Self::WorkflowCleanupFailed
        )
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum LocationField {
    Scenario,
    Generation,
    SchemaVersion,
    Authorization,
    Safety,
    Effects,
    TargetEnv,
    Cases,
    Steps,
    Id,
    SecretRefs,
    ArgumentRefs,
    Captures,
    Cleanup,
    Expect,
    StructuredOutputSchema,
    Request,
    Meta,
    ProtocolVersion,
    NegotiatedProtocolVersion,
    Server,
    ServerInfo,
    Version,
    Instructions,
    SupportedVersions,
    Tools,
    Prompts,
    Resources,
    ResourceTemplates,
    Capabilities,
    Logging,
    Completions,
    Experimental,
    Tasks,
    List,
    Cancel,
    Requests,
    Call,
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
    Execution,
    TaskSupport,
    Content,
    StructuredContent,
    IsError,
    Schema,
    Vocabulary,
    Type,
    Pattern,
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
    Endpoint,
    Resolution,
    Address,
    Peer,
    Trust,
    Tls,
    Http,
    Headers,
    Status,
    Body,
    Event,
    Credentials,
    Session,
}

impl LocationField {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Scenario => "scenario",
            Self::Generation => "generation",
            Self::SchemaVersion => "schema_version",
            Self::Authorization => "authorization",
            Self::Safety => "safety",
            Self::Effects => "effects",
            Self::TargetEnv => "target_env",
            Self::Cases => "cases",
            Self::Steps => "steps",
            Self::Id => "id",
            Self::SecretRefs => "secret_refs",
            Self::ArgumentRefs => "argument_refs",
            Self::Captures => "captures",
            Self::Cleanup => "cleanup",
            Self::Expect => "expect",
            Self::StructuredOutputSchema => "structured_output_schema",
            Self::Request => "request",
            Self::Meta => "_meta",
            Self::ProtocolVersion => "io.modelcontextprotocol/protocolVersion",
            Self::NegotiatedProtocolVersion => "protocolVersion",
            Self::Server => "server",
            Self::ServerInfo => "serverInfo",
            Self::Version => "version",
            Self::Instructions => "instructions",
            Self::SupportedVersions => "supportedVersions",
            Self::Tools => "tools",
            Self::Prompts => "prompts",
            Self::Resources => "resources",
            Self::ResourceTemplates => "resourceTemplates",
            Self::Capabilities => "capabilities",
            Self::Logging => "logging",
            Self::Completions => "completions",
            Self::Experimental => "experimental",
            Self::Tasks => "tasks",
            Self::List => "list",
            Self::Cancel => "cancel",
            Self::Requests => "requests",
            Self::Call => "call",
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
            Self::Execution => "execution",
            Self::TaskSupport => "taskSupport",
            Self::Content => "content",
            Self::StructuredContent => "structuredContent",
            Self::IsError => "isError",
            Self::Schema => "$schema",
            Self::Vocabulary => "$vocabulary",
            Self::Type => "type",
            Self::Pattern => "pattern",
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
            Self::Endpoint => "endpoint",
            Self::Resolution => "resolution",
            Self::Address => "address",
            Self::Peer => "peer",
            Self::Trust => "trust",
            Self::Tls => "tls",
            Self::Http => "http",
            Self::Headers => "headers",
            Self::Status => "status",
            Self::Body => "body",
            Self::Event => "event",
            Self::Credentials => "credentials",
            Self::Session => "session",
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

/// A fixed-size, value-free summary of one generated input. Object member
/// names, strings, numbers, and booleans are deliberately not retained.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct StructuralInput {
    root: JsonKind,
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

impl StructuralInput {
    #[allow(clippy::too_many_arguments)]
    pub(super) const fn new(
        root: JsonKind,
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
    ) -> Self {
        Self {
            root,
            byte_count,
            node_count,
            maximum_depth,
            nulls,
            booleans,
            numbers,
            strings,
            arrays,
            array_items,
            objects,
            object_members,
        }
    }

    pub(super) const fn root(&self) -> JsonKind {
        self.root
    }

    pub(super) const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    pub(super) const fn node_count(&self) -> u64 {
        self.node_count
    }

    pub(super) const fn maximum_depth(&self) -> u64 {
        self.maximum_depth
    }

    pub(super) const fn nulls(&self) -> u64 {
        self.nulls
    }

    pub(super) const fn booleans(&self) -> u64 {
        self.booleans
    }

    pub(super) const fn numbers(&self) -> u64 {
        self.numbers
    }

    pub(super) const fn strings(&self) -> u64 {
        self.strings
    }

    pub(super) const fn arrays(&self) -> u64 {
        self.arrays
    }

    pub(super) const fn array_items(&self) -> u64 {
        self.array_items
    }

    pub(super) const fn objects(&self) -> u64 {
        self.objects
    }

    pub(super) const fn object_members(&self) -> u64 {
        self.object_members
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct GeneratedCaseReproduction {
    generator: &'static str,
    seed: u64,
    input: StructuralInput,
    mutation_kind: Option<&'static str>,
}

impl GeneratedCaseReproduction {
    pub(super) const fn new(generator: &'static str, seed: u64, input: StructuralInput) -> Self {
        Self {
            generator,
            seed,
            input,
            mutation_kind: None,
        }
    }

    pub(super) const fn with_mutation_kind(mut self, mutation_kind: &'static str) -> Self {
        self.mutation_kind = Some(mutation_kind);
        self
    }

    pub(super) const fn generator(&self) -> &'static str {
        self.generator
    }

    pub(super) const fn seed(&self) -> u64 {
        self.seed
    }

    pub(super) const fn input(&self) -> &StructuralInput {
        &self.input
    }

    pub(super) const fn mutation_kind(&self) -> Option<&'static str> {
        self.mutation_kind
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
    ExpectedTaskSupport {
        observed: JsonKind,
    },
    ExpectedCurrentRevision,
    ExpectedSelectedRevision,
    ExpectedInputSchemaRootObject {
        observed: JsonKind,
    },
    ExpectedToolSchemaRootObject {
        observed: JsonKind,
    },
    ServerErrorResponse,
    DuplicateIdentifier,
    RepeatedCursor,
    UnsupportedSchemaDialect {
        observed: JsonKind,
    },
    UnsupportedSchemaVocabulary,
    UnsupportedLinearPattern,
    ExternalSchemaReference,
    UnresolvedLocalReference,
    InvalidDraft202012 {
        error_count: u64,
    },
    InvalidScenarioShape,
    UnsupportedScenarioVersion,
    UnsupportedScenarioRevision,
    DuplicateCaseId,
    InvalidEnvironmentReference,
    MissingEnvironmentValue,
    InvalidGenerationConfiguration,
    NoValidBoundaryInput,
    ToolAuthorizationMismatch,
    SideEffectsAuthorizationRequired,
    ToolNotFound,
    ArgumentsDoNotMatchSchema {
        error_count: u64,
    },
    ToolCallRejected,
    TaskExecutionRequired,
    ExpectedSuccess,
    ExpectedToolError,
    AdvertisedOutputMismatch {
        error_count: u64,
    },
    ScenarioOutputMismatch {
        error_count: u64,
    },
    AdvertisedAndScenarioOutputMismatch {
        error_count: u64,
    },
    InvalidToolResult,
    InvalidEndpoint,
    PrivateNetworkAuthorizationRequired,
    CleartextAuthorizationRequired,
    CredentialAuthorizationRequired,
    CredentialsRequireHttps,
    InvalidCredential,
    InvalidCustomField,
    InvalidTrustFile,
    ResolutionUnavailable,
    ProhibitedAddress,
    MixedAddressClasses,
    PeerOutsidePinnedSet,
    TlsVerificationFailed,
    HttpRequestFailed,
    RedirectRejected {
        status: u16,
    },
    AuthenticationRejected {
        status: u16,
    },
    HttpStatusRejected {
        status: u16,
    },
    ContentEncodingRejected,
    MediaTypeRejected,
    InvalidResponseMessage,
    InvalidSseEvent,
    InvalidHttpHeaderAnnotation,
    InvalidMirroredHeaderValue,
    HeaderMismatch,
    InvalidSession,
    SessionChanged,
    SessionRequired {
        status: u16,
    },
    SessionLost {
        status: u16,
    },
    InitializedRejected {
        status: u16,
    },
    ProtocolVersionRejected,
    UnsupportedProtocolVersion,
}

impl RuleViolation {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ExpectedShape { .. } => "expected_shape",
            Self::ExpectedCompleteResult { .. } => "expected_complete_result",
            Self::ExpectedCacheScope { .. } => "expected_cache_scope",
            Self::ExpectedTaskSupport { .. } => "expected_task_support",
            Self::ExpectedCurrentRevision => "expected_current_revision",
            Self::ExpectedSelectedRevision => "expected_selected_revision",
            Self::ExpectedInputSchemaRootObject { .. } => "expected_input_schema_root_object",
            Self::ExpectedToolSchemaRootObject { .. } => "expected_tool_schema_root_object",
            Self::ServerErrorResponse => "server_error_response",
            Self::DuplicateIdentifier => "duplicate_identifier",
            Self::RepeatedCursor => "repeated_cursor",
            Self::UnsupportedSchemaDialect { .. } => "unsupported_schema_dialect",
            Self::UnsupportedSchemaVocabulary => "unsupported_schema_vocabulary",
            Self::UnsupportedLinearPattern => "unsupported_linear_pattern",
            Self::ExternalSchemaReference => "external_schema_reference",
            Self::UnresolvedLocalReference => "unresolved_local_reference",
            Self::InvalidDraft202012 { .. } => "invalid_draft_2020_12",
            Self::InvalidScenarioShape => "invalid_scenario_shape",
            Self::UnsupportedScenarioVersion => "unsupported_scenario_version",
            Self::UnsupportedScenarioRevision => "unsupported_scenario_revision",
            Self::DuplicateCaseId => "duplicate_case_id",
            Self::InvalidEnvironmentReference => "invalid_environment_reference",
            Self::MissingEnvironmentValue => "missing_environment_value",
            Self::InvalidGenerationConfiguration => "invalid_generation_configuration",
            Self::NoValidBoundaryInput => "no_valid_boundary_input",
            Self::ToolAuthorizationMismatch => "exact_tool_authorization_required",
            Self::SideEffectsAuthorizationRequired => "side_effects_authorization_required",
            Self::ToolNotFound => "exact_tool_not_found",
            Self::ArgumentsDoNotMatchSchema { .. } => "arguments_do_not_match_input_schema",
            Self::ToolCallRejected => "tool_call_rejected",
            Self::TaskExecutionRequired => "task_execution_required",
            Self::ExpectedSuccess => "expected_success",
            Self::ExpectedToolError => "expected_tool_error",
            Self::AdvertisedOutputMismatch { .. } => "advertised_output_schema_mismatch",
            Self::ScenarioOutputMismatch { .. } => "scenario_output_schema_mismatch",
            Self::AdvertisedAndScenarioOutputMismatch { .. } => {
                "advertised_and_scenario_output_schema_mismatch"
            }
            Self::InvalidToolResult => "invalid_tool_result",
            Self::InvalidEndpoint => "invalid_endpoint",
            Self::PrivateNetworkAuthorizationRequired => "private_network_authorization_required",
            Self::CleartextAuthorizationRequired => "cleartext_authorization_required",
            Self::CredentialAuthorizationRequired => "credential_authorization_required",
            Self::CredentialsRequireHttps => "credentials_require_https",
            Self::InvalidCredential => "invalid_credential",
            Self::InvalidCustomField => "invalid_custom_field",
            Self::InvalidTrustFile => "invalid_trust_file",
            Self::ResolutionUnavailable => "resolution_unavailable",
            Self::ProhibitedAddress => "prohibited_address",
            Self::MixedAddressClasses => "mixed_address_classes",
            Self::PeerOutsidePinnedSet => "peer_outside_pinned_set",
            Self::TlsVerificationFailed => "tls_verification_failed",
            Self::HttpRequestFailed => "http_request_failed",
            Self::RedirectRejected { .. } => "redirect_rejected",
            Self::AuthenticationRejected { .. } => "authentication_rejected",
            Self::HttpStatusRejected { .. } => "http_status_rejected",
            Self::ContentEncodingRejected => "content_encoding_rejected",
            Self::MediaTypeRejected => "media_type_rejected",
            Self::InvalidResponseMessage => "invalid_response_message",
            Self::InvalidSseEvent => "invalid_sse_event",
            Self::InvalidHttpHeaderAnnotation => "invalid_http_header_annotation",
            Self::InvalidMirroredHeaderValue => "invalid_mirrored_header_value",
            Self::HeaderMismatch => "header_mismatch",
            Self::InvalidSession => "invalid_session_id",
            Self::SessionChanged => "session_id_changed",
            Self::SessionRequired { .. } => "session_id_required",
            Self::SessionLost { .. } => "session_lost",
            Self::InitializedRejected { .. } => "initialized_notification_rejected",
            Self::ProtocolVersionRejected => "protocol_version_header_rejected",
            Self::UnsupportedProtocolVersion => "unsupported_protocol_version",
        }
    }

    pub(super) const fn observed(self) -> Option<JsonKind> {
        match self {
            Self::ExpectedShape { observed, .. }
            | Self::ExpectedCompleteResult { observed }
            | Self::ExpectedCacheScope { observed }
            | Self::ExpectedTaskSupport { observed }
            | Self::ExpectedInputSchemaRootObject { observed }
            | Self::ExpectedToolSchemaRootObject { observed }
            | Self::UnsupportedSchemaDialect { observed } => Some(observed),
            Self::ExpectedCurrentRevision
            | Self::ExpectedSelectedRevision
            | Self::ServerErrorResponse
            | Self::DuplicateIdentifier
            | Self::RepeatedCursor
            | Self::UnsupportedSchemaVocabulary
            | Self::UnsupportedLinearPattern
            | Self::ExternalSchemaReference
            | Self::UnresolvedLocalReference
            | Self::InvalidDraft202012 { .. }
            | Self::InvalidScenarioShape
            | Self::UnsupportedScenarioVersion
            | Self::UnsupportedScenarioRevision
            | Self::DuplicateCaseId
            | Self::InvalidEnvironmentReference
            | Self::MissingEnvironmentValue
            | Self::InvalidGenerationConfiguration
            | Self::NoValidBoundaryInput
            | Self::ToolAuthorizationMismatch
            | Self::SideEffectsAuthorizationRequired
            | Self::ToolNotFound
            | Self::ArgumentsDoNotMatchSchema { .. }
            | Self::ToolCallRejected
            | Self::TaskExecutionRequired
            | Self::ExpectedSuccess
            | Self::ExpectedToolError
            | Self::AdvertisedOutputMismatch { .. }
            | Self::ScenarioOutputMismatch { .. }
            | Self::AdvertisedAndScenarioOutputMismatch { .. }
            | Self::InvalidToolResult => None,
            Self::InvalidEndpoint
            | Self::PrivateNetworkAuthorizationRequired
            | Self::CleartextAuthorizationRequired
            | Self::CredentialAuthorizationRequired
            | Self::CredentialsRequireHttps
            | Self::InvalidCredential
            | Self::InvalidCustomField
            | Self::InvalidTrustFile
            | Self::ResolutionUnavailable
            | Self::ProhibitedAddress
            | Self::MixedAddressClasses
            | Self::PeerOutsidePinnedSet
            | Self::TlsVerificationFailed
            | Self::HttpRequestFailed
            | Self::RedirectRejected { .. }
            | Self::AuthenticationRejected { .. }
            | Self::HttpStatusRejected { .. }
            | Self::ContentEncodingRejected
            | Self::MediaTypeRejected
            | Self::InvalidResponseMessage
            | Self::InvalidSseEvent
            | Self::InvalidHttpHeaderAnnotation
            | Self::InvalidMirroredHeaderValue
            | Self::HeaderMismatch => None,
            Self::InvalidSession
            | Self::SessionChanged
            | Self::SessionRequired { .. }
            | Self::SessionLost { .. }
            | Self::InitializedRejected { .. }
            | Self::ProtocolVersionRejected
            | Self::UnsupportedProtocolVersion => None,
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
            Self::InvalidDraft202012 { error_count }
            | Self::ArgumentsDoNotMatchSchema { error_count }
            | Self::AdvertisedOutputMismatch { error_count }
            | Self::ScenarioOutputMismatch { error_count }
            | Self::AdvertisedAndScenarioOutputMismatch { error_count } => Some(error_count),
            _ => None,
        }
    }

    pub(super) const fn http_status(self) -> Option<u16> {
        match self {
            Self::RedirectRejected { status }
            | Self::AuthenticationRejected { status }
            | Self::HttpStatusRejected { status }
            | Self::SessionRequired { status }
            | Self::SessionLost { status }
            | Self::InitializedRejected { status } => Some(status),
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

    pub(super) fn remote_target_invalid(location: Location, violation: RuleViolation) -> Self {
        Self::new(
            FindingCode::RemoteTargetInvalid,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(violation),
        )
    }

    pub(super) fn network_authorization_missing(
        location: Location,
        violation: RuleViolation,
    ) -> Self {
        Self::new(
            FindingCode::NetworkAuthorizationMissing,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(violation),
        )
    }

    pub(super) fn resolution_failed(location: Location, violation: RuleViolation) -> Self {
        Self::new(
            FindingCode::ResolutionFailed,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(violation),
        )
    }

    pub(super) fn address_policy_blocked(location: Location, violation: RuleViolation) -> Self {
        Self::new(
            FindingCode::AddressPolicyBlocked,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(violation),
        )
    }

    pub(super) fn peer_address_mismatch(location: Location) -> Self {
        Self::new(
            FindingCode::PeerAddressMismatch,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(RuleViolation::PeerOutsidePinnedSet),
        )
    }

    pub(super) fn tls_verification_failed(location: Location) -> Self {
        Self::new(
            FindingCode::TlsVerificationFailed,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(RuleViolation::TlsVerificationFailed),
        )
    }

    pub(super) fn http_exchange_failed(location: Location, violation: RuleViolation) -> Self {
        Self::new(
            FindingCode::HttpExchangeFailed,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(violation),
        )
    }

    pub(super) fn http_response_invalid(location: Location, violation: RuleViolation) -> Self {
        Self::new(
            FindingCode::HttpResponseInvalid,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(violation),
        )
    }

    pub(super) fn remote_authentication_rejected(location: Location, status: u16) -> Self {
        Self::new(
            FindingCode::RemoteAuthenticationRejected,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(RuleViolation::AuthenticationRejected { status }),
        )
    }

    pub(super) fn http_header_mapping_invalid(
        location: Location,
        violation: RuleViolation,
    ) -> Self {
        Self::new(
            FindingCode::HttpHeaderMappingInvalid,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(violation),
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

    pub(super) fn unsupported_protocol_version(
        revision: SupportedRevision,
        location: Location,
    ) -> Self {
        Self::new(
            FindingCode::UnsupportedProtocolRevision,
            revision,
            location,
            FindingEvidence::RuleViolation(RuleViolation::UnsupportedProtocolVersion),
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

    pub(super) fn revision_mismatch(revision: SupportedRevision, location: Location) -> Self {
        Self::new(
            FindingCode::ProtocolRevisionMismatch,
            revision,
            location,
            FindingEvidence::RuleViolation(RuleViolation::ExpectedSelectedRevision),
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

    pub(super) fn session_cleanup_failed(revision: SupportedRevision, location: Location) -> Self {
        Self::new(
            FindingCode::SessionCleanupFailed,
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

    pub(super) fn ambiguous_schema_dialect(
        revision: SupportedRevision,
        location: Location,
    ) -> Self {
        Self::new(
            FindingCode::AmbiguousSchemaDialect,
            revision,
            location,
            FindingEvidence::None,
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

    pub(super) fn scenario_invalid(location: Location, violation: RuleViolation) -> Self {
        Self::new(
            FindingCode::ScenarioInvalid,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(violation),
        )
    }

    pub(super) fn secret_reference_invalid(location: Location, violation: RuleViolation) -> Self {
        Self::new(
            FindingCode::SecretReferenceInvalid,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(violation),
        )
    }

    pub(super) fn scenario_schema_invalid(location: Location, violation: RuleViolation) -> Self {
        Self::new(
            FindingCode::ScenarioSchemaInvalid,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(violation),
        )
    }

    pub(super) fn case_generation_failed(location: Location, violation: RuleViolation) -> Self {
        debug_assert!(matches!(
            violation,
            RuleViolation::InvalidGenerationConfiguration | RuleViolation::NoValidBoundaryInput
        ));
        Self::new(
            FindingCode::CaseGenerationFailed,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(violation),
        )
    }

    pub(super) fn tool_authorization_missing(location: Location) -> Self {
        Self::new(
            FindingCode::ToolAuthorizationMissing,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(RuleViolation::ToolAuthorizationMismatch),
        )
    }

    pub(super) fn side_effects_not_authorized(location: Location) -> Self {
        Self::new(
            FindingCode::SideEffectsNotAuthorized,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(RuleViolation::SideEffectsAuthorizationRequired),
        )
    }

    pub(super) fn tool_not_found(location: Location) -> Self {
        Self::new(
            FindingCode::ToolNotFound,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(RuleViolation::ToolNotFound),
        )
    }

    pub(super) fn tool_arguments_mismatch(location: Location, error_count: u64) -> Self {
        Self::new(
            FindingCode::ToolArgumentsMismatch,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(RuleViolation::ArgumentsDoNotMatchSchema {
                error_count,
            }),
        )
    }

    pub(super) fn tool_call_rejected(location: Location) -> Self {
        Self::new(
            FindingCode::ToolCallRejected,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(RuleViolation::ToolCallRejected),
        )
    }

    pub(super) fn tool_result_mismatch(location: Location, violation: RuleViolation) -> Self {
        debug_assert!(matches!(
            violation,
            RuleViolation::ExpectedSuccess | RuleViolation::ExpectedToolError
        ));
        Self::new(
            FindingCode::ToolResultMismatch,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(violation),
        )
    }

    pub(super) fn tool_output_mismatch(location: Location, violation: RuleViolation) -> Self {
        debug_assert!(matches!(
            violation,
            RuleViolation::AdvertisedOutputMismatch { .. }
                | RuleViolation::ScenarioOutputMismatch { .. }
                | RuleViolation::AdvertisedAndScenarioOutputMismatch { .. }
        ));
        Self::new(
            FindingCode::ToolOutputMismatch,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(violation),
        )
    }

    pub(super) fn tool_result_invalid(location: Location) -> Self {
        Self::new(
            FindingCode::ToolResultInvalid,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::RuleViolation(RuleViolation::InvalidToolResult),
        )
    }

    pub(super) fn tool_task_required(revision: SupportedRevision, location: Location) -> Self {
        Self::new(
            FindingCode::ToolTaskRequired,
            revision,
            location,
            FindingEvidence::RuleViolation(RuleViolation::TaskExecutionRequired),
        )
    }

    pub(super) fn schema_invalid_arguments_accepted(location: Location) -> Self {
        Self::new(
            FindingCode::SchemaInvalidArgumentsAccepted,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::None,
        )
    }

    pub(super) fn workflow_capture_missing(location: Location) -> Self {
        Self::new(
            FindingCode::WorkflowCaptureMissing,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::None,
        )
    }

    pub(super) fn workflow_cleanup_failed(location: Location) -> Self {
        Self::new(
            FindingCode::WorkflowCleanupFailed,
            SupportedRevision::CURRENT,
            location,
            FindingEvidence::None,
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

    pub(super) fn with_revision(mut self, revision: SupportedRevision) -> Self {
        self.revision = revision;
        self
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
    reproduction: Option<GeneratedCaseReproduction>,
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
            reproduction: None,
        }
    }

    pub(super) const fn skipped(id: CheckId, requirement: Requirement, reason: SkipReason) -> Self {
        Self {
            id,
            requirement,
            state: CheckState::Skipped { reason },
            reproduction: None,
        }
    }

    pub(super) fn with_reproduction(mut self, reproduction: GeneratedCaseReproduction) -> Self {
        assert!(
            matches!(self.id, CheckId::RuntimeToolCase(_)),
            "only a generated runtime case can retain reproduction evidence"
        );
        self.reproduction = Some(reproduction);
        self
    }

    pub(super) const fn id(&self) -> CheckId {
        self.id
    }

    pub(super) const fn requirement(&self) -> Requirement {
        self.requirement
    }

    pub(super) const fn reproduction(&self) -> Option<&GeneratedCaseReproduction> {
        self.reproduction.as_ref()
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
            (
                FindingCode::ProtocolRevisionMismatch,
                "MCP-PROTOCOL-005",
                Severity::Error,
            ),
            (FindingCode::LimitExceeded, "MCP-LIMIT-001", Severity::Error),
            (
                FindingCode::CleanupFailed,
                "MCP-SAFETY-001",
                Severity::Critical,
            ),
            (
                FindingCode::SessionCleanupFailed,
                "MCP-SAFETY-002",
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
            (
                FindingCode::AmbiguousSchemaDialect,
                "MCP-SCHEMA-004",
                Severity::Warning,
            ),
            (
                FindingCode::ScenarioInvalid,
                "MCP-SCENARIO-001",
                Severity::Error,
            ),
            (
                FindingCode::SecretReferenceInvalid,
                "MCP-SCENARIO-002",
                Severity::Error,
            ),
            (
                FindingCode::ScenarioSchemaInvalid,
                "MCP-SCENARIO-003",
                Severity::Error,
            ),
            (
                FindingCode::ToolAuthorizationMissing,
                "MCP-AUTH-001",
                Severity::Error,
            ),
            (
                FindingCode::SideEffectsNotAuthorized,
                "MCP-AUTH-002",
                Severity::Error,
            ),
            (
                FindingCode::CaseGenerationFailed,
                "MCP-GENERATION-001",
                Severity::Error,
            ),
            (FindingCode::ToolNotFound, "MCP-ACTIVE-001", Severity::Error),
            (
                FindingCode::ToolArgumentsMismatch,
                "MCP-ACTIVE-002",
                Severity::Error,
            ),
            (
                FindingCode::ToolCallRejected,
                "MCP-ACTIVE-003",
                Severity::Error,
            ),
            (
                FindingCode::ToolResultMismatch,
                "MCP-ACTIVE-004",
                Severity::Error,
            ),
            (
                FindingCode::ToolOutputMismatch,
                "MCP-ACTIVE-005",
                Severity::Error,
            ),
            (
                FindingCode::ToolResultInvalid,
                "MCP-ACTIVE-006",
                Severity::Error,
            ),
            (
                FindingCode::ToolTaskRequired,
                "MCP-ACTIVE-007",
                Severity::Error,
            ),
            (
                FindingCode::SchemaInvalidArgumentsAccepted,
                "MCP-ACTIVE-008",
                Severity::Critical,
            ),
            (
                FindingCode::WorkflowCaptureMissing,
                "MCP-WORKFLOW-001",
                Severity::Error,
            ),
            (
                FindingCode::WorkflowCleanupFailed,
                "MCP-SAFETY-003",
                Severity::Critical,
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
            (CheckId::ScenarioConfiguration, "scenario.configuration"),
            (CheckId::GenerationConfiguration, "generation.configuration"),
            (CheckId::ActiveAuthorization, "authorization.active"),
            (CheckId::TransportStdio, "transport.stdio"),
            (CheckId::ProtocolEnvelope, "protocol.envelope"),
            (CheckId::ProtocolRevision, "protocol.revision"),
            (CheckId::DiscoveryCatalogs, "discovery.catalogs"),
            (CheckId::SchemaContracts, "schema.contracts"),
            (CheckId::CaseGeneration, "generation.cases"),
            (CheckId::RuntimeTools, "runtime.tools"),
            (CheckId::RuntimeToolCase(17), "runtime.tools.case[17]"),
            (
                CheckId::RuntimeWorkflowStep(17),
                "runtime.workflow.step[17]",
            ),
            (
                CheckId::RuntimeWorkflowCleanup(17),
                "runtime.workflow.cleanup[17]",
            ),
        ];
        let skip_reasons = [
            (SkipReason::NotAuthorized, "not_authorized"),
            (SkipReason::AuthorizationFailed, "authorization_failed"),
            (SkipReason::NotAdvertised, "not_advertised"),
            (SkipReason::InputRequired, "input_required"),
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
        assert!(SkipReason::AuthorizationFailed.is_causal());
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
