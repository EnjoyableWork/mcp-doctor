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
fn inspect_help_documents_the_explicit_literal_target_boundary() {
    let output = run_cli(&["inspect", "--help"]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    assert!(stdout.contains("Passively inspect a local MCP server over STDIO"));
    assert!(stdout.contains("Usage: mcp-doctor inspect -- <TARGET>..."));
    assert!(stdout.contains("literal arguments"));
}

#[test]
fn inspect_requires_a_target_after_the_separator() {
    let output = run_cli(&["inspect"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("error output should be UTF-8");
    assert!(stderr.contains("required arguments"));
    assert!(stderr.contains("Usage: mcp-doctor inspect -- <TARGET>..."));
}

#[test]
fn cli_processes_receive_only_disposable_user_locations() {
    let environment = TestEnvironment::new();
    let command: Command = environment.command();
    environment.assert_command_is_isolated(&command);
}
