use std::ffi::OsString;

use crate::contract::{
    ActiveConversation, ActiveProtocolRevision, ActiveScenario, Diagnostic, DiagnosticLimitProfile,
    REJECTION_CASE_COUNT, ReportTransport, diagnostic_http_limit_profile,
    diagnostic_stdio_limit_profile, http_diagnostic, http_diagnostic_with_cleanup,
    render_authorization_failure_for_revision,
    render_generation_configuration_failure_for_revision, stdio_diagnostic,
};
use crate::status::{StatusCeiling, StatusCeilingKind, StatusObserver, StatusPhase};
use crate::transport::http::{
    HttpLimits, HttpTarget, HttpTransport, RemoteOptions, SystemResolver,
};
use crate::transport::stdio::{StdioLimits, StdioTarget, StdioTransport, TargetError};

pub(crate) struct RejectOptions<'a> {
    pub(crate) tool: String,
    pub(crate) allowed_tool: &'a str,
    pub(crate) side_effecting: bool,
    pub(crate) allow_side_effects: bool,
    pub(crate) seed: u64,
}

pub(crate) async fn run_stdio(
    target: Vec<OsString>,
    options: RejectOptions<'_>,
    status: &mut dyn StatusObserver,
) -> Result<Diagnostic, TargetError> {
    status.phase_started(StatusPhase::InputPreparation, None);
    let revision = ActiveProtocolRevision::CURRENT;
    let mut scenario =
        match ActiveScenario::rejection(options.tool, options.side_effecting, options.seed) {
            Ok(scenario) => scenario,
            Err(failure) => {
                return Ok(render_generation_configuration_failure_for_revision(
                    failure,
                    REJECTION_CASE_COUNT,
                    ReportTransport::Stdio,
                    revision,
                ));
            }
        };
    if let Err(failure) = scenario.authorize(options.allowed_tool, options.allow_side_effects) {
        return Ok(render_authorization_failure_for_revision(
            &scenario,
            failure,
            ReportTransport::Stdio,
            revision,
        ));
    }
    scenario.discard_target_environment_names();

    status.phase_started(StatusPhase::TargetPreparation, None);
    let (executable, arguments) = target.split_first().expect("clap requires a reject target");
    let target = StdioTarget::new(executable.clone(), arguments.to_vec())?;
    let profile = diagnostic_stdio_limit_profile(DiagnosticLimitProfile::Default);
    let transport = StdioTransport::new_for_active_protocol(
        StdioLimits {
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
        },
        false,
    );
    let mut conversation = ActiveConversation::for_revision(scenario, revision);
    let result = transport
        .probe_with_status(&target, &mut conversation, status)
        .await;
    let diagnostic = stdio_diagnostic(
        result.failure(),
        result.cleanup_failed() || internal_test_cleanup_failure(),
    );
    Ok(conversation.into_diagnostic(diagnostic))
}

pub(crate) async fn run_http(
    remote: RemoteOptions,
    options: RejectOptions<'_>,
    status: &mut dyn StatusObserver,
) -> Diagnostic {
    status.phase_started(StatusPhase::InputPreparation, None);
    let revision = ActiveProtocolRevision::CURRENT;
    let mut scenario =
        match ActiveScenario::rejection(options.tool, options.side_effecting, options.seed) {
            Ok(scenario) => scenario,
            Err(failure) => {
                return render_generation_configuration_failure_for_revision(
                    failure,
                    REJECTION_CASE_COUNT,
                    ReportTransport::Http,
                    revision,
                );
            }
        };
    if let Err(failure) = scenario.authorize(options.allowed_tool, options.allow_side_effects) {
        return render_authorization_failure_for_revision(
            &scenario,
            failure,
            ReportTransport::Http,
            revision,
        );
    }
    scenario.discard_target_environment_names();

    let mut conversation = ActiveConversation::new_http_for_revision(scenario, revision);
    let limits = http_limits();
    status.phase_started(
        StatusPhase::TargetPreparation,
        Some(StatusCeiling {
            kind: StatusCeilingKind::Startup,
            milliseconds: limits.startup_ms,
        }),
    );
    let target = match HttpTarget::prepare(remote, limits, &SystemResolver).await {
        Ok(target) => target,
        Err(failure) => {
            return conversation.into_http_diagnostic(http_diagnostic(Some(failure), None));
        }
    };
    let transport = match HttpTransport::new_for_active_protocol(target, revision.as_str(), false) {
        Ok(transport) => transport,
        Err(failure) => {
            return conversation.into_http_diagnostic(http_diagnostic(Some(failure), Some(true)));
        }
    };
    let result = transport.probe_with_status(&mut conversation, status).await;
    conversation.into_http_diagnostic(http_diagnostic_with_cleanup(
        result.failure(),
        Some(result.tls_applicable()),
        result.session_cleanup_failed(),
    ))
}

fn http_limits() -> HttpLimits {
    let profile = diagnostic_http_limit_profile(DiagnosticLimitProfile::Default);
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
        protocol_revisions: profile.protocol_revisions,
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
