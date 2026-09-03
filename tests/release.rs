use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const CANDIDATE_RELEASE_VERSION: &str = "0.4.2";
const LINUX_TARGETS: [&str; 2] = ["aarch64-unknown-linux-gnu", "x86_64-unknown-linux-gnu"];
const SOURCE_TARGETS: [&str; 4] = [
    "aarch64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
];

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repository_file(path: &str) -> String {
    fs::read_to_string(repository_root().join(path))
        .unwrap_or_else(|error| panic!("could not read {path}: {error}"))
}

fn assert_actions_are_commit_pinned(workflow: &str) {
    let actions = workflow
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("uses: "))
        .filter(|action| !action.starts_with("./"))
        .collect::<Vec<_>>();

    assert!(
        !actions.is_empty(),
        "workflow should use at least one action"
    );
    for action in actions {
        let (_, revision) = action
            .split_once('@')
            .unwrap_or_else(|| panic!("action is missing a revision: {action}"));
        let revision = revision
            .split_whitespace()
            .next()
            .expect("an action revision should not be empty");
        assert!(
            revision.len() == 40
                && revision
                    .chars()
                    .all(|character| character.is_ascii_hexdigit()),
            "action must use a full immutable commit SHA: {action}"
        );
    }
}

#[test]
fn release_identity_and_toolchain_are_exact() {
    let manifest = repository_file("Cargo.toml");
    let toolchain = repository_file("rust-toolchain.toml");

    for contract in [
        "name = \"mcp-doctor\"",
        "version = \"0.4.2\"",
        "publish = [\"crates-io\"]",
        "repository = \"https://github.com/EnjoyableWork/mcp-doctor\"",
        "\"/.bestpractices.json\"",
        "\"/.agents/skills/**\"",
        "\"/.github/assurance-controls.json\"",
        "\"/.github/community-license-controls.json\"",
        "\"/.github/organization-controls.json\"",
        "\"/.github/security-controls.json\"",
        "\"/.github/rulesets/**\"",
        "\"/.github/workflows/*.yml\"",
        "\"/schemas/**\"",
        "\"/scripts/**\"",
        "\"/docs/**\"",
    ] {
        assert!(
            manifest.contains(contract),
            "manifest should preserve {contract}"
        );
    }
    assert!(toolchain.contains("channel = \"1.97.1\""));
    assert!(!toolchain.contains("channel = \"stable\""));
}

#[test]
fn live_status_guidance_preserves_stream_evidence_and_duration_scope() {
    for (path, contracts) in [
        (
            "docs/commands.md",
            &[
                "--status plain",
                "--status jsonl",
                "mcp-doctor.status/v1",
                "Status always goes to stderr",
                "is liveness context, never diagnostic evidence",
            ][..],
        ),
        (
            "docs/automation.md",
            &[
                ">mcp-doctor-report.json 2>mcp-doctor-status.jsonl",
                ".diagnostic_time_ceiling_profiles",
                "whole_process_exit_guarantee",
                "Keep the streams separate",
            ][..],
        ),
        (
            "docs/safety.md",
            &[
                "## Status-channel safety",
                "never retains or renders endpoints",
                "Status is capped at 512 bytes per event, 128 events, and 65,536 aggregate",
                "exit `4`",
            ][..],
        ),
        (
            "docs/agents.md",
            &[
                "## Observe a direct CLI preflight",
                "Parse stderr one complete",
                "Do not diagnose from status",
                "whole_process_exit_guarantee",
            ][..],
        ),
    ] {
        let document = repository_file(path);
        for contract in contracts {
            assert!(document.contains(contract), "{path} omitted {contract}");
        }
    }
}

#[test]
fn preflight_is_secretless_nonpublishing_and_covers_every_source_host() {
    let workflow = repository_file(".github/workflows/release-preflight.yml");

    for target in SOURCE_TARGETS {
        assert!(workflow.contains(target), "preflight should cover {target}");
    }
    for contract in [
        "cargo package --locked",
        "scripts/generate-release-channels.sh",
        "scripts/package-agent-skill.sh",
        "scripts/package-release.sh",
        "scripts/smoke-installed.sh",
        "scripts/smoke-installed.ps1",
        "scripts/smoke-archive.sh",
        "scripts/verify-release-assets.sh",
        "scripts/verify-agent-skill.sh",
        "scripts/verify-published-release.sh",
        "scripts/rehearse-release-handoffs.sh",
        "synthetic-rehearsal",
        "brew install --build-from-source",
        "scripts/generate-release-sbom.sh",
        "retention-days: 1",
    ] {
        assert!(
            workflow.contains(contract),
            "preflight should enforce {contract}"
        );
    }
    assert!(!workflow.contains("secrets."));
    assert!(!workflow.contains("contents: write"));
    assert!(!workflow.contains("id-token: write"));
    assert!(!workflow.contains("attest-build-provenance"));
    assert_actions_are_commit_pinned(&workflow);
}

#[test]
fn future_tag_workflow_preserves_release_proof_before_oidc_publication() {
    let workflow = repository_file(".github/workflows/release.yml");

    for target in LINUX_TARGETS {
        assert!(workflow.contains(target));
    }
    for excluded_target in [
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
        "aarch64-pc-windows-msvc",
        "x86_64-pc-windows-msvc",
    ] {
        assert!(
            !workflow.contains(excluded_target),
            "release must not create an unsigned project binary for {excluded_target}"
        );
    }
    for contract in [
        "tags:\n      - \"v*.*.*\"",
        "workflow_dispatch:",
        "default: 0.3.3",
        "recovery_version:",
        "group: mcp-doctor-release",
        "scripts/validate-release-version.sh",
        "published_stable_versions",
        "all(.versions[];",
        "cargo package --locked",
        "cargo publish --locked --package mcp-doctor",
        "scripts/generate-release-channels.sh",
        "scripts/package-agent-skill.sh",
        "scripts/create-release-handoff.sh",
        "scripts/verify-release-handoff.sh",
        "scripts/rehearse-release-handoffs.sh",
        "scripts/verify-release-assets.sh",
        "scripts/verify-published-release.sh",
        "rust-lang/crates-io-auth-action@c6f97d42243bad5fab37ca0427f495c86d5b1a18",
        "actions/attest-build-provenance@",
        "gh attestation verify",
        "gh release create",
        "--draft",
        "gh release verify",
        ".immutable",
        "contents: write",
        "id-token: write",
        "attestations: write",
        "name: release",
        "Reject crates.io OIDC without the protected environment",
        "Revalidate current main and annotated tag authority",
        "REHEARSED_VERSION: ${{ needs.rehearse.outputs.version }}",
        "REQUESTED_VERSION: ${{ inputs.rehearsal_version }}",
        "inputs.recovery_version == ''",
    ] {
        assert!(
            workflow.contains(contract),
            "release should enforce {contract}"
        );
    }
    for forbidden in ["secrets.", "brew install", "winget"] {
        assert!(
            !workflow.contains(forbidden),
            "release must not contain {forbidden}"
        );
    }
    assert!(workflow.contains("no publish command exists in this job"));
    assert!(!workflow.contains("reuses only immutable v0.1.0"));
    assert_actions_are_commit_pinned(&workflow);
}

#[test]
fn immutable_partial_release_recovery_is_explicit_single_attempt_and_byte_bound() {
    let workflow = repository_file(".github/workflows/release.yml");

    for contract in [
        "Validate exact immutable partial-release recovery",
        "inputs.recovery_version != ''",
        "Check out the controlled default branch",
        "partial-release recovery must be dispatched from exact main",
        "predicate_type=release",
        ".attestations | type == \"array\" and length == 1",
        "--workflow release.yml",
        ".[0].attempt == 1 and .[0].conclusion == \"failure\"",
        "all(.versions[]; .num != $version)",
        "needs.recover.result == 'success'",
        "needs.validate.outputs.version || needs.recover.outputs.version",
        "test \"$(git rev-parse HEAD)\" = \"$release_commit\"",
        "--source-ref \"$release_ref\"",
    ] {
        assert!(
            workflow.contains(contract),
            "partial-release recovery should enforce {contract}"
        );
    }

    let publish_start = workflow.find("\n  publish:\n").unwrap();
    let recover_start = workflow.find("\n  recover:\n").unwrap();
    let source_start = workflow.find("\n  source:\n").unwrap();
    let crates_start = workflow.find("\n  crates:\n").unwrap();
    let rehearse_start = workflow.find("\n  rehearse:\n").unwrap();
    let recover_job = &workflow[recover_start..source_start];
    let publish_job = &workflow[publish_start..crates_start];
    let crates_job = &workflow[crates_start..rehearse_start];
    assert!(
        recover_job
            .find("scripts/validate-release-version.sh")
            .unwrap()
            < recover_job.find("git fetch --force origin").unwrap()
    );
    assert!(!recover_job.contains("ref: refs/tags/v${{ inputs.recovery_version }}"));
    assert!(publish_job.contains("Require immutable release state"));
    assert!(!publish_job.contains("gh release verify"));
    assert!(crates_job.contains("gh release verify"));
    assert!(!workflow.contains("sleep "));
}

#[test]
fn crates_oidc_wrong_workflow_case_is_nonpublishing_and_must_be_rejected() {
    let workflow = repository_file(".github/workflows/release-authorization-negative.yml");

    for contract in [
        "workflow_dispatch:",
        "name: release",
        "id-token: write",
        "continue-on-error: true",
        "rust-lang/crates-io-auth-action@c6f97d42243bad5fab37ca0427f495c86d5b1a18",
        "AUTH_OUTCOME",
        "accepted OIDC from an unauthorized workflow",
    ] {
        assert!(
            workflow.contains(contract),
            "negative OIDC workflow should enforce {contract}"
        );
    }
    for forbidden in ["cargo publish", "contents: write", "secrets."] {
        assert!(
            !workflow.contains(forbidden),
            "negative OIDC workflow must not contain {forbidden}"
        );
    }
    assert_actions_are_commit_pinned(&workflow);
}

#[test]
fn published_channel_verifier_is_read_only_and_runs_installed_smokes() {
    let workflow = repository_file(".github/workflows/release-channels.yml");

    for target in SOURCE_TARGETS {
        assert!(
            workflow.contains(target),
            "channel verifier should cover {target}"
        );
    }
    for contract in [
        ".immutable == true",
        "gh release verify",
        "gh attestation verify",
        ".github/workflows/release.yml",
        "mcp-doctor/$RELEASE_VERSION/download",
        "cargo install mcp-doctor",
        "brew install --build-from-source EnjoyableWork/tap/mcp-doctor",
        "scripts/smoke-archive.sh",
        "scripts/smoke-installed.sh",
        "scripts/smoke-installed.ps1",
        "EnjoyableWork/homebrew-tap/main/Formula/mcp-doctor.rb",
    ] {
        assert!(
            workflow.contains(contract),
            "channel verifier should enforce {contract}"
        );
    }
    for forbidden in [
        "contents: write",
        "id-token: write",
        "secrets.",
        "winget install",
    ] {
        assert!(
            !workflow.contains(forbidden),
            "channel verifier must not contain {forbidden}"
        );
    }
    assert!(workflow.contains(
        "User-Agent: mcp-doctor-channel-verifier/0.1 (+https://github.com/EnjoyableWork/mcp-doctor)"
    ));
    assert!(workflow.contains("canonical stable semantic version"));
    assert!(!workflow.contains("accepts only version 0.1.0"));
    assert_actions_are_commit_pinned(&workflow);
}

#[test]
fn generator_and_verifiers_enforce_the_exact_source_built_release() {
    let generator = repository_file("scripts/generate-release-channels.sh");
    let asset_verifier = repository_file("scripts/verify-release-assets.sh");
    let published_verifier = repository_file("scripts/verify-published-release.sh");
    let archive_packager = repository_file("scripts/package-release.sh");
    let agent_skill_packager = repository_file("scripts/package-agent-skill.sh");
    let agent_skill_verifier = repository_file("scripts/verify-agent-skill.sh");
    let handoff_creator = repository_file("scripts/create-release-handoff.sh");
    let handoff_verifier = repository_file("scripts/verify-release-handoff.sh");
    let handoff_rehearsal = repository_file("scripts/rehearse-release-handoffs.sh");
    let control_verifier = repository_file("scripts/verify-repeat-release-controls.sh");

    for contract in [
        "mcp-doctor-${release_version}.crate",
        "class McpDoctor < Formula",
        "depends_on \"rust\" => :build",
        "*std_cargo_args(path: \".\")",
        "server/discover",
        "tools/list",
        "active tool call attempted",
        "\"skip_reason\": \"not_authorized\"",
    ] {
        assert!(
            generator.contains(contract),
            "generator should enforce {contract}"
        );
    }
    for target in LINUX_TARGETS {
        assert!(asset_verifier.contains(target));
        assert!(published_verifier.contains(target));
        assert!(archive_packager.contains(target));
    }
    assert!(asset_verifier.contains("SPDX-2.3"));
    assert!(published_verifier.contains("verify-release-assets.sh"));
    for verifier in [&asset_verifier, &published_verifier] {
        assert!(verifier.contains("SHA256SUMS"));
        assert!(verifier.contains("mcp-doctor.rb"));
        assert!(
            verifier.contains("mcp-doctor-${published_release_version}.crate")
                || verifier.contains("mcp-doctor-${release_asset_version}.crate")
        );
    }
    assert!(archive_packager.contains("--sort=name"));
    assert!(archive_packager.contains("gzip -n -9"));
    for contract in [
        "mcp-doctor-agent-skill-v${agent_package_version}.tar.gz",
        "--sort=name",
        "--format=ustar",
        "gzip -n -9",
        "verify-agent-skill.sh",
    ] {
        assert!(
            agent_skill_packager.contains(contract),
            "Agent Skill packager should enforce {contract}"
        );
    }
    for contract in [
        "[source root]",
        "canonical Agent Skill has an unexpected file or directory",
        "allowed-tools:",
        "mcp-doctor check --",
        "mcp-doctor-agent-skill-v${agent_skill_version}.tar.gz",
        "mcp-doctor/SKILL.md",
    ] {
        assert!(
            agent_skill_verifier.contains(contract),
            "Agent Skill verifier should enforce {contract}"
        );
    }

    for contract in [
        "mcp-doctor.release-handoff/v1",
        "EnjoyableWork/mcp-doctor",
        ".github/workflows/release.yml",
        "release_handoff_environment=release",
        "release_handoff_environment=synthetic-rehearsal",
        "release_handoff_immutable=true",
        "release_handoff_provenance=true",
    ] {
        assert!(
            handoff_creator.contains(contract),
            "handoff creator should enforce {contract}"
        );
    }
    for contract in [
        ".immutable == true",
        ".provenance_verified == true",
        "Cargo handoff bytes do not match",
        "Homebrew handoff bytes do not match",
    ] {
        assert!(
            handoff_verifier.contains(contract),
            "handoff verifier should enforce {contract}"
        );
    }
    for contract in [
        "out-of-order handoff without verified provenance",
        "synthetic evidence at the verified-publication boundary",
        "mismatched Cargo bytes",
        "mismatched Homebrew bytes",
    ] {
        assert!(
            handoff_rehearsal.contains(contract),
            "handoff rehearsal should reject {contract}"
        );
    }
    for contract in [
        "environments/release/deployment-branch-policies",
        "environments/release/secrets",
        "orgs/${repeat_release_organization}/actions/secrets",
        "organization Actions secret inventory could not be verified",
        "repeat_release_organization_secret_rows",
        "actions/permissions/workflow",
        "default_workflow_permissions == \"read\"",
        "CARGO_REGISTRY_TOKEN",
        "CARGO_REGISTRIES_CRATES_IO_TOKEN",
        "rust-lang/crates-io-auth-action@c6f97d42243bad5fab37ca0427f495c86d5b1a18",
        "contents: write",
    ] {
        assert!(
            control_verifier.contains(contract),
            "repeat-release control verifier should enforce {contract}"
        );
    }
    assert!(
        !control_verifier.contains("done < <("),
        "repeat-release inventory must not lose API failures in process substitution"
    );
}

#[cfg(unix)]
#[test]
fn repeat_release_inventory_fails_closed_when_organization_secrets_are_unavailable() {
    use std::os::unix::fs::PermissionsExt as _;

    let source_sha = "1111111111111111111111111111111111111111";
    let tap_sha = "2222222222222222222222222222222222222222";
    let temporary = tempfile::tempdir().expect("repeat-release fixture root should be disposable");
    let fake_bin = temporary.path().join("bin");
    let fake_home = temporary.path().join("home");
    let fake_cargo_home = temporary.path().join("cargo");
    fs::create_dir_all(&fake_bin).expect("fake executable directory should be created");
    fs::create_dir_all(&fake_home).expect("fake home should be created");
    fs::create_dir_all(&fake_cargo_home).expect("fake Cargo home should be created");
    let fake_gh = fake_bin.join("gh");
    let fake_log = temporary.path().join("gh.log");
    let fake_gh_source = r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == auth && "$2" == status ]]; then
  exit 0
fi
if [[ "$1" != api ]]; then
  exit 2
fi
endpoint=$2
printf '%s\n' "$endpoint" >>"$REPEAT_RELEASE_GH_LOG"
case "$endpoint" in
  repos/EnjoyableWork/mcp-doctor)
    printf '{"visibility":"public","id":101}\n'
    ;;
  repos/EnjoyableWork/homebrew-tap)
    printf '{"visibility":"public","id":202}\n'
    ;;
  repos/EnjoyableWork/mcp-doctor/commits/main)
    printf '{"sha":"SOURCE_SHA_SENTINEL"}\n'
    ;;
  repos/EnjoyableWork/homebrew-tap/commits/main)
    printf '{"sha":"TAP_SHA_SENTINEL"}\n'
    ;;
  repos/EnjoyableWork/mcp-doctor/immutable-releases)
    printf '{"enabled":true}\n'
    ;;
  repos/EnjoyableWork/mcp-doctor/environments/release)
    printf '%s\n' '{"name":"release","deployment_branch_policy":{"protected_branches":false,"custom_branch_policies":true},"protection_rules":[{"type":"required_reviewers","prevent_self_review":false,"reviewers":[{}]},{"type":"branch_policy"}]}'
    ;;
  repos/EnjoyableWork/homebrew-tap/environments/release)
    printf '%s\n' '{"name":"release","deployment_branch_policy":{"protected_branches":false,"custom_branch_policies":true},"protection_rules":[{"type":"required_reviewers","prevent_self_review":false,"reviewers":[{}]},{"type":"branch_policy"}]}'
    ;;
  repos/EnjoyableWork/mcp-doctor/environments/release/deployment-branch-policies)
    printf '%s\n' '{"branch_policies":[{"name":"main","type":"branch"},{"name":"v*.*.*","type":"tag"}]}'
    ;;
  repos/EnjoyableWork/homebrew-tap/environments/release/deployment-branch-policies)
    printf '%s\n' '{"branch_policies":[{"name":"main","type":"branch"}]}'
    ;;
  repos/EnjoyableWork/mcp-doctor/actions/secrets|repos/EnjoyableWork/mcp-doctor/environments/release/secrets|repos/EnjoyableWork/homebrew-tap/actions/secrets|repos/EnjoyableWork/homebrew-tap/environments/release/secrets)
    printf '{"total_count":0}\n'
    ;;
  repos/EnjoyableWork/mcp-doctor/actions/permissions/workflow|repos/EnjoyableWork/homebrew-tap/actions/permissions/workflow)
    printf '{"default_workflow_permissions":"read","can_approve_pull_request_reviews":false}\n'
    ;;
  'orgs/EnjoyableWork/actions/secrets?per_page=100')
    if [[ "$REPEAT_RELEASE_ORG_SECRET_MODE" == unavailable ]]; then
      printf 'synthetic unavailable inventory\n' >&2
      exit 1
    fi
    printf '{"secrets":[]}\n'
    ;;
  'repos/EnjoyableWork/mcp-doctor/contents/.github/workflows/release.yml?ref=SOURCE_SHA_SENTINEL')
    content=$(printf '%s' 'rust-lang/crates-io-auth-action@c6f97d42243bad5fab37ca0427f495c86d5b1a18' | base64 | tr -d '\n')
    printf '{"content":"%s"}\n' "$content"
    ;;
  'repos/EnjoyableWork/homebrew-tap/contents/.github/workflows/publish-mcp-doctor.yml?ref=TAP_SHA_SENTINEL')
    content=$(printf '%s' 'permissions: contents: write' | base64 | tr -d '\n')
    printf '{"content":"%s"}\n' "$content"
    ;;
  *)
    printf 'unexpected synthetic endpoint\n' >&2
    exit 2
    ;;
esac
"#
    .replace("SOURCE_SHA_SENTINEL", source_sha)
    .replace("TAP_SHA_SENTINEL", tap_sha);
    fs::write(&fake_gh, fake_gh_source).expect("fake GitHub CLI should be written");
    let mut permissions = fs::metadata(&fake_gh)
        .expect("fake GitHub CLI metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_gh, permissions).expect("fake GitHub CLI should be executable");
    let inherited_path = std::env::var("PATH").expect("test PATH should be available");
    let fixture_path = format!("{}:{inherited_path}", fake_bin.display());

    let run_fixture = |mode: &str| {
        Command::new("bash")
            .arg(repository_root().join("scripts/verify-repeat-release-controls.sh"))
            .arg(source_sha)
            .arg(tap_sha)
            .current_dir(repository_root())
            .env("PATH", &fixture_path)
            .env("HOME", &fake_home)
            .env("CARGO_HOME", &fake_cargo_home)
            .env("REPEAT_RELEASE_GH_LOG", &fake_log)
            .env("REPEAT_RELEASE_ORG_SECRET_MODE", mode)
            .env_remove("CARGO_REGISTRY_TOKEN")
            .env_remove("CARGO_REGISTRIES_CRATES_IO_TOKEN")
            .env_remove("GH_TOKEN")
            .env_remove("GITHUB_TOKEN")
            .output()
            .expect("repeat-release verifier should execute")
    };

    let unavailable = run_fixture("unavailable");
    assert!(!unavailable.status.success());
    assert!(
        !String::from_utf8_lossy(&unavailable.stdout)
            .contains("Verified clean repeat-release controls")
    );
    assert!(
        String::from_utf8_lossy(&unavailable.stderr)
            .contains("organization Actions secret inventory could not be verified")
    );

    let empty = run_fixture("empty");
    assert!(
        empty.status.success(),
        "empty synthetic inventory should pass: {}",
        String::from_utf8_lossy(&empty.stderr)
    );
    assert!(
        String::from_utf8_lossy(&empty.stdout).contains("Verified clean repeat-release controls")
    );
}

#[test]
#[cfg_attr(
    windows,
    ignore = "release version script executes only in Unix release jobs"
)]
fn release_version_guard_accepts_only_canonical_intentional_versions() {
    let accepted: &[&[&str]] = &[
        &["future", "v0.1.1", "0.1.1", "0.1.0"],
        &["future", "v0.1.1", "0.1.1", "0.1.0", "0.1.1"],
        &["future", "v0.2.0", "0.2.0", "0.1.0", "0.1.9"],
        &[
            "future", "v0.4.2", "0.4.2", "0.1.0", "0.2.0", "0.3.0", "0.3.1", "0.3.2", "0.3.3",
            "0.4.0", "0.4.1",
        ],
        &["future", "v1.0.0", "1.0.0", "0.1.0", "0.99.99"],
        &[
            "future",
            "v18446744073709551616.0.0",
            "18446744073709551616.0.0",
            "18446744073709551615.99.99",
        ],
        &["published", "v0.1.0", "0.1.0"],
        &["published", "v12.34.56", "12.34.56"],
        &["rehearsal", "v0.1.0", "0.1.0"],
        &["rehearsal", "v0.3.0", "0.3.0"],
        &["rehearsal", "v12.34.56", "12.34.56"],
    ];
    for arguments in accepted {
        assert_release_version_case(arguments, true);
    }

    let rejected: &[&[&str]] = &[
        &["future", "v0.1.1", "0.1.1"],
        &["future", "v0.1.0", "0.1.0", "0.1.0"],
        &["future", "v0.0.9", "0.0.9", "0.1.0"],
        &["future", "v0.1.1", "0.1.2", "0.1.0"],
        &["future", "v0.1.1", "0.1.1", "0.2.0"],
        &["future", "v0.4.0", "0.4.0", "0.4.1"],
        &["future", "0.1.1", "0.1.1", "0.1.0"],
        &["future", "v01.2.3", "01.2.3", "0.1.0"],
        &["future", "v1.2.3-rc.1", "1.2.3-rc.1", "0.1.0"],
        &["future", "v1.2.3", "1.2.3", "not-stable"],
        &["published", "v0.0.99", "0.0.99"],
        &["rehearsal", "v0.0.99", "0.0.99"],
        &["rehearsal", "v0.3.0", "0.3.1"],
        &["rehearsal", "v0.3.0", "0.3.0", "0.3.0"],
        &["unknown", "v1.0.0", "1.0.0"],
    ];
    for arguments in rejected {
        assert_release_version_case(arguments, false);
    }
}

fn assert_release_version_case(arguments: &[&str], expected_success: bool) {
    let output = Command::new("bash")
        .arg(repository_root().join("scripts/validate-release-version.sh"))
        .args(arguments)
        .output()
        .expect("release version guard should execute");
    assert_eq!(
        output.status.success(),
        expected_success,
        "unexpected release version result for {arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_runbook_bounds_the_session_scoped_verification_credential() {
    let release = repository_file("docs/release.md");
    let normalized_release = release.split_whitespace().collect::<Vec<_>>().join(" ");
    let organization_controls: serde_json::Value =
        serde_json::from_str(&repository_file(".github/organization-controls.json"))
            .expect("organization controls should be valid JSON");
    let profile = organization_controls
        .pointer("/automation_credentials/verification_operator_credential")
        .expect("verification operator profile should exist");

    for repository in profile["repositories"]
        .as_array()
        .expect("verification repositories should be an array")
    {
        let repository = repository
            .as_str()
            .expect("verification repository should be a string");
        assert!(
            release.contains(&format!("`{repository}`")),
            "release runbook should preserve verification repository {repository}"
        );
    }
    for permission_group in ["organization_permissions", "repository_permissions"] {
        for permission in profile[permission_group]
            .as_object()
            .expect("verification permissions should be an object")
            .keys()
        {
            assert!(
                release.contains(&format!("`{permission}`")),
                "release runbook should preserve verification permission {permission}"
            );
        }
    }

    let maximum_lifetime = profile["maximum_lifetime_days"]
        .as_u64()
        .expect("verification lifetime should be an integer");
    for contract in [
        "with an expiration of one day",
        &format!("`{maximum_lifetime}`-day maximum is an absolute policy ceiling"),
        "URL-prefilled permissions do not select the repositories",
        "explicitly choose `Only select repositories`",
        "second review dialog",
        "return to an empty token list",
        "is not issuance evidence",
        "only as `GH_TOKEN`",
        "Do not use `gh auth login`",
        "Before any rehearsal, run both live audits",
        "verify-supply-chain-controls.sh --source-ref",
        "verify-repeat-release-controls.sh",
        "inspect the exact run's pending deployment",
        "Approve only that deployment",
        "Each protected job creates its own review boundary",
        "send the selected environment identifiers as JSON integers, not strings",
        "HTTP `422` is not approval evidence",
        "disappear from the run's pending set",
        "rerun both live audits once because the workflow runs changed external state",
        "Revoke the fine-grained token immediately",
        "exactly one bounded `GET /user` request",
        "HTTP `401` response as revocation evidence",
        "do not retry it as correctness evidence",
        "Never retain the token value",
    ] {
        assert!(
            normalized_release.contains(contract),
            "release runbook should preserve session credential contract: {contract}"
        );
    }
}

#[test]
fn release_docs_keep_scope_and_adoption_evidence_honest() {
    let release = repository_file("docs/release.md");
    let first_notes = repository_file("docs/releases/v0.1.0.md");
    let retained_notes = repository_file("docs/releases/v0.2.0.md");
    let expanded_notes = repository_file("docs/releases/v0.3.0.md");
    let security_notes = repository_file("docs/releases/v0.3.1.md");
    let agent_notes = repository_file("docs/releases/v0.3.2.md");
    let bounded_work_notes = repository_file("docs/releases/v0.3.3.md");
    let candidate_notes = repository_file("docs/releases/v0.4.0.md");
    let patch_notes = repository_file("docs/releases/v0.4.1.md");
    let runtime_notes = repository_file("docs/releases/v0.4.2.md");
    let security_record = repository_file("docs/assurance/v0.3.1-security-release.md");
    let bounded_work_record = repository_file("docs/assurance/v0.3.3-security-release.md");
    let interruption_record = repository_file("docs/assurance/v0.4.1-security-release.md");
    let adoption = repository_file("docs/adoption.md");

    assert!(release.contains("exactly these seven assets"));
    assert!(release.contains("never replace an"));
    assert!(release.contains("asset, move the tag, or overwrite downstream bytes"));
    assert!(release.contains("does not issue macOS or Windows binaries"));
    assert!(release.contains("Workflow filename | `release.yml`"));
    assert!(release.contains("Environment | `release`"));
    assert!(release.contains("No publish command exists in the authorization job"));
    assert!(
        release.contains(
            "currently represented identically across GitHub Releases, Cargo, and Homebrew"
        )
    );
    assert!(release.contains("`0.3.3` at this review"));
    assert!(release.contains("cross-repository personal token"));
    assert!(release.contains("test alone is not completion evidence"));
    for contract in [
        "This source tree represents the published `mcp-doctor` `0.4.2` release",
        "GitHub Releases determines whether a version has completed public\npublication.",
        "b0805a8f685e46814e358de368e2a270c21704af",
        "31528649356",
        "31528649333",
        "31529740214",
        "31530330361",
        "a57736ea1a7abf73eeff9a8278af11110247bd20",
        "31530466930",
        "d9b96bbeb84baccb8e5c890e9c655a559a12a474",
        "31746397550",
        "31746397557",
        "31754685159",
        "31754685137",
        "31755736570",
        "31756253855",
        "2b62e11902c7461cddbc0b96075e3745fdf6f260",
        "31756413098",
        "d4db369a2789f7b6f89b2daad4adc1b6f4900f7e",
        "31985219134",
        "31985523936",
        "31985595470",
        "d117cf4c7cbbd5bfb6dd43c01af2607ae64cc1d2",
        "31996320198",
        "31996322032",
        "31996325240",
        "31996323697",
        "31996837111",
        "b3bfd0d084ee5fdaf6553ee6d3c225cd5ad7d302",
        "31997316851",
        "31997406753",
        "9f3a838751856bd20d670053071b6d537f430d37",
        "31999383802",
        "31999383050",
        "31999383921",
        "32000204735",
        "32000204694",
        "32000204757",
        "32000204919",
        "074a62dbbfce5fa417f2b7080d509ebd86433b1f",
        "32332461578",
        "32332461575",
        "32386249809",
        "32386561455",
        "32386712018",
        "32386772185",
        "32389641937",
        "b87aff88710cce5a8d4d42b8429041bdda2dd51485c80910808312a0b0e035fe",
        "44ee744b19b01d9a69c4d6f1c23248cb6e08b90c76556f202c417c75d48e6e97",
        "32391019736",
        "205e112a17498b3e817240283ef9e16bf7f81027",
        "32391172160",
        "6aab8cd2019e495370dd246ceb89efc056af47a6",
        "33706588903",
        "33706588930",
        "0dc34abd92f87597e851f7378e94de97271aa906",
        "33707483408",
        "5b464c7005cf997039284eb0f1f91ee60abccb6f",
        "33707924886",
        "33708074873",
        "GHSA-rw9q-ggrp-frwc",
        "v0.4.1-security-release.md",
        "v0.3.3-security-release.md",
        "21c3ad8dba319339060c02523aed049282ada790cbecb691f4f270297b456341",
        "f7ee6903c839a268648bf8114e75817396a78f7b08f38a424541fe4b0c483a51",
        "passed all ten jobs",
    ] {
        assert!(
            release.contains(contract),
            "release guide should retain v0.2.0 evidence: {contract}"
        );
    }
    assert!(first_notes.contains("does not call tools"));
    assert!(first_notes.contains("does not call tools, connect to remote HTTP endpoints"));
    assert!(retained_notes.contains("mcp-doctor.report/v1"));
    for contract in [
        "2025-11-25",
        "contract snapshots",
        "JSON and JUnit report files",
        "offline aggregation",
        "unsupported-version response",
        "does not add SARIF",
        "become a general security scanner",
    ] {
        assert!(
            expanded_notes.contains(contract),
            "v0.3.0 release notes should preserve {contract}"
        );
    }
    for contract in [
        "security patch",
        "scenario, custom-CA, snapshot, and aggregate",
        "without following a symbolic link or Windows reparse",
        "Custom CA material is now read and validated",
        "resolution, DNS, or connection activity",
        "report artifact publication",
        "before\nlinking",
        "immediately\nafter linking",
        "Cleanup and rollback remove only paths",
        "JSON, JUnit, and\naggregate output",
        "Users of `0.2.0` or `0.3.0` should upgrade",
        "GHSA-92m2-749h-2gv5",
        "GHSA-8r6p-qf9j-vpvx",
        "cargo install mcp-doctor --version '=0.3.1' --locked",
    ] {
        assert!(
            security_notes.contains(contract),
            "v0.3.1 release notes should preserve {contract}"
        );
    }
    for contract in [
        "Portable Agent Skill",
        "mcp-doctor-agent-skill-v0.3.2.tar.gz",
        "mcp-doctor --help",
        "do not modify any coding-agent host",
        "refuses inferred targets, installations, secrets, `check`, `break`, and",
        "cargo install mcp-doctor --version '=0.3.2' --locked",
    ] {
        assert!(
            agent_notes.contains(contract),
            "v0.3.2 release notes should preserve {contract}"
        );
    }
    for contract in [
        "security patch",
        "Fragmented request-scoped SSE",
        "schema_evaluation_steps",
        "unsupported_linear_pattern",
        "separate advisories\nbecause",
        "GHSA-3vpj-fcvj-28pm",
        "GHSA-jr72-f9q4-424m",
        "cargo install mcp-doctor --version '=0.3.3' --locked",
    ] {
        assert!(
            bounded_work_notes.contains(contract),
            "v0.3.3 release notes should preserve {contract}"
        );
    }
    for contract in [
        "MCP-QUALITY-001",
        "MCP-QUALITY-002",
        "MCP-QUALITY-003",
        "MCP-SECURITY-001",
        "bounded `auto`",
        "MCP-SCHEMA-005",
        "mcp-doctor.markdown/v1",
        "mcp-doctor.badge/v1",
        "least-permission GitHub Actions passive\npreflight",
        "cargo install mcp-doctor --version '=0.4.0' --locked",
    ] {
        assert!(
            candidate_notes.contains(contract),
            "v0.4.0 release notes should preserve {contract}"
        );
    }
    let candidate_notes_lower = candidate_notes.to_ascii_lowercase();
    for forbidden in [
        "projected score",
        "target rank",
        "ranking",
        "certified",
        "leadership claim",
    ] {
        assert!(
            !candidate_notes_lower.contains(forbidden),
            "v0.4.0 release notes must not contain {forbidden}"
        );
    }
    for contract in [
        "Unix STDIO",
        "SIGINT",
        "SIGTERM",
        "completion_reason",
        "interrupted",
        "4,000 ms",
        "GHSA-rw9q-ggrp-frwc",
        "cargo install mcp-doctor --version '=0.4.1' --locked",
    ] {
        assert!(
            patch_notes.contains(contract),
            "v0.4.1 release notes should preserve {contract}"
        );
    }
    for contract in [
        "hostname-based Streamable HTTP",
        "runtime shutdown waits at most 100 ms",
        "limits.runtime_shutdown_timeout_ms: 100",
        "whole_process_exit_guarantee: false",
        "MCP-LIMIT-001",
        "adds no dependency",
        "GHSA-924w-xv6c-7vw3",
        "cargo install mcp-doctor --version '=0.4.2' --locked",
    ] {
        assert!(
            runtime_notes.contains(contract),
            "v0.4.2 release notes should preserve {contract}"
        );
    }
    for contract in [
        "v0.3.1 coordinated security-release record",
        "2026-08-17T00:22:40.893Z",
        "31981850276",
        "No unchanged-source workflow",
        "3aabbbd31b54b81d42531918766a6d2794259fb6",
        "GHSA-92m2-749h-2gv5",
        "GHSA-8r6p-qf9j-vpvx",
    ] {
        assert!(
            security_record.contains(contract),
            "v0.3.1 security-release record should preserve {contract}"
        );
    }
    for contract in [
        "v0.3.3 coordinated bounded-work security-release record",
        "32095369800",
        "995d471b0024a6d1e16b85e1778168bd27d3aebc",
        "7e5fff3b7fa953a4ae371739a6046db9cd56feca",
        "32099327284",
        "4a2e2f3ba88dad5a8d80cba42c3ee07c38da18bc",
        "32099683447",
        "GHSA-3vpj-fcvj-28pm",
        "GHSA-jr72-f9q4-424m",
        "One bounded later observation proved GitHub\nrejected it",
    ] {
        assert!(
            bounded_work_record.contains(contract),
            "v0.3.3 security-release record should preserve {contract}"
        );
    }
    for contract in [
        "v0.4.1 Unix interruption security-release record",
        "3028d2d683bf13cab8e8ceb4e35060fb63430a83",
        "33705162938",
        "33705162922",
        "33705162929",
        "33705161875",
        "5ea157c74da0ed13c71a0cd2f5451b1a50bed4a6",
        "33705872376",
        "33705872402",
        "33705869770",
        "33705872268",
        "6aab8cd2019e495370dd246ceb89efc056af47a6",
        "7b752e16bad9c4f36a2b3df03ed8cb29a439df20",
        "33706588903",
        "33706588930",
        "0dc34abd92f87597e851f7378e94de97271aa906",
        "33707483408",
        "d6f59e72ca2ae68299a1b22a9be1f3f549c21484909e3e71f26f64c3f7c614d9",
        "86a08514b756c62ac68fc5de1bfca0075ba8552afda8319d1f0f02cfea85170c",
        "5b464c7005cf997039284eb0f1f91ee60abccb6f",
        "33707924886",
        "33708074873",
        "GHSA-rw9q-ggrp-frwc",
    ] {
        assert!(
            interruption_record.contains(contract),
            "v0.4.1 security-release record should preserve {contract}"
        );
    }
    assert!(adoption.contains("Opened: 2026-08-10"));
    assert!(adoption.contains("Closed: 2026-08-10"));
    assert!(adoption.contains("zero independent adoption reports at opening"));
    assert!(adoption.contains("no adoption or repeat-use claim"));
    assert!(adoption.contains("does not block later\nscoped feature work"));
    assert!(adoption.contains("at least five independently authored servers"));
    for sensitive in [
        "endpoint URLs",
        "credentials",
        "authentication headers",
        "raw MCP payloads",
        "tool arguments or results",
    ] {
        assert!(adoption.contains(sensitive));
    }
}

#[test]
fn junit_compatibility_evidence_is_scoped_and_pinned() {
    let evidence = repository_file("tests/junit/README.md");

    for contract in [
        "not a JUnit standard claim",
        "process exit status",
        "67a81935603ce6740d5036f23f867ada49bd5cb3",
        "7f38b981fe5d1895345f265b70773e98927b0893",
        "1 success, 1 failed, and 1 skipped",
        "two selected independent parsers at pinned commits",
    ] {
        assert!(
            evidence.contains(contract),
            "JUnit evidence should preserve {contract}"
        );
    }
}

#[test]
fn repository_guidance_keeps_active_authorization_boundary_explicit() {
    let agents = repository_file("AGENTS.md");

    for contract in [
        "mcp-doctor.scenario/v1alpha1",
        "--allow-tool",
        "--allow-side-effects",
        "RFC 6901",
        "Treat `input_required` as incomplete",
    ] {
        assert!(
            agents.contains(contract),
            "AGENTS.md should retain active safety rule: {contract}"
        );
    }
}

#[test]
fn repository_guidance_keeps_network_boundary_explicit() {
    let agents = repository_file("AGENTS.md");

    for contract in [
        "Follow the Streamable HTTP safety contract",
        "16-address cap",
        "zero redirects and application retries",
        "full chain and service-identity",
        "`--allow-credentials-to` HTTPS endpoint",
        "Do not fetch OAuth metadata",
        "stateless MCP `2026-07-28` POST binding",
    ] {
        assert!(
            agents.contains(contract),
            "AGENTS.md should retain remote safety rule: {contract}"
        );
    }
}

#[test]
fn command_guide_keeps_generation_boundary_explicit() {
    let commands = repository_file("docs/commands.md");

    for contract in [
        "--tool search",
        "--allow-tool search",
        "--effects read_only",
        "--cases 50",
        "--seed 4242",
        "A `side_effecting` run also requires `--allow-side-effects`",
        "reports never contain raw generated\narguments or tool results",
    ] {
        assert!(
            commands.contains(contract),
            "the command guide should describe the bounded break contract: {contract}"
        );
    }
}

#[test]
fn command_guide_freezes_the_a1_tool_description_contract() {
    let commands = repository_file("docs/commands.md");

    for contract in [
        "### Tool-description quality",
        "`MCP-QUALITY-001`",
        "`MCP-QUALITY-003`",
        "`MCP-QUALITY-004`",
        "`tools[index].description`",
        "`todo`,\n`tbd`, `tool`, `description`, or `placeholder`",
        "One tool receives at most one description-quality\nfinding",
        "`MCP-QUALITY-001` remains authoritative",
        "`A1 normalization v1`",
        "`U+0009`–`U+000D`",
        "`U+0020`",
        "`U+0085`",
        "`U+00A0`",
        "`U+1680`",
        "`U+2000`–`U+200A`",
        "`U+2028`",
        "`U+2029`",
        "`U+202F`",
        "`U+205F`",
        "`U+3000`",
        "trims boundary ASCII whitespace",
        "removes ASCII punctuation, and lowercases ASCII letters",
        "does\nnot transliterate, locale-fold, or infer semantic similarity",
        "compares only exact normalized values",
        "do not grade\nnear-duplicate or semantically similar prose",
        "occurs exactly once in the accepted catalog prefix",
        "first eligible occurrence of one exact normalized value is canonical",
        "`first_matching_tool_index`",
        "An entry beyond the `catalog_items`\nbound cannot create a match",
        "`MCP-LIMIT-001` identifies `report_findings`",
        "delivered response prefixes are discarded",
        "Badge and\naggregate projections retain only their existing summary information",
        "non-string description remains `MCP-CATALOG-001`",
        "without an extra request, tool\ncall, dependency, retry, fallback, target launch, network activity, or",
    ] {
        assert!(
            commands.contains(contract),
            "the command guide should retain the A1 description-quality contract: {contract}"
        );
    }
}

#[test]
fn command_guide_records_the_required_input_description_contract() {
    let commands = repository_file("docs/commands.md");

    for contract in [
        "### Required-input description quality",
        "`MCP-QUALITY-002`",
        "`tools[index].inputSchema.properties[index].description`",
        "finite `A1\nnormalization v1` whitespace set",
        "A local-reference\nwrapper therefore needs its own description",
        "Optional properties and names in\n`required` that have no direct `properties` entry are not diagnosed",
        "instead of a partial\nrequired-input quality result",
        "adds no request, tool call, dependency,\nor LLM evaluation",
    ] {
        assert!(
            commands.contains(contract),
            "the command guide should retain the required-input quality contract: {contract}"
        );
    }
}

#[test]
fn command_guide_records_the_rejection_boundary() {
    let commands = repository_file("docs/commands.md");
    let agents = repository_file("AGENTS.md");

    for contract in [
        "## Schema-invalid `reject` cases",
        "mcp-doctor reject",
        "wrong root type",
        "exactly `-32602`",
        "including `isError: true` or `input_required`",
        "An expected rejection is not an execution safeguard",
        "Reports retain only the generator version, seed,",
    ] {
        assert!(
            commands.contains(contract),
            "the command guide should describe the bounded reject contract: {contract}"
        );
    }

    for contract in [
        "`reject` uses only MCP `2026-07-28`",
        "transmit a case only after the local validator proves exactly",
        "integer code `-32602` and a string message",
        "Treat any result—including `isError: true` or",
        "`reject` never selects or",
    ] {
        assert!(
            agents.contains(contract),
            "AGENTS.md should retain the reject safety rule: {contract}"
        );
    }
}

#[test]
fn readme_leads_with_an_accessible_bounded_diagnosis_screenshot() {
    let readme = repository_file("README.md");
    let introduction = readme
        .split("## Install")
        .next()
        .expect("README should have an introductory diagnosis");

    for contract in [
        "docs/assets/mcp-doctor-inspect-report.png",
        "alt=\"Terminal screenshot of mcp-doctor passively inspecting",
        "MCP 2025-11-25 server",
        "two MCP-SCHEMA-002 input schema findings.",
        "width=\"1044\"",
    ] {
        assert!(
            introduction.contains(contract),
            "README introduction should preserve {contract}"
        );
    }
    for removed_example in [
        "A diagnosis you can act on:",
        "Your weather server starts correctly",
        "weather_forecast",
    ] {
        assert!(
            !introduction.contains(removed_example),
            "README introduction should not retain {removed_example}"
        );
    }

    let screenshot = fs::read(repository_root().join("docs/assets/mcp-doctor-inspect-report.png"))
        .expect("README diagnosis screenshot should be readable");
    assert!(
        screenshot.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]),
        "README diagnosis screenshot should be a PNG"
    );
    assert!(
        screenshot.len() >= 24,
        "README diagnosis screenshot should contain a complete PNG header"
    );
    let width = u32::from_be_bytes(screenshot[16..20].try_into().expect("PNG width should fit"));
    let height = u32::from_be_bytes(
        screenshot[20..24]
            .try_into()
            .expect("PNG height should fit"),
    );
    assert_eq!((width, height), (2088, 1323));
    assert!(
        screenshot.len() <= 256 * 1024,
        "README diagnosis screenshot should remain reasonably small"
    );

    assert!(
        readme.lines().count() <= 250,
        "README should remain an onboarding page rather than a full manual"
    );
    for guide in [
        "docs/commands.md",
        "docs/protocol-support.md",
        "docs/automation.md",
        "docs/safety.md",
    ] {
        assert!(
            readme.contains(guide),
            "README should route detailed material to {guide}"
        );
    }
}

#[test]
fn readme_quick_start_is_task_focused_and_routes_detailed_guidance() {
    let readme = repository_file("README.md");
    let quick_start = readme
        .split_once("## Quick start")
        .and_then(|(_, remainder)| remainder.split_once("## Why mcp-doctor?"))
        .map(|(section, _)| section)
        .expect("README should keep a bounded quick-start section");

    for contract in [
        "### Diagnose a local STDIO server",
        "### Diagnose a remote Streamable HTTP server",
        "### Select a protocol revision",
        "`auto`, `2026-07-28`, `2025-11-25`, and `2025-06-18`",
    ] {
        assert!(
            quick_start.contains(contract),
            "README quick start should preserve {contract}"
        );
    }

    let documentation = readme
        .split_once("## Documentation")
        .and_then(|(_, remainder)| remainder.split_once("## Assurance"))
        .map(|(section, _)| section)
        .expect("README should keep a focused documentation table");
    for route in [
        "[Coding-agent guide](docs/agents.md)",
        "[Automation and CI](docs/automation.md)",
        "[Badge artifact contract](docs/automation.md#badge-artifacts)",
        "[MCP revision support](docs/protocol-support.md)",
    ] {
        assert!(
            documentation.contains(route),
            "README documentation table should route readers to {route}"
        );
    }
}

#[test]
fn readme_navigation_follows_the_onboarding_order() {
    let readme = repository_file("README.md");
    let navigation = [
        "<a href=\"#install\">Install</a>",
        "<a href=\"#quick-start\">Quick start</a>",
        "<a href=\"#why-mcp-doctor\">Why mcp-doctor?</a>",
        "<a href=\"docs/automation.md\">Automation and CI</a>",
        "<a href=\"#agent-skill\">Agent Skill</a>",
        "<a href=\"#choose-a-diagnostic\">Commands</a>",
        "<a href=\"#documentation\">Docs</a>",
        "<a href=\"#assurance\">Assurance</a>",
    ];

    let mut previous = None;
    for link in navigation {
        let position = readme
            .find(link)
            .unwrap_or_else(|| panic!("README navigation omitted {link}"));
        if let Some(previous) = previous {
            assert!(
                previous < position,
                "README navigation placed {link} out of order"
            );
        }
        previous = Some(position);
    }
}

#[test]
fn readme_exposes_simple_verified_installation_channels() {
    let readme = repository_file("README.md");
    let installation = readme
        .split_once("## Install")
        .and_then(|(_, remainder)| remainder.split_once("## Agent Skill"))
        .map(|(section, _)| section)
        .expect("README should present CLI installation before the Agent Skill");

    for contract in [
        "| Homebrew | macOS, GNU/Linux | `brew install EnjoyableWork/tap/mcp-doctor` |",
        "| Cargo | macOS, GNU/Linux, Windows | `cargo install mcp-doctor` |",
        "| GitHub Releases | GNU/Linux (ARM64, x64) | [Download the latest archive]",
        "for exact-version installs\nand artifact verification",
    ] {
        assert!(
            installation.contains(contract),
            "README installation should preserve {contract}"
        );
    }

    for unnecessary_or_unpublished in [
        "--build-from-source",
        "--version",
        "--locked",
        "winget install",
    ] {
        assert!(
            !installation.contains(unnecessary_or_unpublished),
            "README installation should not present {unnecessary_or_unpublished}"
        );
    }
}

#[test]
fn compiled_capability_discovery_has_no_target_authority() {
    let automation = repository_file("docs/automation.md");
    let implementation = repository_file("src/capabilities.rs");
    let schema = repository_file("schemas/mcp-doctor.capabilities.v1.schema.json");
    let posix_smoke = repository_file("scripts/smoke-installed.sh");
    let powershell_smoke = repository_file("scripts/smoke-installed.ps1");

    for contract in [
        "mcp-doctor capabilities --format json",
        "mcp-doctor.capabilities/v1",
        "supported, unsupported, or unknown",
        "does not inspect user configuration or host inventory",
        "schemas/mcp-doctor.capabilities.v1.schema.json",
        "Capability discovery reports only fixed compiled facts",
    ] {
        assert!(
            automation.contains(contract),
            "the automation guide should preserve capability-discovery contract: {contract}"
        );
    }

    for contract in [
        "CAPABILITIES_SCHEMA_VERSION",
        "MAXIMUM_OUTPUT_BYTES",
        "render_unsupported_schema",
        "protocol_support",
        "recognized_unsupported",
    ] {
        assert!(
            implementation.contains(contract),
            "compiled manifest should retain {contract}"
        );
    }
    for prohibited in [
        "std::env::var",
        "std::fs",
        "std::net",
        "std::process::Command",
        "reqwest",
        "tokio::net",
    ] {
        assert!(
            !implementation.contains(prohibited),
            "compiled manifest gained prohibited activity surface {prohibited}"
        );
    }

    for contract in [
        "mcp-doctor.capabilities/v1",
        "unsupported_schema_version",
        "additionalProperties",
        "65536",
    ] {
        assert!(
            schema.contains(contract),
            "capability schema omitted {contract}"
        );
    }
    for smoke in [posix_smoke, powershell_smoke] {
        for contract in [
            "capabilities",
            "mcp-doctor.capabilities/v1",
            "mcp-doctor.exit/v1",
            "mcp-doctor.generator/v1",
            "--protocol-version",
            "2025-11-25",
            "2025-06-18",
            "legacy-active-success",
            "legacy-break-success",
            "negotiated_protocol_revision",
        ] {
            assert!(
                smoke.contains(contract),
                "represented installed smoke omitted {contract}"
            );
        }
    }
}

#[test]
fn protocol_support_guide_revision_matrix_uses_two_semantic_support_states() {
    let readme = repository_file("README.md");
    let support = repository_file("docs/protocol-support.md");
    assert!(readme.contains("[MCP revision support](docs/protocol-support.md)"));
    assert!(!readme.contains("| MCP revision |"));
    assert!(support.contains("**Legend:** ✅ = supported; ❌ = not supported."));
    assert!(support.contains("is the authoritative machine-readable\ncontract."));

    let mut lines = support.lines();
    let header = lines
        .find(|line| line.starts_with("| MCP revision | Est. usage"))
        .expect("protocol support guide should contain the MCP revision matrix");
    assert!(header.contains("`inspect`") && header.contains("`reject`"));
    let separator = lines
        .next()
        .expect("the revision matrix should have a separator");
    assert!(separator.starts_with("| --- |"));

    let rows = lines
        .take_while(|line| line.starts_with('|'))
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 7, "the revision matrix inventory drifted");
    for row in rows {
        let cells = row.split('|').map(str::trim).collect::<Vec<_>>();
        assert_eq!(cells.len(), 10, "revision matrix column count drifted");
        for status in &cells[3..9] {
            assert!(
                matches!(
                    *status,
                    "✅ <!-- mcp-doctor-support=supported -->"
                        | "❌ <!-- mcp-doctor-support=unsupported -->"
                ),
                "revision support must use one of two semantic states: {status}"
            );
        }
    }
}

#[test]
fn automation_guide_exit_table_matches_the_stable_contract() {
    let automation = repository_file("docs/automation.md");
    let mut lines = automation.lines();
    lines
        .find(|line| *line == "| Exit | Stable meaning | In practice |")
        .expect("the automation guide should contain the exit-code table");
    assert_eq!(
        lines.next(),
        Some("| ---: | --- | --- |"),
        "the exit-code table should retain its three columns"
    );

    let actual = lines
        .take_while(|line| line.starts_with('|'))
        .map(|row| {
            let cells = row.split('|').map(str::trim).collect::<Vec<_>>();
            assert_eq!(cells.len(), 5, "exit-code table column count drifted");
            (cells[1], cells[2])
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            ("`0`", "`success`"),
            ("`1`", "`unsuccessful_result`"),
            ("`2`", "`invalid_invocation_or_input`"),
            ("`3`", "`incomplete_evidence`"),
            ("`4`", "`internal_or_output_failure`"),
        ]
    );
}

#[test]
fn automation_guide_defines_conservative_offline_report_aggregation() {
    let automation = repository_file("docs/automation.md");
    let posix_smoke = repository_file("scripts/smoke-installed.sh");
    let powershell_smoke = repository_file("scripts/smoke-installed.ps1");

    for contract in [
        "`aggregate` combines",
        "mcp-doctor.aggregate/v1",
        "Members are identified only by zero-based input ordinal",
        "There is no waiver, score, baseline",
        "never starts a process, opens a connection",
        "resolves DNS or\ncredentials",
        "must not retrieve either\nat runtime",
    ] {
        assert!(
            automation.contains(contract),
            "the automation guide should describe offline aggregation: {contract}"
        );
    }
    for (name, smoke) in [("POSIX", posix_smoke), ("PowerShell", powershell_smoke)] {
        assert!(
            smoke.contains("aggregate"),
            "{name} smoke omitted aggregate"
        );
        assert!(
            smoke.contains("mcp-doctor.aggregate/v1"),
            "{name} smoke omitted the aggregate schema contract"
        );
    }
}

#[test]
fn emergency_exercise_preserves_scoped_public_evidence() {
    let exercise = repository_file("docs/assurance/emergency-bypass-exercise-2026-08-11.md");

    for contract in [
        "Status: closed at `2026-08-11T21:55:10Z`",
        "emergency-bypass-2026-08-11",
        "05090b3b62ae145f06dbdd69f3346e4cd2fa607a",
        "`BLOCKED`; neither `Required CI` nor `Required release preflight` had reported",
        "2026-08-11T21:42:39.603Z",
        "8487b47dbddb2dd1c50020b5b157d9807bc4fcd7",
        "2026-08-11T21:42:44.005Z",
        "31539153287/job/93938063807",
        "31539153316/job/93940246247",
        "first activation at `2026-08-11T21:41:04Z`",
        "restored the empty bypass list at `2026-08-11T21:41:38.192Z`",
        "The ruleset was never disabled",
        "No actor identity",
    ] {
        assert!(
            exercise.contains(contract),
            "emergency record should retain bounded closure evidence: {contract}"
        );
    }

    assert!(!exercise.contains("Status: pending"));
}

#[test]
fn security_policy_defines_private_reporting_support_and_coordination() {
    let security = repository_file("SECURITY.md");

    for contract in [
        "## Security contact and private reporting",
        "https://github.com/EnjoyableWork/mcp-doctor/security/advisories/new",
        "This is the project's recognized security contact and confidential reporting\nroute.",
        "acknowledge a private report within 3 business days",
        "initial assessment or status within 7 calendar days",
        "update at least every 14 calendar days",
        "not a service-level agreement",
        "public disclosure within 90 days of\nacknowledgement",
        "GitHub Security Advisory",
        "request a CVE through GitHub when warranted",
        "does not operate\na bug-bounty program",
        "| `0.4.x` | Supported |",
        "| `0.3.x` and earlier | Unsupported |",
        "| `main` | Development only; no release or backport guarantee |",
        "## Safe research boundary",
        "Test only systems you own or are explicitly authorized to assess.",
    ] {
        assert!(
            security.contains(contract),
            "SECURITY.md should preserve the disclosure contract: {contract}"
        );
    }

    for stale_or_unsafe in [
        "There is no supported public release yet",
        "will eventually connect to remote MCP endpoints",
        "guaranteed remediation",
    ] {
        assert!(
            !security.contains(stale_or_unsafe),
            "SECURITY.md must not retain {stale_or_unsafe}"
        );
    }
}

#[test]
fn security_control_projection_matches_the_live_security_contract() {
    let canonical = repository_file(".github/security-controls.json");
    let canonical: serde_json::Value =
        serde_json::from_str(&canonical).expect("security controls should be valid JSON");

    assert_eq!(
        canonical["schema_version"],
        "mcp-doctor.github-security-controls/v1"
    );
    assert_eq!(canonical["api_version"], "2026-03-10");
    assert_eq!(canonical["repository"], "EnjoyableWork/mcp-doctor");
    assert_eq!(canonical["repository_visibility"], "public");
    assert_eq!(canonical["organization_plan"], "free");
    assert_eq!(canonical["default_branch"], "main");
    assert_eq!(
        canonical["security_policy"],
        serde_json::json!({
            "path": "SECURITY.md",
            "private_reporting_url": "https://github.com/EnjoyableWork/mcp-doctor/security/advisories/new",
            "supported_release_lines": ["0.3.x"],
            "unsupported_release_lines": ["0.2.x and earlier"],
            "acknowledgement_business_days": 3,
            "initial_assessment_calendar_days": 7,
            "update_calendar_days": 14,
            "coordinated_disclosure_target_days": 90
        })
    );
    assert_eq!(
        canonical["controls"],
        serde_json::json!({
            "vulnerability_alerts": true,
            "automated_security_fixes": true,
            "dependabot_security_updates": "enabled",
            "private_vulnerability_reporting": true,
            "code_scanning_default_setup": {
                "state": "configured",
                "languages": ["actions", "rust"],
                "query_suite": "default",
                "runner_type": "standard",
                "schedule": "weekly",
                "threat_model": "remote"
            },
            "secret_scanning": "enabled",
            "secret_scanning_push_protection": "enabled",
            "secret_scanning_non_provider_patterns": "disabled",
            "secret_scanning_validity_checks": "disabled"
        })
    );
    assert_eq!(
        canonical["clean_baseline"],
        serde_json::json!({
            "require_readable_dependency_graph": true,
            "require_successful_default_branch_codeql_analyses": ["actions", "rust"],
            "require_repository_visible_secret_alert_endpoint": true,
            "require_zero_open_dependabot_alerts": true,
            "require_zero_open_code_scanning_alerts": true,
            "require_zero_open_secret_scanning_alerts": true
        })
    );

    let unavailable = canonical["unavailable_features"]
        .as_array()
        .expect("unavailable features should be an array")
        .iter()
        .map(|entry| {
            entry["feature"]
                .as_str()
                .expect("feature should be a string")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        unavailable,
        [
            "secret_scanning_validity_checks",
            "secret_scanning_scan_history_readback",
            "secret_scanning_non_provider_and_generic_patterns",
            "ai_generic_secret_detection",
            "delegated_push_protection_bypass",
            "enterprise_public_leak_monitoring",
        ]
    );

    let excluded = canonical["excluded_evidence"]
        .as_array()
        .expect("excluded evidence should be an array")
        .iter()
        .map(|entry| {
            entry["surface"]
                .as_str()
                .expect("surface should be a string")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        excluded,
        [
            "partner_only_public_repository_secret_alerts",
            "mcp_doctor_product_security_scanner",
            "complete_assurance_baseline",
        ]
    );
}

#[test]
fn security_control_verifier_is_authenticated_bounded_and_non_disclosing() {
    let verifier = repository_file("scripts/verify-security-controls.sh");

    for contract in [
        "GH_PROMPT_DISABLED=1 GH_PAGER=cat gh api",
        "umask 077",
        "mktemp -d",
        "trap cleanup EXIT",
        ">\"$destination\" 2>/dev/null",
        "contents/SECURITY.md?ref=$default_branch",
        "private-vulnerability-reporting",
        "code-scanning/default-setup",
        "code-scanning/analyses?ref=refs/heads/$default_branch&tool_name=CodeQL&per_page=100",
        "dependabot/alerts?state=open&per_page=1",
        "code-scanning/alerts?state=open&per_page=1",
        "secret-scanning/alerts?state=open&hide_secret=true&per_page=1",
        "dependency-graph/sbom",
        ".commit_sha == $sha",
        ".category == (\"/language:\" + $language)",
        "date=%s canonical_sha256=%s result=FAIL",
        "date=%s canonical_sha256=%s result=PASS",
    ] {
        assert!(
            verifier.contains(contract),
            "security verifier should preserve {contract}"
        );
    }
    for forbidden in [
        "set -x",
        "--verbose",
        "jq '.'",
        "hide_secret=false",
        "secret-scanning/scan-history",
    ] {
        assert!(
            !verifier.contains(forbidden),
            "security verifier must not contain {forbidden}"
        );
    }
}

#[test]
fn community_license_projection_matches_the_public_scope_contract() {
    let canonical = repository_file(".github/community-license-controls.json");
    let canonical: serde_json::Value = serde_json::from_str(&canonical)
        .expect("community and license controls should be valid JSON");

    assert_eq!(
        canonical["schema_version"],
        "mcp-doctor.github-community-license-controls/v1"
    );
    assert_eq!(canonical["reviewed_on"], "2026-08-24");
    assert_eq!(canonical["api_version"], "2026-03-10");
    assert_eq!(canonical["organization"], "EnjoyableWork");
    assert_eq!(canonical["project_repository"], "EnjoyableWork/mcp-doctor");

    let inventory = canonical["public_repository_inventory"]
        .as_array()
        .expect("repository inventory should be an array");
    assert_eq!(inventory.len(), 3);
    let inventory_contract = inventory
        .iter()
        .map(|entry| {
            (
                entry["repository"]
                    .as_str()
                    .expect("repository should be a string"),
                entry["classification"]
                    .as_str()
                    .expect("classification should be a string"),
                entry["license"]
                    .as_str()
                    .expect("license should be a string"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        inventory_contract,
        [
            ("EnjoyableWork/homebrew-tap", "in_scope_distribution", "MIT"),
            ("EnjoyableWork/mcp-doctor", "in_scope_primary", "MIT"),
            ("EnjoyableWork/mcp-sync", "separate_project", "MIT"),
        ]
    );

    assert_eq!(
        canonical["community_contract"]["public_discussion_uri"],
        "https://github.com/EnjoyableWork/mcp-doctor/issues"
    );
    assert_eq!(
        canonical["community_contract"]["issue_intake_uri"],
        "https://github.com/EnjoyableWork/mcp-doctor/issues/new/choose"
    );
    assert_eq!(
        canonical["community_contract"]["private_vulnerability_uri"],
        "https://github.com/EnjoyableWork/mcp-doctor/security/advisories/new"
    );
    assert_eq!(
        canonical["community_contract"]["content_reports_enabled"],
        true
    );
    assert_eq!(
        canonical["community_contract"]["blank_issues_enabled"],
        false
    );
    assert_eq!(canonical["community_contract"]["inbound_license"], "MIT");
    assert_eq!(canonical["community_contract"]["outbound_license"], "MIT");

    let official_channels = canonical["official_channels"]
        .as_array()
        .expect("official channels should be an array");
    assert_eq!(official_channels.len(), 8);
    assert!(official_channels.iter().all(|entry| {
        entry["uri"]
            .as_str()
            .is_some_and(|uri| uri.starts_with("https://") && !uri.contains('@'))
    }));
    assert!(official_channels.iter().any(|entry| {
        entry["channel"] == "third_party_agent_skill_registry"
            && entry["uri"] == "https://smithery.ai/skills/enjoyable/mcp-doctor"
    }));

    assert_eq!(canonical["source_license"]["spdx_expression"], "MIT");
    assert_eq!(
        canonical["source_license"]["license_sha256"],
        "32a82b79c71a3a633dc51fcb306f0d4768551aaff7c8862f67a5997a5f75faea"
    );
    assert_eq!(
        canonical["tap_contract"]["reviewed_commit"],
        "2b62e11902c7461cddbc0b96075e3745fdf6f260"
    );
    assert_eq!(
        canonical["release_license_contract"]["source_commit"],
        "d9b96bbeb84baccb8e5c890e9c655a559a12a474"
    );
    assert_eq!(canonical["release_license_contract"]["version"], "0.3.0");
    assert_eq!(canonical["release_license_contract"]["tag"], "v0.3.0");
    assert_eq!(
        canonical["release_license_contract"]["spdx_expression"],
        "MIT"
    );

    let assets = canonical["release_license_contract"]["assets"]
        .as_array()
        .expect("release assets should be an array");
    assert_eq!(assets.len(), 7);
    assert!(assets.iter().all(|asset| {
        asset["bytes"].as_u64().is_some_and(|bytes| bytes > 0)
            && asset["sha256"]
                .as_str()
                .is_some_and(|digest| digest.len() == 64)
    }));
    assert_eq!(
        canonical["limitations"][0]["surface"],
        "sbom_project_package_license"
    );
    assert_eq!(
        canonical["limitations"][0]["state"],
        "not_used_as_mit_evidence"
    );
    assert_eq!(
        canonical["mapped_controls"],
        serde_json::json!([
            "OSPS-BR-03.01",
            "OSPS-DO-02.01",
            "OSPS-GV-02.01",
            "OSPS-GV-03.01",
            "OSPS-LE-02.01",
            "OSPS-LE-02.02",
            "OSPS-LE-03.01",
            "OSPS-LE-03.02",
            "OSPS-QA-04.01"
        ])
    );
}

#[test]
fn community_routes_are_reachable_by_contract_and_keep_sensitive_intake_private() {
    let conduct = repository_file("CODE_OF_CONDUCT.md");
    let contributing = repository_file("CONTRIBUTING.md");
    let support = repository_file("SUPPORT.md");
    let readme = repository_file("README.md");
    let scope = repository_file("docs/project-scope.md");
    let bug_form = repository_file(".github/ISSUE_TEMPLATE/01-bug-report.yml");

    for contract in [
        "private **Report content** action",
        "Repository content reporting is enabled",
        "https://support.github.com/contact/report-abuse",
        "[SECURITY.md](SECURITY.md)",
    ] {
        assert!(
            conduct.contains(contract),
            "conduct policy should preserve {contract}"
        );
    }
    for contract in [
        "same inbound and outbound terms",
        "requires neither a\nContributor License Agreement",
        "right to license",
    ] {
        assert!(
            contributing.contains(contract),
            "contribution policy should preserve {contract}"
        );
    }
    for contract in [
        "This source tree represents `0.4.2`",
        "a\nversion is publicly available only when its canonical GitHub Release and\nchannel evidence exist",
        "issues/new?template=01-bug-report.yml",
        "issues/new?template=02-feature-request.yml",
        "Suspected vulnerabilities and accidentally exposed secrets",
    ] {
        assert!(
            support.contains(contract),
            "support guide should preserve {contract}"
        );
    }
    assert!(!support.contains("project is pre-release"));
    assert!(bug_form.contains("placeholder: mcp-doctor 0.4.2 or commit SHA"));
    assert!(readme.contains("[project scope](docs/project-scope.md)"));
    for contract in [
        "## In-scope repositories",
        "## Community and defect routes",
        "## Official channels",
        "## License evidence",
        "A new or unclassified public\norganization repository makes the verifier fail",
        "The two immutable SPDX documents use `CC0-1.0`",
        "They are therefore not used as proof of the software's MIT license.",
        "does not authenticate the supply chain",
        "https://smithery.ai/skills/enjoyable/mcp-doctor",
        "the GitHub skill directory remains canonical",
    ] {
        assert!(
            scope.contains(contract),
            "scope guide should preserve {contract}"
        );
    }
}

#[test]
fn community_license_verifier_is_credential_free_bounded_and_exact() {
    let verifier = repository_file("scripts/verify-community-license.sh");

    for contract in [
        "--source-ref main|40-hex-commit",
        "env \\",
        "-u GITHUB_TOKEN",
        "-u GH_TOKEN",
        "curl --disable",
        "--proto '=https'",
        "--proto-redir '=https'",
        "--proxy ''",
        "--max-filesize",
        "--connect-timeout 10",
        "umask 077",
        "mktemp -d",
        "trap community_cleanup EXIT",
        "orgs/${community_organization}/repos?type=public&per_page=100",
        "community/profile",
        "raw.githubusercontent.com",
        "content_reports_enabled == true",
        "releases/tags/${community_tag}",
        ".immutable == true",
        "git/ref/tags/${community_tag}",
        "static.crates.io",
        "third_party_agent_skill_registry",
        "version.license == \"MIT\"",
        "tar -xOzf",
        ".dataLicense == \"CC0-1.0\"",
        ".licenseDeclared == \"NOASSERTION\"",
        "grep -Fx '  license \"MIT\"'",
        "date=%s canonical_sha256=%s source_sha=%s result=FAIL",
        "date=%s canonical_sha256=%s source_sha=%s result=PASS",
    ] {
        assert!(
            verifier.contains(contract),
            "community and license verifier should preserve {contract}"
        );
    }
    for forbidden in [
        "set -x",
        "Authorization: Bearer",
        "curl --netrc",
        "gh api",
        "http://",
    ] {
        assert!(
            !verifier.contains(forbidden),
            "credential-free verifier must not contain {forbidden}"
        );
    }
}

#[test]
fn main_protection_canonical_config_matches_the_branch_contract() {
    let canonical = repository_file(".github/rulesets/main.json");
    let canonical: serde_json::Value =
        serde_json::from_str(&canonical).expect("main protection config should be valid JSON");

    assert_eq!(
        canonical["schema_version"],
        "mcp-doctor.github-main-protection/v1"
    );
    assert_eq!(canonical["api_version"], "2026-03-10");
    assert_eq!(canonical["repository"], "EnjoyableWork/mcp-doctor");
    assert_eq!(canonical["default_branch"], "main");
    assert_eq!(
        canonical["merge_settings"],
        serde_json::json!({
            "allow_squash_merge": true,
            "allow_merge_commit": false,
            "allow_rebase_merge": false,
            "allow_auto_merge": false,
            "delete_branch_on_merge": true,
            "allow_update_branch": true,
            "squash_merge_commit_title": "PR_TITLE",
            "squash_merge_commit_message": "PR_BODY"
        })
    );

    let ruleset = &canonical["ruleset"];
    assert_eq!(ruleset["name"], "Protect main");
    assert_eq!(ruleset["target"], "branch");
    assert_eq!(ruleset["enforcement"], "active");
    assert_eq!(ruleset["bypass_actors"], serde_json::json!([]));
    assert_eq!(
        ruleset["conditions"],
        serde_json::json!({
            "ref_name": {
                "include": ["refs/heads/main"],
                "exclude": []
            }
        })
    );

    let rules = ruleset["rules"]
        .as_array()
        .expect("canonical rules should be an array");
    assert_eq!(rules.len(), 5);
    for required_type in [
        "deletion",
        "non_fast_forward",
        "required_linear_history",
        "pull_request",
        "required_status_checks",
    ] {
        assert!(
            rules.iter().any(|rule| rule["type"] == required_type),
            "canonical ruleset should contain {required_type}"
        );
    }
    for forbidden_type in ["creation", "update", "required_signatures", "merge_queue"] {
        assert!(
            rules.iter().all(|rule| rule["type"] != forbidden_type),
            "canonical ruleset must not contain {forbidden_type}"
        );
    }

    let pull_request = rules
        .iter()
        .find(|rule| rule["type"] == "pull_request")
        .expect("pull-request rule should exist");
    assert_eq!(
        pull_request["parameters"],
        serde_json::json!({
            "allowed_merge_methods": ["squash"],
            "dismiss_stale_reviews_on_push": false,
            "dismissal_restriction": {
                "allowed_actors": [],
                "enabled": false
            },
            "require_code_owner_review": false,
            "require_last_push_approval": false,
            "required_approving_review_count": 0,
            "required_review_thread_resolution": true,
            "required_reviewers": []
        })
    );

    let required_status_checks = rules
        .iter()
        .find(|rule| rule["type"] == "required_status_checks")
        .expect("required-status-check rule should exist");
    assert_eq!(
        required_status_checks["parameters"],
        serde_json::json!({
            "do_not_enforce_on_create": false,
            "required_status_checks": [
                {"context": "Required CI", "integration_id": 15368},
                {"context": "Required release preflight", "integration_id": 15368}
            ],
            "strict_required_status_checks_policy": true
        })
    );
}

#[test]
fn required_aggregate_jobs_cannot_hide_unsuccessful_dependencies() {
    let ci = repository_file(".github/workflows/ci.yml");
    let preflight = repository_file(".github/workflows/release-preflight.yml");

    for workflow in [&ci, &preflight] {
        assert!(workflow.contains("push:\n    branches:\n      - main"));
    }

    for contract in [
        "required-ci:\n    name: Required CI\n    if: always()",
        "needs:\n      - dependencies\n      - unix-quality\n      - windows-quality",
        "DEPENDENCIES_RESULT: ${{ needs.dependencies.result }}",
        "UNIX_QUALITY_RESULT: ${{ needs.unix-quality.result }}",
        "WINDOWS_QUALITY_RESULT: ${{ needs.windows-quality.result }}",
        "test \"$DEPENDENCIES_RESULT\" = success",
        "test \"$UNIX_QUALITY_RESULT\" = success",
        "test \"$WINDOWS_QUALITY_RESULT\" = success",
    ] {
        assert!(ci.contains(contract), "CI should preserve {contract}");
    }

    for contract in [
        "required-release-preflight:\n    name: Required release preflight\n    if: always()",
        "needs:\n      - source\n      - unix\n      - windows\n      - payload",
        "SOURCE_RESULT: ${{ needs.source.result }}",
        "UNIX_RESULT: ${{ needs.unix.result }}",
        "WINDOWS_RESULT: ${{ needs.windows.result }}",
        "PAYLOAD_RESULT: ${{ needs.payload.result }}",
        "test \"$SOURCE_RESULT\" = success",
        "test \"$UNIX_RESULT\" = success",
        "test \"$WINDOWS_RESULT\" = success",
        "test \"$PAYLOAD_RESULT\" = success",
    ] {
        assert!(
            preflight.contains(contract),
            "release preflight should preserve {contract}"
        );
    }
}

#[test]
fn protection_verifiers_keep_public_and_private_evidence_separate() {
    let public = repository_file("scripts/verify-main-protection-public.sh");
    let admin = repository_file("scripts/verify-main-protection-admin.sh");

    for contract in [
        "-u GH_TOKEN",
        "-u GITHUB_TOKEN",
        "curl --disable",
        "--noproxy '*'",
        "--max-filesize 1048576",
        "rules/branches/$default_branch?per_page=100",
        "has(\"bypass_actors\") | not",
        "effective main rules include an unexpected layer or source",
    ] {
        assert!(
            public.contains(contract),
            "public verifier should preserve {contract}"
        );
    }
    for forbidden in ["gh api", "Authorization:", "set -x"] {
        assert!(
            !public.contains(forbidden),
            "public verifier must not contain {forbidden}"
        );
    }
    for authenticated_only_field in [
        "allow_squash_merge",
        "allow_merge_commit",
        "allow_rebase_merge",
        "delete_branch_on_merge",
    ] {
        assert!(
            !public.contains(authenticated_only_field),
            "public verifier must not claim unauthenticated access to {authenticated_only_field}"
        );
    }

    for contract in [
        "GH_PROMPT_DISABLED=1 GH_PAGER=cat gh api",
        ">\"$repository_path\" 2>/dev/null",
        ">\"$rulesets_path\" 2>/dev/null",
        ">\"$ruleset_path\" 2>/dev/null",
        "allow_squash_merge",
        "allow_merge_commit",
        "allow_rebase_merge",
        "allow_auto_merge",
        "delete_branch_on_merge",
        "allow_update_branch",
        "squash_merge_commit_title",
        "squash_merge_commit_message",
        "cmp -s \"$canonical_repository_projection\" \"$live_repository_projection\"",
        "jq -e '.bypass_actors == []'",
        "date=%s canonical_sha256=%s result=FAIL",
        "date=%s canonical_sha256=%s result=PASS",
    ] {
        assert!(
            admin.contains(contract),
            "admin verifier should preserve {contract}"
        );
    }
    for forbidden in ["set -x", "--verbose", "jq '.'"] {
        assert!(
            !admin.contains(forbidden),
            "admin verifier must not contain {forbidden}"
        );
    }
}

#[test]
fn protocol_support_preserves_the_active_legacy_boundary() {
    let protocol_support = repository_file("docs/protocol-support.md");
    let compatibility = repository_file("tests/compatibility/README.md");
    let agents = repository_file("AGENTS.md");

    for contract in [
        "--protocol-version 2025-06-18",
        "| `2025-06-18` | 8.1% | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ❌ <!-- mcp-doctor-support=unsupported --> |",
        "every advertised output schema that `mcp-doctor` interprets",
        "Active MCP `2025-06-18`",
        "No broad legacy ecosystem claim follows",
    ] {
        assert!(
            protocol_support.contains(contract),
            "the protocol guide should retain the scoped active legacy contract: {contract}"
        );
    }
    assert!(!protocol_support.contains("MCP `2025-06-18` remains passive-only"));
    assert!(compatibility.contains("### Active MCP 2025-06-18"));
    assert!(compatibility.contains("does not add a `2025-06-18` case"));
    assert!(agents.contains("exact-selected `check` and\n  `break`"));
    assert!(agents.contains("exact supported Draft\n  2020-12 declaration"));
}

#[test]
fn repository_only_references_the_scaffolding_project_in_the_explicit_inventory() {
    let forbidden = ["mcp", "sync"].join("-");
    let allowed_inventory_files = [
        ".github/community-license-controls.json",
        "docs/project-scope.md",
        "scripts/verify-community-license.sh",
        "tests/release.rs",
    ]
    .map(|relative| repository_root().join(relative));
    let roots = [
        "README.md",
        "AGENTS.md",
        "Cargo.toml",
        ".github",
        "docs",
        "scripts",
        "src",
        "tests",
    ];

    for root in roots {
        inspect_text_path(
            &repository_root().join(root),
            &forbidden,
            &allowed_inventory_files,
            "a reference to the scaffolding project",
        );
    }
}

#[test]
fn public_repository_text_does_not_depend_on_private_coordination() {
    let allowed_inventory_files: [PathBuf; 0] = [];
    let roots = [
        "README.md",
        "AGENTS.md",
        "CONTRIBUTING.md",
        "SECURITY.md",
        "SUPPORT.md",
        "Cargo.toml",
        ".agents",
        ".github",
        "docs",
        "schemas",
        "scripts",
        "src",
        "tests",
    ];
    let forbidden = [
        ["linear", ".app"].join(""),
        ["enj", "-"].join(""),
        ["governing", " linear"].join(""),
        ["linear", " project"].join(""),
        ["linear", " issue"].join(""),
        ["project", ".md"].join(""),
        ["mcpd", "-"].join(""),
        ["dec", "-"].join(""),
    ];

    for root in roots {
        for value in &forbidden {
            inspect_text_path(
                &repository_root().join(root),
                value,
                &allowed_inventory_files,
                "private coordination context",
            );
        }
    }

    let agents = repository_file("AGENTS.md");
    for required in [
        "### Public context boundary",
        "publicly accessible sources alone",
        "Source comments must explain the invariant",
    ] {
        assert!(
            agents.contains(required),
            "AGENTS.md must preserve {required}"
        );
    }
    for obsolete_runtime_instruction in [
        ["persistent", " goals"].join(""),
        ["thread", " goal"].join(""),
        ["token", " budget"].join(""),
    ] {
        assert!(
            !agents
                .to_ascii_lowercase()
                .contains(&obsolete_runtime_instruction),
            "AGENTS.md contains an obsolete coding-agent runtime instruction"
        );
    }
}

#[test]
fn repository_guidance_requires_independent_comparative_evaluation() {
    let agents = repository_file("AGENTS.md");

    for required in [
        "### Comparative evaluation independence",
        "must not\npredict a row state, point value, score delta, total, or rank",
        "neutral evidence pack that omits prior and proposed scores",
        "before deriving\nany subtotal, total, or ranking",
        "have a separate reviewer verify row completeness and\narithmetic",
        "reassign the evaluation or label the result\nnon-independent",
        "must not support a public comparative claim, leaderboard, badge, or delivery\ntarget",
        "never give them to a future evaluator\nbefore row lock",
    ] {
        assert!(
            agents.contains(required),
            "AGENTS.md must preserve comparative-evaluation rule: {required}"
        );
    }
}

fn inspect_text_path(
    path: &Path,
    forbidden: &str,
    allowed_inventory_files: &[PathBuf],
    description: &str,
) {
    if allowed_inventory_files
        .iter()
        .any(|allowed| allowed == path)
    {
        return;
    }
    assert!(
        !path
            .to_string_lossy()
            .to_ascii_lowercase()
            .contains(forbidden),
        "{} path contains {description}",
        path.display(),
    );
    if path.is_dir() {
        for entry in fs::read_dir(path)
            .unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()))
        {
            let entry = entry.expect("repository directory entry should be readable");
            inspect_text_path(
                &entry.path(),
                forbidden,
                allowed_inventory_files,
                description,
            );
        }
        return;
    }

    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };
    assert!(
        !contents.to_ascii_lowercase().contains(forbidden),
        "{} contains {description}",
        path.display(),
    );
}

#[test]
fn release_version_constant_matches_the_current_version() {
    assert_eq!(CANDIDATE_RELEASE_VERSION, "0.4.2");
}
