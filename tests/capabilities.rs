mod support;

use std::net::TcpListener;
use std::process::{Command, Output};

use serde::Deserialize;
use serde_json::{Value, json};
use support::{TestEnvironment, parse_and_validate_capabilities};

const CAPABILITIES_SCHEMA_VERSION: &str = "mcp-doctor.capabilities/v1";
const PRIVATE_SENTINEL: &str = "synthetic-private-capability-value-never-report";

fn capabilities_command(environment: &TestEnvironment) -> Command {
    let mut command = environment.command();
    command.args(["capabilities", "--format", "json"]);
    command
}

fn run_capabilities(environment: &TestEnvironment) -> Output {
    capabilities_command(environment)
        .output()
        .expect("the capabilities command should start")
}

#[test]
fn help_exposes_only_compiled_capability_options() {
    let environment = TestEnvironment::new();
    let output = environment
        .command()
        .args(["capabilities", "--help"])
        .output()
        .expect("capabilities help should return");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("deterministic compiled capabilities"));
    assert!(stdout.contains("Usage: mcp-doctor capabilities [OPTIONS]"));
    assert!(stdout.contains("--format <FORMAT>"));
    assert!(stdout.contains("[possible values: human, json]"));
    assert!(stdout.contains("--schema-version <SCHEMA>"));
    assert!(stdout.contains("mcp-doctor.capabilities/v1"));
    assert!(stdout.contains("unsupported versions never fall back"));
    for prohibited in [
        "--endpoint",
        "--allow-private-network",
        "--allow-cleartext-http",
        "--allow-credentials-to",
        "--bearer-token-env",
        "--header-env",
        "--tls-ca-file",
        "--allow-tool",
        "--allow-side-effects",
        "--scenario",
        "--snapshot",
        "--output",
    ] {
        assert!(
            !stdout.contains(prohibited),
            "capabilities help exposed {prohibited}"
        );
    }
}

#[test]
fn json_manifest_is_schema_valid_deterministic_bounded_and_golden() {
    let environment = TestEnvironment::new();
    let first = run_capabilities(&environment);
    let second = run_capabilities(&environment);

    assert!(first.status.success());
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, second.stdout);
    assert!(second.status.success());
    assert!(second.stderr.is_empty());

    let mut manifest = parse_and_validate_capabilities(&first.stdout);
    assert_eq!(manifest["schema_version"], CAPABILITIES_SCHEMA_VERSION);
    assert_eq!(manifest["schema_stability"], "stable");
    assert_eq!(manifest["product"]["name"], "mcp-doctor");
    assert_eq!(manifest["product"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(manifest["limits"]["output_bytes"], 65_536);
    assert_eq!(
        manifest["schema_versions"]["contract_snapshot"],
        json!(["mcp-doctor.contract-snapshot/v1alpha1"])
    );
    assert_eq!(
        manifest["schema_versions"]["contract_diff"],
        json!(["mcp-doctor.contract-diff/v1alpha1"])
    );
    for transport in ["stdio", "streamable_http"] {
        let revisions = manifest["protocol_support"]
            .as_array()
            .expect("protocol support should be an array")
            .iter()
            .find(|support| support["command"] == "inspect" && support["transport"] == transport)
            .expect("inspect transport support should be declared")["revisions"]
            .clone();
        assert_eq!(revisions, json!(["2026-07-28", "2025-11-25", "2025-06-18"]));
    }
    assert!(first.stdout.len() <= 65_536);

    manifest["product"]["version"] = json!("0.0.0-test");
    manifest["platform"] = json!({
        "family": "compiled_test",
        "process_tree_control": "compiled_test",
        "file_identity": "compiled_test"
    });
    let golden: Value = serde_json::from_str(include_str!("fixtures/capabilities/golden.json"))
        .expect("the capability golden should be JSON");
    assert_eq!(manifest, golden);

    #[cfg(unix)]
    {
        let platform = parse_and_validate_capabilities(&first.stdout)["platform"].clone();
        assert_eq!(platform["family"], "unix");
        assert_eq!(platform["process_tree_control"], "process_group");
        assert_eq!(platform["file_identity"], "device_inode");
    }
    #[cfg(windows)]
    {
        let platform = parse_and_validate_capabilities(&first.stdout)["platform"].clone();
        assert_eq!(platform["family"], "windows");
        assert_eq!(platform["process_tree_control"], "job_object");
        assert_eq!(platform["file_identity"], "volume_file_id");
    }
}

#[test]
fn human_manifest_is_a_deterministic_summary_of_the_same_contract() {
    let environment = TestEnvironment::new();
    let first = environment
        .command()
        .arg("capabilities")
        .output()
        .expect("human capabilities should run");
    let second = environment
        .command()
        .arg("capabilities")
        .output()
        .expect("human capabilities should repeat");

    assert!(first.status.success());
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, second.stdout);
    assert!(first.stdout.len() <= 65_536);
    let stdout = String::from_utf8(first.stdout).unwrap();
    assert!(stdout.starts_with("mcp-doctor capabilities · mcp-doctor.capabilities/v1\n"));
    assert!(stdout.contains(&format!(
        "Product: mcp-doctor {}",
        env!("CARGO_PKG_VERSION")
    )));
    assert!(stdout.contains("inspect · passive"));
    assert!(stdout.contains("check · stdio · 2026-07-28,2025-11-25,2025-06-18"));
    assert!(stdout.contains("Exit semantics: mcp-doctor.exit/v1"));
}

#[test]
fn unknown_schema_requests_fail_exactly_without_reflecting_the_request() {
    let environment = TestEnvironment::new();
    let json_output = environment
        .command()
        .args([
            "capabilities",
            "--format",
            "json",
            "--schema-version",
            PRIVATE_SENTINEL,
        ])
        .output()
        .expect("an unknown JSON schema request should return");
    assert_eq!(json_output.status.code(), Some(2));
    assert!(json_output.stderr.is_empty());
    assert!(
        !json_output
            .stdout
            .windows(PRIVATE_SENTINEL.len())
            .any(|window| window == PRIVATE_SENTINEL.as_bytes())
    );
    let error = parse_and_validate_capabilities(&json_output.stdout);
    assert_eq!(error["schema_version"], CAPABILITIES_SCHEMA_VERSION);
    assert_eq!(error["error"]["code"], "unsupported_schema_version");
    assert_eq!(
        error["error"]["supported_schema_versions"],
        json!([CAPABILITIES_SCHEMA_VERSION])
    );
    assert!(error.get("product").is_none());

    let human_output = environment
        .command()
        .args(["capabilities", "--schema-version", PRIVATE_SENTINEL])
        .output()
        .expect("an unknown human schema request should return");
    assert_eq!(human_output.status.code(), Some(2));
    assert!(human_output.stdout.is_empty());
    let stderr = String::from_utf8(human_output.stderr).unwrap();
    assert_eq!(
        stderr,
        "error: unsupported capabilities schema; supported schema: mcp-doctor.capabilities/v1\n"
    );
    assert!(!stderr.contains(PRIVATE_SENTINEL));

    let invalid_format = environment
        .command()
        .args(["capabilities", "--format", PRIVATE_SENTINEL])
        .output()
        .expect("an unknown capability format should return");
    assert_eq!(invalid_format.status.code(), Some(2));
    assert!(invalid_format.stdout.is_empty());
    let stderr = String::from_utf8(invalid_format.stderr).unwrap();
    assert_eq!(
        stderr,
        "error: invalid capabilities invocation; use `mcp-doctor capabilities --help`\n"
    );
    assert!(!stderr.contains(PRIVATE_SENTINEL));
}

#[derive(Debug, Deserialize)]
struct ConsumerCase {
    command: String,
    transport: String,
    protocol_revision: String,
    expected: String,
}

#[test]
fn consumer_selects_skips_or_defers_without_help_or_product_version_checks() {
    let environment = TestEnvironment::new();
    let output = run_capabilities(&environment);
    assert!(output.status.success());
    let manifest = parse_and_validate_capabilities(&output.stdout);
    let cases: Vec<ConsumerCase> =
        serde_json::from_str(include_str!("fixtures/capabilities/consumer-cases.json"))
            .expect("the consumer cases should be JSON");

    for case in &cases {
        assert_eq!(classify(&manifest, case), case.expected, "{case:?}");
    }

    let mut forward = manifest;
    forward["future_top_level"] = json!({
        "target": PRIVATE_SENTINEL,
        "instructions": "must be ignored"
    });
    forward["product"]["future_build_contract"] = json!("unknown");
    forward["commands"][0]["future_activity_hint"] = json!({
        "command": PRIVATE_SENTINEL,
        "endpoint": PRIVATE_SENTINEL
    });
    forward["protocol_support"][0]["future_transport_detail"] = json!(true);
    forward["protocol_revisions"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "revision": "2027-01-01",
            "recognition": "future_status"
        }));
    forward["schema_versions"]["future_artifact"] = json!(["mcp-doctor.future/v1"]);
    let forward_bytes = serde_json::to_vec(&forward).unwrap();
    let forward = parse_and_validate_capabilities(&forward_bytes);
    for case in &cases {
        assert_eq!(classify(&forward, case), case.expected, "{case:?}");
    }
}

fn classify(manifest: &Value, request: &ConsumerCase) -> &'static str {
    if manifest["schema_version"] != CAPABILITIES_SCHEMA_VERSION {
        return "unknown";
    }
    let Some(support) = manifest["protocol_support"].as_array().and_then(|entries| {
        entries.iter().find(|entry| {
            entry["command"] == request.command && entry["transport"] == request.transport
        })
    }) else {
        return "unknown";
    };
    if support["revisions"].as_array().is_some_and(|revisions| {
        revisions
            .iter()
            .any(|revision| revision == &request.protocol_revision)
    }) {
        return "supported";
    }
    manifest["protocol_revisions"]
        .as_array()
        .and_then(|revisions| {
            revisions
                .iter()
                .find(|revision| revision["revision"] == request.protocol_revision)
        })
        .and_then(|revision| revision["recognition"].as_str())
        .filter(|recognition| matches!(*recognition, "supported" | "recognized_unsupported"))
        .map_or("unknown", |_| "unsupported")
}

#[test]
fn successful_manifest_ignores_ambient_secrets_proxies_and_target_like_arguments() {
    let environment = TestEnvironment::new();
    let trap = TcpListener::bind("127.0.0.1:0").expect("a loopback trap should bind");
    trap.set_nonblocking(true)
        .expect("the loopback trap should become nonblocking");
    let trap_url = format!("http://{}/private", trap.local_addr().unwrap());
    let output = capabilities_command(&environment)
        .env("SYNTHETIC_PRIVATE_CREDENTIAL", PRIVATE_SENTINEL)
        .env("HTTP_PROXY", &trap_url)
        .env("HTTPS_PROXY", &trap_url)
        .env("ALL_PROXY", &trap_url)
        .output()
        .expect("compiled capabilities should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(
        matches!(trap.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
        "capabilities contacted an ambient proxy or network trap"
    );
    for excluded in [PRIVATE_SENTINEL, trap_url.as_str()] {
        assert!(
            !output
                .stdout
                .windows(excluded.len())
                .any(|window| window == excluded.as_bytes()),
            "capabilities retained ambient input"
        );
    }

    let rejected = environment
        .command()
        .args([
            "capabilities",
            "--",
            "synthetic-target-must-not-start",
            PRIVATE_SENTINEL,
        ])
        .output()
        .expect("target-like capability arguments should be rejected");
    assert_eq!(rejected.status.code(), Some(2));
    assert!(rejected.stdout.is_empty());
    let stderr = String::from_utf8(rejected.stderr).unwrap();
    assert_eq!(
        stderr,
        "error: invalid capabilities invocation; use `mcp-doctor capabilities --help`\n"
    );
    assert!(!stderr.contains(PRIVATE_SENTINEL));
    assert!(!stderr.contains("synthetic-target-must-not-start"));
}
