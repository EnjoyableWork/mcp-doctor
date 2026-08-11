#![cfg(feature = "internal-test-fixtures")]

mod support;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use support::{TestEnvironment, parse_and_validate_junit, parse_and_validate_report};

const TOOL: &str = "synthetic.reviewed";
const SECRET_VALUE: &str = "synthetic-secret-payload-7f2c";
const ARGUMENT_SECRET_NAME: &str = "SYNTHETIC_TOOL_SECRET_7F2C";
const TARGET_SECRET_NAME: &str = "ACTIVE_TARGET_SECRET";

fn fixture() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_mcp-doctor-stdio-fixture"))
}

fn write_scenario(
    environment: &TestEnvironment,
    name: &str,
    scenario: &Value,
) -> std::path::PathBuf {
    let path = environment.artifact_path(name);
    fs::write(
        &path,
        serde_json::to_vec_pretty(scenario).expect("the synthetic scenario should serialize"),
    )
    .expect("the synthetic scenario should be writable");
    path
}

fn scenario(effects: &str, cases: Vec<Value>) -> Value {
    json!({
        "schema_version": "mcp-doctor.scenario/v1alpha1",
        "tool": TOOL,
        "safety": {"effects": effects},
        "cases": cases
    })
}

fn reviewed_case(sequence: i64, result: &str) -> Value {
    json!({
        "id": format!("author-only-case-{sequence}-never-report"),
        "arguments": {"sequence": sequence},
        "expect": {
            "result": result,
            "structured_output_schema": {
                "type": "object",
                "properties": {"ok": {"type": "boolean"}},
                "required": ["ok"],
                "additionalProperties": false
            }
        }
    })
}

fn check_command(
    environment: &TestEnvironment,
    scenario_path: &Path,
    allowed_tool: &str,
    mode: &str,
) -> Command {
    let mut command = environment.command();
    command
        .arg("check")
        .arg("--scenario")
        .arg(scenario_path)
        .arg("--allow-tool")
        .arg(allowed_tool)
        .arg("--")
        .arg(fixture())
        .arg(mode);
    command
}

fn run_check(scenario: Value, mode: &str) -> Output {
    let environment = TestEnvironment::new();
    let scenario_path = write_scenario(&environment, "scenario.json", &scenario);
    check_command(&environment, &scenario_path, TOOL, mode)
        .output()
        .expect("mcp-doctor check should start")
}

fn text(output: &Output) -> (&str, &str) {
    let stdout = std::str::from_utf8(&output.stdout).expect("STDOUT should be UTF-8");
    let stderr = std::str::from_utf8(&output.stderr).expect("STDERR should be UTF-8");
    (stdout, stderr)
}

fn assert_redacted(output: &Output, extra: &[&str]) {
    let (stdout, stderr) = text(output);
    for forbidden in [SECRET_VALUE, ARGUMENT_SECRET_NAME, TARGET_SECRET_NAME]
        .into_iter()
        .chain(extra.iter().copied())
    {
        assert!(
            !stdout.contains(forbidden),
            "STDOUT disclosed protected data"
        );
        assert!(
            !stderr.contains(forbidden),
            "STDERR disclosed protected data"
        );
    }
}

#[test]
fn active_success_replays_exact_order_with_environment_only_secrets_and_ignores_annotations() {
    let environment = TestEnvironment::new();
    let mut first = reviewed_case(0, "success");
    first["arguments"]["secret"] = Value::Null;
    first["secret_refs"] = json!({"/secret": ARGUMENT_SECRET_NAME});
    let mut second = reviewed_case(1, "tool_error");
    second["arguments"]["secret"] = Value::Null;
    second["secret_refs"] = json!({"/secret": ARGUMENT_SECRET_NAME});
    let mut document = scenario("read_only", vec![first, second]);
    document["target_env"] = json!([TARGET_SECRET_NAME]);
    let path = write_scenario(&environment, "active-success.json", &document);

    let output = check_command(&environment, &path, TOOL, "active-success")
        .env(ARGUMENT_SECRET_NAME, SECRET_VALUE)
        .env(TARGET_SECRET_NAME, SECRET_VALUE)
        .env("MCP_DOCTOR_UNLISTED_SECRET", SECRET_VALUE)
        .output()
        .expect("the active success journey should run");
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    for expected in [
        "PASS  scenario.configuration",
        "PASS  authorization.active",
        "PASS  transport.stdio",
        "PASS  protocol.revision",
        "PASS  discovery.catalogs",
        "PASS  schema.contracts",
        "PASS  runtime.tools.case[0]",
        "PASS  runtime.tools.case[1]",
        "outcome passed · exit 0",
    ] {
        assert!(stdout.contains(expected), "{stdout}");
    }
    assert_redacted(
        &output,
        &[
            TOOL,
            "author-only-case-0-never-report",
            "author-only-case-1-never-report",
            "/secret",
        ],
    );
}

#[test]
fn machine_success_report_redacts_resolved_arguments_results_and_author_identifiers() {
    let environment = TestEnvironment::new();
    let mut first = reviewed_case(0, "success");
    first["arguments"]["secret"] = Value::Null;
    first["secret_refs"] = json!({"/secret": ARGUMENT_SECRET_NAME});
    let mut second = reviewed_case(1, "tool_error");
    second["arguments"]["secret"] = Value::Null;
    second["secret_refs"] = json!({"/secret": ARGUMENT_SECRET_NAME});
    let mut document = scenario("read_only", vec![first, second]);
    document["target_env"] = json!([TARGET_SECRET_NAME]);
    let path = write_scenario(&environment, "active-machine-success.json", &document);
    let output = environment
        .command()
        .arg("check")
        .arg("--scenario")
        .arg(&path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json")
        .arg("--")
        .arg(fixture())
        .arg("active-success")
        .env(ARGUMENT_SECRET_NAME, SECRET_VALUE)
        .env(TARGET_SECRET_NAME, SECRET_VALUE)
        .output()
        .expect("the active machine success journey should run");
    let (_, stderr) = text(&output);
    let report = parse_and_validate_report(&output.stdout);

    assert!(output.status.success(), "{report:#}");
    assert!(stderr.is_empty());
    assert_eq!(report["outcome"], "passed");
    assert_eq!(report["exit_code"], 0);
    assert_redacted(
        &output,
        &[
            TOOL,
            "/secret",
            "author-only-case-0-never-report",
            "author-only-case-1-never-report",
            "sequence",
            "structuredContent",
        ],
    );
}

#[test]
fn junit_active_success_preserves_each_case_and_the_process_exit_gate() {
    let environment = TestEnvironment::new();
    let path = write_scenario(
        &environment,
        "active-junit-success.json",
        &scenario("read_only", vec![reviewed_case(0, "success")]),
    );
    let output = environment
        .command()
        .arg("check")
        .arg("--scenario")
        .arg(&path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("junit")
        .arg("--")
        .arg(fixture())
        .arg("active-one-success")
        .output()
        .expect("the active JUnit journey should run");
    let (_, stderr) = text(&output);
    let (document, summary) = parse_and_validate_junit(&output.stdout);

    assert!(output.status.success(), "{document}\n{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(summary.failures, 0);
    assert_eq!(summary.skipped, 0);
    assert!(document.contains("name=\"runtime.tools.case[0]\""));
    assert!(document.contains("report_outcome=passed\nexit_code=0"));
    assert_redacted(
        &output,
        &[TOOL, "author-only-case-0-never-report", "sequence"],
    );
}

#[test]
fn exact_tool_and_side_effect_gates_reject_before_starting_the_target() {
    for (effects, allowed_tool, expected_code) in [
        ("read_only", "synthetic.other", "MCP-AUTH-001"),
        ("read_only", "*", "MCP-AUTH-001"),
        ("read_only", "synthetic.*", "MCP-AUTH-001"),
        ("side_effecting", TOOL, "MCP-AUTH-002"),
    ] {
        let environment = TestEnvironment::new();
        let path = write_scenario(
            &environment,
            "authorization.json",
            &scenario(effects, vec![reviewed_case(0, "success")]),
        );
        let marker = environment.artifact_path("target-started");
        let mut command = check_command(&environment, &path, allowed_tool, "active-started-marker");
        command.arg(&marker);
        let output = command
            .output()
            .expect("the authorization rejection should run");
        let (stdout, stderr) = text(&output);

        assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
        assert!(stderr.is_empty(), "{stderr}");
        assert!(stdout.contains(expected_code), "{stdout}");
        assert!(stdout.contains("SKIP  transport.stdio"), "{stdout}");
        assert!(!marker.exists(), "the rejected target was started");
        assert_redacted(&output, &[TOOL, allowed_tool]);
    }
}

#[test]
fn side_effecting_cases_run_only_with_the_additional_gate() {
    let environment = TestEnvironment::new();
    let path = write_scenario(
        &environment,
        "side-effects.json",
        &scenario("side_effecting", vec![reviewed_case(0, "success")]),
    );
    let output = environment
        .command()
        .arg("check")
        .arg("--scenario")
        .arg(&path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--allow-side-effects")
        .arg("--")
        .arg(fixture())
        .arg("active-one-success")
        .output()
        .expect("the explicitly gated side-effecting scenario should run");
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("PASS  runtime.tools.case[0]"), "{stdout}");
    assert!(!stdout.contains("MCP-AUTH-002"), "{stdout}");
}

#[test]
fn invalid_or_missing_secret_references_fail_before_target_start_without_disclosure() {
    let invalid_cases = [
        json!({
            "id": "missing-location",
            "arguments": {"sequence": 0},
            "secret_refs": {"/missing": ARGUMENT_SECRET_NAME},
            "expect": {"result": "success"}
        }),
        json!({
            "id": "non-null",
            "arguments": {"sequence": 0, "secret": "literal-must-not-be-used"},
            "secret_refs": {"/secret": ARGUMENT_SECRET_NAME},
            "expect": {"result": "success"}
        }),
        json!({
            "id": "root-pointer",
            "arguments": {"sequence": 0},
            "secret_refs": {"": ARGUMENT_SECRET_NAME},
            "expect": {"result": "success"}
        }),
        json!({
            "id": "invalid-name",
            "arguments": {"sequence": 0, "secret": null},
            "secret_refs": {"/secret": "INVALID-NAME"},
            "expect": {"result": "success"}
        }),
    ];

    for (index, case) in invalid_cases.into_iter().enumerate() {
        let environment = TestEnvironment::new();
        let path = write_scenario(
            &environment,
            &format!("invalid-secret-{index}.json"),
            &scenario("read_only", vec![case]),
        );
        let marker = environment.artifact_path("target-started");
        let output = check_command(&environment, &path, TOOL, "active-started-marker")
            .arg(&marker)
            .env(ARGUMENT_SECRET_NAME, SECRET_VALUE)
            .output()
            .expect("the invalid reference should be rejected");
        let (stdout, stderr) = text(&output);

        assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
        assert!(stderr.is_empty());
        assert!(stdout.contains("MCP-SCENARIO-002"), "{stdout}");
        assert!(!marker.exists(), "the invalid scenario started its target");
        assert_redacted(
            &output,
            &[
                "/missing",
                "/secret",
                "INVALID-NAME",
                "literal-must-not-be-used",
            ],
        );
    }

    let environment = TestEnvironment::new();
    let mut case = reviewed_case(0, "success");
    case["arguments"]["secret"] = Value::Null;
    case["secret_refs"] = json!({"/secret": ARGUMENT_SECRET_NAME});
    let path = write_scenario(
        &environment,
        "missing-secret.json",
        &scenario("read_only", vec![case]),
    );
    let marker = environment.artifact_path("target-started");
    let output = check_command(&environment, &path, TOOL, "active-started-marker")
        .arg(&marker)
        .output()
        .expect("the missing invoking environment value should be rejected");
    assert_eq!(output.status.code(), Some(2));
    assert!(!marker.exists());
    assert_redacted(&output, &["/secret"]);
}

#[test]
fn missing_target_environment_value_is_a_prestart_configuration_failure() {
    let environment = TestEnvironment::new();
    let mut document = scenario("read_only", vec![reviewed_case(0, "success")]);
    document["target_env"] = json!([TARGET_SECRET_NAME]);
    let path = write_scenario(&environment, "missing-target-env.json", &document);
    let marker = environment.artifact_path("target-started");
    let output = check_command(&environment, &path, TOOL, "active-started-marker")
        .arg(&marker)
        .output()
        .expect("the missing target environment value should be rejected");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stdout.contains("MCP-SCENARIO-002"), "{stdout}");
    assert!(!marker.exists());
    assert_redacted(&output, &[]);
}

#[test]
fn ordinary_silent_failure_and_output_mismatches_continue_to_later_ordered_cases() {
    let environment = TestEnvironment::new();
    let scenario = scenario(
        "read_only",
        vec![
            reviewed_case(0, "tool_error"),
            reviewed_case(1, "success"),
            reviewed_case(2, "success"),
        ],
    );
    let path = write_scenario(&environment, "mismatches.json", &scenario);
    let marker = environment.artifact_path("third-case-called");
    let output = check_command(&environment, &path, TOOL, "active-mismatch-continue")
        .arg(&marker)
        .output()
        .expect("the mismatch continuation journey should run");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(marker.exists(), "a later reviewed case was not called");
    assert!(stdout.contains("MCP-ACTIVE-004"), "{stdout}");
    assert!(stdout.contains("MCP-ACTIVE-005"), "{stdout}");
    assert!(stdout.contains("PASS  runtime.tools.case[2]"), "{stdout}");
    assert_redacted(
        &output,
        &[
            TOOL,
            "author-only-case-0-never-report",
            "author-only-case-1-never-report",
            "author-only-case-2-never-report",
        ],
    );
}

#[test]
fn protocol_level_tool_rejection_is_redacted_and_later_cases_continue() {
    let environment = TestEnvironment::new();
    let path = write_scenario(
        &environment,
        "rejection.json",
        &scenario(
            "read_only",
            vec![reviewed_case(0, "success"), reviewed_case(1, "success")],
        ),
    );
    let marker = environment.artifact_path("second-case-called");
    let output = check_command(&environment, &path, TOOL, "active-tool-rejection")
        .arg(&marker)
        .output()
        .expect("the rejection continuation journey should run");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(marker.exists());
    assert!(stdout.contains("MCP-ACTIVE-003"), "{stdout}");
    assert!(stdout.contains("PASS  runtime.tools.case[1]"), "{stdout}");
    assert_redacted(&output, &[TOOL]);
}

#[test]
fn input_required_is_incomplete_without_retry_and_later_declared_cases_continue() {
    let output = run_check(
        scenario(
            "read_only",
            vec![reviewed_case(0, "success"), reviewed_case(1, "success")],
        ),
        "active-input-required",
    );
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(3), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("runtime.tools.case[0]"), "{stdout}");
    assert!(
        stdout.contains("input that mcp-doctor is not authorized to provide"),
        "{stdout}"
    );
    assert!(stdout.contains("PASS  runtime.tools.case[1]"), "{stdout}");
    assert!(stdout.contains("outcome incomplete · exit 3"), "{stdout}");
    assert_redacted(&output, &[TOOL, "requestState", "inputRequests"]);
}

#[test]
fn advertised_and_scenario_schemas_are_local_bounded_and_checked_before_activity() {
    let environment = TestEnvironment::new();
    let mut document = scenario("read_only", vec![reviewed_case(0, "success")]);
    document["cases"][0]["expect"]["structured_output_schema"] = json!({
        "type": "object",
        "properties": {"value": {"$ref": "https://invalid.example/private-schema"}}
    });
    let path = write_scenario(&environment, "external-scenario-schema.json", &document);
    let marker = environment.artifact_path("target-started");
    let output = check_command(&environment, &path, TOOL, "active-started-marker")
        .arg(&marker)
        .output()
        .expect("the external scenario schema should be rejected");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stdout.contains("MCP-SCHEMA-003"), "{stdout}");
    assert!(!marker.exists());
    assert!(!stdout.contains("invalid.example"));

    let output = run_check(
        scenario("read_only", vec![reviewed_case(0, "success")]),
        "active-advertised-schema-invalid",
    );
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-SCHEMA-003"), "{stdout}");
    assert!(stdout.contains("SKIP  runtime.tools.case[0]"), "{stdout}");
    assert!(!stdout.contains("invalid.example"));

    let environment = TestEnvironment::new();
    let mut nested = json!({"type": "boolean"});
    for _ in 0..70 {
        nested = json!({"not": nested});
    }
    let mut document = scenario("read_only", vec![reviewed_case(0, "success")]);
    document["cases"][0]["expect"]["structured_output_schema"] = nested;
    let path = write_scenario(&environment, "bounded-scenario-schema.json", &document);
    let marker = environment.artifact_path("target-started");
    let output = check_command(&environment, &path, TOOL, "active-started-marker")
        .arg(&marker)
        .output()
        .expect("the over-depth scenario schema should be rejected");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stdout.contains("MCP-LIMIT-001"), "{stdout}");
    assert!(stdout.contains("schema_depth"), "{stdout}");
    assert!(!marker.exists());

    let output = run_check(
        scenario("read_only", vec![reviewed_case(0, "success")]),
        "active-advertised-output-depth",
    );
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stdout.contains("MCP-LIMIT-001"), "{stdout}");
    assert!(stdout.contains("schema_depth"), "{stdout}");
    assert!(stdout.contains("SKIP  runtime.tools.case[0]"), "{stdout}");
}

#[test]
fn hostile_scenario_schema_findings_are_capped_before_target_start() {
    let environment = TestEnvironment::new();
    let branches = (0..300)
        .map(|index| json!({"$ref": format!("https://invalid.example/private-{index}")}))
        .collect::<Vec<_>>();
    let mut document = scenario("read_only", vec![reviewed_case(0, "success")]);
    document["cases"][0]["expect"]["structured_output_schema"] = json!({
        "type": "object",
        "allOf": branches
    });
    let path = write_scenario(&environment, "schema-finding-overflow.json", &document);
    let marker = environment.artifact_path("target-started");
    let output = environment
        .command()
        .arg("check")
        .arg("--scenario")
        .arg(&path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json")
        .arg("--")
        .arg(fixture())
        .arg("active-started-marker")
        .arg(&marker)
        .output()
        .expect("the bounded scenario-schema rejection should run");
    let (_, stderr) = text(&output);
    let report = parse_and_validate_report(&output.stdout);

    assert_eq!(output.status.code(), Some(2), "{report:#}");
    assert!(stderr.is_empty());
    assert!(!marker.exists());
    let checks = report["checks"]
        .as_array()
        .expect("checks should be an array");
    let finding_count = checks
        .iter()
        .map(|check| {
            check["findings"]
                .as_array()
                .expect("findings should be an array")
                .len()
        })
        .sum::<usize>();
    assert_eq!(finding_count, 256, "{report:#}");
    let scenario_check = checks
        .iter()
        .find(|check| check["id"] == "scenario.configuration")
        .expect("the scenario failure should be explicit");
    assert!(
        scenario_check["findings"]
            .as_array()
            .is_some_and(|findings| {
                findings.iter().any(|finding| {
                    finding["code"] == "MCP-LIMIT-001"
                        && finding["evidence"]["limit"] == "report_findings"
                })
            })
    );
    let transport = checks
        .iter()
        .find(|check| check["id"] == "transport.stdio")
        .expect("the blocked transport should remain explicit");
    assert_eq!(transport["state"], "skipped");
    assert_eq!(
        transport["blocked_by"]["check_id"],
        "scenario.configuration"
    );
    assert_redacted(
        &output,
        &[TOOL, "invalid.example", "private-0", "private-299"],
    );
}

#[test]
fn invalid_case_arguments_are_reported_without_calling_the_tool() {
    let mut case = reviewed_case(0, "success");
    case["arguments"]["sequence"] = Value::String("private-invalid-value".to_owned());
    let output = run_check(scenario("read_only", vec![case]), "active-no-calls");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-ACTIVE-002"), "{stdout}");
    assert!(stdout.contains("the case was not called"), "{stdout}");
    assert!(!stdout.contains("private-invalid-value"));
}

#[test]
fn runtime_output_validation_bounds_stop_later_cases_with_causal_skips() {
    for (mode, limit) in [
        ("active-output-instance-depth", "schema_depth"),
        ("active-output-evaluation-limit", "schema_evaluation_steps"),
    ] {
        let output = run_check(
            scenario(
                "read_only",
                vec![reviewed_case(0, "success"), reviewed_case(1, "success")],
            ),
            mode,
        );
        let (stdout, stderr) = text(&output);

        assert_eq!(output.status.code(), Some(1), "{mode}: {stdout}\n{stderr}");
        assert!(stderr.is_empty());
        assert!(stdout.contains("MCP-LIMIT-001"), "{stdout}");
        assert!(stdout.contains(limit), "{stdout}");
        assert!(stdout.contains("FAIL  runtime.tools.case[0]"), "{stdout}");
        assert!(stdout.contains("SKIP  runtime.tools.case[1]"), "{stdout}");
        assert!(
            stdout.contains("blocked by runtime.tools.case[0]"),
            "{stdout}"
        );
        assert_redacted(&output, &[TOOL, "nested", "values"]);
    }

    let required = (0..101)
        .map(|index| Value::String(format!("private-required-{index}")))
        .collect::<Vec<_>>();
    let mut first = reviewed_case(0, "success");
    first["expect"]["structured_output_schema"] = json!({
        "type": "object",
        "required": required
    });
    let output = run_check(
        scenario("read_only", vec![first, reviewed_case(1, "success")]),
        "active-one-success",
    );
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-LIMIT-001"), "{stdout}");
    assert!(stdout.contains("validation_errors"), "{stdout}");
    assert!(stdout.contains("SKIP  runtime.tools.case[1]"), "{stdout}");
    assert_redacted(
        &output,
        &[TOOL, "private-required-0", "private-required-100"],
    );
}

#[test]
fn crash_and_output_limit_stop_remaining_calls_with_causal_skips() {
    for (mode, expected_code, expected_limit) in [
        ("active-crash", "MCP-TRANSPORT-004", None),
        ("active-oversize", "MCP-LIMIT-001", Some("message_bytes")),
    ] {
        let output = run_check(
            scenario(
                "read_only",
                vec![reviewed_case(0, "success"), reviewed_case(1, "success")],
            ),
            mode,
        );
        let (stdout, stderr) = text(&output);

        assert_eq!(output.status.code(), Some(1), "{mode}: {stdout}\n{stderr}");
        assert!(stderr.is_empty());
        assert!(stdout.contains(expected_code), "{stdout}");
        if let Some(limit) = expected_limit {
            assert!(stdout.contains(limit), "{stdout}");
        }
        assert!(stdout.contains("SKIP  runtime.tools.case[0]"), "{stdout}");
        assert!(stdout.contains("SKIP  runtime.tools.case[1]"), "{stdout}");
        assert!(stdout.contains("blocked by transport.stdio"), "{stdout}");
        assert_redacted(&output, &[TOOL]);
    }
}

#[test]
fn invalid_result_envelopes_stop_later_calls_but_missing_exact_tools_never_start_calls() {
    let output = run_check(
        scenario(
            "read_only",
            vec![reviewed_case(0, "success"), reviewed_case(1, "success")],
        ),
        "active-invalid-result",
    );
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-ACTIVE-006"), "{stdout}");
    assert!(stdout.contains("SKIP  runtime.tools.case[1]"), "{stdout}");
    assert!(
        stdout.contains("blocked by runtime.tools.case[0]"),
        "{stdout}"
    );

    let output = run_check(
        scenario("read_only", vec![reviewed_case(0, "success")]),
        "active-tool-not-found",
    );
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-ACTIVE-001"), "{stdout}");
    assert!(stdout.contains("SKIP  runtime.tools.case[0]"), "{stdout}");
    assert_redacted(&output, &[TOOL, "synthetic.other"]);
}

#[test]
fn invalid_discovery_and_tool_catalog_envelopes_stop_before_any_tool_call() {
    for (mode, diagnosis) in [
        ("active-discovery-contract-invalid", "protocol.envelope"),
        ("active-tools-contract-invalid", "discovery.catalogs"),
    ] {
        let output = run_check(
            scenario("read_only", vec![reviewed_case(0, "success")]),
            mode,
        );
        let (stdout, stderr) = text(&output);

        assert_eq!(output.status.code(), Some(1), "{mode}: {stdout}\n{stderr}");
        assert!(stderr.is_empty());
        assert!(stdout.contains("MCP-CATALOG-001"), "{stdout}");
        assert!(stdout.contains("SKIP  runtime.tools.case[0]"), "{stdout}");
        assert!(
            stdout.contains(&format!("blocked by {diagnosis}")),
            "{stdout}"
        );
        assert_redacted(&output, &[TOOL]);
    }
}

#[test]
fn active_revision_count_is_bounded_before_catalog_or_tool_activity() {
    let output = run_check(
        scenario("read_only", vec![reviewed_case(0, "success")]),
        "active-revision-limit",
    );
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-LIMIT-001"), "{stdout}");
    assert!(stdout.contains("protocol_revisions"), "{stdout}");
    assert!(stdout.contains("SKIP  discovery.catalogs"), "{stdout}");
    assert!(stdout.contains("SKIP  runtime.tools.case[0]"), "{stdout}");
    assert!(stdout.contains("blocked by protocol.revision"), "{stdout}");
    assert_redacted(&output, &[TOOL, "synthetic-private-revision"]);
}

#[test]
fn hostile_catalog_findings_are_capped_in_a_valid_causal_report() {
    let environment = TestEnvironment::new();
    let path = write_scenario(
        &environment,
        "finding-overflow.json",
        &scenario("read_only", vec![reviewed_case(0, "success")]),
    );
    let output = environment
        .command()
        .arg("check")
        .arg("--scenario")
        .arg(&path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json")
        .arg("--")
        .arg(fixture())
        .arg("active-catalog-finding-overflow")
        .output()
        .expect("the bounded finding-overflow journey should run");
    let (_, stderr) = text(&output);
    let report = parse_and_validate_report(&output.stdout);

    assert_eq!(output.status.code(), Some(1), "{report:#}");
    assert!(stderr.is_empty());
    let checks = report["checks"]
        .as_array()
        .expect("checks should be an array");
    let finding_count = checks
        .iter()
        .map(|check| {
            check["findings"]
                .as_array()
                .expect("findings should be an array")
                .len()
        })
        .sum::<usize>();
    assert_eq!(finding_count, 256, "{report:#}");
    let diagnosed_check = checks
        .iter()
        .find(|check| check["id"] == "discovery.catalogs")
        .expect("the bounded causal check should be reported");
    assert!(
        diagnosed_check["findings"]
            .as_array()
            .is_some_and(|findings| {
                findings.iter().any(|finding| {
                    finding["code"] == "MCP-LIMIT-001"
                        && finding["evidence"]["limit"] == "report_findings"
                })
            })
    );
    let case = checks
        .iter()
        .find(|check| check["id"] == "runtime.tools.case[0]")
        .expect("the blocked case should remain explicit");
    assert_eq!(case["state"], "skipped");
    assert_eq!(case["blocked_by"]["check_id"], "discovery.catalogs");
    assert_redacted(&output, &[TOOL]);
}

#[test]
fn active_cleanup_terminates_and_reaps_a_resistant_process_tree() {
    let environment = TestEnvironment::new();
    let path = write_scenario(
        &environment,
        "cleanup.json",
        &scenario("read_only", vec![reviewed_case(0, "success")]),
    );
    let marker = environment.artifact_path("descendant-survived");
    let started = Instant::now();
    let output = check_command(&environment, &path, TOOL, "active-resistant-child")
        .arg(&marker)
        .output()
        .expect("the active cleanup journey should run");
    let elapsed = started.elapsed();
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(elapsed >= Duration::from_millis(1_800), "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(8), "{elapsed:?}");
    thread::sleep(Duration::from_secs(2));
    assert!(!marker.exists(), "the active descendant survived cleanup");
}

#[test]
fn stable_json_reports_indexed_cases_without_arguments_results_or_ids() {
    let environment = TestEnvironment::new();
    let path = write_scenario(
        &environment,
        "machine-report.json",
        &scenario("read_only", vec![reviewed_case(0, "success")]),
    );
    let output = environment
        .command()
        .arg("check")
        .arg("--scenario")
        .arg(&path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json")
        .arg("--")
        .arg(fixture())
        .arg("active-crash")
        .output()
        .expect("the active JSON report should run");
    let (_, stderr) = text(&output);
    let report = parse_and_validate_report(&output.stdout);

    assert_eq!(output.status.code(), Some(1), "{report:#}");
    assert!(stderr.is_empty());
    assert_eq!(report["schema_version"], "mcp-doctor.report/v1");
    assert_eq!(report["outcome"], "failed");
    assert_eq!(report["exit_code"], 1);
    let case = report["checks"]
        .as_array()
        .expect("checks should be an array")
        .iter()
        .find(|check| check["id"] == "runtime.tools.case[0]")
        .expect("the declared case index should remain explicit");
    assert_eq!(case["state"], "skipped");
    assert_eq!(case["skip_reason"], "prerequisite_failed");
    assert!(case.get("arguments").is_none());
    assert!(case.get("result").is_none());
    assert_redacted(
        &output,
        &[TOOL, "author-only-case-0-never-report", "sequence"],
    );
}

#[test]
fn scenario_case_bounds_uniqueness_and_duplicate_members_are_enforced_prestart() {
    let environment = TestEnvironment::new();
    let accepted_cases = (0..100)
        .map(|index| {
            let mut case = reviewed_case(index, "success");
            case["arguments"]["sequence"] = Value::String("invalid-without-call".to_owned());
            case
        })
        .collect();
    let accepted_path = write_scenario(
        &environment,
        "exactly-100.json",
        &scenario("read_only", accepted_cases),
    );
    let accepted = check_command(&environment, &accepted_path, TOOL, "active-no-calls")
        .output()
        .expect("the exact case maximum should be accepted");
    let (stdout, stderr) = text(&accepted);
    assert_eq!(accepted.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(!stdout.contains("MCP-LIMIT-001"), "{stdout}");
    assert!(stdout.contains("runtime.tools.case[99]"), "{stdout}");

    let rejection_documents = [
        scenario("read_only", Vec::new()),
        scenario(
            "read_only",
            (0..101)
                .map(|index| reviewed_case(index, "success"))
                .collect(),
        ),
        scenario(
            "read_only",
            vec![reviewed_case(0, "success"), reviewed_case(0, "success")],
        ),
    ];
    for (index, document) in rejection_documents.into_iter().enumerate() {
        let environment = TestEnvironment::new();
        let path = write_scenario(
            &environment,
            &format!("rejected-case-contract-{index}.json"),
            &document,
        );
        let marker = environment.artifact_path("target-started");
        let output = check_command(&environment, &path, TOOL, "active-started-marker")
            .arg(&marker)
            .output()
            .expect("the invalid case contract should be rejected");
        assert_eq!(output.status.code(), Some(2));
        assert!(!marker.exists());
    }

    let environment = TestEnvironment::new();
    let duplicate_path = environment.artifact_path("duplicate-members.json");
    fs::write(
        &duplicate_path,
        format!(
            r#"{{"schema_version":"mcp-doctor.scenario/v1alpha1","tool":"{TOOL}","safety":{{"effects":"read_only"}},"cases":[{{"id":"private-id","arguments":{{"sequence":0,"secret":null}},"secret_refs":{{"/private-pointer":"{ARGUMENT_SECRET_NAME}","/private-pointer":"{ARGUMENT_SECRET_NAME}"}},"expect":{{"result":"success"}}}}]}}"#
        ),
    )
    .expect("the duplicate-member scenario should be writable");
    let marker = environment.artifact_path("target-started");
    let output = check_command(&environment, &duplicate_path, TOOL, "active-started-marker")
        .arg(&marker)
        .output()
        .expect("duplicate JSON members should be rejected");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stdout.contains("MCP-SCENARIO-001"), "{stdout}");
    assert!(!marker.exists());
    assert_redacted(&output, &[TOOL, "private-id", "/private-pointer"]);
}

#[test]
fn scenario_file_bytes_and_regular_file_shape_are_bounded_prestart() {
    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("target-started");
    let oversized_path = environment.artifact_path("oversized-scenario.json");
    fs::write(&oversized_path, vec![b' '; 1_048_577])
        .expect("the oversized scenario should be writable");
    let output = check_command(&environment, &oversized_path, TOOL, "active-started-marker")
        .arg(&marker)
        .output()
        .expect("the scenario byte rejection should run");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-LIMIT-001"), "{stdout}");
    assert!(stdout.contains("scenario_bytes"), "{stdout}");
    assert!(stdout.contains("SKIP  transport.stdio"), "{stdout}");
    assert!(!marker.exists());

    let directory_path = environment.artifact_path("scenario-directory");
    fs::create_dir(&directory_path).expect("the scenario directory should be creatable");
    let output = check_command(&environment, &directory_path, TOOL, "active-started-marker")
        .arg(&marker)
        .output()
        .expect("the non-file scenario rejection should run");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-SCENARIO-001"), "{stdout}");
    assert!(stdout.contains("SKIP  transport.stdio"), "{stdout}");
    assert!(!marker.exists());
}
