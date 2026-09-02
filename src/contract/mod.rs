#![allow(
    dead_code,
    reason = "the complete diagnostic model includes checks used by optional command paths"
)]

mod active;
mod active_protocol;
mod catalog;
mod generate;
mod http_headers;
mod limits;
mod model;
mod protocol;
mod redaction;
mod report;
mod schema_budget;
mod snapshot;

use crate::transport::ProbeResponse;
use crate::transport::http::{
    HttpFailure, HttpLimit, ResolutionFailure, ResponseFailure, TargetFailure,
};
use crate::transport::stdio::{StdioFailure, StdioLimit, StdioStream as TransportStream};
use limits::{DiagnosticLimits, LimitKind, LimitViolation};
use model::{
    CheckId, CheckResult, Finding, JsonRpcErrorKind, Location, LocationField, Requirement,
    RuleViolation, SkipReason,
};
use protocol::SupportedRevision;
use redaction::RedactedValue;
use report::{DiagnosticReport, render_reports};

pub(crate) use active::{
    ActiveConversation, ActiveScenario, MAX_SCENARIO_BYTES, REJECTION_CASE_COUNT,
    SCENARIO_SCHEMA_VERSION, ScenarioFailure, WORKFLOW_SCHEMA_VERSION,
    render_authorization_failure_for_revision,
    render_generation_configuration_failure_for_revision,
    render_resolved_scenario_failure_for_revision, render_scenario_failure_for_revision,
    resolve_target_environment,
};
pub(crate) use catalog::{AutoDiscoveryOutcome, PassiveCatalogConversation};
pub(crate) use generate::GENERATOR_VERSION;
pub(crate) use limits::DiagnosticLimitProfile;
pub(crate) use protocol::{
    ActiveProtocolRevision, KnownRevision, PassiveProtocolSelection, ProtocolSelectionEvidence,
    ProtocolSelectionMode, ProtocolSelectionPath, SupportedRevision as ProtocolRevision,
};
pub(crate) use report::{
    BADGE_REPORT_VERSION, ExitStatus, MARKDOWN_REPORT_VERSION, REPORT_SCHEMA_VERSION,
    RenderedReportArtifact, ReportArtifactFormat, ReportFormat, ReportRequest,
};
pub(crate) use snapshot::{
    DIFF_SCHEMA_VERSION, DiffFormat, RenderedContractDiff, SNAPSHOT_SCHEMA_VERSION,
    SnapshotDestination, SnapshotDestinationError, capture_contract_snapshot,
    prepare_snapshot_destination, render_contract_diff,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ReportTransport {
    Stdio,
    Http,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum StdioLimitKind {
    StartupTime,
    DiscoveryTime,
    RequestTime,
    ResponseTime,
    TotalTime,
    MessageBytes,
    StdoutBytes,
    StderrBytes,
    AggregateOutputBytes,
    MessageCount,
}

impl StdioLimitKind {
    const fn contract_kind(self) -> LimitKind {
        match self {
            Self::StartupTime => LimitKind::StartupTime,
            Self::DiscoveryTime => LimitKind::DiscoveryTime,
            Self::RequestTime => LimitKind::RequestTime,
            Self::ResponseTime => LimitKind::ResponseTime,
            Self::TotalTime => LimitKind::TotalTime,
            Self::MessageBytes => LimitKind::MessageBytes,
            Self::StdoutBytes => LimitKind::StdoutBytes,
            Self::StderrBytes => LimitKind::StderrBytes,
            Self::AggregateOutputBytes => LimitKind::AggregateOutputBytes,
            Self::MessageCount => LimitKind::MessageCount,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct StdioLimitProfile {
    pub(crate) startup_ms: u64,
    pub(crate) discovery_ms: u64,
    pub(crate) request_ms: u64,
    pub(crate) response_ms: u64,
    pub(crate) shutdown_grace_ms: u64,
    pub(crate) total_ms: u64,
    pub(crate) message_bytes: u64,
    pub(crate) stdout_bytes: u64,
    pub(crate) stderr_bytes: u64,
    pub(crate) aggregate_output_bytes: u64,
    pub(crate) message_count: u64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HttpLimitProfile {
    pub(crate) startup_ms: u64,
    pub(crate) discovery_ms: u64,
    pub(crate) request_ms: u64,
    pub(crate) response_ms: u64,
    pub(crate) shutdown_grace_ms: u64,
    pub(crate) total_ms: u64,
    pub(crate) endpoint_bytes: u64,
    pub(crate) resolution_addresses: u64,
    pub(crate) trust_bytes: u64,
    pub(crate) trust_certificates: u64,
    pub(crate) request_fields: u64,
    pub(crate) request_field_name_bytes: u64,
    pub(crate) request_field_value_bytes: u64,
    pub(crate) request_fields_bytes: u64,
    pub(crate) response_fields: u64,
    pub(crate) response_field_name_bytes: u64,
    pub(crate) response_field_value_bytes: u64,
    pub(crate) response_fields_bytes: u64,
    pub(crate) message_bytes: u64,
    pub(crate) aggregate_output_bytes: u64,
    pub(crate) message_count: u64,
    pub(crate) protocol_revisions: u64,
}

pub(crate) fn diagnostic_http_limit_profile(selected: DiagnosticLimitProfile) -> HttpLimitProfile {
    let values = selected.limits().values();
    HttpLimitProfile {
        startup_ms: values.startup_ms,
        discovery_ms: values.discovery_ms,
        request_ms: values.request_ms,
        response_ms: values.response_ms,
        shutdown_grace_ms: values.shutdown_grace_ms,
        total_ms: values.total_ms,
        endpoint_bytes: values.endpoint_bytes,
        resolution_addresses: values.resolution_addresses,
        trust_bytes: values.trust_bytes,
        trust_certificates: values.trust_certificates,
        request_fields: values.request_fields,
        request_field_name_bytes: values.request_field_name_bytes,
        request_field_value_bytes: values.request_field_value_bytes,
        request_fields_bytes: values.request_fields_bytes,
        response_fields: values.response_fields,
        response_field_name_bytes: values.response_field_name_bytes,
        response_field_value_bytes: values.response_field_value_bytes,
        response_fields_bytes: values.response_fields_bytes,
        message_bytes: values.message_bytes,
        aggregate_output_bytes: values.aggregate_output_bytes,
        message_count: values.message_count,
        protocol_revisions: values.protocol_revisions,
    }
}

pub(crate) fn diagnostic_stdio_limit_profile(
    selected: DiagnosticLimitProfile,
) -> StdioLimitProfile {
    let values = selected.limits().values();
    StdioLimitProfile {
        startup_ms: values.startup_ms,
        discovery_ms: values.discovery_ms,
        request_ms: values.request_ms,
        response_ms: values.response_ms,
        shutdown_grace_ms: values.shutdown_grace_ms,
        total_ms: values.total_ms,
        message_bytes: values.message_bytes,
        stdout_bytes: values.stdout_bytes,
        stderr_bytes: values.stderr_bytes,
        aggregate_output_bytes: values.aggregate_output_bytes,
        message_count: values.message_count,
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum StdioStream {
    Process,
    Stdin,
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum StdioPrimaryFailure {
    ProcessStart,
    Io {
        stream: StdioStream,
    },
    InvalidMessage {
        byte_count: usize,
        index: usize,
    },
    EarlyExit,
    Limit {
        kind: StdioLimitKind,
        observed: u64,
        maximum: u64,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct StdioDiagnostic {
    pub(crate) primary: Option<StdioPrimaryFailure>,
    pub(crate) cleanup_failed: bool,
}

pub(crate) fn stdio_diagnostic(
    failure: Option<StdioFailure>,
    cleanup_failed: bool,
) -> StdioDiagnostic {
    StdioDiagnostic {
        primary: failure.map(map_stdio_failure),
        cleanup_failed,
    }
}

fn map_stdio_failure(failure: StdioFailure) -> StdioPrimaryFailure {
    match failure {
        StdioFailure::ProcessStart => StdioPrimaryFailure::ProcessStart,
        StdioFailure::Io { stream } => StdioPrimaryFailure::Io {
            stream: match stream {
                TransportStream::Process => StdioStream::Process,
                TransportStream::Stdin => StdioStream::Stdin,
                TransportStream::Stdout => StdioStream::Stdout,
                TransportStream::Stderr => StdioStream::Stderr,
            },
        },
        StdioFailure::InvalidMessage { byte_count, index } => {
            StdioPrimaryFailure::InvalidMessage { byte_count, index }
        }
        StdioFailure::EarlyExit => StdioPrimaryFailure::EarlyExit,
        StdioFailure::Limit {
            kind,
            observed,
            maximum,
        } => StdioPrimaryFailure::Limit {
            kind: match kind {
                StdioLimit::StartupTime => StdioLimitKind::StartupTime,
                StdioLimit::DiscoveryTime => StdioLimitKind::DiscoveryTime,
                StdioLimit::RequestTime => StdioLimitKind::RequestTime,
                StdioLimit::ResponseTime => StdioLimitKind::ResponseTime,
                StdioLimit::TotalTime => StdioLimitKind::TotalTime,
                StdioLimit::MessageBytes => StdioLimitKind::MessageBytes,
                StdioLimit::StdoutBytes => StdioLimitKind::StdoutBytes,
                StdioLimit::StderrBytes => StdioLimitKind::StderrBytes,
                StdioLimit::AggregateOutputBytes => StdioLimitKind::AggregateOutputBytes,
                StdioLimit::MessageCount => StdioLimitKind::MessageCount,
            },
            observed,
            maximum,
        },
    }
}

pub(crate) struct RenderedDiagnostic {
    pub(crate) output: String,
    pub(crate) artifacts: Vec<RenderedReportArtifact>,
    pub(crate) exit: ExitStatus,
    pub(crate) error: Option<String>,
}

pub(crate) struct Diagnostic {
    report: DiagnosticReport,
}

impl Diagnostic {
    fn from_report(report: DiagnosticReport) -> Self {
        Self { report }
    }

    pub(crate) fn with_limit_profile(mut self, profile: DiagnosticLimitProfile) -> Self {
        self.report = self.report.with_limit_profile(profile);
        self
    }

    pub(crate) fn with_protocol_selection(mut self, selection: ProtocolSelectionEvidence) -> Self {
        self.report = self.report.with_protocol_selection(selection);
        self
    }

    pub(crate) fn render(self, request: ReportRequest) -> RenderedDiagnostic {
        match render_reports(&self.report, request) {
            Ok(reports) => RenderedDiagnostic {
                output: reports.stdout,
                artifacts: reports.artifacts,
                exit: self.report.exit_status(),
                error: None,
            },
            Err(error) => RenderedDiagnostic {
                output: String::new(),
                artifacts: Vec::new(),
                exit: ExitStatus::InternalError,
                error: Some(error.to_string()),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct HttpDiagnostic {
    failure: Option<HttpFailure>,
    tls_applicable: Option<bool>,
    session_cleanup_failed: bool,
}

impl HttpDiagnostic {
    pub(in crate::contract) const fn failed(self) -> bool {
        self.failure.is_some()
    }

    pub(in crate::contract) const fn unsupported_protocol_version(self) -> bool {
        matches!(
            self.failure,
            Some(HttpFailure::Response(
                ResponseFailure::UnsupportedProtocolVersion
            ))
        )
    }

    const fn modern_lifecycle_rejected(self) -> bool {
        matches!(
            self.failure,
            Some(HttpFailure::Response(
                ResponseFailure::MissingRequiredClientCapability
                    | ResponseFailure::ContradictoryProtocolVersion
            ))
        )
    }

    const fn protocol_revision_limit(self) -> Option<(u64, u64)> {
        match self.failure {
            Some(HttpFailure::Limit {
                kind: HttpLimit::ProtocolRevisions,
                observed,
                maximum,
            }) => Some((observed, maximum)),
            _ => None,
        }
    }

    pub(in crate::contract) const fn without_primary_failure(self) -> Self {
        Self {
            failure: None,
            tls_applicable: self.tls_applicable,
            session_cleanup_failed: self.session_cleanup_failed,
        }
    }
}

pub(crate) const fn http_diagnostic(
    failure: Option<HttpFailure>,
    tls_applicable: Option<bool>,
) -> HttpDiagnostic {
    HttpDiagnostic {
        failure,
        tls_applicable,
        session_cleanup_failed: false,
    }
}

pub(crate) const fn http_diagnostic_with_cleanup(
    failure: Option<HttpFailure>,
    tls_applicable: Option<bool>,
    session_cleanup_failed: bool,
) -> HttpDiagnostic {
    HttpDiagnostic {
        failure,
        tls_applicable,
        session_cleanup_failed,
    }
}

pub(crate) fn render_http_diagnostic(diagnostic: HttpDiagnostic) -> Diagnostic {
    render_http_diagnostic_for_revision(diagnostic, SupportedRevision::CURRENT)
}

pub(crate) fn render_http_diagnostic_for_revision(
    diagnostic: HttpDiagnostic,
    revision: SupportedRevision,
) -> Diagnostic {
    render_http_diagnostic_for_revision_with_negotiated(diagnostic, revision, None)
}

pub(crate) fn render_http_diagnostic_for_revision_with_negotiated(
    diagnostic: HttpDiagnostic,
    revision: SupportedRevision,
    negotiated_revision: Option<protocol::KnownRevision>,
) -> Diagnostic {
    if let Some((observed, maximum)) = diagnostic.protocol_revision_limit() {
        let mut checks = http_checks_for_revision(diagnostic.without_primary_failure(), revision);
        checks.extend(protocol_revision_limit_checks(revision, observed, maximum));
        return render_checks_for_revision(checks, revision, negotiated_revision);
    }
    if diagnostic.unsupported_protocol_version() {
        let mut checks = http_checks_for_revision(diagnostic.without_primary_failure(), revision);
        checks.extend(protocol_version_rejection_checks(revision));
        return render_checks_for_revision(checks, revision, negotiated_revision);
    }
    if diagnostic.modern_lifecycle_rejected() {
        let mut checks = http_checks_for_revision(diagnostic.without_primary_failure(), revision);
        checks.extend([
            CheckResult::performed(CheckId::ProtocolEnvelope, Requirement::Required, Vec::new()),
            CheckResult::performed(
                CheckId::ProtocolRevision,
                Requirement::Required,
                vec![Finding::lifecycle_method_rejected(
                    revision,
                    Location::root(LocationField::ServerDiscover).field(LocationField::Response),
                    JsonRpcErrorKind::Other,
                )],
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
            CheckResult::skipped(
                CheckId::RuntimeTools,
                Requirement::Optional,
                SkipReason::NotAuthorized,
            ),
        ]);
        return render_checks_for_revision(checks, revision, negotiated_revision);
    }
    let checks = http_checks_for_revision(diagnostic, revision);
    let failed = diagnostic.failure.is_some();
    let mut checks = checks;
    if failed {
        checks.extend(protocol_skips(
            SkipReason::PrerequisiteFailed,
            Requirement::Optional,
        ));
    } else {
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
            CheckResult::skipped(
                CheckId::RuntimeTools,
                Requirement::Optional,
                SkipReason::NotAuthorized,
            ),
        ]);
    }
    render_checks_for_revision(checks, revision, negotiated_revision)
}

fn protocol_revision_limit_checks(
    revision: SupportedRevision,
    observed: u64,
    maximum: u64,
) -> Vec<CheckResult> {
    vec![
        CheckResult::performed(CheckId::ProtocolEnvelope, Requirement::Required, Vec::new()),
        CheckResult::performed(
            CheckId::ProtocolRevision,
            Requirement::Required,
            vec![Finding::limit_exceeded(
                revision,
                Location::root(LocationField::ServerDiscover).field(LocationField::Response),
                LimitViolation::new(LimitKind::ProtocolRevisions, observed, maximum)
                    .expect("the HTTP revision advertisement exceeds its maximum"),
            )],
        ),
        CheckResult::skipped(
            CheckId::DiscoveryCatalogs,
            Requirement::Required,
            SkipReason::LimitReached,
        ),
        CheckResult::skipped(
            CheckId::SchemaContracts,
            Requirement::Required,
            SkipReason::LimitReached,
        ),
        CheckResult::skipped(
            CheckId::RuntimeTools,
            Requirement::Optional,
            SkipReason::NotAuthorized,
        ),
    ]
}

fn protocol_version_rejection_checks(revision: SupportedRevision) -> Vec<CheckResult> {
    vec![
        CheckResult::performed(CheckId::ProtocolEnvelope, Requirement::Required, Vec::new()),
        CheckResult::performed(
            CheckId::ProtocolRevision,
            Requirement::Required,
            vec![Finding::unsupported_protocol_version(
                revision,
                Location::root(LocationField::Http).field(LocationField::Body),
            )],
        ),
        CheckResult::skipped(
            CheckId::DiscoveryCatalogs,
            Requirement::Required,
            SkipReason::UnsupportedRevision,
        ),
        CheckResult::skipped(
            CheckId::SchemaContracts,
            Requirement::Required,
            SkipReason::UnsupportedRevision,
        ),
        CheckResult::skipped(
            CheckId::RuntimeTools,
            Requirement::Optional,
            SkipReason::NotAuthorized,
        ),
    ]
}

pub(crate) fn render_http_catalog_diagnostic(
    diagnostic: HttpDiagnostic,
    conversation: &PassiveCatalogConversation,
    responses: &[ProbeResponse],
) -> Diagnostic {
    let revision = conversation.revision();
    let mut checks = http_checks_for_revision(diagnostic, revision);
    let reserved_findings = checks
        .iter()
        .filter_map(CheckResult::findings)
        .map(<[Finding]>::len)
        .sum();
    checks.extend(catalog::diagnose(
        conversation,
        responses,
        reserved_findings,
    ));
    render_checks_for_revision(checks, revision, conversation.negotiated_revision())
}

pub(in crate::contract) fn http_checks(diagnostic: HttpDiagnostic) -> Vec<CheckResult> {
    http_checks_for_revision(diagnostic, SupportedRevision::CURRENT)
}

pub(in crate::contract) fn http_checks_for_revision(
    diagnostic: HttpDiagnostic,
    revision: SupportedRevision,
) -> Vec<CheckResult> {
    let stage = diagnostic.failure.map(http_failure_stage);
    let mut checks = Vec::new();
    checks.push(stage_check(
        CheckId::NetworkTarget,
        HttpStage::Target,
        stage,
        diagnostic
            .failure
            .and_then(|failure| http_finding(failure, revision)),
    ));
    checks.push(stage_check(
        CheckId::NetworkResolution,
        HttpStage::Resolution,
        stage,
        diagnostic
            .failure
            .and_then(|failure| http_finding(failure, revision)),
    ));
    if diagnostic.tls_applicable == Some(false) && stage != Some(HttpStage::Tls) {
        checks.push(CheckResult::skipped(
            CheckId::TransportTls,
            Requirement::Optional,
            SkipReason::NotApplicable,
        ));
    } else {
        checks.push(stage_check(
            CheckId::TransportTls,
            HttpStage::Tls,
            stage,
            diagnostic
                .failure
                .and_then(|failure| http_finding(failure, revision)),
        ));
    }
    let mut http_findings = if stage == Some(HttpStage::Http) {
        diagnostic
            .failure
            .and_then(|failure| http_finding(failure, revision))
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    if diagnostic.session_cleanup_failed {
        http_findings.push(Finding::session_cleanup_failed(
            revision,
            Location::root(LocationField::Http).field(LocationField::Session),
        ));
    }
    checks.push(
        if stage.is_some_and(|failure| failure < HttpStage::Http) && http_findings.is_empty() {
            CheckResult::skipped(
                CheckId::TransportHttp,
                Requirement::Required,
                SkipReason::PrerequisiteFailed,
            )
        } else {
            CheckResult::performed(CheckId::TransportHttp, Requirement::Required, http_findings)
        },
    );
    checks
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum HttpStage {
    Target,
    Resolution,
    Tls,
    Http,
}

fn stage_check(
    id: CheckId,
    current: HttpStage,
    failure_stage: Option<HttpStage>,
    finding: Option<Finding>,
) -> CheckResult {
    match failure_stage {
        Some(stage) if current == stage => {
            CheckResult::performed(id, Requirement::Required, finding.into_iter().collect())
        }
        Some(stage) if current > stage => {
            CheckResult::skipped(id, Requirement::Required, SkipReason::PrerequisiteFailed)
        }
        _ => CheckResult::performed(id, Requirement::Required, Vec::new()),
    }
}

fn http_failure_stage(failure: HttpFailure) -> HttpStage {
    match failure {
        HttpFailure::Target(_) => HttpStage::Target,
        HttpFailure::Resolution(_) | HttpFailure::PeerMismatch => HttpStage::Resolution,
        HttpFailure::Tls => HttpStage::Tls,
        HttpFailure::Request => HttpStage::Resolution,
        HttpFailure::ResponseIo | HttpFailure::Response(_) => HttpStage::Http,
        HttpFailure::Limit { kind, .. } => match kind {
            HttpLimit::EndpointBytes
            | HttpLimit::TrustBytes
            | HttpLimit::TrustCertificates
            | HttpLimit::RequestFields
            | HttpLimit::RequestFieldNameBytes
            | HttpLimit::RequestFieldValueBytes
            | HttpLimit::RequestFieldsBytes => HttpStage::Target,
            HttpLimit::StartupTime
            | HttpLimit::DiscoveryTime
            | HttpLimit::RequestTime
            | HttpLimit::TotalTime
            | HttpLimit::ResolutionAddresses => HttpStage::Resolution,
            HttpLimit::ResponseTime
            | HttpLimit::ResponseFields
            | HttpLimit::ResponseFieldNameBytes
            | HttpLimit::ResponseFieldValueBytes
            | HttpLimit::ResponseFieldsBytes
            | HttpLimit::MessageBytes
            | HttpLimit::AggregateOutputBytes
            | HttpLimit::MessageCount
            | HttpLimit::ProtocolRevisions => HttpStage::Http,
        },
    }
}

fn http_finding(failure: HttpFailure, revision: SupportedRevision) -> Option<Finding> {
    Some(
        match failure {
            HttpFailure::Target(failure) => match failure {
                TargetFailure::InvalidEndpoint => Finding::remote_target_invalid(
                    Location::root(LocationField::Endpoint),
                    RuleViolation::InvalidEndpoint,
                ),
                TargetFailure::PrivateNetworkAuthorizationRequired => {
                    Finding::network_authorization_missing(
                        Location::root(LocationField::Endpoint),
                        RuleViolation::PrivateNetworkAuthorizationRequired,
                    )
                }
                TargetFailure::CleartextAuthorizationRequired => {
                    Finding::network_authorization_missing(
                        Location::root(LocationField::Endpoint),
                        RuleViolation::CleartextAuthorizationRequired,
                    )
                }
                TargetFailure::CredentialAuthorizationRequired => {
                    Finding::network_authorization_missing(
                        Location::root(LocationField::Credentials),
                        RuleViolation::CredentialAuthorizationRequired,
                    )
                }
                TargetFailure::CredentialsRequireHttps => Finding::remote_target_invalid(
                    Location::root(LocationField::Credentials),
                    RuleViolation::CredentialsRequireHttps,
                ),
                TargetFailure::InvalidCredential => Finding::remote_target_invalid(
                    Location::root(LocationField::Credentials),
                    RuleViolation::InvalidCredential,
                ),
                TargetFailure::InvalidCustomField => Finding::remote_target_invalid(
                    Location::root(LocationField::Http).field(LocationField::Headers),
                    RuleViolation::InvalidCustomField,
                ),
                TargetFailure::InvalidTrustFile => Finding::remote_target_invalid(
                    Location::root(LocationField::Tls).field(LocationField::Trust),
                    RuleViolation::InvalidTrustFile,
                ),
            },
            HttpFailure::Resolution(failure) => match failure {
                ResolutionFailure::Unavailable => Finding::resolution_failed(
                    Location::root(LocationField::Resolution),
                    RuleViolation::ResolutionUnavailable,
                ),
                ResolutionFailure::ProhibitedAddress => Finding::address_policy_blocked(
                    Location::root(LocationField::Resolution).field(LocationField::Address),
                    RuleViolation::ProhibitedAddress,
                ),
                ResolutionFailure::MixedAddressClasses => Finding::address_policy_blocked(
                    Location::root(LocationField::Resolution).field(LocationField::Address),
                    RuleViolation::MixedAddressClasses,
                ),
            },
            HttpFailure::Tls => {
                Finding::tls_verification_failed(Location::root(LocationField::Tls))
            }
            HttpFailure::Request | HttpFailure::ResponseIo => Finding::http_exchange_failed(
                Location::root(LocationField::Http),
                RuleViolation::HttpRequestFailed,
            ),
            HttpFailure::PeerMismatch => Finding::peer_address_mismatch(
                Location::root(LocationField::Resolution).field(LocationField::Peer),
            ),
            HttpFailure::Response(failure) => match failure {
                ResponseFailure::Redirect { status } => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Status),
                    RuleViolation::RedirectRejected { status },
                ),
                ResponseFailure::Authentication { status } => {
                    Finding::remote_authentication_rejected(
                        Location::root(LocationField::Http).field(LocationField::Status),
                        status,
                    )
                }
                ResponseFailure::Status { status } => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Status),
                    RuleViolation::HttpStatusRejected { status },
                ),
                ResponseFailure::ContentEncoding => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Headers),
                    RuleViolation::ContentEncodingRejected,
                ),
                ResponseFailure::MediaType => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Headers),
                    RuleViolation::MediaTypeRejected,
                ),
                ResponseFailure::InvalidMessage => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Body),
                    RuleViolation::InvalidResponseMessage,
                ),
                ResponseFailure::InvalidSse => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Event),
                    RuleViolation::InvalidSseEvent,
                ),
                ResponseFailure::HeaderMismatch => Finding::http_header_mapping_invalid(
                    Location::root(LocationField::Http).field(LocationField::Headers),
                    RuleViolation::HeaderMismatch,
                ),
                ResponseFailure::MissingRequiredClientCapability => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Body),
                    RuleViolation::InvalidResponseMessage,
                ),
                ResponseFailure::ContradictoryProtocolVersion => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Body),
                    RuleViolation::InvalidResponseMessage,
                ),
                ResponseFailure::LegacyEra => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Status),
                    RuleViolation::HttpStatusRejected { status: 400 },
                ),
                ResponseFailure::InvalidSession => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Headers),
                    RuleViolation::InvalidSession,
                ),
                ResponseFailure::SessionChanged => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Headers),
                    RuleViolation::SessionChanged,
                ),
                ResponseFailure::SessionRequired { status } => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Status),
                    RuleViolation::SessionRequired { status },
                ),
                ResponseFailure::SessionLost { status } => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Status),
                    RuleViolation::SessionLost { status },
                ),
                ResponseFailure::InitializedRejected { status } => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Status),
                    RuleViolation::InitializedRejected { status },
                ),
                ResponseFailure::ProtocolVersionRejected => Finding::http_response_invalid(
                    Location::root(LocationField::Http).field(LocationField::Headers),
                    RuleViolation::ProtocolVersionRejected,
                ),
                ResponseFailure::UnsupportedProtocolVersion => {
                    Finding::unsupported_protocol_version(
                        revision,
                        Location::root(LocationField::Http).field(LocationField::Body),
                    )
                }
            },
            HttpFailure::Limit {
                kind,
                observed,
                maximum,
            } => Finding::limit_exceeded(
                revision,
                http_limit_location(kind),
                LimitViolation::new(http_limit_kind(kind), observed, maximum)
                    .expect("an HTTP limit failure exceeds its maximum"),
            ),
        }
        .with_revision(revision),
    )
}

fn http_limit_kind(kind: HttpLimit) -> LimitKind {
    match kind {
        HttpLimit::StartupTime => LimitKind::StartupTime,
        HttpLimit::DiscoveryTime => LimitKind::DiscoveryTime,
        HttpLimit::RequestTime => LimitKind::RequestTime,
        HttpLimit::ResponseTime => LimitKind::ResponseTime,
        HttpLimit::TotalTime => LimitKind::TotalTime,
        HttpLimit::EndpointBytes => LimitKind::EndpointBytes,
        HttpLimit::ResolutionAddresses => LimitKind::ResolutionAddresses,
        HttpLimit::TrustBytes => LimitKind::TrustBytes,
        HttpLimit::TrustCertificates => LimitKind::TrustCertificates,
        HttpLimit::RequestFields => LimitKind::RequestFields,
        HttpLimit::RequestFieldNameBytes => LimitKind::RequestFieldNameBytes,
        HttpLimit::RequestFieldValueBytes => LimitKind::RequestFieldValueBytes,
        HttpLimit::RequestFieldsBytes => LimitKind::RequestFieldsBytes,
        HttpLimit::ResponseFields => LimitKind::ResponseFields,
        HttpLimit::ResponseFieldNameBytes => LimitKind::ResponseFieldNameBytes,
        HttpLimit::ResponseFieldValueBytes => LimitKind::ResponseFieldValueBytes,
        HttpLimit::ResponseFieldsBytes => LimitKind::ResponseFieldsBytes,
        HttpLimit::MessageBytes => LimitKind::MessageBytes,
        HttpLimit::AggregateOutputBytes => LimitKind::AggregateOutputBytes,
        HttpLimit::MessageCount => LimitKind::MessageCount,
        HttpLimit::ProtocolRevisions => LimitKind::ProtocolRevisions,
    }
}

fn http_limit_location(kind: HttpLimit) -> Location {
    match kind {
        HttpLimit::EndpointBytes => Location::root(LocationField::Endpoint),
        HttpLimit::ResolutionAddresses | HttpLimit::StartupTime => {
            Location::root(LocationField::Resolution)
        }
        HttpLimit::TrustBytes | HttpLimit::TrustCertificates => {
            Location::root(LocationField::Tls).field(LocationField::Trust)
        }
        HttpLimit::RequestFields
        | HttpLimit::RequestFieldNameBytes
        | HttpLimit::RequestFieldValueBytes
        | HttpLimit::RequestFieldsBytes => Location::root(LocationField::Http)
            .field(LocationField::Request)
            .field(LocationField::Headers),
        HttpLimit::ResponseFields
        | HttpLimit::ResponseFieldNameBytes
        | HttpLimit::ResponseFieldValueBytes
        | HttpLimit::ResponseFieldsBytes => Location::root(LocationField::Http)
            .field(LocationField::Result)
            .field(LocationField::Headers),
        HttpLimit::MessageBytes
        | HttpLimit::AggregateOutputBytes
        | HttpLimit::MessageCount
        | HttpLimit::ProtocolRevisions => {
            Location::root(LocationField::Http).field(LocationField::Body)
        }
        HttpLimit::DiscoveryTime
        | HttpLimit::RequestTime
        | HttpLimit::ResponseTime
        | HttpLimit::TotalTime => Location::root(LocationField::Http),
    }
}

fn protocol_skips(reason: SkipReason, runtime_requirement: Requirement) -> Vec<CheckResult> {
    vec![
        CheckResult::skipped(CheckId::ProtocolEnvelope, Requirement::Required, reason),
        CheckResult::skipped(CheckId::ProtocolRevision, Requirement::Required, reason),
        CheckResult::skipped(CheckId::DiscoveryCatalogs, Requirement::Required, reason),
        CheckResult::skipped(CheckId::SchemaContracts, Requirement::Required, reason),
        CheckResult::skipped(CheckId::RuntimeTools, runtime_requirement, reason),
    ]
}

pub(crate) fn render_stdio_diagnostic(diagnostic: StdioDiagnostic) -> Diagnostic {
    render_stdio_diagnostic_for_revision(diagnostic, SupportedRevision::CURRENT)
}

pub(crate) fn render_stdio_diagnostic_for_revision(
    diagnostic: StdioDiagnostic,
    revision: SupportedRevision,
) -> Diagnostic {
    let findings = stdio_findings_for_revision(diagnostic, revision);

    render_checks_for_revision(
        vec![
            CheckResult::performed(CheckId::TransportStdio, Requirement::Required, findings),
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
            CheckResult::skipped(
                CheckId::RuntimeTools,
                Requirement::Optional,
                SkipReason::NotAuthorized,
            ),
        ],
        revision,
        None,
    )
}

pub(crate) fn render_catalog_diagnostic(
    diagnostic: StdioDiagnostic,
    conversation: &PassiveCatalogConversation,
    responses: &[ProbeResponse],
) -> Diagnostic {
    let revision = conversation.revision();
    let transport_findings = stdio_findings_for_revision(diagnostic, revision);
    let reserved_findings = transport_findings.len();
    let mut checks = vec![CheckResult::performed(
        CheckId::TransportStdio,
        Requirement::Required,
        transport_findings,
    )];
    checks.extend(catalog::diagnose(
        conversation,
        responses,
        reserved_findings,
    ));
    render_checks_for_revision(checks, revision, conversation.negotiated_revision())
}

fn stdio_findings(diagnostic: StdioDiagnostic) -> Vec<Finding> {
    stdio_findings_for_revision(diagnostic, SupportedRevision::CURRENT)
}

pub(in crate::contract) fn stdio_findings_for_revision(
    diagnostic: StdioDiagnostic,
    revision: SupportedRevision,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    if let Some(primary) = diagnostic.primary {
        findings.push(match primary {
            StdioPrimaryFailure::ProcessStart => {
                Finding::process_start_failed(revision, stdio_location(StdioStream::Process))
            }
            StdioPrimaryFailure::Io { stream } => {
                Finding::stdio_io_failed(revision, stdio_location(stream))
            }
            StdioPrimaryFailure::InvalidMessage { byte_count, index } => {
                Finding::invalid_stdio_message(
                    revision,
                    stdio_location(StdioStream::Stdout)
                        .field(LocationField::Message)
                        .index(index),
                    RedactedValue::new(byte_count),
                )
            }
            StdioPrimaryFailure::EarlyExit => {
                Finding::server_exited_early(revision, stdio_location(StdioStream::Process))
            }
            StdioPrimaryFailure::Limit {
                kind,
                observed,
                maximum,
            } => {
                let violation = LimitViolation::new(kind.contract_kind(), observed, maximum)
                    .expect("a transport limit failure must exceed its maximum");
                Finding::limit_exceeded(revision, limit_location(kind), violation)
            }
        });
    }

    if diagnostic.cleanup_failed {
        findings.push(Finding::cleanup_failed(
            revision,
            stdio_location(StdioStream::Process),
        ));
    }

    findings
}

fn render_checks(checks: Vec<CheckResult>) -> Diagnostic {
    render_checks_for_revision(checks, SupportedRevision::CURRENT, None)
}

fn render_checks_for_revision(
    checks: Vec<CheckResult>,
    revision: SupportedRevision,
    negotiated_revision: Option<protocol::KnownRevision>,
) -> Diagnostic {
    let mut report = DiagnosticReport::new(revision, DiagnosticLimits::DEFAULTS, checks)
        .expect("the STDIO application must construct a valid diagnostic report");
    if let Some(negotiated_revision) = negotiated_revision {
        report = report.with_negotiated_revision(negotiated_revision);
    }

    Diagnostic::from_report(report)
}

fn stdio_location(stream: StdioStream) -> Location {
    let location = Location::root(LocationField::Process);
    match stream {
        StdioStream::Process => location,
        StdioStream::Stdin => location.field(LocationField::Stdin),
        StdioStream::Stdout => location.field(LocationField::Stdout),
        StdioStream::Stderr => location.field(LocationField::Stderr),
    }
}

fn limit_location(kind: StdioLimitKind) -> Location {
    match kind {
        StdioLimitKind::MessageBytes | StdioLimitKind::MessageCount => {
            stdio_location(StdioStream::Stdout).field(LocationField::Message)
        }
        StdioLimitKind::StdoutBytes => stdio_location(StdioStream::Stdout),
        StdioLimitKind::StderrBytes => stdio_location(StdioStream::Stderr),
        StdioLimitKind::AggregateOutputBytes
        | StdioLimitKind::StartupTime
        | StdioLimitKind::DiscoveryTime
        | StdioLimitKind::RequestTime
        | StdioLimitKind::ResponseTime
        | StdioLimitKind::TotalTime => stdio_location(StdioStream::Process),
    }
}

pub(super) fn success_exit() -> std::process::ExitCode {
    report::ExitStatus::Success.into()
}

#[cfg(test)]
mod transport_contract_tests {
    use super::{
        ReportFormat, ReportRequest, StdioDiagnostic, StdioLimitKind, StdioPrimaryFailure,
        http_diagnostic, render_http_diagnostic, render_stdio_diagnostic,
    };
    use crate::transport::http::HttpFailure;

    #[test]
    fn transport_and_cleanup_failures_remain_distinct_in_one_safe_report() {
        let rendered = render_stdio_diagnostic(StdioDiagnostic {
            primary: Some(StdioPrimaryFailure::InvalidMessage {
                byte_count: 37,
                index: 2,
            }),
            cleanup_failed: true,
        })
        .render(ReportRequest::stdout_only(ReportFormat::Human));

        assert!(rendered.output.contains("MCP-TRANSPORT-003"));
        assert!(rendered.output.contains("process.stdout.message[2]"));
        assert!(rendered.output.contains("observed [REDACTED] (37 bytes)"));
        assert!(rendered.output.contains("MCP-SAFETY-001"));
        assert!(rendered.output.contains("INDEPENDENT SAFETY FINDINGS · 1"));
        assert!(rendered.output.contains(
            "blocked by transport.stdio (MCP-TRANSPORT-003 at process.stdout.message[2])"
        ));
        assert!(rendered.output.contains("outcome failed · exit 1"));
    }

    #[test]
    fn transport_limits_use_the_canonical_kind_and_structural_location() {
        let rendered = render_stdio_diagnostic(StdioDiagnostic {
            primary: Some(StdioPrimaryFailure::Limit {
                kind: StdioLimitKind::StderrBytes,
                observed: 1_048_577,
                maximum: 1_048_576,
            }),
            cleanup_failed: false,
        })
        .render(ReportRequest::stdout_only(ReportFormat::Human));

        assert!(rendered.output.contains("MCP-LIMIT-001"));
        assert!(rendered.output.contains("process.stderr"));
        assert!(
            rendered
                .output
                .contains("stderr_bytes observed 1048577 bytes")
        );
        assert!(rendered.output.contains("maximum 1048576 bytes"));
    }

    #[test]
    fn a_pre_response_connection_failure_never_claims_tls_verification() {
        let rendered =
            render_http_diagnostic(http_diagnostic(Some(HttpFailure::Request), Some(true)))
                .render(ReportRequest::stdout_only(ReportFormat::Human));

        assert!(
            rendered
                .output
                .contains("PRIMARY DIAGNOSIS · network.resolution")
        );
        assert!(rendered.output.contains("MCP-HTTP-001"));
        assert!(rendered.output.contains("SKIP  transport.tls"));
        assert!(!rendered.output.contains("PASS  transport.tls"));
    }
}
