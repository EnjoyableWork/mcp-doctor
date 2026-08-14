mod support;

use std::process::{Command, Output};
use support::TestEnvironment;

fn run_cli(arguments: &[&str]) -> Output {
    let environment = TestEnvironment::new();
    let mut command = environment.command();
    command.args(arguments);
    command.output().expect("mcp-doctor should start")
}

#[test]
fn help_describes_the_installed_binary() {
    let output = run_cli(&["--help"]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    assert!(stdout.contains("Diagnose protocol, schema, and runtime failures in MCP servers"));
    assert!(stdout.contains("Usage: mcp-doctor"));
    assert!(stdout.contains("--version"));
    assert!(stdout.contains("diff"));
    assert!(stdout.contains("aggregate"));
}

#[test]
fn version_uses_the_binary_name_and_package_version() {
    let output = run_cli(&["--version"]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("version output should be UTF-8");
    assert_eq!(
        stdout,
        format!("mcp-doctor {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn empty_invocation_remains_a_no_op() {
    let output = run_cli(&[]);

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn inspect_help_documents_local_and_remote_target_boundaries() {
    let output = run_cli(&["inspect", "--help"]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    assert!(
        stdout.contains("Passively inspect a local STDIO server or one Streamable HTTP endpoint")
    );
    assert!(stdout.contains("Usage: mcp-doctor inspect [OPTIONS] <URL|TARGET>"));
    assert!(stdout.contains("literal arguments"));
    assert!(stdout.contains("--allow-private-network <EXACT-URL>"));
    assert!(stdout.contains("--allow-cleartext-http <EXACT-URL>"));
    assert!(stdout.contains("--allow-credentials-to <EXACT-URL>"));
    assert!(stdout.contains("--bearer-token-env <NAME>"));
    assert!(stdout.contains("--header-env <FIELD=NAME>"));
    assert!(stdout.contains("--tls-ca-file <PATH>"));
    assert!(stdout.contains("--format <FORMAT>"));
    assert!(stdout.contains("--json-report <PATH>"));
    assert!(stdout.contains("--junit-report <PATH>"));
    assert!(stdout.contains("--protocol-version <PROTOCOL_VERSION>"));
    assert!(stdout.contains("--snapshot <PATH>"));
    assert!(stdout.contains("--allow-sensitive-snapshot <EXACT-PATH>"));
    assert!(stdout.contains("sensitive selected-revision contract snapshot"));
    assert!(stdout.contains("2026-07-28"));
    assert!(stdout.contains("2025-11-25"));
    assert!(stdout.contains("2025-06-18"));
    assert!(stdout.contains("stable mcp-doctor.report/v1"));
    assert!(stdout.contains("[default: human]"));
    assert!(stdout.contains("[possible values: human, json, junit]"));
}

#[test]
fn diff_help_is_explicitly_local_and_has_only_human_or_json_output() {
    let output = run_cli(&["diff", "--help"]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    assert!(stdout.contains("without starting or contacting a target"));
    assert!(stdout.contains("Usage: mcp-doctor diff [OPTIONS] <BEFORE> <AFTER>"));
    assert!(stdout.contains("same-revision bounded snapshots"));
    assert!(stdout.contains("Earlier bounded local contract snapshot"));
    assert!(stdout.contains("Later bounded local contract snapshot"));
    assert!(stdout.contains("mcp-doctor.contract-diff/v1alpha1"));
    assert!(stdout.contains("[possible values: human, json]"));
    for prohibited in [
        "--endpoint",
        "--allow-private-network",
        "--allow-credentials-to",
        "--allow-tool",
        "--tls-ca-file",
        "--json-report",
        "--junit-report",
    ] {
        assert!(
            !stdout.contains(prohibited),
            "diff help exposed {prohibited}"
        );
    }
}

#[test]
fn aggregate_help_is_explicitly_offline_bounded_and_requires_an_artifact() {
    let output = run_cli(&["aggregate", "--help"]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    assert!(stdout.contains("without starting or contacting a target"));
    assert!(stdout.contains("Usage: mcp-doctor aggregate [OPTIONS] --output <PATH> <REPORT>..."));
    assert!(stdout.contains("--output <PATH>"));
    assert!(stdout.contains("stable mcp-doctor.aggregate/v1"));
    assert!(stdout.contains("Ordered stable mcp-doctor.report/v1 JSON files"));
    assert!(stdout.contains("[possible values: human, json]"));
    for prohibited in [
        "--endpoint",
        "--allow-private-network",
        "--allow-credentials-to",
        "--allow-tool",
        "--tls-ca-file",
        "--json-report",
        "--junit-report",
        "--scenario",
        "--target",
    ] {
        assert!(
            !stdout.contains(prohibited),
            "aggregate help exposed {prohibited}"
        );
    }
}

#[test]
fn aggregate_parse_failures_do_not_echo_paths_or_untrusted_values() {
    const PRIVATE: &str = "synthetic-private-aggregate-cli-value";

    let format_environment = TestEnvironment::new();
    let output_path = format_environment.artifact_path("aggregate.json");
    let input_path = format_environment.artifact_path(PRIVATE);
    let format = format_environment
        .command()
        .arg("aggregate")
        .arg("--output")
        .arg(&output_path)
        .arg("--format")
        .arg(PRIVATE)
        .arg(&input_path)
        .output()
        .expect("the rejected aggregate format should return");
    assert_eq!(format.status.code(), Some(2));
    assert!(format.stdout.is_empty());
    let stderr = String::from_utf8(format.stderr).unwrap();
    assert_eq!(
        stderr,
        "error: invalid aggregate invocation; use `mcp-doctor aggregate --help`\n"
    );
    assert!(!stderr.contains(PRIVATE));
    assert!(!stderr.contains(&output_path.to_string_lossy().to_string()));
    assert!(!output_path.exists());

    let count_environment = TestEnvironment::new();
    let output_path = count_environment.artifact_path("aggregate.json");
    let mut command = count_environment.command();
    command.arg("aggregate").arg("--output").arg(&output_path);
    for index in 0..33 {
        command.arg(format!("{PRIVATE}-{index}.json"));
    }
    let count = command
        .output()
        .expect("the rejected aggregate input count should return");
    assert_eq!(count.status.code(), Some(2));
    assert!(count.stdout.is_empty());
    let stderr = String::from_utf8(count.stderr).unwrap();
    assert_eq!(
        stderr,
        "error: invalid aggregate invocation; use `mcp-doctor aggregate --help`\n"
    );
    assert!(!stderr.contains(PRIVATE));
    assert!(!stderr.contains(&output_path.to_string_lossy().to_string()));
    assert!(!output_path.exists());
}

#[test]
fn legacy_revision_selection_is_exact_for_each_command() {
    let unknown = run_cli(&[
        "inspect",
        "--protocol-version",
        "2025-03-26",
        "--",
        "synthetic-target-must-not-start",
    ]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(unknown.stdout.is_empty());
    let stderr = String::from_utf8(unknown.stderr).expect("error output should be UTF-8");
    assert!(stderr.contains("invalid value '2025-03-26'"), "{stderr}");
    assert!(!stderr.contains("No such file"));

    let selected_v2025_06 = run_cli(&[
        "check",
        "--protocol-version",
        "2025-06-18",
        "--scenario",
        "synthetic.json",
        "--allow-tool",
        "synthetic.tool",
        "--",
        "synthetic-target-must-not-start",
    ]);
    assert_eq!(selected_v2025_06.status.code(), Some(2));
    assert!(selected_v2025_06.stderr.is_empty());
    let stdout =
        String::from_utf8(selected_v2025_06.stdout).expect("report output should be UTF-8");
    assert!(stdout.contains("mcp-doctor report · MCP 2025-06-18"));
    assert!(stdout.contains("MCP-SCENARIO-001"));
    assert!(!stdout.contains("synthetic-target-must-not-start"));

    let selected_active = run_cli(&[
        "check",
        "--protocol-version",
        "2025-11-25",
        "--scenario",
        "synthetic.json",
        "--allow-tool",
        "synthetic.tool",
        "--",
        "synthetic-target-must-not-start",
    ]);
    assert_eq!(selected_active.status.code(), Some(2));
    assert!(selected_active.stderr.is_empty());
    let stdout = String::from_utf8(selected_active.stdout).expect("report output should be UTF-8");
    assert!(stdout.contains("mcp-doctor report · MCP 2025-11-25"));
    assert!(stdout.contains("MCP-SCENARIO-001"));
    assert!(!stdout.contains("synthetic-target-must-not-start"));
}

#[test]
fn inspect_requires_exactly_one_local_or_remote_target() {
    let output = run_cli(&["inspect"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("error output should be UTF-8");
    assert!(stderr.contains("required arguments"));
    assert!(stderr.contains("Usage: mcp-doctor inspect <URL|TARGET>"));
}

#[test]
fn inspect_rejects_an_unknown_report_format_before_starting_a_target() {
    let output = run_cli(&[
        "inspect",
        "--format",
        "xml",
        "--",
        "synthetic-target-must-not-start",
    ]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("error output should be UTF-8");
    assert!(stderr.contains("invalid value 'xml'"));
    assert!(stderr.contains("[possible values: human, json, junit]"));
    assert!(!stderr.contains("No such file"));
}

#[test]
fn check_help_documents_every_redundant_active_gate() {
    let output = run_cli(&["check", "--help"]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    assert!(stdout.contains("Replay reviewed cases"));
    assert!(stdout.contains("--scenario <PATH>"));
    assert!(stdout.contains("--allow-tool <EXACT-NAME>"));
    assert!(stdout.contains("--allow-side-effects"));
    assert!(stdout.contains("--protocol-version <PROTOCOL_VERSION>"));
    assert!(stdout.contains("2026-07-28"));
    assert!(stdout.contains("2025-11-25"));
    assert!(stdout.contains("2025-06-18"));
    assert!(stdout.contains(
        "Usage: mcp-doctor check [OPTIONS] --scenario <PATH> --allow-tool <EXACT-NAME> <URL|TARGET>"
    ));
    assert!(stdout.contains("--allow-private-network <EXACT-URL>"));
    assert!(stdout.contains("--allow-credentials-to <EXACT-URL>"));
    assert!(stdout.contains("--json-report <PATH>"));
    assert!(stdout.contains("--junit-report <PATH>"));
}

#[test]
fn check_requires_scenario_tool_authorization_and_literal_target() {
    for arguments in [
        vec![
            "check",
            "--allow-tool",
            "synthetic.reviewed",
            "--",
            "target",
        ],
        vec!["check", "--scenario", "scenario.json", "--", "target"],
        vec![
            "check",
            "--scenario",
            "scenario.json",
            "--allow-tool",
            "synthetic.reviewed",
        ],
    ] {
        let output = run_cli(&arguments);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).expect("error output should be UTF-8");
        assert!(stderr.contains("required arguments"), "{stderr}");
    }
}

#[test]
fn break_help_documents_selection_consent_effect_seed_and_case_bounds() {
    let output = run_cli(&["break", "--help"]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    assert!(stdout.contains("Generate deterministic boundary cases"));
    assert!(stdout.contains("--tool <EXACT-NAME>"));
    assert!(stdout.contains("--allow-tool <EXACT-NAME>"));
    assert!(stdout.contains("--effects <EFFECTS>"));
    assert!(stdout.contains("[possible values: read_only, side_effecting]"));
    assert!(stdout.contains("--allow-side-effects"));
    assert!(stdout.contains("--cases <COUNT>"));
    assert!(stdout.contains("--seed <U64>"));
    assert!(stdout.contains("--protocol-version <PROTOCOL_VERSION>"));
    assert!(stdout.contains("2026-07-28"));
    assert!(stdout.contains("2025-11-25"));
    assert!(stdout.contains("2025-06-18"));
    assert!(stdout.contains("--allow-private-network <EXACT-URL>"));
    assert!(stdout.contains("--allow-credentials-to <EXACT-URL>"));
    assert!(stdout.contains("--json-report <PATH>"));
    assert!(stdout.contains("--junit-report <PATH>"));
}

#[test]
fn break_requires_every_generation_authority_and_one_literal_target() {
    for arguments in [
        vec![
            "break",
            "--allow-tool",
            "synthetic.generated",
            "--effects",
            "read_only",
            "--cases",
            "1",
            "--seed",
            "1",
            "--",
            "target",
        ],
        vec![
            "break",
            "--tool",
            "synthetic.generated",
            "--effects",
            "read_only",
            "--cases",
            "1",
            "--seed",
            "1",
            "--",
            "target",
        ],
        vec![
            "break",
            "--tool",
            "synthetic.generated",
            "--allow-tool",
            "synthetic.generated",
            "--cases",
            "1",
            "--seed",
            "1",
            "--",
            "target",
        ],
        vec![
            "break",
            "--tool",
            "synthetic.generated",
            "--allow-tool",
            "synthetic.generated",
            "--effects",
            "read_only",
            "--seed",
            "1",
            "--",
            "target",
        ],
        vec![
            "break",
            "--tool",
            "synthetic.generated",
            "--allow-tool",
            "synthetic.generated",
            "--effects",
            "read_only",
            "--cases",
            "1",
            "--",
            "target",
        ],
        vec![
            "break",
            "--tool",
            "synthetic.generated",
            "--allow-tool",
            "synthetic.generated",
            "--effects",
            "read_only",
            "--cases",
            "1",
            "--seed",
            "1",
        ],
    ] {
        let output = run_cli(&arguments);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).expect("error output should be UTF-8");
        assert!(stderr.contains("required arguments"), "{stderr}");
    }
}

#[test]
fn cli_processes_receive_only_disposable_user_locations() {
    let environment = TestEnvironment::new();
    let command: Command = environment.command();
    environment.assert_command_is_isolated(&command);
}
