use std::ffi::OsString;

use crate::contract::{
    PassiveCatalogConversation, RenderedDiagnostic, ReportFormat, m1_stdio_limit_profile,
    render_catalog_diagnostic, render_stdio_diagnostic, stdio_diagnostic,
};
use crate::transport::stdio::{StdioLimits, StdioTarget, StdioTransport, TargetError};

pub(crate) async fn run(
    target: Vec<OsString>,
    format: ReportFormat,
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
    let mut conversation = PassiveCatalogConversation::new();
    let result = transport.probe(&target, &mut conversation).await;

    debug_assert!(result.failure().is_some() || result.response().is_some());
    let diagnostic = stdio_diagnostic(
        result.failure(),
        result.cleanup_failed() || internal_test_cleanup_failure(),
    );
    if result.failure().is_some() {
        Ok(render_stdio_diagnostic(diagnostic, format))
    } else {
        Ok(render_catalog_diagnostic(
            diagnostic,
            &conversation,
            result.responses(),
            format,
        ))
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
