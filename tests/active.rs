#![cfg(feature = "internal-test-fixtures")]

mod support;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::{Value, json};
use support::{
    TestEnvironment, assert_descendant_was_ready_and_terminated, parse_and_validate_junit,
    parse_and_validate_report,
};

const TOOL: &str = "synthetic.reviewed";
const WORKFLOW_LOOKUP: &str = "synthetic.workflow.lookup";
const WORKFLOW_MUTATE: &str = "synthetic.workflow.mutate";
const WORKFLOW_READ: &str = "synthetic.workflow.read";
const WORKFLOW_CLEANUP: &str = "synthetic.workflow.cleanup";
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

fn workflow_read_only_scenario() -> Value {
    json!({
        "schema_version": "mcp-doctor.scenario/v2alpha1",
        "steps": [
            {
                "id": "private-lookup-step-never-report",
                "tool": WORKFLOW_LOOKUP,
                "safety": {"effects": "read_only"},
                "arguments": {"query": SECRET_VALUE},
                "captures": {"private_resource_id": "/resource/id"},
                "expect": {"result": "success"}
            },
            {
                "id": "private-read-step-never-report",
                "tool": WORKFLOW_READ,
                "safety": {"effects": "read_only"},
                "arguments": {"id": null},
                "argument_refs": {"/id": "private_resource_id"},
                "expect": {
                    "result": "success",
                    "structured_output_schema": {
                        "type": "object",
                        "properties": {
                            "value": {"const": SECRET_VALUE},
                            "version": {"const": 1}
                        },
                        "required": ["value", "version"],
                        "additionalProperties": false
                    }
                }
            }
        ]
    })
}

fn workflow_mutation_scenario() -> Value {
    json!({
        "schema_version": "mcp-doctor.scenario/v2alpha1",
        "steps": [
            {
                "id": "private-lookup-step-never-report",
                "tool": WORKFLOW_LOOKUP,
                "safety": {"effects": "read_only"},
                "arguments": {"query": SECRET_VALUE},
                "captures": {"private_resource_id": "/resource/id"},
                "expect": {"result": "success"}
            },
            {
                "id": "private-mutation-step-never-report",
                "tool": WORKFLOW_MUTATE,
                "safety": {"effects": "side_effecting"},
                "arguments": {"id": null, "value": SECRET_VALUE},
                "argument_refs": {"/id": "private_resource_id"},
                "captures": {"private_updated_version": "/version"},
                "expect": {
                    "result": "success",
                    "structured_output_schema": {
                        "type": "object",
                        "properties": {"version": {"const": 2}},
                        "required": ["version"],
                        "additionalProperties": false
                    }
                }
            },
            {
                "id": "private-fresh-read-step-never-report",
                "tool": WORKFLOW_READ,
                "safety": {"effects": "read_only"},
                "arguments": {"id": null, "expectedVersion": null},
                "argument_refs": {
                    "/id": "private_resource_id",
                    "/expectedVersion": "private_updated_version"
                },
                "expect": {
                    "result": "success",
                    "structured_output_schema": {
                        "type": "object",
                        "properties": {
                            "value": {"const": SECRET_VALUE},
                            "version": {"const": 2}
                        },
                        "required": ["value", "version"],
                        "additionalProperties": false
                    }
                }
            },
            {
                "id": "private-cleanup-step-never-report",
                "tool": WORKFLOW_CLEANUP,
                "safety": {"effects": "side_effecting"},
                "cleanup": true,
                "arguments": {"id": null},
                "argument_refs": {"/id": "private_resource_id"},
                "expect": {"result": "success"}
            }
        ]
    })
}

fn workflow_cleanup_scenario() -> Value {
    let mut document = workflow_mutation_scenario();
    document["steps"]
        .as_array_mut()
        .expect("workflow steps should be an array")
        .remove(2);
    document
}

fn workflow_command(
    environment: &TestEnvironment,
    scenario_path: &Path,
    allowed_tools: &[&str],
) -> Command {
    let mut command = environment.command();
    command.arg("check").arg("--scenario").arg(scenario_path);
    for tool in allowed_tools {
        command.arg("--allow-tool").arg(tool);
    }
    command
}

fn finish_workflow_command(command: &mut Command, mode: &str) {
    command.arg("--").arg(fixture()).arg(mode);
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

fn legacy_check_command(
    environment: &TestEnvironment,
    scenario_path: &Path,
    mode: &str,
) -> Command {
    let mut command = environment.command();
    command
        .arg("check")
        .arg("--protocol-version")
        .arg("2025-11-25")
        .arg("--scenario")
        .arg(scenario_path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--")
        .arg(fixture())
        .arg(mode);
    command
}

fn run_legacy_check(mode: &str) -> Output {
    let environment = TestEnvironment::new();
    let scenario_path = write_scenario(
        &environment,
        "legacy-scenario.json",
        &scenario("read_only", vec![reviewed_case(0, "success")]),
    );
    legacy_check_command(&environment, &scenario_path, mode)
        .output()
        .expect("the legacy check should start")
}

fn run_legacy_check_json(mode: &str) -> Output {
    let environment = TestEnvironment::new();
    let scenario_path = write_scenario(
        &environment,
        "legacy-scenario.json",
        &scenario("read_only", vec![reviewed_case(0, "success")]),
    );
    environment
        .command()
        .arg("check")
        .arg("--protocol-version")
        .arg("2025-11-25")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json")
        .arg("--")
        .arg(fixture())
        .arg(mode)
        .output()
        .expect("the legacy JSON check should start")
}

fn v2025_06_check_command(
    environment: &TestEnvironment,
    scenario_path: &Path,
    mode: &str,
) -> Command {
    let mut command = environment.command();
    command
        .arg("check")
        .arg("--protocol-version")
        .arg("2025-06-18")
        .arg("--scenario")
        .arg(scenario_path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--")
        .arg(fixture())
        .arg(mode);
    command
}

fn run_v2025_06_check(mode: &str, mode_arguments: &[&str], format: Option<&str>) -> Output {
    run_v2025_06_scenario(
        scenario("read_only", vec![reviewed_case(0, "success")]),
        mode,
        mode_arguments,
        format,
    )
}

fn run_v2025_06_scenario(
    document: Value,
    mode: &str,
    mode_arguments: &[&str],
    format: Option<&str>,
) -> Output {
    let environment = TestEnvironment::new();
    let scenario_path = write_scenario(&environment, "v2025-06-scenario.json", &document);
    let mut command = environment.command();
    command
        .arg("check")
        .arg("--protocol-version")
        .arg("2025-06-18")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--allow-tool")
        .arg(TOOL);
    if let Some(format) = format {
        command.arg("--format").arg(format);
    }
    command
        .arg("--")
        .arg(fixture())
        .arg(mode)
        .args(mode_arguments);
    command
        .output()
        .expect("the MCP 2025-06-18 check should start")
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

fn report_check<'a>(report: &'a Value, id: &str) -> &'a Value {
    report["checks"]
        .as_array()
        .expect("checks should be an array")
        .iter()
        .find(|check| check["id"] == id)
        .unwrap_or_else(|| panic!("report should contain {id}"))
}

#[test]
fn committed_workflow_schema_accepts_the_reviewed_examples_and_stays_strict() {
    let schema: Value = serde_json::from_str(include_str!(
        "../schemas/mcp-doctor.scenario.v2alpha1.schema.json"
    ))
    .expect("the workflow schema should be JSON");
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("the workflow schema should follow Draft 2020-12");
    for document in [workflow_read_only_scenario(), workflow_mutation_scenario()] {
        let errors = validator
            .iter_errors(&document)
            .map(|error| error.instance_path().to_string())
            .collect::<Vec<_>>();
        assert!(errors.is_empty(), "workflow schema rejected {errors:?}");
    }

    let mut unreviewed = workflow_read_only_scenario();
    unreviewed["steps"][0]["script"] = json!("unbounded()");
    assert!(!validator.is_valid(&unreviewed));
}

#[test]
fn workflow_structural_capture_is_deterministic_and_reporter_safe() {
    let environment = TestEnvironment::new();
    let path = write_scenario(
        &environment,
        "workflow-read-only.json",
        &workflow_read_only_scenario(),
    );
    let json_path = environment.artifact_path("workflow-report.json");
    let junit_path = environment.artifact_path("workflow-report.xml");
    let mut first_command =
        workflow_command(&environment, &path, &[WORKFLOW_LOOKUP, WORKFLOW_READ]);
    first_command
        .arg("--format")
        .arg("json")
        .arg("--json-report")
        .arg(&json_path)
        .arg("--junit-report")
        .arg(&junit_path);
    finish_workflow_command(&mut first_command, "workflow-read-only");
    let first = first_command
        .output()
        .expect("the read-only workflow should run");

    let second_environment = TestEnvironment::new();
    let second_path = write_scenario(
        &second_environment,
        "workflow-read-only.json",
        &workflow_read_only_scenario(),
    );
    let mut second_command = workflow_command(
        &second_environment,
        &second_path,
        &[WORKFLOW_LOOKUP, WORKFLOW_READ],
    );
    second_command.arg("--format").arg("json");
    finish_workflow_command(&mut second_command, "workflow-read-only");
    let second = second_command
        .output()
        .expect("the repeated read-only workflow should run");

    let (_, first_stderr) = text(&first);
    let (_, second_stderr) = text(&second);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stdout)
    );
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stdout)
    );
    assert!(first_stderr.is_empty());
    assert!(second_stderr.is_empty());
    assert_eq!(first.stdout, second.stdout);

    let report = parse_and_validate_report(&first.stdout);
    assert_eq!(report["outcome"], "passed");
    assert_eq!(
        report_check(&report, "runtime.workflow.step[0]")["state"],
        "performed"
    );
    assert_eq!(
        report_check(&report, "runtime.workflow.step[1]")["state"],
        "performed"
    );
    assert_eq!(
        parse_and_validate_report(&fs::read(&json_path).expect("JSON artifact should exist")),
        report
    );
    let (junit, summary) =
        parse_and_validate_junit(&fs::read(&junit_path).expect("JUnit artifact should exist"));
    assert_eq!(summary.failures, 0);
    assert_eq!(summary.skipped, 0);
    assert!(junit.contains("runtime.workflow.step[0]"));
    assert!(junit.contains("runtime.workflow.step[1]"));
    assert_redacted(
        &first,
        &[
            WORKFLOW_LOOKUP,
            WORKFLOW_READ,
            "private-lookup-step-never-report",
            "private-read-step-never-report",
            "private_resource_id",
            "/resource/id",
            "/id",
        ],
    );
}

#[test]
fn workflow_mutation_requires_the_complete_exact_authority_set() {
    let tools = [
        WORKFLOW_LOOKUP,
        WORKFLOW_MUTATE,
        WORKFLOW_READ,
        WORKFLOW_CLEANUP,
    ];
    for (name, allowed, allow_side_effects) in [
        ("missing", tools[..3].to_vec(), true),
        (
            "wildcard",
            vec!["*", WORKFLOW_MUTATE, WORKFLOW_READ, WORKFLOW_CLEANUP],
            true,
        ),
        (
            "duplicate",
            vec![
                WORKFLOW_LOOKUP,
                WORKFLOW_LOOKUP,
                WORKFLOW_MUTATE,
                WORKFLOW_READ,
                WORKFLOW_CLEANUP,
            ],
            true,
        ),
        ("side-effects", tools.to_vec(), false),
    ] {
        let environment = TestEnvironment::new();
        let path = write_scenario(
            &environment,
            &format!("workflow-{name}.json"),
            &workflow_mutation_scenario(),
        );
        let marker_name = format!("workflow-{name}-started");
        let marker = environment.artifact_path(&marker_name);
        let mut command = workflow_command(&environment, &path, &allowed);
        if allow_side_effects {
            command.arg("--allow-side-effects");
        }
        command
            .arg("--")
            .arg(fixture())
            .arg("active-started-marker")
            .arg(&marker);
        let output = command
            .output()
            .expect("invalid workflow authority should be rejected");
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(2), "{name}: {stdout}\n{stderr}");
        assert!(stderr.is_empty());
        assert!(!marker.exists(), "{name} authority started the target");
        if name == "side-effects" {
            assert!(stdout.contains("MCP-AUTH-002"), "{stdout}");
        } else {
            assert!(stdout.contains("MCP-AUTH-001"), "{stdout}");
        }
        assert_redacted(&output, &tools);
    }

    let environment = TestEnvironment::new();
    let path = write_scenario(
        &environment,
        "workflow-mutation.json",
        &workflow_mutation_scenario(),
    );
    let mut command = workflow_command(&environment, &path, &tools);
    command
        .arg("--allow-side-effects")
        .arg("--format")
        .arg("json");
    finish_workflow_command(&mut command, "workflow-mutation");
    let output = command
        .output()
        .expect("the authorized mutation should run");
    let (_, stderr) = text(&output);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(stderr.is_empty());
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["outcome"], "passed");
    for id in [
        "runtime.workflow.step[0]",
        "runtime.workflow.step[1]",
        "runtime.workflow.step[2]",
        "runtime.workflow.cleanup[3]",
    ] {
        assert_eq!(report_check(&report, id)["state"], "performed", "{id}");
    }
    assert_redacted(
        &output,
        &tools
            .into_iter()
            .chain([
                "private_resource_id",
                "private_updated_version",
                "private-cleanup-step-never-report",
            ])
            .collect::<Vec<_>>(),
    );
}

#[test]
fn workflow_first_failure_skips_later_main_work_and_runs_only_cleanup() {
    let tools = [
        WORKFLOW_LOOKUP,
        WORKFLOW_MUTATE,
        WORKFLOW_READ,
        WORKFLOW_CLEANUP,
    ];
    for (mode, expected_code) in [
        ("workflow-main-failure-cleanup", "MCP-ACTIVE-004"),
        ("workflow-schema-mismatch-cleanup", "MCP-ACTIVE-005"),
    ] {
        let environment = TestEnvironment::new();
        let path = write_scenario(
            &environment,
            &format!("{mode}.json"),
            &workflow_mutation_scenario(),
        );
        let mut command = workflow_command(&environment, &path, &tools);
        command
            .arg("--allow-side-effects")
            .arg("--format")
            .arg("json");
        finish_workflow_command(&mut command, mode);
        let output = command.output().expect("the failing workflow should run");
        let (_, stderr) = text(&output);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(stderr.is_empty());
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(
            report["primary_diagnosis"]["check_id"],
            "runtime.workflow.step[1]"
        );
        assert_eq!(
            report["primary_diagnosis"]["findings"][0]["code"],
            expected_code
        );
        let blocked = report_check(&report, "runtime.workflow.step[2]");
        assert_eq!(blocked["state"], "skipped");
        assert_eq!(
            blocked["blocked_by"]["check_id"],
            "runtime.workflow.step[1]"
        );
        assert_eq!(
            report_check(&report, "runtime.workflow.cleanup[3]")["state"],
            "performed"
        );
        assert_redacted(&output, &tools);
    }
}

#[test]
fn workflow_input_required_is_incomplete_but_still_runs_declared_cleanup() {
    let tools = [
        WORKFLOW_LOOKUP,
        WORKFLOW_MUTATE,
        WORKFLOW_READ,
        WORKFLOW_CLEANUP,
    ];
    let environment = TestEnvironment::new();
    let path = write_scenario(
        &environment,
        "workflow-input-required.json",
        &workflow_mutation_scenario(),
    );
    let mut command = workflow_command(&environment, &path, &tools);
    command
        .arg("--allow-side-effects")
        .arg("--format")
        .arg("json");
    finish_workflow_command(&mut command, "workflow-input-required-cleanup");
    let output = command
        .output()
        .expect("the incomplete workflow should return");
    let (_, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(3));
    assert!(stderr.is_empty());
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["outcome"], "incomplete");
    assert_eq!(
        report_check(&report, "runtime.workflow.step[1]")["skip_reason"],
        "input_required"
    );
    assert_eq!(
        report_check(&report, "runtime.workflow.step[2]")["state"],
        "skipped"
    );
    assert_eq!(
        report_check(&report, "runtime.workflow.cleanup[3]")["state"],
        "performed"
    );
    assert_redacted(&output, &tools);
}

#[test]
fn workflow_missing_capture_and_invalid_pointer_fail_without_value_disclosure() {
    let environment = TestEnvironment::new();
    let path = write_scenario(
        &environment,
        "workflow-missing-capture.json",
        &workflow_read_only_scenario(),
    );
    let mut command = workflow_command(&environment, &path, &[WORKFLOW_LOOKUP, WORKFLOW_READ]);
    command.arg("--format").arg("json");
    finish_workflow_command(&mut command, "workflow-missing-capture");
    let output = command
        .output()
        .expect("the missing capture should be diagnosed");
    let (_, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.is_empty());
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(
        report["primary_diagnosis"]["check_id"],
        "runtime.workflow.step[0]"
    );
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-WORKFLOW-001"
    );
    assert_eq!(
        report_check(&report, "runtime.workflow.step[1]")["state"],
        "skipped"
    );
    assert_redacted(
        &output,
        &[WORKFLOW_LOOKUP, WORKFLOW_READ, "private_resource_id"],
    );

    let environment = TestEnvironment::new();
    let mut invalid = workflow_read_only_scenario();
    invalid["steps"][0]["captures"]["private_resource_id"] = json!("not-a-json-pointer");
    let path = write_scenario(&environment, "workflow-invalid-pointer.json", &invalid);
    let marker = environment.artifact_path("invalid-pointer-started");
    let mut command = workflow_command(&environment, &path, &[WORKFLOW_LOOKUP, WORKFLOW_READ]);
    command
        .arg("--")
        .arg(fixture())
        .arg("active-started-marker")
        .arg(&marker);
    let rejected = command
        .output()
        .expect("the invalid pointer should be rejected");
    let (stdout, stderr) = text(&rejected);
    assert_eq!(rejected.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-SCENARIO-001"), "{stdout}");
    assert!(!marker.exists());
    assert_redacted(&rejected, &["not-a-json-pointer", "private_resource_id"]);
}

#[test]
fn workflow_cleanup_failure_remains_an_independent_critical_finding() {
    let tools = [WORKFLOW_LOOKUP, WORKFLOW_MUTATE, WORKFLOW_CLEANUP];
    let environment = TestEnvironment::new();
    let path = write_scenario(
        &environment,
        "workflow-cleanup-failure.json",
        &workflow_cleanup_scenario(),
    );
    let mut command = workflow_command(&environment, &path, &tools);
    command
        .arg("--allow-side-effects")
        .arg("--format")
        .arg("json");
    finish_workflow_command(&mut command, "workflow-cleanup-failure");
    let output = command.output().expect("the cleanup failure should run");
    let (_, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.is_empty());
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(
        report["primary_diagnosis"]["check_id"],
        "runtime.workflow.cleanup[2]"
    );
    assert!(
        report["independent_findings"]
            .as_array()
            .is_some_and(|findings| findings.iter().any(|finding| {
                finding["check_id"] == "runtime.workflow.cleanup[2]"
                    && finding["code"] == "MCP-SAFETY-003"
            }))
    );
    assert_redacted(&output, &tools);
}

#[test]
fn workflow_timeout_disconnect_and_legacy_selection_stop_causally_without_retry() {
    for mode in ["workflow-call-timeout", "workflow-disconnect"] {
        let environment = TestEnvironment::new();
        let path = write_scenario(
            &environment,
            &format!("{mode}.json"),
            &workflow_read_only_scenario(),
        );
        let mut command = workflow_command(&environment, &path, &[WORKFLOW_LOOKUP, WORKFLOW_READ]);
        command.arg("--format").arg("json");
        finish_workflow_command(&mut command, mode);
        let output = command
            .output()
            .expect("the failed transport should return");
        let (_, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(1));
        assert!(stderr.is_empty());
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(
            report_check(&report, "runtime.workflow.step[0]")["state"],
            "skipped"
        );
        assert_eq!(
            report_check(&report, "runtime.workflow.step[1]")["state"],
            "skipped"
        );
        assert_redacted(&output, &[WORKFLOW_LOOKUP, WORKFLOW_READ]);
    }

    let environment = TestEnvironment::new();
    let path = write_scenario(
        &environment,
        "workflow-legacy-rejected.json",
        &workflow_read_only_scenario(),
    );
    let marker = environment.artifact_path("legacy-workflow-started");
    let mut command = workflow_command(&environment, &path, &[WORKFLOW_LOOKUP, WORKFLOW_READ]);
    command
        .arg("--protocol-version")
        .arg("2025-11-25")
        .arg("--")
        .arg(fixture())
        .arg("active-started-marker")
        .arg(&marker);
    let output = command
        .output()
        .expect("legacy workflow selection should reject");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("unsupported_scenario_revision"), "{stdout}");
    assert!(!marker.exists());
    assert_redacted(&output, &[WORKFLOW_LOOKUP, WORKFLOW_READ]);
}

#[test]
fn explicit_legacy_check_uses_initialize_and_legacy_results_with_reporter_parity() {
    let environment = TestEnvironment::new();
    let scenario_path = write_scenario(
        &environment,
        "legacy-success.json",
        &scenario("read_only", vec![reviewed_case(0, "success")]),
    );
    let json_path = environment.artifact_path("legacy-report.json");
    let junit_path = environment.artifact_path("legacy-report.xml");
    let output = environment
        .command()
        .arg("check")
        .arg("--protocol-version")
        .arg("2025-11-25")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json")
        .arg("--json-report")
        .arg(&json_path)
        .arg("--junit-report")
        .arg(&junit_path)
        .arg("--")
        .arg(fixture())
        .arg("legacy-active-success")
        .output()
        .expect("the selected legacy success journey should run");
    let (_, stderr) = text(&output);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(stderr.is_empty(), "{stderr}");

    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["protocol_revision"], "2025-11-25");
    assert_eq!(report["negotiated_protocol_revision"], "2025-11-25");
    assert_eq!(report["outcome"], "passed");
    assert_eq!(
        parse_and_validate_report(
            &fs::read(&json_path).expect("the legacy JSON artifact should exist")
        ),
        report
    );
    let (junit, summary) = parse_and_validate_junit(
        &fs::read(&junit_path).expect("the legacy JUnit artifact should exist"),
    );
    assert_eq!(summary.failures, 0);
    assert_eq!(summary.skipped, 0);
    assert!(junit.contains("protocol_revision=2025-11-25"));
    assert!(junit.contains("negotiated_protocol_revision=2025-11-25"));
    assert_redacted(&output, &[TOOL, "sequence"]);
}

#[test]
fn explicit_v2025_06_check_requires_exact_schemas_and_preserves_reporter_parity() {
    let environment = TestEnvironment::new();
    let scenario_path = write_scenario(
        &environment,
        "v2025-06-success.json",
        &scenario("read_only", vec![reviewed_case(0, "success")]),
    );
    let json_path = environment.artifact_path("v2025-06-report.json");
    let junit_path = environment.artifact_path("v2025-06-report.xml");
    let output = environment
        .command()
        .arg("check")
        .arg("--protocol-version")
        .arg("2025-06-18")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json")
        .arg("--json-report")
        .arg(&json_path)
        .arg("--junit-report")
        .arg(&junit_path)
        .arg("--")
        .arg(fixture())
        .arg("legacy-active-success")
        .output()
        .expect("the MCP 2025-06-18 success journey should run");
    let (_, stderr) = text(&output);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(stderr.is_empty(), "{stderr}");

    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["protocol_revision"], "2025-06-18");
    assert_eq!(report["negotiated_protocol_revision"], "2025-06-18");
    assert_eq!(report["outcome"], "passed");
    assert_eq!(
        parse_and_validate_report(
            &fs::read(&json_path).expect("the MCP 2025-06-18 JSON artifact should exist")
        ),
        report
    );
    let (junit, summary) = parse_and_validate_junit(
        &fs::read(&junit_path).expect("the MCP 2025-06-18 JUnit artifact should exist"),
    );
    assert_eq!(summary.failures, 0);
    assert_eq!(summary.skipped, 0);
    assert!(junit.contains("protocol_revision=2025-06-18"));
    assert!(junit.contains("negotiated_protocol_revision=2025-06-18"));
    assert_redacted(&output, &[TOOL, "sequence"]);

    let human = run_v2025_06_check("legacy-active-success", &[], None);
    let (stdout, stderr) = text(&human);
    assert!(human.status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains("mcp-doctor report · MCP 2025-06-18"));
    assert!(stdout.contains("outcome passed · exit 0"));
    assert!(!stdout.contains("MCP-SCHEMA-004"));
    assert_redacted(&human, &[TOOL, "sequence"]);
}

#[test]
fn v2025_06_schema_dialect_failures_are_typed_and_stop_before_tools_call() {
    for (mode, code, rule, field) in [
        (
            "input-missing",
            "MCP-SCHEMA-002",
            "unsupported_schema_dialect",
            "inputSchema.$schema",
        ),
        (
            "input-malformed",
            "MCP-SCHEMA-002",
            "unsupported_schema_dialect",
            "inputSchema.$schema",
        ),
        (
            "input-wrong",
            "MCP-SCHEMA-002",
            "unsupported_schema_dialect",
            "inputSchema.$schema",
        ),
        (
            "input-vocabulary",
            "MCP-SCHEMA-001",
            "unsupported_schema_vocabulary",
            "inputSchema.$vocabulary",
        ),
        (
            "output-missing",
            "MCP-SCHEMA-002",
            "unsupported_schema_dialect",
            "outputSchema.$schema",
        ),
        (
            "output-malformed",
            "MCP-SCHEMA-002",
            "unsupported_schema_dialect",
            "outputSchema.$schema",
        ),
        (
            "output-wrong",
            "MCP-SCHEMA-002",
            "unsupported_schema_dialect",
            "outputSchema.$schema",
        ),
    ] {
        let output = run_v2025_06_check("legacy-active-2025-06-schema", &[mode], Some("json"));
        let (_, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(1), "{mode}: {stderr}");
        assert!(stderr.is_empty(), "{mode}: {stderr}");
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(report["protocol_revision"], "2025-06-18");
        assert_eq!(report["negotiated_protocol_revision"], "2025-06-18");
        assert_eq!(report["primary_diagnosis"]["check_id"], "schema.contracts");
        let schema_check = report["checks"]
            .as_array()
            .and_then(|checks| {
                checks
                    .iter()
                    .find(|check| check["id"] == "schema.contracts")
            })
            .expect("the failed schema check should remain in the report");
        let finding = &schema_check["findings"][0];
        assert_eq!(finding["code"], code, "{mode}: {report:#}");
        assert_eq!(finding["evidence"]["rule"], rule, "{mode}: {report:#}");
        assert!(
            finding["location"]
                .as_str()
                .is_some_and(|location| location.ends_with(field)),
            "{mode}: {report:#}"
        );
        let runtime = report["checks"]
            .as_array()
            .and_then(|checks| {
                checks
                    .iter()
                    .find(|check| check["id"] == "runtime.tools.case[0]")
            })
            .expect("the causally skipped case should remain in the report");
        assert_eq!(runtime["state"], "skipped");
        assert_eq!(runtime["blocked_by"]["check_id"], "schema.contracts");
        assert_redacted(&output, &[TOOL, "synthetic.invalid", "private-vocabulary"]);
    }
}

#[test]
fn v2025_06_schema_failure_has_human_json_and_junit_causal_parity() {
    let human = run_v2025_06_check("legacy-active-2025-06-schema", &["input-missing"], None);
    let json = run_v2025_06_check(
        "legacy-active-2025-06-schema",
        &["input-missing"],
        Some("json"),
    );
    let junit = run_v2025_06_check(
        "legacy-active-2025-06-schema",
        &["input-missing"],
        Some("junit"),
    );
    for output in [&human, &json, &junit] {
        let (_, stderr) = text(output);
        assert_eq!(output.status.code(), Some(1), "{stderr}");
        assert!(stderr.is_empty());
        assert_redacted(output, &[TOOL]);
    }

    let (human_stdout, _) = text(&human);
    assert!(human_stdout.contains("mcp-doctor report · MCP 2025-06-18"));
    assert!(human_stdout.contains("PRIMARY DIAGNOSIS · schema.contracts"));
    assert!(human_stdout.contains("MCP-SCHEMA-002"));
    assert!(human_stdout.contains("blocked by schema.contracts"));
    assert!(human_stdout.contains("outcome failed · exit 1"));

    let report = parse_and_validate_report(&json.stdout);
    assert_eq!(report["protocol_revision"], "2025-06-18");
    assert_eq!(report["negotiated_protocol_revision"], "2025-06-18");
    assert_eq!(report["primary_diagnosis"]["check_id"], "schema.contracts");
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-SCHEMA-002"
    );
    assert_eq!(report["outcome"], "failed");
    assert_eq!(report["exit_code"], 1);

    let (document, summary) = parse_and_validate_junit(&junit.stdout);
    assert_eq!(summary.failures, 1);
    assert_eq!(summary.skipped, 1);
    assert!(document.contains("protocol_revision=2025-06-18"));
    assert!(document.contains("negotiated_protocol_revision=2025-06-18"));
    assert!(document.contains("type=\"MCP-SCHEMA-002\""));
    assert!(document.contains("primary=true"));
    assert!(document.contains("blocked_by.check_id=schema.contracts"));
    assert!(document.contains("report_outcome=failed\nexit_code=1"));
}

#[test]
fn v2025_06_optional_and_exact_output_contracts_are_distinguished() {
    let standard_vocabulary = run_v2025_06_check(
        "legacy-active-2025-06-schema",
        &["input-standard-vocabulary"],
        Some("json"),
    );
    let (_, stderr) = text(&standard_vocabulary);
    assert!(standard_vocabulary.status.success(), "{stderr}");

    let omitted = run_v2025_06_check(
        "legacy-active-2025-06-schema",
        &["output-omitted"],
        Some("json"),
    );
    let (_, stderr) = text(&omitted);
    assert!(omitted.status.success(), "{stderr}");
    let report = parse_and_validate_report(&omitted.stdout);
    assert_eq!(report["outcome"], "passed");

    let mismatch = run_v2025_06_check(
        "legacy-active-2025-06-schema",
        &["output-mismatch"],
        Some("json"),
    );
    let (_, stderr) = text(&mismatch);
    assert_eq!(mismatch.status.code(), Some(1), "{stderr}");
    let report = parse_and_validate_report(&mismatch.stdout);
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-ACTIVE-005"
    );
    assert_redacted(
        &mismatch,
        &[TOOL, "synthetic-private-result-never-report-7f2c"],
    );
}

#[test]
fn v2025_06_external_and_over_limit_schemas_stop_before_activity() {
    for (mode, code, limit) in [
        ("input-external", "MCP-SCHEMA-003", None),
        ("input-depth", "MCP-LIMIT-001", Some("schema_depth")),
    ] {
        let output = run_v2025_06_check("legacy-active-2025-06-schema", &[mode], Some("json"));
        let (_, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(1), "{mode}: {stderr}");
        assert!(stderr.is_empty(), "{mode}: {stderr}");
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(report["protocol_revision"], "2025-06-18");
        assert_eq!(report["primary_diagnosis"]["check_id"], "schema.contracts");
        assert_eq!(
            report["primary_diagnosis"]["findings"][0]["code"], code,
            "{mode}: {report:#}"
        );
        if let Some(limit) = limit {
            let schema = report["checks"]
                .as_array()
                .and_then(|checks| {
                    checks
                        .iter()
                        .find(|check| check["id"] == "schema.contracts")
                })
                .expect("the failed schema check should remain in the report");
            assert_eq!(
                schema["findings"][0]["evidence"]["limit"], limit,
                "{mode}: {report:#}"
            );
        }
        let runtime = report["checks"]
            .as_array()
            .and_then(|checks| {
                checks
                    .iter()
                    .find(|check| check["id"] == "runtime.tools.case[0]")
            })
            .expect("the blocked runtime case should remain in the report");
        assert_eq!(runtime["state"], "skipped");
        assert_eq!(runtime["blocked_by"]["check_id"], "schema.contracts");
        assert_redacted(&output, &[TOOL, "synthetic.invalid", "private-schema"]);
    }
}

#[test]
fn v2025_06_legacy_result_and_additional_input_contracts_remain_value_free() {
    let tool_error = run_v2025_06_scenario(
        scenario("read_only", vec![reviewed_case(0, "tool_error")]),
        "legacy-active-tool-error",
        &[],
        Some("json"),
    );
    let (_, stderr) = text(&tool_error);
    assert!(tool_error.status.success(), "{stderr}");
    assert_eq!(
        parse_and_validate_report(&tool_error.stdout)["outcome"],
        "passed"
    );

    let invalid = run_v2025_06_check("legacy-active-invalid-result", &[], None);
    let (stdout, stderr) = text(&invalid);
    assert_eq!(invalid.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stdout.contains("MCP-ACTIVE-006"));

    for mode in [
        "legacy-active-url-elicitation",
        "legacy-active-server-request",
    ] {
        let output = run_v2025_06_check(mode, &[], Some("json"));
        let (_, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(3), "{mode}: {stderr}");
        assert!(stderr.is_empty());
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(report["outcome"], "incomplete");
        assert_eq!(report["exit_code"], 3);
        assert_redacted(
            &output,
            &[
                TOOL,
                "synthetic-server-request-never-report-7f2c",
                "synthetic.invalid",
            ],
        );
    }
    assert_redacted(&tool_error, &[TOOL, "sequence"]);
    assert_redacted(&invalid, &[TOOL]);
}

#[test]
fn v2025_06_handshake_failures_stop_without_fallback() {
    let mismatch = run_v2025_06_check("legacy-active-revision-mismatch", &[], Some("json"));
    let (_, stderr) = text(&mismatch);
    assert_eq!(mismatch.status.code(), Some(1), "{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_report(&mismatch.stdout);
    assert_eq!(report["protocol_revision"], "2025-06-18");
    assert_eq!(report["negotiated_protocol_revision"], "2025-11-25");
    assert_eq!(report["primary_diagnosis"]["check_id"], "protocol.revision");
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-PROTOCOL-005"
    );

    for (mode, expected) in [
        ("legacy-malformed", "MCP-PROTOCOL-003"),
        ("legacy-oversized", "message_bytes"),
    ] {
        let output = run_v2025_06_check(mode, &[], None);
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(1), "{mode}: {stdout}\n{stderr}");
        assert!(stderr.is_empty());
        assert!(stdout.contains(expected), "{mode}: {stdout}");
        assert!(stdout.contains("SKIP  runtime.tools.case[0]"));
        assert_redacted(&output, &[TOOL]);
    }

    let timeout = run_v2025_06_check("legacy-timeout", &[], None);
    let (stdout, stderr) = text(&timeout);
    assert_eq!(timeout.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-LIMIT-001"));
    assert!(stdout.contains("discovery_time"));
    assert_redacted(&timeout, &[TOOL]);
}

#[test]
fn legacy_tool_error_and_malformed_result_follow_the_selected_result_contract() {
    let environment = TestEnvironment::new();
    let scenario_path = write_scenario(
        &environment,
        "legacy-tool-error.json",
        &scenario("read_only", vec![reviewed_case(0, "tool_error")]),
    );
    let tool_error = environment
        .command()
        .arg("check")
        .arg("--protocol-version")
        .arg("2025-11-25")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json")
        .arg("--")
        .arg(fixture())
        .arg("legacy-active-tool-error")
        .output()
        .expect("the legacy tool-error check should start");
    let (_, stderr) = text(&tool_error);
    assert!(tool_error.status.success(), "{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_report(&tool_error.stdout);
    assert_eq!(report["protocol_revision"], "2025-11-25");
    assert_eq!(report["negotiated_protocol_revision"], "2025-11-25");
    assert_eq!(report["outcome"], "passed");
    assert_redacted(&tool_error, &[TOOL, "sequence"]);

    let invalid = run_legacy_check("legacy-active-invalid-result");
    let (stdout, stderr) = text(&invalid);
    assert_eq!(invalid.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-ACTIVE-006"), "{stdout}");
    assert!(stdout.contains("scenario.cases[0].content"), "{stdout}");
    assert_redacted(&invalid, &[TOOL]);
}

#[test]
fn legacy_required_tasks_stop_before_tools_call_with_actionable_diagnosis() {
    let output = run_legacy_check("legacy-active-task-required");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-ACTIVE-007"), "{stdout}");
    assert!(stdout.contains("task augmentation"), "{stdout}");
    assert!(stdout.contains("SKIP  runtime.tools.case[0]"), "{stdout}");
    assert_redacted(&output, &[TOOL]);
}

#[test]
fn legacy_initialize_mismatch_and_missing_tools_stop_before_initialized() {
    let mismatch = run_legacy_check_json("legacy-active-revision-mismatch");
    let (_, stderr) = text(&mismatch);
    assert_eq!(mismatch.status.code(), Some(1), "{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_report(&mismatch.stdout);
    assert_eq!(report["protocol_revision"], "2025-11-25");
    assert_eq!(report["negotiated_protocol_revision"], "2025-06-18");
    assert_eq!(report["primary_diagnosis"]["check_id"], "protocol.revision");
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-PROTOCOL-005"
    );
    assert_redacted(&mismatch, &[TOOL]);

    let missing = run_legacy_check("legacy-active-no-tools");
    let (stdout, stderr) = text(&missing);
    assert_eq!(missing.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-ACTIVE-001"), "{stdout}");
    assert!(stdout.contains("SKIP  runtime.tools.case[0]"), "{stdout}");
    assert_redacted(&missing, &[TOOL]);
}

#[test]
fn legacy_active_handshake_failures_retain_protocol_and_resource_bounds() {
    for (mode, expected) in [
        ("legacy-malformed", "MCP-PROTOCOL-003"),
        ("legacy-oversized", "message_bytes"),
    ] {
        let output = run_legacy_check(mode);
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(1), "{mode}: {stdout}\n{stderr}");
        assert!(stderr.is_empty(), "{mode}: {stderr}");
        assert!(stdout.contains(expected), "{mode}: {stdout}");
        assert!(stdout.contains("SKIP  discovery.catalogs"), "{stdout}");
        assert!(stdout.contains("SKIP  runtime.tools.case[0]"), "{stdout}");
        assert_redacted(&output, &[TOOL]);
    }

    let output = run_legacy_check("legacy-timeout");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("MCP-LIMIT-001"), "{stdout}");
    assert!(stdout.contains("discovery_time"), "{stdout}");
    assert!(stdout.contains("SKIP  runtime.tools.case[0]"), "{stdout}");
    assert_redacted(&output, &[TOOL]);
}

#[test]
fn legacy_additional_input_signals_are_incomplete_without_answer_or_retry() {
    for mode in [
        "legacy-active-url-elicitation",
        "legacy-active-server-request",
    ] {
        let output = run_legacy_check(mode);
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(3), "{mode}: {stdout}\n{stderr}");
        assert!(stderr.is_empty(), "{mode}: {stderr}");
        assert!(stdout.contains("outcome incomplete · exit 3"), "{stdout}");
        assert!(!stdout.contains("MCP-ACTIVE-003"), "{stdout}");
        assert_redacted(
            &output,
            &[
                TOOL,
                "synthetic-server-request-never-report-7f2c",
                "synthetic.invalid",
            ],
        );
    }
}

#[test]
fn legacy_server_request_stops_before_any_later_case() {
    let environment = TestEnvironment::new();
    let scenario_path = write_scenario(
        &environment,
        "legacy-server-request.json",
        &scenario(
            "read_only",
            vec![reviewed_case(0, "success"), reviewed_case(1, "success")],
        ),
    );
    let output = environment
        .command()
        .arg("check")
        .arg("--protocol-version")
        .arg("2025-11-25")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json")
        .arg("--")
        .arg(fixture())
        .arg("legacy-active-server-request")
        .output()
        .expect("the legacy server-request boundary should run");
    let (_, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(3), "{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_report(&output.stdout);
    let cases = report["checks"]
        .as_array()
        .expect("checks should be an array")
        .iter()
        .filter(|check| {
            check["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("runtime.tools.case["))
        })
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 2);
    assert!(cases.iter().all(|case| case["state"] == "skipped"));
    assert!(
        cases
            .iter()
            .all(|case| case["skip_reason"] == "input_required")
    );
    assert_redacted(
        &output,
        &[TOOL, "synthetic-server-request-never-report-7f2c"],
    );
}

#[test]
fn unrelated_legacy_server_requests_are_protocol_failures_not_input_requests() {
    let output = run_legacy_check("legacy-active-unexpected-request");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-ACTIVE-006"), "{stdout}");
    assert!(!stdout.contains("outcome incomplete"), "{stdout}");
    assert_redacted(
        &output,
        &[TOOL, "synthetic-server-request-never-report-7f2c"],
    );
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
fn slow_start_profile_does_not_grant_tool_or_side_effect_authority() {
    for (effects, allowed_tool, expected_code) in [
        ("read_only", "synthetic.other", "MCP-AUTH-001"),
        ("side_effecting", TOOL, "MCP-AUTH-002"),
    ] {
        let environment = TestEnvironment::new();
        let path = write_scenario(
            &environment,
            "profile-authority.json",
            &scenario(effects, vec![reviewed_case(0, "success")]),
        );
        let marker = environment.artifact_path("profile-target-started");
        let output = environment
            .command()
            .arg("check")
            .arg("--limit-profile")
            .arg("slow-start")
            .arg("--scenario")
            .arg(&path)
            .arg("--allow-tool")
            .arg(allowed_tool)
            .arg("--format")
            .arg("json")
            .arg("--")
            .arg(fixture())
            .arg("active-started-marker")
            .arg(&marker)
            .output()
            .expect("the profile authorization rejection should run");
        let (_, stderr) = text(&output);
        let report = parse_and_validate_report(&output.stdout);

        assert_eq!(output.status.code(), Some(2), "{report:#}\n{stderr}");
        assert!(stderr.is_empty());
        assert_eq!(report["limits"]["profile"], "slow-start");
        assert_eq!(report["limits"]["total_ms"], 240_000);
        assert_eq!(
            report["primary_diagnosis"]["findings"][0]["code"],
            expected_code
        );
        assert!(!marker.exists(), "the limit profile started the target");
        assert_redacted(&output, &[TOOL, allowed_tool]);
    }
}

#[test]
fn v2025_06_side_effect_authority_is_required_before_target_start() {
    let environment = TestEnvironment::new();
    let path = write_scenario(
        &environment,
        "v2025-06-side-effects.json",
        &scenario("side_effecting", vec![reviewed_case(0, "success")]),
    );
    let marker = environment.artifact_path("v2025-06-target-started");
    let output = v2025_06_check_command(&environment, &path, "active-started-marker")
        .arg(&marker)
        .output()
        .expect("the explicit 2025-06-18 authorization rejection should run");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("MCP-AUTH-002"), "{stdout}");
    assert!(stdout.contains("SKIP  transport.stdio"), "{stdout}");
    assert!(
        !marker.exists(),
        "the rejected 2025-06-18 target was started"
    );
    assert_redacted(&output, &[TOOL]);
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

    let output = run_legacy_check("legacy-active-schema-external");
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

    let environment = TestEnvironment::new();
    let mut document = scenario("read_only", vec![reviewed_case(0, "success")]);
    document["cases"][0]["expect"]["structured_output_schema"] = json!({
        "type": "string",
        "pattern": "a".repeat(100_001)
    });
    let path = write_scenario(&environment, "schema-work-before-target.json", &document);
    let marker = environment.artifact_path("schema-work-target-started");
    let output = check_command(&environment, &path, TOOL, "active-started-marker")
        .arg(&marker)
        .output()
        .expect("schema work exhaustion should be rejected before target preparation");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(3), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-SCHEMA-005"), "{stdout}");
    assert!(stdout.contains("compile_construction"), "{stdout}");
    assert!(stdout.contains("schema_evaluation_steps"), "{stdout}");
    assert!(!marker.exists());
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
fn schema_operation_exhaustion_stops_before_the_reviewed_tool_call() {
    let mut case = reviewed_case(0, "success");
    case["arguments"]["sequence"] =
        Value::Array((0..2_000).map(|_| json!(999)).collect::<Vec<_>>());
    let output = run_check(
        scenario("read_only", vec![case]),
        "active-input-evaluation-limit",
    );
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-LIMIT-001"), "{stdout}");
    assert!(stdout.contains("schema_evaluation_steps"), "{stdout}");
    assert!(stdout.contains("FAIL  runtime.tools.case[0]"), "{stdout}");
    assert_redacted(&output, &[TOOL]);

    let mut pattern_case = reviewed_case(0, "success");
    pattern_case["arguments"]["sequence"] = Value::String("a".repeat(100));
    let output = run_check(
        scenario("read_only", vec![pattern_case]),
        "active-input-pattern-evaluation-limit",
    );
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-LIMIT-001"), "{stdout}");
    assert!(stdout.contains("schema_evaluation_steps"), "{stdout}");
    assert!(stdout.contains("FAIL  runtime.tools.case[0]"), "{stdout}");
    assert_redacted(&output, &[TOOL]);

    let environment = TestEnvironment::new();
    let mut case = reviewed_case(0, "success");
    case["arguments"]["sequence"] =
        Value::Array((0..2_000).map(|_| json!(999)).collect::<Vec<_>>());
    let path = write_scenario(
        &environment,
        "schema-work-artifacts.json",
        &scenario("read_only", vec![case]),
    );
    let json_path = environment.artifact_path("schema-work-report.json");
    let junit_path = environment.artifact_path("schema-work-report.xml");
    let output = environment
        .command()
        .arg("check")
        .arg("--scenario")
        .arg(&path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--json-report")
        .arg(&json_path)
        .arg("--junit-report")
        .arg(&junit_path)
        .arg("--")
        .arg(fixture())
        .arg("active-input-evaluation-limit")
        .output()
        .expect("schema work exhaustion should produce all reports without a tool call");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("schema_evaluation_steps"), "{stdout}");

    let json_report = fs::read(&json_path).expect("the JSON report artifact should exist");
    assert!(!String::from_utf8_lossy(&json_report).contains(TOOL));
    let report = parse_and_validate_report(&json_report);
    assert_eq!(
        report["primary_diagnosis"]["check_id"],
        "runtime.tools.case[0]"
    );
    assert_eq!(
        report_check(&report, "runtime.tools.case[0]")["findings"][0]["evidence"]["limit"],
        "schema_evaluation_steps"
    );
    assert_eq!(report["outcome"], "failed");
    assert_eq!(report["exit_code"], 1);

    let junit_report = fs::read(&junit_path).expect("the JUnit report artifact should exist");
    assert!(!String::from_utf8_lossy(&junit_report).contains(TOOL));
    let (junit, summary) = parse_and_validate_junit(&junit_report);
    assert_eq!(summary.failures, 1);
    assert!(junit.contains("runtime.tools.case[0]"));
    assert!(junit.contains("schema_evaluation_steps"));
    assert_redacted(&output, &[TOOL]);
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
    let marker = environment.artifact_path("descendant-ready");
    let output = check_command(&environment, &path, TOOL, "active-resistant-child")
        .arg(&marker)
        .output()
        .expect("the active cleanup journey should run");
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert_descendant_was_ready_and_terminated(&marker);
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
