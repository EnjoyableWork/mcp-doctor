use std::ffi::OsString;

use crate::contract::{
    ActiveConversation, ActiveScenario, Diagnostic, ReportTransport, http_diagnostic,
    m1_http_limit_profile, m1_stdio_limit_profile, render_authorization_failure,
    render_generation_configuration_failure, stdio_diagnostic,
};
use crate::transport::http::{
    HttpLimits, HttpTarget, HttpTransport, RemoteOptions, SystemResolver,
};
use crate::transport::stdio::{StdioLimits, StdioTarget, StdioTransport, TargetError};

pub(crate) struct BreakOptions<'a> {
    pub(crate) tool: String,
    pub(crate) allowed_tool: &'a str,
    pub(crate) side_effecting: bool,
    pub(crate) allow_side_effects: bool,
    pub(crate) cases: usize,
    pub(crate) seed: u64,
}

pub(crate) async fn run_stdio(
    target: Vec<OsString>,
    options: BreakOptions<'_>,
) -> Result<Diagnostic, TargetError> {
    let mut scenario = match ActiveScenario::generated(
        options.tool,
        options.side_effecting,
        options.cases,
        options.seed,
    ) {
        Ok(scenario) => scenario,
        Err(failure) => {
            return Ok(render_generation_configuration_failure(
                failure,
                options.cases,
                ReportTransport::Stdio,
            ));
        }
    };
    if let Err(failure) = scenario.authorize(options.allowed_tool, options.allow_side_effects) {
        return Ok(render_authorization_failure(
            &scenario,
            failure,
            ReportTransport::Stdio,
        ));
    }
    scenario.discard_target_environment_names();

    let (executable, arguments) = target.split_first().expect("clap requires a break target");
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
    let mut conversation = ActiveConversation::new(scenario);
    let result = transport.probe(&target, &mut conversation).await;
    let diagnostic = stdio_diagnostic(
        result.failure(),
        result.cleanup_failed() || internal_test_cleanup_failure(),
    );
    Ok(conversation.into_diagnostic(diagnostic))
}

pub(crate) async fn run_http(remote: RemoteOptions, options: BreakOptions<'_>) -> Diagnostic {
    let mut scenario = match ActiveScenario::generated(
        options.tool,
        options.side_effecting,
        options.cases,
        options.seed,
    ) {
        Ok(scenario) => scenario,
        Err(failure) => {
            return render_generation_configuration_failure(
                failure,
                options.cases,
                ReportTransport::Http,
            );
        }
    };
    if let Err(failure) = scenario.authorize(options.allowed_tool, options.allow_side_effects) {
        return render_authorization_failure(&scenario, failure, ReportTransport::Http);
    }
    scenario.discard_target_environment_names();

    let mut conversation = ActiveConversation::new_http(scenario);
    let target = match HttpTarget::prepare(remote, http_limits(), &SystemResolver).await {
        Ok(target) => target,
        Err(failure) => {
            return conversation.into_http_diagnostic(http_diagnostic(Some(failure), None));
        }
    };
    let transport = match HttpTransport::new(target) {
        Ok(transport) => transport,
        Err(failure) => {
            return conversation.into_http_diagnostic(http_diagnostic(Some(failure), Some(true)));
        }
    };
    let result = transport.probe(&mut conversation).await;
    conversation.into_http_diagnostic(http_diagnostic(
        result.failure(),
        Some(result.tls_applicable()),
    ))
}

fn http_limits() -> HttpLimits {
    let profile = m1_http_limit_profile();
    HttpLimits {
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
