use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::fs;

const WORKFLOW: &str = include_str!("../.github/workflows/mcp-doctor-preflight.yml");
const AUTOMATION: &str = include_str!("../docs/automation.md");
const REPORT_VERIFIER: &str = include_str!("../scripts/verify-mcp-doctor-preflight-reports.sh");

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn starter_is_exact_passive_least_permission_and_narrowly_triggered() {
    for contract in [
        "pull_request:\n    paths:",
        "workflow_dispatch:",
        "permissions:\n  contents: read",
        "cancel-in-progress: false",
        "runs-on: ubuntu-24.04",
        "timeout-minutes: 20",
        "permissions:\n      contents: read",
        "MCP_DOCTOR_VERSION: 0.3.3",
        "cargo install mcp-doctor",
        "--version \"=$MCP_DOCTOR_VERSION\"",
        "--locked",
        "mcp-doctor $MCP_DOCTOR_VERSION",
        "mcp-doctor\" inspect",
        "--protocol-version 2026-07-28",
        "--json-report artifacts/mcp-doctor/report.json",
        "--junit-report artifacts/mcp-doctor/report.junit.xml",
        "./target/release/mcp-doctor-stdio-fixture \"$fixture_mode\"",
    ] {
        assert!(WORKFLOW.contains(contract), "workflow omitted {contract}");
    }

    for forbidden in [
        "pull_request_target:",
        "workflow_run:",
        "contents: write",
        "actions: write",
        "id-token: write",
        "secrets.",
        "continue-on-error:",
        "set +e",
        "|| true",
        "--allow-tool",
        "--allow-side-effects",
        "--allow-private-network",
        "--allow-cleartext-http",
        "--allow-credentials-to",
        "--bearer-token-env",
        "--header-env",
    ] {
        assert!(
            !WORKFLOW.contains(forbidden),
            "workflow unexpectedly contains {forbidden}"
        );
    }

    assert!(!WORKFLOW.contains("\n  push:"));
    assert!(!WORKFLOW.contains("docs/**"));
}

#[test]
fn every_external_action_is_immutable_and_checkout_drops_credentials() {
    let actions = WORKFLOW
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("uses: "))
        .filter(|action| !action.starts_with("./"))
        .collect::<Vec<_>>();

    assert_eq!(actions.len(), 2);
    for action in actions {
        let (name, revision) = action
            .split_once('@')
            .unwrap_or_else(|| panic!("action is not pinned: {action}"));
        let revision = revision
            .split_whitespace()
            .next()
            .expect("action revision should exist");
        assert!(!name.is_empty());
        assert_eq!(revision.len(), 40, "action is not commit-pinned: {action}");
        assert!(
            revision
                .chars()
                .all(|character| character.is_ascii_hexdigit()),
            "action is not commit-pinned: {action}"
        );
    }
    assert_eq!(WORKFLOW.matches("uses: actions/checkout@").count(), 1);
    assert_eq!(WORKFLOW.matches("persist-credentials: false").count(), 1);
}

#[test]
fn report_upload_runs_after_success_or_failure_without_changing_the_exit() {
    let diagnostic = WORKFLOW
        .find("- name: Diagnose the selected MCP server without calling tools")
        .expect("diagnostic step should exist");
    let verification = WORKFLOW
        .find("- name: Verify the safe report boundary")
        .expect("verification step should exist");
    let upload = WORKFLOW
        .find("- name: Upload the safe reports without masking the diagnostic exit")
        .expect("upload step should exist");
    assert!(diagnostic < verification && verification < upload);

    assert_eq!(WORKFLOW.matches("if: ${{ always() }}").count(), 2);
    for contract in [
        "name: mcp-doctor-preflight-${{ inputs.fixture || 'passing' }}",
        "artifacts/mcp-doctor/report.json",
        "artifacts/mcp-doctor/report.junit.xml",
        "if-no-files-found: error",
        "retention-days: 7",
    ] {
        assert!(WORKFLOW.contains(contract), "upload omitted {contract}");
    }
    assert!(WORKFLOW.contains("passing) fixture_mode=catalog-valid"));
    assert!(WORKFLOW.contains("diagnosed) fixture_mode=protocol-unsupported"));
}

#[test]
fn public_guidance_explains_copy_boundaries_artifacts_and_stable_exits() {
    for contract in [
        "least-permission preflight workflow",
        "synthetic and repository-owned",
        "literal arguments after `--`",
        "`pull_request.paths`",
        "immutable action commits, explicit",
        "`contents: read` permission",
        "Exit `1`, `2`, `3`, or",
        "`4` fails the diagnostic step",
        "uploaded successfully afterward",
        "grants no tool-call, side-effect, credential, private-network",
        "intentionally red",
    ] {
        assert!(AUTOMATION.contains(contract), "guidance omitted {contract}");
    }
}

#[test]
fn report_verifier_owns_the_fixed_negative_scan_without_rendering_values() {
    for contract in [
        "synthetic-private-revision-never-report-7f2c",
        "synthetic-private-ci-stderr-never-report-7f2c",
        "'/Users/'",
        "'/home/runner/'",
        "'C:\\Users\\'",
        "an mcp-doctor report crossed the safe publication boundary",
    ] {
        assert!(
            REPORT_VERIFIER.contains(contract),
            "report verifier omitted {contract}"
        );
    }
    for protected in [
        "synthetic-private-revision-never-report-7f2c",
        "synthetic-private-ci-stderr-never-report-7f2c",
    ] {
        assert!(!WORKFLOW.contains(protected));
        assert!(!AUTOMATION.contains(protected));
    }
}

#[test]
fn verifier_accepts_safe_reports_and_rejects_a_sentinel_without_echoing_it() {
    #[cfg(unix)]
    {
        use std::process::Command;

        let temporary = tempfile::tempdir().expect("temporary root should exist");
        let json = temporary.path().join("report.json");
        let junit = temporary.path().join("report.junit.xml");
        fs::write(&json, br#"{"schema_version":"mcp-doctor.report/v1"}"#)
            .expect("safe JSON should be writable");
        fs::write(&junit, b"<testsuites/>").expect("safe JUnit should be writable");

        let verifier = repository_root().join("scripts/verify-mcp-doctor-preflight-reports.sh");
        let accepted = Command::new(&verifier)
            .arg(&json)
            .arg(&junit)
            .output()
            .expect("safe verification should run");
        assert!(accepted.status.success());
        assert!(accepted.stdout.is_empty());
        assert!(accepted.stderr.is_empty());

        let sentinel = "synthetic-private-ci-stderr-never-report-7f2c";
        fs::write(&json, sentinel).expect("sentinel JSON should be writable");
        let rejected = Command::new(&verifier)
            .arg(&json)
            .arg(&junit)
            .output()
            .expect("negative verification should run");
        assert_eq!(rejected.status.code(), Some(1));
        assert!(rejected.stdout.is_empty());
        let stderr = String::from_utf8(rejected.stderr).expect("stderr should be UTF-8");
        assert!(stderr.contains("safe publication boundary"));
        assert!(!stderr.contains(sentinel));
    }
}

#[test]
fn workflow_references_existing_owned_files() {
    for path in [
        ".github/workflows/mcp-doctor-preflight.yml",
        "scripts/verify-mcp-doctor-preflight-reports.sh",
        "tests/fixtures/stdio_server.rs",
    ] {
        assert!(
            repository_root().join(Path::new(path)).is_file(),
            "missing {path}"
        );
    }
}
