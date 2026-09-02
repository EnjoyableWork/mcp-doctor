#![cfg(feature = "internal-test-fixtures")]

mod support;

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde_json::{Value, json};
use support::{
    TestEnvironment, assert_descendant_was_ready_and_terminated, parse_and_validate_report,
    parse_and_validate_status_jsonl,
};

const TOOL: &str = "synthetic.reviewed";
const GENERATED_TOOL: &str = "synthetic.generated";
const SECRET_VALUE: &str = "synthetic-secret-payload-7f2c";
const ARGUMENT_SECRET_NAME: &str = "SYNTHETIC_TOOL_SECRET_7F2C";
const TARGET_SECRET_NAME: &str = "ACTIVE_TARGET_SECRET";
const STATUS_GOLDEN: &[u8] = include_bytes!("fixtures/status/inspect-success.jsonl");
const STATUS_SCHEMA: &str = include_str!("../schemas/mcp-doctor.status.v1.schema.json");

fn fixture() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_mcp-doctor-stdio-fixture"))
}

fn exact_inspect_command(
    environment: &TestEnvironment,
    status: Option<&str>,
    executable: &Path,
    mode: &str,
) -> Command {
    let mut command = environment.command();
    command
        .arg("inspect")
        .arg("--protocol-version")
        .arg("2026-07-28")
        .arg("--format")
        .arg("json");
    if let Some(status) = status {
        command.arg("--status").arg(status);
    }
    command.arg("--").arg(executable).arg(mode);
    command
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

fn scenario(cases: Vec<Value>) -> Value {
    json!({
        "schema_version": "mcp-doctor.scenario/v1alpha1",
        "tool": TOOL,
        "safety": {"effects": "read_only"},
        "cases": cases
    })
}

fn write_scenario(environment: &TestEnvironment, name: &str, document: &Value) -> PathBuf {
    let path = environment.artifact_path(name);
    fs::write(
        &path,
        serde_json::to_vec_pretty(document).expect("the synthetic scenario should serialize"),
    )
    .expect("the synthetic scenario should be writable");
    path
}

fn status_records(output: &Output) -> Vec<Value> {
    parse_and_validate_status_jsonl(&output.stderr)
}

fn assert_common_status(records: &[Value], command: &str, transport: &str, profile: &str) {
    assert!(!records.is_empty());
    for record in records {
        assert_eq!(record["schema_version"], "mcp-doctor.status/v1");
        assert_eq!(record["command"], command);
        assert_eq!(record["transport"], transport);
        assert_eq!(record["limit_profile"], profile);
    }
    assert_eq!(records[0]["event"], "invocation_accepted");
}

fn assert_completed(records: &[Value], exit_code: u8, exit_meaning: &str) {
    let terminal = records.last().expect("status should have a terminal event");
    assert_eq!(terminal["event"], "completed");
    assert_eq!(terminal["exit_code"], exit_code);
    assert_eq!(terminal["exit_meaning"], exit_meaning);
    assert_eq!(
        records
            .iter()
            .filter(|record| record["event"] == "completed")
            .count(),
        1
    );
}

fn phase_index(records: &[Value], phase: &str) -> usize {
    records
        .iter()
        .position(|record| record["event"] == "phase_started" && record["phase"] == phase)
        .unwrap_or_else(|| panic!("status omitted phase {phase}: {records:#?}"))
}

fn assert_cleanup_publication_completion_order(records: &[Value]) {
    let cleanup = phase_index(records, "cleanup");
    let publication = phase_index(records, "report_publication");
    let completion = records
        .iter()
        .position(|record| record["event"] == "completed")
        .expect("status should complete");
    assert!(
        cleanup < publication && publication < completion,
        "{records:#?}"
    );
}

fn case_ordinals(records: &[Value]) -> Vec<(u64, u64)> {
    records
        .iter()
        .filter(|record| record["event"] == "case_started")
        .map(|record| {
            (
                record["ordinal"]
                    .as_u64()
                    .expect("the case ordinal should be numeric"),
                record["total"]
                    .as_u64()
                    .expect("the case total should be numeric"),
            )
        })
        .collect()
}

fn configured_report_command(
    environment: &TestEnvironment,
    prefix: &str,
    status: Option<&str>,
) -> (Command, Vec<PathBuf>) {
    let destinations = ["json", "xml", "md", "badge.json", "snapshot.json"]
        .map(|suffix| environment.artifact_path(&format!("{prefix}.{suffix}")))
        .to_vec();
    let mut command = environment.command();
    command
        .arg("inspect")
        .arg("--protocol-version")
        .arg("2026-07-28")
        .arg("--format")
        .arg("json")
        .arg("--json-report")
        .arg(&destinations[0])
        .arg("--junit-report")
        .arg(&destinations[1])
        .arg("--markdown-report")
        .arg(&destinations[2])
        .arg("--badge-report")
        .arg(&destinations[3])
        .arg("--snapshot")
        .arg(&destinations[4])
        .arg("--allow-sensitive-snapshot")
        .arg(&destinations[4]);
    if let Some(status) = status {
        command.arg("--status").arg(status);
    }
    command.arg("--").arg(fixture()).arg("catalog-valid");
    (command, destinations)
}

#[test]
fn status_is_opt_in_and_preserves_stdout_artifacts_exit_and_target_activity() {
    let environment = TestEnvironment::new();
    let (mut baseline_command, baseline_paths) =
        configured_report_command(&environment, "baseline", None);
    let baseline = baseline_command
        .output()
        .expect("the baseline inspection should run");
    let (mut status_command, status_paths) =
        configured_report_command(&environment, "with-status", Some("jsonl"));
    let with_status = status_command
        .output()
        .expect("the status-enabled inspection should run");

    assert!(baseline.status.success());
    assert!(with_status.status.success());
    assert!(baseline.stderr.is_empty());
    assert_eq!(baseline.status.code(), with_status.status.code());
    assert_eq!(baseline.stdout, with_status.stdout);
    for (baseline_path, status_path) in baseline_paths.iter().zip(&status_paths) {
        assert_eq!(
            fs::read(baseline_path).expect("the baseline artifact should exist"),
            fs::read(status_path).expect("the status-enabled artifact should exist")
        );
    }

    let report = parse_and_validate_report(&with_status.stdout);
    assert_eq!(report["protocol_selection"]["process_launches"], 1);
    assert_eq!(report["protocol_selection"]["lifecycle_requests"], 1);
    let records = status_records(&with_status);
    assert_common_status(&records, "inspect", "stdio", "default");
    assert_cleanup_publication_completion_order(&records);
    assert_completed(&records, 0, "success");
    assert_eq!(with_status.stderr, STATUS_GOLDEN);

    let plain = exact_inspect_command(&environment, Some("plain"), fixture(), "catalog-valid")
        .output()
        .expect("plain status should run");
    assert!(plain.status.success());
    let plain_status = std::str::from_utf8(&plain.stderr).expect("plain status should be UTF-8");
    for expected in [
        "invocation_accepted",
        "phase=target_preparation",
        "phase=target_startup · ceiling_kind=startup · ceiling_ms=10000",
        "phase=discovery · ceiling_kind=discovery · ceiling_ms=10000",
        "phase=cleanup · ceiling_kind=cleanup_grace · ceiling_ms=2000",
        "phase=report_publication",
        "completed · command=inspect · transport=stdio · limit_profile=default · exit_code=0 · exit_meaning=success",
    ] {
        assert!(plain_status.contains(expected), "{plain_status}");
    }
    assert!(!plain_status.contains('\r'));
    assert!(!plain_status.contains("\u{1b}["));
    assert_eq!(plain.stdout, baseline.stdout);
}

#[test]
fn every_target_command_emits_fixed_context_and_exact_active_case_ordinals() {
    let environment = TestEnvironment::new();
    let scenario_path = write_scenario(
        &environment,
        "one-case.json",
        &scenario(vec![reviewed_case(0, "success")]),
    );
    let check = environment
        .command()
        .arg("check")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json")
        .arg("--status")
        .arg("jsonl")
        .arg("--")
        .arg(fixture())
        .arg("active-one-success")
        .output()
        .expect("status-enabled check should run");
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    let check_records = status_records(&check);
    assert_common_status(&check_records, "check", "stdio", "default");
    assert_eq!(case_ordinals(&check_records), vec![(1, 1)]);
    assert_cleanup_publication_completion_order(&check_records);

    let plain_check = environment
        .command()
        .arg("check")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json")
        .arg("--status")
        .arg("plain")
        .arg("--")
        .arg(fixture())
        .arg("active-one-success")
        .output()
        .expect("plain active status should run");
    assert!(plain_check.status.success());
    let plain_check_status = String::from_utf8(plain_check.stderr).unwrap();
    assert!(plain_check_status.contains("phase=input_preparation"));
    assert!(plain_check_status.contains(
        "case_started · command=check · transport=stdio · limit_profile=default · ordinal=1 · total=1 · request_ceiling_ms=30000 · response_ceiling_ms=30000"
    ));

    let break_marker = environment.artifact_path("break-observations.json");
    let broken = environment
        .command()
        .arg("break")
        .arg("--tool")
        .arg(GENERATED_TOOL)
        .arg("--allow-tool")
        .arg(GENERATED_TOOL)
        .arg("--effects")
        .arg("read_only")
        .arg("--cases")
        .arg("2")
        .arg("--seed")
        .arg("4242")
        .arg("--format")
        .arg("json")
        .arg("--status")
        .arg("jsonl")
        .arg("--")
        .arg(fixture())
        .arg("break-success")
        .arg(&break_marker)
        .arg("2")
        .output()
        .expect("status-enabled break should run");
    assert!(
        broken.status.success(),
        "{}",
        String::from_utf8_lossy(&broken.stderr)
    );
    let break_records = status_records(&broken);
    assert_common_status(&break_records, "break", "stdio", "default");
    assert_eq!(case_ordinals(&break_records), vec![(1, 2), (2, 2)]);
    assert_cleanup_publication_completion_order(&break_records);

    let reject_marker = environment.artifact_path("reject-call-count");
    let rejected = environment
        .command()
        .arg("reject")
        .arg("--tool")
        .arg(TOOL)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--effects")
        .arg("read_only")
        .arg("--seed")
        .arg("4242")
        .arg("--format")
        .arg("json")
        .arg("--status")
        .arg("jsonl")
        .arg("--")
        .arg(fixture())
        .arg("reject-success")
        .arg(&reject_marker)
        .output()
        .expect("status-enabled reject should run");
    assert!(
        rejected.status.success(),
        "{}",
        String::from_utf8_lossy(&rejected.stderr)
    );
    let reject_records = status_records(&rejected);
    assert_common_status(&reject_records, "reject", "stdio", "default");
    assert_eq!(
        case_ordinals(&reject_records),
        (1..=7).map(|ordinal| (ordinal, 7)).collect::<Vec<_>>()
    );
    assert_cleanup_publication_completion_order(&reject_records);
    assert_eq!(fs::read(&reject_marker).unwrap(), b"7");

    for (records, command) in [
        (&check_records, "check"),
        (&break_records, "break"),
        (&reject_records, "reject"),
    ] {
        assert_eq!(phase_index(records, "input_preparation"), 1, "{command}");
        assert!(
            phase_index(records, "input_preparation") < phase_index(records, "target_preparation")
        );
        assert_completed(records, 0, "success");
    }
}

#[test]
fn supported_legacy_stdio_revisions_use_the_same_status_contract() {
    for revision in ["2025-11-25", "2025-06-18"] {
        let environment = TestEnvironment::new();
        let output = environment
            .command()
            .arg("inspect")
            .arg("--protocol-version")
            .arg(revision)
            .arg("--format")
            .arg("json")
            .arg("--status")
            .arg("jsonl")
            .arg("--")
            .arg(fixture())
            .arg("legacy-success")
            .output()
            .expect("legacy status inspection should run");
        assert!(
            output.status.success(),
            "{revision}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let records = status_records(&output);
        assert_common_status(&records, "inspect", "stdio", "default");
        assert_cleanup_publication_completion_order(&records);
        assert_completed(&records, 0, "success");
    }

    let environment = TestEnvironment::new();
    let slow = environment
        .command()
        .arg("inspect")
        .arg("--protocol-version")
        .arg("2026-07-28")
        .arg("--limit-profile")
        .arg("slow-start")
        .arg("--format")
        .arg("json")
        .arg("--status")
        .arg("jsonl")
        .arg("--")
        .arg(fixture())
        .arg("catalog-valid")
        .output()
        .expect("slow-start status should run");
    assert!(slow.status.success());
    let records = status_records(&slow);
    assert_common_status(&records, "inspect", "stdio", "slow-start");
    let startup = &records[phase_index(&records, "target_startup")];
    assert_eq!(startup["ceiling_kind"], "startup");
    assert_eq!(startup["ceiling_ms"], 30_000);
    let discovery = &records[phase_index(&records, "discovery")];
    assert_eq!(discovery["ceiling_kind"], "discovery");
    assert_eq!(discovery["ceiling_ms"], 30_000);
}

#[test]
fn terminal_status_covers_failure_incomplete_prestart_timeout_early_exit_and_cleanup_failure() {
    for (mode, expected_code) in [("catalog-invalid", 1_u8), ("early-exit", 1_u8)] {
        let environment = TestEnvironment::new();
        let output = exact_inspect_command(&environment, Some("jsonl"), fixture(), mode)
            .output()
            .expect("the status outcome fixture should run");
        assert_eq!(output.status.code(), Some(i32::from(expected_code)));
        let records = status_records(&output);
        assert_cleanup_publication_completion_order(&records);
        assert_completed(&records, expected_code, "unsuccessful_result");
    }

    let timeout_environment = TestEnvironment::new();
    let timeout = exact_inspect_command(&timeout_environment, Some("jsonl"), fixture(), "timeout")
        .output()
        .expect("the discovery timeout should remain bounded");
    assert_eq!(timeout.status.code(), Some(1));
    let timeout_records = status_records(&timeout);
    assert_cleanup_publication_completion_order(&timeout_records);
    assert_completed(&timeout_records, 1, "unsuccessful_result");

    let cleanup_environment = TestEnvironment::new();
    let cleanup = exact_inspect_command(
        &cleanup_environment,
        Some("jsonl"),
        fixture(),
        "catalog-valid",
    )
    .env("MCP_DOCTOR_INTERNAL_TEST_CLEANUP_FAILURE", "1")
    .output()
    .expect("the cleanup-failure status fixture should run");
    assert_eq!(cleanup.status.code(), Some(1));
    let cleanup_records = status_records(&cleanup);
    assert_cleanup_publication_completion_order(&cleanup_records);
    assert_completed(&cleanup_records, 1, "unsuccessful_result");

    let publication_environment = TestEnvironment::new();
    let publication_path = publication_environment.artifact_path("publication-failure.json");
    let publication = publication_environment
        .command()
        .arg("inspect")
        .arg("--protocol-version")
        .arg("2026-07-28")
        .arg("--format")
        .arg("json")
        .arg("--status")
        .arg("jsonl")
        .arg("--json-report")
        .arg(&publication_path)
        .arg("--")
        .arg(fixture())
        .arg("catalog-valid")
        .env("MCP_DOCTOR_INTERNAL_TEST_REPORT_WRITE_FAILURE", "1")
        .output()
        .expect("the report-publication failure status fixture should run");
    assert_eq!(publication.status.code(), Some(4));
    assert!(!publication_path.exists());
    let publication_records = status_records(&publication);
    assert_cleanup_publication_completion_order(&publication_records);
    let publication_phase = phase_index(&publication_records, "report_publication");
    let publication_error = publication_records
        .iter()
        .position(|record| {
            record["event"] == "error" && record["error_kind"] == "internal_or_output_failure"
        })
        .expect("publication failure should have a fixed status error");
    assert!(publication_phase < publication_error);
    assert_completed(&publication_records, 4, "internal_or_output_failure");

    let incomplete_environment = TestEnvironment::new();
    let scenario_path = write_scenario(
        &incomplete_environment,
        "input-required.json",
        &scenario(vec![
            reviewed_case(0, "success"),
            reviewed_case(1, "success"),
        ]),
    );
    let incomplete = incomplete_environment
        .command()
        .arg("check")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json")
        .arg("--status")
        .arg("jsonl")
        .arg("--")
        .arg(fixture())
        .arg("active-input-required")
        .output()
        .expect("the incomplete status fixture should run");
    assert_eq!(incomplete.status.code(), Some(3));
    let incomplete_records = status_records(&incomplete);
    assert_cleanup_publication_completion_order(&incomplete_records);
    assert_completed(&incomplete_records, 3, "incomplete_evidence");

    let invalid_environment = TestEnvironment::new();
    let invalid_path = invalid_environment.artifact_path("invalid-scenario-never-status-7f2c.json");
    fs::write(&invalid_path, b"{").expect("the invalid scenario should be writable");
    let target_marker = invalid_environment.artifact_path("target-started-never-status-7f2c");
    let invalid = invalid_environment
        .command()
        .arg("check")
        .arg("--scenario")
        .arg(&invalid_path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--format")
        .arg("json")
        .arg("--status")
        .arg("jsonl")
        .arg("--")
        .arg(fixture())
        .arg("active-started-marker")
        .arg(&target_marker)
        .output()
        .expect("the invalid prestart status fixture should run");
    assert_eq!(invalid.status.code(), Some(2));
    assert!(!target_marker.exists());
    let invalid_records = status_records(&invalid);
    assert!(
        invalid_records
            .iter()
            .all(|record| record["phase"] != "target_preparation")
    );
    assert!(
        phase_index(&invalid_records, "input_preparation")
            < phase_index(&invalid_records, "report_publication")
    );
    assert_completed(&invalid_records, 2, "invalid_invocation_or_input");
}

fn copied_fixture_with_sentinel(environment: &TestEnvironment) -> PathBuf {
    let mut name = String::from("synthetic-secret-executable-never-status-7f2c");
    if let Some(extension) = fixture().extension().and_then(|value| value.to_str()) {
        name.push('.');
        name.push_str(extension);
    }
    let destination = environment.artifact_path(&name);
    fs::copy(fixture(), &destination).expect("the executable sentinel fixture should be copied");
    destination
}

#[test]
fn status_never_contains_target_progress_stderr_or_invocation_values() {
    for format in ["plain", "jsonl"] {
        let environment = TestEnvironment::new();
        let executable = copied_fixture_with_sentinel(&environment);
        let report_path =
            environment.artifact_path(&format!("synthetic-secret-artifact-{format}-7f2c.json"));
        let output = environment
            .command()
            .arg("inspect")
            .arg("--protocol-version")
            .arg("2026-07-28")
            .arg("--format")
            .arg("json")
            .arg("--status")
            .arg(format)
            .arg("--json-report")
            .arg(&report_path)
            .arg("--")
            .arg(&executable)
            .arg("status-redaction")
            .arg("synthetic-secret-target-argument-never-status-7f2c")
            .output()
            .expect("the status-redaction fixture should run");
        assert!(
            output.status.success(),
            "{format}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let status = std::str::from_utf8(&output.stderr).expect("status should be UTF-8");
        for forbidden in [
            "synthetic-secret-executable-never-status-7f2c",
            "synthetic-secret-target-argument-never-status-7f2c",
            "synthetic-secret-artifact",
            "synthetic-secret-target-stderr-never-status-7f2c",
            "synthetic-secret-progress-token-never-status-7f2c",
            "synthetic-secret-progress-message-never-status-7f2c",
        ] {
            assert!(
                !status.contains(forbidden),
                "{format} disclosed {forbidden}"
            );
        }
        if format == "jsonl" {
            status_records(&output);
        }
    }

    for format in ["plain", "jsonl"] {
        let environment = TestEnvironment::new();
        let mut first = reviewed_case(0, "success");
        first["arguments"]["secret"] = Value::Null;
        first["secret_refs"] = json!({"/secret": ARGUMENT_SECRET_NAME});
        let mut second = reviewed_case(1, "tool_error");
        second["arguments"]["secret"] = Value::Null;
        second["secret_refs"] = json!({"/secret": ARGUMENT_SECRET_NAME});
        let mut document = scenario(vec![first, second]);
        document["target_env"] = json!([TARGET_SECRET_NAME]);
        let path = write_scenario(
            &environment,
            "synthetic-secret-scenario-never-status-7f2c.json",
            &document,
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
            .arg("--status")
            .arg(format)
            .arg("--")
            .arg(fixture())
            .arg("active-success")
            .arg("synthetic-secret-target-argument-never-status-7f2c")
            .env(ARGUMENT_SECRET_NAME, SECRET_VALUE)
            .env(TARGET_SECRET_NAME, SECRET_VALUE)
            .output()
            .expect("the active redaction status fixture should run");
        assert!(output.status.success());
        let status = std::str::from_utf8(&output.stderr).expect("status should be UTF-8");
        for forbidden in [
            TOOL,
            SECRET_VALUE,
            ARGUMENT_SECRET_NAME,
            TARGET_SECRET_NAME,
            "author-only-case",
            "synthetic-secret-scenario",
            "synthetic-secret-target-argument",
        ] {
            assert!(
                !status.contains(forbidden),
                "{format} disclosed {forbidden}"
            );
        }
        if format == "jsonl" {
            let records = status_records(&output);
            assert_eq!(case_ordinals(&records), vec![(1, 2), (2, 2)]);
        }
    }

    for format in ["plain", "jsonl"] {
        let environment = TestEnvironment::new();
        let invalid_endpoint = "https://synthetic-secret-endpoint-never-status-7f2c.invalid/%";
        let endpoint_output = environment
            .command()
            .arg("inspect")
            .arg("--status")
            .arg(format)
            .arg("--format")
            .arg("json")
            .arg(invalid_endpoint)
            .arg("--bearer-token-env")
            .arg("SYNTHETIC_SECRET_BEARER_SOURCE_NEVER_STATUS_7F2C")
            .arg("--header-env")
            .arg("X-Synthetic-Secret=SYNTHETIC_SECRET_HEADER_SOURCE_NEVER_STATUS_7F2C")
            .env(
                "SYNTHETIC_SECRET_BEARER_SOURCE_NEVER_STATUS_7F2C",
                "synthetic-secret-bearer-value-never-status-7f2c",
            )
            .env(
                "SYNTHETIC_SECRET_HEADER_SOURCE_NEVER_STATUS_7F2C",
                "synthetic-secret-header-value-never-status-7f2c",
            )
            .output()
            .expect("the invalid remote status fixture should run");
        let endpoint_status = std::str::from_utf8(&endpoint_output.stderr).unwrap();
        assert!(
            !endpoint_status
                .to_ascii_lowercase()
                .contains("synthetic-secret"),
            "{format} disclosed a remote invocation value"
        );
        if format == "jsonl" {
            assert_common_status(
                &status_records(&endpoint_output),
                "inspect",
                "streamable_http",
                "default",
            );
        }
    }
}

#[test]
fn semantic_errors_after_status_selection_keep_jsonl_stderr_exclusive() {
    let environment = TestEnvironment::new();
    let duplicate = environment.artifact_path("synthetic-secret-duplicate-path-never-status-7f2c");
    let target_marker = environment.artifact_path("target-must-not-start");
    let output = environment
        .command()
        .arg("inspect")
        .arg("--status")
        .arg("jsonl")
        .arg("--json-report")
        .arg(&duplicate)
        .arg("--junit-report")
        .arg(&duplicate)
        .arg("--")
        .arg(fixture())
        .arg("snapshot-started-marker")
        .arg(&target_marker)
        .output()
        .expect("the semantic status error should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!target_marker.exists());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("synthetic-secret"));
    let records = status_records(&output);
    assert_eq!(
        records
            .iter()
            .map(|record| record["event"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["invocation_accepted", "error", "completed"]
    );
    assert_eq!(records[1]["error_kind"], "invalid_invocation_or_input");
    assert_completed(&records, 2, "invalid_invocation_or_input");

    let parse_error = environment
        .command()
        .arg("inspect")
        .arg("--status")
        .arg("--")
        .arg(fixture())
        .arg("catalog-valid")
        .output()
        .expect("a missing status value should be rejected by CLI parsing");
    assert_eq!(parse_error.status.code(), Some(2));
    assert!(parse_error.stdout.is_empty());
    assert!(!parse_error.stderr.is_empty());
}

struct RunningCapture {
    child: Child,
    status_lines: Receiver<Vec<u8>>,
    stdout_reader: JoinHandle<Vec<u8>>,
    stderr_reader: JoinHandle<Vec<u8>>,
}

impl RunningCapture {
    fn spawn(command: &mut Command) -> Self {
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = command.spawn().expect("the captured command should start");
        let stdout = child.stdout.take().expect("captured stdout should exist");
        let stderr = child.stderr.take().expect("captured stderr should exist");
        let stdout_reader = thread::spawn(move || {
            let mut bytes = Vec::new();
            BufReader::new(stdout)
                .read_to_end(&mut bytes)
                .expect("captured stdout should be readable");
            bytes
        });
        let (sender, status_lines) = mpsc::channel();
        let stderr_reader = thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut output = Vec::new();
            loop {
                let mut line = Vec::new();
                let read = reader
                    .read_until(b'\n', &mut line)
                    .expect("captured status should be readable");
                if read == 0 {
                    break;
                }
                output.extend_from_slice(&line);
                let _ = sender.send(line);
            }
            output
        });
        Self {
            child,
            status_lines,
            stdout_reader,
            stderr_reader,
        }
    }

    fn wait_for_phase(&mut self, phase: &str) {
        loop {
            let line = match self.status_lines.recv_timeout(Duration::from_secs(10)) {
                Ok(line) => line,
                Err(error) => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    panic!("status did not flush phase {phase}: {error}");
                }
            };
            let record: Value = serde_json::from_slice(&line)
                .unwrap_or_else(|error| panic!("status line should be JSON: {error}"));
            if record["event"] == "phase_started" && record["phase"] == phase {
                return;
            }
        }
    }

    fn finish(mut self) -> Output {
        let status = self.child.wait().expect("the captured command should exit");
        let stdout = self
            .stdout_reader
            .join()
            .expect("the stdout reader should not panic");
        let stderr = self
            .stderr_reader
            .join()
            .expect("the stderr reader should not panic");
        Output {
            status,
            stdout,
            stderr,
        }
    }
}

fn accept_with_watchdog(listener: TcpListener) -> TcpStream {
    let (sender, receiver) = mpsc::sync_channel(1);
    let acceptor = thread::spawn(move || {
        let accepted = listener.accept().map(|(stream, _)| stream);
        let _ = sender.send(accepted);
    });
    let stream = receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("the barrier fixture should connect")
        .expect("the barrier fixture connection should be accepted");
    acceptor
        .join()
        .expect("the barrier acceptor should not panic");
    stream
}

#[test]
fn stdio_fixture_observes_flushed_discovery_status_before_acknowledgement() {
    let environment = TestEnvironment::new();
    let listener = TcpListener::bind("127.0.0.1:0").expect("the status barrier should bind");
    let address = listener
        .local_addr()
        .expect("the status barrier should have an address");
    let mut command =
        exact_inspect_command(&environment, Some("jsonl"), fixture(), "status-barrier");
    command.arg(address.to_string());
    let mut running = RunningCapture::spawn(&mut command);
    let mut barrier = accept_with_watchdog(listener);
    barrier
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("the status readiness read should be bounded");
    barrier
        .set_write_timeout(Some(Duration::from_secs(10)))
        .expect("the status acknowledgement write should be bounded");
    let mut readiness = [0_u8; 1];
    barrier
        .read_exact(&mut readiness)
        .expect("the fixture should publish readiness");
    assert_eq!(readiness, [1]);

    running.wait_for_phase("discovery");
    barrier
        .write_all(&[2])
        .expect("the fixture acknowledgement should be writable");
    barrier
        .flush()
        .expect("the fixture acknowledgement should flush");
    let output = running.finish();
    assert!(output.status.success());
    parse_and_validate_report(&output.stdout);
    let records = status_records(&output);
    assert_cleanup_publication_completion_order(&records);
    assert_completed(&records, 0, "success");
}

fn read_http_request(stream: &mut TcpStream) -> Value {
    let mut reader = BufReader::new(stream.try_clone().expect("the HTTP stream should clone"));
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .expect("the HTTP request line should be readable");
    assert_eq!(request_line, "POST /mcp HTTP/1.1\r\n");
    let mut content_length = None;
    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .expect("the HTTP request fields should be readable");
        if line == "\r\n" {
            break;
        }
        if let Some((name, value)) = line.trim_end().split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse::<usize>().ok();
        }
    }
    let mut body = vec![0_u8; content_length.expect("the HTTP request should have a body length")];
    reader
        .read_exact(&mut body)
        .expect("the HTTP request body should be readable");
    serde_json::from_slice(&body).expect("the HTTP request body should be JSON")
}

fn serve_barrier_http(
    listener: TcpListener,
    ready: mpsc::SyncSender<()>,
    acknowledged: Receiver<()>,
) {
    let mut stream = accept_with_watchdog(listener);
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("the HTTP fixture read should be bounded");
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .expect("the HTTP fixture write should be bounded");
    let request = read_http_request(&mut stream);
    assert_eq!(request["method"], "server/discover");
    ready
        .send(())
        .expect("the HTTP fixture should signal readiness");
    acknowledged
        .recv_timeout(Duration::from_secs(10))
        .expect("the HTTP fixture should receive acknowledgement");
    let body = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "resultType": "complete",
            "supportedVersions": ["2026-07-28"],
            "capabilities": {},
            "ttlMs": 0,
            "cacheScope": "private"
        }
    }))
    .expect("the HTTP fixture response should serialize");
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .expect("the HTTP response fields should be writable");
    stream
        .write_all(&body)
        .expect("the HTTP response body should be writable");
    stream.flush().expect("the HTTP response should flush");
}

#[test]
fn local_http_fixture_observes_flushed_discovery_status_before_acknowledgement() {
    let environment = TestEnvironment::new();
    let listener = TcpListener::bind("127.0.0.1:0").expect("the HTTP fixture should bind");
    let endpoint = format!(
        "http://{}/mcp",
        listener
            .local_addr()
            .expect("the HTTP fixture should have an address")
    );
    let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
    let (ack_sender, ack_receiver) = mpsc::sync_channel(1);
    let server = thread::spawn(move || serve_barrier_http(listener, ready_sender, ack_receiver));

    let mut command = environment.command();
    command
        .arg("inspect")
        .arg("--protocol-version")
        .arg("2026-07-28")
        .arg("--format")
        .arg("json")
        .arg("--status")
        .arg("jsonl")
        .arg(&endpoint)
        .arg("--allow-private-network")
        .arg(&endpoint)
        .arg("--allow-cleartext-http")
        .arg(&endpoint);
    let mut running = RunningCapture::spawn(&mut command);
    ready_receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("the HTTP fixture should receive the request");
    running.wait_for_phase("discovery");
    ack_sender
        .send(())
        .expect("the HTTP fixture acknowledgement should be sent");

    let output = running.finish();
    server.join().expect("the HTTP fixture should not panic");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    parse_and_validate_report(&output.stdout);
    let records = status_records(&output);
    assert_common_status(&records, "inspect", "streamable_http", "default");
    let preparation = &records[phase_index(&records, "target_preparation")];
    assert_eq!(preparation["ceiling_kind"], "startup");
    assert_eq!(preparation["ceiling_ms"], 10_000);
    assert_cleanup_publication_completion_order(&records);
    assert_completed(&records, 0, "success");
}

#[cfg(unix)]
#[test]
fn a_closed_status_sink_still_reaps_the_target_and_never_returns_success() {
    use std::os::fd::OwnedFd;
    use std::os::unix::net::UnixStream;

    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("status-sink-descendant-ready");
    let (closed_reader, status_writer) =
        UnixStream::pair().expect("the closed status sink should be created");
    drop(closed_reader);
    let output = exact_inspect_command(&environment, Some("jsonl"), fixture(), "resistant-child")
        .arg(&marker)
        .stderr(Stdio::from(OwnedFd::from(status_writer)))
        .output()
        .expect("the closed status sink journey should run");
    assert_eq!(output.status.code(), Some(4));
    assert_descendant_was_ready_and_terminated(&marker);

    let failed_environment = TestEnvironment::new();
    let (closed_reader, status_writer) =
        UnixStream::pair().expect("the second closed status sink should be created");
    drop(closed_reader);
    let failed = exact_inspect_command(
        &failed_environment,
        Some("jsonl"),
        fixture(),
        "catalog-invalid",
    )
    .stderr(Stdio::from(OwnedFd::from(status_writer)))
    .output()
    .expect("the failed closed-status journey should run");
    assert_eq!(failed.status.code(), Some(4));
    let report = parse_and_validate_report(&failed.stdout);
    assert_eq!(report["outcome"], "failed");
}

#[test]
fn status_schema_accepts_compatible_extensions_and_help_scopes_the_flag() {
    let records = parse_and_validate_status_jsonl(STATUS_GOLDEN);
    let mut extended = records[0].clone();
    extended["future_fixed_field"] = json!("future_fixed_value");
    let mut extended_line = serde_json::to_vec(&extended).expect("the extension should serialize");
    extended_line.push(b'\n');
    parse_and_validate_status_jsonl(&extended_line);

    let validator = jsonschema::draft202012::options()
        .build(&serde_json::from_str(STATUS_SCHEMA).expect("the status schema should be JSON"))
        .expect("the status schema should compile");
    let mut mismatched_terminal = records.last().unwrap().clone();
    mismatched_terminal["exit_code"] = json!(1);
    assert!(validator.validate(&mismatched_terminal).is_err());
    let mut cross_event_field = records[0].clone();
    cross_event_field["ordinal"] = json!(1);
    assert!(validator.validate(&cross_event_field).is_err());
    let mut incomplete_discovery = records[3].clone();
    incomplete_discovery
        .as_object_mut()
        .unwrap()
        .remove("ceiling_ms");
    assert!(validator.validate(&incomplete_discovery).is_err());

    for command_name in ["inspect", "check", "break", "reject"] {
        let environment = TestEnvironment::new();
        let output = environment
            .command()
            .arg(command_name)
            .arg("--help")
            .output()
            .expect("target command help should render");
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).expect("help should be UTF-8");
        assert!(help.contains("--status <FORMAT>"), "{command_name}: {help}");
        assert!(help.contains("plain"), "{command_name}: {help}");
        assert!(help.contains("jsonl"), "{command_name}: {help}");
    }
    for command_name in ["diff", "aggregate", "capabilities"] {
        let environment = TestEnvironment::new();
        let output = environment
            .command()
            .arg(command_name)
            .arg("--help")
            .output()
            .expect("offline command help should render");
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).expect("help should be UTF-8");
        assert!(!help.contains("--status"), "{command_name}: {help}");
    }
}
