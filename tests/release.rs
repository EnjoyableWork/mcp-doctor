use std::fs;
use std::path::{Path, PathBuf};

const RELEASE_VERSION: &str = "0.1.0";
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
        "version = \"0.1.0\"",
        "publish = [\"crates-io\"]",
        "repository = \"https://github.com/EnjoyableWork/mcp-doctor\"",
        "\"/.github/workflows/*.yml\"",
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
fn preflight_is_secretless_nonpublishing_and_covers_every_source_host() {
    let workflow = repository_file(".github/workflows/release-preflight.yml");

    for target in SOURCE_TARGETS {
        assert!(workflow.contains(target), "preflight should cover {target}");
    }
    for contract in [
        "cargo package --locked",
        "scripts/generate-release-channels.sh",
        "scripts/package-release.sh",
        "scripts/smoke-installed.sh",
        "scripts/smoke-installed.ps1",
        "scripts/smoke-archive.sh",
        "scripts/verify-release-assets.sh",
        "scripts/verify-published-release.sh",
        "brew install --build-from-source",
        "syft-version: v1.50.0",
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
fn tag_workflow_publishes_only_attested_source_and_linux_outputs() {
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
        "tags:\n      - v0.1.0",
        "cargo package --locked",
        "scripts/generate-release-channels.sh",
        "scripts/verify-release-assets.sh",
        "scripts/verify-published-release.sh",
        "actions/attest-build-provenance@",
        "gh attestation verify",
        "gh release create",
        "--draft",
        "gh release verify",
        ".immutable",
        "contents: write",
        "id-token: write",
        "attestations: write",
    ] {
        assert!(
            workflow.contains(contract),
            "release should enforce {contract}"
        );
    }
    for forbidden in ["secrets.", "cargo publish", "brew install", "winget"] {
        assert!(
            !workflow.contains(forbidden),
            "release must not contain {forbidden}"
        );
    }
    assert_actions_are_commit_pinned(&workflow);
}

#[test]
fn published_channel_verifier_is_read_only_and_runs_passive_installed_smokes() {
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
    assert_actions_are_commit_pinned(&workflow);
}

#[test]
fn generator_and_verifiers_enforce_the_exact_source_built_release() {
    let generator = repository_file("scripts/generate-release-channels.sh");
    let asset_verifier = repository_file("scripts/verify-release-assets.sh");
    let published_verifier = repository_file("scripts/verify-published-release.sh");
    let archive_packager = repository_file("scripts/package-release.sh");

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
}

#[test]
fn release_docs_keep_scope_and_adoption_evidence_honest() {
    let release = repository_file("docs/release.md");
    let notes = repository_file("docs/releases/v0.1.0.md");
    let adoption = repository_file("docs/adoption.md");

    assert!(release.contains("exactly these seven assets"));
    assert!(release.contains("never replace an"));
    assert!(release.contains("asset, move the tag, or overwrite downstream bytes"));
    assert!(release.contains("does not issue macOS or Windows binaries"));
    assert!(notes.contains("does not call tools"));
    assert!(notes.contains("does not call tools, connect to remote HTTP endpoints"));
    assert!(adoption.contains("Opened: 2026-08-10"));
    assert!(adoption.contains("zero independent adoption reports at opening"));
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
fn repository_does_not_reference_the_scaffolding_project() {
    let forbidden = ["mcp", "sync"].join("-");
    let roots = [
        "README.md",
        "PROJECT.md",
        "AGENTS.md",
        "Cargo.toml",
        ".github",
        "docs",
        "scripts",
        "src",
        "tests",
    ];

    for root in roots {
        inspect_text_path(&repository_root().join(root), &forbidden);
    }
}

fn inspect_text_path(path: &Path, forbidden: &str) {
    if path.is_dir() {
        for entry in fs::read_dir(path)
            .unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()))
        {
            let entry = entry.expect("repository directory entry should be readable");
            inspect_text_path(&entry.path(), forbidden);
        }
        return;
    }

    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };
    assert!(
        !contents.to_ascii_lowercase().contains(forbidden),
        "{} contains a reference to the scaffolding project",
        path.display()
    );
}

#[test]
fn release_version_constant_matches_the_first_version() {
    assert_eq!(RELEASE_VERSION, "0.1.0");
}
