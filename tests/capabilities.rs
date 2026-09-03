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
        "--limit-profile",
        "--status",
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
    assert_eq!(manifest["limits"]["runtime_shutdown_timeout_ms"], 100);
    let diagnostic_limits = manifest["limit_profiles"]
        .as_array()
        .expect("limit profiles should be an array")
        .iter()
        .find(|profile| profile["id"] == "mcp-doctor.limits/diagnostic/v1")
        .expect("the diagnostic limit contract should be declared");
    assert_eq!(
        diagnostic_limits["selections"],
        json!(["default", "slow-start"])
    );
    assert_eq!(
        diagnostic_limits["selectable_for"],
        json!(["break", "check", "inspect"])
    );
    assert_eq!(
        manifest["schema_versions"]["contract_snapshot"],
        json!(["mcp-doctor.contract-snapshot/v1alpha1"])
    );
    assert_eq!(
        manifest["schema_versions"]["contract_diff"],
        json!(["mcp-doctor.contract-diff/v1alpha1"])
    );
    assert_eq!(
        manifest["schema_versions"]["markdown_report"],
        json!(["mcp-doctor.markdown/v1"])
    );
    assert_eq!(
        manifest["schema_versions"]["badge_report"],
        json!(["mcp-doctor.badge/v1"])
    );
    assert_eq!(
        manifest["schema_versions"]["scenario"],
        json!([
            "mcp-doctor.scenario/v1alpha1",
            "mcp-doctor.scenario/v2alpha1"
        ])
    );
    assert_eq!(
        manifest["schema_versions"]["status"],
        json!(["mcp-doctor.status/v1"])
    );
    assert_eq!(manifest["status"]["default"], "off");
    assert_eq!(manifest["status"]["stream"], "stderr");
    assert_eq!(
        manifest["status"]["commands"],
        json!(["break", "check", "inspect", "reject"])
    );
    assert_eq!(
        manifest["status"]["representations"],
        json!([
            {"name": "plain", "machine_readable": false},
            {"name": "jsonl", "machine_readable": true}
        ])
    );
    assert_eq!(manifest["status"]["jsonl_stderr_exclusive"], true);
    assert_eq!(manifest["status"]["limits"]["event_bytes"], 512);
    assert_eq!(manifest["status"]["limits"]["events"], 128);
    assert_eq!(manifest["status"]["limits"]["output_bytes"], 65_536);
    assert_eq!(manifest["status"]["limits"]["write_retries"], 0);
    let time_profiles = manifest["diagnostic_time_ceiling_profiles"]
        .as_array()
        .expect("diagnostic time ceiling profiles should be an array");
    assert_eq!(time_profiles.len(), 2);
    assert_eq!(time_profiles[0]["profile"], "default");
    assert_eq!(time_profiles[0]["startup"]["milliseconds"], 10_000);
    assert_eq!(time_profiles[0]["discovery"]["milliseconds"], 10_000);
    assert_eq!(time_profiles[0]["request"]["milliseconds"], 30_000);
    assert_eq!(time_profiles[0]["response"]["milliseconds"], 30_000);
    assert_eq!(time_profiles[0]["cleanup_grace"]["milliseconds"], 2_000);
    assert_eq!(time_profiles[0]["total"]["milliseconds"], 120_000);
    assert_eq!(time_profiles[1]["profile"], "slow-start");
    assert_eq!(time_profiles[1]["startup"]["milliseconds"], 30_000);
    assert_eq!(time_profiles[1]["discovery"]["milliseconds"], 30_000);
    assert_eq!(time_profiles[1]["request"]["milliseconds"], 60_000);
    assert_eq!(time_profiles[1]["response"]["milliseconds"], 60_000);
    assert_eq!(time_profiles[1]["cleanup_grace"]["milliseconds"], 2_000);
    assert_eq!(time_profiles[1]["total"]["milliseconds"], 240_000);
    let expected_scopes = [
        ("startup", "target_preparation_or_process_start"),
        ("discovery", "one_discovery_phase"),
        ("request", "one_request_write_or_http_exchange"),
        ("response", "one_response_wait"),
        (
            "cleanup_grace",
            "graceful_cleanup_before_forced_termination",
        ),
        ("total", "stdio_startup_or_http_preparation_through_cleanup"),
    ];
    for profile in time_profiles {
        assert_eq!(profile["whole_process_exit_guarantee"], false);
        for (phase, expected_scope) in expected_scopes {
            assert_eq!(profile[phase]["scope"], expected_scope, "{phase}");
        }
    }
    assert_eq!(manifest["protocol_selection"]["command"], "inspect");
    assert_eq!(manifest["protocol_selection"]["default_mode"], "auto");
    assert_eq!(
        manifest["protocol_selection"]["modes"],
        json!(["auto", "exact"])
    );
    assert_eq!(
        manifest["protocol_selection"]["compiled_modern_revisions"],
        json!(["2026-07-28"])
    );
    assert_eq!(
        manifest["protocol_selection"]["exact_revisions"],
        json!(["2026-07-28", "2025-11-25", "2025-06-18"])
    );
    assert_eq!(
        manifest["protocol_selection"]["exact_max_lifecycle_requests"],
        1
    );
    assert_eq!(manifest["protocol_selection"]["exact_max_fallbacks"], 0);
    let auto_transports = manifest["protocol_selection"]["auto_transports"]
        .as_array()
        .expect("auto selection transports should be an array");
    assert_eq!(auto_transports.len(), 2);
    assert_eq!(auto_transports[0]["transport"], "stdio");
    assert_eq!(auto_transports[0]["max_process_launches"], 2);
    assert_eq!(auto_transports[1]["transport"], "streamable_http");
    assert_eq!(auto_transports[1]["max_prepared_targets"], 1);
    for transport in auto_transports {
        assert_eq!(transport["max_lifecycle_requests"], 2);
        assert_eq!(transport["max_lifecycle_notifications"], 1);
        assert_eq!(transport["max_fallbacks"], 1);
        assert_eq!(transport["shared_total_and_aggregate_budgets"], true);
    }
    for transport in ["stdio", "streamable_http"] {
        let revisions = manifest["protocol_support"]
            .as_array()
            .expect("protocol support should be an array")
            .iter()
            .find(|support| support["command"] == "inspect" && support["transport"] == transport)
            .expect("inspect transport support should be declared")["revisions"]
            .clone();
        assert_eq!(revisions, json!(["2026-07-28", "2025-11-25", "2025-06-18"]));

        let reject_revisions = manifest["protocol_support"]
            .as_array()
            .expect("protocol support should be an array")
            .iter()
            .find(|support| support["command"] == "reject" && support["transport"] == transport)
            .expect("reject transport support should be declared")["revisions"]
            .clone();
        assert_eq!(reject_revisions, json!(["2026-07-28"]));
    }
    let inspect = manifest["commands"]
        .as_array()
        .expect("commands should be an array")
        .iter()
        .find(|command| command["name"] == "inspect")
        .expect("inspect should be declared");
    assert_eq!(inspect["reporters"], json!(["human", "json", "junit"]));
    assert_eq!(
        inspect["artifact_reporters"],
        json!(["json", "junit", "markdown", "badge"])
    );
    let markdown = manifest["reporters"]
        .as_array()
        .expect("reporters should be an array")
        .iter()
        .find(|reporter| reporter["name"] == "markdown")
        .expect("Markdown should be declared");
    assert_eq!(markdown["machine_readable"], false);
    let badge = manifest["reporters"]
        .as_array()
        .expect("reporters should be an array")
        .iter()
        .find(|reporter| reporter["name"] == "badge")
        .expect("badge should be declared");
    assert_eq!(badge["machine_readable"], true);
    assert!(first.stdout.len() <= 65_536);

    #[cfg(unix)]
    {
        let interruption = &manifest["interruption"];
        assert_eq!(interruption["platform_family"], "unix");
        assert_eq!(interruption["transport"], "stdio");
        assert_eq!(
            interruption["commands"],
            json!(["break", "check", "inspect", "reject"])
        );
        assert_eq!(interruption["signals"], json!(["SIGINT", "SIGTERM"]));
        assert_eq!(interruption["graceful_cleanup_ms"], 2_000);
        assert_eq!(interruption["forced_reap_ms"], 2_000);
        assert_eq!(interruption["cleanup_ceiling_ms"], 4_000);
        assert_eq!(interruption["incomplete_exit_code"], 3);
        assert_eq!(interruption["status_completion_reason"], "interrupted");
        assert_eq!(interruption["publishes_report"], false);
        assert_eq!(interruption["repeated_signal_forces_exit"], false);
    }
    #[cfg(not(unix))]
    assert!(manifest.get("interruption").is_none());

    manifest["product"]["version"] = json!("0.0.0-test");
    manifest["platform"] = json!({
        "family": "compiled_test",
        "process_tree_control": "compiled_test",
        "file_identity": "compiled_test"
    });
    manifest
        .as_object_mut()
        .expect("the manifest should be an object")
        .remove("interruption");
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
    assert!(stdout.contains(
        "inspect · passive · reporters human,json,junit · artifacts json,junit,markdown,badge"
    ));
    assert!(stdout.contains("check · stdio · 2026-07-28,2025-11-25,2025-06-18"));
    assert!(stdout.contains("reject · stdio · 2026-07-28"));
    assert!(stdout.contains(
        "Passive selection: inspect · default auto · modes auto,exact · modern 2026-07-28 · exact 2026-07-28,2025-11-25,2025-06-18"
    ));
    assert!(stdout.contains(
        "auto · stdio · path stdio_legacy_initialization · prepared_targets 0 · process_launches 2 · lifecycle_requests 2 · lifecycle_notifications 1 · fallbacks 1 · shared_budgets true"
    ));
    assert!(stdout.contains(
        "auto · streamable_http · path http_legacy_initialization · prepared_targets 1 · process_launches 0 · lifecycle_requests 2 · lifecycle_notifications 1 · fallbacks 1 · shared_budgets true"
    ));
    assert!(stdout.contains(
        "Limit selections: mcp-doctor.limits/diagnostic/v1 · default,slow-start · commands break,check,inspect"
    ));
    assert!(stdout.contains(
        "Status: mcp-doctor.status/v1 · default off · stream stderr · commands break,check,inspect,reject · representations plain,jsonl"
    ));
    #[cfg(unix)]
    assert!(stdout.contains(
        "Interruption: unix · stdio · signals SIGINT,SIGTERM · commands break,check,inspect,reject · graceful_cleanup_ms=2000 · forced_reap_ms=2000 · cleanup_ceiling_ms=4000 · exit_code=3 · completion_reason=interrupted · publishes_report=false · repeated_signal_forces_exit=false"
    ));
    #[cfg(not(unix))]
    assert!(!stdout.contains("Interruption:"));
    assert!(stdout.contains(
        "Time ceiling scopes: startup=target_preparation_or_process_start · discovery=one_discovery_phase · request=one_request_write_or_http_exchange · response=one_response_wait · cleanup_grace=graceful_cleanup_before_forced_termination · total=stdio_startup_or_http_preparation_through_cleanup"
    ));
    assert!(stdout.contains(
        "Time ceilings: default · startup_ms=10000 · discovery_ms=10000 · request_ms=30000 · response_ms=30000 · cleanup_grace_ms=2000 · total_ms=120000 · whole_process_exit_guarantee=false"
    ));
    assert!(stdout.contains(
        "Time ceilings: slow-start · startup_ms=30000 · discovery_ms=30000 · request_ms=60000 · response_ms=60000 · cleanup_grace_ms=2000 · total_ms=240000 · whole_process_exit_guarantee=false"
    ));
    assert!(stdout.contains("Runtime shutdown: timeout_ms=100 · scope=after_command_completion"));
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
