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
            "legacy",
            vec![
                "--protocol-version",
                "2025-11-25",
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
        assert!(stderr.contains("bounded current-revision snapshot"));
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
