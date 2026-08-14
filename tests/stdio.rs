#![cfg(feature = "internal-test-fixtures")]

mod support;

use std::ffi::OsStr;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};

use support::{
    TestEnvironment, assert_descendant_was_ready_and_terminated, parse_and_validate_junit,
    parse_and_validate_report, validate_report_value,
};

const REDACTION_SENTINEL: &str = "synthetic-secret-payload-7f2c";
const CATALOG_SENTINEL: &str = "synthetic-secret-payload-never-report-7f2c";
const REPORT_ONLY_HUMAN: &str = include_str!("fixtures/reports/unsupported-revision.txt");
const REPORT_ONLY_JSON: &str = include_str!("fixtures/reports/unsupported-revision.json");

fn fixture() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_mcp-doctor-stdio-fixture"))
}

fn inspect_command(environment: &TestEnvironment, mode: &str) -> Command {
    let mut command = environment.command();
    command.arg("inspect").arg("--").arg(fixture()).arg(mode);
    command
}

fn json_inspect_command(environment: &TestEnvironment, mode: &str) -> Command {
    let mut command = environment.command();
    command
        .arg("inspect")
        .arg("--format")
        .arg("json")
        .arg("--")
        .arg(fixture())
        .arg(mode);
    command
}

fn junit_inspect_command(environment: &TestEnvironment, mode: &str) -> Command {
    let mut command = environment.command();
    command
        .arg("inspect")
        .arg("--format")
        .arg("junit")
        .arg("--")
        .arg(fixture())
        .arg(mode);
    command
}

fn legacy_inspect_command(
    environment: &TestEnvironment,
    revision: &str,
    format: Option<&str>,
    mode: &str,
) -> Command {
    let mut command = environment.command();
    command
        .arg("inspect")
        .arg("--protocol-version")
        .arg(revision);
    if let Some(format) = format {
        command.arg("--format").arg(format);
    }
    command.arg("--").arg(fixture()).arg(mode);
    command
}

fn run_mode(mode: &str) -> Output {
    let environment = TestEnvironment::new();
    inspect_command(&environment, mode)
        .output()
        .expect("mcp-doctor should inspect the fixture")
}

fn run_json_mode(mode: &str) -> Output {
    let environment = TestEnvironment::new();
    json_inspect_command(&environment, mode)
        .output()
        .expect("mcp-doctor should inspect the fixture as JSON")
}

fn text(output: &Output) -> (&str, &str) {
    let stdout = std::str::from_utf8(&output.stdout).expect("STDOUT should be UTF-8");
    let stderr = std::str::from_utf8(&output.stderr).expect("STDERR should be UTF-8");
    (stdout, stderr)
}

fn json_report(output: &Output) -> serde_json::Value {
    parse_and_validate_report(&output.stdout)
}

#[test]
fn explicit_legacy_stdio_revisions_initialize_once_and_remain_passive() {
    for revision in ["2025-11-25", "2025-06-18"] {
        let environment = TestEnvironment::new();
        let json_path = environment.artifact_path("legacy-report.json");
        let junit_path = environment.artifact_path("legacy-report.xml");
        let output = environment
            .command()
            .arg("inspect")
            .arg("--protocol-version")
            .arg(revision)
            .arg("--format")
            .arg("json")
            .arg("--json-report")
            .arg(&json_path)
            .arg("--junit-report")
            .arg(&junit_path)
            .arg("--")
            .arg(fixture())
            .arg("legacy-success")
            .output()
            .expect("mcp-doctor should inspect the selected legacy revision");
        let (stdout, stderr) = text(&output);
        assert!(output.status.success(), "{revision}: {stdout}\n{stderr}");
        assert!(stderr.is_empty(), "{revision}: {stderr}");
        let report = json_report(&output);
        assert_eq!(report["protocol_revision"], revision);
        assert_eq!(report["negotiated_protocol_revision"], revision);
        assert_eq!(report["outcome"], "passed");
        assert_eq!(report["summary"]["failed"], 0);
        let artifact = parse_and_validate_report(
            &std::fs::read(&json_path).expect("the legacy JSON artifact should exist"),
        );
        assert_eq!(artifact["protocol_revision"], revision);
        assert_eq!(artifact["negotiated_protocol_revision"], revision);
        let (junit_artifact, _) = parse_and_validate_junit(
            &std::fs::read(&junit_path).expect("the legacy JUnit artifact should exist"),
        );
        assert!(junit_artifact.contains(&format!("protocol_revision={revision}")));
        assert!(!stdout.contains(CATALOG_SENTINEL));

        let human = legacy_inspect_command(&environment, revision, None, "legacy-success")
            .output()
            .expect("mcp-doctor should render the selected legacy revision for a person");
        let (human_stdout, human_stderr) = text(&human);
        assert!(
            human.status.success(),
            "{revision}: {human_stdout}\n{human_stderr}"
        );
        assert!(human_stderr.is_empty());
        assert!(
            human_stdout.contains(&format!(
                "protocol selection · selected {revision} · negotiated {revision}"
            )),
            "{human_stdout}"
        );
        assert!(!human_stdout.contains(CATALOG_SENTINEL));

        let junit = legacy_inspect_command(&environment, revision, Some("junit"), "legacy-success")
            .output()
            .expect("mcp-doctor should project the legacy result as JUnit");
        assert!(junit.status.success(), "{revision}: {:?}", text(&junit));
        let (junit_text, _) = parse_and_validate_junit(&junit.stdout);
        assert!(
            junit_text.contains(&format!("protocol_revision={revision}")),
            "{junit_text}"
        );
        assert!(
            junit_text.contains(&format!("negotiated_protocol_revision={revision}")),
            "{junit_text}"
        );
    }
}

#[test]
fn legacy_stdio_revision_mismatch_and_malformed_values_fail_without_fallback() {
    for revision in ["2025-11-25", "2025-06-18"] {
        let environment = TestEnvironment::new();
        let mismatch =
            legacy_inspect_command(&environment, revision, Some("json"), "legacy-mismatch")
                .output()
                .expect("mcp-doctor should diagnose a legacy mismatch");
        let (stdout, stderr) = text(&mismatch);
        assert_eq!(
            mismatch.status.code(),
            Some(1),
            "{revision}: {stdout}\n{stderr}"
        );
        assert!(stderr.is_empty());
        let report = json_report(&mismatch);
        assert_eq!(report["protocol_revision"], revision);
        assert_ne!(report["negotiated_protocol_revision"], revision);
        assert_eq!(report["primary_diagnosis"]["check_id"], "protocol.revision");
        assert_eq!(
            report["primary_diagnosis"]["findings"][0]["code"],
            "MCP-PROTOCOL-005"
        );

        let malformed = legacy_inspect_command(&environment, revision, None, "legacy-malformed")
            .output()
            .expect("mcp-doctor should diagnose a malformed negotiated revision");
        let (stdout, stderr) = text(&malformed);
        assert_eq!(
            malformed.status.code(),
            Some(1),
            "{revision}: {stdout}\n{stderr}"
        );
        assert!(stderr.is_empty());
        assert!(stdout.contains("MCP-PROTOCOL-003"), "{stdout}");
        assert!(!stdout.contains(REDACTION_SENTINEL), "{stdout}");
    }
}

#[test]
fn legacy_stdio_revisions_retain_message_and_discovery_bounds() {
    for revision in ["2025-11-25", "2025-06-18"] {
        let environment = TestEnvironment::new();
        let oversized = legacy_inspect_command(&environment, revision, None, "legacy-oversized")
            .output()
            .expect("mcp-doctor should bound a legacy response");
        let (stdout, stderr) = text(&oversized);
        assert_eq!(
            oversized.status.code(),
            Some(1),
            "{revision}: {stdout}\n{stderr}"
        );
        assert!(stderr.is_empty());
        assert!(stdout.contains("message_bytes"), "{stdout}");
    }
}

#[test]
fn legacy_stdio_schema_dialects_follow_the_selected_revision() {
    let environment = TestEnvironment::new();
    let ambiguous =
        legacy_inspect_command(&environment, "2025-06-18", None, "legacy-ambiguous-schema")
            .output()
            .expect("mcp-doctor should report an ambiguous 2025-06-18 schema dialect");
    let (stdout, stderr) = text(&ambiguous);
    assert!(ambiguous.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-SCHEMA-004"), "{stdout}");
    assert!(stdout.contains("inputSchema.$schema"), "{stdout}");
    assert!(
        stdout.contains("bounded structural checks only"),
        "{stdout}"
    );

    let defaulted =
        legacy_inspect_command(&environment, "2025-11-25", None, "legacy-ambiguous-schema")
            .output()
            .expect("mcp-doctor should apply the 2025-11-25 default schema dialect");
    let (stdout, stderr) = text(&defaulted);
    assert!(defaulted.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(!stdout.contains("MCP-SCHEMA-004"), "{stdout}");
}

#[test]
fn legacy_stdio_revisions_retain_the_discovery_deadline() {
    for revision in ["2025-11-25", "2025-06-18"] {
        let environment = TestEnvironment::new();
        let output = legacy_inspect_command(&environment, revision, None, "legacy-timeout")
            .output()
            .expect("mcp-doctor should bound an unresponsive legacy server");
        let (stdout, stderr) = text(&output);

        assert_eq!(
            output.status.code(),
            Some(1),
            "{revision}: {stdout}\n{stderr}"
        );
        assert!(stderr.is_empty(), "{revision}: {stderr}");
        assert!(stdout.contains("MCP-LIMIT-001"), "{revision}: {stdout}");
        assert!(stdout.contains("discovery_time"), "{revision}: {stdout}");
    }
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
    let output = run_mode("timeout");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-LIMIT-001"), "{stdout}");
    assert!(stdout.contains("discovery_time"), "{stdout}");
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
    let readiness_marker = environment.artifact_path("descendant-ready");
    let output = inspect_command(&environment, "resistant-child")
        .arg(&readiness_marker)
        .output()
        .expect("mcp-doctor should inspect the resistant fixture");
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert_descendant_was_ready_and_terminated(&readiness_marker);
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
fn stable_json_is_a_complete_passive_built_binary_report() {
    let output = run_json_mode("catalog-valid");
    let (_, stderr) = text(&output);
    let report = json_report(&output);

    assert!(output.status.success(), "{report:#}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(report["schema_version"], "mcp-doctor.report/v1");
    assert_eq!(report["schema_stability"], "stable");
    assert_eq!(report["protocol_revision"], "2026-07-28");
    assert_eq!(report["outcome"], "passed");
    assert_eq!(report["exit_code"], 0);
    assert!(report["primary_diagnosis"].is_null());
    assert_eq!(report["independent_findings"], serde_json::json!([]));
    assert_eq!(report["summary"]["checks"], 6);
    assert_eq!(report["summary"]["performed"], 5);
    assert_eq!(report["summary"]["skipped"], 1);
    assert_eq!(report["limits"]["profile"], "default");
    assert_eq!(report["limits"]["total_ms"], 120_000);

    let runtime = report["checks"]
        .as_array()
        .expect("checks should be an array")
        .iter()
        .find(|check| check["id"] == "runtime.tools")
        .expect("runtime.tools should remain explicit");
    assert_eq!(runtime["state"], "skipped");
    assert_eq!(runtime["skip_reason"], "not_authorized");
    assert!(runtime.get("blocked_by").is_none());
}

#[test]
fn slow_start_profile_is_reported_without_expanding_capacity_or_activity() {
    let environment = TestEnvironment::new();
    let json_path = environment.artifact_path("slow-start.json");
    let junit_path = environment.artifact_path("slow-start.xml");
    let output = environment
        .command()
        .arg("inspect")
        .arg("--limit-profile")
        .arg("slow-start")
        .arg("--format")
        .arg("json")
        .arg("--json-report")
        .arg(&json_path)
        .arg("--junit-report")
        .arg(&junit_path)
        .arg("--")
        .arg(fixture())
        .arg("catalog-valid")
        .output()
        .expect("the bounded slow-start inspection should run");
    let (_, stderr) = text(&output);
    let report = json_report(&output);

    assert!(output.status.success(), "{report:#}\n{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(report["limits"]["profile"], "slow-start");
    assert_eq!(report["limits"]["startup_ms"], 30_000);
    assert_eq!(report["limits"]["discovery_ms"], 30_000);
    assert_eq!(report["limits"]["request_ms"], 60_000);
    assert_eq!(report["limits"]["response_ms"], 60_000);
    assert_eq!(report["limits"]["shutdown_grace_ms"], 2_000);
    assert_eq!(report["limits"]["total_ms"], 240_000);
    assert_eq!(report["limits"]["message_bytes"], 1_048_576);
    assert_eq!(report["limits"]["active_cases"], 100);
    assert_eq!(report["limits"]["redirects"], 0);
    assert_eq!(report["limits"]["retries"], 0);
    assert_eq!(report["limits"]["concurrency"], 1);

    let artifact = parse_and_validate_report(
        &std::fs::read(&json_path).expect("the slow-start JSON artifact should exist"),
    );
    assert_eq!(artifact["limits"], report["limits"]);
    let (junit, _) = parse_and_validate_junit(
        &std::fs::read(&junit_path).expect("the slow-start JUnit artifact should exist"),
    );
    assert!(junit.contains("limits.profile=slow-start"));

    let human = environment
        .command()
        .arg("inspect")
        .arg("--limit-profile")
        .arg("slow-start")
        .arg("--")
        .arg(fixture())
        .arg("catalog-valid")
        .output()
        .expect("the slow-start human report should run");
    let (human_stdout, human_stderr) = text(&human);
    assert!(human.status.success(), "{human_stdout}\n{human_stderr}");
    assert!(human_stderr.is_empty());
    assert!(human_stdout.contains("LIMITS · profile=slow-start"));
    assert!(human_stdout.contains("startup_ms=30000"));
    assert!(human_stdout.contains("total_ms=240000"));
}

#[test]
fn ordinary_report_alone_identifies_the_unsupported_revision_correction() {
    let human_output = run_mode("protocol-unsupported");
    let json_output = run_json_mode("protocol-unsupported");
    let (human, human_stderr) = text(&human_output);
    let (json, json_stderr) = text(&json_output);

    assert_eq!(human_output.status.code(), Some(1), "{human}");
    assert_eq!(json_output.status.code(), Some(1), "{json}");
    assert!(human_stderr.is_empty());
    assert!(json_stderr.is_empty());
    assert_eq!(human, REPORT_ONLY_HUMAN);
    assert_eq!(json, REPORT_ONLY_JSON);

    // From this point onward the assertions consume only the checked-in reports;
    // they do not inspect the fixture response, target stderr, or implementation.
    let report = validate_report_value(
        serde_json::from_str(REPORT_ONLY_JSON).expect("report-only JSON should parse"),
    );
    let diagnosis = &report["primary_diagnosis"];
    let diagnosis_check = diagnosis["check_id"]
        .as_str()
        .expect("the primary check should be named");
    let diagnosis_finding = diagnosis["findings"]
        .as_array()
        .expect("primary findings should be an array")
        .first()
        .expect("the primary diagnosis should reference a finding");
    let finding = find_json_check(&report, diagnosis_check)["findings"]
        .as_array()
        .expect("the diagnosed check should contain findings")
        .iter()
        .find(|candidate| {
            candidate["code"] == diagnosis_finding["code"]
                && candidate["location"] == diagnosis_finding["location"]
        })
        .expect("the primary reference should resolve inside the ordinary report");

    assert_eq!(diagnosis_check, "protocol.revision");
    assert_eq!(finding["code"], "MCP-PROTOCOL-002");
    assert_eq!(finding["location"], "server.supportedVersions");
    assert_eq!(
        finding["message"],
        "The server does not support the required protocol revision."
    );
    assert_eq!(
        finding["impact"],
        "Applying rules for a different revision could produce a false diagnosis."
    );
    assert_eq!(
        finding["expectation"],
        "The server must support MCP protocol revision 2026-07-28 for this diagnosis."
    );
    assert_eq!(
        finding["remediation"],
        "Add MCP 2026-07-28 support, then rerun the same diagnosis without falling back."
    );
    assert_eq!(
        finding["reference"],
        "selected MCP revision lifecycle contract"
    );
    for field in [
        "location",
        "message",
        "impact",
        "expectation",
        "remediation",
        "reference",
    ] {
        assert!(
            REPORT_ONLY_HUMAN.contains(
                finding[field]
                    .as_str()
                    .expect("report-only action field should be text")
            ),
            "human report should carry the same {field}"
        );
    }
    assert!(REPORT_ONLY_HUMAN.contains("blocked by protocol.revision"));
    assert!(!REPORT_ONLY_HUMAN.contains("synthetic-private-revision"));
    assert!(!REPORT_ONLY_JSON.contains("synthetic-private-revision"));
}

#[test]
fn junit_inspection_preserves_the_stdio_diagnosis_skips_and_exit_status() {
    let environment = TestEnvironment::new();
    let output = junit_inspect_command(&environment, "protocol-unsupported")
        .output()
        .expect("mcp-doctor should emit JUnit for the STDIO journey");
    let (_, stderr) = text(&output);
    let (document, summary) = parse_and_validate_junit(&output.stdout);

    assert_eq!(output.status.code(), Some(1), "{document}\n{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(summary.failures, 1);
    assert!(summary.skipped > 0);
    assert!(document.contains("type=\"MCP-PROTOCOL-002\""));
    assert!(document.contains("blocked_by.check_id=protocol.revision"));
    assert!(document.contains("report_outcome=failed\nexit_code=1"));
    assert!(!document.contains(REDACTION_SENTINEL));
}

#[test]
fn human_and_json_choose_the_same_earliest_layer_and_causal_skips() {
    for (mode, expected_layer, expected_code, blocked_checks) in [
        (
            "malformed",
            "transport.stdio",
            "MCP-TRANSPORT-003",
            [
                "protocol.envelope",
                "protocol.revision",
                "discovery.catalogs",
                "schema.contracts",
            ]
            .as_slice(),
        ),
        (
            "protocol-unsupported",
            "protocol.revision",
            "MCP-PROTOCOL-002",
            ["discovery.catalogs", "schema.contracts"].as_slice(),
        ),
        (
            "layered-protocol-failure",
            "protocol.envelope",
            "MCP-CATALOG-001",
            ["discovery.catalogs", "schema.contracts"].as_slice(),
        ),
        (
            "catalog-blocks-schema",
            "discovery.catalogs",
            "MCP-CATALOG-001",
            ["schema.contracts"].as_slice(),
        ),
        (
            "schema-invalid",
            "schema.contracts",
            "MCP-SCHEMA-001",
            [].as_slice(),
        ),
    ] {
        let human_output = run_mode(mode);
        let json_output = run_json_mode(mode);
        let (human, human_stderr) = text(&human_output);
        let (_, json_stderr) = text(&json_output);
        let report = json_report(&json_output);

        assert_eq!(human_output.status.code(), Some(1), "{mode}: {human}");
        assert_eq!(json_output.status.code(), Some(1), "{mode}: {report:#}");
        assert!(human_stderr.is_empty(), "{mode}: {human_stderr}");
        assert!(json_stderr.is_empty(), "{mode}: {json_stderr}");
        assert_eq!(
            report["primary_diagnosis"]["check_id"], expected_layer,
            "{mode}: {report:#}"
        );
        assert!(
            report["primary_diagnosis"]["findings"]
                .as_array()
                .expect("primary findings should be an array")
                .iter()
                .any(|finding| finding["code"] == expected_code),
            "{mode}: {report:#}"
        );
        assert!(
            human.contains(&format!("PRIMARY DIAGNOSIS · {expected_layer}")),
            "{mode}: {human}"
        );
        assert!(human.contains(expected_code), "{mode}: {human}");

        for check_id in blocked_checks {
            let check = find_json_check(&report, check_id);
            assert_eq!(check["state"], "skipped", "{mode}: {check:#}");
            assert_eq!(
                check["blocked_by"]["check_id"], expected_layer,
                "{mode}: {check:#}"
            );
            assert!(
                human.contains(&format!("blocked by {expected_layer}")),
                "{mode}: {human}"
            );
        }

        let runtime = find_json_check(&report, "runtime.tools");
        assert_eq!(runtime["skip_reason"], "not_authorized");
        assert!(runtime.get("blocked_by").is_none());
        assert_human_json_summary_and_limits_match(human, &report);
        assert_report_findings_are_actionable(&report, human);
        for sentinel in [
            REDACTION_SENTINEL,
            CATALOG_SENTINEL,
            "synthetic-private-revision-never-report-7f2c",
            "synthetic-private-result-never-report-7f2c",
            "synthetic-private-tools-never-report-7f2c",
        ] {
            assert!(!human.contains(sentinel), "{mode}: {human}");
            assert!(
                !String::from_utf8_lossy(&json_output.stdout).contains(sentinel),
                "{mode}: {report:#}"
            );
        }
    }
}

#[test]
fn built_binary_keeps_independent_cleanup_failure_out_of_the_primary_cause() {
    let human_environment = TestEnvironment::new();
    let human_output = inspect_command(&human_environment, "malformed")
        .env("MCP_DOCTOR_INTERNAL_TEST_CLEANUP_FAILURE", "1")
        .output()
        .expect("mcp-doctor should render the synthetic independent failure");
    let json_environment = TestEnvironment::new();
    let json_output = json_inspect_command(&json_environment, "malformed")
        .env("MCP_DOCTOR_INTERNAL_TEST_CLEANUP_FAILURE", "1")
        .output()
        .expect("mcp-doctor should render the synthetic independent failure as JSON");
    let (human, human_stderr) = text(&human_output);
    let (_, json_stderr) = text(&json_output);
    let report = json_report(&json_output);

    assert_eq!(human_output.status.code(), Some(1), "{human}");
    assert_eq!(json_output.status.code(), Some(1), "{report:#}");
    assert!(human_stderr.is_empty());
    assert!(json_stderr.is_empty());
    assert_eq!(report["primary_diagnosis"]["check_id"], "transport.stdio");
    assert_eq!(
        report["primary_diagnosis"]["findings"],
        serde_json::json!([{
            "code": "MCP-TRANSPORT-003",
            "location": "process.stdout.message[0]"
        }])
    );
    assert_eq!(
        report["independent_findings"],
        serde_json::json!([{
            "check_id": "transport.stdio",
            "code": "MCP-SAFETY-001",
            "location": "process"
        }])
    );
    assert!(human.contains("PRIMARY DIAGNOSIS · transport.stdio"));
    assert!(human.contains("INDEPENDENT SAFETY FINDINGS · 1"));
    assert!(human.contains("MCP-SAFETY-001 · transport.stdio · process"));
    assert_human_json_summary_and_limits_match(human, &report);
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

fn find_json_check<'a>(report: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    report["checks"]
        .as_array()
        .expect("checks should be an array")
        .iter()
        .find(|check| check["id"] == id)
        .unwrap_or_else(|| panic!("report should contain check {id}"))
}

fn assert_human_json_summary_and_limits_match(human: &str, report: &serde_json::Value) {
    let summary = &report["summary"];
    let summary_line = format!(
        "{} failed · {} warned · {} passed · {} skipped · outcome {} · exit {}",
        summary["failed"]
            .as_u64()
            .expect("failed should be a count"),
        summary["warned"]
            .as_u64()
            .expect("warned should be a count"),
        summary["passed"]
            .as_u64()
            .expect("passed should be a count"),
        summary["skipped"]
            .as_u64()
            .expect("skipped should be a count"),
        report["outcome"]
            .as_str()
            .expect("outcome should be a string"),
        report["exit_code"]
            .as_u64()
            .expect("exit_code should be a number")
    );
    assert!(human.contains(&summary_line), "{human}");

    for (name, value) in report["limits"]
        .as_object()
        .expect("limits should be an object")
    {
        if name == "profile" {
            let profile = value
                .as_str()
                .expect("the limit profile should be a fixed string");
            assert!(
                human.contains(&format!("LIMITS · profile={profile}")),
                "human report is missing JSON limit profile {profile}: {human}"
            );
            continue;
        }
        let value = value.as_u64().expect("every limit should be an integer");
        assert!(
            human.contains(&format!("{name}={value}")),
            "human report is missing JSON limit {name}={value}: {human}"
        );
    }
}

fn assert_report_findings_are_actionable(report: &serde_json::Value, human: &str) {
    for check in report["checks"]
        .as_array()
        .expect("checks should be an array")
    {
        for finding in check["findings"]
            .as_array()
            .expect("findings should be an array")
        {
            for field in [
                "code",
                "severity",
                "protocol_revision",
                "location",
                "message",
                "impact",
                "expectation",
                "remediation",
                "reference",
            ] {
                let value = finding[field]
                    .as_str()
                    .unwrap_or_else(|| panic!("{field} should be a string in {finding:#}"));
                assert!(!value.is_empty(), "{field} should not be empty");
                assert!(
                    human.contains(value),
                    "human report should contain JSON {field} value {value:?}: {human}"
                );
            }
        }
    }
}

fn contains_path(output: &str, path: &Path) -> bool {
    path.to_str().is_some_and(|path| output.contains(path))
        || path
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| output.contains(name))
}
