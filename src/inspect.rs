use std::ffi::OsString;

use crate::contract::{
    PassiveCatalogConversation, ProtocolRevision, RenderedDiagnostic, ReportFormat,
    http_diagnostic, http_diagnostic_with_cleanup, m1_http_limit_profile, m1_stdio_limit_profile,
    render_catalog_diagnostic, render_http_catalog_diagnostic, render_http_diagnostic_for_revision,
    render_http_diagnostic_for_revision_with_negotiated, render_stdio_diagnostic_for_revision,
    stdio_diagnostic,
};
use crate::transport::http::{
    HttpLimits, HttpTarget, HttpTransport, RemoteOptions, SystemResolver,
};
use crate::transport::stdio::{StdioLimits, StdioTarget, StdioTransport, TargetError};

pub(crate) async fn run_stdio(
    target: Vec<OsString>,
    format: ReportFormat,
    revision: ProtocolRevision,
) -> Result<RenderedDiagnostic, TargetError> {
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
    let mut conversation = PassiveCatalogConversation::for_revision(revision);
    let result = transport.probe(&target, &mut conversation).await;

    debug_assert!(result.failure().is_some() || result.response().is_some());
    let diagnostic = stdio_diagnostic(
        result.failure(),
        result.cleanup_failed() || internal_test_cleanup_failure(),
    );
    if result.failure().is_some() {
        Ok(render_stdio_diagnostic_for_revision(
            diagnostic, format, revision,
        ))
    } else {
        Ok(render_catalog_diagnostic(
            diagnostic,
            &conversation,
            result.responses(),
            format,
        ))
    }
}

pub(crate) async fn run_http(
    options: RemoteOptions,
    format: ReportFormat,
    revision: ProtocolRevision,
) -> RenderedDiagnostic {
    let profile = m1_http_limit_profile();
    let limits = HttpLimits {
        startup_ms: profile.startup_ms,
        discovery_ms: profile.discovery_ms,
        request_ms: profile.request_ms,
        response_ms: profile.response_ms,
        shutdown_grace_ms: profile.shutdown_grace_ms,
        total_ms: profile.total_ms,
        endpoint_bytes: profile.endpoint_bytes,
        resolution_addresses: profile.resolution_addresses,
        trust_bytes: profile.trust_bytes,
        trust_certificates: profile.trust_certificates,
        request_fields: profile.request_fields,
        request_field_name_bytes: profile.request_field_name_bytes,
        request_field_value_bytes: profile.request_field_value_bytes,
        request_fields_bytes: profile.request_fields_bytes,
        response_fields: profile.response_fields,
        response_field_name_bytes: profile.response_field_name_bytes,
        response_field_value_bytes: profile.response_field_value_bytes,
        response_fields_bytes: profile.response_fields_bytes,
        message_bytes: profile.message_bytes,
        aggregate_output_bytes: profile.aggregate_output_bytes,
        message_count: profile.message_count,
    };
    let target = match HttpTarget::prepare(options, limits, &SystemResolver).await {
        Ok(target) => target,
        Err(failure) => {
            return render_http_diagnostic_for_revision(
                http_diagnostic(Some(failure), None),
                format,
                revision,
            );
        }
    };
    let transport = match HttpTransport::new_for_protocol(
        target,
        revision.as_str(),
        revision.uses_initialize(),
    ) {
        Ok(transport) => transport,
        Err(failure) => {
            return render_http_diagnostic_for_revision(
                http_diagnostic(Some(failure), Some(true)),
                format,
                revision,
            );
        }
    };
    let mut conversation = PassiveCatalogConversation::new_http_for_revision(revision);
    let result = transport.probe(&mut conversation).await;
    let diagnostic = http_diagnostic_with_cleanup(
        result.failure(),
        Some(result.tls_applicable()),
        result.session_cleanup_failed(),
    );
    if result.failure().is_some() {
        render_http_diagnostic_for_revision_with_negotiated(
            diagnostic,
            format,
            revision,
            conversation.negotiated_revision(),
        )
    } else {
        render_http_catalog_diagnostic(diagnostic, &conversation, result.responses(), format)
    }
}

#[cfg(feature = "internal-test-fixtures")]
fn internal_test_cleanup_failure() -> bool {
    std::env::var_os("MCP_DOCTOR_TEST_MODE").as_deref() == Some(std::ffi::OsStr::new("1"))
        && std::env::var_os("MCP_DOCTOR_INTERNAL_TEST_CLEANUP_FAILURE").as_deref()
            == Some(std::ffi::OsStr::new("1"))
}

#[cfg(not(feature = "internal-test-fixtures"))]
const fn internal_test_cleanup_failure() -> bool {
    false
}
