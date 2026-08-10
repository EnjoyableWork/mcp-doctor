#![allow(
    dead_code,
    reason = "the M1 contract includes checks consumed by later ordered tickets"
)]

mod limits;
mod model;
mod protocol;
mod redaction;
mod report;

use limits::{DiagnosticLimits, LimitKind, LimitViolation};
use model::{CheckId, CheckResult, Finding, Location, LocationField, Requirement};
use protocol::SupportedRevision;
use redaction::RedactedValue;
use report::{DiagnosticReport, HumanReporter};

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

pub(crate) fn m1_stdio_limit_profile() -> StdioLimitProfile {
    let values = DiagnosticLimits::M1_DEFAULTS.values();
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

pub(crate) struct RenderedDiagnostic {
    pub(crate) output: String,
    pub(crate) exit: std::process::ExitCode,
}

pub(crate) fn render_stdio_diagnostic(diagnostic: StdioDiagnostic) -> RenderedDiagnostic {
    let revision = SupportedRevision::CURRENT;
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

    let report = DiagnosticReport::new(
        revision,
        DiagnosticLimits::M1_DEFAULTS,
        vec![CheckResult::performed(
            CheckId::TransportStdio,
            Requirement::Required,
            findings,
        )],
    )
    .expect("the STDIO application must construct a valid diagnostic report");

    RenderedDiagnostic {
        output: HumanReporter::render(&report),
        exit: report.exit_status().into(),
    }
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
mod stdio_contract_tests {
    use super::{StdioDiagnostic, StdioLimitKind, StdioPrimaryFailure, render_stdio_diagnostic};

    #[test]
    fn transport_and_cleanup_failures_remain_distinct_in_one_safe_report() {
        let rendered = render_stdio_diagnostic(StdioDiagnostic {
            primary: Some(StdioPrimaryFailure::InvalidMessage {
                byte_count: 37,
                index: 2,
            }),
            cleanup_failed: true,
        });

        assert!(rendered.output.contains("MCP-TRANSPORT-003"));
        assert!(rendered.output.contains("process.stdout.message[2]"));
        assert!(rendered.output.contains("observed [REDACTED] (37 bytes)"));
        assert!(rendered.output.contains("MCP-SAFETY-001"));
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
        });

        assert!(rendered.output.contains("MCP-LIMIT-001"));
        assert!(rendered.output.contains("process.stderr"));
        assert!(
            rendered
                .output
                .contains("stderr_bytes observed 1048577 bytes")
        );
        assert!(rendered.output.contains("maximum 1048576 bytes"));
    }
}
