use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::process::{Command, Output};

const WORKFLOW: &str = include_str!("../.github/workflows/mcp-doctor.yml");
const COMMENT_WORKFLOW: &str = include_str!("../.github/workflows/mcp-doctor-comment.yml");
const README: &str = include_str!("../README.md");
const AUTOMATION: &str = include_str!("../docs/automation.md");
const CAPABILITY_VERIFIER: &str =
    include_str!("../scripts/verify-mcp-doctor-preflight-capabilities.sh");
const REPORT_VERIFIER: &str = include_str!("../scripts/verify-mcp-doctor-preflight-reports.sh");

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn starter_is_exact_passive_least_permission_and_copy_focused() {
    for contract in [
        "name: Enjoyable Work",
        "name: MCP Doctor",
        "pull_request:\n  workflow_dispatch:",
        "workflow_dispatch:",
        "Consumer adaptation: copy this workflow with its companion publisher and two",
        "ADAPT 1/2: replace this step with the project's deterministic server",
        "ADAPT 2/2: replace only the literal command and arguments after --.",
        "permissions:\n  contents: read",
        "cancel-in-progress: false",
        "runs-on: ubuntu-24.04",
        "timeout-minutes: 20",
        "permissions:\n      contents: read",
        "MCP_DOCTOR_VERSION: 0.4.0",
        "MCP_DOCTOR_TARGET: x86_64-unknown-linux-gnu",
        "MCP_DOCTOR_ARCHIVE_BYTES: 5548845",
        "MCP_DOCTOR_ARCHIVE_SHA256: f8ddc1eb0d1cc9f8ed6ab186109ed4d881fea181c5e6896b029e535ae7ecfba6",
        "MCP_DOCTOR_BINARY_BYTES: 17026648",
        "name: MCP Doctor",
        "name: Download the exact released mcp-doctor binary",
        "--proto '=https'",
        "--tlsv1.2",
        "--connect-timeout 10",
        "--max-time 120",
        "--max-filesize \"$MCP_DOCTOR_ARCHIVE_BYTES\"",
        "releases/download/v$MCP_DOCTOR_VERSION/$archive_name",
        "test \"$archive_bytes\" -eq \"$MCP_DOCTOR_ARCHIVE_BYTES\"",
        "sha256sum --check --strict -",
        "$'Cargo.lock\\nLICENSE\\nREADME.md\\nmcp-doctor'",
        "--no-same-owner",
        "--no-same-permissions",
        "test ! -L \"$binary\"",
        "test \"$binary_bytes\" -eq \"$MCP_DOCTOR_BINARY_BYTES\"",
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
        "./target/release/mcp-doctor-stdio-fixture catalog-valid",
        "name: mcp-doctor-reports",
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
        "cargo install mcp-doctor",
        "--retry",
        "--allow-tool",
        "--allow-side-effects",
        "--allow-private-network",
        "--allow-cleartext-http",
        "--allow-credentials-to",
        "--bearer-token-env",
        "--header-env",
        "\n    paths:",
        "inputs.fixture",
        "PREFLIGHT_FIXTURE",
        "fixture_mode",
        "mcp-doctor-reports-${{",
    ] {
        assert!(
            !WORKFLOW.contains(forbidden),
            "workflow unexpectedly contains {forbidden}"
        );
    }

    assert!(!WORKFLOW.contains("\n  push:"));
}

#[test]
fn sticky_comment_publisher_is_default_branch_bounded_and_never_executes_pr_code() {
    for contract in [
        "name: MCP Doctor comment",
        "workflow_run:\n    workflows:\n      - Enjoyable Work\n    types:\n      - completed",
        "permissions: {}",
        "cancel-in-progress: false",
        "github.event.workflow_run.event == 'pull_request'",
        "runs-on: ubuntu-24.04",
        "timeout-minutes: 5",
        "actions: read",
        "contents: read",
        "pull-requests: write",
        "EXPECTED_WORKFLOW_PATH: .github/workflows/mcp-doctor.yml",
        "EXPECTED_WORKFLOW_NAME: Enjoyable Work",
        ".event == \"pull_request\"",
        ".status == \"completed\"",
        ".head_repository.id == $repository_id",
        ".pull_requests | length) == 1",
        ".pull_requests[0].base.repo.id == $repository_id",
        ".pull_requests[0].head.repo.id == $repository_id",
        ".state == \"open\"",
        ".base.ref == $default_branch",
        ".head.repo.id == $repository_id",
        ".head.sha == $head_sha",
        "repos/$repository/contents/$contract_path?ref=$head_sha",
        "repos/$repository/contents/$contract_path?ref=$publisher_sha",
        "scripts/verify-mcp-doctor-preflight-capabilities.sh",
        "scripts/verify-mcp-doctor-preflight-reports.sh",
        "contract_matches=false",
        "head_sha=$head_sha&per_page=100",
        "mcp-doctor-comment.json",
        ".size_in_bytes <= 4096",
        ".workflow_run.repository_id == $repository_id",
        ".workflow_run.head_repository_id == $repository_id",
        ".workflow_run.head_sha == $head_sha",
        "skip-decompress: true",
        "id: download_descriptor",
        "DOWNLOAD_OUTCOME: ${{ steps.download_descriptor.outcome }}",
        "\"$DOWNLOAD_OUTCOME\" == success",
        "keys == [\"outcome\", \"summary\", \"version\"]",
        "(.summary.skipped - .summary.required_skipped) <= .summary.optional",
        "<!-- mcp-doctor:pr-comment:v1 -->",
        "## 🩺 MCP Doctor",
        "✅ **Passed**",
        "❌ **Failed**",
        "⚪ **Incomplete**",
        "⚠️ **Summary withheld**",
        "could not be verified against the trusted default branch",
        "⚠️ **No structurally validated summary**",
        "**Mode: Passive** · Inspects the server without calling tools.",
        "CI presentation, not certification",
        "[View full check]",
        "[Explore MCP Doctor modes]",
        "docs/automation.md#mcp-doctor-ci-modes",
        "Add MCP Doctor to another project",
        "(${#comment_body} <= 49152)",
        "(.body | startswith($marker + \"\\n\"))",
        ".user.login == \"github-actions[bot]\"",
        ".user.type == \"Bot\"",
        ".user.id == 41898282",
        "type == \"array\" and length < 100",
        "issues/$PR_NUMBER/comments",
        "issues/comments/${comment_ids[0]}",
        "multiple owned MCP Doctor comments prevent a safe update",
        "steps.render.outcome == 'success'",
        "steps.render.outcome != 'success' || steps.render.outputs.verified != 'true'",
    ] {
        assert!(
            COMMENT_WORKFLOW.contains(contract),
            "comment publisher omitted {contract}"
        );
    }

    for forbidden in [
        "pull_request_target:",
        "\n  pull_request:",
        "contents: write",
        "actions: write",
        "id-token: write",
        "secrets.",
        "uses: actions/checkout@",
        "persist-credentials:",
        "report.md",
        "eval ",
        "source ",
        "bash -c",
        "continue-on-error:",
        "|| true",
    ] {
        assert!(
            !COMMENT_WORKFLOW.contains(forbidden),
            "comment publisher unexpectedly contains {forbidden}"
        );
    }
}

#[test]
fn sticky_comment_publisher_noops_forks_and_revalidates_before_every_mutation() {
    let context = COMMENT_WORKFLOW
        .find("- name: Validate the completed run and current pull request")
        .expect("context step should exist");
    let contract = COMMENT_WORKFLOW
        .find("contract_matches=true")
        .expect("contract comparison should exist");
    let download = COMMENT_WORKFLOW
        .find("- name: Download the exact bounded comment descriptor")
        .expect("descriptor download should exist");
    let render = COMMENT_WORKFLOW
        .find("- name: Render the fixed Markdown comment")
        .expect("comment renderer should exist");
    let update = COMMENT_WORKFLOW
        .find("- name: Create or update the MCP Doctor comment")
        .expect("comment mutation should exist");
    let fail_closed = COMMENT_WORKFLOW
        .find("- name: Fail closed when the latest report was not verified")
        .expect("fail-closed step should exist");
    assert!(context < contract && contract < download && download < render && render < update);

    let context_step = &COMMENT_WORKFLOW[context..download];
    assert_eq!(
        context_step
            .matches("printf 'publish=false\\n' >>\"$GITHUB_OUTPUT\"")
            .count(),
        3,
        "fork, stale-head, and superseded-run paths should be clean no-ops"
    );
    let latest = context_step
        .find("actions/workflows/$workflow_id/runs?event=pull_request")
        .expect("initial latest-run check should exist");
    assert!(
        latest < context_step.find("contract_matches=true").unwrap(),
        "latest-run validation must not depend on producer-contract matching"
    );

    let download_step = &COMMENT_WORKFLOW[download..render];
    assert!(download_step.contains("id: download_descriptor"));
    let render_step = &COMMENT_WORKFLOW[render..update];
    assert!(render_step.contains("if: ${{ always() && steps.context.outputs.publish == 'true' }}"));
    assert!(render_step.contains("DOWNLOAD_OUTCOME: ${{ steps.download_descriptor.outcome }}"));
    assert!(render_step.contains("\"$DOWNLOAD_OUTCOME\" == success"));

    let update_step = &COMMENT_WORKFLOW[update..fail_closed];
    assert!(update_step.contains(
        "if: ${{ always() && steps.context.outputs.publish == 'true' && steps.render.outcome == 'success' }}"
    ));
    let latest_before_mutation = update_step
        .find("actions/workflows/$EXPECTED_WORKFLOW_ID/runs?event=pull_request")
        .expect("latest run should be revalidated immediately before mutation");
    let comment_history = update_step
        .find("issues/$PR_NUMBER/comments?per_page=100&page=1")
        .expect("comment history request should exist");
    assert!(latest_before_mutation < comment_history);

    let fail_step = &COMMENT_WORKFLOW[fail_closed..];
    assert!(fail_step.contains("if: ${{ always()"));
    assert!(fail_step.contains("steps.render.outcome != 'success'"));
    assert!(fail_step.contains("steps.render.outputs.verified != 'true'"));
}

#[test]
fn producer_and_publisher_reject_impossible_optional_skip_counts() {
    assert!(WORKFLOW.contains(
        "($report.summary.skipped - $report.summary.required_skipped) <= $report.summary.optional"
    ));
    assert!(
        COMMENT_WORKFLOW
            .contains("(.summary.skipped - .summary.required_skipped) <= .summary.optional")
    );
}

#[test]
fn every_external_action_is_immutable_and_checkout_drops_credentials() {
    let actions = [WORKFLOW, COMMENT_WORKFLOW]
        .into_iter()
        .flat_map(str::lines)
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("uses: "))
        .filter(|action| !action.starts_with("./"))
        .collect::<Vec<_>>();

    assert_eq!(actions.len(), 4);
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
    assert_eq!(
        WORKFLOW.matches("uses: actions/upload-artifact@").count(),
        2
    );
    assert_eq!(
        COMMENT_WORKFLOW
            .matches("uses: actions/download-artifact@")
            .count(),
        1
    );
    assert!(!COMMENT_WORKFLOW.contains("uses: actions/checkout@"));
}

#[test]
fn capability_gate_precedes_target_and_verified_publication_for_every_diagnostic_exit() {
    let capability = WORKFLOW
        .find("- name: Verify the released passive reporting capabilities")
        .expect("capability step should exist");
    let diagnostic = WORKFLOW
        .find("- name: Diagnose the MCP server without calling tools")
        .expect("diagnostic step should exist");
    let verification = WORKFLOW
        .find("- name: Verify the safe report boundary")
        .expect("verification step should exist");
    let summary = WORKFLOW
        .find("- name: Add the verified diagnosis to the job summary")
        .expect("summary step should exist");
    let descriptor = WORKFLOW
        .find("- name: Prepare the bounded PR comment descriptor")
        .expect("comment descriptor step should exist");
    let upload = WORKFLOW
        .find("- name: Upload the safe reports without masking the diagnostic exit")
        .expect("upload step should exist");
    assert!(
        capability < diagnostic
            && diagnostic < verification
            && verification < summary
            && summary < descriptor
            && descriptor < upload
    );

    assert_eq!(WORKFLOW.matches("if: ${{ always() }}").count(), 1);
    assert_eq!(
        WORKFLOW
            .matches("if: ${{ always() && steps.verify_reports.outcome == 'success' }}")
            .count(),
        4
    );
    assert!(WORKFLOW[verification..summary].contains("id: verify_reports"));
    assert!(WORKFLOW[summary..upload].contains("cat artifacts/mcp-doctor/report.md"));
    assert!(WORKFLOW[summary..upload].contains(">>\"$GITHUB_STEP_SUMMARY\""));
    assert!(WORKFLOW[summary..upload].contains("Add MCP Doctor to another project"));
    assert!(WORKFLOW[descriptor..upload].contains("mcp-doctor-comment.json"));
    assert!(WORKFLOW[descriptor..upload].contains("(${#descriptor} <= 2048)"));
    assert!(
        WORKFLOW[upload..]
            .contains("if: ${{ always() && steps.verify_reports.outcome == 'success' }}")
    );
    for contract in [
        "name: mcp-doctor-reports",
        "artifacts/mcp-doctor/report.json",
        "artifacts/mcp-doctor/report.junit.xml",
        "artifacts/mcp-doctor/report.md",
        "artifacts/mcp-doctor/badge.json",
        "if-no-files-found: error",
        "retention-days: 7",
        "path: artifacts/mcp-doctor/mcp-doctor-comment.json",
        "archive: false",
        "retention-days: 1",
    ] {
        assert!(WORKFLOW.contains(contract), "upload omitted {contract}");
    }
    assert!(!WORKFLOW.contains("inputs.fixture"));
    assert!(!WORKFLOW.contains("fixture_mode"));
}

#[test]
fn public_guidance_explains_copy_boundaries_artifacts_and_stable_exits() {
    assert!(README.contains(
        "Add an MCP Doctor check and sticky GitHub PR comment | [GitHub Actions starter](docs/automation.md#github-actions-starter)"
    ));
    assert!(README.contains(
        "Compare Passive, Standard, and Full CI coverage | [MCP Doctor CI modes](docs/automation.md#mcp-doctor-ci-modes)"
    ));

    for contract in [
        "least-permission GitHub Actions starter",
        "Copy all four files to the same paths",
        "The two workflow files are separate by design",
        "only workflow given\ncomment-write permission",
        "A\nsingle `pull_request` workflow would put the write token beside the proposed\nbuild and server",
        "Keep both files for this\nYAML-only integration",
        "different architecture rather than a safe\none-file equivalent",
        "**Enjoyable Work / MCP Doctor**",
        "synthetic and repository-owned",
        "literal executable and arguments after `--`",
        "only two consumer-specific blocks with `ADAPT` comments",
        "replace `ADAPT 1/2` with that repository's deterministic runtime setup",
        "replace only the literal executable and arguments after `--` at `ADAPT 2/2`",
        "starter deliberately runs on every pull request",
        "Commit all four files together",
        "no sticky comment\nis expected",
        "next pull\nrequest receives both **Enjoyable Work / MCP Doctor** and the sticky comment",
        "immutable action commits, explicit",
        "`contents: read` permission",
        "verified-only summary and upload conditions",
        "If report verification fails, neither the Markdown summary nor any artifact is\npublished",
        "diagnostic exits `1` or `3`",
        "reports are still verified, summarized, and uploaded",
        "failure exits `2` or `4`",
        "verifier blocks both carriers",
        "all four reports",
        "capability check verifies those compiled contracts before\nthe target process starts",
        "separate `workflow_run` workflow",
        "never checks out\nor executes pull-request code",
        "receives only `actions: read` and\n`contents: read` plus `pull-requests: write`",
        "associated same-repository pull request",
        "same Git blob identities as the trusted",
        "default-branch files; an unavailable or mismatched contract",
        "downloads it without archive extraction",
        "creates or updates one `github-actions[bot]` comment",
        "## 🩺 MCP Doctor",
        "**Mode: Passive**",
        "Explore MCP Doctor modes",
        "Add MCP Doctor to another project",
        "A later run updates the same comment instead of adding another one",
        "**No\nstructurally validated summary**",
        "**Summary withheld**",
        "publisher never copies `report.md`",
        "not an\nindependent attestation, certification, security result, conformance claim, or\nmerge authority",
        "provider-native job conclusion remains the merge-enforcement authority",
        "### MCP Doctor CI modes",
        "human-facing CI\ncoverage labels, not values accepted by an `mcp-doctor --mode` option",
        "| **Passive** | `inspect` |",
        "| **Standard** | Passive plus one or more reviewed `check` scenarios |",
        "| **Full** | Standard plus targeted `break` and `reject` runs |",
        "There is intentionally no generic one-line switch",
        "**Full** never means “call every tool”",
        "provider-neutral public badge input",
        "starter neither publishes nor hosts it",
        "Private and air-gapped projects can retain\nonly native status",
        "Public and private repositories",
        "identical in public\nand private GitHub.com repositories",
        "Cannot be accessed externally",
        "included minutes and storage",
        "Product-led discovery",
        "support internal adoption",
        "diagnostic workflow requests only `contents: read` and no secrets",
        "Fork pull-request runs keep that\nrestricted posture",
        "does not comment on fork pull requests",
        "not differences in the `mcp-doctor` diagnostic",
        "trusted default-branch event",
        "workflow result, not a broader assurance claim",
        "reviewed default-branch `push` trigger",
        "actions/workflows/mcp-doctor.yml/badge.svg?branch=DEFAULT_BRANCH&event=push",
        "synthetic server fixture",
        "grants no tool-call, side-effect, credential, private-network",
        "A manual dispatch repeats the same passing synthetic target",
        "failed and\nincomplete diagnostics retain their non-success exits",
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
        ".github/workflows/mcp-doctor.yml",
        ".github/workflows/mcp-doctor-comment.yml",
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
