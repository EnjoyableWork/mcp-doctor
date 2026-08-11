use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const CURRENT_RELEASE_VERSION: &str = "0.2.0";
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
        "version = \"0.2.0\"",
        "publish = [\"crates-io\"]",
        "repository = \"https://github.com/EnjoyableWork/mcp-doctor\"",
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
        "scripts/rehearse-release-handoffs.sh",
        "synthetic-rehearsal",
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
        "group: mcp-doctor-release",
        "scripts/validate-release-version.sh",
        "published_stable_versions",
        "all(.versions[];",
        "cargo package --locked",
        "cargo publish --locked --package mcp-doctor",
        "scripts/generate-release-channels.sh",
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
    assert_actions_are_commit_pinned(&workflow);
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
        &["future", "0.1.1", "0.1.1", "0.1.0"],
        &["future", "v01.2.3", "01.2.3", "0.1.0"],
        &["future", "v1.2.3-rc.1", "1.2.3-rc.1", "0.1.0"],
        &["future", "v1.2.3", "1.2.3", "not-stable"],
        &["published", "v0.0.99", "0.0.99"],
        &["rehearsal", "v0.1.1", "0.1.1"],
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
fn release_docs_keep_scope_and_adoption_evidence_honest() {
    let release = repository_file("docs/release.md");
    let first_notes = repository_file("docs/releases/v0.1.0.md");
    let current_notes = repository_file("docs/releases/v0.2.0.md");
    let adoption = repository_file("docs/adoption.md");

    assert!(release.contains("exactly these seven assets"));
    assert!(release.contains("never replace an"));
    assert!(release.contains("asset, move the tag, or overwrite downstream bytes"));
    assert!(release.contains("does not issue macOS or Windows binaries"));
    assert!(release.contains("Workflow filename | `release.yml`"));
    assert!(release.contains("Environment | `release`"));
    assert!(release.contains("No publish command exists in the authorization job"));
    assert!(release.contains("cross-repository personal token"));
    assert!(release.contains("test alone is not completion evidence"));
    assert!(first_notes.contains("does not call tools"));
    assert!(first_notes.contains("does not call tools, connect to remote HTTP endpoints"));
    for contract in [
        "mcp-doctor.report/v1",
        "JUnit",
        "--allow-tool",
        "does not add SARIF",
        "does not become a general security scanner",
    ] {
        assert!(
            current_notes.contains(contract),
            "current release notes should preserve {contract}"
        );
    }
    assert!(adoption.contains("Opened: 2026-08-10"));
    assert!(adoption.contains("Closed: 2026-08-10"));
    assert!(adoption.contains("zero independent adoption reports at opening"));
    assert!(adoption.contains("no adoption or repeat-use claim"));
    assert!(adoption.contains("does not block M3"));
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
fn project_keeps_mcpd_009_active_authorization_boundary_explicit() {
    let project = repository_file("PROJECT.md");
    let agents = repository_file("AGENTS.md");

    for contract in [
        "mcp-doctor.scenario/v1alpha1",
        "RFC 6901",
        "`read_only` or `side_effecting`",
        "`--allow-tool <exact-name>`",
        "`--allow-side-effects`",
        "An `input_required` result is recorded as incomplete",
        "| MCPD-009 | Add explicit, budgeted, deterministic `check` scenario replay and result-schema validation | M3 | Done |",
    ] {
        assert!(
            project.contains(contract),
            "PROJECT.md should retain MCPD-009 contract: {contract}"
        );
    }

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
fn project_keeps_mcpd_010_network_boundary_explicit() {
    let project = repository_file("PROJECT.md");
    let agents = repository_file("AGENTS.md");

    for contract in [
        "| DEC-030 | Resolve `OPEN-06` with one direct, pinned, credential-scoped Streamable HTTP endpoint | Accepted |",
        "`--allow-private-network <exact-url>`",
        "`--allow-cleartext-http <exact-url>`",
        "`--allow-credentials-to <exact-url>`",
        "outside all reviewed IANA special-purpose blocks",
        "Name resolution runs once",
        "Redirects and application retries remain exactly zero",
        "ignores `HTTP_PROXY`, `HTTPS_PROXY`",
        "requires TLS 1.2 or 1.3",
        "does not fetch `resource_metadata`",
        "`Mcp-Param-*`",
        "| MCPD-010 | Add a bounded Streamable HTTP transport with explicit remote-target and credential policy | M3 | Done |",
    ] {
        assert!(
            project.contains(contract),
            "PROJECT.md should retain MCPD-010 network contract: {contract}"
        );
    }
    assert!(
        !project.contains("| OPEN-06 |"),
        "accepted OPEN-06 should leave the open-decision table"
    );

    for contract in [
        "Follow `DEC-030` for Streamable HTTP",
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
fn project_keeps_mcpd_011_generation_boundary_explicit() {
    let project = repository_file("PROJECT.md");
    let readme = repository_file("README.md");

    for contract in [
        "| DEC-031 | Generate only versioned, bounded, schema-valid cases for one redundantly authorized tool | Accepted |",
        "`mcp-doctor.generator/v1`",
        "`--tool <exact-name>` and `--allow-tool <exact-name>`",
        "256 synthesis attempts, 64 retained",
        "100,000 synthesis steps",
        "`MCP-GENERATION-001`",
        "Case `n` uses the base seed with wrapping addition",
        "| MCPD-011 | Add the bounded adversarial `break` command for authorized tools | M3 | Done |",
        "`proptest` or another property framework | `MCPD-011` — evaluated 2026-08-11 | Not adopted",
    ] {
        assert!(
            project.contains(contract),
            "PROJECT.md should retain MCPD-011 generation contract: {contract}"
        );
    }

    for contract in [
        "--tool search",
        "--allow-tool search",
        "--effects read_only",
        "--cases 50",
        "--seed 4242",
        "A `side_effecting` run also requires `--allow-side-effects`",
        "reports never contain raw generated arguments or tool results",
    ] {
        assert!(
            readme.contains(contract),
            "README.md should describe the bounded break contract: {contract}"
        );
    }
}

#[test]
fn project_resolves_open_07_with_stable_json_and_junit_without_security_scanner_scope() {
    let project = repository_file("PROJECT.md");

    for contract in [
        "| DEC-032 | Resolve `OPEN-07` with stable vendor-neutral JSON and a JUnit-compatible CI projection | Accepted |",
        "`mcp-doctor.report/v1`",
        "Stable `v1` JSON is the authoritative, vendor-neutral automation contract.",
        "ignore unknown optional fields and handle a previously unknown finding code",
        "JUnit is a projection of that same immutable result",
        "produced without rerunning a target",
        "process exit status remains the CI",
        "SARIF is deferred",
        "not a general\nsecurity scanner",
        "Keep SARIF and general security-scanner positioning out of scope.",
    ] {
        assert!(
            project.contains(contract),
            "PROJECT.md should retain DEC-032 reporting contract: {contract}"
        );
    }

    assert!(
        !project.contains("| OPEN-07 |"),
        "accepted OPEN-07 should leave the open-decision table"
    );
}

#[test]
fn project_resolves_open_08_with_locked_assurance_versions_and_exact_proof() {
    let project = repository_file("PROJECT.md");

    for contract in [
        "| DEC-034 | Resolve `OPEN-08` with one activation-locked assurance version set and exact proof routes | Accepted |",
        "OpenSSF OSPS Baseline `v2026.02.19`",
        "official BadgeApp baseline series displaying OSPS `v2026.02.19`",
        "approved [SLSA `v1.2`]",
        "M4 never silently floats, mixes framework versions",
        "`docs/assurance/osps-v2026.02.19-level-1.md`",
        "The result is an official-hosted self-assessment, not independent certification.",
        "`docs/assurance/slsa-v1.2-build-l2.md`",
        "exact repository, signer workflow, tag ref, and source commit",
        "`predicateType` is `https://slsa.dev/provenance/v1`",
        "publication of the assessment are explicit owner actions",
    ] {
        assert!(
            project.contains(contract),
            "PROJECT.md should retain DEC-034 assurance contract: {contract}"
        );
    }

    assert!(
        !project.contains("| OPEN-08 |"),
        "accepted OPEN-08 should leave the open-decision table"
    );
}

#[test]
fn project_resolves_open_09_with_a_single_maintainer_branch_policy() {
    let project = repository_file("PROJECT.md");

    for contract in [
        "| DEC-035 | Resolve `OPEN-09` with a usable single-maintainer default-branch policy | Accepted |",
        "`required_approving_review_count` to `0`",
        "`Required CI` and `Required release preflight`",
        "Both aggregate jobs use `needs` with `always()`",
        "GitHub omits bypass actors from credential-free REST readback",
        "authenticated owner check must verify the exact empty list",
        "a future `mcp-doctor` MCP security scanner is product behavior rather than a repository check by default",
        "Enable squash merge only",
        "Keep auto-merge and merge queue disabled",
        "Keep `bypass_actors` empty",
        "temporarily add only the repository-administrator role with `pull_request` bypass mode",
        "Never disable the ruleset, grant `always` bypass, push directly, delete `main`, or force-push.",
        "Required commit signing stays off.",
        "no ruleset or legacy branch protection on `main`",
        "resolving this policy does not activate M4 early",
    ] {
        assert!(
            project.contains(contract),
            "PROJECT.md should retain DEC-035 default-branch contract: {contract}"
        );
    }

    assert!(
        !project.contains("| OPEN-09 |"),
        "accepted OPEN-09 should leave the open-decision table"
    );
}

#[test]
fn project_keeps_a_result_free_dynamic_product_evaluation_method() {
    let project = repository_file("PROJECT.md");

    for contract in [
        "## Product category and comparative evaluation",
        "safety-bounded MCP server-author diagnostic\npreflight",
        "does not retain a current score, ranking,\nwinner, or market-dominance claim",
        "| Causal diagnosis and remediation | 18 |",
        "| Protocol and contract correctness | 15 |",
        "| Runtime testing and reproducibility | 15 |",
        "| Safety and containment | 17 |",
        "| CI and machine interoperability | 10 |",
        "| Adoption UX and integration reach | 10 |",
        "| Security-vulnerability detection | 10 |",
        "| Release and project assurance | 5 |",
        "| **Total** | **100** |",
        "`weight * rating / 5`",
        "| `P` | Accepted plan or documented product intention only |",
        "| `L` | Exact source implementation with local, reproducible test evidence |",
        "| `H` | Exact source verified by project-hosted automation on a named host |",
        "| `R` | Exact immutable release artifact reproduced on a claimed platform |",
        "| `I` | Exact behavior independently reproduced, or sustained use independently evidenced |",
        "cap the reported total at 49",
        "90–100",
        "These are capability bands, not market-adoption or market-dominance bands.",
        "### Dynamic assessment procedure",
        "### Seed comparison set",
        "https://github.com/destilabs/mcp-doctor",
        "https://github.com/realwigu/mcp-doctor",
        "https://github.com/Jiansen/mcp-doctor",
        "https://github.com/stephenywilson/MCP-Doctor",
        "https://github.com/modelcontextprotocol/inspector",
        "https://github.com/modelcontextprotocol/conformance",
        "https://github.com/MCPJam/inspector",
        "https://github.com/cisco-ai-defense/mcp-scanner",
        "https://github.com/snyk/agent-scan",
        "https://github.com/ModelContextProtocol-Security/mcpserver-audit",
        "| DEC-033 | Retain a result-free weighted product and market evaluation method | Accepted |",
    ] {
        assert!(
            project.contains(contract),
            "PROJECT.md should retain the dynamic product-evaluation contract: {contract}"
        );
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
fn release_version_constant_matches_the_current_version() {
    assert_eq!(CURRENT_RELEASE_VERSION, "0.2.0");
}
