use std::ffi::OsString;

use crate::contract::{
    RenderedDiagnostic, StdioDiagnostic, StdioLimitKind, StdioPrimaryFailure,
    StdioStream as ContractStream, m1_stdio_limit_profile, render_stdio_diagnostic,
};
use crate::transport::stdio::{
    StdioFailure, StdioLimit, StdioLimits, StdioStream, StdioTarget, StdioTransport, TargetError,
};

pub(crate) async fn run(target: Vec<OsString>) -> Result<RenderedDiagnostic, TargetError> {
    let (executable, arguments) = target
        .split_first()
        .expect("clap requires an inspect target");
    let target = StdioTarget::new(executable.clone(), arguments.to_vec())?;
    let profile = m1_stdio_limit_profile();
    let transport = StdioTransport::new(StdioLimits {
        startup_ms: profile.startup_ms,
        discovery_ms: profile.discovery_ms,
        request_ms: profile.request_ms,
        response_ms: profile.response_ms,
        shutdown_grace_ms: profile.shutdown_grace_ms,
        total_ms: profile.total_ms,
        message_bytes: profile.message_bytes,
        stdout_bytes: profile.stdout_bytes,
        stderr_bytes: profile.stderr_bytes,
        aggregate_output_bytes: profile.aggregate_output_bytes,
        message_count: profile.message_count,
    });
    let result = transport.probe(&target).await;

    debug_assert!(result.failure().is_some() || result.response().is_some());
    Ok(render_stdio_diagnostic(StdioDiagnostic {
        primary: result.failure().map(map_failure),
        cleanup_failed: result.cleanup_failed(),
    }))
}

fn map_failure(failure: StdioFailure) -> StdioPrimaryFailure {
    match failure {
        StdioFailure::ProcessStart => StdioPrimaryFailure::ProcessStart,
        StdioFailure::Io { stream } => StdioPrimaryFailure::Io {
            stream: map_stream(stream),
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
            kind: map_limit(kind),
            observed,
            maximum,
        },
    }
}

const fn map_stream(stream: StdioStream) -> ContractStream {
    match stream {
        StdioStream::Process => ContractStream::Process,
        StdioStream::Stdin => ContractStream::Stdin,
        StdioStream::Stdout => ContractStream::Stdout,
        StdioStream::Stderr => ContractStream::Stderr,
    }
}

const fn map_limit(limit: StdioLimit) -> StdioLimitKind {
    match limit {
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
    }
}
