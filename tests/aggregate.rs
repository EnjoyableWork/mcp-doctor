mod support;

use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};
use support::{TestEnvironment, parse_and_validate_aggregate};

const SECRET_SENTINEL: &str = "synthetic-private-aggregate-value-never-report";
const NETWORK_SENTINEL: &str = "http://127.0.0.1:9/synthetic-never-connect";

fn failed_report() -> Value {
    serde_json::from_str(include_str!("fixtures/contracts/failed-report.json"))
        .expect("the committed failed report fixture should be JSON")
}

fn passed_report() -> Value {
    let mut report = failed_report();
    let failed = report["checks"]
        .as_array_mut()
        .expect("the fixture should contain checks")
        .iter_mut()
        .find(|check| check["id"] == "schema.contracts")
        .expect("the fixture should contain its failed check");
    failed["findings"] = json!([]);
    failed["outcome"] = json!("passed");
    report["primary_diagnosis"] = Value::Null;
    report["independent_findings"] = json!([]);
    recompute_report(&mut report);
    report
}

fn incomplete_report() -> Value {
    let mut report = passed_report();
    let skipped = report["checks"]
        .as_array_mut()
        .expect("the fixture should contain checks")
        .iter_mut()
        .find(|check| check["id"] == "protocol.revision")
        .expect("the fixture should contain a required check");
    let object = skipped
        .as_object_mut()
        .expect("the check should be an object");
    object.insert("state".to_owned(), json!("skipped"));
    object.remove("outcome");
    object.insert("skip_reason".to_owned(), json!("input_required"));
    object.insert("findings".to_owned(), json!([]));
    recompute_report(&mut report);
    report
}

fn negotiated_mismatch_report() -> Value {
    let mut report = failed_report();
    let checks = report["checks"].as_array_mut().unwrap();
    checks.retain(|check| check["id"] != "protocol.revision");
    let failed = checks
        .iter_mut()
        .find(|check| check["id"] == "schema.contracts")
        .unwrap();
    failed["id"] = json!("protocol.revision");
    let finding = &mut failed["findings"][0];
    finding["code"] = json!("MCP-PROTOCOL-005");
    finding["location"] = json!("server.protocolVersion");
    finding["message"] =
        json!("The negotiated protocol revision differs from the selected revision.");
    finding["impact"] = json!("Continuing would apply the wrong protocol rules.");
    finding["expectation"] = json!("The negotiated revision must match the selected revision.");
    finding["remediation"] = json!("Configure the server to negotiate the selected revision.");
    finding["reference"] = json!("selected MCP revision lifecycle contract");
    finding["evidence"] = json!({"kind": "none"});
    report["primary_diagnosis"] = json!({
        "check_id": "protocol.revision",
        "findings": [{
            "code": "MCP-PROTOCOL-005",
            "location": "server.protocolVersion"
        }]
    });
    report["negotiated_protocol_revision"] = json!("2025-11-25");
    recompute_report(&mut report);
    report
}

fn report_with_checks(count: usize) -> Value {
    let mut report = passed_report();
    report["checks"] = Value::Array(
        (0..count)
            .map(|index| {
                json!({
                    "id": format!("synthetic.check[{index}]"),
                    "requirement": "required",
                    "state": "performed",
                    "outcome": "passed",
                    "findings": []
                })
            })
            .collect(),
    );
    report["primary_diagnosis"] = Value::Null;
    report["independent_findings"] = json!([]);
    recompute_report(&mut report);
    report
}

fn report_with_findings(padding_bytes: usize) -> Value {
    let mut report = failed_report();
    let padding = "x".repeat(padding_bytes);
    let findings = (0..256)
        .map(|index| {
            json!({
                "code": format!("MCP-SYNTHETIC-{index:03}"),
                "severity": "error",
                "protocol_revision": "2026-07-28",
                "location": format!("server.items[{index}]"),
                "message": padding,
                "impact": padding,
                "expectation": padding,
                "remediation": padding,
                "reference": padding,
                "evidence": {"kind": "none"}
            })
        })
        .collect::<Vec<_>>();
    report["checks"] = json!([{
        "id": "schema.contracts",
        "requirement": "required",
        "state": "performed",
        "outcome": "failed",
        "findings": findings
    }]);
    report["primary_diagnosis"] = Value::Object(
        [
            ("check_id".to_owned(), json!("schema.contracts")),
            (
                "findings".to_owned(),
                Value::Array(
                    (0..256)
                        .map(|index| {
                            json!({
                                "code": format!("MCP-SYNTHETIC-{index:03}"),
                                "location": format!("server.items[{index}]")
                            })
                        })
                        .collect(),
                ),
            ),
        ]
        .into_iter()
        .collect(),
    );
    report["independent_findings"] = json!([]);
    recompute_report(&mut report);
    report
}

fn recompute_report(report: &mut Value) {
    let checks = report["checks"]
        .as_array()
        .expect("a synthetic report should contain checks");
    let mut summary = json!({
        "checks": checks.len(),
        "required": 0,
        "optional": 0,
        "performed": 0,
        "skipped": 0,
        "passed": 0,
        "warned": 0,
        "failed": 0,
        "required_skipped": 0,
        "findings": {"info": 0, "warning": 0, "error": 0, "critical": 0}
    });
    for check in checks {
        let requirement = check["requirement"]
            .as_str()
            .expect("a check should declare its requirement");
        summary[requirement] = json!(summary[requirement].as_u64().unwrap() + 1);
        if check["state"] == "performed" {
            summary["performed"] = json!(summary["performed"].as_u64().unwrap() + 1);
            let outcome = check["outcome"]
                .as_str()
                .expect("a performed check should declare its outcome");
            let summary_name = match outcome {
                "passed" => "passed",
                "warning" => "warned",
                "failed" => "failed",
                _ => panic!("unexpected synthetic check outcome"),
            };
            summary[summary_name] = json!(summary[summary_name].as_u64().unwrap() + 1);
            for finding in check["findings"]
                .as_array()
                .expect("a performed check should contain findings")
            {
                let severity = finding["severity"]
                    .as_str()
                    .expect("a finding should declare severity");
                summary["findings"][severity] =
                    json!(summary["findings"][severity].as_u64().unwrap() + 1);
            }
        } else {
            summary["skipped"] = json!(summary["skipped"].as_u64().unwrap() + 1);
            if requirement == "required" {
                summary["required_skipped"] =
                    json!(summary["required_skipped"].as_u64().unwrap() + 1);
            }
        }
    }
    let outcome = if summary["failed"].as_u64().unwrap() > 0 {
        "failed"
    } else if summary["required"].as_u64().unwrap() == 0
        || summary["performed"].as_u64().unwrap() == 0
        || summary["required_skipped"].as_u64().unwrap() > 0
    {
        "incomplete"
    } else {
        "passed"
    };
    report["summary"] = summary;
    report["outcome"] = json!(outcome);
    report["exit_code"] = json!(match outcome {
        "passed" => 0,
        "failed" => 1,
        "incomplete" => 3,
        _ => unreachable!(),
    });
}

fn write_report(environment: &TestEnvironment, name: &str, report: &Value) -> PathBuf {
    let path = environment.artifact_path(name);
    fs::write(
        &path,
        serde_json::to_vec_pretty(report).expect("the synthetic report should serialize"),
    )
    .expect("the synthetic report should be writable");
    path
}

fn aggregate_command(
    environment: &TestEnvironment,
    output: &Path,
    format: &str,
    reports: &[PathBuf],
) -> Command {
    let mut command = environment.command();
    command
        .arg("aggregate")
        .arg("--output")
        .arg(output)
        .arg("--format")
        .arg(format)
        .args(reports);
    command
}

fn text(output: &Output) -> (&str, &str) {
    (
        std::str::from_utf8(&output.stdout).expect("aggregate stdout should be UTF-8"),
        std::str::from_utf8(&output.stderr).expect("aggregate stderr should be UTF-8"),
    )
}

fn assert_no_stages(root: &Path) {
    let stages = fs::read_dir(root)
        .expect("the disposable aggregate root should be readable")
        .map(|entry| entry.expect("the disposable entry should be readable"))
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".mcp-doctor-report-")
        })
        .collect::<Vec<_>>();
    assert!(
        stages.is_empty(),
        "owned aggregate stages should be removed"
    );
}

#[test]
fn workflow_step_and_cleanup_checks_survive_offline_aggregation() {
    let environment = TestEnvironment::new();
    let mut report = passed_report();
    report["checks"][0]["id"] = json!("runtime.workflow.step[0]");
    report["checks"].as_array_mut().unwrap().push(json!({
        "id": "runtime.workflow.cleanup[1]",
        "requirement": "required",
        "state": "performed",
        "outcome": "passed",
        "findings": []
    }));
    recompute_report(&mut report);
    let input = write_report(&environment, "workflow-report.json", &report);
    let output_path = environment.artifact_path("workflow-aggregate.json");
    let output = aggregate_command(&environment, &output_path, "json", &[input])
        .output()
        .expect("the workflow report should aggregate");

    assert!(output.status.success(), "{:?}", text(&output));
    let aggregate = parse_and_validate_aggregate(&output.stdout);
    let checks = aggregate["members"][0]["report"]["checks"]
        .as_array()
        .expect("the member checks should remain an array");
    assert!(
        checks
            .iter()
            .any(|check| check["id"] == "runtime.workflow.step[0]")
    );
    assert!(
        checks
            .iter()
            .any(|check| check["id"] == "runtime.workflow.cleanup[1]")
    );
}

#[test]
fn all_pass_json_is_deterministic_schema_valid_and_byte_identical_to_the_artifact() {
    let environment = TestEnvironment::new();
    let mut stdio = passed_report();
    stdio["limits"]["profile"] = json!("slow-start");
    stdio["limits"]["startup_ms"] = json!(30_000);
    stdio["limits"]["discovery_ms"] = json!(30_000);
    stdio["limits"]["request_ms"] = json!(60_000);
    stdio["limits"]["response_ms"] = json!(60_000);
    stdio["limits"]["total_ms"] = json!(240_000);
    stdio["checks"].as_array_mut().unwrap().insert(
        0,
        json!({
            "id": "transport.stdio",
            "requirement": "required",
            "state": "performed",
            "outcome": "passed",
            "findings": []
        }),
    );
    recompute_report(&mut stdio);
    let first = write_report(&environment, "first.json", &stdio);
    let mut legacy = passed_report();
    legacy["protocol_revision"] = json!("2025-11-25");
    legacy["negotiated_protocol_revision"] = json!("2025-11-25");
    legacy["checks"][0]["id"] = json!("runtime.tools.case[0]");
    legacy["checks"][0]["reproduction"] = json!({
        "generator": "mcp-doctor.generator/v1",
        "seed": 4242,
        "mutation_kind": "wrong_root_type",
        "input": {
            "root": "object",
            "byte_count": 2,
            "node_count": 1,
            "maximum_depth": 0,
            "nulls": 0,
            "booleans": 0,
            "numbers": 0,
            "strings": 0,
            "arrays": 0,
            "array_items": 0,
            "objects": 1,
            "object_members": 0
        }
    });
    legacy["checks"].as_array_mut().unwrap().insert(
        0,
        json!({
            "id": "transport.http",
            "requirement": "required",
            "state": "performed",
            "outcome": "passed",
            "findings": []
        }),
    );
    recompute_report(&mut legacy);
    let second = write_report(&environment, "second.json", &legacy);
    let output_path = environment.artifact_path("aggregate.json");
    let output = aggregate_command(
        &environment,
        &output_path,
        "json",
        &[first.clone(), second.clone()],
    )
    .output()
    .expect("the offline aggregate should run");
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let artifact = fs::read(&output_path).expect("the required aggregate should exist");
    assert_eq!(output.stdout, artifact);
    let aggregate = parse_and_validate_aggregate(&artifact);
    assert_eq!(aggregate["outcome"], "passed");
    assert_eq!(aggregate["exit_code"], 0);
    assert_eq!(
        aggregate["summary"],
        json!({
            "members": 2,
            "passed": 2,
            "failed": 0,
            "incomplete": 0
        })
    );
    assert_eq!(aggregate["members"][0]["ordinal"], 0);
    assert_eq!(aggregate["members"][1]["ordinal"], 1);
    assert_eq!(
        aggregate["members"][0]["report"]["limits"]["profile"],
        "slow-start"
    );
    assert_eq!(
        aggregate["members"][0]["report"]["limits"]["total_ms"],
        240_000
    );
    assert_eq!(
        aggregate["members"][1]["report"]["limits"]["profile"],
        "default"
    );
    assert_eq!(aggregate["limits"]["retries"], 0);
    assert_eq!(aggregate["limits"]["concurrency"], 1);
    assert_eq!(aggregate["limits"]["total_ms"], 10_000);
    assert_eq!(
        aggregate["members"][1]["report"]["negotiated_protocol_revision"],
        "2025-11-25"
    );
    assert_eq!(
        aggregate["members"][1]["report"]["checks"][1]["reproduction"]["seed"],
        4242
    );
    assert_eq!(
        aggregate["members"][1]["report"]["checks"][1]["reproduction"]["mutation_kind"],
        "wrong_root_type"
    );
    assert!(
        aggregate["members"][0]["report"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check["id"] == "transport.stdio")
    );
    assert!(
        aggregate["members"][1]["report"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check["id"] == "transport.http")
    );
    for excluded in [first, second, output_path.clone()] {
        assert!(
            !artifact
                .windows(excluded.as_os_str().as_encoded_bytes().len())
                .any(|window| window == excluded.as_os_str().as_encoded_bytes()),
            "aggregate disclosed an input or output path"
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(
            fs::metadata(&output_path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    assert_no_stages(environment.artifact_path("").as_path());

    let repeat_environment = TestEnvironment::new();
    let repeat_first = write_report(&repeat_environment, "renamed-a.json", &stdio);
    let repeat_second = write_report(&repeat_environment, "renamed-b.json", &legacy);
    let repeat_path = repeat_environment.artifact_path("repeat.json");
    let repeat = aggregate_command(
        &repeat_environment,
        &repeat_path,
        "json",
        &[repeat_first, repeat_second],
    )
    .output()
    .expect("the repeated aggregate should run");
    assert!(repeat.status.success(), "{:?}", text(&repeat));
    assert_eq!(
        repeat.stdout, artifact,
        "aggregate bytes must be deterministic"
    );
}

#[test]
fn committed_json_and_human_goldens_lock_the_stable_projection() {
    let fixture = |name: &str| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/aggregates")
            .join(name)
    };

    let json_environment = TestEnvironment::new();
    let json_path = json_environment.artifact_path("aggregate.json");
    let json_output = aggregate_command(
        &json_environment,
        &json_path,
        "json",
        &[fixture("passed-report.json")],
    )
    .output()
    .expect("the all-pass golden aggregate should run");
    assert!(json_output.status.success(), "{:?}", text(&json_output));
    assert_eq!(
        json_output.stdout,
        include_bytes!("fixtures/aggregates/all-pass.json")
    );
    assert_eq!(fs::read(json_path).unwrap(), json_output.stdout);
    let mut compatible = parse_and_validate_aggregate(&json_output.stdout);
    compatible["future_optional"] = json!({"consumer": "must ignore"});
    compatible["members"][0]["future_optional"] = json!(true);
    parse_and_validate_aggregate(&serde_json::to_vec(&compatible).unwrap());

    let human_environment = TestEnvironment::new();
    let artifact_path = human_environment.artifact_path("aggregate.json");
    let human_output = aggregate_command(
        &human_environment,
        &artifact_path,
        "human",
        &[
            fixture("passed-report.json"),
            fixture("incomplete-report.json"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/contracts/failed-report.json"),
        ],
    )
    .output()
    .expect("the mixed human golden aggregate should run");
    assert_eq!(
        human_output.status.code(),
        Some(1),
        "{:?}",
        text(&human_output)
    );
    assert_eq!(
        human_output.stdout,
        include_bytes!("fixtures/aggregates/mixed.txt")
    );
    let aggregate = parse_and_validate_aggregate(&fs::read(artifact_path).unwrap());
    assert_eq!(
        aggregate["summary"],
        json!({
            "members": 3,
            "passed": 1,
            "failed": 1,
            "incomplete": 1
        })
    );
}

#[test]
fn failed_then_incomplete_then_passed_precedence_never_demotes_a_member() {
    let incomplete_environment = TestEnvironment::new();
    let passed = write_report(&incomplete_environment, "passed.json", &passed_report());
    let incomplete = write_report(
        &incomplete_environment,
        "incomplete.json",
        &incomplete_report(),
    );
    let incomplete_output = incomplete_environment.artifact_path("aggregate.json");
    let result = aggregate_command(
        &incomplete_environment,
        &incomplete_output,
        "human",
        &[passed, incomplete],
    )
    .output()
    .expect("the incomplete aggregate should run");
    let (stdout, stderr) = text(&result);
    assert_eq!(result.status.code(), Some(3), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("0 failed · 1 incomplete · 1 passed · outcome incomplete · exit 3"));
    let aggregate = parse_and_validate_aggregate(&fs::read(incomplete_output).unwrap());
    assert_eq!(aggregate["outcome"], "incomplete");
    assert_eq!(aggregate["members"][1]["report"]["outcome"], "incomplete");

    let failed_environment = TestEnvironment::new();
    let passed = write_report(&failed_environment, "passed.json", &passed_report());
    let incomplete = write_report(&failed_environment, "incomplete.json", &incomplete_report());
    let failed = write_report(&failed_environment, "failed.json", &failed_report());
    let mut invocation_failed = failed_report();
    invocation_failed["exit_code"] = json!(2);
    let second_failed = write_report(
        &failed_environment,
        "failed-invocation.json",
        &invocation_failed,
    );
    let failed_output = failed_environment.artifact_path("aggregate.json");
    let result = aggregate_command(
        &failed_environment,
        &failed_output,
        "human",
        &[passed, incomplete, failed, second_failed],
    )
    .output()
    .expect("the failed aggregate should run");
    let (stdout, stderr) = text(&result);
    assert_eq!(result.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("2 failed · 1 incomplete · 1 passed · outcome failed · exit 1"));
    assert!(stdout.contains("PRIMARY DIAGNOSIS · schema.contracts"));
    assert!(stdout.contains("MCP-LIMIT-001 · tools[3].inputSchema"));
    let aggregate = parse_and_validate_aggregate(&fs::read(failed_output).unwrap());
    assert_eq!(aggregate["outcome"], "failed");
    assert_eq!(aggregate["summary"]["failed"], 2);
    assert_eq!(aggregate["members"][3]["report"]["exit_code"], 2);
}

#[test]
fn compatible_unknown_values_are_not_reflected_but_unknown_codes_keep_safe_metadata() {
    let environment = TestEnvironment::new();
    let mut report = failed_report();
    report["future_private_value"] = json!(SECRET_SENTINEL);
    report["checks"][1]["findings"][0]["future_private_value"] = json!({
        "secret": SECRET_SENTINEL,
        "endpoint": NETWORK_SENTINEL
    });
    report["checks"][1]["findings"][0]["code"] = json!("MCP-FUTURE-999");
    report["primary_diagnosis"]["findings"][0]["code"] = json!("MCP-FUTURE-999");
    report["independent_findings"] = json!([{
        "check_id": "schema.contracts",
        "code": "MCP-FUTURE-999",
        "location": "tools[3].inputSchema"
    }]);
    let input = write_report(&environment, "future.json", &report);
    let output_path = environment.artifact_path("aggregate.json");
    let output = aggregate_command(&environment, &output_path, "human", &[input])
        .output()
        .expect("the compatible future report should aggregate");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-FUTURE-999"));
    assert!(stdout.contains("INDEPENDENT SAFETY FINDINGS · 1"));
    let artifact = fs::read(&output_path).unwrap();
    assert!(!stdout.contains(SECRET_SENTINEL));
    assert!(!stdout.contains(NETWORK_SENTINEL));
    assert!(
        !artifact
            .windows(SECRET_SENTINEL.len())
            .any(|v| v == SECRET_SENTINEL.as_bytes())
    );
    assert!(
        !artifact
            .windows("SYNTHETIC_PRIVATE_ENV".len())
            .any(|v| v == b"SYNTHETIC_PRIVATE_ENV")
    );
    assert!(
        !artifact
            .windows(NETWORK_SENTINEL.len())
            .any(|v| v == NETWORK_SENTINEL.as_bytes())
    );
    let aggregate = parse_and_validate_aggregate(&artifact);
    assert_eq!(
        aggregate["members"][0]["report"]["checks"][1]["findings"][0]["code"],
        "MCP-FUTURE-999"
    );
    assert!(
        aggregate["members"][0]["report"]
            .get("future_private_value")
            .is_none()
    );
}

#[test]
fn a_diagnosed_negotiated_revision_mismatch_is_retained_without_coercion() {
    let environment = TestEnvironment::new();
    let input = write_report(
        &environment,
        "negotiated-mismatch.json",
        &negotiated_mismatch_report(),
    );
    let output_path = environment.artifact_path("aggregate.json");
    let output = aggregate_command(&environment, &output_path, "json", &[input])
        .output()
        .expect("the diagnosed mismatch should aggregate");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    let aggregate = parse_and_validate_aggregate(&fs::read(output_path).unwrap());
    let report = &aggregate["members"][0]["report"];
    assert_eq!(report["protocol_revision"], "2026-07-28");
    assert_eq!(report["negotiated_protocol_revision"], "2025-11-25");
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-PROTOCOL-005"
    );
}

#[test]
fn causal_skips_and_independent_safety_evidence_remain_visible() {
    let environment = TestEnvironment::new();
    let mut report: Value =
        serde_json::from_str(include_str!("fixtures/reports/unsupported-revision.json"))
            .expect("the committed causal report fixture should be JSON");
    let primary = report["primary_diagnosis"].clone();
    let primary_check = primary["check_id"].as_str().unwrap();
    let primary_finding = primary["findings"][0].clone();
    report["independent_findings"] = json!([{
        "check_id": primary_check,
        "code": primary_finding["code"],
        "location": primary_finding["location"]
    }]);
    let input = write_report(&environment, "causal.json", &report);
    let output_path = environment.artifact_path("aggregate.json");
    let output = aggregate_command(&environment, &output_path, "human", &[input])
        .output()
        .expect("the causal aggregate should run");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("INDEPENDENT SAFETY FINDINGS · 1"));
    assert!(stdout.contains("blocked by protocol.revision"));
    let aggregate = parse_and_validate_aggregate(&fs::read(output_path).unwrap());
    assert_eq!(
        aggregate["members"][0]["report"]["checks"][3]["blocked_by"],
        primary
    );
    assert_eq!(
        aggregate["members"][0]["report"]["checks"][2]["findings"][0]["evidence"],
        json!({
            "kind": "revision_advertisement",
            "required": "2026-07-28",
            "offered": 2,
            "recognized_legacy": 1,
            "unknown_date": 0,
            "opaque": 1
        })
    );
}

#[test]
fn schema_semantic_and_malformed_failures_are_atomic_and_value_free() {
    let cases = [
        ("malformed.json", Value::String("not-json".to_owned()), true),
        (
            "schema.json",
            {
                let mut report = passed_report();
                report["schema_version"] = json!("mcp-doctor.report/v2-private");
                report
            },
            false,
        ),
        (
            "summary.json",
            {
                let mut report = passed_report();
                report["summary"]["passed"] = json!(99);
                report
            },
            false,
        ),
        (
            "exit.json",
            {
                let mut report = passed_report();
                report["exit_code"] = json!(1);
                report
            },
            false,
        ),
        (
            "revision.json",
            {
                let mut report = failed_report();
                report["checks"][1]["findings"][0]["protocol_revision"] = json!("2025-11-25");
                report
            },
            false,
        ),
        (
            "reference.json",
            {
                let mut report = failed_report();
                report["primary_diagnosis"]["findings"][0]["location"] =
                    json!("server.privateNeverReport");
                report
            },
            false,
        ),
        (
            "duplicate-check.json",
            {
                let mut report = passed_report();
                report["checks"][1]["id"] = report["checks"][0]["id"].clone();
                report
            },
            false,
        ),
        (
            "finding-outcome.json",
            {
                let mut report = failed_report();
                report["checks"][1]["outcome"] = json!("passed");
                report
            },
            false,
        ),
        (
            "report-outcome.json",
            {
                let mut report = passed_report();
                report["outcome"] = json!("failed");
                report["exit_code"] = json!(1);
                report
            },
            false,
        ),
        (
            "primary-missing.json",
            {
                let mut report = failed_report();
                report["primary_diagnosis"] = Value::Null;
                report
            },
            false,
        ),
        (
            "independent-dangling.json",
            {
                let mut report = passed_report();
                report["independent_findings"] = json!([{
                    "check_id": "schema.contracts",
                    "code": "MCP-FUTURE-999",
                    "location": "server.synthetic"
                }]);
                report
            },
            false,
        ),
        (
            "noncausal-block.json",
            {
                let mut report = failed_report();
                report["checks"][2]["blocked_by"] = report["primary_diagnosis"].clone();
                report
            },
            false,
        ),
        (
            "causal-block-missing.json",
            {
                let mut report: Value = serde_json::from_str(include_str!(
                    "fixtures/reports/unsupported-revision.json"
                ))
                .unwrap();
                let causal = report["checks"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|check| check["skip_reason"] == "unsupported_revision")
                    .unwrap()
                    .as_object_mut()
                    .unwrap();
                causal.remove("blocked_by");
                report
            },
            false,
        ),
        (
            "undiagnosed-negotiated-mismatch.json",
            {
                let mut report = passed_report();
                report["negotiated_protocol_revision"] = json!("2025-11-25");
                report
            },
            false,
        ),
    ];

    for (name, report, literal) in cases {
        let environment = TestEnvironment::new();
        let input = environment.artifact_path(name);
        if literal {
            fs::write(&input, SECRET_SENTINEL).unwrap();
        } else {
            let mut report = report;
            report["future_private_value"] = json!(SECRET_SENTINEL);
            fs::write(&input, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        }
        let output_path = environment.artifact_path("must-not-exist.json");
        let output = aggregate_command(
            &environment,
            &output_path,
            "json",
            std::slice::from_ref(&input),
        )
        .output()
        .expect("the rejected aggregate should return");
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(2), "{name}: {stdout}\n{stderr}");
        assert!(stdout.is_empty(), "{name}: rejected input emitted stdout");
        assert!(!output_path.exists(), "{name}: rejected input left output");
        assert!(!stderr.contains(SECRET_SENTINEL), "{name}: {stderr}");
        assert!(
            !stderr.contains(&input.to_string_lossy().to_string()),
            "{name}: {stderr}"
        );
        assert!(stderr.contains("aggregate input [0]"), "{name}: {stderr}");
        assert_no_stages(environment.artifact_path("").as_path());
    }
}

#[test]
fn duplicate_hardlink_symlink_and_oversized_inputs_are_rejected_before_evidence() {
    let duplicate_environment = TestEnvironment::new();
    let input = write_report(&duplicate_environment, "input.json", &passed_report());
    let output_path = duplicate_environment.artifact_path("aggregate.json");
    let duplicate = aggregate_command(
        &duplicate_environment,
        &output_path,
        "human",
        &[input.clone(), input.clone()],
    )
    .output()
    .expect("the duplicate aggregate should return");
    assert_eq!(duplicate.status.code(), Some(2), "{:?}", text(&duplicate));
    assert!(duplicate.stdout.is_empty());
    assert!(!output_path.exists());

    let canonical_environment = TestEnvironment::new();
    let input = write_report(&canonical_environment, "input.json", &passed_report());
    let subdirectory = canonical_environment.artifact_path("subdirectory");
    fs::create_dir(&subdirectory).unwrap();
    let alias = subdirectory.join("..").join("input.json");
    let output_path = canonical_environment.artifact_path("aggregate.json");
    let canonical = aggregate_command(
        &canonical_environment,
        &output_path,
        "human",
        &[input, alias],
    )
    .output()
    .expect("the canonical-alias aggregate should return");
    assert_eq!(canonical.status.code(), Some(2), "{:?}", text(&canonical));
    assert!(canonical.stdout.is_empty());
    assert!(!output_path.exists());

    let hardlink_environment = TestEnvironment::new();
    let input = write_report(&hardlink_environment, "input.json", &passed_report());
    let alias = hardlink_environment.artifact_path("hardlink.json");
    fs::hard_link(&input, &alias).expect("the hard-link fixture should be created");
    let output_path = hardlink_environment.artifact_path("aggregate.json");
    let hardlink = aggregate_command(
        &hardlink_environment,
        &output_path,
        "human",
        &[input, alias],
    )
    .output()
    .expect("the hard-link aggregate should return");
    assert_eq!(hardlink.status.code(), Some(2), "{:?}", text(&hardlink));
    assert!(hardlink.stdout.is_empty());
    assert!(!output_path.exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let symlink_environment = TestEnvironment::new();
        let input = write_report(&symlink_environment, "input.json", &passed_report());
        let alias = symlink_environment.artifact_path("symlink.json");
        symlink(&input, &alias).expect("the symbolic-link fixture should be created");
        let output_path = symlink_environment.artifact_path("aggregate.json");
        let symbolic = aggregate_command(&symlink_environment, &output_path, "human", &[alias])
            .output()
            .expect("the symbolic aggregate should return");
        assert_eq!(symbolic.status.code(), Some(2), "{:?}", text(&symbolic));
        assert!(symbolic.stdout.is_empty());
        assert!(!output_path.exists());
    }

    let oversized_environment = TestEnvironment::new();
    let input = oversized_environment.artifact_path("oversized.json");
    let file = fs::File::create(&input).expect("the oversized fixture should be created");
    file.set_len(4 * 1024 * 1024 + 1)
        .expect("the oversized fixture should be extended");
    let output_path = oversized_environment.artifact_path("aggregate.json");
    let oversized = aggregate_command(&oversized_environment, &output_path, "human", &[input])
        .output()
        .expect("the oversized aggregate should return");
    assert_eq!(oversized.status.code(), Some(2), "{:?}", text(&oversized));
    assert!(oversized.stdout.is_empty());
    assert!(!output_path.exists());
}

#[test]
fn aggregate_destination_must_be_one_explicit_new_nonaliased_file() {
    for name in [
        "existing",
        "directory",
        "missing-parent",
        "input-alias",
        "dash",
    ] {
        let environment = TestEnvironment::new();
        let input = write_report(&environment, "input.json", &passed_report());
        let output_path = match name {
            "existing" => {
                let path = environment.artifact_path("existing.json");
                fs::write(&path, "external unchanged").unwrap();
                path
            }
            "directory" => {
                let path = environment.artifact_path("directory");
                fs::create_dir(&path).unwrap();
                path
            }
            "missing-parent" => environment.artifact_path("missing/aggregate.json"),
            "input-alias" => input.clone(),
            "dash" => PathBuf::from("-"),
            _ => unreachable!(),
        };
        let output = aggregate_command(
            &environment,
            &output_path,
            "json",
            std::slice::from_ref(&input),
        )
        .output()
        .expect("the unsafe destination should return");
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(2), "{name}: {stdout}\n{stderr}");
        assert!(
            stdout.is_empty(),
            "{name}: destination failure emitted stdout"
        );
        if name != "dash" {
            assert!(
                !stderr.contains(&output_path.to_string_lossy().to_string()),
                "{name}: destination path escaped"
            );
        }
        if name == "existing" {
            assert_eq!(
                fs::read_to_string(&output_path).unwrap(),
                "external unchanged"
            );
        }
        assert_no_stages(environment.artifact_path("").as_path());
    }
}

#[test]
fn input_count_depth_total_bytes_and_validation_work_are_finite() {
    let missing_environment = TestEnvironment::new();
    let output_path = missing_environment.artifact_path("aggregate.json");
    let missing = aggregate_command(&missing_environment, &output_path, "human", &[])
        .output()
        .expect("the missing-input invocation should return");
    assert_eq!(missing.status.code(), Some(2), "{:?}", text(&missing));
    assert!(missing.stdout.is_empty());
    assert!(!output_path.exists());

    let maximum_environment = TestEnvironment::new();
    let inputs = (0..32)
        .map(|index| {
            write_report(
                &maximum_environment,
                &format!("report-{index}.json"),
                &passed_report(),
            )
        })
        .collect::<Vec<_>>();
    let output_path = maximum_environment.artifact_path("aggregate.json");
    let maximum = aggregate_command(&maximum_environment, &output_path, "human", &inputs)
        .output()
        .expect("the maximum input count should aggregate");
    assert!(maximum.status.success(), "{:?}", text(&maximum));
    let aggregate = parse_and_validate_aggregate(&fs::read(output_path).unwrap());
    assert_eq!(aggregate["summary"]["members"], 32);
    assert_eq!(aggregate["members"][31]["ordinal"], 31);

    let count_environment = TestEnvironment::new();
    let output_path = count_environment.artifact_path("aggregate.json");
    let nonexistent = (0..33)
        .map(|index| PathBuf::from(format!("synthetic-report-{index}.json")))
        .collect::<Vec<_>>();
    let count = aggregate_command(&count_environment, &output_path, "human", &nonexistent)
        .output()
        .expect("the over-count invocation should return");
    assert_eq!(count.status.code(), Some(2), "{:?}", text(&count));
    assert!(count.stdout.is_empty());
    assert!(!output_path.exists());

    let depth_environment = TestEnvironment::new();
    let input = depth_environment.artifact_path("deep.json");
    let mut document = serde_json::to_string(&passed_report()).unwrap();
    assert_eq!(document.pop(), Some('}'));
    document.push_str(",\"future_depth\":");
    document.push_str(&"[".repeat(65));
    document.push_str("null");
    document.push_str(&"]".repeat(65));
    document.push('}');
    fs::write(&input, document).unwrap();
    let output_path = depth_environment.artifact_path("aggregate.json");
    let depth = aggregate_command(&depth_environment, &output_path, "human", &[input])
        .output()
        .expect("the over-depth invocation should return");
    assert_eq!(depth.status.code(), Some(2), "{:?}", text(&depth));
    assert!(depth.stdout.is_empty());
    assert!(!output_path.exists());

    let total_environment = TestEnvironment::new();
    let mut padded = passed_report();
    padded["future_padding"] = json!("x".repeat(3_400_000));
    let bytes = serde_json::to_vec(&padded).unwrap();
    assert!(bytes.len() < 4 * 1024 * 1024);
    let inputs = (0..5)
        .map(|index| {
            let name = format!("padded-{index}.json");
            let path = total_environment.artifact_path(&name);
            fs::write(&path, &bytes).unwrap();
            path
        })
        .collect::<Vec<_>>();
    let output_path = total_environment.artifact_path("aggregate.json");
    let total = aggregate_command(&total_environment, &output_path, "human", &inputs)
        .output()
        .expect("the over-total invocation should return");
    assert_eq!(total.status.code(), Some(2), "{:?}", text(&total));
    assert!(total.stdout.is_empty());
    assert!(!output_path.exists());

    let work_environment = TestEnvironment::new();
    let mut work = passed_report();
    work["future_nodes"] = Value::Array(vec![Value::Null; 170_000]);
    let inputs = (0..3)
        .map(|index| write_report(&work_environment, &format!("nodes-{index}.json"), &work))
        .collect::<Vec<_>>();
    let output_path = work_environment.artifact_path("aggregate.json");
    let work = aggregate_command(&work_environment, &output_path, "human", &inputs)
        .output()
        .expect("the over-work invocation should return");
    assert_eq!(work.status.code(), Some(2), "{:?}", text(&work));
    assert!(work.stdout.is_empty());
    assert!(!output_path.exists());
}

#[test]
fn aggregate_check_finding_and_rendered_output_limits_fail_atomically() {
    let check_environment = TestEnvironment::new();
    let check_report = report_with_checks(512);
    let inputs = (0..9)
        .map(|index| {
            write_report(
                &check_environment,
                &format!("checks-{index}.json"),
                &check_report,
            )
        })
        .collect::<Vec<_>>();
    let output_path = check_environment.artifact_path("aggregate.json");
    let checks = aggregate_command(&check_environment, &output_path, "human", &inputs)
        .output()
        .expect("the over-check invocation should return");
    assert_eq!(checks.status.code(), Some(2), "{:?}", text(&checks));
    assert!(checks.stdout.is_empty());
    assert!(!output_path.exists());

    let finding_environment = TestEnvironment::new();
    let finding_report = report_with_findings(1);
    let inputs = (0..9)
        .map(|index| {
            write_report(
                &finding_environment,
                &format!("findings-{index}.json"),
                &finding_report,
            )
        })
        .collect::<Vec<_>>();
    let output_path = finding_environment.artifact_path("aggregate.json");
    let findings = aggregate_command(&finding_environment, &output_path, "human", &inputs)
        .output()
        .expect("the over-finding invocation should return");
    assert_eq!(findings.status.code(), Some(2), "{:?}", text(&findings));
    assert!(findings.stdout.is_empty());
    assert!(!output_path.exists());

    let render_environment = TestEnvironment::new();
    let large_report = report_with_findings(1_800);
    let first = write_report(&render_environment, "large-a.json", &large_report);
    let second = write_report(&render_environment, "large-b.json", &large_report);
    assert!(fs::metadata(&first).unwrap().len() < 4 * 1024 * 1024);
    assert!(fs::metadata(&second).unwrap().len() < 4 * 1024 * 1024);
    let output_path = render_environment.artifact_path("aggregate.json");
    let rendered = aggregate_command(&render_environment, &output_path, "json", &[first, second])
        .output()
        .expect("the over-render invocation should return");
    assert_eq!(rendered.status.code(), Some(4), "{:?}", text(&rendered));
    assert!(rendered.stdout.is_empty());
    assert!(!output_path.exists());
    assert_no_stages(render_environment.artifact_path("").as_path());
}

#[test]
fn unknown_activity_hints_cannot_start_a_process_or_contact_a_listener() {
    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("must-not-start.marker");
    let listener = TcpListener::bind("127.0.0.1:0").expect("the local trap should bind");
    listener
        .set_nonblocking(true)
        .expect("the local trap should be nonblocking");
    let endpoint = format!("http://{}/private", listener.local_addr().unwrap());
    let mut report = passed_report();
    report["future_activity"] = json!({
        "command": marker,
        "endpoint": endpoint,
        "credential_env": "SYNTHETIC_PRIVATE_ENV",
        "schema": {"$ref": endpoint}
    });
    let input = write_report(&environment, "offline.json", &report);
    let output_path = environment.artifact_path("aggregate.json");
    let output = aggregate_command(&environment, &output_path, "json", &[input])
        .env("SYNTHETIC_PRIVATE_ENV", SECRET_SENTINEL)
        .output()
        .expect("the offline aggregate should run");

    assert!(output.status.success(), "{:?}", text(&output));
    assert!(
        !marker.exists(),
        "aggregate interpreted an unknown process hint"
    );
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
        "aggregate contacted an unknown endpoint"
    );
    let artifact = fs::read(output_path).unwrap();
    assert!(
        !artifact
            .windows(SECRET_SENTINEL.len())
            .any(|v| v == SECRET_SENTINEL.as_bytes())
    );
    assert!(
        !artifact
            .windows(endpoint.len())
            .any(|v| v == endpoint.as_bytes())
    );
}

#[cfg(feature = "internal-test-fixtures")]
#[test]
fn destination_create_write_and_cleanup_failures_are_atomic_with_correct_precedence() {
    for (name, variable, expected_exit) in [
        (
            "create",
            "MCP_DOCTOR_INTERNAL_TEST_REPORT_CREATE_FAILURE",
            2,
        ),
        ("write", "MCP_DOCTOR_INTERNAL_TEST_REPORT_WRITE_FAILURE", 4),
        (
            "cleanup",
            "MCP_DOCTOR_INTERNAL_TEST_REPORT_CLEANUP_FAILURE",
            4,
        ),
    ] {
        let environment = TestEnvironment::new();
        let input = write_report(&environment, "input.json", &passed_report());
        let output_path = environment.artifact_path("aggregate.json");
        let output = aggregate_command(&environment, &output_path, "json", &[input])
            .env(variable, "1")
            .output()
            .expect("the injected destination failure should return");
        let (stdout, stderr) = text(&output);
        assert_eq!(
            output.status.code(),
            Some(expected_exit),
            "{name}: {stdout}\n{stderr}"
        );
        assert!(
            stdout.is_empty(),
            "{name}: failed publication emitted evidence"
        );
        assert!(
            !output_path.exists(),
            "{name}: failed publication left output"
        );
        assert!(!stderr.contains(&output_path.to_string_lossy().to_string()));
        assert_no_stages(environment.artifact_path("").as_path());
    }
}
