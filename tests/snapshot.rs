#![cfg(feature = "internal-test-fixtures")]

mod support;

use std::fs;
use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Output;

use serde_json::{Value, json};
use support::{
    TestEnvironment, parse_and_validate_contract_diff, parse_and_validate_contract_snapshot,
    parse_and_validate_junit, parse_and_validate_report,
};

const EXCLUDED_SENTINEL: &str = "synthetic-snapshot-excluded-never-persist-36";
const BEFORE: &str = include_str!("fixtures/snapshots/before.json");
const AFTER: &str = include_str!("fixtures/snapshots/after.json");
const REORDERED: &str = include_str!("fixtures/snapshots/reordered-equivalent.json");
const CURRENT_CAPTURED: &[u8] = include_bytes!("fixtures/snapshots/current-captured.json");
const CHANGED_DIFF_HUMAN: &str = include_str!("fixtures/snapshots/changed-diff.txt");
const CHANGED_DIFF_JSON: &str = include_str!("fixtures/snapshots/changed-diff.json");

fn fixture() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_mcp-doctor-stdio-fixture"))
}

fn text(output: &Output) -> (&str, &str) {
    (
        std::str::from_utf8(&output.stdout).expect("STDOUT should be UTF-8"),
        std::str::from_utf8(&output.stderr).expect("STDERR should be UTF-8"),
    )
}

fn write_fixture(environment: &TestEnvironment, name: &str, contents: &str) -> PathBuf {
    let path = environment.artifact_path(name);
    fs::write(&path, contents).expect("the synthetic snapshot should be writable");
    path
}

fn run_diff(environment: &TestEnvironment, before: &Path, after: &Path, format: &str) -> Output {
    environment
        .command()
        .arg("diff")
        .arg("--format")
        .arg(format)
        .arg(before)
        .arg(after)
        .output()
        .expect("mcp-doctor should compare local snapshots")
}

fn capture_legacy_stdio_snapshot(
    environment: &TestEnvironment,
    revision: &str,
    name: &str,
) -> (Output, PathBuf, PathBuf) {
    let snapshot_path = environment.artifact_path(&format!("{name}.json"));
    let run_marker = environment.artifact_path(&format!("{name}.run"));
    let output = environment
        .command()
        .arg("inspect")
        .arg("--format")
        .arg("json")
        .arg("--protocol-version")
        .arg(revision)
        .arg("--snapshot")
        .arg(&snapshot_path)
        .arg("--allow-sensitive-snapshot")
        .arg(&snapshot_path)
        .arg("--")
        .arg(fixture())
        .arg("legacy-report-single-run")
        .arg(&run_marker)
        .output()
        .expect("mcp-doctor should create the selected legacy snapshot");
    (output, snapshot_path, run_marker)
}

fn snapshot_for_revision(source: &str, revision: &str, dialect: &str) -> Value {
    let mut snapshot: Value =
        serde_json::from_str(source).expect("snapshot fixture should be JSON");
    snapshot["protocol_revision"] = Value::String(revision.to_owned());
    snapshot["negotiated_protocol_revision"] = Value::String(revision.to_owned());
    for contract in snapshot["catalogs"]["tools"]["contracts"]
        .as_array_mut()
        .expect("tools should be an array")
    {
        contract["input_schema_dialect"] = Value::String(dialect.to_owned());
        if contract.get("output_schema").is_some() {
            contract["output_schema_dialect"] = Value::String(dialect.to_owned());
        }
    }
    snapshot
}

fn declare_tool_schema_dialect(snapshot: &mut Value, declaration: &str) {
    for contract in snapshot["catalogs"]["tools"]["contracts"]
        .as_array_mut()
        .expect("tools should be an array")
    {
        contract["input_schema"]
            .as_object_mut()
            .expect("input schema should be an object")
            .insert("$schema".to_owned(), json!(declaration));
        if let Some(output) = contract.get_mut("output_schema") {
            output
                .as_object_mut()
                .expect("output schema should be an object")
                .insert("$schema".to_owned(), json!(declaration));
        }
    }
}

fn diff_codes_for_catalog<'a>(report: &'a Value, catalog: &str) -> Vec<&'a str> {
    report["findings"]
        .as_array()
        .expect("findings should be an array")
        .iter()
        .filter(|finding| finding["catalog"] == catalog)
        .map(|finding| finding["code"].as_str().expect("code should be a string"))
        .collect()
}

#[test]
fn current_stdio_snapshot_is_sensitive_bounded_and_from_the_report_conversation() {
    let environment = TestEnvironment::new();
    let snapshot_path = environment.artifact_path("contract.json");
    let output = environment
        .command()
        .arg("inspect")
        .arg("--format")
        .arg("json")
        .arg("--snapshot")
        .arg(&snapshot_path)
        .arg("--allow-sensitive-snapshot")
        .arg(&snapshot_path)
        .arg("--")
        .arg(fixture())
        .arg("catalog-valid")
        .output()
        .expect("mcp-doctor should create the acknowledged snapshot");
    let (stdout, stderr) = text(&output);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["outcome"], "passed");
    assert!(!stdout.contains("synthetic.complex"));
    assert!(!stdout.contains("synthetic://guide"));

    let snapshot_bytes = fs::read(&snapshot_path).expect("the snapshot should exist");
    assert_eq!(
        snapshot_bytes, CURRENT_CAPTURED,
        "the existing current-revision artifact bytes changed"
    );
    let snapshot = parse_and_validate_contract_snapshot(&snapshot_bytes);
    assert_eq!(snapshot["protocol_revision"], "2026-07-28");
    assert_eq!(
        snapshot["catalogs"]["tools"]["contracts"][0]["name"],
        "synthetic.complex"
    );
    assert_eq!(
        snapshot["catalogs"]["resources"]["contracts"][0]["uri"],
        "synthetic://guide"
    );
    let snapshot_text = std::str::from_utf8(&snapshot_bytes).expect("snapshot should be UTF-8");
    assert!(!snapshot_text.contains("A synthetic complex schema fixture."));
    assert!(!snapshot_text.contains("nextCursor"));
    assert!(!snapshot_text.contains("ttlMs"));

    #[cfg(unix)]
    assert_eq!(
        fs::metadata(&snapshot_path)
            .expect("snapshot metadata should exist")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );

    let equivalent = run_diff(&environment, &snapshot_path, &snapshot_path, "json");
    let (stdout, stderr) = text(&equivalent);
    assert!(equivalent.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let diff = parse_and_validate_contract_diff(&equivalent.stdout);
    assert_eq!(diff["outcome"], "unchanged");
    assert_eq!(diff["summary"]["total"], 0);
    assert!(!stdout.contains("synthetic.complex"));
    assert!(!stdout.contains("synthetic://guide"));
}

#[test]
fn legacy_stdio_snapshots_are_revision_correct_passive_and_same_conversation() {
    for (revision, dialect) in [("2025-11-25", "draft_2020_12"), ("2025-06-18", "ambiguous")] {
        let environment = TestEnvironment::new();
        let (output, snapshot_path, run_marker) =
            capture_legacy_stdio_snapshot(&environment, revision, "legacy-contract");
        let (stdout, stderr) = text(&output);
        assert!(output.status.success(), "{revision}: {stdout}\n{stderr}");
        assert!(stderr.is_empty(), "{revision}: {stderr}");
        assert!(
            run_marker.exists(),
            "the fixture should record one target run"
        );
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(report["protocol_revision"], revision);
        assert_eq!(report["negotiated_protocol_revision"], revision);
        assert_eq!(report["outcome"], "passed");

        let bytes = fs::read(&snapshot_path).expect("the legacy snapshot should exist");
        let snapshot = parse_and_validate_contract_snapshot(&bytes);
        assert_eq!(snapshot["protocol_revision"], revision);
        assert_eq!(snapshot["negotiated_protocol_revision"], revision);
        assert_eq!(
            snapshot["catalogs"]["tools"]["contracts"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );
        let tool = &snapshot["catalogs"]["tools"]["contracts"][0];
        assert_eq!(tool["name"], "synthetic.passive");
        assert_eq!(tool["input_schema_dialect"], dialect);
        assert_eq!(tool["output_schema_dialect"], dialect);
        if revision == "2025-11-25" {
            assert_eq!(snapshot["capabilities"]["logging"]["advertised"], true);
            assert_eq!(snapshot["capabilities"]["tasks"]["advertised"], true);
            assert_eq!(snapshot["capabilities"]["tasks"]["list"], true);
            assert_eq!(snapshot["capabilities"]["tasks"]["cancel"], true);
            assert_eq!(
                snapshot["capabilities"]["tasks"]["requests_tools_call"],
                true
            );
        } else {
            assert!(snapshot["capabilities"].get("logging").is_none());
            assert!(snapshot["capabilities"].get("tasks").is_none());
        }

        let artifact = std::str::from_utf8(&bytes).expect("snapshot should be UTF-8");
        for excluded in [
            "synthetic-legacy",
            "synthetic instructions never rendered",
            "synthetic-private-legacy-cursor-never-report-7f2c",
            "experimental",
            "synthetic-private-legacy-stderr-never-report-7f2c",
            "synthetic-private-legacy-log-never-report-7f2c",
            "synthetic-private-completion-never-report-7f2c",
            "synthetic-private-task-never-report-7f2c",
        ] {
            assert!(!artifact.contains(excluded), "snapshot exposed {excluded}");
            assert!(!stdout.contains(excluded), "report exposed {excluded}");
        }

        let equivalent = run_diff(&environment, &snapshot_path, &snapshot_path, "json");
        let (diff_stdout, diff_stderr) = text(&equivalent);
        assert!(
            equivalent.status.success(),
            "{revision}: {diff_stdout}\n{diff_stderr}"
        );
        assert!(diff_stderr.is_empty());
        let diff = parse_and_validate_contract_diff(&equivalent.stdout);
        assert_eq!(diff["protocol_revision"], revision);
        assert_eq!(diff["outcome"], "unchanged");
        assert_eq!(diff["summary"]["total"], 0);
        assert!(!diff_stdout.contains("synthetic.passive"));
    }
}

#[test]
fn legacy_snapshot_is_not_published_when_cleanup_fails() {
    for revision in ["2025-11-25", "2025-06-18"] {
        let environment = TestEnvironment::new();
        let snapshot_path = environment.artifact_path(&format!("cleanup-{revision}.json"));
        let run_marker = environment.artifact_path(&format!("cleanup-{revision}.run"));
        let output = environment
            .command()
            .env("MCP_DOCTOR_INTERNAL_TEST_CLEANUP_FAILURE", "1")
            .arg("inspect")
            .arg("--protocol-version")
            .arg(revision)
            .arg("--snapshot")
            .arg(&snapshot_path)
            .arg("--allow-sensitive-snapshot")
            .arg(&snapshot_path)
            .arg("--")
            .arg(fixture())
            .arg("legacy-report-single-run")
            .arg(&run_marker)
            .output()
            .expect("the synthetic cleanup failure should return");
        let (stdout, stderr) = text(&output);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{revision}: {stdout}\n{stderr}"
        );
        assert!(stderr.is_empty());
        assert!(stdout.contains("MCP-SAFETY-001"), "{stdout}");
        assert!(run_marker.exists());
        assert!(!snapshot_path.exists());
    }
}

#[test]
fn legacy_revision_mismatch_writes_no_snapshot_and_reflects_no_identity() {
    for revision in ["2025-11-25", "2025-06-18"] {
        let environment = TestEnvironment::new();
        let snapshot_path = environment.artifact_path(&format!("mismatch-{revision}.json"));
        let output = environment
            .command()
            .arg("inspect")
            .arg("--protocol-version")
            .arg(revision)
            .arg("--snapshot")
            .arg(&snapshot_path)
            .arg("--allow-sensitive-snapshot")
            .arg(&snapshot_path)
            .arg("--")
            .arg(fixture())
            .arg("legacy-mismatch")
            .output()
            .expect("the selected/negotiated mismatch should return");
        let (stdout, stderr) = text(&output);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{revision}: {stdout}\n{stderr}"
        );
        assert!(stdout.is_empty());
        assert!(stderr.contains("bounded revision-correct snapshot"));
        assert!(!snapshot_path.exists());
        for excluded in ["2025-11-25", "2025-06-18", "synthetic"] {
            assert!(!stderr.contains(excluded), "error reflected {excluded}");
        }
    }
}

#[test]
fn legacy_capture_rejects_external_and_over_depth_schemas_without_artifacts() {
    for revision in ["2025-11-25", "2025-06-18"] {
        for (mode, excluded) in [
            (
                "legacy-schema-external",
                "https://synthetic.invalid/legacy-private-schema-never-report-7f2c",
            ),
            ("legacy-schema-depth-limit", "synthetic.legacy-bounded"),
        ] {
            let environment = TestEnvironment::new();
            let snapshot_path = environment.artifact_path(&format!("{revision}-{mode}.json"));
            let output = environment
                .command()
                .arg("inspect")
                .arg("--protocol-version")
                .arg(revision)
                .arg("--snapshot")
                .arg(&snapshot_path)
                .arg("--allow-sensitive-snapshot")
                .arg(&snapshot_path)
                .arg("--")
                .arg(fixture())
                .arg(mode)
                .output()
                .expect("the unsafe legacy contract should fail closed");
            let (stdout, stderr) = text(&output);
            assert_eq!(
                output.status.code(),
                Some(2),
                "{revision}/{mode}: {stdout}\n{stderr}"
            );
            assert!(stdout.is_empty());
            assert!(stderr.contains("bounded revision-correct snapshot"));
            assert!(!stderr.contains(excluded));
            assert!(!snapshot_path.exists());
        }
    }
}

#[test]
fn malformed_legacy_capability_writes_no_snapshot_and_echoes_no_value() {
    const CAPABILITY_SENTINEL: &str = "synthetic-private-malformed-capability-never-report-7f2c";
    for revision in ["2025-11-25", "2025-06-18"] {
        let environment = TestEnvironment::new();
        let snapshot_path = environment.artifact_path(&format!("capability-{revision}.json"));
        let output = environment
            .command()
            .arg("inspect")
            .arg("--protocol-version")
            .arg(revision)
            .arg("--snapshot")
            .arg(&snapshot_path)
            .arg("--allow-sensitive-snapshot")
            .arg(&snapshot_path)
            .arg("--")
            .arg(fixture())
            .arg("legacy-malformed-capability")
            .output()
            .expect("the malformed legacy capability should fail closed");
        let (stdout, stderr) = text(&output);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{revision}: {stdout}\n{stderr}"
        );
        assert!(stdout.is_empty());
        assert!(stderr.contains("bounded revision-correct snapshot"));
        assert!(!stderr.contains(CAPABILITY_SENTINEL));
        assert!(!snapshot_path.exists());
    }
}

#[test]
fn invalid_schema_ordinal_resolves_only_through_the_same_run_snapshot() {
    let environment = TestEnvironment::new();
    let snapshot_path = environment.artifact_path("correlated.json");
    let output = environment
        .command()
        .arg("inspect")
        .arg("--format")
        .arg("json")
        .arg("--snapshot")
        .arg(&snapshot_path)
        .arg("--allow-sensitive-snapshot")
        .arg(&snapshot_path)
        .arg("--")
        .arg(fixture())
        .arg("snapshot-correlation")
        .output()
        .expect("mcp-doctor should retain a bounded invalid local schema contract");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["primary_diagnosis"]["check_id"], "schema.contracts");
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["location"],
        "tools[73].inputSchema.required"
    );
    assert!(!stdout.contains("synthetic.tool.026"));
    assert!(!stdout.contains(EXCLUDED_SENTINEL));

    let snapshot_bytes = fs::read(&snapshot_path).expect("the correlated snapshot should exist");
    let snapshot = parse_and_validate_contract_snapshot(&snapshot_bytes);
    assert_eq!(
        snapshot["catalogs"]["tools"]["contracts"]
            .as_array()
            .map(Vec::len),
        Some(100)
    );
    let correlation = snapshot["catalogs"]["tools"]["correlation"]
        .as_array()
        .expect("correlation should be an array")
        .iter()
        .find(|entry| entry["discovery_ordinal"] == 73)
        .expect("ordinal 73 should be mapped");
    assert_eq!(correlation["contract_index"], 26);
    assert_eq!(
        snapshot["catalogs"]["tools"]["contracts"][26]["name"],
        "synthetic.tool.026"
    );
    assert!(
        !std::str::from_utf8(&snapshot_bytes)
            .expect("snapshot should be UTF-8")
            .contains(EXCLUDED_SENTINEL)
    );

    for format in [None, Some("junit")] {
        let mut command = environment.command();
        command.arg("inspect");
        if let Some(format) = format {
            command.arg("--format").arg(format);
        }
        let ordinary = command
            .arg("--")
            .arg(fixture())
            .arg("snapshot-correlation")
            .output()
            .expect("ordinary report should remain identifier-free");
        assert_eq!(ordinary.status.code(), Some(1));
        let (ordinary_stdout, ordinary_stderr) = text(&ordinary);
        assert!(ordinary_stderr.is_empty());
        assert!(!ordinary_stdout.contains("synthetic.tool.026"));
        assert!(!ordinary_stdout.contains(EXCLUDED_SENTINEL));
        if format == Some("junit") {
            parse_and_validate_junit(&ordinary.stdout);
        }
    }
}

#[test]
fn bounded_malformed_local_schema_shape_still_produces_the_requested_snapshot() {
    let environment = TestEnvironment::new();
    let snapshot_path = environment.artifact_path("invalid-shape.json");
    let output = environment
        .command()
        .arg("inspect")
        .arg("--format")
        .arg("json")
        .arg("--snapshot")
        .arg(&snapshot_path)
        .arg("--allow-sensitive-snapshot")
        .arg(&snapshot_path)
        .arg("--")
        .arg(fixture())
        .arg("snapshot-invalid-shape")
        .output()
        .expect("a bounded malformed local shape should remain available for correlation");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(report["primary_diagnosis"]["check_id"], "schema.contracts");
    assert!(!stdout.contains("synthetic.invalid-shape"));

    let bytes = fs::read(&snapshot_path).expect("the malformed contract snapshot should exist");
    let snapshot = parse_and_validate_contract_snapshot(&bytes);
    assert_eq!(
        snapshot["catalogs"]["tools"]["contracts"][0]["input_schema"],
        7
    );
    assert!(
        !std::str::from_utf8(&bytes)
            .expect("snapshot should be UTF-8")
            .contains("synthetic-invalid-shape-description-never-persist-36")
    );
}

#[test]
fn every_snapshot_gate_fails_before_the_target_starts_and_existing_output_is_unchanged() {
    let environment = TestEnvironment::new();
    let output_path = environment.artifact_path("existing.json");
    let other_path = environment.artifact_path("other.json");
    fs::write(&output_path, b"existing synthetic bytes")
        .expect("the existing output should be writable");

    for (name, options) in [
        (
            "missing-ack",
            vec!["--snapshot", output_path.to_str().expect("UTF-8 path")],
        ),
        (
            "mismatched-ack",
            vec![
                "--snapshot",
                output_path.to_str().expect("UTF-8 path"),
                "--allow-sensitive-snapshot",
                other_path.to_str().expect("UTF-8 path"),
            ],
        ),
        (
            "unsupported-revision",
            vec![
                "--protocol-version",
                "2025-03-26",
                "--snapshot",
                other_path.to_str().expect("UTF-8 path"),
                "--allow-sensitive-snapshot",
                other_path.to_str().expect("UTF-8 path"),
            ],
        ),
        (
            "existing",
            vec![
                "--snapshot",
                output_path.to_str().expect("UTF-8 path"),
                "--allow-sensitive-snapshot",
                output_path.to_str().expect("UTF-8 path"),
            ],
        ),
        (
            "legacy-existing",
            vec![
                "--protocol-version",
                "2025-11-25",
                "--snapshot",
                output_path.to_str().expect("UTF-8 path"),
                "--allow-sensitive-snapshot",
                output_path.to_str().expect("UTF-8 path"),
            ],
        ),
    ] {
        let marker = environment.artifact_path(&format!("{name}.started"));
        let mut command = environment.command();
        command
            .arg("inspect")
            .args(options)
            .arg("--")
            .arg(fixture())
            .arg("snapshot-started-marker")
            .arg(&marker);
        let output = command.output().expect("the snapshot gate should return");
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(2), "{name}: {stdout}\n{stderr}");
        assert!(stdout.is_empty(), "{name}: {stdout}");
        assert!(!marker.exists(), "{name}: the target started");
    }
    assert_eq!(
        fs::read(&output_path).expect("the existing output should remain"),
        b"existing synthetic bytes"
    );
}

#[test]
fn prohibited_or_over_limit_contract_content_writes_no_snapshot_and_echoes_no_values() {
    let environment = TestEnvironment::new();
    let external = "https://synthetic.invalid/private-schema-never-report-36";
    for (name, mode, extra, excluded) in [
        ("external", "schema-external", Some(external), external),
        ("deep", "schema-depth-limit", None, EXCLUDED_SENTINEL),
        (
            "protocol",
            "protocol-unsupported",
            None,
            "synthetic.tool.026",
        ),
    ] {
        let snapshot_path = environment.artifact_path(&format!("{name}.json"));
        let mut command = environment.command();
        command
            .arg("inspect")
            .arg("--snapshot")
            .arg(&snapshot_path)
            .arg("--allow-sensitive-snapshot")
            .arg(&snapshot_path)
            .arg("--")
            .arg(fixture())
            .arg(mode);
        if let Some(extra) = extra {
            command.arg(extra);
        }
        let output = command
            .output()
            .expect("unsafe snapshot content should fail closed");
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(2), "{name}: {stdout}\n{stderr}");
        assert!(stdout.is_empty());
        assert!(stderr.contains("bounded revision-correct snapshot"));
        assert!(!stderr.contains(excluded));
        assert!(!snapshot_path.exists());
    }
}

#[test]
fn reordered_artifacts_are_equal_and_documented_changes_have_stable_value_free_codes() {
    let environment = TestEnvironment::new();
    let before = write_fixture(&environment, "before.json", BEFORE);
    let reordered = write_fixture(&environment, "reordered.json", REORDERED);
    let after = write_fixture(&environment, "after.json", AFTER);

    let equivalent = run_diff(&environment, &before, &reordered, "json");
    assert!(equivalent.status.success(), "{:?}", text(&equivalent));
    let equivalent_report = parse_and_validate_contract_diff(&equivalent.stdout);
    assert_eq!(equivalent_report["outcome"], "unchanged");

    let changed = run_diff(&environment, &before, &after, "json");
    let (stdout, stderr) = text(&changed);
    assert_eq!(changed.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_contract_diff(&changed.stdout);
    assert_eq!(stdout, CHANGED_DIFF_JSON);
    assert_eq!(report["outcome"], "potentially_breaking");
    let codes = report["findings"]
        .as_array()
        .expect("findings should be an array")
        .iter()
        .map(|finding| finding["code"].as_str().expect("code should be a string"))
        .collect::<Vec<_>>();
    for code in [
        "MCP-DIFF-001",
        "MCP-DIFF-002",
        "MCP-DIFF-003",
        "MCP-DIFF-005",
        "MCP-DIFF-007",
        "MCP-DIFF-009",
        "MCP-DIFF-010",
    ] {
        assert!(codes.contains(&code), "missing {code}: {stdout}");
    }
    for catalog in ["prompts", "resource_templates"] {
        for code in ["MCP-DIFF-001", "MCP-DIFF-002"] {
            assert!(
                report["findings"]
                    .as_array()
                    .is_some_and(|findings| findings
                        .iter()
                        .any(|finding| finding["code"] == code && finding["catalog"] == catalog)),
                "missing {code} for {catalog}: {stdout}"
            );
        }
    }
    for sensitive in ["synthetic.alpha", "synthetic://old", "query", "limit"] {
        assert!(
            !stdout.contains(sensitive),
            "diff exposed {sensitive}: {stdout}"
        );
    }

    let reverse = run_diff(&environment, &after, &before, "json");
    assert_eq!(reverse.status.code(), Some(1));
    let reverse = parse_and_validate_contract_diff(&reverse.stdout);
    let reverse_codes = reverse["findings"]
        .as_array()
        .expect("findings should be an array")
        .iter()
        .map(|finding| finding["code"].as_str().expect("code should be a string"))
        .collect::<Vec<_>>();
    for code in ["MCP-DIFF-004", "MCP-DIFF-006", "MCP-DIFF-008"] {
        assert!(reverse_codes.contains(&code), "missing reverse {code}");
    }

    let human_one = run_diff(&environment, &before, &after, "human");
    let human_two = run_diff(&environment, &before, &after, "human");
    assert_eq!(human_one.status.code(), Some(1));
    assert_eq!(
        std::str::from_utf8(&human_one.stdout).expect("human diff should be UTF-8"),
        CHANGED_DIFF_HUMAN
    );
    assert_eq!(human_one.stdout, human_two.stdout);
    assert!(human_one.stderr.is_empty());
    assert!(human_two.stderr.is_empty());
}

#[test]
fn legacy_diff_uses_only_revision_defined_schema_semantics() {
    for (revision, dialect, infer_draft) in [
        ("2025-11-25", "draft_2020_12", true),
        ("2025-06-18", "ambiguous", false),
    ] {
        let environment = TestEnvironment::new();
        let before = snapshot_for_revision(BEFORE, revision, dialect);
        let after = snapshot_for_revision(AFTER, revision, dialect);
        let reordered = snapshot_for_revision(REORDERED, revision, dialect);
        let before_path = write_value(&environment, "legacy-before.json", &before);
        let after_path = write_value(&environment, "legacy-after.json", &after);
        let reordered_path = write_value(&environment, "legacy-reordered.json", &reordered);
        let equivalent = run_diff(&environment, &before_path, &reordered_path, "json");
        let (equivalent_stdout, equivalent_stderr) = text(&equivalent);
        assert!(
            equivalent.status.success(),
            "{revision}: {equivalent_stdout}\n{equivalent_stderr}"
        );
        assert!(equivalent_stderr.is_empty());
        let equivalent_report = parse_and_validate_contract_diff(&equivalent.stdout);
        assert_eq!(equivalent_report["protocol_revision"], revision);
        assert_eq!(equivalent_report["outcome"], "unchanged");

        let output = run_diff(&environment, &before_path, &after_path, "json");
        let (stdout, stderr) = text(&output);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{revision}: {stdout}\n{stderr}"
        );
        assert!(stderr.is_empty());
        let report = parse_and_validate_contract_diff(&output.stdout);
        assert_eq!(report["protocol_revision"], revision);
        let tool_codes = diff_codes_for_catalog(&report, "tools");
        if infer_draft {
            assert!(tool_codes.contains(&"MCP-DIFF-005"), "{stdout}");
            assert!(tool_codes.contains(&"MCP-DIFF-007"), "{stdout}");
        } else {
            assert!(tool_codes.contains(&"MCP-DIFF-009"), "{stdout}");
            for prohibited in [
                "MCP-DIFF-005",
                "MCP-DIFF-006",
                "MCP-DIFF-007",
                "MCP-DIFF-008",
            ] {
                assert!(!tool_codes.contains(&prohibited), "{stdout}");
            }
        }
        for sensitive in ["synthetic.alpha", "query", "limit", "minLength"] {
            assert!(!stdout.contains(sensitive), "diff exposed {sensitive}");
        }
    }
}

#[test]
fn explicit_and_unsupported_legacy_dialects_compare_conservatively_without_uri_reflection() {
    const DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";
    const UNSUPPORTED: &str = "https://synthetic.invalid/private-dialect-never-report-36";
    for (name, declaration, dialect, infer_draft) in [
        ("explicit", DRAFT, "draft_2020_12", true),
        ("unsupported", UNSUPPORTED, "unsupported", false),
    ] {
        let environment = TestEnvironment::new();
        let mut before = snapshot_for_revision(BEFORE, "2025-06-18", dialect);
        let mut after = snapshot_for_revision(AFTER, "2025-06-18", dialect);
        declare_tool_schema_dialect(&mut before, declaration);
        declare_tool_schema_dialect(&mut after, declaration);
        let before_path = write_value(&environment, &format!("{name}-before.json"), &before);
        let after_path = write_value(&environment, &format!("{name}-after.json"), &after);
        let output = run_diff(&environment, &before_path, &after_path, "json");
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(1), "{name}: {stdout}\n{stderr}");
        assert!(stderr.is_empty());
        let report = parse_and_validate_contract_diff(&output.stdout);
        assert_eq!(report["protocol_revision"], "2025-06-18");
        let tool_codes = diff_codes_for_catalog(&report, "tools");
        assert_eq!(
            tool_codes.contains(&"MCP-DIFF-005"),
            infer_draft,
            "{stdout}"
        );
        assert_eq!(
            tool_codes.contains(&"MCP-DIFF-007"),
            infer_draft,
            "{stdout}"
        );
        assert!(tool_codes.contains(&"MCP-DIFF-009"), "{stdout}");
        assert!(
            !stdout.contains(declaration),
            "diff reflected a dialect URI"
        );
        assert!(!stdout.contains("synthetic.alpha"));
    }
}

#[test]
fn cross_revision_mismatch_and_incompatible_legacy_artifacts_fail_without_value_reflection() {
    let environment = TestEnvironment::new();
    let current: Value = serde_json::from_str(BEFORE).expect("current fixture should be JSON");
    let legacy_11 = snapshot_for_revision(BEFORE, "2025-11-25", "draft_2020_12");
    let legacy_06 = snapshot_for_revision(BEFORE, "2025-06-18", "ambiguous");
    let current_path = write_value(&environment, "cross-current.json", &current);
    let legacy_11_path = write_value(&environment, "cross-11.json", &legacy_11);
    let legacy_06_path = write_value(&environment, "cross-06.json", &legacy_06);

    for (before, after) in [
        (&current_path, &legacy_11_path),
        (&legacy_11_path, &legacy_06_path),
    ] {
        let output = run_diff(&environment, before, after, "json");
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
        assert!(stderr.is_empty());
        let report = parse_and_validate_contract_diff(&output.stdout);
        assert_eq!(report["outcome"], "invalid");
        assert!(report["protocol_revision"].is_null());
        assert_eq!(report["findings"][0]["code"], "MCP-SNAPSHOT-007");
        assert_eq!(report["checks"][1]["state"], "skipped");
        for excluded in ["2026-07-28", "2025-11-25", "2025-06-18", "synthetic.alpha"] {
            assert!(!stdout.contains(excluded), "diff reflected {excluded}");
        }
    }

    let mut identity_mismatch = legacy_11.clone();
    identity_mismatch["negotiated_protocol_revision"] = json!("2025-06-18");
    let identity_path = write_value(&environment, "identity-mismatch.json", &identity_mismatch);
    let output = run_diff(&environment, &identity_path, &legacy_11_path, "json");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_contract_diff(&output.stdout);
    assert_eq!(report["findings"][0]["code"], "MCP-SNAPSHOT-007");
    assert!(report["protocol_revision"].is_null());
    for excluded in ["2025-11-25", "2025-06-18", "synthetic.alpha"] {
        assert!(!stdout.contains(excluded), "diff reflected {excluded}");
    }

    let mut unexpected_current_identity = current.clone();
    unexpected_current_identity["negotiated_protocol_revision"] = Value::Null;
    let unexpected_current_path = write_value(
        &environment,
        "unexpected-current-identity.json",
        &unexpected_current_identity,
    );
    let output = run_diff(
        &environment,
        &unexpected_current_path,
        &current_path,
        "json",
    );
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_contract_diff(&output.stdout);
    assert_eq!(report["findings"][0]["code"], "MCP-SNAPSHOT-007");
    assert!(report["protocol_revision"].is_null());
    for excluded in ["2026-07-28", "2025-11-25", "synthetic.alpha"] {
        assert!(!stdout.contains(excluded), "diff reflected {excluded}");
    }

    let mut null_revision_field = current;
    null_revision_field["catalogs"]["tools"]["contracts"][0]["input_schema_dialect"] = Value::Null;
    let null_field_path = write_value(
        &environment,
        "null-revision-field.json",
        &null_revision_field,
    );
    let output = run_diff(&environment, &null_field_path, &current_path, "json");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_contract_diff(&output.stdout);
    assert_eq!(report["findings"][0]["code"], "MCP-SNAPSHOT-008");
    assert!(report["protocol_revision"].is_null());
    assert!(!stdout.contains("synthetic.alpha"));

    let mut incompatible = legacy_06;
    incompatible["catalogs"]["tools"]["contracts"][0]["input_schema_dialect"] =
        json!("draft_2020_12");
    let incompatible_path = write_value(&environment, "incompatible.json", &incompatible);
    let output = run_diff(&environment, &incompatible_path, &legacy_06_path, "json");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_contract_diff(&output.stdout);
    assert_eq!(report["findings"][0]["code"], "MCP-SNAPSHOT-008");
    assert!(report["protocol_revision"].is_null());
    for excluded in [
        "draft_2020_12",
        "ambiguous",
        "2025-06-18",
        "synthetic.alpha",
    ] {
        assert!(!stdout.contains(excluded), "diff reflected {excluded}");
    }
}

#[test]
fn malformed_limited_external_cross_version_and_correlation_inputs_fail_structurally() {
    let environment = TestEnvironment::new();
    let baseline = write_fixture(&environment, "baseline.json", BEFORE);
    let baseline_value: Value = serde_json::from_str(BEFORE).expect("baseline should be JSON");
    let mut cases = Vec::<(PathBuf, &'static str, &'static str)>::new();

    let malformed = environment.artifact_path("malformed.json");
    fs::write(&malformed, b"{not-json").expect("malformed fixture should be writable");
    cases.push((malformed, "MCP-SNAPSHOT-001", ""));

    let mut version = baseline_value.clone();
    version["schema_version"] = Value::String("synthetic.future/v9".to_owned());
    cases.push((
        write_value(&environment, "version.json", &version),
        "MCP-SNAPSHOT-002",
        "synthetic.future",
    ));

    let mut revision = baseline_value.clone();
    revision["protocol_revision"] = Value::String("2099-01-01".to_owned());
    cases.push((
        write_value(&environment, "revision.json", &revision),
        "MCP-SNAPSHOT-003",
        "2099-01-01",
    ));

    let oversized = environment.artifact_path("oversized.json");
    fs::write(&oversized, vec![b' '; 8_388_609]).expect("oversized fixture should be writable");
    cases.push((oversized, "MCP-SNAPSHOT-004", ""));

    let mut deep = baseline_value.clone();
    let mut nested = json!({"type": "string"});
    for _ in 0..65 {
        nested = json!({"not": nested});
    }
    deep["catalogs"]["tools"]["contracts"][0]["input_schema"] =
        json!({"type": "object", "properties": {"field": nested}});
    cases.push((
        write_value(&environment, "deep.json", &deep),
        "MCP-SNAPSHOT-004",
        "",
    ));

    let external_value = "https://synthetic.invalid/private-schema-never-report-36";
    let mut external = baseline_value.clone();
    external["catalogs"]["tools"]["contracts"][0]["input_schema"] =
        json!({"type": "object", "properties": {"field": {"$ref": external_value}}});
    cases.push((
        write_value(&environment, "external.json", &external),
        "MCP-SNAPSHOT-005",
        external_value,
    ));

    let mut correlation = baseline_value;
    correlation["catalogs"]["tools"]["correlation"][1]["discovery_ordinal"] = json!(0);
    cases.push((
        write_value(&environment, "correlation.json", &correlation),
        "MCP-SNAPSHOT-006",
        "synthetic.removed",
    ));

    let mut out_of_range: Value = serde_json::from_str(BEFORE).expect("baseline should be JSON");
    out_of_range["catalogs"]["tools"]["correlation"][1]["contract_index"] = json!(100);
    cases.push((
        write_value(&environment, "out-of-range.json", &out_of_range),
        "MCP-SNAPSHOT-006",
        "synthetic.removed",
    ));

    for (path, code, excluded) in cases {
        let output = run_diff(&environment, &path, &baseline, "json");
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(2), "{code}: {stdout}\n{stderr}");
        assert!(stderr.is_empty());
        let report = parse_and_validate_contract_diff(&output.stdout);
        assert_eq!(report["outcome"], "invalid");
        assert_eq!(report["findings"][0]["code"], code);
        assert_eq!(report["checks"][1]["state"], "skipped");
        assert_eq!(report["checks"][1]["blocked_by"], "artifact_validation");
        assert!(!stdout.contains(path.to_string_lossy().as_ref()));
        if !excluded.is_empty() {
            assert!(!stdout.contains(excluded), "{code} exposed input content");
        }
    }
}

#[test]
fn legacy_artifacts_retain_file_catalog_schema_and_reference_bounds() {
    let environment = TestEnvironment::new();
    let baseline = snapshot_for_revision(BEFORE, "2025-06-18", "ambiguous");
    let baseline_path = write_value(&environment, "legacy-bounded-baseline.json", &baseline);
    let mut cases = Vec::<(PathBuf, &'static str, &'static str)>::new();

    let oversized = environment.artifact_path("legacy-oversized.json");
    fs::write(&oversized, vec![b' '; 8_388_609])
        .expect("the oversized legacy artifact should be writable");
    cases.push((oversized, "MCP-SNAPSHOT-004", ""));

    let mut schema_bytes = baseline.clone();
    schema_bytes["catalogs"]["tools"]["contracts"][0]["input_schema"] = json!({
        "description": format!("{EXCLUDED_SENTINEL}{}", "x".repeat(1_048_577))
    });
    cases.push((
        write_value(&environment, "legacy-schema-bytes.json", &schema_bytes),
        "MCP-SNAPSHOT-004",
        EXCLUDED_SENTINEL,
    ));

    let mut schema_nodes = baseline.clone();
    schema_nodes["catalogs"]["tools"]["contracts"][0]["input_schema"] =
        Value::Array(vec![Value::Bool(false); 100_001]);
    cases.push((
        write_value(&environment, "legacy-schema-nodes.json", &schema_nodes),
        "MCP-SNAPSHOT-004",
        "",
    ));

    let mut schema_depth = baseline.clone();
    let mut nested = json!({"type": "string"});
    for _ in 0..65 {
        nested = json!({"not": nested});
    }
    schema_depth["catalogs"]["tools"]["contracts"][0]["input_schema"] = nested;
    cases.push((
        write_value(&environment, "legacy-schema-depth.json", &schema_depth),
        "MCP-SNAPSHOT-004",
        "",
    ));

    let mut schema_references = baseline.clone();
    let mut definitions = serde_json::Map::new();
    for index in 0..33 {
        let target = if index == 32 {
            json!({"type": "string"})
        } else {
            json!({"$ref": format!("#/$defs/node{}", index + 1)})
        };
        definitions.insert(format!("node{index}"), target);
    }
    schema_references["catalogs"]["tools"]["contracts"][0]["input_schema"] =
        json!({"$defs": definitions, "$ref": "#/$defs/node0"});
    cases.push((
        write_value(
            &environment,
            "legacy-schema-reference-depth.json",
            &schema_references,
        ),
        "MCP-SNAPSHOT-004",
        "",
    ));

    let external_reference = "https://synthetic.invalid/legacy-private-schema-36";
    let mut external = baseline.clone();
    external["catalogs"]["tools"]["contracts"][0]["input_schema"] =
        json!({"$ref": external_reference});
    cases.push((
        write_value(&environment, "legacy-external-reference.json", &external),
        "MCP-SNAPSHOT-005",
        external_reference,
    ));

    let mut catalog_items = baseline;
    catalog_items["catalogs"]["tools"]["contracts"] = Value::Array(
        (0..10_001)
            .map(|index| {
                json!({
                    "name": format!("synthetic.legacy.limit.{index:05}"),
                    "input_schema": {},
                    "input_schema_dialect": "ambiguous"
                })
            })
            .collect(),
    );
    catalog_items["catalogs"]["tools"]["correlation"] = Value::Array(
        (0..10_001)
            .map(|index| json!({"discovery_ordinal": index, "contract_index": index}))
            .collect(),
    );
    cases.push((
        write_value(&environment, "legacy-catalog-items.json", &catalog_items),
        "MCP-SNAPSHOT-004",
        "synthetic.legacy.limit",
    ));

    for (path, code, excluded) in cases {
        let output = run_diff(&environment, &path, &baseline_path, "json");
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(2), "{code}: {stdout}\n{stderr}");
        assert!(stderr.is_empty());
        let report = parse_and_validate_contract_diff(&output.stdout);
        assert_eq!(report["outcome"], "invalid");
        assert!(report["protocol_revision"].is_null());
        assert_eq!(report["findings"][0]["code"], code);
        assert_eq!(report["checks"][1]["state"], "skipped");
        if !excluded.is_empty() {
            assert!(
                !stdout.contains(excluded),
                "{code} reflected bounded content"
            );
        }
    }
}

fn write_value(environment: &TestEnvironment, name: &str, value: &Value) -> PathBuf {
    let path = environment.artifact_path(name);
    fs::write(
        &path,
        serde_json::to_vec_pretty(value).expect("synthetic JSON should serialize"),
    )
    .expect("synthetic JSON should be writable");
    path
}

#[test]
fn diff_cli_has_no_target_or_network_surface() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("a loopback trap should bind");
    listener
        .set_nonblocking(true)
        .expect("the loopback trap should be nonblocking");
    let endpoint = format!(
        "http://127.0.0.1:{}/mcp",
        listener.local_addr().expect("address").port()
    );
    let environment = TestEnvironment::new();
    let before = write_fixture(&environment, "offline-before.json", BEFORE);
    let after = write_fixture(&environment, "offline-after.json", REORDERED);
    let output = environment
        .command()
        .arg("diff")
        .arg(&before)
        .arg(&after)
        .arg("--endpoint")
        .arg(&endpoint)
        .output()
        .expect("clap should reject a network option");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        listener.accept().is_err(),
        "diff contacted the loopback trap"
    );
}

#[test]
fn diff_finding_collection_stops_at_the_finite_limit() {
    let environment = TestEnvironment::new();
    let mut before: Value = serde_json::from_str(BEFORE).expect("baseline should be JSON");
    before["capabilities"]["prompts"] = json!({});
    before["capabilities"]["resources"] = json!({});
    before["catalogs"]["tools"] = json!({"contracts": [], "correlation": []});
    before["catalogs"]["prompts"] = json!({"contracts": [], "correlation": []});
    before["catalogs"]["resources"] = json!({"contracts": [], "correlation": []});
    before["catalogs"]["resource_templates"] = json!({"contracts": [], "correlation": []});
    let mut after = before.clone();
    after["catalogs"]["tools"]["contracts"] = Value::Array(
        (0..257)
            .map(|index| {
                json!({
                    "name": format!("synthetic.limit.{index:03}"),
                    "input_schema": {"type": "object", "properties": {}}
                })
            })
            .collect(),
    );
    after["catalogs"]["tools"]["correlation"] = Value::Array(
        (0..257)
            .map(|index| json!({"discovery_ordinal": index, "contract_index": index}))
            .collect(),
    );
    let before_path = write_value(&environment, "limit-before.json", &before);
    let after_path = write_value(&environment, "limit-after.json", &after);
    let output = run_diff(&environment, &before_path, &after_path, "json");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_contract_diff(&output.stdout);
    assert_eq!(report["outcome"], "invalid");
    assert_eq!(report["findings"].as_array().map(Vec::len), Some(1));
    assert_eq!(report["findings"][0]["code"], "MCP-SNAPSHOT-004");
    assert_eq!(report["findings"][0]["change"], "comparison_limit");
    assert!(!stdout.contains("synthetic.limit"));
}
