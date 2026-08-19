#![cfg(feature = "internal-test-fixtures")]

mod support;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use support::{
    TestEnvironment, parse_and_validate_badge, parse_and_validate_junit,
    parse_and_validate_markdown, parse_and_validate_report,
};

const TOOL: &str = "synthetic.reviewed";
const REDACTION_SENTINEL: &str = "synthetic-secret-payload-7f2c";
const PRIVATE_ENUM_PROPERTY: &str = "synthetic_private_mode_never_report_7f2c";

fn fixture() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_mcp-doctor-stdio-fixture"))
}

fn reject_command(
    environment: &TestEnvironment,
    tool: &str,
    allowed_tool: &str,
    effects: &str,
    seed: u64,
) -> Command {
    let mut command = environment.command();
    command
        .arg("reject")
        .arg("--tool")
        .arg(tool)
        .arg("--allow-tool")
        .arg(allowed_tool)
        .arg("--effects")
        .arg(effects)
        .arg("--seed")
        .arg(seed.to_string());
    command
}

fn stdio_reject_command(environment: &TestEnvironment, seed: u64) -> Command {
    reject_command(environment, TOOL, TOOL, "read_only", seed)
}

fn text(output: &Output) -> (&str, &str) {
    (
        std::str::from_utf8(&output.stdout).expect("STDOUT should be UTF-8"),
        std::str::from_utf8(&output.stderr).expect("STDERR should be UTF-8"),
    )
}

fn assert_redacted(output: &Output, extra: &[&str]) {
    let (stdout, stderr) = text(output);
    for forbidden in [
        TOOL,
        REDACTION_SENTINEL,
        PRIVATE_ENUM_PROPERTY,
        "mcp-doctor-invalid-enum",
        "sequence",
        "secret",
    ]
    .into_iter()
    .chain(extra.iter().copied())
    .filter(|forbidden| !forbidden.is_empty())
    {
        assert!(!stdout.contains(forbidden), "STDOUT disclosed {forbidden}");
        assert!(!stderr.contains(forbidden), "STDERR disclosed {forbidden}");
    }
}

#[test]
fn exact_invalid_params_rejections_pass_with_fixed_value_free_reproduction() {
    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("reject-call-count.txt");
    let json_artifact = environment.artifact_path("reject-report.json");
    let junit = environment.artifact_path("reject-report.xml");
    let markdown_path = environment.artifact_path("reject-report.md");
    let badge_path = environment.artifact_path("reject-badge.json");
    let output = stdio_reject_command(&environment, 4242)
        .arg("--format")
        .arg("json")
        .arg("--json-report")
        .arg(&json_artifact)
        .arg("--junit-report")
        .arg(&junit)
        .arg("--markdown-report")
        .arg(&markdown_path)
        .arg("--badge-report")
        .arg(&badge_path)
        .arg("--")
        .arg(fixture())
        .arg("reject-success")
        .arg(&marker)
        .output()
        .expect("the bounded rejection diagnostic should run");
    let (_, stderr) = text(&output);
    let report = parse_and_validate_report(&output.stdout);

    assert!(output.status.success(), "{report:#}\n{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(fs::read_to_string(&marker).unwrap(), "7");
    assert_eq!(fs::read(&json_artifact).unwrap(), output.stdout);
    assert_eq!(report["protocol_revision"], "2026-07-28");
    assert_eq!(report["outcome"], "passed");

    let cases = report["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|check| {
            check["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("runtime.tools.case["))
        })
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 7);
    let expected = [
        Some("missing_arguments"),
        Some("wrong_root_type"),
        Some("omitted_required_property"),
        Some("wrong_property_type"),
        Some("forbidden_null"),
        Some("invalid_enum"),
        Some("unexpected_property"),
    ];
    for (case, expected) in cases.iter().zip(expected) {
        match expected {
            Some(kind) => {
                assert_eq!(case["state"], "performed");
                assert_eq!(case["outcome"], "passed");
                assert_eq!(case["reproduction"]["generator"], "mcp-doctor.generator/v1");
                assert_eq!(case["reproduction"]["seed"], 4242);
                assert_eq!(case["reproduction"]["mutation_kind"], kind);
                assert!(case["findings"].as_array().unwrap().is_empty());
            }
            None => {
                assert_eq!(case["state"], "skipped");
                assert_eq!(case["skip_reason"], "not_applicable");
                assert!(case.get("reproduction").is_none());
            }
        }
    }
    let (document, summary) = parse_and_validate_junit(&fs::read(&junit).unwrap());
    assert_eq!(summary.failures, 0);
    assert_eq!(summary.skipped, 0);
    assert!(document.contains("reproduction.mutation_kind=missing_arguments"));
    assert!(document.contains("report_outcome=passed\nexit_code=0"));
    let markdown = parse_and_validate_markdown(&fs::read(&markdown_path).unwrap());
    assert!(markdown.contains("| Outcome | `passed` |"));
    assert!(markdown.contains("`mutation=missing_arguments`"));
    assert!(markdown.contains("`runtime.tools.case[0]`"));
    let badge_bytes = fs::read(&badge_path).unwrap();
    let badge = parse_and_validate_badge(&badge_bytes);
    assert_eq!(badge["message"], "pass");
    for forbidden in [
        TOOL,
        REDACTION_SENTINEL,
        PRIVATE_ENUM_PROPERTY,
        "mcp-doctor-invalid-enum",
        "sequence",
        "secret",
        marker.to_str().unwrap(),
        json_artifact.to_str().unwrap(),
        junit.to_str().unwrap(),
        markdown_path.to_str().unwrap(),
        badge_path.to_str().unwrap(),
    ] {
        assert!(!document.contains(forbidden), "JUnit disclosed {forbidden}");
        assert!(
            !markdown.contains(forbidden),
            "Markdown disclosed {forbidden}"
        );
        assert!(
            !badge_bytes
                .windows(forbidden.len())
                .any(|window| window == forbidden.as_bytes()),
            "badge disclosed {forbidden}"
        );
    }
    assert_redacted(
        &output,
        &[
            marker.to_str().unwrap(),
            json_artifact.to_str().unwrap(),
            junit.to_str().unwrap(),
            markdown_path.to_str().unwrap(),
        ],
    );
}

#[test]
fn any_result_is_critical_unsafe_acceptance_and_stops_later_calls() {
    let environment = TestEnvironment::new();
    let output = stdio_reject_command(&environment, 9)
        .arg("--format")
        .arg("json")
        .arg("--")
        .arg(fixture())
        .arg("reject-unsafe-success")
        .output()
        .expect("the unsafe-success rejection journey should run");
    let report = parse_and_validate_report(&output.stdout);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(report["outcome"], "failed");
    assert_eq!(
        report["primary_diagnosis"]["check_id"],
        "runtime.tools.case[0]"
    );
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-ACTIVE-008"
    );
    let first_case = report["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|check| check["id"] == "runtime.tools.case[0]")
        .unwrap();
    assert_eq!(first_case["findings"][0]["severity"], "critical");
    for case in report["checks"].as_array().unwrap().iter().filter(|check| {
        check["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("runtime.tools.case[") && !id.ends_with("[0]"))
    }) {
        assert_eq!(case["state"], "skipped");
        assert_eq!(case["blocked_by"]["check_id"], "runtime.tools.case[0]");
    }
    assert_redacted(&output, &[]);
}

#[test]
fn unsafe_acceptance_has_matching_human_json_and_junit_primary_evidence() {
    let mut outputs = Vec::new();
    for format in ["human", "json", "junit"] {
        let environment = TestEnvironment::new();
        let mut command = stdio_reject_command(&environment, 10);
        if format != "human" {
            command.arg("--format").arg(format);
        }
        outputs.push((
            format,
            command
                .arg("--")
                .arg(fixture())
                .arg("reject-unsafe-success")
                .output()
                .expect("the unsafe acceptance reporter journey should run"),
        ));
    }

    for (_, output) in &outputs {
        assert_eq!(output.status.code(), Some(1));
        assert_redacted(output, &[]);
    }
    let human = text(&outputs[0].1).0;
    assert!(human.contains("PRIMARY DIAGNOSIS · runtime.tools.case[0]"));
    assert!(human.contains("MCP-ACTIVE-008"));

    let report = parse_and_validate_report(&outputs[1].1.stdout);
    assert_eq!(
        report["primary_diagnosis"]["check_id"],
        "runtime.tools.case[0]"
    );
    assert_eq!(
        report["primary_diagnosis"]["findings"][0]["code"],
        "MCP-ACTIVE-008"
    );

    let (junit, summary) = parse_and_validate_junit(&outputs[2].1.stdout);
    assert_eq!(summary.failures, 1);
    assert!(junit.contains("check_id=runtime.tools.case[0]"));
    assert!(junit.contains("primary_diagnosis=true"));
    assert!(junit.contains("type=\"MCP-ACTIVE-008\""));
    assert!(junit.contains("report_outcome=failed\nexit_code=1"));
}

#[test]
fn wrong_code_and_malformed_error_are_distinct_redacted_contract_failures() {
    for mode in ["reject-wrong-error", "reject-malformed-error"] {
        let environment = TestEnvironment::new();
        let output = stdio_reject_command(&environment, 19)
            .arg("--format")
            .arg("json")
            .arg("--")
            .arg(fixture())
            .arg(mode)
            .output()
            .expect("the rejection-envelope negative should run");
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            report["primary_diagnosis"]["check_id"],
            "runtime.tools.case[0]"
        );
        assert_eq!(
            report["primary_diagnosis"]["findings"][0]["code"],
            "MCP-ACTIVE-006"
        );
        assert_redacted(&output, &[mode]);
    }
}

#[test]
fn exact_tool_and_side_effect_authority_fail_before_target_start() {
    let environment = TestEnvironment::new();
    for (allowed, effects, side_effects) in [
        ("synthetic.other", "read_only", false),
        (TOOL, "side_effecting", false),
    ] {
        let marker = environment.artifact_path(&format!("not-started-{effects}-{allowed}"));
        let mut command = reject_command(&environment, TOOL, allowed, effects, 23);
        if side_effects {
            command.arg("--allow-side-effects");
        }
        let output = command
            .arg("--format")
            .arg("json")
            .arg("--")
            .arg(fixture())
            .arg("active-started-marker")
            .arg(&marker)
            .output()
            .expect("the authorization rejection should return");
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(output.status.code(), Some(2));
        let authorization = report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|check| check["id"] == "authorization.active")
            .unwrap();
        assert_eq!(authorization["outcome"], "failed");
        assert!(!marker.exists(), "authorization failure started the target");
        assert_redacted(&output, &[allowed, marker.to_str().unwrap()]);
    }
}

#[test]
fn invalid_report_destination_fails_before_target_start() {
    let environment = TestEnvironment::new();
    let existing = environment.artifact_path("existing-reject-report.json");
    let marker = environment.artifact_path("invalid-destination-target-started.marker");
    fs::write(&existing, b"unchanged").unwrap();

    let output = stdio_reject_command(&environment, 24)
        .arg("--json-report")
        .arg(&existing)
        .arg("--")
        .arg(fixture())
        .arg("active-started-marker")
        .arg(&marker)
        .output()
        .expect("the destination preflight should return");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stdout.is_empty());
    assert!(stderr.contains("already exists"));
    assert_eq!(fs::read(&existing).unwrap(), b"unchanged");
    assert!(!marker.exists(), "destination rejection started the target");
    for path in [&existing, &marker] {
        let path = path.to_string_lossy();
        assert!(!stdout.contains(path.as_ref()));
        assert!(!stderr.contains(path.as_ref()));
    }
}

#[test]
fn explicitly_authorized_side_effecting_rejection_runs() {
    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("side-effecting-reject-count.txt");
    let output = reject_command(&environment, TOOL, TOOL, "side_effecting", 27)
        .arg("--allow-side-effects")
        .arg("--")
        .arg(fixture())
        .arg("reject-success")
        .arg(&marker)
        .output()
        .expect("the redundantly authorized rejection diagnostic should run");
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert_eq!(fs::read_to_string(&marker).unwrap(), "7");
    assert!(stdout.contains("PASS  authorization.active"));
    assert_redacted(&output, &[marker.to_str().unwrap()]);
}

#[test]
fn cleanup_failure_remains_a_critical_independent_finding_after_rejections_pass() {
    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("cleanup-reject-count.txt");
    let output = stdio_reject_command(&environment, 28)
        .arg("--format")
        .arg("json")
        .arg("--")
        .arg(fixture())
        .arg("reject-success")
        .arg(&marker)
        .env("MCP_DOCTOR_INTERNAL_TEST_CLEANUP_FAILURE", "1")
        .output()
        .expect("the synthetic cleanup failure should return");
    let report = parse_and_validate_report(&output.stdout);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(fs::read_to_string(&marker).unwrap(), "7");
    assert!(report["checks"].as_array().unwrap().iter().any(|check| {
        check["id"] == "transport.stdio"
            && check["findings"].as_array().is_some_and(|findings| {
                findings.iter().any(|finding| {
                    finding["code"] == "MCP-SAFETY-001" && finding["severity"] == "critical"
                })
            })
    }));
    assert!(report["checks"].as_array().unwrap().iter().any(|check| {
        check["id"] == "runtime.tools.case[6]"
            && check["state"] == "performed"
            && check["outcome"] == "passed"
    }));
    assert_redacted(&output, &[marker.to_str().unwrap()]);
}

#[test]
fn invalid_schema_generation_and_transport_failures_keep_causal_evidence() {
    let cases = [
        ("reject-schema-invalid", "MCP-SCHEMA-001"),
        ("reject-schema-external", "MCP-SCHEMA-003"),
        ("reject-oversized-input", "MCP-LIMIT-001"),
        ("reject-impossible", "MCP-GENERATION-001"),
        ("reject-wrong-id", "MCP-TRANSPORT-003"),
        ("reject-clean-exit", "MCP-TRANSPORT-004"),
        ("reject-crash", "MCP-TRANSPORT-004"),
        ("reject-timeout", "MCP-LIMIT-001"),
        ("reject-oversize", "MCP-LIMIT-001"),
    ];
    for (mode, code) in cases {
        let environment = TestEnvironment::new();
        let output = stdio_reject_command(&environment, 29)
            .arg("--format")
            .arg("json")
            .arg("--")
            .arg(fixture())
            .arg(mode)
            .output()
            .expect("the bounded failure journey should return");
        let report = parse_and_validate_report(&output.stdout);
        assert_eq!(output.status.code(), Some(1), "{mode}: {report:#}");
        assert_eq!(
            report["primary_diagnosis"]["findings"][0]["code"], code,
            "{mode}: {report:#}"
        );
        assert_redacted(&output, &[mode]);
    }
}

#[test]
fn repeated_seed_runs_are_byte_deterministic_and_passive_inspection_calls_no_tool() {
    let mut reports = Vec::new();
    for ordinal in 0..2 {
        let environment = TestEnvironment::new();
        let marker = environment.artifact_path(&format!("deterministic-reject-{ordinal}.txt"));
        let output = stdio_reject_command(&environment, 7_529)
            .arg("--format")
            .arg("json")
            .arg("--")
            .arg(fixture())
            .arg("reject-success")
            .arg(&marker)
            .output()
            .expect("the repeated rejection run should complete");
        assert!(output.status.success(), "{:?}", text(&output));
        assert!(output.stderr.is_empty());
        assert_eq!(fs::read_to_string(marker).unwrap(), "7");
        assert_redacted(&output, &[]);
        reports.push(output.stdout);
    }
    assert_eq!(reports[0], reports[1]);

    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("passive-negative-call-count.txt");
    let output = environment
        .command()
        .arg("inspect")
        .arg("--format")
        .arg("json")
        .arg("--")
        .arg(fixture())
        .arg("reject-passive")
        .arg(&marker)
        .output()
        .expect("passive inspection should complete without a negative call");
    let report = parse_and_validate_report(&output.stdout);
    assert!(output.status.success(), "{report:#}");
    assert_eq!(fs::read_to_string(marker).unwrap(), "0");
    assert!(report["checks"].as_array().unwrap().iter().any(|check| {
        check["id"] == "runtime.tools"
            && check["state"] == "skipped"
            && check["skip_reason"] == "not_authorized"
    }));
    assert_redacted(&output, &[]);
}

#[test]
fn human_report_names_only_fixed_mutations_and_safe_correction() {
    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("human-reject-count.txt");
    let output = stdio_reject_command(&environment, u64::MAX)
        .arg("--")
        .arg(fixture())
        .arg("reject-success")
        .arg(&marker)
        .output()
        .expect("the human rejection report should run");
    let (stdout, stderr) = text(&output);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains("mutation=missing_arguments"));
    assert!(stdout.contains("mutation=unexpected_property"));
    assert!(stdout.contains("outcome passed · exit 0"));
    assert_redacted(&output, &[marker.to_str().unwrap()]);
}
