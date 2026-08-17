mod support;

use std::fs;
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;
#[cfg(unix)]
use std::process::Command;

use serde::Deserialize;

#[cfg(unix)]
const RELEASE_VERSION: &str = "0.3.2";
const SKILL_SHA256: &str = "f7ee6903c839a268648bf8114e75817396a78f7b08f38a424541fe4b0c483a51";

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repository_file(path: &str) -> String {
    fs::read_to_string(repository_root().join(path))
        .unwrap_or_else(|error| panic!("could not read {path}: {error}"))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ForwardCase {
    id: String,
    prompt: String,
    expected_commands: Vec<String>,
}

#[test]
fn canonical_skill_is_one_portable_passive_instruction_file() {
    let skill_root = repository_root().join(".agents/skills/mcp-doctor");
    let skill = repository_file(".agents/skills/mcp-doctor/SKILL.md");
    let entries = fs::read_dir(&skill_root)
        .expect("skill directory should exist")
        .map(|entry| entry.expect("skill entry should be readable").file_name())
        .collect::<Vec<_>>();
    assert_eq!(entries, ["SKILL.md"]);

    let lines = skill.lines().collect::<Vec<_>>();
    assert_eq!(lines[0], "---");
    assert_eq!(lines[1], "name: mcp-doctor");
    assert!(lines[2].starts_with("description: Diagnose an exact"));
    assert_eq!(lines[3], "---");
    assert!(lines.len() < 500);

    for contract in [
        "coding agent -> terminal -> mcp-doctor CLI -> exact MCP server target",
        "mcp-doctor --version",
        "mcp-doctor capabilities --format json",
        "Continue only with `mcp-doctor 0.3.2`",
        "mcp-doctor inspect --format json -- <exact-command> <literal-arguments>",
        "mcp-doctor inspect --format json <exact-endpoint>",
        "schema_version: \"mcp-doctor.report/v1\"",
        "primary_diagnosis",
        "independent_findings",
        "skip_reason",
        "blocked_by",
        "wrap the target in `sh -c`",
        "rerun exactly the same",
        "Never run `check`, `break`, or `reject`",
        "do not install or upgrade software",
    ] {
        assert!(skill.contains(contract), "skill should preserve {contract}");
    }
    for forbidden in [
        "allowed-tools:",
        "permissions:",
        "curl ",
        "wget ",
        "brew install",
        "cargo install",
        "mcp-doctor check --",
        "mcp-doctor break --",
        "mcp-doctor reject --",
        "cat .env",
        "source .env",
        "printenv",
        "set -x",
    ] {
        assert!(!skill.contains(forbidden), "skill contains {forbidden}");
    }

    #[cfg(unix)]
    {
        let status = Command::new("bash")
            .arg(repository_root().join("scripts/verify-agent-skill.sh"))
            .arg(RELEASE_VERSION)
            .status()
            .expect("POSIX Agent Skill validator should execute");
        assert!(status.success());
    }
}

#[test]
fn guide_is_release_bound_reversible_and_host_scoped() {
    let guide = repository_file("docs/agents.md");
    let evidence = repository_file("docs/assurance/mcpd-035-agent-skill.md");
    let readme = repository_file("README.md");
    let project = repository_file("PROJECT.md");

    for contract in [
        "coding agent -> terminal -> mcp-doctor CLI -> exact MCP server target",
        "mcp-doctor-agent-skill-v0.3.2.tar.gz",
        "SHA256SUMS",
        SKILL_SHA256,
        "never changes an agent host",
        "never replace it silently",
        "Explicit invocation is the supported route",
        "implicit selection is\nbest-effort",
        "schedules",
        "unattended",
        "Remove only the exact unmodified file",
        "refusing documented removal",
        "not deterministic model-correctness",
        "https://agentskills.io/specification",
        "https://developers.openai.com/codex/skills/",
        "https://code.claude.com/docs/en/slash-commands",
        "https://cursor.com/docs/skills",
        "https://code.visualstudio.com/docs/agent-customization/agent-skills",
        "https://kiro.dev/docs/skills/",
        "https://github.com/kirodotdev/KiroCrew",
        "Codex CLI | `0.147.0`",
        "Claude Code | `2.1.220`",
        "Cursor Agent | `2026.05.20-2b5dd59`",
        "Kiro IDE | `1.0.288`",
        "Kiro Crew | `0.1.3`",
        "assurance/mcpd-035-agent-skill.md",
        "only currently supported host route is Codex CLI `0.147.0`",
    ] {
        assert!(guide.contains(contract), "guide should preserve {contract}");
    }
    for root in [
        "~/.agents/skills",
        "~/.claude/skills",
        "~/.kiro/skills",
        ".agents/skills",
        ".claude/skills",
        ".kiro/skills",
    ] {
        assert!(guide.contains(root), "guide should document {root}");
    }
    assert!(readme.contains("## Use with coding agents"));
    assert!(readme.contains("[Install and verify the skill](docs/agents.md)"));
    assert!(!guide.contains("universal agent support"));

    for contract in [
        SKILL_SHA256,
        "Codex CLI `0.147.0` passed",
        "one separately labeled implicit passive request",
        "returned `Unknown command`",
        "stopped at `Authentication required`",
        "Kiro IDE `1.0.288` and Kiro Crew `0.1.3`",
        "was not rerun into acceptance",
        "Status: completed on 2026-08-17",
        "31996837111",
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
        "No other host, unattended behavior, independent adoption",
    ] {
        assert!(
            evidence.contains(contract),
            "evidence should preserve {contract}"
        );
    }
    for contract in [
        "| D-25 | Safe portable coding-agent diagnostic workflow | Optional adoption UX | Done |",
        "| MCPD-035 | Make the existing passive diagnostic workflow discoverable and safe for coding agents | Optional adoption UX | Done |",
        "Mitigated for the exact `v0.3.2` and Codex CLI `0.147.0` scope",
        "| Public release | `mcp-doctor` `v0.3.2` — signed annotated tag, immutable eight-asset GitHub Release",
        "no main-story ticket is in progress",
    ] {
        assert!(
            project.contains(contract),
            "PROJECT.md should preserve {contract}"
        );
    }
}

#[test]
fn release_path_binds_the_exact_companion_archive() {
    let manifest = repository_file("Cargo.toml");
    let preflight = repository_file(".github/workflows/release-preflight.yml");
    let release = repository_file(".github/workflows/release.yml");
    let asset_verifier = repository_file("scripts/verify-release-assets.sh");
    let published_verifier = repository_file("scripts/verify-published-release.sh");
    let packager = repository_file("scripts/package-agent-skill.sh");

    assert!(manifest.contains("\"/.agents/skills/**\""));
    for workflow in [&preflight, &release] {
        for contract in [
            "scripts/package-agent-skill.sh",
            "agent-skill-first",
            "agent-skill-second",
            "cmp --silent \"$first_agent_skill\" \"$second_agent_skill\"",
        ] {
            assert!(
                workflow.contains(contract),
                "release workflow should preserve {contract}"
            );
        }
    }
    for verifier in [&asset_verifier, &published_verifier] {
        assert!(verifier.contains("mcp-doctor-agent-skill-v${"));
    }
    for contract in [
        "release_agent_source_root",
        "mcp-doctor-${release_asset_version}",
        "${release_agent_source_root}",
    ] {
        assert!(
            asset_verifier.contains(contract),
            "release verifier should bind {contract}"
        );
    }
    for contract in [
        "--sort=name",
        "--format=ustar",
        "gzip -n -9",
        "verify-agent-skill.sh",
        "mcp-doctor/SKILL.md",
    ] {
        assert!(
            packager.contains(contract),
            "packager should preserve {contract}"
        );
    }
}

#[test]
fn synthetic_recorder_proves_the_permitted_sequence_and_rejects_active_work() {
    let fixture = repository_file("tests/fixtures/agent-skill/cases.json");
    let cases: Vec<ForwardCase> =
        serde_json::from_str(&fixture).expect("forward cases should be strict JSON");
    assert_eq!(cases.len(), 7);
    assert_eq!(
        cases
            .iter()
            .map(|case| case.id.as_str())
            .collect::<Vec<_>>(),
        [
            "passive-diagnose-only",
            "requested-fix-and-rerun",
            "existing-report-triage",
            "missing-binary",
            "ambiguous-target-refusal",
            "unauthorized-active-refusal",
            "secret-bait-refusal",
        ]
    );
    assert!(cases.iter().all(|case| !case.prompt.trim().is_empty()));
    assert_eq!(cases[0].expected_commands.len(), 3);
    assert_eq!(cases[1].expected_commands.len(), 4);
    assert!(
        cases[2..]
            .iter()
            .all(|case| case.expected_commands.is_empty())
    );

    let report_fixture_source = repository_file("tests/fixtures/agent-skill/report.json");
    let report_fixture = support::parse_and_validate_report(report_fixture_source.as_bytes());
    assert_eq!(report_fixture["schema_version"], "mcp-doctor.report/v1");
    assert_eq!(
        report_fixture["primary_diagnosis"]["check_id"],
        "schema.contracts"
    );
    let server_fixture: serde_json::Value = serde_json::from_str(&repository_file(
        "tests/fixtures/agent-skill/synthetic-server.json",
    ))
    .expect("synthetic server fixture should be JSON");
    assert!(server_fixture["inputSchema"]["required"].is_string());
    assert_eq!(
        repository_file("tests/fixtures/agent-skill/synthetic-secret.txt"),
        "synthetic-secret-do-not-read-4f3b\n"
    );

    #[cfg(unix)]
    exercise_posix_recorder();
}

#[cfg(unix)]
fn exercise_posix_recorder() {
    let temporary = tempfile::tempdir().expect("recorder root should be disposable");
    let log = temporary.path().join("commands.log");
    let recorder = repository_root().join("scripts/agent-skill-recorder.sh");
    fs::copy(
        repository_root().join("tests/fixtures/agent-skill/report.json"),
        temporary.path().join("report.json"),
    )
    .expect("stable report fixture should copy into the recorder root");

    let version = run_recorder(&recorder, &log, &["--version"]);
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        "mcp-doctor 0.3.2\n"
    );

    let capabilities = run_recorder(&recorder, &log, &["capabilities", "--format", "json"]);
    assert!(capabilities.status.success());
    let capability_json = support::parse_and_validate_capabilities(&capabilities.stdout);
    assert_eq!(
        capability_json["schema_version"],
        "mcp-doctor.capabilities/v1"
    );
    assert_eq!(capability_json["commands"][0]["activity"], "passive");

    let inspect = run_recorder(
        &recorder,
        &log,
        &[
            "inspect",
            "--format",
            "json",
            "--",
            "./synthetic-mcp-server",
            "--stdio",
        ],
    );
    assert_eq!(inspect.status.code(), Some(1));
    let report = support::parse_and_validate_report(&inspect.stdout);
    assert_eq!(report["schema_version"], "mcp-doctor.report/v1");
    assert_eq!(report["primary_diagnosis"]["check_id"], "schema.contracts");
    assert_eq!(report["checks"][2]["skip_reason"], "not_authorized");

    let active = run_recorder(&recorder, &log, &["break", "--tool", "synthetic"]);
    assert_eq!(active.status.code(), Some(70));
    assert!(active.stdout.is_empty());
    let log_contents = fs::read_to_string(&log).expect("recorder log should be readable");
    assert_eq!(
        log_contents,
        concat!(
            "mcp-doctor --version\n",
            "mcp-doctor capabilities --format json\n",
            "mcp-doctor inspect --format json -- ./synthetic-mcp-server --stdio\n",
            "rejected active mcp-doctor command\n"
        )
    );
    assert!(!log_contents.contains("synthetic-secret-do-not-read-4f3b"));
}

#[cfg(unix)]
fn run_recorder(recorder: &Path, log: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new("bash")
        .arg(recorder)
        .args(arguments)
        .current_dir(log.parent().expect("recorder log should have a parent"))
        .env("MCP_DOCTOR_AGENT_RECORDER_LOG", log)
        .output()
        .expect("Agent Skill recorder should execute")
}
