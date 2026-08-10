#![cfg(feature = "internal-test-fixtures")]

mod support;

use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant};

use support::TestEnvironment;

const REDACTION_SENTINEL: &str = "synthetic-secret-payload-7f2c";

fn fixture() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_mcp-doctor-stdio-fixture"))
}

fn inspect_command(environment: &TestEnvironment, mode: &str) -> Command {
    let mut command = environment.command();
    command.arg("inspect").arg("--").arg(fixture()).arg(mode);
    command
}

fn run_mode(mode: &str) -> Output {
    let environment = TestEnvironment::new();
    inspect_command(&environment, mode)
        .output()
        .expect("mcp-doctor should inspect the fixture")
}

fn text(output: &Output) -> (&str, &str) {
    let stdout = std::str::from_utf8(&output.stdout).expect("STDOUT should be UTF-8");
    let stderr = std::str::from_utf8(&output.stderr).expect("STDERR should be UTF-8");
    (stdout, stderr)
}

#[test]
fn successful_inspection_is_passive_and_cleans_up() {
    let environment = TestEnvironment::new();
    let unexpected_request = environment.artifact_path("unexpected-request");
    let output = inspect_command(&environment, "success")
        .arg(&unexpected_request)
        .output()
        .expect("mcp-doctor should inspect the conforming fixture");
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("PASS  transport.stdio"), "{stdout}");
    assert!(stdout.contains("outcome passed"), "{stdout}");
    assert!(
        !unexpected_request.exists(),
        "no second request is permitted"
    );
}

#[test]
fn target_arguments_are_passed_literally_without_expansion() {
    let environment = TestEnvironment::new();
    let output = inspect_command(&environment, "literal-arguments")
        .arg("space value")
        .arg("$MCP_DOCTOR_LITERAL")
        .arg("; synthetic-command")
        .arg("$(synthetic-command)")
        .env("MCP_DOCTOR_LITERAL", "expanded-value-must-not-appear")
        .output()
        .expect("mcp-doctor should pass literal fixture arguments");
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(!stdout.contains("expanded-value-must-not-appear"));
}

#[test]
fn target_environment_excludes_user_and_secret_values() {
    let environment = TestEnvironment::new();
    let output = inspect_command(&environment, "environment")
        .env("MCP_DOCTOR_ENV_SENTINEL", "secret-must-not-be-inherited")
        .output()
        .expect("mcp-doctor should constrain the fixture environment");
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(!stdout.contains("secret-must-not-be-inherited"));
}

#[test]
fn malformed_server_output_is_distinct_and_redacted() {
    let output = run_mode("malformed");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-TRANSPORT-003"), "{stdout}");
    assert!(stdout.contains("[REDACTED]"), "{stdout}");
    assert!(!stdout.contains(REDACTION_SENTINEL), "{stdout}");
}

#[test]
fn every_stdio_output_boundary_fails_at_its_named_limit() {
    for (mode, limit) in [
        ("oversized-message", "message_bytes"),
        ("stdout-oversize", "stdout_bytes"),
        ("stderr-oversize", "stderr_bytes"),
        ("aggregate-oversize", "aggregate_output_bytes"),
        ("message-count", "message_count"),
    ] {
        let output = run_mode(mode);
        let (stdout, stderr) = text(&output);

        assert_eq!(output.status.code(), Some(1), "{mode}: {stdout}\n{stderr}");
        assert!(stderr.is_empty(), "{mode}: {stderr}");
        assert!(stdout.contains("MCP-LIMIT-001"), "{mode}: {stdout}");
        assert!(stdout.contains(limit), "{mode}: {stdout}");
        assert!(!stdout.contains(REDACTION_SENTINEL), "{mode}: {stdout}");
    }
}

#[test]
fn an_unresponsive_server_hits_the_discovery_deadline() {
    let started = Instant::now();
    let output = run_mode("timeout");
    let elapsed = started.elapsed();
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-LIMIT-001"), "{stdout}");
    assert!(stdout.contains("discovery_time"), "{stdout}");
    assert!(elapsed >= Duration::from_secs(9), "elapsed: {elapsed:?}");
    assert!(elapsed < Duration::from_secs(20), "elapsed: {elapsed:?}");
}

#[test]
fn an_early_exit_has_a_distinct_transport_finding() {
    let output = run_mode("early-exit");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-TRANSPORT-004"), "{stdout}");
}

#[test]
fn cleanup_terminates_a_resistant_process_tree_before_returning() {
    let environment = TestEnvironment::new();
    let survival_marker = environment.artifact_path("descendant-survived");
    let started = Instant::now();
    let output = inspect_command(&environment, "resistant-child")
        .arg(&survival_marker)
        .output()
        .expect("mcp-doctor should inspect the resistant fixture");
    let elapsed = started.elapsed();
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(
        elapsed >= Duration::from_millis(1_800),
        "elapsed: {elapsed:?}"
    );
    assert!(elapsed < Duration::from_secs(8), "elapsed: {elapsed:?}");

    thread::sleep(Duration::from_secs(2));
    assert!(
        !survival_marker.exists(),
        "the descendant survived process-tree cleanup"
    );
}

#[test]
fn process_start_failures_do_not_reveal_the_target() {
    let environment = TestEnvironment::new();
    let missing = environment.artifact_path("synthetic-secret-missing-target-7f2c");
    let output = environment
        .command()
        .arg("inspect")
        .arg("--")
        .arg(&missing)
        .output()
        .expect("mcp-doctor should report a missing target");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-TRANSPORT-001"), "{stdout}");
    assert!(!contains_path(stdout, &missing), "{stdout}");
}

fn contains_path(output: &str, path: &Path) -> bool {
    path.to_str().is_some_and(|path| output.contains(path))
        || path
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| output.contains(name))
}
