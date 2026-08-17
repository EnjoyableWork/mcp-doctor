#![cfg(feature = "internal-test-fixtures")]

mod support;

use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::process::Output;

use serde_json::{Value, json};
use support::{
    TestEnvironment, parse_and_validate_contract_diff, parse_and_validate_report,
    run_with_bound_file_mutation,
};

const TOOL: &str = "synthetic.reviewed";
const BEFORE: &str = include_str!("fixtures/snapshots/before.json");
const AFTER: &str = include_str!("fixtures/snapshots/after.json");
const REPORT: &[u8] = include_bytes!("fixtures/aggregates/passed-report.json");
const CA: &[u8] = include_bytes!("fixtures/http/ca.pem");

fn fixture() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_mcp-doctor-stdio-fixture"))
}

fn replace_with_distinct_file(path: &Path, retained: &Path, bytes: &[u8]) -> std::io::Result<()> {
    fs::rename(path, retained)?;
    fs::write(path, bytes)
}

fn text(output: &Output) -> (&str, &str) {
    (
        std::str::from_utf8(&output.stdout).expect("stdout should be UTF-8"),
        std::str::from_utf8(&output.stderr).expect("stderr should be UTF-8"),
    )
}

fn finding<'a>(report: &'a Value, code: &str) -> &'a Value {
    report["checks"]
        .as_array()
        .expect("checks should be an array")
        .iter()
        .flat_map(|check| {
            check["findings"]
                .as_array()
                .expect("findings should be an array")
        })
        .find(|finding| finding["code"] == code)
        .unwrap_or_else(|| panic!("report should contain {code}"))
}

#[test]
fn replaced_scenario_fails_before_target_start() {
    let environment = TestEnvironment::new();
    let scenario_path = environment.artifact_path("scenario.json");
    let retained_path = environment.artifact_path("retained-scenario.json");
    let marker = environment.artifact_path("target-started");
    let scenario = serde_json::to_vec_pretty(&json!({
        "schema_version": "mcp-doctor.scenario/v1alpha1",
        "tool": TOOL,
        "safety": {"effects": "read_only"},
        "cases": [{
            "arguments": {"sequence": 1},
            "expect": {"result": "success"}
        }]
    }))
    .expect("the synthetic scenario should serialize");
    fs::write(&scenario_path, &scenario).expect("the scenario should be written");

    let mut command = environment.command();
    command
        .arg("check")
        .arg("--format")
        .arg("json")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--")
        .arg(fixture())
        .arg("active-started-marker")
        .arg(&marker);
    let output = run_with_bound_file_mutation(&mut command, &scenario_path, || {
        replace_with_distinct_file(&scenario_path, &retained_path, &scenario)
    });
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(!marker.exists(), "a replaced scenario started the target");
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(finding(&report, "MCP-SCENARIO-001")["location"], "scenario");
    for private in [scenario_path, retained_path, marker] {
        assert!(!stdout.contains(&private.to_string_lossy().to_string()));
    }
}

#[test]
fn oversized_scenario_replacement_fails_before_target_start() {
    let environment = TestEnvironment::new();
    let scenario_path = environment.artifact_path("scenario.json");
    let retained_path = environment.artifact_path("retained-scenario.json");
    let marker = environment.artifact_path("target-started");
    let scenario = serde_json::to_vec_pretty(&json!({
        "schema_version": "mcp-doctor.scenario/v1alpha1",
        "tool": TOOL,
        "safety": {"effects": "read_only"},
        "cases": [{
            "arguments": {"sequence": 1},
            "expect": {"result": "success"}
        }]
    }))
    .expect("the synthetic scenario should serialize");
    fs::write(&scenario_path, scenario).expect("the scenario should be written");

    let mut command = environment.command();
    command
        .arg("check")
        .arg("--format")
        .arg("json")
        .arg("--scenario")
        .arg(&scenario_path)
        .arg("--allow-tool")
        .arg(TOOL)
        .arg("--")
        .arg(fixture())
        .arg("active-started-marker")
        .arg(&marker);
    let output = run_with_bound_file_mutation(&mut command, &scenario_path, || {
        fs::rename(&scenario_path, &retained_path)?;
        fs::write(&scenario_path, vec![b' '; 1_048_577])
    });
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(
        !marker.exists(),
        "an oversized replacement started the target"
    );
    let report = parse_and_validate_report(&output.stdout);
    assert_eq!(finding(&report, "MCP-SCENARIO-001")["location"], "scenario");
    for private in [scenario_path, retained_path, marker] {
        assert!(!stdout.contains(&private.to_string_lossy().to_string()));
    }
}

#[test]
fn replaced_custom_ca_fails_before_credentials_or_connection() {
    let endpoint_listener =
        TcpListener::bind("127.0.0.1:0").expect("the endpoint trap should bind");
    let endpoint = format!(
        "https://127.0.0.1:{}/mcp",
        endpoint_listener
            .local_addr()
            .expect("the endpoint trap should have an address")
            .port()
    );
    let environment = TestEnvironment::new();
    let ca_path = environment.artifact_path("ca.pem");
    let retained_path = environment.artifact_path("retained-ca.pem");
    fs::write(&ca_path, CA).expect("the synthetic CA should be written");

    let mut command = environment.command();
    command
        .arg("inspect")
        .arg("--format")
        .arg("json")
        .arg(&endpoint)
        .arg("--allow-private-network")
        .arg(&endpoint)
        .arg("--allow-credentials-to")
        .arg(&endpoint)
        .arg("--bearer-token-env")
        .arg("MCP_DOCTOR_SYNTHETIC_MISSING_CREDENTIAL")
        .arg("--tls-ca-file")
        .arg(&ca_path);
    let output = run_with_bound_file_mutation(&mut command, &ca_path, || {
        replace_with_distinct_file(&ca_path, &retained_path, CA)
    });
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_report(&output.stdout);
    let finding = finding(&report, "MCP-TARGET-001");
    assert_eq!(finding["location"], "tls.trust");
    assert_eq!(finding["evidence"]["rule"], "invalid_trust_file");
    endpoint_listener
        .set_nonblocking(true)
        .expect("the endpoint trap should become nonblocking");
    assert!(
        endpoint_listener.accept().is_err(),
        "a replaced CA permitted a connection"
    );
    for private in [
        endpoint,
        "MCP_DOCTOR_SYNTHETIC_MISSING_CREDENTIAL".to_owned(),
        ca_path.to_string_lossy().into_owned(),
        retained_path.to_string_lossy().into_owned(),
    ] {
        assert!(!stdout.contains(&private));
        assert!(!stderr.contains(&private));
    }
}

#[test]
fn replaced_snapshot_produces_no_accepted_diff_evidence() {
    let environment = TestEnvironment::new();
    let before_path = environment.artifact_path("before.json");
    let retained_path = environment.artifact_path("retained-before.json");
    let after_path = environment.artifact_path("after.json");
    fs::write(&before_path, BEFORE).expect("the before snapshot should be written");
    fs::write(&after_path, AFTER).expect("the after snapshot should be written");

    let mut command = environment.command();
    command
        .arg("diff")
        .arg("--format")
        .arg("json")
        .arg(&before_path)
        .arg(&after_path);
    let output = run_with_bound_file_mutation(&mut command, &before_path, || {
        replace_with_distinct_file(&before_path, &retained_path, BEFORE.as_bytes())
    });
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let report = parse_and_validate_contract_diff(&output.stdout);
    assert_eq!(report["outcome"], "invalid");
    assert_eq!(report["findings"][0]["code"], "MCP-SNAPSHOT-001");
    assert_eq!(report["findings"][0]["input"], "before");
    assert_eq!(report["findings"][0]["change"], "artifact_invalid");
    for private in [before_path, retained_path, after_path] {
        assert!(!stdout.contains(&private.to_string_lossy().to_string()));
    }
}

#[test]
fn replaced_aggregate_input_leaves_no_output_evidence() {
    let environment = TestEnvironment::new();
    let input_path = environment.artifact_path("report.json");
    let retained_path = environment.artifact_path("retained-report.json");
    let output_path = environment.artifact_path("aggregate.json");
    fs::write(&input_path, REPORT).expect("the stable report should be written");

    let mut command = environment.command();
    command
        .arg("aggregate")
        .arg("--output")
        .arg(&output_path)
        .arg("--format")
        .arg("json")
        .arg(&input_path);
    let output = run_with_bound_file_mutation(&mut command, &input_path, || {
        replace_with_distinct_file(&input_path, &retained_path, REPORT)
    });
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stdout.is_empty());
    assert!(stderr.contains("aggregate input [0] could not be opened safely"));
    assert!(!output_path.exists());
    for private in [input_path, retained_path, output_path] {
        assert!(!stderr.contains(&private.to_string_lossy().to_string()));
    }
}
