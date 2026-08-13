#![cfg(feature = "internal-test-fixtures")]

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};
use support::{TestEnvironment, parse_and_validate_junit, parse_and_validate_report};

const REVIEWED_TOOL: &str = "synthetic.reviewed";
const GENERATED_TOOL: &str = "synthetic.generated";

fn fixture() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_mcp-doctor-stdio-fixture"))
}

fn add_report_destinations(command: &mut Command, json: &Path, junit: &Path) {
    command
        .arg("--json-report")
        .arg(json)
        .arg("--junit-report")
        .arg(junit);
}

fn text(output: &Output) -> (&str, &str) {
    (
        std::str::from_utf8(&output.stdout).expect("STDOUT should be UTF-8"),
        std::str::from_utf8(&output.stderr).expect("STDERR should be UTF-8"),
    )
}

fn assert_artifact_parity(
    json_path: &Path,
    junit_path: &Path,
    expected_outcome: &str,
    expected_exit: i64,
) -> Value {
    let json_bytes = fs::read(json_path).expect("the requested JSON artifact should exist");
    let junit_bytes = fs::read(junit_path).expect("the requested JUnit artifact should exist");
    let report = parse_and_validate_report(&json_bytes);
    let (junit, summary) = parse_and_validate_junit(&junit_bytes);

    assert_eq!(report["outcome"], expected_outcome);
    assert_eq!(report["exit_code"], expected_exit);
    let checks = report["checks"]
        .as_array()
        .expect("the stable report should contain checks");
    assert_eq!(summary.tests, checks.len());
    assert_eq!(summary.failures, report["summary"]["failed"]);
    assert_eq!(summary.skipped, report["summary"]["skipped"]);
    assert!(junit.contains(&format!(
        "report_outcome={expected_outcome}\nexit_code={expected_exit}"
    )));
    for check in checks {
        let id = check["id"]
            .as_str()
            .expect("every report check should have an identifier");
        assert!(
            junit.contains(&format!("name=\"{id}\"")),
            "JUnit omitted {id}"
        );
    }
    report
}

fn assert_no_report_stages(path: &Path) {
    for entry in fs::read_dir(path).expect("the disposable artifact root should be readable") {
        let entry = entry.expect("each disposable artifact entry should be readable");
        let file_type = entry
            .file_type()
            .expect("the disposable artifact type should be readable");
        assert!(
            !entry
                .file_name()
                .to_string_lossy()
                .starts_with(".mcp-doctor-report-"),
            "an owned report stage remained"
        );
        if file_type.is_dir() {
            assert_no_report_stages(&entry.path());
        }
    }
}

fn reviewed_case(sequence: i64) -> Value {
    json!({
        "id": format!("synthetic-case-{sequence}"),
        "arguments": {"sequence": sequence},
        "expect": {
            "result": "success",
            "structured_output_schema": {
                "type": "object",
                "properties": {"ok": {"type": "boolean"}},
                "required": ["ok"],
                "additionalProperties": false
            }
        }
    })
}

fn write_scenario(environment: &TestEnvironment, cases: Vec<Value>) -> PathBuf {
    let path = environment.artifact_path("scenario.json");
    let scenario = json!({
        "schema_version": "mcp-doctor.scenario/v1alpha1",
        "tool": REVIEWED_TOOL,
        "safety": {"effects": "read_only"},
        "cases": cases
    });
    fs::write(
        &path,
        serde_json::to_vec_pretty(&scenario).expect("the scenario should serialize"),
    )
    .expect("the scenario should be writable");
    path
}

#[test]
fn existing_stdout_formats_are_byte_identical_and_one_run_writes_both_artifacts() {
    for format in ["human", "json", "junit"] {
        let baseline_environment = TestEnvironment::new();
        let baseline = baseline_environment
            .command()
            .arg("inspect")
            .arg("--format")
            .arg(format)
            .arg("--")
            .arg(fixture())
            .arg("catalog-valid")
            .output()
            .expect("the baseline inspection should run");
        assert!(baseline.status.success(), "{format}: {:?}", text(&baseline));
        assert!(baseline.stderr.is_empty());

        let environment = TestEnvironment::new();
        let json_path = environment.artifact_path("report.json");
        let junit_path = environment.artifact_path("report.xml");
        let run_marker = environment.artifact_path("one-run.marker");
        let mut command = environment.command();
        command.arg("inspect").arg("--format").arg(format);
        add_report_destinations(&mut command, &json_path, &junit_path);
        let output = command
            .arg("--")
            .arg(fixture())
            .arg("report-single-run")
            .arg(&run_marker)
            .output()
            .expect("the fan-out inspection should run");
        let (_, stderr) = text(&output);

        assert!(output.status.success(), "{format}: {:?}", text(&output));
        assert!(stderr.is_empty());
        assert_eq!(output.stdout, baseline.stdout, "{format} stdout changed");
        assert_eq!(fs::read(&run_marker).unwrap(), b"one target run");
        assert_artifact_parity(&json_path, &junit_path, "passed", 0);
        let artifact_bytes = [
            fs::read(&json_path).unwrap(),
            fs::read(&junit_path).unwrap(),
        ];
        for protected in [&json_path, &junit_path, &run_marker] {
            let protected = protected.to_string_lossy();
            assert!(
                !output
                    .stdout
                    .windows(protected.len())
                    .any(|v| v == protected.as_bytes())
            );
            for artifact in &artifact_bytes {
                assert!(
                    !artifact
                        .windows(protected.len())
                        .any(|v| v == protected.as_bytes()),
                    "an artifact disclosed an operating-system path"
                );
            }
            assert!(
                !output
                    .stderr
                    .windows(protected.len())
                    .any(|v| v == protected.as_bytes())
            );
        }
        #[cfg(unix)]
        for artifact in [&json_path, &junit_path] {
            use std::os::unix::fs::PermissionsExt as _;
            assert_eq!(
                fs::metadata(artifact).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        assert_no_report_stages(&environment.artifact_path(""));
    }
}

#[test]
fn failed_and_incomplete_diagnostics_publish_both_artifacts_with_their_diagnostic_exit() {
    let failed_environment = TestEnvironment::new();
    let failed_json = failed_environment.artifact_path("failed.json");
    let failed_junit = failed_environment.artifact_path("failed.xml");
    let mut failed_command = failed_environment.command();
    failed_command.arg("inspect");
    add_report_destinations(&mut failed_command, &failed_json, &failed_junit);
    let failed = failed_command
        .arg("--")
        .arg(fixture())
        .arg("protocol-unsupported")
        .output()
        .expect("the failing inspection should run");
    assert_eq!(failed.status.code(), Some(1), "{:?}", text(&failed));
    assert!(failed.stderr.is_empty());
    assert!(text(&failed).0.contains("outcome failed · exit 1"));
    assert_artifact_parity(&failed_json, &failed_junit, "failed", 1);

    let incomplete_environment = TestEnvironment::new();
    let scenario = write_scenario(
        &incomplete_environment,
        vec![reviewed_case(0), reviewed_case(1)],
    );
    let incomplete_json = incomplete_environment.artifact_path("incomplete.json");
    let incomplete_junit = incomplete_environment.artifact_path("incomplete.xml");
    let mut incomplete_command = incomplete_environment.command();
    incomplete_command
        .arg("check")
        .arg("--scenario")
        .arg(&scenario)
        .arg("--allow-tool")
        .arg(REVIEWED_TOOL);
    add_report_destinations(&mut incomplete_command, &incomplete_json, &incomplete_junit);
    let incomplete = incomplete_command
        .arg("--")
        .arg(fixture())
        .arg("active-input-required")
        .output()
        .expect("the incomplete check should run");
    assert_eq!(incomplete.status.code(), Some(3), "{:?}", text(&incomplete));
    assert!(incomplete.stderr.is_empty());
    assert!(text(&incomplete).0.contains("outcome incomplete · exit 3"));
    assert_artifact_parity(&incomplete_json, &incomplete_junit, "incomplete", 3);

    let rejected_environment = TestEnvironment::new();
    let scenario = write_scenario(&rejected_environment, vec![reviewed_case(0)]);
    let rejected_json = rejected_environment.artifact_path("rejected.json");
    let rejected_junit = rejected_environment.artifact_path("rejected.xml");
    let target_marker = rejected_environment.artifact_path("target-started.marker");
    let mut rejected_command = rejected_environment.command();
    rejected_command
        .arg("check")
        .arg("--scenario")
        .arg(&scenario)
        .arg("--allow-tool")
        .arg("synthetic.other");
    add_report_destinations(&mut rejected_command, &rejected_json, &rejected_junit);
    let rejected = rejected_command
        .arg("--")
        .arg(fixture())
        .arg("active-started-marker")
        .arg(&target_marker)
        .output()
        .expect("the rejected authorization should produce reports");
    assert_eq!(rejected.status.code(), Some(2), "{:?}", text(&rejected));
    assert!(rejected.stderr.is_empty());
    assert!(
        !target_marker.exists(),
        "authorization rejection started the target"
    );
    assert_artifact_parity(&rejected_json, &rejected_junit, "failed", 2);
}

#[test]
fn active_check_and_break_fan_out_without_replaying_execution() {
    let check_environment = TestEnvironment::new();
    let scenario = write_scenario(&check_environment, vec![reviewed_case(0)]);
    let check_json = check_environment.artifact_path("check.json");
    let check_junit = check_environment.artifact_path("check.xml");
    let check_run = check_environment.artifact_path("check-run.marker");
    let mut check_command = check_environment.command();
    check_command
        .arg("check")
        .arg("--scenario")
        .arg(&scenario)
        .arg("--allow-tool")
        .arg(REVIEWED_TOOL);
    add_report_destinations(&mut check_command, &check_json, &check_junit);
    let checked = check_command
        .arg("--")
        .arg(fixture())
        .arg("active-report-single-run")
        .arg(&check_run)
        .output()
        .expect("the one-run active check should run");
    assert!(checked.status.success(), "{:?}", text(&checked));
    assert!(checked.stderr.is_empty());
    assert_eq!(fs::read(check_run).unwrap(), b"one target run");
    assert_artifact_parity(&check_json, &check_junit, "passed", 0);

    let break_environment = TestEnvironment::new();
    let break_json = break_environment.artifact_path("break.json");
    let break_junit = break_environment.artifact_path("break.xml");
    let break_run = break_environment.artifact_path("break-run.marker");
    let observations = break_environment.artifact_path("observations.json");
    let mut break_command = break_environment.command();
    break_command
        .arg("break")
        .arg("--tool")
        .arg(GENERATED_TOOL)
        .arg("--allow-tool")
        .arg(GENERATED_TOOL)
        .arg("--effects")
        .arg("read_only")
        .arg("--cases")
        .arg("1")
        .arg("--seed")
        .arg("44");
    add_report_destinations(&mut break_command, &break_json, &break_junit);
    let broken = break_command
        .arg("--")
        .arg(fixture())
        .arg("break-report-single-run")
        .arg(&break_run)
        .arg(&observations)
        .arg("1")
        .output()
        .expect("the one-run generated check should run");
    assert!(broken.status.success(), "{:?}", text(&broken));
    assert!(broken.stderr.is_empty());
    assert_eq!(fs::read(break_run).unwrap(), b"one target run");
    let observations: Value = serde_json::from_slice(&fs::read(observations).unwrap()).unwrap();
    assert_eq!(observations.as_array().map(Vec::len), Some(1));
    assert_artifact_parity(&break_json, &break_junit, "passed", 0);
}

fn assert_preactivity_rejection(
    environment: &TestEnvironment,
    configure: impl FnOnce(&mut Command),
    expected: &str,
    protected_paths: &[&Path],
) {
    let marker = environment.artifact_path("target-started.marker");
    let mut command = environment.command();
    command.arg("inspect");
    configure(&mut command);
    let output = command
        .arg("--")
        .arg(fixture())
        .arg("snapshot-started-marker")
        .arg(&marker)
        .output()
        .expect("the rejected invocation should run");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert!(stdout.is_empty());
    assert!(stderr.contains(expected), "{stderr}");
    assert!(!marker.exists(), "artifact preflight started the target");
    for path in protected_paths.iter().copied().chain([marker.as_path()]) {
        let path = path.to_string_lossy();
        assert!(!stdout.contains(path.as_ref()));
        assert!(!stderr.contains(path.as_ref()));
    }
    assert_no_report_stages(&environment.artifact_path(""));
}

#[test]
fn destination_conflicts_and_creation_failures_are_rejected_before_activity() {
    let existing_environment = TestEnvironment::new();
    let existing = existing_environment.artifact_path("existing.json");
    fs::write(&existing, b"unchanged").unwrap();
    assert_preactivity_rejection(
        &existing_environment,
        |command| {
            command.arg("--json-report").arg(&existing);
        },
        "already exists",
        &[&existing],
    );
    assert_eq!(fs::read(existing).unwrap(), b"unchanged");

    let duplicate_environment = TestEnvironment::new();
    let duplicate = duplicate_environment.artifact_path("duplicate.out");
    assert_preactivity_rejection(
        &duplicate_environment,
        |command| add_report_destinations(command, &duplicate, &duplicate),
        "distinct report destinations",
        &[&duplicate],
    );

    let repeated_environment = TestEnvironment::new();
    let first = repeated_environment.artifact_path("first.json");
    let second = repeated_environment.artifact_path("second.json");
    assert_preactivity_rejection(
        &repeated_environment,
        |command| {
            command
                .arg("--json-report")
                .arg(&first)
                .arg("--json-report")
                .arg(&second);
        },
        "cannot be used multiple times",
        &[&first, &second],
    );

    let alias_environment = TestEnvironment::new();
    let alias_parent = alias_environment.artifact_path("alias-parent");
    fs::create_dir(&alias_parent).unwrap();
    let direct = alias_environment.artifact_path("aliased.out");
    let indirect = alias_parent.join("..").join("aliased.out");
    assert_preactivity_rejection(
        &alias_environment,
        |command| add_report_destinations(command, &direct, &indirect),
        "alias the same filesystem path",
        &[&direct, &indirect],
    );

    let missing_parent_environment = TestEnvironment::new();
    let unavailable = missing_parent_environment
        .artifact_path("missing")
        .join("report.json");
    assert_preactivity_rejection(
        &missing_parent_environment,
        |command| {
            command.arg("--json-report").arg(&unavailable);
        },
        "parent must already exist",
        &[&unavailable],
    );

    let directory_environment = TestEnvironment::new();
    let directory = directory_environment.artifact_path("report-directory");
    fs::create_dir(&directory).unwrap();
    assert_preactivity_rejection(
        &directory_environment,
        |command| {
            command.arg("--json-report").arg(&directory);
        },
        "non-regular filesystem entry",
        &[&directory],
    );

    let stdout_environment = TestEnvironment::new();
    assert_preactivity_rejection(
        &stdout_environment,
        |command| {
            command.arg("--json-report").arg("-");
        },
        "not a valid new-file target",
        &[],
    );
    assert!(!stdout_environment.artifact_path("-").exists());

    let snapshot_environment = TestEnvironment::new();
    let shared = snapshot_environment.artifact_path("shared.json");
    assert_preactivity_rejection(
        &snapshot_environment,
        |command| {
            command
                .arg("--snapshot")
                .arg(&shared)
                .arg("--allow-sensitive-snapshot")
                .arg(&shared)
                .arg("--json-report")
                .arg(&shared);
        },
        "alias the same filesystem path",
        &[&shared],
    );

    let create_environment = TestEnvironment::new();
    let create_path = create_environment.artifact_path("create.json");
    assert_preactivity_rejection(
        &create_environment,
        |command| {
            command
                .env("MCP_DOCTOR_INTERNAL_TEST_REPORT_CREATE_FAILURE", "1")
                .arg("--json-report")
                .arg(&create_path);
        },
        "could not be prepared safely",
        &[&create_path],
    );
}

#[test]
fn render_write_and_cleanup_failures_are_visible_and_publish_no_artifact_set() {
    for (hook, expected, expects_stdout) in [
        (
            "MCP_DOCTOR_INTERNAL_TEST_REPORT_RENDER_FAILURE",
            "could not be rendered safely",
            false,
        ),
        (
            "MCP_DOCTOR_INTERNAL_TEST_REPORT_WRITE_FAILURE",
            "could not be written completely",
            true,
        ),
        (
            "MCP_DOCTOR_INTERNAL_TEST_REPORT_CLEANUP_FAILURE",
            "cleanup did not complete",
            true,
        ),
    ] {
        let environment = TestEnvironment::new();
        let json_path = environment.artifact_path("report.json");
        let junit_path = environment.artifact_path("report.xml");
        let marker = environment.artifact_path("one-run.marker");
        let mut command = environment.command();
        command.arg("inspect").env(hook, "1");
        add_report_destinations(&mut command, &json_path, &junit_path);
        let output = command
            .arg("--")
            .arg(fixture())
            .arg("report-single-run")
            .arg(&marker)
            .output()
            .expect("the injected report failure should run");
        let (stdout, stderr) = text(&output);

        assert_eq!(output.status.code(), Some(4), "{hook}: {stdout}\n{stderr}");
        assert_eq!(!stdout.is_empty(), expects_stdout, "{hook}: {stdout}");
        assert!(stderr.contains(expected), "{hook}: {stderr}");
        assert!(
            marker.exists(),
            "{hook}: target did not run before postflight"
        );
        assert!(
            !json_path.exists(),
            "{hook}: partial JSON artifact remained"
        );
        assert!(
            !junit_path.exists(),
            "{hook}: partial JUnit artifact remained"
        );
        for protected in [&json_path, &junit_path, &marker] {
            let protected = protected.to_string_lossy();
            assert!(!stdout.contains(protected.as_ref()));
            assert!(!stderr.contains(protected.as_ref()));
        }
        assert_no_report_stages(&environment.artifact_path(""));
    }
}

#[test]
fn report_failure_exit_takes_precedence_over_a_diagnostic_failure_exit() {
    let environment = TestEnvironment::new();
    let json_path = environment.artifact_path("report.json");
    let junit_path = environment.artifact_path("report.xml");
    let mut command = environment.command();
    command
        .arg("inspect")
        .env("MCP_DOCTOR_INTERNAL_TEST_REPORT_WRITE_FAILURE", "1");
    add_report_destinations(&mut command, &json_path, &junit_path);
    let output = command
        .arg("--")
        .arg(fixture())
        .arg("protocol-unsupported")
        .output()
        .expect("the failing diagnostic with a report failure should run");
    let (stdout, stderr) = text(&output);

    assert_eq!(output.status.code(), Some(4), "{stdout}\n{stderr}");
    assert!(stdout.contains("outcome failed · exit 1"), "{stdout}");
    assert!(
        stderr.contains("could not be written completely"),
        "{stderr}"
    );
    assert!(!json_path.exists());
    assert!(!junit_path.exists());
    assert_no_report_stages(&environment.artifact_path(""));
}
