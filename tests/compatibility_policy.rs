use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

const MATRIX: &str = include_str!("compatibility/matrix.json");
const EVIDENCE: &str = include_str!("compatibility/README.md");
const RUNNER: &str = include_str!("../scripts/compatibility.sh");
const WORKFLOW: &str = include_str!("../.github/workflows/compatibility.yml");
const DART_LOCK: &str = include_str!("compatibility/locks/mcp_dart-v2.4.0.pubspec.lock");
const PHP_LOCK: &str = include_str!("compatibility/locks/mcp-sdk-php-v2.0.0.composer.lock");
const GO_SCENARIO: &str = include_str!("compatibility/scenarios/official-go-greet.json");
const PHP_SCENARIO: &str = include_str!("compatibility/scenarios/independent-php-add.json");

fn object<'a>(value: &'a Value, context: &str) -> &'a serde_json::Map<String, Value> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("{context} must be an object"))
}

fn string<'a>(object: &'a serde_json::Map<String, Value>, key: &str) -> &'a str {
    object
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{key} must be a string"))
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[test]
fn compatibility_matrix_is_pinned_scoped_and_evidence_backed() {
    let matrix: Value = serde_json::from_str(MATRIX).expect("matrix must be valid JSON");
    let matrix = object(&matrix, "matrix");

    assert_eq!(
        string(matrix, "schema_version"),
        "mcp-doctor.compatibility/v1"
    );
    assert_eq!(string(matrix, "protocol_revision"), "2026-07-28");
    assert_eq!(string(matrix, "tool_execution"), "forbidden");
    assert_eq!(string(matrix, "transport"), "stdio");
    assert_eq!(string(matrix, "release_position"), "broad-current-revision");

    let runtimes = object(
        matrix.get("runtimes").expect("runtimes must exist"),
        "runtimes",
    );
    assert_eq!(runtimes.len(), 4);
    for (runtime_id, runtime) in runtimes {
        let runtime = object(runtime, runtime_id);
        let image = string(runtime, "image");
        let (_, digest) = image
            .split_once("@sha256:")
            .unwrap_or_else(|| panic!("{runtime_id} image must use a SHA-256 digest"));
        assert!(
            is_lower_hex(digest, 64),
            "{runtime_id} image digest is invalid"
        );
        assert!(!string(runtime, "version").is_empty());
    }

    let cases = matrix
        .get("cases")
        .and_then(Value::as_array)
        .expect("cases must be an array");
    assert_eq!(cases.len(), 4);

    let mut ids = BTreeSet::new();
    let mut languages = BTreeSet::new();
    let mut provenance_counts = BTreeMap::new();
    let mut reviewed_locks = BTreeSet::new();

    for case in cases {
        let case = object(case, "case");
        let id = string(case, "id");
        assert!(ids.insert(id), "case IDs must be unique");
        languages.insert(string(case, "language"));
        *provenance_counts
            .entry(string(case, "provenance"))
            .or_insert(0_usize) += 1;

        assert!(string(case, "repository").starts_with("https://github.com/"));
        assert!(!string(case, "release").is_empty());
        assert!(is_lower_hex(string(case, "commit"), 40));
        assert!(!string(case, "server").is_empty());
        assert_eq!(string(case, "expected_outcome"), "passed");

        let lock = object(
            case.get("dependency_lock")
                .expect("dependency_lock must exist"),
            "dependency_lock",
        );
        assert!(is_lower_hex(string(lock, "sha256"), 64));
        if string(lock, "source") == "mcp-doctor-reviewed" {
            let path = string(lock, "path");
            assert!(reviewed_locks.insert(path));
            assert!(
                Path::new(env!("CARGO_MANIFEST_DIR")).join(path).is_file(),
                "reviewed lock must be checked in: {path}"
            );
        } else {
            assert_eq!(string(lock, "source"), "upstream");
        }
    }

    assert_eq!(languages.len(), 4);
    assert_eq!(provenance_counts.get("official"), Some(&2));
    assert_eq!(provenance_counts.get("independent"), Some(&2));
    assert_eq!(reviewed_locks.len(), 2);
    assert!(DART_LOCK.contains("sdks:\n  dart:"));
    assert!(PHP_LOCK.contains("\"name\": \"psr/log\""));

    let last_verified = object(
        matrix
            .get("last_verified")
            .expect("last_verified must exist"),
        "last_verified",
    );
    assert_eq!(string(last_verified, "date"), "2026-08-10");
    assert_eq!(string(last_verified, "result"), "4/4 passed");

    let active = object(
        matrix
            .get("active_legacy")
            .expect("active_legacy must exist"),
        "active_legacy",
    );
    assert_eq!(string(active, "protocol_revision"), "2025-11-25");
    assert_eq!(string(active, "transport"), "stdio");
    assert_eq!(active["commands"], serde_json::json!(["check", "break"]));
    let active_verified = object(
        active
            .get("last_verified")
            .expect("active last_verified must exist"),
        "active last_verified",
    );
    assert_eq!(string(active_verified, "date"), "2026-08-14");
    assert_eq!(string(active_verified, "result"), "4/4 passed");

    let active_cases = active["cases"]
        .as_array()
        .expect("active cases must be an array");
    assert_eq!(active_cases.len(), 2);
    let mut active_languages = BTreeSet::new();
    let mut active_provenance = BTreeSet::new();
    for case in active_cases {
        let case = object(case, "active case");
        let id = string(case, "id");
        assert!(ids.contains(id), "active cases must reuse reviewed servers");
        active_languages.insert(string(case, "language"));
        active_provenance.insert(string(case, "provenance"));
        assert_eq!(string(case, "effects"), "read_only");
        assert_eq!(case["break_cases"].as_u64(), Some(3));
        assert_eq!(case["break_seed"].as_u64(), Some(6027));
        assert_eq!(string(case, "expected_outcome"), "passed");
        assert!(is_lower_hex(string(case, "scenario_sha256"), 64));

        let scenario_path = string(case, "scenario");
        let scenario_text = match scenario_path {
            "tests/compatibility/scenarios/official-go-greet.json" => GO_SCENARIO,
            "tests/compatibility/scenarios/independent-php-add.json" => PHP_SCENARIO,
            unexpected => panic!("unexpected active scenario: {unexpected}"),
        };
        assert!(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(scenario_path)
                .is_file()
        );
        let scenario: Value =
            serde_json::from_str(scenario_text).expect("active scenario must be JSON");
        assert_eq!(scenario["schema_version"], "mcp-doctor.scenario/v1alpha1");
        assert_eq!(scenario["tool"], string(case, "tool"));
        assert_eq!(scenario["safety"]["effects"], "read_only");
        assert_eq!(scenario["cases"].as_array().map(Vec::len), Some(1));
    }
    assert_eq!(active_languages.len(), 2);
    assert_eq!(
        active_provenance,
        BTreeSet::from(["official", "independent"])
    );
}

#[test]
fn active_legacy_compatibility_runner_retains_exact_authority_and_claim_scope() {
    for contract in [
        "--protocol-version 2025-11-25",
        "--allow-tool \"${tool}\"",
        "--effects \"${effects}\"",
        "--cases \"${break_cases}\"",
        "--seed \"${break_seed}\"",
        ".negotiated_protocol_revision == \"2025-11-25\"",
        "--network none",
        "--read-only",
        "--cap-drop ALL",
        "scenario_sha256",
    ] {
        assert!(
            RUNNER.contains(contract),
            "compatibility runner must preserve {contract}"
        );
    }
    assert!(!RUNNER.contains("--allow-side-effects"));
    for scope in [
        "narrow active STDIO reach across two implementations and two languages",
        "not broad legacy compatibility",
        "No legacy HTTP",
        "installed-channel",
        "official-conformance claim",
    ] {
        assert!(
            EVIDENCE.contains(scope),
            "compatibility evidence must preserve {scope}"
        );
    }
}

#[test]
fn public_compatibility_workflow_authenticates_one_exact_released_artifact() {
    for contract in [
        "version:\n        description: Exact released mcp-doctor version",
        "default: 0.4.2",
        "attestations: read",
        "contents: read",
        "gh release download \"$release_tag\"",
        "gh release verify \"$release_tag\"",
        "gh attestation verify \"$release_crate\"",
        "--source-digest \"$release_commit\"",
        "https://crates.io/api/v1/crates/mcp-doctor/$MCP_DOCTOR_VERSION/download",
        "--retry 0",
        "test \"$release_sha\" = \"$registry_sha\"",
        "cargo install mcp-doctor",
        "--version \"=$MCP_DOCTOR_VERSION\"",
        "--locked",
        "MCP_DOCTOR_COMPAT_BINARY=",
        "MCP_DOCTOR_COMPAT_VERSION=",
    ] {
        assert!(
            WORKFLOW.contains(contract),
            "compatibility workflow must preserve {contract}"
        );
    }
    for forbidden in [
        "pull_request_target:",
        "contents: write",
        "id-token: write",
        "secrets.",
        "continue-on-error:",
        "--allow-side-effects",
    ] {
        assert!(
            !WORKFLOW.contains(forbidden),
            "compatibility workflow unexpectedly contains {forbidden}"
        );
    }

    for contract in [
        "MCP_DOCTOR_COMPAT_BINARY",
        "MCP_DOCTOR_COMPAT_VERSION",
        "Released-artifact compatibility requires an exact stable version.",
        "Released-artifact compatibility requires one absolute executable regular file.",
        "Released-artifact compatibility binary and version do not agree.",
        "-L \"${compat_external_binary}\"",
        "! -x \"${compat_external_binary}\"",
    ] {
        assert!(
            RUNNER.contains(contract),
            "compatibility runner must preserve {contract}"
        );
    }
    for contract in [
        "verifies the immutable release and build\nprovenance",
        "public crates.io byte to have the same SHA-256 digest",
        "exact locked Cargo package",
        "rejects a symlink",
        "released-artifact workflow run is additional exact-identity evidence",
    ] {
        assert!(
            EVIDENCE.contains(contract),
            "compatibility evidence must preserve {contract}"
        );
    }
}
