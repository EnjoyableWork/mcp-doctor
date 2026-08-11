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
    for contract in [
        "The current public release is `mcp-doctor` `0.2.0`",
        "b0805a8f685e46814e358de368e2a270c21704af",
        "31528649356",
        "31528649333",
        "31529740214",
        "31530330361",
        "a57736ea1a7abf73eeff9a8278af11110247bd20",
        "31530466930",
        "passed all ten jobs",
    ] {
        assert!(
            release.contains(contract),
            "release guide should retain v0.2.0 evidence: {contract}"
        );
    }
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
fn project_records_m3_completion_against_exact_v020_evidence() {
    let project = repository_file("PROJECT.md");

    for contract in [
        "| Current milestone | M4 — enterprise assurance and adoption; `MCPD-014` is Done and `MCPD-015` is Ready |",
        "| Public release | `mcp-doctor` `v0.2.0`",
        "| M3 | Every retained expansion is explicitly authorized and bounded; inherited safety and stable CI output remain intact; one expanded immutable release passes every retained journey | Done |",
        "| D-08 | Bounded diagnostic expansion release | M3 | Done |",
        "| MCPD-012 | Stabilize machine reports and CI integration, then publish and independently verify the retained M3 journeys | M3 | Done |",
        "b0805a8f685e46814e358de368e2a270c21704af",
        "31528649356",
        "31528649333",
        "31529740214",
        "31530330361",
        "a57736ea1a7abf73eeff9a8278af11110247bd20",
        "31530466930",
        "`MCPD-012`, D-08, and M3 are Done as\nof 2026-08-11.",
    ] {
        assert!(
            project.contains(contract),
            "PROJECT.md should retain exact M3 completion evidence: {contract}"
        );
    }
}

#[test]
fn project_resolves_open_08_with_locked_assurance_versions_and_exact_proof() {
    let project = repository_file("PROJECT.md");

    for contract in [
        "| DEC-034 | Resolve `OPEN-08` with one activation-locked assurance version set and exact proof routes | Accepted |",
        "OpenSSF OSPS Baseline `v2026.02.19`",
        "official BadgeApp baseline series displaying OSPS `v2026.02.19`",
        "approved [SLSA `v1.2`]",
        "The `MCPD-013` activation recheck on 2026-08-11 passed.",
        "73db726e5bc898903995ad63e471ff6f820086e2",
        "This clears the\npre-activation drift gate only; it is not an achieved assurance result.",
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
        "GitHub omits repository merge settings and bypass actors from credential-free REST readback",
        "authenticated owner check must verify the exact canonical merge projection and empty bypass list",
        "A future `mcp-doctor` MCP security scanner remains product behavior rather than a repository check by default",
        "Enable squash merge only",
        "Keep auto-merge and merge queue disabled",
        "Keep `bypass_actors` empty",
        "temporarily add only the repository-administrator role with `pull_request` bypass mode",
        "Never disable the ruleset, grant `always` bypass, push directly, delete `main`, or force-push.",
        "Required commit signing stays off.",
        "| DEC-036 | Refine the `DEC-035` verification boundary to match GitHub's live observable fields | Accepted |",
        "Credential-free readback verifies `default_branch` plus the configured and effective public rules",
        "exact canonical merge settings and empty hidden bypass list",
        "no ruleset or legacy branch protection on `main`",
        "resolving this policy did not activate them early",
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
fn project_records_mcpd_013_completion_with_scoped_public_and_private_evidence() {
    let project = repository_file("PROJECT.md");
    let exercise = repository_file("docs/assurance/mcpd-013-emergency-exercise.md");

    for contract in [
        "| MCPD-013 | Protect the default branch and define a contributor-compatible merge policy | M4 | Done |",
        "| MCPD-014 | Establish vulnerability disclosure and live repository-security controls | M4 | Done |",
        "### MCPD-013 completion evidence",
        "`MCPD-013` completed on 2026-08-11",
        "2e3377a5101c513c02bb177cbc95acc3707f77bab4c3ab8ed3e8576a3f828794",
        "https://github.com/EnjoyableWork/mcp-doctor/pull/16",
        "https://github.com/EnjoyableWork/mcp-doctor/pull/17",
        "31537654995/job/93933333425",
        "31537655042/job/93935386965",
        "29d83e094b1112b6c86fbcabeb93667e11e02a53",
        "direct update, primary-branch",
        "leased same-tree non-fast-forward force-update attempts were each",
        "https://github.com/EnjoyableWork/mcp-doctor/pull/18",
        "https://github.com/EnjoyableWork/mcp-doctor/pull/19",
        "31539153287/job/93938063807",
        "31539153316/job/93940246247",
        "Mitigated for the 2026-08-11 `MCPD-013` scope",
        "An administrator can still change repository policy",
        "Security controls are\nowned by `MCPD-014`",
    ] {
        assert!(
            project.contains(contract),
            "PROJECT.md should retain MCPD-013 completion evidence: {contract}"
        );
    }

    for contract in [
        "Status: closed at `2026-08-11T21:55:10Z`",
        "MCPD-013-EXERCISE-20260811-01",
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
        "| `0.2.x` | Supported |",
        "| `0.1.x` | Unsupported |",
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
fn security_control_projection_matches_dec_037_and_dec_038() {
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
            "supported_release_lines": ["0.2.x"],
            "unsupported_release_lines": ["0.1.x"],
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
            "complete_m4_assurance_baseline",
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
fn project_records_mcpd_014_completion_without_a_complete_baseline_claim() {
    let project = repository_file("PROJECT.md");

    for contract in [
        "| MCPD-014 | Establish vulnerability disclosure and live repository-security controls | M4 | Done |",
        "| MCPD-015 | Verify the public contribution, community, repository, and licensing contract | M4 | Ready |",
        "### Accepted vulnerability-disclosure and repository-security policy",
        "`DEC-037` fixes the `MCPD-014` contract.",
        "Support only the latest published minor line, currently `0.2.x`.",
        "within 3 business days",
        "within 7 calendar days",
        "every 14 calendar days",
        "within 90 days of acknowledgement",
        "default query suite, standard runner, weekly schedule, and remote threat model",
        "Enable secret scanning and push protection, require the repository-visible alert endpoint",
        "the baseline therefore does not attest backfill completion",
        "emits only UTC date, canonical SHA-256, and `PASS` or `FAIL`",
        "Do not add CodeQL or secret scanning to the `DEC-035` required ruleset in this ticket.",
        "pre-activation gap record, not a clean baseline or achieved assurance result",
        "https://github.com/EnjoyableWork/mcp-doctor/pull/20",
        "7097b683fc6619447b31db0b55db12467626e446",
        "https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31545582099",
        "Provider-routed partner alerts for public\nrepositories are not visible to repository administrators",
        "does not prove all OSPS Level 1 controls",
        "### MCPD-014 completion evidence",
        "`MCPD-014` completed on 2026-08-11",
        "d379f2c86b9571da14cdb9c51cfc83075f098688a4660aecb67eb60fa385e66a",
        "https://github.com/EnjoyableWork/mcp-doctor/pull/21",
        "https://github.com/EnjoyableWork/mcp-doctor/pull/22",
        "31546161736",
        "31546164626",
        "31546164631",
        "7f777b32e88356cea8f0212ec9bfa61a7373907b",
        "31547028561",
        "31547028549",
        "31547028600",
        "date=2026-08-11 canonical_sha256=d379f2c86b9571da14cdb9c51cfc83075f098688a4660aecb67eb60fa385e66a result=PASS",
        "No alert body,\ncount payload, secret value, credential source, or finding detail was retained",
        "complete-M4 exclusions above remain part\nof the result rather than being treated as passes",
        "| DEC-037 | Support the latest release line through private coordinated disclosure and every entitled repository-security control | Accepted |",
        "| DEC-038 | Refine the `MCPD-014` clean baseline to GitHub Free's observable security surfaces | Accepted |",
        "Mitigated for the scoped 2026-08-11 `MCPD-014` surfaces",
    ] {
        assert!(
            project.contains(contract),
            "PROJECT.md should preserve the scoped MCPD-014 contract: {contract}"
        );
    }

    for stale_status in [
        "`MCPD-014` is In progress",
        "| MCPD-014 | Establish vulnerability disclosure and live repository-security controls | M4 | In progress |",
        "| MCPD-015 | Verify the public contribution, community, repository, and licensing contract | M4 | Proposed |",
    ] {
        assert!(
            !project.contains(stale_status),
            "PROJECT.md must not retain stale MCPD-014 status: {stale_status}"
        );
    }
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
fn main_protection_canonical_config_matches_dec_035() {
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
