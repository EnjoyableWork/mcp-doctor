use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Read as _;
use std::path::Path;

use crate::contract::{
    ActiveConversation, ActiveProtocolRevision, ActiveScenario, Diagnostic, DiagnosticLimitProfile,
    MAX_SCENARIO_BYTES, ReportTransport, ScenarioFailure, diagnostic_http_limit_profile,
    diagnostic_stdio_limit_profile, http_diagnostic, http_diagnostic_with_cleanup,
    render_authorization_failure_for_revision, render_resolved_scenario_failure_for_revision,
    render_scenario_failure_for_revision, resolve_target_environment, stdio_diagnostic,
};
use crate::transport::http::{
    HttpLimits, HttpTarget, HttpTransport, RemoteOptions, SystemResolver,
};
use crate::transport::stdio::{StdioLimits, StdioTarget, StdioTransport, TargetError};

pub(crate) async fn run_stdio(
    target: Vec<OsString>,
    scenario_path: &Path,
    allowed_tool: &str,
    allow_side_effects: bool,
    revision: ActiveProtocolRevision,
    limit_profile: DiagnosticLimitProfile,
) -> Result<Diagnostic, TargetError> {
    let bytes = match read_scenario(scenario_path) {
        Ok(bytes) => bytes,
        Err(failure) => {
            return Ok(render_scenario_failure_for_revision(
                failure,
                ReportTransport::Stdio,
                revision,
            ));
        }
    };
    let mut scenario = match ActiveScenario::parse(&bytes) {
        Ok(scenario) => scenario,
        Err(failure) => {
            return Ok(render_scenario_failure_for_revision(
                failure,
                ReportTransport::Stdio,
                revision,
            ));
        }
    };
    if let Err(failure) = scenario.authorize(allowed_tool, allow_side_effects) {
        return Ok(render_authorization_failure_for_revision(
            &scenario,
            failure,
            ReportTransport::Stdio,
            revision,
        ));
    }

    let target_environment =
        match resolve_target_environment(&scenario, |name| std::env::var_os(name)) {
            Ok(environment) => environment,
            Err(failure) => {
                return Ok(render_resolved_scenario_failure_for_revision(
                    &scenario,
                    failure,
                    ReportTransport::Stdio,
                    revision,
                ));
            }
        };
    if let Err(failure) = scenario.resolve_argument_secrets(|name| std::env::var(name).ok()) {
        return Ok(render_resolved_scenario_failure_for_revision(
            &scenario,
            failure,
            ReportTransport::Stdio,
            revision,
        ));
    }
    scenario.discard_target_environment_names();

    let (executable, arguments) = target.split_first().expect("clap requires a check target");
    let target =
        StdioTarget::with_environment(executable.clone(), arguments.to_vec(), target_environment)?;
    let profile = diagnostic_stdio_limit_profile(limit_profile);
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
        revision.uses_initialize(),
    );
    let mut conversation = ActiveConversation::for_revision(scenario, revision);
    let result = transport.probe(&target, &mut conversation).await;
    let diagnostic = stdio_diagnostic(
        result.failure(),
        result.cleanup_failed() || internal_test_cleanup_failure(),
    );
    Ok(conversation.into_diagnostic(diagnostic))
}

pub(crate) async fn run_http(
    options: RemoteOptions,
    scenario_path: &Path,
    allowed_tool: &str,
    allow_side_effects: bool,
    revision: ActiveProtocolRevision,
    limit_profile: DiagnosticLimitProfile,
) -> Diagnostic {
    let bytes = match read_scenario(scenario_path) {
        Ok(bytes) => bytes,
        Err(failure) => {
            return render_scenario_failure_for_revision(failure, ReportTransport::Http, revision);
        }
    };
    let mut scenario = match ActiveScenario::parse(&bytes) {
        Ok(scenario) => scenario,
        Err(failure) => {
            return render_scenario_failure_for_revision(failure, ReportTransport::Http, revision);
        }
    };
    if let Err(failure) = scenario.authorize(allowed_tool, allow_side_effects) {
        return render_authorization_failure_for_revision(
            &scenario,
            failure,
            ReportTransport::Http,
            revision,
        );
    }
    if let Err(failure) = scenario.reject_remote_target_environment() {
        return render_resolved_scenario_failure_for_revision(
            &scenario,
            failure,
            ReportTransport::Http,
            revision,
        );
    }
    if let Err(failure) = scenario.resolve_argument_secrets(|name| std::env::var(name).ok()) {
        return render_resolved_scenario_failure_for_revision(
            &scenario,
            failure,
            ReportTransport::Http,
            revision,
        );
    }
    scenario.discard_target_environment_names();

    let mut conversation = ActiveConversation::new_http_for_revision(scenario, revision);
    let target =
        match HttpTarget::prepare(options, http_limits(limit_profile), &SystemResolver).await {
            Ok(target) => target,
            Err(failure) => {
                return conversation.into_http_diagnostic(http_diagnostic(Some(failure), None));
            }
        };
    let transport = match HttpTransport::new_for_active_protocol(
        target,
        revision.as_str(),
        revision.uses_initialize(),
    ) {
        Ok(transport) => transport,
        Err(failure) => {
            return conversation.into_http_diagnostic(http_diagnostic(Some(failure), Some(true)));
        }
    };
    let result = transport.probe(&mut conversation).await;
    conversation.into_http_diagnostic(http_diagnostic_with_cleanup(
        result.failure(),
        Some(result.tls_applicable()),
        result.session_cleanup_failed(),
    ))
}

fn http_limits(selected: DiagnosticLimitProfile) -> HttpLimits {
    let profile = diagnostic_http_limit_profile(selected);
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

fn read_scenario(path: &Path) -> Result<Vec<u8>, ScenarioFailure> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ScenarioFailure::unreadable())?;
    if !metadata.file_type().is_file() {
        return Err(ScenarioFailure::unreadable());
    }
    if metadata.len() > MAX_SCENARIO_BYTES {
        return Err(ScenarioFailure::file_limit(metadata.len()));
    }
    let file = File::open(path).map_err(|_| ScenarioFailure::unreadable())?;
    let mut bytes = Vec::new();
    file.take(MAX_SCENARIO_BYTES.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| ScenarioFailure::unreadable())?;
    let observed = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if observed > MAX_SCENARIO_BYTES {
        return Err(ScenarioFailure::file_limit(observed));
    }
    Ok(bytes)
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
