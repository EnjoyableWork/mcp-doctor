use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repository_file(path: impl AsRef<Path>) -> String {
    fs::read_to_string(repository_root().join(path))
        .expect("repository text file should be readable")
}

fn controls() -> Value {
    serde_json::from_str(&repository_file(".github/supply-chain-controls.json"))
        .expect("supply-chain controls should be valid JSON")
}

fn workflow_paths() -> Vec<PathBuf> {
    let mut paths = fs::read_dir(repository_root().join(".github/workflows"))
        .expect("workflow directory should be readable")
        .map(|entry| entry.expect("workflow entry should be readable").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "yml"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

#[test]
fn every_selected_action_is_closed_inventoried_and_commit_pinned() {
    let controls = controls();
    let actions = controls["actions"]
        .as_array()
        .expect("actions should be an array");
    assert_eq!(actions.len(), 8);

    let mut direct = BTreeMap::new();
    let mut nested = BTreeMap::new();
    for action in actions {
        let selection = action["selection"]
            .as_str()
            .expect("action selection should be a string");
        let uses = action["uses"]
            .as_str()
            .expect("action uses should be a string");
        let sha = action["sha"]
            .as_str()
            .expect("action SHA should be a string");
        let tag = action["tag"]
            .as_str()
            .expect("action tag should be a string");
        assert!(is_full_sha(sha), "{uses} must use a full lowercase SHA");
        assert!(
            !tag.is_empty(),
            "{uses} must retain its reviewed release tag"
        );
        assert!(
            action["license_files"]
                .as_array()
                .is_some_and(|files| !files.is_empty()),
            "{uses} must retain exact license evidence"
        );
        let inventory = (sha.to_owned(), tag.to_owned());
        match selection {
            "direct" => assert!(direct.insert(uses.to_owned(), inventory).is_none()),
            "nested" => {
                assert!(
                    action["selected_by"].as_str().is_some(),
                    "nested Action {uses} must name its selecting Action"
                );
                assert!(nested.insert(uses.to_owned(), inventory).is_none())
            }
            other => panic!("unsupported Action selection {other}"),
        }
    }

    assert_eq!(
        nested.keys().cloned().collect::<Vec<_>>(),
        ["actions/attest"]
    );
    let mut observed = BTreeSet::new();
    for path in workflow_paths() {
        let workflow = fs::read_to_string(&path).expect("workflow should be readable");
        assert!(
            !workflow.contains("EmbarkStudios/cargo-deny-action@"),
            "the checksum-free cargo-deny Action must not return"
        );
        for raw_line in workflow.lines() {
            let line = raw_line.trim_start();
            let Some(value) = line
                .strip_prefix("uses: ")
                .or_else(|| line.strip_prefix("- uses: "))
            else {
                continue;
            };
            assert!(
                !value.starts_with("./"),
                "local Actions require explicit policy"
            );
            let (selection, comment) = value
                .split_once('#')
                .expect("every Action pin should retain its reviewed tag comment");
            let (uses, sha) = selection
                .trim()
                .split_once('@')
                .expect("Action selection should contain @");
            assert!(is_full_sha(sha), "{uses} is not pinned by full commit SHA");
            let (expected_sha, expected_tag) = direct
                .get(uses)
                .unwrap_or_else(|| panic!("{uses} is not in the closed direct Action inventory"));
            assert_eq!(sha, expected_sha);
            assert_eq!(
                comment.split_whitespace().next(),
                Some(expected_tag.as_str()),
                "{uses} tag comment drifted from the reviewed inventory"
            );
            observed.insert(uses.to_owned());
        }
    }

    assert_eq!(
        observed,
        direct.keys().cloned().collect(),
        "every direct inventoried Action should be selected and no other Action may run"
    );
}

#[test]
fn pull_request_workflows_are_read_only_secretless_and_hosted() {
    let controls = controls();
    let checked_in = controls["workflow_inventory"]["checked_in"]
        .as_array()
        .expect("checked-in workflows should be an array")
        .iter()
        .map(|path| {
            path.as_str()
                .expect("workflow path should be a string")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();
    let observed_checked_in = workflow_paths()
        .iter()
        .map(|path| {
            path.strip_prefix(repository_root())
                .expect("workflow should be below repository root")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(checked_in, observed_checked_in);
    assert_eq!(
        controls["workflow_inventory"]["provider_managed"]
            .as_array()
            .expect("provider workflows should be an array")
            .iter()
            .map(|workflow| workflow["path"]
                .as_str()
                .expect("provider path should be a string"))
            .collect::<Vec<_>>(),
        [
            "dynamic/dependabot/dependabot-updates",
            "dynamic/github-code-scanning/codeql",
        ]
    );
    let expected_paths = controls["untrusted_workflows"]
        .as_array()
        .expect("untrusted workflows should be an array")
        .iter()
        .map(|workflow| {
            workflow["path"]
                .as_str()
                .expect("workflow path should be a string")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected_paths,
        BTreeSet::from([
            ".github/workflows/ci.yml".to_owned(),
            ".github/workflows/release-preflight.yml".to_owned(),
        ])
    );

    let forbidden = [
        "pull_request_target:",
        "workflow_run:",
        "issue_comment:",
        "secrets.",
        "environment:",
        "self-hosted",
        "contents: write",
        "actions: write",
        "attestations: write",
        "id-token: write",
        "packages: write",
        "allow-unsafe-pr-checkout",
        "github.event.pull_request",
        "github.head_ref",
        "github.token",
    ];

    for path in workflow_paths() {
        let relative = path
            .strip_prefix(repository_root())
            .expect("workflow should be below repository root")
            .to_string_lossy()
            .replace('\\', "/");
        let workflow = fs::read_to_string(&path).expect("workflow should be readable");
        assert!(
            !workflow.contains("pull_request_target:") && !workflow.contains("workflow_run:"),
            "no repository workflow may elevate untrusted code"
        );
        let is_pull_request = workflow.contains("on:\n  pull_request:");
        assert_eq!(
            is_pull_request,
            expected_paths.contains(&relative),
            "the closed untrusted-workflow inventory changed for {relative}"
        );
        if !is_pull_request {
            continue;
        }
        assert!(workflow.contains("permissions:\n  contents: read"));
        for value in forbidden {
            assert!(
                !workflow.contains(value),
                "untrusted workflow {relative} contains {value}"
            );
        }
        assert_eq!(
            workflow.matches("uses: actions/checkout@").count(),
            workflow.matches("persist-credentials: false").count(),
            "every checkout in {relative} should avoid persisted credentials"
        );
    }

    let preflight = repository_file(".github/workflows/release-preflight.yml");
    for explicit_empty_credential in [
        "github-token: \"\"",
        "token: \"\"",
        "brew-gh-api-token: \"\"",
    ] {
        assert!(
            preflight.contains(explicit_empty_credential),
            "release preflight should preserve {explicit_empty_credential}"
        );
    }
}

#[test]
fn dependabot_groups_version_and_security_proposals_without_merge_authority() {
    let dependabot = repository_file(".github/dependabot.yml");
    for contract in [
        "package-ecosystem: cargo",
        "cargo-version-updates:",
        "cargo-security-updates:",
        "package-ecosystem: github-actions",
        "github-actions-version-updates:",
        "github-actions-security-updates:",
        "applies-to: version-updates",
        "applies-to: security-updates",
        "rebase-strategy: auto",
    ] {
        assert!(
            dependabot.contains(contract),
            "Dependabot should preserve {contract}"
        );
    }
    assert_eq!(dependabot.matches("applies-to: version-updates").count(), 2);
    assert_eq!(
        dependabot.matches("applies-to: security-updates").count(),
        2
    );
    for forbidden in ["target-branch:", "registries:", "auto-merge", "secrets."] {
        assert!(
            !dependabot.contains(forbidden),
            "Dependabot must not contain {forbidden}"
        );
    }

    let contributing = repository_file("CONTRIBUTING.md");
    let template = repository_file(".github/pull_request_template.md");
    for review_dimension in [
        "release notes",
        "maintenance",
        "ownership/provenance",
        "selected features",
        "graph",
        "licenses",
        "advisories",
        "build-script",
        "Rust/platform",
        "behavior",
    ] {
        assert!(
            contributing.contains(review_dimension) || template.contains(review_dimension),
            "dependency review should preserve {review_dimension}"
        );
    }
    assert!(contributing.contains("Do not enable auto-merge"));
    assert!(template.contains("old and new exact identities"));
}

#[test]
fn direct_dependency_versions_features_and_scopes_require_reviewed_inventory() {
    let output = Command::new("cargo")
        .args(["metadata", "--locked", "--no-deps", "--format-version", "1"])
        .current_dir(repository_root())
        .output()
        .expect("locked Cargo metadata should execute");
    assert!(
        output.status.success(),
        "locked Cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Value =
        serde_json::from_slice(&output.stdout).expect("Cargo metadata should be valid JSON");
    let package = metadata["packages"]
        .as_array()
        .expect("metadata packages should be an array")
        .iter()
        .find(|package| package["name"] == "mcp-doctor")
        .expect("mcp-doctor package should exist");

    let mut observed = package["dependencies"]
        .as_array()
        .expect("package dependencies should be an array")
        .iter()
        .map(|dependency| {
            assert_eq!(
                dependency["source"],
                "registry+https://github.com/rust-lang/crates.io-index"
            );
            assert_eq!(dependency["optional"], false);
            assert!(dependency["rename"].is_null());
            assert!(dependency["target"].is_null());
            assert!(dependency["registry"].is_null());
            let mut features = dependency["features"]
                .as_array()
                .expect("dependency features should be an array")
                .clone();
            features.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
            json!({
                "name": dependency["name"],
                "scope": if dependency["kind"].is_null() {
                    "runtime"
                } else {
                    assert_eq!(dependency["kind"], "dev");
                    "development"
                },
                "version": dependency["req"]
                    .as_str()
                    .expect("dependency requirement should be a string")
                    .strip_prefix('=')
                    .expect("dependency requirement should remain exact"),
                "default_features": dependency["uses_default_features"],
                "features": features,
            })
        })
        .collect::<Vec<_>>();
    observed.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));

    let controls = controls();
    let mut expected = controls["direct_dependencies"]
        .as_array()
        .expect("direct dependency inventory should be an array")
        .clone();
    for dependency in &mut expected {
        dependency["features"]
            .as_array_mut()
            .expect("inventoried features should be an array")
            .sort_by(|left, right| left.as_str().cmp(&right.as_str()));
    }
    expected.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    assert_eq!(observed, expected);

    let project = repository_file("PROJECT.md");
    for dependency in expected {
        let name = dependency["name"]
            .as_str()
            .expect("name should be a string");
        let version = dependency["version"]
            .as_str()
            .expect("version should be a string");
        assert!(
            project.contains(&format!("`{name}` `={version}`")),
            "PROJECT.md should retain the dated {name} ={version} review"
        );
    }
}

#[test]
fn external_tool_and_live_audit_paths_are_digest_bounded_and_non_mutating() {
    let controls = controls();
    assert_eq!(
        controls["distribution_authentication"]["cargo_package"],
        "https://static.crates.io/crates/mcp-doctor/mcp-doctor-0.2.0.crate"
    );
    assert_eq!(
        controls["distribution_authentication"]["homebrew_source"],
        "https://github.com/EnjoyableWork/mcp-doctor/releases/download/v0.2.0/mcp-doctor-0.2.0.crate"
    );

    let installer = repository_file("scripts/install-cargo-deny.sh");
    for contract in [
        "deny_version=0.20.2",
        "x86_64-unknown-linux-musl",
        "9f12ed4c49936e09b48bf862b595cde2fe64fcbd9d74dfacac6131ca824c8d5f",
        "--proto '=https'",
        "--proto-redir '=https'",
        "--proxy ''",
        "--max-filesize 6000000",
        "cargo-deny archive layout is not the reviewed layout",
        "cargo-deny $deny_version",
    ] {
        assert!(
            installer.contains(contract),
            "installer should preserve {contract}"
        );
    }
    for forbidden in ["cargo install", "curl |", "set -x", "http://"] {
        assert!(
            !installer.contains(forbidden),
            "installer must not contain {forbidden}"
        );
    }

    let verifier = repository_file("scripts/verify-supply-chain-controls.sh");
    for contract in [
        "actions/permissions/selected-actions",
        "actions/workflows?per_page=100",
        "sha_pinning_required == true",
        "default_workflow_permissions == \"read\"",
        "fork-pr-contributor-approval",
        "allow_auto_merge == false",
        "actions/secrets?per_page=100",
        "verify-source-artifacts.sh",
        "git/ref/tags/$supply_tag",
        "commit.verification.verified == true",
        "gh release verify",
        "gh attestation verify",
        "--signer-workflow",
        "--source-digest",
        ".distribution_authentication.cargo_package",
        "homebrew_source",
        "homebrew_formula_sha256",
        "result=PASS",
    ] {
        assert!(
            verifier.contains(contract),
            "live verifier should preserve {contract}"
        );
    }
    for forbidden in ["set -x", "http://", "cargo publish", "gh release create"] {
        assert!(
            !verifier.contains(forbidden),
            "live verifier must not contain {forbidden}"
        );
    }
}

#[test]
fn project_records_mcpd_016_completion() {
    let project = repository_file("PROJECT.md");
    for contract in [
        "`MCPD-016` is Done and `MCPD-017` is Ready",
        "### Accepted dependency, automation, artifact, and distribution supply-chain contract",
        "`DEC-040` fixes the `MCPD-016` boundary.",
        ".github/supply-chain-controls.json",
        "Dependabot opens separate grouped weekly version and security proposals",
        "The canonical inventory closes that direct set at seven",
        "Only `CI` and `Release preflight` execute a pull request's code.",
        "The former full-SHA `cargo-deny-action` still fetched a mutable release executable",
        "There are no binary exceptions.",
        "Authenticate only canonical immutable `v0.2.0`",
        "At that pre-activation point, no",
        "Dependabot proposal existed for this repository",
        "pull/26#issuecomment-5268391783",
        "pull/27#issuecomment-5268400437",
        "pull/29",
        "40234363e8a1764498b524bc86c39afff0584355",
        "Several Node Actions execute generated JavaScript bundles",
        "both grouped proposals above",
        "proved read-only",
        "and a rejected write before closing unmerged",
        "### MCPD-016 completion evidence",
        "`MCPD-016` completed on 2026-08-12",
        "ea63855124cae11a0230aabc982c5c722b2154876133b7437e2c72a0a1b69ef5",
        "d11e8378999c057a74a18a83767179d220897897",
        "5cdc032336ca5e9cc2dba3c0052eff36be0fc83c",
        "31611427951",
        "31611427635",
        "31612642595/job/94168634038",
        "31612642612/job/94171302909",
        "31612643730",
        "4ba3f51a8f2ae443ec3f41c154556aa33ff56e0c",
        "31609790299/job/94157892254",
        "verified 111 reviewable regular UTF-8 source files",
        "without changing a published byte",
        "`MCPD-017` is Ready but has not begun",
        "| DEC-040 | Close dependency, Action, untrusted-workflow, source-artifact, and published-distribution maintenance under one reviewable supply-chain contract | Accepted |",
    ] {
        assert!(
            project.contains(contract),
            "PROJECT.md should preserve {contract}"
        );
    }
    for stale in [
        "`MCPD-016` is In progress",
        "| MCPD-016 | Harden dependency maintenance and the CI, artifact, and distribution supply chains | M4 | In progress |",
        "| MCPD-017 | Establish organization access, credential, ownership, and recovery policy | M4 | Proposed |",
    ] {
        assert!(
            !project.contains(stale),
            "PROJECT.md must not retain {stale}"
        );
    }
}

#[test]
#[cfg_attr(
    windows,
    ignore = "artifact rehearsal executes through Bash and a disposable Git repository"
)]
fn source_artifact_policy_rejects_executables_and_binary_artifacts() {
    let controls = controls();
    assert_eq!(controls["source_artifact_policy"]["text_encoding"], "UTF-8");
    assert_eq!(
        controls["source_artifact_policy"]["disallowed_ascii_controls"],
        true
    );
    assert_eq!(
        controls["source_artifact_policy"]["binary_exceptions"],
        json!([])
    );

    let output = Command::new("bash")
        .arg("scripts/rehearse-supply-chain-controls.sh")
        .current_dir(repository_root())
        .output()
        .expect("supply-chain artifact rehearsal should execute");
    assert!(
        output.status.success(),
        "supply-chain artifact rehearsal failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(
            "Supply-chain artifact negative exercises passed in a disposable repository."
        )
    );
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
