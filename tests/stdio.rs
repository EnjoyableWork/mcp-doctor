#![cfg(feature = "internal-test-fixtures")]

mod support;

#[path = "fixtures/schema_gate_corpus.rs"]
mod schema_gate_corpus;

use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};

use support::{
    TestEnvironment, assert_descendant_was_ready_and_terminated, parse_and_validate_badge,
    parse_and_validate_junit, parse_and_validate_markdown, parse_and_validate_report,
    validate_report_value,
};

const REDACTION_SENTINEL: &str = "synthetic-secret-payload-7f2c";
const CATALOG_SENTINEL: &str = "synthetic-secret-payload-never-report-7f2c";
const REPORT_ONLY_HUMAN: &str = include_str!("fixtures/reports/unsupported-revision.txt");
const REPORT_ONLY_JSON: &str = include_str!("fixtures/reports/unsupported-revision.json");
const TOOL_DESCRIPTION_QUALITY_HUMAN: &str =
    include_str!("fixtures/reports/tool-description-quality.finding.txt");
const TOOL_DESCRIPTION_PLACEHOLDER_HUMAN: &str =
    include_str!("fixtures/reports/tool-description-placeholder.finding.txt");

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

fn current_inspect_command(
    environment: &TestEnvironment,
    format: Option<&str>,
    mode: &str,
) -> Command {
    let mut command = environment.command();
    command
        .arg("inspect")
        .arg("--protocol-version")
        .arg("2026-07-28");
    if let Some(format) = format {
        command.arg("--format").arg(format);
    }
    command.arg("--").arg(fixture()).arg(mode);
    command
}

fn auto_legacy_inspect_command(
    environment: &TestEnvironment,
    format: Option<&str>,
    explicit_auto: bool,
    signal: &str,
    selected_revision: &str,
) -> Command {
    let mut command = environment.command();
    command.arg("inspect");
    if explicit_auto {
        command.arg("--protocol-version").arg("auto");
    }
    if let Some(format) = format {
        command.arg("--format").arg(format);
    }
    command
        .arg("--")
        .arg(fixture())
        .arg("auto-legacy")
        .arg(environment.artifact_path("auto-process-state"))
        .arg(signal)
        .arg(selected_revision);
    command
}

fn assert_protocol_selection(
    report: &serde_json::Value,
    mode: &str,
    path: &str,
    selected_revision: Option<&str>,
    counts: [u64; 4],
) {
    let [
        process_launches,
        lifecycle_requests,
        lifecycle_notifications,
        fallbacks,
    ] = counts;
    let selection = &report["protocol_selection"];
    assert_eq!(selection["mode"], mode);
    assert_eq!(selection["path"], path);
    match selected_revision {
        Some(revision) => assert_eq!(selection["selected_revision"], revision),
        None => assert!(selection.get("selected_revision").is_none()),
    }
    assert_eq!(selection["process_launches"], process_launches);
    assert_eq!(selection["lifecycle_requests"], lifecycle_requests);
    assert_eq!(
        selection["lifecycle_notifications"],
        lifecycle_notifications
    );
    assert_eq!(selection["fallbacks"], fallbacks);
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

fn structural_metrics(value: &serde_json::Value) -> (usize, usize) {
    let mut nodes = 0_usize;
    let mut maximum_depth = 0_usize;
    let mut stack = vec![(value, 1_usize)];
    while let Some((value, depth)) = stack.pop() {
        nodes += 1;
        maximum_depth = maximum_depth.max(depth);
        match value {
            serde_json::Value::Array(values) => {
                stack.extend(values.iter().map(|value| (value, depth + 1)));
            }
            serde_json::Value::Object(values) => {
                stack.extend(values.values().map(|value| (value, depth + 1)));
            }
            _ => {}
        }
    }
    (nodes, maximum_depth)
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
        assert_protocol_selection(&report, "exact", "exact_pin", Some(revision), [1, 1, 1, 0]);
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
fn explicit_current_lifecycle_rejections_are_revision_diagnoses_in_every_reporter() {
    for (mode, error_kind, code) in [
        (
            "passive-lifecycle-method-not-found",
            "method_not_found",
            -32601,
        ),
        ("passive-lifecycle-invalid-params", "invalid_params", -32602),
    ] {
        let environment = TestEnvironment::new();
        let json = current_inspect_command(&environment, Some("json"), mode)
            .output()
            .expect("mcp-doctor should diagnose the selected lifecycle rejection");
        let (json_text, json_stderr) = text(&json);
        assert_eq!(json.status.code(), Some(1), "{json_text}\n{json_stderr}");
        assert!(json_stderr.is_empty());
        let report = json_report(&json);
        assert_protocol_selection(
            &report,
            "exact",
            "exact_pin",
            Some("2026-07-28"),
            [1, 1, 0, 0],
        );
        assert_eq!(report["primary_diagnosis"]["check_id"], "protocol.revision");
        let revision = find_json_check(&report, "protocol.revision");
        assert_eq!(revision["state"], "performed");
        assert_eq!(revision["outcome"], "failed");
        let finding = &revision["findings"][0];
        assert_eq!(finding["code"], "MCP-PROTOCOL-006");
        assert_eq!(finding["location"], "server/discover.response");
        assert_eq!(finding["evidence"]["kind"], "json_rpc_error");
        assert_eq!(finding["evidence"]["error_kind"], error_kind);
        assert_eq!(finding["evidence"]["code"], code);
        assert_eq!(
            find_json_check(&report, "protocol.envelope")["outcome"],
            "passed"
        );
        for check_id in ["discovery.catalogs", "schema.contracts"] {
            let check = find_json_check(&report, check_id);
            assert_eq!(check["state"], "skipped");
            assert_eq!(check["skip_reason"], "prerequisite_failed");
            assert_eq!(check["blocked_by"]["check_id"], "protocol.revision");
        }
        assert!(!json_text.contains("MCP-CATALOG-001"));
        assert!(!json_text.contains(REDACTION_SENTINEL));

        let human = current_inspect_command(&environment, None, mode)
            .output()
            .expect("mcp-doctor should render the lifecycle diagnosis for a person");
        let (human_text, human_stderr) = text(&human);
        assert_eq!(human.status.code(), Some(1), "{human_text}\n{human_stderr}");
        assert!(human_stderr.is_empty());
        assert!(human_text.contains("PRIMARY DIAGNOSIS · protocol.revision"));
        assert!(human_text.contains("MCP-PROTOCOL-006 · server/discover.response"));
        assert!(human_text.contains(&format!("json_rpc_error {error_kind} · code {code}")));
        assert!(human_text.contains("--protocol-version 2025-11-25"));
        assert!(human_text.contains("--protocol-version 2025-06-18"));
        assert!(!human_text.contains(REDACTION_SENTINEL));

        let junit = current_inspect_command(&environment, Some("junit"), mode)
            .output()
            .expect("mcp-doctor should project the lifecycle diagnosis as JUnit");
        assert_eq!(junit.status.code(), Some(1), "{:?}", text(&junit));
        let (junit_text, summary) = parse_and_validate_junit(&junit.stdout);
        assert_eq!(summary.failures, 1);
        assert!(junit_text.contains("type=\"MCP-PROTOCOL-006\""));
        assert!(junit_text.contains("finding[0].location=server/discover.response"));
        assert!(junit_text.contains(&format!("finding[0].evidence.error_kind={error_kind}")));
        assert!(junit_text.contains(&format!("finding[0].evidence.code={code}")));
        assert!(!junit_text.contains(REDACTION_SENTINEL));
    }
}

#[test]
fn application_defined_lifecycle_error_codes_are_never_retained() {
    let environment = TestEnvironment::new();
    for format in [None, Some("json"), Some("junit")] {
        let output =
            current_inspect_command(&environment, format, "passive-lifecycle-application-error")
                .output()
                .expect("mcp-doctor should redact an application-defined lifecycle error");
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
        assert!(stderr.is_empty());
        assert!(stdout.contains("MCP-PROTOCOL-006"));
        assert!(stdout.contains("other"));
        assert!(!stdout.contains("-31999"));
        assert!(!stdout.contains("918273"));
        assert!(!stdout.contains(REDACTION_SENTINEL));
        if format == Some("json") {
            let report = json_report(&output);
            let evidence =
                &find_json_check(&report, "protocol.revision")["findings"][0]["evidence"];
            assert_eq!(evidence["error_kind"], "other");
            assert!(evidence.get("code").is_none());
        } else if format == Some("junit") {
            parse_and_validate_junit(&output.stdout);
        }
    }
}

#[test]
fn explicit_legacy_lifecycle_rejections_stop_at_the_revision_layer() {
    for revision in ["2025-11-25", "2025-06-18"] {
        let environment = TestEnvironment::new();
        let output = legacy_inspect_command(
            &environment,
            revision,
            Some("json"),
            "legacy-lifecycle-method-not-found",
        )
        .output()
        .expect("mcp-doctor should diagnose a selected legacy lifecycle rejection");
        let (stdout, stderr) = text(&output);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{revision}: {stdout}\n{stderr}"
        );
        assert!(stderr.is_empty());
        let report = json_report(&output);
        assert_eq!(report["protocol_revision"], revision);
        assert_eq!(report["primary_diagnosis"]["check_id"], "protocol.revision");
        let finding = &find_json_check(&report, "protocol.revision")["findings"][0];
        assert_eq!(finding["code"], "MCP-PROTOCOL-006");
        assert_eq!(finding["location"], "initialize.response");
        assert_eq!(finding["evidence"]["error_kind"], "method_not_found");
        assert_eq!(finding["evidence"]["code"], -32601);
        assert_eq!(
            find_json_check(&report, "protocol.envelope")["outcome"],
            "passed"
        );
        assert_eq!(
            find_json_check(&report, "discovery.catalogs")["skip_reason"],
            "prerequisite_failed"
        );
        assert!(!stdout.contains(REDACTION_SENTINEL));
        assert!(!stdout.contains("MCP-CATALOG-001"));
    }
}

#[test]
fn selected_revision_catalog_rejections_name_each_fixed_method() {
    let locations = [
        "tools/list.response",
        "prompts/list.response",
        "resources/list.response",
        "resources/templates/list.response",
    ];
    for (revision, legacy, mode) in [
        ("2026-07-28", false, "passive-catalog-method-errors"),
        ("2025-11-25", true, "legacy-catalog-method-errors"),
        ("2025-06-18", true, "legacy-catalog-method-errors"),
    ] {
        let environment = TestEnvironment::new();
        let output = if legacy {
            legacy_inspect_command(&environment, revision, Some("json"), mode)
        } else {
            current_inspect_command(&environment, Some("json"), mode)
        }
        .output()
        .expect("mcp-doctor should diagnose selected-revision catalog rejections");
        let (stdout, stderr) = text(&output);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{revision}: {stdout}\n{stderr}"
        );
        assert!(stderr.is_empty());
        let report = json_report(&output);
        assert_eq!(
            report["primary_diagnosis"]["check_id"],
            "discovery.catalogs"
        );
        assert_eq!(
            find_json_check(&report, "protocol.revision")["outcome"],
            "passed"
        );
        let findings = find_json_check(&report, "discovery.catalogs")["findings"]
            .as_array()
            .expect("catalog findings should be an array");
        assert_eq!(findings.len(), locations.len(), "{report:#}");
        for (finding, location) in findings.iter().zip(locations) {
            assert_eq!(finding["code"], "MCP-CATALOG-004");
            assert_eq!(finding["location"], location);
            assert_eq!(finding["evidence"]["error_kind"], "method_not_found");
            assert_eq!(finding["evidence"]["code"], -32601);
        }
        assert_eq!(
            find_json_check(&report, "schema.contracts")["skip_reason"],
            "prerequisite_failed"
        );
        assert!(!stdout.contains("MCP-CATALOG-001"));
        assert!(!stdout.contains(REDACTION_SENTINEL));
    }
}

#[test]
fn current_catalog_rejection_human_and_junit_reports_match_json_diagnosis() {
    let environment = TestEnvironment::new();
    let human = current_inspect_command(&environment, None, "passive-catalog-method-errors")
        .output()
        .expect("mcp-doctor should render catalog rejections for a person");
    let (human_text, human_stderr) = text(&human);
    assert_eq!(human.status.code(), Some(1), "{human_text}\n{human_stderr}");
    assert!(human_stderr.is_empty());
    assert!(human_text.contains("PRIMARY DIAGNOSIS · discovery.catalogs"));
    for location in [
        "tools/list.response",
        "prompts/list.response",
        "resources/list.response",
        "resources/templates/list.response",
    ] {
        assert!(
            human_text.contains(&format!("MCP-CATALOG-004 · {location}")),
            "{human_text}"
        );
    }
    assert!(human_text.contains("json_rpc_error method_not_found · code -32601"));
    assert!(!human_text.contains(REDACTION_SENTINEL));

    let junit =
        current_inspect_command(&environment, Some("junit"), "passive-catalog-method-errors")
            .output()
            .expect("mcp-doctor should project catalog rejections as JUnit");
    assert_eq!(junit.status.code(), Some(1), "{:?}", text(&junit));
    let (junit_text, summary) = parse_and_validate_junit(&junit.stdout);
    assert_eq!(summary.failures, 1);
    assert!(junit_text.contains("type=\"MCP-CATALOG-004\""));
    assert!(junit_text.contains("finding["));
    assert!(junit_text.contains(".evidence.kind=json_rpc_error"));
    assert!(junit_text.contains(".evidence.error_kind=method_not_found"));
    assert!(junit_text.contains(".evidence.code=-32601"));
    assert!(!junit_text.contains(REDACTION_SENTINEL));
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
fn default_and_explicit_auto_select_modern_stdio_once() {
    for explicit_auto in [false, true] {
        let environment = TestEnvironment::new();
        let unexpected_request = environment.artifact_path("unexpected-request");
        let mut command = environment.command();
        command.arg("inspect");
        if explicit_auto {
            command.arg("--protocol-version").arg("auto");
        }
        let output = command
            .arg("--format")
            .arg("json")
            .arg("--")
            .arg(fixture())
            .arg("success")
            .arg(&unexpected_request)
            .output()
            .expect("mcp-doctor should auto-select the current STDIO revision");
        let (stdout, stderr) = text(&output);

        assert!(output.status.success(), "{stdout}\n{stderr}");
        assert!(stderr.is_empty());
        let report = json_report(&output);
        assert_protocol_selection(
            &report,
            "auto",
            "modern_discovery",
            Some("2026-07-28"),
            [1, 1, 0, 0],
        );
        assert_eq!(report["protocol_revision"], "2026-07-28");
        assert!(!unexpected_request.exists());
    }
}

#[test]
fn auto_stdio_legacy_signals_restart_once_and_select_supported_revisions() {
    for (signal, selected_revision, explicit_auto) in [
        ("method-not-found", "2025-11-25", false),
        ("invalid-params", "2025-11-25", true),
        ("application-error", "2025-06-18", false),
    ] {
        let environment = TestEnvironment::new();
        let output = auto_legacy_inspect_command(
            &environment,
            Some("json"),
            explicit_auto,
            signal,
            selected_revision,
        )
        .output()
        .expect("mcp-doctor should restart once for a finite STDIO legacy signal");
        let (stdout, stderr) = text(&output);

        assert!(
            output.status.success(),
            "{signal} {selected_revision}: {stdout}\n{stderr}"
        );
        assert!(stderr.is_empty());
        let report = json_report(&output);
        assert_protocol_selection(
            &report,
            "auto",
            "stdio_legacy_initialization",
            Some(selected_revision),
            [2, 2, 1, 1],
        );
        assert_eq!(report["protocol_revision"], selected_revision);
        assert_eq!(report["negotiated_protocol_revision"], selected_revision);
        assert!(!stdout.contains(REDACTION_SENTINEL));
    }
}

#[test]
fn auto_stdio_selection_evidence_matches_human_json_and_junit_reporters() {
    let environment = TestEnvironment::new();
    let json_path = environment.artifact_path("auto-report.json");
    let junit_path = environment.artifact_path("auto-report.xml");
    let output = environment
        .command()
        .arg("inspect")
        .arg("--json-report")
        .arg(&json_path)
        .arg("--junit-report")
        .arg(&junit_path)
        .arg("--")
        .arg(fixture())
        .arg("auto-legacy")
        .arg(environment.artifact_path("auto-process-state"))
        .arg("method-not-found")
        .arg("2025-06-18")
        .output()
        .expect("mcp-doctor should render one auto-selected result in every reporter");
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains(
        "protocol negotiation · mode=auto · path=stdio_legacy_initialization · selected=2025-06-18 · process_launches=2 · lifecycle_requests=2 · lifecycle_notifications=1 · fallbacks=1"
    ));
    let report = parse_and_validate_report(
        &std::fs::read(&json_path).expect("the auto JSON report should exist"),
    );
    assert_protocol_selection(
        &report,
        "auto",
        "stdio_legacy_initialization",
        Some("2025-06-18"),
        [2, 2, 1, 1],
    );
    let (junit, _) = parse_and_validate_junit(
        &std::fs::read(&junit_path).expect("the auto JUnit report should exist"),
    );
    for evidence in [
        "protocol_selection.mode=auto",
        "protocol_selection.path=stdio_legacy_initialization",
        "protocol_selection.selected_revision=2025-06-18",
        "protocol_selection.process_launches=2",
        "protocol_selection.lifecycle_requests=2",
        "protocol_selection.lifecycle_notifications=1",
        "protocol_selection.fallbacks=1",
    ] {
        assert!(junit.contains(evidence), "missing {evidence}: {junit}");
    }
    assert!(!stdout.contains(REDACTION_SENTINEL));
    assert!(!junit.contains(REDACTION_SENTINEL));
}

#[test]
fn auto_stdio_clean_exit_and_discovery_timeout_are_bounded_legacy_signals() {
    for signal in ["clean-exit", "timeout"] {
        let environment = TestEnvironment::new();
        let output =
            auto_legacy_inspect_command(&environment, Some("json"), false, signal, "2025-11-25")
                .output()
                .expect("mcp-doctor should bound the STDIO era probe and restart once");
        let (stdout, stderr) = text(&output);

        assert!(output.status.success(), "{signal}: {stdout}\n{stderr}");
        assert!(stderr.is_empty());
        let report = json_report(&output);
        assert_protocol_selection(
            &report,
            "auto",
            "stdio_legacy_initialization",
            Some("2025-11-25"),
            [2, 2, 1, 1],
        );
    }
}

#[test]
fn auto_stdio_modern_evidence_never_enters_the_legacy_wire_era() {
    for advertisement in ["no-mutual", "contradictory", "limit"] {
        let environment = TestEnvironment::new();
        let marker = environment.artifact_path("single-modern-process");
        let output = environment
            .command()
            .arg("inspect")
            .arg("--format")
            .arg("json")
            .arg("--")
            .arg(fixture())
            .arg("auto-modern-error")
            .arg(&marker)
            .arg(advertisement)
            .output()
            .expect("mcp-doctor should fail closed on recognized modern evidence");
        let (stdout, stderr) = text(&output);

        assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
        assert!(stderr.is_empty());
        let report = json_report(&output);
        assert_protocol_selection(&report, "auto", "modern_discovery", None, [1, 1, 0, 0]);
        assert_eq!(report["primary_diagnosis"]["check_id"], "protocol.revision");
        assert_eq!(
            report["primary_diagnosis"]["findings"][0]["code"],
            if advertisement == "contradictory" {
                "MCP-PROTOCOL-006"
            } else if advertisement == "limit" {
                "MCP-LIMIT-001"
            } else {
                "MCP-PROTOCOL-002"
            }
        );
        if advertisement == "limit" {
            let finding = &find_json_check(&report, "protocol.revision")["findings"][0];
            assert_eq!(finding["code"], "MCP-LIMIT-001");
            assert_eq!(finding["evidence"]["limit"], "protocol_revisions");
            assert_eq!(finding["evidence"]["observed"], 33);
            assert_eq!(finding["evidence"]["maximum"], 32);
        }
        assert!(!stdout.contains(REDACTION_SENTINEL));
    }

    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("single-modern-process");
    let output = environment
        .command()
        .arg("inspect")
        .arg("--format")
        .arg("json")
        .arg("--")
        .arg(fixture())
        .arg("auto-modern-no-mutual")
        .arg(&marker)
        .output()
        .expect("mcp-doctor should not treat a modern result as legacy authority");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = json_report(&output);
    assert_protocol_selection(&report, "auto", "modern_discovery", None, [1, 1, 0, 0]);
    assert_eq!(report["primary_diagnosis"]["check_id"], "protocol.revision");
    assert!(!stdout.contains(REDACTION_SENTINEL));

    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("single-invalid-modern-process");
    let output = environment
        .command()
        .arg("inspect")
        .arg("--format")
        .arg("json")
        .arg("--")
        .arg(fixture())
        .arg("auto-modern-invalid-result")
        .arg(&marker)
        .output()
        .expect("an invalid modern result must stop before catalog work");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = json_report(&output);
    assert_protocol_selection(&report, "auto", "modern_discovery", None, [1, 1, 0, 0]);
    assert_eq!(report["primary_diagnosis"]["check_id"], "protocol.envelope");
    assert!(!stdout.contains(REDACTION_SENTINEL));
}

#[test]
fn auto_stdio_terminal_failures_and_cleanup_never_restart() {
    for mode in ["malformed", "oversized-message", "message-count"] {
        let environment = TestEnvironment::new();
        let output = json_inspect_command(&environment, mode)
            .output()
            .expect("mcp-doctor should fail a terminal STDIO probe without fallback");
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(1), "{mode}: {stdout}\n{stderr}");
        assert!(stderr.is_empty());
        let report = json_report(&output);
        assert_protocol_selection(&report, "auto", "modern_discovery", None, [1, 1, 0, 0]);
    }

    let environment = TestEnvironment::new();
    let output = auto_legacy_inspect_command(
        &environment,
        Some("json"),
        false,
        "malformed-exit",
        "2025-11-25",
    )
    .output()
    .expect("partial output must override an apparent clean legacy signal");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = json_report(&output);
    assert_protocol_selection(&report, "auto", "modern_discovery", None, [1, 1, 0, 0]);
    assert!(stdout.contains("MCP-TRANSPORT-003"));

    let environment = TestEnvironment::new();
    let output = auto_legacy_inspect_command(
        &environment,
        Some("json"),
        false,
        "method-not-found",
        "2025-11-25",
    )
    .env("MCP_DOCTOR_INTERNAL_TEST_CLEANUP_FAILURE", "1")
    .output()
    .expect("mcp-doctor should stop auto selection on cleanup failure");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = json_report(&output);
    assert_protocol_selection(&report, "auto", "modern_discovery", None, [1, 1, 0, 0]);
    assert!(
        report["independent_findings"]
            .as_array()
            .is_some_and(|findings| !findings.is_empty())
    );
}

#[test]
fn auto_stdio_rejects_unsupported_or_unknown_legacy_counteroffers() {
    for selected_revision in [
        "2025-03-26",
        "synthetic-private-unknown-revision-never-report-7f2c",
    ] {
        let environment = TestEnvironment::new();
        let output = auto_legacy_inspect_command(
            &environment,
            Some("json"),
            false,
            "method-not-found",
            selected_revision,
        )
        .output()
        .expect("mcp-doctor should reject a non-supported legacy counteroffer");
        let (stdout, stderr) = text(&output);

        assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
        assert!(stderr.is_empty());
        let report = json_report(&output);
        assert_protocol_selection(
            &report,
            "auto",
            "stdio_legacy_initialization",
            None,
            [2, 2, 0, 1],
        );
        assert_eq!(report["primary_diagnosis"]["check_id"], "protocol.revision");
        if selected_revision.starts_with("synthetic-private-") {
            assert!(!stdout.contains(selected_revision));
        }
        assert!(!stdout.contains(REDACTION_SENTINEL));
    }
}

#[test]
fn auto_stdio_uses_one_cumulative_output_and_message_budget() {
    for (signal, limit) in [
        ("cumulative-stderr", "stderr_bytes"),
        ("cumulative-messages", "message_count"),
    ] {
        let environment = TestEnvironment::new();
        let output =
            auto_legacy_inspect_command(&environment, Some("json"), false, signal, "2025-11-25")
                .output()
                .expect("mcp-doctor should preserve aggregate STDIO budgets across phases");
        let (stdout, stderr) = text(&output);

        assert_eq!(
            output.status.code(),
            Some(1),
            "{signal}: {stdout}\n{stderr}"
        );
        assert!(stderr.is_empty());
        let report = json_report(&output);
        assert_protocol_selection(
            &report,
            "auto",
            "stdio_legacy_initialization",
            None,
            [2, 2, 0, 1],
        );
        assert!(stdout.contains(limit), "{signal}: {stdout}");
        assert!(!stdout.contains(REDACTION_SENTINEL));
    }

    let environment = TestEnvironment::new();
    let output = auto_legacy_inspect_command(
        &environment,
        Some("json"),
        false,
        "method-not-found",
        "2025-11-25",
    )
    .env("MCP_DOCTOR_INTERNAL_TEST_EXHAUST_AUTO_TOTAL_BUDGET", "1")
    .output()
    .expect("mcp-doctor should preserve the original total deadline across phases");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = json_report(&output);
    assert_protocol_selection(
        &report,
        "auto",
        "stdio_legacy_initialization",
        None,
        [1, 1, 0, 1],
    );
    let finding = &find_json_check(&report, "transport.stdio")["findings"][0];
    assert_eq!(finding["code"], "MCP-LIMIT-001");
    assert_eq!(finding["evidence"]["limit"], "total_time");
    assert!(!stdout.contains(REDACTION_SENTINEL));
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
    let environment = TestEnvironment::new();
    let output = current_inspect_command(&environment, None, "timeout")
        .output()
        .expect("mcp-doctor should bound exact-current discovery");
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
fn missing_and_blank_tool_descriptions_are_value_free_across_reporters_and_revisions() {
    const SENTINEL: &str = "synthetic-private-tool-description-never-report-61";
    const REMEDIATION: &str =
        "Provide a concise description of what the tool does and when to select it.";
    let human_output = run_mode("tool-description-quality");
    let json_output = run_json_mode("tool-description-quality");
    let environment = TestEnvironment::new();
    let junit_output = junit_inspect_command(&environment, "tool-description-quality")
        .output()
        .expect("mcp-doctor should project description warnings as JUnit");
    let (human, human_stderr) = text(&human_output);
    let (json_text, json_stderr) = text(&json_output);
    let (_, junit_stderr) = text(&junit_output);
    let report = json_report(&json_output);
    let (junit, junit_summary) = parse_and_validate_junit(&junit_output.stdout);

    assert!(human_output.status.success(), "{human}\n{human_stderr}");
    assert!(json_output.status.success(), "{report:#}\n{json_stderr}");
    assert!(junit_output.status.success(), "{junit}\n{junit_stderr}");
    assert!(human_stderr.is_empty());
    assert!(json_stderr.is_empty());
    assert!(junit_stderr.is_empty());
    assert_eq!(report["outcome"], "passed");
    assert_eq!(report["exit_code"], 0);
    assert_eq!(report["primary_diagnosis"], serde_json::Value::Null);
    assert_eq!(report["summary"]["warned"], 1);
    assert_eq!(junit_summary.failures, 0);
    assert!(human.contains("WARN  discovery.catalogs"), "{human}");
    assert!(
        human.contains(TOOL_DESCRIPTION_QUALITY_HUMAN),
        "the human report should retain the reviewed quality-finding golden output: {human}"
    );
    assert!(
        junit.contains("report_outcome=passed\nexit_code=0"),
        "{junit}"
    );

    let findings = find_json_check(&report, "discovery.catalogs")["findings"]
        .as_array()
        .expect("the discovery check should contain findings");
    assert_eq!(findings.len(), 3, "{report:#}");
    for (index, finding) in findings.iter().enumerate() {
        let location = format!("tools[{index}].description");
        assert_eq!(finding["code"], "MCP-QUALITY-001");
        assert_eq!(finding["severity"], "warning");
        assert_eq!(finding["protocol_revision"], "2026-07-28");
        assert_eq!(finding["location"], location);
        assert_eq!(
            finding["message"],
            "An advertised tool has no usable description."
        );
        assert_eq!(finding["remediation"], REMEDIATION);
        assert!(human.contains("MCP-QUALITY-001"), "{human}");
        assert!(human.contains(&location), "{human}");
        assert!(human.contains(REMEDIATION), "{human}");
        assert!(
            junit.contains(&format!("finding[{index}].code=MCP-QUALITY-001")),
            "{junit}"
        );
        assert!(
            junit.contains(&format!("finding[{index}].location={location}")),
            "{junit}"
        );
        assert!(
            junit.contains(&format!("finding[{index}].remediation={REMEDIATION}")),
            "{junit}"
        );
    }
    assert!(!human.contains("tools[3].description"), "{human}");
    assert!(!json_text.contains("tools[3].description"), "{json_text}");
    assert!(!junit.contains("tools[3].description"), "{junit}");
    for output in [human, json_text, junit.as_str()] {
        assert!(!output.contains(SENTINEL), "{output}");
        assert!(!output.contains("synthetic-private"), "{output}");
    }
    assert_report_findings_are_actionable(&report, human);

    for revision in ["2025-11-25", "2025-06-18"] {
        let environment = TestEnvironment::new();
        let output = legacy_inspect_command(
            &environment,
            revision,
            Some("json"),
            "legacy-tool-description-quality",
        )
        .output()
        .expect("mcp-doctor should inspect legacy descriptions through the shared rule");
        let (legacy_json, legacy_stderr) = text(&output);
        let legacy = json_report(&output);
        assert!(
            output.status.success(),
            "{revision}: {legacy:#}\n{legacy_stderr}"
        );
        assert!(legacy_stderr.is_empty());
        assert_eq!(legacy["outcome"], "passed");
        let legacy_findings = find_json_check(&legacy, "discovery.catalogs")["findings"]
            .as_array()
            .expect("the legacy discovery check should contain findings");
        assert_eq!(legacy_findings.len(), 3, "{legacy:#}");
        for (index, finding) in legacy_findings.iter().enumerate() {
            assert_eq!(finding["code"], "MCP-QUALITY-001");
            assert_eq!(finding["severity"], "warning");
            assert_eq!(finding["protocol_revision"], revision);
            assert_eq!(finding["location"], format!("tools[{index}].description"));
            assert_eq!(finding["remediation"], REMEDIATION);
        }
        assert!(!legacy_json.contains(SENTINEL), "{legacy_json}");
        assert!(!legacy_json.contains("synthetic-private"), "{legacy_json}");
    }
}

#[test]
fn placeholder_and_name_only_descriptions_are_value_free_across_reporters_and_revisions() {
    const SENTINEL: &str = "synthetic-private-placeholder-never-report-64";
    const NORMALIZED_SENTINEL: &str = "syntheticprivateplaceholderneverreport64nameonly";
    const REMEDIATION: &str = "Replace the placeholder or name-only description with what the tool does and when to select it.";
    let human_output = run_mode("tool-description-placeholder");
    let json_output = run_json_mode("tool-description-placeholder");
    let environment = TestEnvironment::new();
    let junit_output = junit_inspect_command(&environment, "tool-description-placeholder")
        .output()
        .expect("mcp-doctor should project placeholder warnings as JUnit");
    let (human, human_stderr) = text(&human_output);
    let (json_text, json_stderr) = text(&json_output);
    let (_, junit_stderr) = text(&junit_output);
    let report = json_report(&json_output);
    let (junit, junit_summary) = parse_and_validate_junit(&junit_output.stdout);

    assert!(human_output.status.success(), "{human}\n{human_stderr}");
    assert!(json_output.status.success(), "{report:#}\n{json_stderr}");
    assert!(junit_output.status.success(), "{junit}\n{junit_stderr}");
    assert!(human_stderr.is_empty());
    assert!(json_stderr.is_empty());
    assert!(junit_stderr.is_empty());
    assert_eq!(report["outcome"], "passed");
    assert_eq!(report["exit_code"], 0);
    assert_eq!(report["primary_diagnosis"], serde_json::Value::Null);
    assert_eq!(report["summary"]["warned"], 1);
    assert_eq!(junit_summary.failures, 0);
    assert!(human.contains("WARN  discovery.catalogs"), "{human}");
    assert!(
        human.contains(TOOL_DESCRIPTION_PLACEHOLDER_HUMAN),
        "the human report should retain the reviewed placeholder-finding golden output: {human}"
    );
    assert!(
        junit.contains("report_outcome=passed\nexit_code=0"),
        "{junit}"
    );

    let findings = find_json_check(&report, "discovery.catalogs")["findings"]
        .as_array()
        .expect("the discovery check should contain findings");
    assert_eq!(findings.len(), 4, "{report:#}");
    for (index, finding) in findings.iter().enumerate() {
        let location = format!("tools[{index}].description");
        assert_eq!(finding["code"], "MCP-QUALITY-003");
        assert_eq!(finding["severity"], "warning");
        assert_eq!(finding["protocol_revision"], "2026-07-28");
        assert_eq!(finding["location"], location);
        assert_eq!(
            finding["message"],
            "An advertised tool description provides no selection guidance."
        );
        assert_eq!(finding["remediation"], REMEDIATION);
        assert_eq!(finding["evidence"]["kind"], "none");
        assert!(human.contains("MCP-QUALITY-003"), "{human}");
        assert!(human.contains(&location), "{human}");
        assert!(human.contains(REMEDIATION), "{human}");
        assert!(
            junit.contains(&format!("finding[{index}].code=MCP-QUALITY-003")),
            "{junit}"
        );
        assert!(
            junit.contains(&format!("finding[{index}].location={location}")),
            "{junit}"
        );
        assert!(
            junit.contains(&format!("finding[{index}].remediation={REMEDIATION}")),
            "{junit}"
        );
    }
    assert!(!human.contains("tools[4].description"), "{human}");
    assert!(!json_text.contains("tools[4].description"), "{json_text}");
    assert!(!junit.contains("tools[4].description"), "{junit}");
    assert!(!human.contains("tools[5].description"), "{human}");
    assert!(!json_text.contains("tools[5].description"), "{json_text}");
    assert!(!junit.contains("tools[5].description"), "{junit}");
    for (reporter_index, output) in [human, json_text, junit.as_str()].into_iter().enumerate() {
        for (canary_index, forbidden) in [
            SENTINEL,
            NORMALIZED_SENTINEL,
            "SYNTHETIC_PRIVATE_PLACEHOLDER_NEVER_REPORT_64_NAME_ONLY",
            "T.O.D.O",
            "工具",
            "café",
            "CAFE",
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                !output.contains(forbidden),
                "reporter {reporter_index} retained redaction canary {canary_index}"
            );
        }
        assert!(
            !output.contains("MCP-QUALITY-001"),
            "reporter {reporter_index} emitted the blank-description code"
        );
    }
    assert_report_findings_are_actionable(&report, human);

    for revision in ["2025-11-25", "2025-06-18"] {
        let environment = TestEnvironment::new();
        let output = legacy_inspect_command(
            &environment,
            revision,
            Some("json"),
            "legacy-tool-description-placeholder",
        )
        .output()
        .expect("mcp-doctor should reuse placeholder semantics for legacy revisions");
        let (legacy_json, legacy_stderr) = text(&output);
        let legacy = json_report(&output);
        assert!(
            output.status.success(),
            "{revision}: {legacy:#}\n{legacy_stderr}"
        );
        assert!(legacy_stderr.is_empty());
        assert_eq!(legacy["outcome"], "passed");
        let legacy_findings = find_json_check(&legacy, "discovery.catalogs")["findings"]
            .as_array()
            .expect("the legacy discovery check should contain findings");
        assert_eq!(legacy_findings.len(), 4, "{legacy:#}");
        for (index, finding) in legacy_findings.iter().enumerate() {
            assert_eq!(finding["code"], "MCP-QUALITY-003");
            assert_eq!(finding["severity"], "warning");
            assert_eq!(finding["protocol_revision"], revision);
            assert_eq!(finding["location"], format!("tools[{index}].description"));
            assert_eq!(finding["remediation"], REMEDIATION);
            assert_eq!(finding["evidence"]["kind"], "none");
        }
        for (canary_index, forbidden) in [
            SENTINEL,
            NORMALIZED_SENTINEL,
            "SYNTHETIC_PRIVATE_PLACEHOLDER_NEVER_REPORT_64_NAME_ONLY",
            "T.O.D.O",
            "工具",
            "café",
            "CAFE",
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                !legacy_json.contains(forbidden),
                "revision {revision} retained redaction canary {canary_index}"
            );
        }
    }
}

#[test]
fn reused_normalized_descriptions_are_value_free_across_reporters_and_revisions() {
    const SENTINEL: &str = "synthetic-private-reused-description-never-report-7f3b";
    const NORMALIZED_SENTINEL: &str = "syntheticprivatereuseddescriptionneverreport7f3b";
    const REMEDIATION: &str = "Distinguish what this tool does, when it should and should not be selected, and how it differs from the tool at first_matching_tool_index.";
    let artifact_environment = TestEnvironment::new();
    let json_path = artifact_environment.artifact_path("reused-report.json");
    let junit_path = artifact_environment.artifact_path("reused-report.junit.xml");
    let markdown_path = artifact_environment.artifact_path("reused-report.md");
    let badge_path = artifact_environment.artifact_path("reused-badge.json");
    let mut artifact_command = artifact_environment.command();
    artifact_command
        .arg("inspect")
        .arg("--json-report")
        .arg(&json_path)
        .arg("--junit-report")
        .arg(&junit_path)
        .arg("--markdown-report")
        .arg(&markdown_path)
        .arg("--badge-report")
        .arg(&badge_path)
        .arg("--")
        .arg(fixture())
        .arg("tool-description-reused");
    let human_output = artifact_command
        .output()
        .expect("mcp-doctor should write every reused-description report");
    let json_output = run_json_mode("tool-description-reused");
    let environment = TestEnvironment::new();
    let junit_output = junit_inspect_command(&environment, "tool-description-reused")
        .output()
        .expect("mcp-doctor should project reused descriptions as JUnit");
    let (human, human_stderr) = text(&human_output);
    let (json_text, json_stderr) = text(&json_output);
    let (_, junit_stderr) = text(&junit_output);
    let report = json_report(&json_output);
    let (junit, junit_summary) = parse_and_validate_junit(&junit_output.stdout);
    let artifact_json = fs::read(&json_path).expect("the JSON artifact should exist");
    let artifact_junit = fs::read(&junit_path).expect("the JUnit artifact should exist");
    let markdown = parse_and_validate_markdown(
        &fs::read(&markdown_path).expect("the Markdown artifact should exist"),
    );
    let badge =
        parse_and_validate_badge(&fs::read(&badge_path).expect("the badge artifact should exist"));

    for output in [&human_output, &json_output, &junit_output] {
        assert!(output.status.success(), "{:?}", text(output));
    }
    for stderr in [human_stderr, json_stderr, junit_stderr] {
        assert!(stderr.is_empty(), "{stderr}");
    }
    assert_eq!(artifact_json, json_output.stdout);
    assert_eq!(artifact_junit, junit_output.stdout);
    assert_eq!(report["outcome"], "passed");
    assert_eq!(report["exit_code"], 0);
    assert_eq!(report["summary"]["warned"], 1);
    assert_eq!(junit_summary.failures, 0);
    assert_eq!(badge["message"], "pass");

    let findings = find_json_check(&report, "discovery.catalogs")["findings"]
        .as_array()
        .expect("the discovery check should contain findings");
    assert_eq!(findings.len(), 3, "{report:#}");
    for (finding_index, tool_index) in [2, 3, 4].into_iter().enumerate() {
        let location = format!("tools[{tool_index}].description");
        let finding = &findings[finding_index];
        assert_eq!(finding["code"], "MCP-QUALITY-004");
        assert_eq!(finding["severity"], "warning");
        assert_eq!(finding["protocol_revision"], "2026-07-28");
        assert_eq!(finding["location"], location);
        assert_eq!(finding["remediation"], REMEDIATION);
        assert_eq!(
            finding["evidence"],
            serde_json::json!({
                "kind": "rule_violation",
                "rule": "reused_normalized_tool_description",
                "first_matching_tool_index": 0
            })
        );
        assert!(human.contains(&location), "{human}");
        assert!(human.contains("first_matching_tool_index 0"), "{human}");
        assert!(human.contains(REMEDIATION), "{human}");
        assert!(
            junit.contains(&format!("finding[{finding_index}].location={location}")),
            "{junit}"
        );
        assert!(
            junit.contains(&format!(
                "finding[{finding_index}].evidence.first_matching_tool_index=0"
            )),
            "{junit}"
        );
        assert!(markdown.contains(&format!("#### Finding {}", finding_index + 1)));
        assert!(markdown.contains(&format!("`{location}`")));
        assert!(markdown.contains("`first_matching_tool_index=0`"));
    }
    assert!(!human.contains("tools[0].description"), "{human}");
    assert!(!json_text.contains("tools[0].description"), "{json_text}");
    assert!(!junit.contains("tools[0].description"), "{junit}");
    assert!(!markdown.contains("tools[0].description"), "{markdown}");
    for rendered in [human, json_text, junit.as_str(), markdown.as_str()] {
        for forbidden in [
            SENTINEL,
            NORMALIZED_SENTINEL,
            "SELECT\tTHE synthetic_private_reused_description_never_report_7f3b",
            "résumé",
            "resume",
        ] {
            assert!(!rendered.contains(forbidden), "{rendered}");
        }
    }
    let badge_bytes = fs::read(&badge_path).expect("the badge artifact should remain readable");
    let badge_text = std::str::from_utf8(&badge_bytes).expect("badge should be UTF-8");
    assert!(!badge_text.contains("MCP-QUALITY-004"));
    assert!(!badge_text.contains("first_matching_tool_index"));
    assert_report_findings_are_actionable(&report, human);

    for revision in ["2025-11-25", "2025-06-18"] {
        let environment = TestEnvironment::new();
        let output = legacy_inspect_command(
            &environment,
            revision,
            Some("json"),
            "legacy-tool-description-reused",
        )
        .output()
        .expect("mcp-doctor should reuse duplicate-description semantics for legacy revisions");
        let (legacy_json, legacy_stderr) = text(&output);
        let legacy = json_report(&output);
        assert!(output.status.success(), "{revision}: {legacy:#}");
        assert!(legacy_stderr.is_empty());
        let legacy_findings = find_json_check(&legacy, "discovery.catalogs")["findings"]
            .as_array()
            .expect("the legacy discovery check should contain findings");
        assert_eq!(legacy_findings.len(), 3, "{legacy:#}");
        for (finding, tool_index) in legacy_findings.iter().zip([2, 3, 4]) {
            assert_eq!(finding["code"], "MCP-QUALITY-004");
            assert_eq!(finding["severity"], "warning");
            assert_eq!(finding["protocol_revision"], revision);
            assert_eq!(
                finding["location"],
                format!("tools[{tool_index}].description")
            );
            assert_eq!(finding["evidence"]["first_matching_tool_index"], 0);
        }
        assert!(!legacy_json.contains(SENTINEL));
        assert!(!legacy_json.contains(NORMALIZED_SENTINEL));
    }
}

#[test]
fn later_stdio_page_failure_discards_reused_description_prefix_findings() {
    let output = run_json_mode("tool-description-reused-later-failure");
    let (stdout, stderr) = text(&output);
    let report = json_report(&output);

    assert_eq!(output.status.code(), Some(1), "{report:#}\n{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(report["primary_diagnosis"]["check_id"], "transport.stdio");
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-TRANSPORT-003"
    );
    for check_id in ["discovery.catalogs", "schema.contracts"] {
        let check = find_json_check(&report, check_id);
        assert_eq!(check["state"], "skipped", "{check:#}");
        assert_eq!(check["blocked_by"]["check_id"], "transport.stdio");
    }
    assert!(!stdout.contains("MCP-QUALITY-004"), "{stdout}");
    assert!(
        !stdout.contains("reused_normalized_tool_description"),
        "{stdout}"
    );
    assert!(
        !stdout.contains("synthetic-private-failed-prefix"),
        "{stdout}"
    );
}

#[test]
fn reused_description_saturation_preserves_security_and_reports_the_global_limit() {
    let first = run_json_mode("tool-description-reused-finding-limit");
    let second = run_json_mode("tool-description-reused-finding-limit");
    let (stdout, stderr) = text(&first);
    let report = json_report(&first);

    assert_eq!(first.status.code(), Some(1), "{report:#}\n{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(
        first.stdout, second.stdout,
        "bounded evidence must be stable"
    );
    let catalog_findings = find_json_check(&report, "discovery.catalogs")["findings"]
        .as_array()
        .expect("the discovery check should retain bounded findings");
    let quality = catalog_findings
        .iter()
        .filter(|finding| finding["code"] == "MCP-QUALITY-004")
        .collect::<Vec<_>>();
    let limits = catalog_findings
        .iter()
        .filter(|finding| finding["code"] == "MCP-LIMIT-001")
        .collect::<Vec<_>>();
    assert_eq!(quality.len(), 253, "{report:#}");
    assert_eq!(quality.first().unwrap()["location"], "tools[1].description");
    assert_eq!(
        quality.last().unwrap()["location"],
        "tools[253].description"
    );
    assert!(quality.iter().all(|finding| {
        finding["severity"] == "warning" && finding["evidence"]["first_matching_tool_index"] == 0
    }));
    assert_eq!(limits.len(), 1, "{report:#}");
    assert_eq!(limits[0]["evidence"]["limit"], "report_findings");
    assert_eq!(limits[0]["evidence"]["maximum"], 256);

    let security = find_json_check(&report, "schema.contracts")["findings"]
        .as_array()
        .expect("the schema check should retain independent security evidence");
    assert!(security.iter().any(|finding| {
        finding["code"] == "MCP-SECURITY-001"
            && finding["location"] == "tools[0].inputSchema.properties[0].default"
    }));
    assert!(
        report["independent_findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "MCP-SECURITY-001")
    );
    assert!(!stdout.contains("synthetic-private"), "{stdout}");
    assert!(!stdout.contains("exact identifier"), "{stdout}");
}

#[test]
fn non_string_tool_descriptions_remain_catalog_errors_without_quality_duplicates() {
    let output = run_json_mode("tool-description-non-string");
    let (stdout, stderr) = text(&output);
    let report = json_report(&output);

    assert_eq!(output.status.code(), Some(1), "{report:#}\n{stderr}");
    assert!(stderr.is_empty());
    let findings = find_json_check(&report, "discovery.catalogs")["findings"]
        .as_array()
        .expect("the discovery check should contain findings");
    assert_eq!(findings.len(), 1, "{report:#}");
    assert_eq!(findings[0]["code"], "MCP-CATALOG-001");
    assert_eq!(findings[0]["location"], "tools[0].description");
    assert_eq!(findings[0]["evidence"]["expected"], "string");
    assert_eq!(findings[0]["evidence"]["observed"], "object");
    assert!(!stdout.contains("MCP-QUALITY-001"), "{stdout}");
    assert!(!stdout.contains("synthetic-private"), "{stdout}");
}

#[test]
fn tool_description_warnings_truncate_deterministically_at_the_report_limit() {
    let first = run_json_mode("tool-description-finding-limit");
    let second = run_json_mode("tool-description-finding-limit");
    let (stdout, stderr) = text(&first);
    let report = json_report(&first);

    assert_eq!(first.status.code(), Some(1), "{report:#}\n{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(
        first.stdout, second.stdout,
        "quality truncation must be stable"
    );
    let findings = find_json_check(&report, "discovery.catalogs")["findings"]
        .as_array()
        .expect("the discovery check should contain bounded findings");
    let quality = findings
        .iter()
        .filter(|finding| finding["code"] == "MCP-QUALITY-001")
        .collect::<Vec<_>>();
    let limits = findings
        .iter()
        .filter(|finding| finding["code"] == "MCP-LIMIT-001")
        .collect::<Vec<_>>();
    assert_eq!(quality.len(), 254, "{report:#}");
    assert_eq!(limits.len(), 1, "{report:#}");
    assert_eq!(
        quality.first().expect("a first warning")["location"],
        "tools[0].description"
    );
    assert_eq!(
        quality.last().expect("a last warning")["location"],
        "tools[253].description"
    );
    assert_eq!(limits[0]["evidence"]["limit"], "report_findings");
    assert_eq!(limits[0]["evidence"]["maximum"], 256);
    assert!(!stdout.contains("synthetic-private"), "{stdout}");
}

#[test]
fn credential_literals_are_redacted_and_consistent_across_reporters_and_revisions() {
    const REMEDIATION: &str = "Remove the literal from the schema and obtain the credential through authorized server runtime configuration.";
    let expected = [
        ("default", 1_u64),
        ("const", 1),
        ("examples", 1),
        ("enum", 2),
        ("default", 1),
        ("const", 1),
        ("examples", 1),
        ("enum", 1),
        ("default", 1),
    ];
    let human_output = run_mode("credential-literals");
    let json_output = run_json_mode("credential-literals");
    let environment = TestEnvironment::new();
    let junit_output = junit_inspect_command(&environment, "credential-literals")
        .output()
        .expect("mcp-doctor should project credential findings as JUnit");
    let (human, human_stderr) = text(&human_output);
    let (json_text, json_stderr) = text(&json_output);
    let (_, junit_stderr) = text(&junit_output);
    let report = json_report(&json_output);
    let (junit, junit_summary) = parse_and_validate_junit(&junit_output.stdout);

    assert_eq!(
        human_output.status.code(),
        Some(1),
        "{human}\n{human_stderr}"
    );
    assert_eq!(
        json_output.status.code(),
        Some(1),
        "{report:#}\n{json_stderr}"
    );
    assert_eq!(
        junit_output.status.code(),
        Some(1),
        "{junit}\n{junit_stderr}"
    );
    assert!(human_stderr.is_empty());
    assert!(json_stderr.is_empty());
    assert!(junit_stderr.is_empty());
    assert_eq!(report["outcome"], "failed");
    assert_eq!(report["exit_code"], 1);
    assert_eq!(junit_summary.failures, 1);
    assert!(human.contains("FAIL  schema.contracts"), "{human}");

    let findings = find_json_check(&report, "schema.contracts")["findings"]
        .as_array()
        .expect("the schema check should contain credential findings");
    assert_eq!(findings.len(), expected.len(), "{report:#}");
    assert_eq!(
        report["independent_findings"]
            .as_array()
            .expect("independent findings should be an array")
            .len(),
        expected.len(),
        "{report:#}"
    );

    for (index, (finding, (keyword, literal_count))) in findings.iter().zip(expected).enumerate() {
        let location = format!("tools[{index}].inputSchema.properties[0].{keyword}");
        assert_eq!(finding["code"], "MCP-SECURITY-001");
        assert_eq!(finding["severity"], "error");
        assert_eq!(finding["protocol_revision"], "2026-07-28");
        assert_eq!(finding["location"], location);
        assert_eq!(finding["remediation"], REMEDIATION);
        assert_eq!(finding["evidence"]["kind"], "credential_literal");
        assert_eq!(finding["evidence"]["keyword_class"], keyword);
        assert_eq!(finding["evidence"]["literal_count"], literal_count);
        assert!(human.contains("MCP-SECURITY-001"), "{human}");
        assert!(human.contains(&location), "{human}");
        assert!(human.contains(REMEDIATION), "{human}");
        assert!(
            human.contains(&format!(
                "keyword {keyword} · {literal_count} non-empty string literal(s)"
            )),
            "{human}"
        );
        assert!(
            junit.contains(&format!("finding[{index}].code=MCP-SECURITY-001")),
            "{junit}"
        );
        assert!(
            junit.contains(&format!("finding[{index}].location={location}")),
            "{junit}"
        );
        assert!(
            junit.contains(&format!(
                "finding[{index}].evidence.keyword_class={keyword}"
            )),
            "{junit}"
        );
        assert!(
            junit.contains(&format!(
                "finding[{index}].evidence.literal_count={literal_count}"
            )),
            "{junit}"
        );
        assert!(
            junit.contains(&format!("finding[{index}].independent_safety=true")),
            "{junit}"
        );
    }

    let forbidden = [
        "synthetic-credential-literal-never-report-63",
        "second-synthetic-value",
        "passwordField",
        "userPasswd",
        "client_secret",
        "authToken",
        "api_key",
        "accessToken",
        "private-key",
        "serviceCredential",
        "tokenizer",
        "secretary",
    ];
    for (reporter_index, output) in [human, json_text, junit.as_str()].into_iter().enumerate() {
        for (canary_index, value) in forbidden.into_iter().enumerate() {
            assert!(
                !output.contains(value),
                "reporter {reporter_index} retained redaction canary {canary_index}"
            );
        }
    }
    assert_report_findings_are_actionable(&report, human);

    let modern_projection = findings
        .iter()
        .map(|finding| {
            serde_json::json!({
                "code": finding["code"],
                "severity": finding["severity"],
                "location": finding["location"],
                "evidence": finding["evidence"]
            })
        })
        .collect::<Vec<_>>();
    for revision in ["2025-11-25", "2025-06-18"] {
        let environment = TestEnvironment::new();
        let output = legacy_inspect_command(
            &environment,
            revision,
            Some("json"),
            "legacy-credential-literals",
        )
        .output()
        .expect("mcp-doctor should reuse credential semantics for legacy revisions");
        let (legacy_json, legacy_stderr) = text(&output);
        let legacy = json_report(&output);
        assert_eq!(output.status.code(), Some(1), "{legacy:#}\n{legacy_stderr}");
        assert!(legacy_stderr.is_empty());
        let legacy_findings = find_json_check(&legacy, "schema.contracts")["findings"]
            .as_array()
            .expect("the legacy schema check should contain credential findings");
        let legacy_projection = legacy_findings
            .iter()
            .map(|finding| {
                assert_eq!(finding["protocol_revision"], revision);
                serde_json::json!({
                    "code": finding["code"],
                    "severity": finding["severity"],
                    "location": finding["location"],
                    "evidence": finding["evidence"]
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(legacy_projection, modern_projection, "{legacy:#}");
        for (canary_index, value) in forbidden.into_iter().enumerate() {
            assert!(
                !legacy_json.contains(value),
                "revision {revision} retained redaction canary {canary_index}"
            );
        }
    }
}

#[test]
fn credential_literal_finding_requires_a_valid_schema_and_remains_independent() {
    let output = run_json_mode("credential-literals-combined");
    let (stdout, stderr) = text(&output);
    let report = json_report(&output);

    assert_eq!(output.status.code(), Some(1), "{report:#}\n{stderr}");
    assert!(stderr.is_empty());
    let findings = find_json_check(&report, "schema.contracts")["findings"]
        .as_array()
        .expect("the schema check should contain both findings");
    assert_eq!(findings.len(), 2, "{report:#}");
    let schema_finding = findings
        .iter()
        .find(|finding| finding["code"] == "MCP-SCHEMA-001")
        .expect("the invalid schema should retain its prerequisite finding");
    let security_finding = findings
        .iter()
        .find(|finding| finding["code"] == "MCP-SECURITY-001")
        .expect("the valid sibling schema should retain its security finding");
    assert!(
        schema_finding["location"]
            .as_str()
            .is_some_and(|location| location.starts_with("tools[0].inputSchema.required")),
        "{report:#}"
    );
    assert_eq!(
        security_finding["location"],
        "tools[1].inputSchema.properties[0].const"
    );
    assert_eq!(report["primary_diagnosis"]["check_id"], "schema.contracts");
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-SCHEMA-001"
    );
    assert_eq!(
        report["independent_findings"],
        serde_json::json!([{
            "check_id": "schema.contracts",
            "code": "MCP-SECURITY-001",
            "location": "tools[1].inputSchema.properties[0].const"
        }])
    );
    assert!(!stdout.contains("tools[0].inputSchema.properties[0].default"));
    assert!(!stdout.contains("synthetic-combined-credential-never-report-63"));
    assert!(!stdout.contains("access_token"));
}

#[test]
fn credential_literal_findings_truncate_deterministically_at_the_report_limit() {
    let first = run_json_mode("credential-literal-finding-limit");
    let second = run_json_mode("credential-literal-finding-limit");
    let (stdout, stderr) = text(&first);
    let report = json_report(&first);

    assert_eq!(first.status.code(), Some(1), "{report:#}\n{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(
        first.stdout, second.stdout,
        "security truncation must be stable"
    );
    let security = find_json_check(&report, "schema.contracts")["findings"]
        .as_array()
        .expect("the schema check should contain bounded findings");
    assert_eq!(security.len(), 255, "{report:#}");
    assert!(
        security
            .iter()
            .all(|finding| finding["code"] == "MCP-SECURITY-001"),
        "{report:#}"
    );
    assert_eq!(
        security.first().expect("a first security finding")["location"],
        "tools[0].inputSchema.properties[0].default"
    );
    assert_eq!(
        security.last().expect("a last security finding")["location"],
        "tools[254].inputSchema.properties[0].default"
    );
    let catalog = find_json_check(&report, "discovery.catalogs")["findings"]
        .as_array()
        .expect("the catalog check should contain the report bound");
    assert_eq!(catalog.len(), 1, "{report:#}");
    assert_eq!(catalog[0]["code"], "MCP-LIMIT-001");
    assert_eq!(catalog[0]["evidence"]["limit"], "report_findings");
    assert_eq!(catalog[0]["evidence"]["maximum"], 256);
    assert!(!stdout.contains("synthetic-security-limit-value-never-report-63"));
    assert!(!stdout.contains("password_"));
}

#[test]
fn required_input_descriptions_are_ordinal_redacted_and_consistent_across_reporters_and_revisions()
{
    const REMEDIATION: &str =
        "Describe the accepted value and any important constraints for this required input.";
    let human_output = run_mode("required-input-descriptions");
    let json_output = run_json_mode("required-input-descriptions");
    let environment = TestEnvironment::new();
    let junit_output = junit_inspect_command(&environment, "required-input-descriptions")
        .output()
        .expect("mcp-doctor should project required-input warnings as JUnit");
    let (human, human_stderr) = text(&human_output);
    let (json_text, json_stderr) = text(&json_output);
    let (_, junit_stderr) = text(&junit_output);
    let report = json_report(&json_output);
    let (junit, junit_summary) = parse_and_validate_junit(&junit_output.stdout);

    assert!(human_output.status.success(), "{human}\n{human_stderr}");
    assert!(json_output.status.success(), "{report:#}\n{json_stderr}");
    assert!(junit_output.status.success(), "{junit}\n{junit_stderr}");
    assert!(human_stderr.is_empty());
    assert!(json_stderr.is_empty());
    assert!(junit_stderr.is_empty());
    assert_eq!(report["outcome"], "passed");
    assert_eq!(report["exit_code"], 0);
    assert_eq!(report["primary_diagnosis"], serde_json::Value::Null);
    assert_eq!(report["summary"]["warned"], 1);
    assert_eq!(junit_summary.failures, 0);
    assert!(human.contains("WARN  schema.contracts"), "{human}");

    let findings = find_json_check(&report, "schema.contracts")["findings"]
        .as_array()
        .expect("the schema check should contain required-input findings");
    assert_eq!(findings.len(), 4, "{report:#}");
    for (index, finding) in findings.iter().enumerate() {
        let location = format!("tools[0].inputSchema.properties[{index}].description");
        assert_eq!(finding["code"], "MCP-QUALITY-002");
        assert_eq!(finding["severity"], "warning");
        assert_eq!(finding["protocol_revision"], "2026-07-28");
        assert_eq!(finding["location"], location);
        assert_eq!(
            finding["message"],
            "A required advertised tool input has no usable description."
        );
        assert_eq!(finding["remediation"], REMEDIATION);
        assert_eq!(finding["evidence"]["kind"], "none");
        assert!(human.contains("MCP-QUALITY-002"), "{human}");
        assert!(human.contains(&location), "{human}");
        assert!(human.contains(REMEDIATION), "{human}");
        assert!(
            junit.contains(&format!("finding[{index}].code=MCP-QUALITY-002")),
            "{junit}"
        );
        assert!(
            junit.contains(&format!("finding[{index}].location={location}")),
            "{junit}"
        );
        assert!(
            junit.contains(&format!("finding[{index}].remediation={REMEDIATION}")),
            "{junit}"
        );
    }
    for omitted_index in [4, 5] {
        let location = format!("tools[0].inputSchema.properties[{omitted_index}].description");
        assert!(!human.contains(&location), "{human}");
        assert!(!json_text.contains(&location), "{json_text}");
        assert!(!junit.contains(&location), "{junit}");
    }

    let forbidden = [
        "synthetic-required-input-private-value-never-report",
        "synthetic-required-input-private-tool-never-report",
        "argument_a_absent_never_report",
        "argument_b_empty_never_report",
        "argument_c_blank_never_report",
        "argument_d_reference_never_report",
        "argument_e_described_never_report",
        "argument_f_optional_never_report",
        "Referenced",
        "Accepts one bounded",
    ];
    for output in [human, json_text, junit.as_str()] {
        for value in forbidden {
            assert!(!output.contains(value), "report retained {value}: {output}");
        }
    }
    assert_report_findings_are_actionable(&report, human);

    let modern_projection = findings
        .iter()
        .map(|finding| {
            serde_json::json!({
                "code": finding["code"],
                "severity": finding["severity"],
                "location": finding["location"],
                "message": finding["message"],
                "remediation": finding["remediation"],
                "evidence": finding["evidence"]
            })
        })
        .collect::<Vec<_>>();
    for revision in ["2025-11-25", "2025-06-18"] {
        let environment = TestEnvironment::new();
        let output = legacy_inspect_command(
            &environment,
            revision,
            Some("json"),
            "legacy-required-input-descriptions",
        )
        .output()
        .expect("mcp-doctor should reuse required-input semantics for legacy revisions");
        let (legacy_json, legacy_stderr) = text(&output);
        let legacy = json_report(&output);
        assert!(
            output.status.success(),
            "{revision}: {legacy:#}\n{legacy_stderr}"
        );
        assert!(legacy_stderr.is_empty());
        assert_eq!(legacy["outcome"], "passed");
        let legacy_findings = find_json_check(&legacy, "schema.contracts")["findings"]
            .as_array()
            .expect("the legacy schema check should contain required-input findings");
        let legacy_projection = legacy_findings
            .iter()
            .map(|finding| {
                assert_eq!(finding["protocol_revision"], revision);
                serde_json::json!({
                    "code": finding["code"],
                    "severity": finding["severity"],
                    "location": finding["location"],
                    "message": finding["message"],
                    "remediation": finding["remediation"],
                    "evidence": finding["evidence"]
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(legacy_projection, modern_projection, "{legacy:#}");
        for value in forbidden {
            assert!(!legacy_json.contains(value), "{revision} retained {value}");
        }
    }
}

#[test]
fn required_input_quality_waits_for_each_schema_prerequisite() {
    let output = run_json_mode("required-input-description-prerequisites");
    let (stdout, stderr) = text(&output);
    let report = json_report(&output);

    assert_eq!(output.status.code(), Some(1), "{report:#}\n{stderr}");
    assert!(stderr.is_empty());
    let findings = find_json_check(&report, "schema.contracts")["findings"]
        .as_array()
        .expect("the schema check should retain prerequisite and eligible quality findings");
    assert_eq!(findings.len(), 3, "{report:#}");
    let invalid = findings
        .iter()
        .find(|finding| finding["code"] == "MCP-SCHEMA-001")
        .expect("the invalid schema should retain its prerequisite finding");
    assert!(
        invalid["location"]
            .as_str()
            .is_some_and(|location| location.starts_with("tools[0].inputSchema.required")),
        "{report:#}"
    );
    let external = findings
        .iter()
        .find(|finding| finding["code"] == "MCP-SCHEMA-003")
        .expect("the external schema should retain its prerequisite finding");
    assert_eq!(
        external["location"],
        "tools[1].inputSchema.properties[*].$ref"
    );
    let quality = findings
        .iter()
        .find(|finding| finding["code"] == "MCP-QUALITY-002")
        .expect("the valid sibling schema should receive the quality finding");
    assert_eq!(
        quality["location"],
        "tools[2].inputSchema.properties[0].description"
    );
    assert_eq!(report["primary_diagnosis"]["check_id"], "schema.contracts");
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-SCHEMA-001"
    );
    assert!(!stdout.contains("tools[0].inputSchema.properties[0].description"));
    assert!(!stdout.contains("tools[1].inputSchema.properties[0].description"));
    for (canary_index, forbidden) in [
        "synthetic-invalid-required-input-never-report",
        "synthetic-external-required-input-never-report",
        "synthetic-valid-required-input-never-report",
        "argument_invalid_never_report",
        "argument_external_never_report",
        "argument_valid_never_report",
        "invalid.example",
    ]
    .into_iter()
    .enumerate()
    {
        assert!(
            !stdout.contains(forbidden),
            "report retained redaction canary {canary_index}"
        );
    }
}

#[test]
fn required_input_quality_truncates_deterministically_at_the_report_limit() {
    let first = run_json_mode("required-input-description-finding-limit");
    let second = run_json_mode("required-input-description-finding-limit");
    let (stdout, stderr) = text(&first);
    let report = json_report(&first);

    assert_eq!(first.status.code(), Some(1), "{report:#}\n{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(
        first.stdout, second.stdout,
        "quality truncation must be stable"
    );
    let findings = find_json_check(&report, "schema.contracts")["findings"]
        .as_array()
        .expect("the schema check should contain bounded quality findings");
    assert!(!findings.is_empty(), "{report:#}");
    assert!(
        findings
            .iter()
            .all(|finding| finding["code"] == "MCP-QUALITY-002"),
        "{report:#}"
    );
    assert_eq!(findings.len(), 254, "{report:#}");
    assert_eq!(
        findings.first().expect("a first quality finding")["location"],
        "tools[0].inputSchema.properties[0].description"
    );
    assert_eq!(
        findings.last().expect("a last quality finding")["location"],
        "tools[253].inputSchema.properties[0].description"
    );
    let catalog = find_json_check(&report, "discovery.catalogs")["findings"]
        .as_array()
        .expect("the catalog check should retain the report bound");
    assert_eq!(catalog.len(), 1, "{report:#}");
    assert_eq!(catalog[0]["code"], "MCP-LIMIT-001");
    assert_eq!(catalog[0]["evidence"]["limit"], "report_findings");
    assert_eq!(catalog[0]["evidence"]["maximum"], 256);
    assert!(!stdout.contains("synthetic_required_input_"));
    assert!(!stdout.contains("synthetic-required-input-tool-"));
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
    let human_environment = TestEnvironment::new();
    let json_environment = TestEnvironment::new();
    let human_output = current_inspect_command(&human_environment, None, "protocol-unsupported")
        .output()
        .expect("mcp-doctor should render the exact-current diagnosis");
    let json_output =
        current_inspect_command(&json_environment, Some("json"), "protocol-unsupported")
            .output()
            .expect("mcp-doctor should render the exact-current JSON diagnosis");
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
fn unsupported_backtracking_patterns_receive_a_typed_local_diagnostic() {
    let output = run_mode("schema-unsupported-pattern");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-SCHEMA-001"), "{stdout}");
    assert!(stdout.contains("unsupported_linear_pattern"), "{stdout}");
    assert!(
        stdout.contains("tools[0].inputSchema.properties[*].pattern"),
        "{stdout}"
    );
    assert!(!stdout.contains("synthetic-private-property"), "{stdout}");
    assert!(!stdout.contains("?!private"), "{stdout}");
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
fn representative_schema_work_exhaustion_is_typed_incomplete_across_stdio_artifacts() {
    for case_id in schema_gate_corpus::CASES {
        let schema = schema_gate_corpus::schema(case_id)
            .expect("every fixed representative case should have a schema");
        let bytes = serde_json::to_vec(&schema)
            .expect("the fixed representative schema should serialize")
            .len();
        let (nodes, depth) = structural_metrics(&schema);
        assert!((1_536..=2_048).contains(&bytes), "{case_id}: {bytes}");
        assert!(nodes <= 160, "{case_id}: {nodes}");
        assert!(depth <= 8, "{case_id}: {depth}");

        let environment = TestEnvironment::new();
        let json_path = environment.artifact_path("schema-incomplete.json");
        let junit_path = environment.artifact_path("schema-incomplete.xml");
        let output = environment
            .command()
            .arg("inspect")
            .arg("--protocol-version")
            .arg("2025-11-25")
            .arg("--format")
            .arg("json")
            .arg("--json-report")
            .arg(&json_path)
            .arg("--junit-report")
            .arg(&junit_path)
            .arg("--")
            .arg(fixture())
            .arg("schema-gate")
            .arg(case_id)
            .output()
            .expect("the built CLI should inspect the fixed passive schema once");
        let (_, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(3), "{case_id}: {stderr}");
        assert!(stderr.is_empty(), "{case_id}: {stderr}");

        let report = json_report(&output);
        let artifact = parse_and_validate_report(
            &fs::read(&json_path).expect("the JSON artifact should be readable"),
        );
        assert_eq!(artifact, report);
        assert_eq!(report["outcome"], "incomplete");
        assert_eq!(report["exit_code"], 3);
        assert_eq!(report["summary"]["incomplete"], 1);
        assert_eq!(report["primary_diagnosis"]["check_id"], "schema.contracts");
        assert_eq!(
            report["primary_diagnosis"]["findings"][0]["code"],
            "MCP-SCHEMA-005"
        );
        let schema_check = find_json_check(&report, "schema.contracts");
        assert_eq!(schema_check["state"], "performed");
        assert_eq!(schema_check["outcome"], "incomplete");
        let finding = &schema_check["findings"][0];
        assert_eq!(finding["code"], "MCP-SCHEMA-005");
        assert_eq!(finding["location"], "tools[0].inputSchema");
        assert_eq!(finding["evidence"]["kind"], "schema_validation_limit");
        assert_eq!(finding["evidence"]["phase"], "compile_construction");
        assert_eq!(finding["evidence"]["limit"], "schema_evaluation_steps");
        assert_eq!(finding["evidence"]["unit"], "count");
        assert_eq!(finding["evidence"]["observed"], 100_001);
        assert_eq!(finding["evidence"]["maximum"], 100_000);
        let rendered = std::str::from_utf8(&output.stdout).expect("report should be UTF-8");
        assert!(!rendered.contains("Synthetic private schema sentinel never report 7f2c"));

        let (junit, summary) = parse_and_validate_junit(
            &fs::read(&junit_path).expect("the JUnit artifact should be readable"),
        );
        assert_eq!(summary.failures, 0);
        assert_eq!(
            summary.skipped,
            report["summary"]["skipped"].as_u64().unwrap() as usize + 1
        );
        assert!(junit.contains("<skipped message=\"incomplete\">"));
        assert!(junit.contains("finding[0].evidence.phase=compile_construction"));
        assert!(!junit.contains("Synthetic private schema sentinel never report 7f2c"));
    }
}

#[test]
fn schema_incomplete_preserves_phase_and_true_failure_precedence() {
    let meta = run_json_mode("schema-validator-work-limit");
    let meta_report = json_report(&meta);
    assert_eq!(meta.status.code(), Some(3));
    assert_eq!(meta_report["outcome"], "incomplete");
    let meta_finding = &find_json_check(&meta_report, "schema.contracts")["findings"][0];
    assert_eq!(meta_finding["code"], "MCP-SCHEMA-005");
    assert_eq!(meta_finding["evidence"]["phase"], "meta_validation");

    let preliminary = run_json_mode("schema-evaluation-limit");
    let preliminary_report = json_report(&preliminary);
    assert_eq!(preliminary.status.code(), Some(1));
    assert_eq!(preliminary_report["outcome"], "failed");
    assert_eq!(
        find_json_check(&preliminary_report, "schema.contracts")["findings"][0]["code"],
        "MCP-LIMIT-001"
    );

    let invalid = run_json_mode("schema-invalid");
    let invalid_report = json_report(&invalid);
    assert_eq!(invalid.status.code(), Some(1));
    assert_eq!(invalid_report["outcome"], "failed");
    assert!(
        find_json_check(&invalid_report, "schema.contracts")["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "MCP-SCHEMA-001")
    );

    let mixed = run_json_mode("schema-mixed-failure-incomplete");
    let mixed_report = json_report(&mixed);
    assert_eq!(mixed.status.code(), Some(1));
    assert_eq!(mixed_report["outcome"], "failed");
    assert_eq!(
        find_json_check(&mixed_report, "schema.contracts")["outcome"],
        "failed"
    );
    let mixed_codes = find_json_check(&mixed_report, "schema.contracts")["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(mixed_codes.contains(&"MCP-SCHEMA-001"));
    assert!(mixed_codes.contains(&"MCP-SCHEMA-005"));
    assert_eq!(
        mixed_report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-SCHEMA-001"
    );
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
        "{} failed · {} incomplete · {} warned · {} passed · {} skipped · outcome {} · exit {}",
        summary["failed"]
            .as_u64()
            .expect("failed should be a count"),
        summary["incomplete"]
            .as_u64()
            .expect("incomplete should be a count"),
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
