#![cfg(feature = "internal-test-fixtures")]

mod support;

use std::ffi::OsStr;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant};

use support::TestEnvironment;

const REDACTION_SENTINEL: &str = "synthetic-secret-payload-7f2c";
const CATALOG_SENTINEL: &str = "synthetic-secret-payload-never-report-7f2c";

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

#[test]
fn valid_paginated_catalogs_and_complex_local_schemas_pass_passively() {
    let output = run_mode("catalog-valid");
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    for check in [
        "PASS  transport.stdio",
        "PASS  protocol.revision",
        "PASS  protocol.envelope",
        "PASS  discovery.catalogs",
        "PASS  schema.contracts",
        "SKIP  runtime.tools",
    ] {
        assert!(stdout.contains(check), "{stdout}");
    }
    assert!(stdout.contains("outcome passed"), "{stdout}");
    assert!(!stdout.contains("synthetic-private-cursor"), "{stdout}");
    assert!(!stdout.contains("tools/call"), "{stdout}");
}

#[test]
fn invalid_catalog_is_deterministic_redacted_and_actionable() {
    let first = run_mode("catalog-invalid");
    let second = run_mode("catalog-invalid");
    let (stdout, stderr) = text(&first);

    assert_eq!(first.status.code(), Some(1), "{stdout}\n{stderr}");
    assert_eq!(
        first.stdout, second.stdout,
        "diagnostics must be deterministic"
    );
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("MCP-CATALOG-001"), "{stdout}");
    assert!(stdout.contains("prompts[0].arguments"), "{stdout}");
    assert!(
        stdout.contains("expected array · observed string"),
        "{stdout}"
    );
    assert!(stdout.contains("Expected:"), "{stdout}");
    assert!(stdout.contains("Fix:"), "{stdout}");
    assert!(stdout.contains("Reference:"), "{stdout}");
    assert!(!stdout.contains(CATALOG_SENTINEL), "{stdout}");
    assert!(!stdout.contains("synthetic-private-prompt"), "{stdout}");
}

#[test]
fn duplicate_catalog_identifiers_are_reported_without_echoing_them() {
    let output = run_mode("catalog-duplicate");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("MCP-CATALOG-002"), "{stdout}");
    assert!(stdout.contains("prompts[1].name"), "{stdout}");
    assert!(stdout.contains("Rename or remove"), "{stdout}");
    assert!(!stdout.contains("synthetic-private-duplicate"), "{stdout}");
}

#[test]
fn repeated_pagination_cursor_stops_without_disclosing_the_cursor() {
    let output = run_mode("catalog-repeated-cursor");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("MCP-CATALOG-003"), "{stdout}");
    assert!(stdout.contains("prompts.nextCursor"), "{stdout}");
    assert!(stdout.contains("omit nextCursor"), "{stdout}");
    assert!(!stdout.contains("synthetic-private-repeated"), "{stdout}");
}

#[test]
fn invalid_resources_and_templates_use_safe_structural_locations() {
    let output = run_mode("catalog-invalid-resources");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("resources[0].uri"), "{stdout}");
    assert!(
        stdout.contains("resourceTemplates[0].uriTemplate"),
        "{stdout}"
    );
    assert!(stdout.contains("observed object"), "{stdout}");
    assert!(stdout.contains("observed boolean"), "{stdout}");
    assert!(!stdout.contains("synthetic-private-resource"), "{stdout}");
    assert!(!stdout.contains("synthetic-secret-resource"), "{stdout}");
    assert!(!stdout.contains("synthetic-private-template"), "{stdout}");
}

#[test]
fn invalid_and_unsupported_tool_schemas_have_distinct_corrections() {
    let output = run_mode("schema-invalid");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("MCP-SCHEMA-001"), "{stdout}");
    assert!(stdout.contains("MCP-SCHEMA-002"), "{stdout}");
    assert!(stdout.contains("tools[0].inputSchema.required"), "{stdout}");
    assert!(stdout.contains("tools[1].inputSchema.$schema"), "{stdout}");
    assert!(stdout.contains("tools[2].inputSchema.type"), "{stdout}");
    assert!(stdout.contains("tools[3].inputSchema.$ref"), "{stdout}");
    assert!(stdout.contains("unresolved_local_reference"), "{stdout}");
    assert!(stdout.contains("Draft 2020-12"), "{stdout}");
    assert!(!stdout.contains("synthetic-secret-required"), "{stdout}");
}

#[test]
fn external_schema_references_are_rejected_without_retrieval() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a disposable listener should bind");
    listener
        .set_nonblocking(true)
        .expect("the disposable listener should be nonblocking");
    let address = listener
        .local_addr()
        .expect("the disposable listener should have an address");
    let sentinel = "synthetic-external-reference-never-report-7f2c";
    let reference = format!("http://{address}/{sentinel}");
    let environment = TestEnvironment::new();
    let output = inspect_command(&environment, "schema-external")
        .arg(&reference)
        .output()
        .expect("mcp-doctor should inspect the external-reference fixture");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("MCP-SCHEMA-003"), "{stdout}");
    assert!(
        stdout.contains("tools[0].inputSchema.properties[*].$ref"),
        "{stdout}"
    );
    assert!(stdout.contains("local $defs"), "{stdout}");
    assert!(!stdout.contains(sentinel), "{stdout}");
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == ErrorKind::WouldBlock),
        "schema validation must not connect to an external reference"
    );
}

#[test]
fn schema_depth_and_catalog_item_bounds_stop_with_named_findings() {
    for (mode, limit) in [
        ("schema-depth-limit", "schema_depth"),
        ("schema-node-limit", "schema_nodes"),
        ("schema-ref-depth-limit", "schema_ref_depth"),
        ("schema-evaluation-limit", "schema_evaluation_steps"),
        ("schema-error-limit", "validation_errors"),
        ("catalog-item-limit", "catalog_items"),
        ("report-finding-limit", "report_findings"),
    ] {
        let output = run_mode(mode);
        let (stdout, stderr) = text(&output);

        assert_eq!(output.status.code(), Some(1), "{mode}: {stdout}\n{stderr}");
        assert!(stderr.is_empty(), "{mode}: {stderr}");
        assert!(stdout.contains("MCP-LIMIT-001"), "{mode}: {stdout}");
        assert!(stdout.contains(limit), "{mode}: {stdout}");
        assert!(stdout.contains("maximum"), "{mode}: {stdout}");
        assert!(!stdout.contains("synthetic-private-property"), "{stdout}");
    }
}

#[test]
fn report_finding_limit_does_not_fire_at_the_exact_maximum() {
    let output = run_mode("report-finding-exact");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("MCP-CATALOG-001"), "{stdout}");
    assert!(!stdout.contains("report_findings observed"), "{stdout}");
}

fn contains_path(output: &str, path: &Path) -> bool {
    path.to_str().is_some_and(|path| output.contains(path))
        || path
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| output.contains(name))
}
