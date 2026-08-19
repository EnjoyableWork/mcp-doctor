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
    assert_eq!(actions.len(), 7);

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
        assert!(
            !workflow.contains("anchore/sbom-action@"),
            "the indirectly acquired Syft Action must not return"
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
            ".github/workflows/mcp-doctor-preflight.yml".to_owned(),
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
    for explicit_empty_credential in ["token: \"\"", "brew-gh-api-token: \"\""] {
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
}

#[test]
fn duplicate_dependency_exceptions_remain_exact_and_reviewed() {
    let deny = repository_file("deny.toml");
    assert!(
        deny.contains("[bans]\nmultiple-versions = \"deny\""),
        "the repository-wide duplicate-version ban must remain enabled"
    );
    assert_eq!(
        deny.matches("base64@").count(),
        1,
        "the base64 transition must have one exact exception"
    );
    assert!(deny.contains(
        "base64@0.22.1\", reason = \"reqwest 0.13.4 and hyper-util 0.1.20 retain base64 0.22"
    ));
    assert!(deny.contains(
        "remove this exact transition when every selected upstream converges or a relevant advisory changes the balance"
    ));
    assert!(
        deny.contains("skip-tree = []"),
        "a transitive subtree must not bypass the duplicate-version review"
    );
}

#[test]
fn external_tool_and_live_audit_paths_are_digest_bounded_and_non_mutating() {
    let controls = controls();
    assert_eq!(controls["reviewed_on"], "2026-08-18");
    assert_eq!(
        controls["distribution_authentication"]["cargo_package"],
        "https://static.crates.io/crates/mcp-doctor/mcp-doctor-0.3.0.crate"
    );
    assert_eq!(
        controls["distribution_authentication"]["homebrew_source"],
        "https://github.com/EnjoyableWork/mcp-doctor/releases/download/v0.3.0/mcp-doctor-0.3.0.crate"
    );
    assert_eq!(
        controls["distribution_authentication"]["homebrew_commit_scope"],
        "immutable_historical_handoff"
    );
    assert!(
        controls["limitations"]
            .as_array()
            .expect("limitations should be an array")
            .iter()
            .any(|limitation| limitation.as_str().is_some_and(|value| {
                value.contains("rolling tap main is a separate release-channel control")
            }))
    );

    let deny_installer = repository_file("scripts/install-cargo-deny.sh");
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
            deny_installer.contains(contract),
            "cargo-deny installer should preserve {contract}"
        );
    }
    for forbidden in ["cargo install", "curl |", "set -x", "http://"] {
        assert!(
            !deny_installer.contains(forbidden),
            "cargo-deny installer must not contain {forbidden}"
        );
    }

    let syft_installer = repository_file("scripts/install-syft.sh");
    for contract in [
        "syft_max_attempts=3",
        "syft_attempt_max_seconds=20",
        "syft_retry_delay_seconds=1",
        "($tools[0] as $tool |",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "408 | 429 | 500 | 502 | 503 | 504",
        "6 | 7 | 18 | 28 | 52 | 55 | 56 | 92",
        "--proto '=https'",
        "--proto-redir '=https'",
        "--proxy ''",
        "-u SSL_CERT_DIR -u SSL_CERT_FILE -u SSLKEYLOGFILE",
        "--retry 0",
        "--max-filesize \"$syft_bytes\"",
        "000 | 200 | 408 | 429 | 500 | 502 | 503 | 504",
        "env -u GZIP -u TAR_OPTIONS",
        "COPYFILE_DISABLE=1",
        "Syft archive digest does not match the reviewed value",
        "Syft archive layout is not the reviewed layout",
        "installed Syft did not report the reviewed version and platform",
    ] {
        assert!(
            syft_installer.contains(contract),
            "Syft installer should preserve {contract}"
        );
    }
    for forbidden in [
        "anchore/sbom-action",
        "raw.githubusercontent.com",
        "--retry-all-errors",
        "curl |",
        "set -x",
        "http://",
    ] {
        assert!(
            !syft_installer.contains(forbidden),
            "Syft installer must not contain {forbidden}"
        );
    }

    let generator = repository_file("scripts/generate-release-sbom.sh");
    for contract in [
        "install-syft.sh",
        "SYFT_CHECK_FOR_APP_UPDATE=false",
        "GOMAXPROCS=\"$sbom_generation_max_processors\"",
        "sbom_input_max_bytes=50000000",
        "sbom_output_max_bytes=10000000",
        "sbom_generation_max_seconds=120",
        "vcpkg-allow-git-clone: false",
        "search-local-mod-cache-licenses: false",
        "search-remote-licenses: false",
        "use-maven-local-repository: false",
        "use-network: false",
        "scan \"file:$sbom_input\"",
        "--output spdx-json",
        "sbom_config=\"$sbom_temp_root/syft.yaml\"",
        "--config \"$sbom_config\"",
        "2>\"$sbom_temporary_stderr\"",
    ] {
        assert!(
            generator.contains(contract),
            "SBOM generator should preserve {contract}"
        );
    }

    let syft = controls["standalone_tools"]
        .as_array()
        .expect("standalone tools should be an array")
        .iter()
        .find(|tool| tool["name"] == "syft")
        .expect("Syft should be a standalone tool");
    assert_eq!(syft["version"], "1.51.0");
    assert_eq!(syft["repository"], "anchore/syft");
    assert_eq!(
        syft["tag_object"],
        "57260929138ad516dd4999a5cc43b4a295d2461f"
    );
    assert_eq!(syft["tag_verified"], false);
    assert_eq!(
        syft["source_commit"],
        "2293641e3bd628a01bb37639318d62c0ebe89b39"
    );
    assert_eq!(syft["source_commit_verified"], true);
    assert_eq!(syft["release_immutable"], true);
    assert_eq!(syft["latest_release_required"], true);
    assert_eq!(
        syft["assets"]
            .as_array()
            .expect("Syft assets should be an array")
            .iter()
            .map(|asset| (
                asset["target"].as_str().expect("target should be a string"),
                asset["bytes"].as_u64().expect("bytes should be an integer"),
                asset["sha256"].as_str().expect("digest should be a string"),
            ))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            (
                "aarch64-unknown-linux-gnu",
                26_261_269,
                "6c0466811541ea03add5213a60a1562f0851e4c0b0ecfdee1a694a9455285900",
            ),
            (
                "x86_64-unknown-linux-gnu",
                28_743_977,
                "2a2e837a2c8d59ec9af5472ee22d3b04ee463c4e44476ecf993fd1e5ab6ebc7f",
            ),
        ])
    );
    assert!(
        controls["limitations"]
            .as_array()
            .expect("limitations should be an array")
            .iter()
            .any(|limitation| limitation
                .as_str()
                .is_some_and(|value| value.contains("prebuilt 276-package Go release graph")))
    );

    let verifier = repository_file("scripts/verify-supply-chain-controls.sh");
    for contract in [
        "actions/permissions/selected-actions",
        "actions/workflows?per_page=100",
        "sha_pinning_required == true",
        "default_workflow_permissions == \"read\"",
        "fork-pr-contributor-approval",
        "verify-read-only-repository-settings.sh",
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
        "verify-historical-homebrew-formula.sh",
        ".standalone_tools[]",
        ".immutable == $immutable",
        "releases/latest",
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

    let historical_formula_verifier =
        repository_file("scripts/verify-historical-homebrew-formula.sh");
    assert!(
        historical_formula_verifier
            .contains("contents/$historical_formula_path?ref=$historical_tap_commit")
    );
    assert!(!historical_formula_verifier.contains("commits/main"));
    assert!(!verifier.contains("repos/$supply_tap_repository/commits/main"));
    assert!(!verifier.contains("allow_auto_merge"));

    let readonly_repository_verifier =
        repository_file("scripts/verify-read-only-repository-settings.sh");
    for contract in [
        "gh api graphql",
        "query(\\$owner: String!, \\$name: String!)",
        "-F \"owner=$readonly_owner\"",
        "-F \"name=$readonly_name\"",
        "nameWithOwner",
        "autoMergeAllowed",
        "65536",
    ] {
        assert!(
            readonly_repository_verifier.contains(contract),
            "read-only repository verifier should preserve {contract}"
        );
    }
    for forbidden in ["mutation", "contents:write", "contents: write", "PATCH"] {
        assert!(
            !readonly_repository_verifier.contains(forbidden),
            "read-only repository verifier must not contain {forbidden}"
        );
    }
}

#[test]
fn release_runbook_separates_historical_and_rolling_homebrew_state() {
    let release = repository_file("docs/release.md");
    let normalized_release = release.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(normalized_release.contains("separate historical-evidence"));
    assert!(normalized_release.contains("does not require rolling `homebrew-tap/main` to remain"));
    assert!(normalized_release.contains("Neither boundary may substitute for the other"));
    assert!(normalized_release.contains("read-only GraphQL repository field"));
    assert!(normalized_release.contains("must not gain `Contents: write`"));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "Syft acquisition rehearsal executes through Bash with POSIX fixtures"
)]
fn syft_acquisition_and_generation_fail_closed_offline() {
    let output = Command::new("bash")
        .arg("scripts/rehearse-syft-acquisition.sh")
        .current_dir(repository_root())
        .output()
        .expect("Syft acquisition rehearsal should execute");
    assert!(
        output.status.success(),
        "Syft acquisition rehearsal failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("Syft acquisition and SBOM generation rehearsals passed offline.")
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Syft rehearsal case failed:"));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "supply-chain rehearsal executes through Bash and disposable fixtures"
)]
fn supply_chain_rehearsal_rejects_artifacts_and_historical_formula_drift() {
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
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("Historical Homebrew evidence remains strict after rolling tap advancement.")
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "Read-only repository settings remain verifiable without contents write access."
    ));
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
