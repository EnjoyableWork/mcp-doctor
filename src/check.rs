use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Read as _;
use std::path::Path;

use crate::contract::{
    ActiveConversation, ActiveScenario, MAX_SCENARIO_BYTES, RenderedDiagnostic, ReportFormat,
    ScenarioFailure, m1_stdio_limit_profile, render_authorization_failure,
    render_resolved_scenario_failure, render_scenario_failure, resolve_target_environment,
    stdio_diagnostic,
};
use crate::transport::stdio::{StdioLimits, StdioTarget, StdioTransport, TargetError};

pub(crate) async fn run(
    target: Vec<OsString>,
    scenario_path: &Path,
    allowed_tool: &str,
    allow_side_effects: bool,
    format: ReportFormat,
) -> Result<RenderedDiagnostic, TargetError> {
    let bytes = match read_scenario(scenario_path) {
        Ok(bytes) => bytes,
        Err(failure) => return Ok(render_scenario_failure(failure, format)),
    };
    let mut scenario = match ActiveScenario::parse(&bytes) {
        Ok(scenario) => scenario,
        Err(failure) => return Ok(render_scenario_failure(failure, format)),
    };
    if let Err(failure) = scenario.authorize(allowed_tool, allow_side_effects) {
        return Ok(render_authorization_failure(&scenario, failure, format));
    }

    let target_environment =
        match resolve_target_environment(&scenario, |name| std::env::var_os(name)) {
            Ok(environment) => environment,
            Err(failure) => {
                return Ok(render_resolved_scenario_failure(&scenario, failure, format));
            }
        };
    if let Err(failure) = scenario.resolve_argument_secrets(|name| std::env::var(name).ok()) {
        return Ok(render_resolved_scenario_failure(&scenario, failure, format));
    }
    scenario.discard_target_environment_names();

    let (executable, arguments) = target.split_first().expect("clap requires a check target");
    let target =
        StdioTarget::with_environment(executable.clone(), arguments.to_vec(), target_environment)?;
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
    Ok(conversation.into_diagnostic(diagnostic, format))
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
