#![cfg(feature = "internal-test-fixtures")]

mod support;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;
use support::TestEnvironment;

const TOOL: &str = "synthetic.generated";
const REDACTION_SENTINEL: &str = "synthetic-secret-payload-7f2c";
const PRIVATE_QUERY: &str = "synthetic_private_query_never_report_7f2c";
const PRIVATE_LIMIT: &str = "synthetic_private_limit_never_report_7f2c";
const PRIVATE_FLAGS: &str = "synthetic_private_flags_never_report_7f2c";

fn fixture() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_mcp-doctor-stdio-fixture"))
}

fn break_command(
    environment: &TestEnvironment,
    tool: &str,
    allowed_tool: &str,
    effects: &str,
    cases: usize,
    seed: u64,
) -> Command {
    let mut command = environment.command();
    command
        .arg("break")
        .arg("--tool")
        .arg(tool)
        .arg("--allow-tool")
        .arg(allowed_tool)
        .arg("--effects")
        .arg(effects)
        .arg("--cases")
        .arg(cases.to_string())
        .arg("--seed")
        .arg(seed.to_string());
    command
}

fn stdio_break_command(
    environment: &TestEnvironment,
    cases: usize,
    seed: u64,
    mode: &str,
) -> Command {
    let mut command = break_command(environment, TOOL, TOOL, "read_only", cases, seed);
    command.arg("--").arg(fixture()).arg(mode);
    command
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
        PRIVATE_QUERY,
        PRIVATE_LIMIT,
        PRIVATE_FLAGS,
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
fn generated_cases_are_seeded_schema_valid_sequential_and_exactly_reproducible() {
    let first_environment = TestEnvironment::new();
    let first_marker = first_environment.artifact_path("first-generated-inputs.json");
    let first = stdio_break_command(&first_environment, 8, 4242, "break-success")
        .arg(&first_marker)
        .arg("8")
        .output()
        .expect("the first generated journey should run");
    let (first_stdout, first_stderr) = text(&first);
    assert!(first.status.success(), "{first_stdout}\n{first_stderr}");
    assert!(first_stderr.is_empty());
    for expected in [
        "PASS  generation.configuration",
        "PASS  authorization.active",
        "PASS  schema.contracts",
        "PASS  generation.cases",
        "PASS  runtime.tools.case[0]",
        "PASS  runtime.tools.case[7]",
        "mcp-doctor.generator/v1 · seed=4242 · input=object",
        "mcp-doctor.generator/v1 · seed=4249 · input=object",
        "outcome passed · exit 0",
    ] {
        assert!(first_stdout.contains(expected), "{first_stdout}");
    }
    assert!(!first_stdout.contains("scenario.configuration"));
    assert_redacted(&first, &[first_marker.to_str().unwrap()]);

    let second_environment = TestEnvironment::new();
    let second_marker = second_environment.artifact_path("second-generated-inputs.json");
    let second = stdio_break_command(&second_environment, 8, 4242, "break-success")
        .arg(&second_marker)
        .arg("8")
        .output()
        .expect("the repeated generated journey should run");
    assert!(second.status.success());
    assert_eq!(
        first.stdout, second.stdout,
        "the same seed changed its report"
    );
    assert_eq!(
        fs::read(first_marker).expect("the first generated arguments should be recorded"),
        fs::read(second_marker).expect("the second generated arguments should be recorded"),
        "the same seed changed the exact arguments sent to the tool"
    );
}

#[test]
fn machine_report_records_only_reproducible_seed_and_structural_input_evidence() {
    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("machine-generated-inputs.json");
    let output = break_command(&environment, TOOL, TOOL, "read_only", 3, u64::MAX - 1)
        .arg("--format")
        .arg("json")
        .arg("--")
        .arg(fixture())
        .arg("break-success")
        .arg(&marker)
        .arg("3")
        .output()
        .expect("the generated machine journey should run");
    let (_, stderr) = text(&output);
    assert!(output.status.success());
    assert!(stderr.is_empty());
    let report: Value =
        serde_json::from_slice(&output.stdout).expect("STDOUT should be one JSON report");
    assert_eq!(report["outcome"], "passed");
    assert_eq!(report["limits"]["active_cases"], 100);
    assert_eq!(report["limits"]["generation_attempts"], 256);
    assert_eq!(report["limits"]["generation_candidates"], 64);
    assert_eq!(report["limits"]["generation_steps"], 100_000);

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
    assert_eq!(cases.len(), 3);
    for (index, case) in cases.into_iter().enumerate() {
        let reproduction = &case["reproduction"];
        assert_eq!(reproduction["generator"], "mcp-doctor.generator/v1");
        assert_eq!(
            reproduction["seed"].as_u64(),
            Some((u64::MAX - 1).wrapping_add(index as u64))
        );
        assert_eq!(reproduction["input"]["root"], "object");
        assert!(
            reproduction["input"]["byte_count"]
                .as_u64()
                .is_some_and(|v| v > 0)
        );
        assert!(
            reproduction["input"]["node_count"]
                .as_u64()
                .is_some_and(|v| v > 0)
        );
        assert!(case.get("arguments").is_none());
        assert!(case.get("result").is_none());
    }
    assert_redacted(&output, &[marker.to_str().unwrap()]);
}

#[test]
fn exact_tool_effects_and_case_bounds_reject_before_target_start() {
    let cases = [
        (
            TOOL,
            "synthetic.other",
            "read_only",
            1,
            false,
            "MCP-AUTH-001",
        ),
        (TOOL, "*", "read_only", 1, false, "MCP-AUTH-001"),
        (TOOL, "synthetic.*", "read_only", 1, false, "MCP-AUTH-001"),
        (TOOL, TOOL, "side_effecting", 1, false, "MCP-AUTH-002"),
        ("", "", "read_only", 1, false, "MCP-GENERATION-001"),
        (TOOL, TOOL, "read_only", 0, false, "MCP-GENERATION-001"),
        (TOOL, TOOL, "read_only", 101, false, "MCP-LIMIT-001"),
    ];
    for (index, (tool, allowed, effects, case_count, allow_side_effects, code)) in
        cases.into_iter().enumerate()
    {
        let environment = TestEnvironment::new();
        let marker = environment.artifact_path(&format!("rejected-target-{index}"));
        let mut command = break_command(&environment, tool, allowed, effects, case_count, 9);
        if allow_side_effects {
            command.arg("--allow-side-effects");
        }
        let output = command
            .arg("--")
            .arg(fixture())
            .arg("active-started-marker")
            .arg(&marker)
            .output()
            .expect("the rejected generated invocation should run");
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
        assert!(stderr.is_empty());
        assert!(stdout.contains(code), "{stdout}");
        assert!(!marker.exists(), "a rejected target was started");
        assert_redacted(&output, &[tool, allowed, marker.to_str().unwrap()]);
    }
}

#[test]
fn explicitly_gated_side_effecting_generation_runs_but_annotations_never_authorize() {
    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("side-effecting-generated-input.json");
    let output = break_command(&environment, TOOL, TOOL, "side_effecting", 1, 77)
        .arg("--allow-side-effects")
        .arg("--")
        .arg(fixture())
        .arg("break-success")
        .arg(&marker)
        .arg("1")
        .output()
        .expect("the redundantly authorized generated call should run");
    let (stdout, stderr) = text(&output);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(marker.exists());
    assert!(stdout.contains("PASS  authorization.active"));
    assert_redacted(&output, &[marker.to_str().unwrap()]);
}

#[test]
fn generated_tool_errors_are_reproducible_findings_and_later_cases_continue() {
    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("later-generated-case");
    let output = stdio_break_command(&environment, 2, 5150, "break-tool-error")
        .arg(&marker)
        .output()
        .expect("the generated error continuation journey should run");
    let (stdout, stderr) = text(&output);
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(marker.exists(), "the later generated case was not called");
    assert!(stdout.contains("MCP-ACTIVE-004"), "{stdout}");
    assert!(stdout.contains("PASS  runtime.tools.case[1]"), "{stdout}");
    assert!(stdout.contains("seed=5150"), "{stdout}");
    assert!(stdout.contains("seed=5151"), "{stdout}");
    assert_redacted(&output, &[marker.to_str().unwrap()]);
}

#[test]
fn unsatisfiable_and_generation_limit_schemas_never_reach_tools_call() {
    for (mode, cases, expected, limit) in [
        ("break-impossible", 1, "MCP-GENERATION-001", None),
        (
            "break-oversized-input",
            1,
            "MCP-LIMIT-001",
            Some("instance_bytes"),
        ),
        (
            "break-aggregate-input",
            100,
            "MCP-LIMIT-001",
            Some("active_input_bytes"),
        ),
        (
            "break-generation-steps",
            1,
            "MCP-LIMIT-001",
            Some("generation_steps"),
        ),
    ] {
        let environment = TestEnvironment::new();
        let output = stdio_break_command(&environment, cases, 123, mode)
            .output()
            .expect("the bounded generation failure should run");
        let (stdout, stderr) = text(&output);
        assert_eq!(output.status.code(), Some(1), "{mode}: {stdout}\n{stderr}");
        assert!(stderr.is_empty());
        assert!(stdout.contains(expected), "{stdout}");
        if let Some(limit) = limit {
            assert!(stdout.contains(limit), "{stdout}");
        }
        assert!(
            stdout.contains("PRIMARY DIAGNOSIS · generation.cases"),
            "{stdout}"
        );
        assert!(stdout.contains("SKIP  runtime.tools.case[0]"), "{stdout}");
        assert!(stdout.contains("blocked by generation.cases"), "{stdout}");
        assert_redacted(&output, &[]);
    }
}

#[test]
fn generated_execution_never_retrieves_an_external_schema_or_calls_the_tool() {
    let environment = TestEnvironment::new();
    let output = stdio_break_command(&environment, 1, 123, "break-schema-external")
        .output()
        .expect("the blocked schema journey should run");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("MCP-SCHEMA-003"), "{stdout}");
    assert!(
        stdout.contains("PRIMARY DIAGNOSIS · schema.contracts"),
        "{stdout}"
    );
    assert!(stdout.contains("SKIP  generation.cases"), "{stdout}");
    assert!(stdout.contains("SKIP  runtime.tools.case[0]"), "{stdout}");
    assert!(stdout.contains("blocked by schema.contracts"), "{stdout}");
    assert_redacted(
        &output,
        &["https://synthetic.invalid/private-schema-never-report"],
    );
}

#[test]
fn generated_cleanup_terminates_and_reaps_a_resistant_process_tree() {
    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("generated-descendant-survived");
    let started = Instant::now();
    let output = stdio_break_command(&environment, 1, 303, "break-resistant-child")
        .arg(&marker)
        .output()
        .expect("the generated cleanup journey should run");
    let elapsed = started.elapsed();
    let (stdout, stderr) = text(&output);

    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(elapsed >= Duration::from_millis(1_800), "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(8), "{elapsed:?}");
    thread::sleep(Duration::from_secs(2));
    assert!(
        !marker.exists(),
        "the generated descendant survived cleanup"
    );
    assert_redacted(&output, &[marker.to_str().unwrap()]);
}

#[test]
fn the_exact_hundred_case_ceiling_runs_without_concurrency_or_hidden_expansion() {
    let environment = TestEnvironment::new();
    let marker = environment.artifact_path("hundred-generated-inputs.json");
    let output = stdio_break_command(&environment, 100, 9000, "break-success")
        .arg(&marker)
        .arg("100")
        .output()
        .expect("the exact generated case maximum should run");
    let (stdout, stderr) = text(&output);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.is_empty());
    assert!(stdout.contains("PASS  runtime.tools.case[99]"), "{stdout}");
    let values: Value = serde_json::from_slice(
        &fs::read(marker).expect("the generated arguments should be recorded"),
    )
    .expect("the generated arguments should be JSON");
    assert_eq!(values.as_array().map(Vec::len), Some(100));
    assert_redacted(&output, &[]);
}
