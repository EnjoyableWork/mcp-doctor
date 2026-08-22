use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::process::{Command, Output};

const WORKFLOW: &str = include_str!("../.github/workflows/mcp-doctor-preflight.yml");
const AUTOMATION: &str = include_str!("../docs/automation.md");
const CAPABILITY_VERIFIER: &str =
    include_str!("../scripts/verify-mcp-doctor-preflight-capabilities.sh");
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
        "MCP_DOCTOR_VERSION: 0.4.0",
        "cargo install mcp-doctor",
        "--version \"=$MCP_DOCTOR_VERSION\"",
        "--locked",
        "mcp-doctor $MCP_DOCTOR_VERSION",
        "mcp-doctor\" capabilities",
        "--schema-version mcp-doctor.capabilities/v1",
        "verify-mcp-doctor-preflight-capabilities.sh",
        "mcp-doctor\" inspect",
        "--protocol-version 2026-07-28",
        "--json-report artifacts/mcp-doctor/report.json",
        "--junit-report artifacts/mcp-doctor/report.junit.xml",
        "--markdown-report artifacts/mcp-doctor/report.md",
        "--badge-report artifacts/mcp-doctor/badge.json",
        "./target/release/mcp-doctor-stdio-fixture \"$fixture_mode\"",
        "- incomplete",
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
fn capability_gate_precedes_target_and_reports_upload_for_every_diagnostic_exit() {
    let capability = WORKFLOW
        .find("- name: Verify the released passive reporting capabilities")
        .expect("capability step should exist");
    let diagnostic = WORKFLOW
        .find("- name: Diagnose the selected MCP server without calling tools")
        .expect("diagnostic step should exist");
    let verification = WORKFLOW
        .find("- name: Verify the safe report boundary")
        .expect("verification step should exist");
    let upload = WORKFLOW
        .find("- name: Upload the safe reports without masking the diagnostic exit")
        .expect("upload step should exist");
    assert!(capability < diagnostic && diagnostic < verification && verification < upload);

    assert_eq!(WORKFLOW.matches("if: ${{ always() }}").count(), 2);
    for contract in [
        "name: mcp-doctor-preflight-${{ inputs.fixture || 'passing' }}",
        "artifacts/mcp-doctor/report.json",
        "artifacts/mcp-doctor/report.junit.xml",
        "artifacts/mcp-doctor/report.md",
        "artifacts/mcp-doctor/badge.json",
        "if-no-files-found: error",
        "retention-days: 7",
    ] {
        assert!(WORKFLOW.contains(contract), "upload omitted {contract}");
    }
    assert!(WORKFLOW.contains("passing) fixture_mode=catalog-valid"));
    assert!(WORKFLOW.contains("diagnosed) fixture_mode=protocol-unsupported"));
    assert!(WORKFLOW.contains("incomplete) fixture_mode=schema-validator-work-limit"));
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
        "all four reports",
        "capability check verifies those compiled contracts before\nthe target process starts",
        "provider-native job conclusion remains the merge-enforcement authority",
        "provider-neutral public badge input",
        "starter neither publishes nor hosts it",
        "Private and air-gapped projects can retain\nonly native status",
        "grants no tool-call, side-effect, credential, private-network",
        "intentionally red",
    ] {
        assert!(AUTOMATION.contains(contract), "guidance omitted {contract}");
    }
}

#[test]
fn report_verifier_owns_the_fixed_negative_scan_without_rendering_values() {
    for contract in [
        "the four expected mcp-doctor reports were not published",
        "mcp-doctor.report/v1",
        "<testsuites ",
        "<!-- mcp-doctor.markdown/v1 -->",
        "mcp-doctor badge report disagrees with the fixed outcome mapping",
        "keys == [\"color\", \"label\", \"message\", \"schemaVersion\"]",
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
fn capability_verifier_accepts_only_the_exact_passive_four_report_contract() {
    for contract in [
        "mcp-doctor.capabilities/v1",
        ".activity == \"passive\"",
        ".artifact_reporters == [\"json\", \"junit\", \"markdown\", \"badge\"]",
        "mcp-doctor.markdown/v1",
        "mcp-doctor.badge/v1",
        "lacks the required passive report contracts",
    ] {
        assert!(
            CAPABILITY_VERIFIER.contains(contract),
            "capability verifier omitted {contract}"
        );
    }

    #[cfg(unix)]
    {
        let temporary = tempfile::tempdir().expect("temporary root should exist");
        let capabilities = temporary.path().join("capabilities.json");
        let verifier =
            repository_root().join("scripts/verify-mcp-doctor-preflight-capabilities.sh");
        let exact = serde_json::json!({
            "schema_version": "mcp-doctor.capabilities/v1",
            "schema_stability": "stable",
            "product": {"name": "mcp-doctor", "version": "0.4.0"},
            "commands": [{
                "name": "inspect",
                "activity": "passive",
                "artifact_reporters": ["json", "junit", "markdown", "badge"],
                "output_schema_versions": [
                    "mcp-doctor.report/v1",
                    "mcp-doctor.contract-snapshot/v1alpha1"
                ]
            }],
            "schema_versions": {
                "diagnostic_report": ["mcp-doctor.report/v1"],
                "markdown_report": ["mcp-doctor.markdown/v1"],
                "badge_report": ["mcp-doctor.badge/v1"]
            }
        });
        fs::write(&capabilities, serde_json::to_vec(&exact).unwrap())
            .expect("capability evidence should be writable");

        let accepted = Command::new(&verifier)
            .arg(&capabilities)
            .arg("0.4.0")
            .output()
            .expect("capability verification should run");
        assert!(accepted.status.success());
        assert!(accepted.stdout.is_empty());
        assert!(accepted.stderr.is_empty());

        let mut mismatch = exact;
        mismatch["commands"][0]["artifact_reporters"] =
            serde_json::json!(["json", "junit", "markdown"]);
        fs::write(&capabilities, serde_json::to_vec(&mismatch).unwrap())
            .expect("mismatched capability evidence should be writable");
        let rejected = Command::new(&verifier)
            .arg(&capabilities)
            .arg("0.4.0")
            .output()
            .expect("mismatched capability verification should run");
        assert_eq!(rejected.status.code(), Some(1));
        assert!(rejected.stdout.is_empty());
        let stderr = String::from_utf8(rejected.stderr).expect("stderr should be UTF-8");
        assert!(stderr.contains("lacks the required passive report contracts"));
        assert!(!stderr.contains("artifact_reporters"));
    }
}

#[test]
fn report_verifier_accepts_all_outcomes_and_rejects_mutated_artifacts() {
    #[cfg(unix)]
    {
        let temporary = tempfile::tempdir().expect("temporary root should exist");
        let verifier = repository_root().join("scripts/verify-mcp-doctor-preflight-reports.sh");

        for (outcome, exit, meaning, message, color) in [
            ("passed", 0, "success", "pass", "brightgreen"),
            ("failed", 1, "unsuccessful_result", "fail", "red"),
            (
                "incomplete",
                3,
                "incomplete_evidence",
                "incomplete",
                "lightgrey",
            ),
        ] {
            let reports = write_reports(temporary.path(), outcome, exit, meaning, message, color);
            let accepted = run_report_verifier(&verifier, &reports);
            assert!(accepted.status.success(), "{outcome}: {accepted:?}");
            assert!(accepted.stdout.is_empty());
            assert!(accepted.stderr.is_empty());
        }

        let reports = write_reports(
            temporary.path(),
            "passed",
            0,
            "success",
            "pass",
            "brightgreen",
        );
        fs::write(&reports[0], b"{").expect("malformed JSON should be writable");
        assert_report_rejected(&verifier, &reports, None);

        let reports = write_reports(temporary.path(), "passed", 0, "success", "fail", "red");
        assert_report_rejected(&verifier, &reports, None);

        let reports = write_reports(
            temporary.path(),
            "passed",
            0,
            "success",
            "pass",
            "brightgreen",
        );
        fs::write(
            &reports[3],
            br#"{"schemaVersion":1,"label":"mcp-doctor","message":"pass","color":"brightgreen","score":100}"#,
        )
        .expect("extra-field badge should be writable");
        assert_report_rejected(&verifier, &reports, None);

        let sentinel = "synthetic-private-ci-stderr-never-report-7f2c";
        let reports = write_reports(
            temporary.path(),
            "passed",
            0,
            "success",
            "pass",
            "brightgreen",
        );
        fs::write(
            &reports[0],
            format!(
                "{{\"schema_version\":\"mcp-doctor.report/v1\",\"schema_stability\":\"stable\",\"outcome\":\"passed\",\"exit_code\":0,\"canary\":\"{sentinel}\"}}"
            ),
        )
        .expect("sentinel JSON should be writable");
        assert_report_rejected(&verifier, &reports, Some(sentinel));
    }
}

#[cfg(unix)]
fn write_reports(
    root: &Path,
    outcome: &str,
    exit: i32,
    meaning: &str,
    message: &str,
    color: &str,
) -> [PathBuf; 4] {
    let json = root.join("report.json");
    let junit = root.join("report.junit.xml");
    let markdown = root.join("report.md");
    let badge = root.join("badge.json");
    fs::write(
        &json,
        format!(
            "{{\"schema_version\":\"mcp-doctor.report/v1\",\"schema_stability\":\"stable\",\"outcome\":\"{outcome}\",\"exit_code\":{exit}}}"
        ),
    )
    .expect("JSON report should be writable");
    fs::write(
        &junit,
        format!(
            "<?xml version=\"1.0\"?>\n<testsuites name=\"mcp-doctor\">\nreport_outcome={outcome}\nexit_code={exit}\n</testsuites>\n"
        ),
    )
    .expect("JUnit report should be writable");
    fs::write(
        &markdown,
        format!(
            "<!-- mcp-doctor.markdown/v1 -->\n# mcp-doctor report\n\n| Outcome | `{outcome}` |\n| Exit | `{exit}` (`{meaning}`) |\n"
        ),
    )
    .expect("Markdown report should be writable");
    fs::write(
        &badge,
        format!(
            "{{\"schemaVersion\":1,\"label\":\"mcp-doctor\",\"message\":\"{message}\",\"color\":\"{color}\"}}"
        ),
    )
    .expect("badge report should be writable");
    [json, junit, markdown, badge]
}

#[cfg(unix)]
fn run_report_verifier(verifier: &Path, reports: &[PathBuf; 4]) -> Output {
    Command::new(verifier)
        .args(reports)
        .output()
        .expect("report verification should run")
}

#[cfg(unix)]
fn assert_report_rejected(verifier: &Path, reports: &[PathBuf; 4], protected: Option<&str>) {
    let rejected = run_report_verifier(verifier, reports);
    assert_eq!(rejected.status.code(), Some(1));
    assert!(rejected.stdout.is_empty());
    let stderr = String::from_utf8(rejected.stderr).expect("stderr should be UTF-8");
    assert!(!stderr.is_empty());
    if let Some(protected) = protected {
        assert!(!stderr.contains(protected));
    }
}

#[test]
fn workflow_references_existing_owned_files() {
    for path in [
        ".github/workflows/mcp-doctor-preflight.yml",
        "scripts/verify-mcp-doctor-preflight-capabilities.sh",
        "scripts/verify-mcp-doctor-preflight-reports.sh",
        "tests/fixtures/stdio_server.rs",
    ] {
        assert!(
            repository_root().join(Path::new(path)).is_file(),
            "missing {path}"
        );
    }
}
