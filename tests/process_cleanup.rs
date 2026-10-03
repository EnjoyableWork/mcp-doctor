#![cfg(all(unix, feature = "internal-test-fixtures"))]

mod support;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::{Value, json};
use support::{
    TestEnvironment, parse_and_validate_junit, parse_and_validate_markdown,
    parse_and_validate_report,
};

const REVISION: &str = "2026-07-28";
const REVIEWED_TOOL: &str = "synthetic.reviewed";
const GENERATED_TOOL: &str = "synthetic.generated";
const SENTINEL: &str = "synthetic-secret-payload-7f2c";

fn fixture() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_mcp-doctor-stdio-fixture"))
}

fn inspect_command(environment: &TestEnvironment, revision: &str) -> Command {
    let mut command = environment.command();
    command
        .arg("inspect")
        .arg("--protocol-version")
        .arg(revision);
    command
}

fn auto_command(environment: &TestEnvironment) -> Command {
    let mut command = inspect_command(environment, "auto");
    command
        .args(["--format", "json", "--"])
        .arg(fixture())
        .arg("auto-legacy")
        .arg(environment.artifact_path("synthetic-cleanup-auto-state"))
        .args(["method-not-found", "2025-11-25"]);
    command
}

fn assert_redacted(document: &str, paths: &[&Path]) {
    for forbidden in [
        SENTINEL,
        "synthetic-secret-payload-never-report-7f2c",
        REVIEWED_TOOL,
        GENERATED_TOOL,
        "mcp-doctor-stdio-fixture",
        "\"pid\"",
        "\"pgid\"",
        "pid=",
        "pgid=",
    ] {
        assert!(
            !document.contains(forbidden),
            "a report disclosed an untrusted value or process identifier"
        );
    }
    for path in paths.iter().copied().chain(std::iter::once(fixture())) {
        assert!(
            !document.contains(path.to_str().expect("synthetic paths should be UTF-8")),
            "a report disclosed a target or artifact path"
        );
    }
}

fn json_report(output: &Output, exit_code: i32) -> Value {
    assert_eq!(output.status.code(), Some(exit_code));
    assert!(output.stderr.is_empty());
    let document = std::str::from_utf8(&output.stdout).expect("JSON output should be UTF-8");
    assert_redacted(document, &[]);
    parse_and_validate_report(&output.stdout)
}

fn assert_cleanup(report: &Value, launches: u64, reaped: u64) {
    assert_eq!(
        report["process_cleanup"],
        json!({
            "mechanism": "process_group",
            "scope": "direct_child_and_original_process_group",
            "process_launches": launches,
            "direct_children_reaped": reaped,
            "descendant_containment": false,
            "detached_descendants": "unverified"
        })
    );
    let observed_launches = report["process_cleanup"]["process_launches"]
        .as_u64()
        .expect("successful spawns should be a bounded count");
    let observed_reaps = report["process_cleanup"]["direct_children_reaped"]
        .as_u64()
        .expect("successful owned-child waits should be a bounded count");
    assert!(observed_launches <= 2);
    assert!(observed_reaps <= observed_launches);
}

fn cleanup_line(launches: u64, reaped: u64) -> String {
    format!(
        "process cleanup · mechanism=process_group · scope=direct_child_and_original_process_group · process_launches={launches} · direct_children_reaped={reaped} · descendant_containment=false · detached_descendants=unverified"
    )
}

#[test]
fn exact_inspect_projects_one_observed_child_into_every_reporter() {
    let environment = TestEnvironment::new();
    let json_path = environment.artifact_path("synthetic-cleanup-report.json");
    let junit_path = environment.artifact_path("synthetic-cleanup-report.xml");
    let markdown_path = environment.artifact_path("synthetic-cleanup-report.md");
    let unexpected_request = environment.artifact_path("synthetic-cleanup-unexpected-request");
    let output = inspect_command(&environment, REVISION)
        .arg("--json-report")
        .arg(&json_path)
        .arg("--junit-report")
        .arg(&junit_path)
        .arg("--markdown-report")
        .arg(&markdown_path)
        .arg("--")
        .arg(fixture())
        .arg("success")
        .arg(&unexpected_request)
        .output()
        .expect("the exact passive fixture should return");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(!unexpected_request.exists());
    let human = std::str::from_utf8(&output.stdout).expect("human output should be UTF-8");
    let json_bytes = fs::read(&json_path).expect("the requested JSON report should exist");
    let report = parse_and_validate_report(&json_bytes);
    assert_eq!(report["outcome"], "passed");
    assert_cleanup(&report, 1, 1);
    assert_eq!(report["protocol_selection"]["process_launches"], 1);
    let (junit, summary) = parse_and_validate_junit(
        &fs::read(&junit_path).expect("the requested JUnit report should exist"),
    );
    assert_eq!(summary.failures, 0);
    let markdown = parse_and_validate_markdown(
        &fs::read(&markdown_path).expect("the requested Markdown report should exist"),
    );
    assert!(human.contains(&cleanup_line(1, 1)));
    assert!(markdown.contains(&cleanup_line(1, 1)));
    for evidence in [
        "process_cleanup.mechanism=process_group",
        "process_cleanup.scope=direct_child_and_original_process_group",
        "process_cleanup.process_launches=1",
        "process_cleanup.direct_children_reaped=1",
        "process_cleanup.descendant_containment=false",
        "process_cleanup.detached_descendants=unverified",
    ] {
        assert!(
            junit.contains(evidence),
            "JUnit omitted safe cleanup evidence"
        );
    }
    for document in [
        human,
        std::str::from_utf8(&json_bytes).unwrap(),
        &junit,
        &markdown,
    ] {
        assert_redacted(
            document,
            &[&json_path, &junit_path, &markdown_path, &unexpected_request],
        );
    }
}

#[test]
fn failed_spawn_reports_zero_owned_children_without_disclosing_the_target() {
    let environment = TestEnvironment::new();
    let missing = environment.artifact_path("synthetic-cleanup-missing-target-7f2c");
    let output = inspect_command(&environment, REVISION)
        .args(["--format", "json", "--"])
        .arg(&missing)
        .output()
        .expect("a missing target should produce a diagnostic");
    let report = json_report(&output, 1);
    assert_cleanup(&report, 0, 0);
    assert_eq!(report["protocol_selection"]["process_launches"], 0);
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-TRANSPORT-001"
    );
    assert_redacted(std::str::from_utf8(&output.stdout).unwrap(), &[&missing]);
}

#[test]
fn passive_auto_sums_observed_spawns_and_reaps_across_both_lifecycles() {
    let environment = TestEnvironment::new();
    let output = auto_command(&environment)
        .output()
        .expect("the finite legacy transition should return");
    let report = json_report(&output, 0);
    assert_cleanup(&report, 2, 2);
    assert_eq!(report["protocol_selection"]["process_launches"], 2);
    assert_eq!(report["protocol_selection"]["fallbacks"], 1);
    assert_eq!(report["protocol_revision"], "2025-11-25");
}

#[test]
fn passive_auto_preserves_the_first_reap_when_the_next_launch_exhausts_its_budget() {
    let environment = TestEnvironment::new();
    let output = auto_command(&environment)
        .env("MCP_DOCTOR_INTERNAL_TEST_EXHAUST_AUTO_TOTAL_BUDGET", "1")
        .output()
        .expect("pre-spawn total-budget exhaustion should return");
    let report = json_report(&output, 1);
    assert_cleanup(&report, 1, 1);
    assert_eq!(report["protocol_selection"]["process_launches"], 1);
    assert_eq!(report["protocol_selection"]["fallbacks"], 1);
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-LIMIT-001"
    );
}

#[test]
fn an_independent_cleanup_failure_does_not_erase_an_observed_direct_child_reap() {
    let environment = TestEnvironment::new();
    let output = inspect_command(&environment, REVISION)
        .env("MCP_DOCTOR_INTERNAL_TEST_CLEANUP_FAILURE", "1")
        .args(["--format", "json", "--"])
        .arg(fixture())
        .arg("malformed")
        .output()
        .expect("the independent forced failure should return");
    let report = json_report(&output, 1);
    assert_cleanup(&report, 1, 1);
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-TRANSPORT-003"
    );
    assert_eq!(report["independent_findings"][0]["code"], "MCP-SAFETY-001");
    let finding = report["checks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|check| check["findings"].as_array().unwrap())
        .find(|finding| finding["code"] == "MCP-SAFETY-001")
        .expect("the observed cleanup failure should remain a finding");
    assert_eq!(finding["severity"], "critical");
}

#[test]
fn exact_authorized_active_commands_report_the_same_observed_cleanup_scope() {
    for command_name in ["check", "break", "reject"] {
        let environment = TestEnvironment::new();
        let scenario_path = environment.artifact_path("synthetic-cleanup-scenario.json");
        let marker = environment.artifact_path("synthetic-cleanup-call-marker");
        let mut command = environment.command();
        command.arg(command_name).args(["--format", "json"]);
        if command_name != "reject" {
            command.args(["--protocol-version", REVISION]);
        }
        let mode = match command_name {
            "check" => {
                fs::write(
                    &scenario_path,
                    serde_json::to_vec(&json!({
                        "schema_version": "mcp-doctor.scenario/v1alpha1",
                        "tool": REVIEWED_TOOL,
                        "safety": {"effects": "read_only"},
                        "cases": [{
                            "id": "synthetic-cleanup-case-never-report",
                            "arguments": {"sequence": 0},
                            "expect": {
                                "result": "success",
                                "structured_output_schema": {
                                    "type": "object",
                                    "properties": {"ok": {"type": "boolean"}},
                                    "required": ["ok"],
                                    "additionalProperties": false
                                }
                            }
                        }]
                    }))
                    .unwrap(),
                )
                .expect("the reviewed synthetic scenario should be writable");
                command
                    .arg("--scenario")
                    .arg(&scenario_path)
                    .args(["--allow-tool", REVIEWED_TOOL]);
                "active-one-success"
            }
            "break" => {
                command.args([
                    "--tool",
                    GENERATED_TOOL,
                    "--allow-tool",
                    GENERATED_TOOL,
                    "--effects",
                    "read_only",
                    "--cases",
                    "1",
                    "--seed",
                    "4242",
                ]);
                "break-success"
            }
            "reject" => {
                command.args([
                    "--tool",
                    REVIEWED_TOOL,
                    "--allow-tool",
                    REVIEWED_TOOL,
                    "--effects",
                    "read_only",
                    "--seed",
                    "4242",
                ]);
                "reject-success"
            }
            _ => unreachable!(),
        };
        command.arg("--").arg(fixture()).arg(mode);
        match command_name {
            "break" => {
                command.arg(&marker).arg("1");
            }
            "reject" => {
                command.arg(&marker);
            }
            _ => {}
        }
        let output = command
            .output()
            .expect("the exact authorized synthetic command should return");
        assert!(
            output.status.success(),
            "the reviewed {command_name} journey failed"
        );
        let report = json_report(&output, 0);
        assert_cleanup(&report, 1, 1);
        assert_eq!(report["protocol_revision"], REVISION);
        assert_eq!(report["outcome"], "passed");
        assert_redacted(
            std::str::from_utf8(&output.stdout).unwrap(),
            &[&scenario_path, &marker],
        );
        assert!(
            !String::from_utf8_lossy(&output.stdout)
                .contains("synthetic-cleanup-case-never-report")
        );
    }
}
