mod support;

use std::fs;
use std::path::{Component, Path, PathBuf};
#[cfg(unix)]
use std::process::Command;

use serde::Deserialize;

#[cfg(unix)]
const RELEASE_VERSION: &str = "0.4.2";
const CURRENT_SKILL_SHA256: &str =
    "3f7b0cd490e272ce86c898b9e1a2a56c5086411f8d8b82bb251eca0023549b79";
const CURRENT_OPENAI_SHA256: &str =
    "a56095c3f3eb2ed6bdbceb9b4d6c40289b5bb45733c4c950c32a0c02bbd680d6";
const CURRENT_ICON_SHA256: &str =
    "8140b500f4bc70688a473bc9ec63cdb0b1a3e229596215340588053a3ee1d71b";
const PUBLISHED_V042_SKILL_SHA256: &str =
    "3f7b0cd490e272ce86c898b9e1a2a56c5086411f8d8b82bb251eca0023549b79";
const V032_SKILL_SHA256: &str = "f7ee6903c839a268648bf8114e75817396a78f7b08f38a424541fe4b0c483a51";

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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillEvalCorpus {
    skill_name: String,
    evals: Vec<SkillEvalCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillEvalCase {
    id: u16,
    prompt: String,
    expected_output: String,
    files: Vec<String>,
    assertions: Vec<String>,
}

#[test]
fn canonical_skill_bundle_is_portable_passive_and_presentation_only() {
    let skill_root = repository_root().join(".agents/skills/mcp-doctor");
    let skill = repository_file(".agents/skills/mcp-doctor/SKILL.md");
    let openai = repository_file(".agents/skills/mcp-doctor/agents/openai.yaml");
    let icon = repository_file(".agents/skills/mcp-doctor/assets/icon.svg");
    let mut entries = fs::read_dir(&skill_root)
        .expect("skill directory should exist")
        .map(|entry| {
            entry
                .expect("skill entry should be readable")
                .file_name()
                .into_string()
                .expect("skill entry should be portable UTF-8")
        })
        .collect::<Vec<_>>();
    entries.sort();
    assert_eq!(entries, ["SKILL.md", "agents", "assets"]);
    assert_eq!(
        fs::read_dir(skill_root.join("agents"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>(),
        ["openai.yaml"]
    );
    assert_eq!(
        fs::read_dir(skill_root.join("assets"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>(),
        ["icon.svg"]
    );

    let lines = skill.lines().collect::<Vec<_>>();
    assert_eq!(lines[0], "---");
    assert_eq!(lines[1], "name: mcp-doctor");
    assert!(lines[2].starts_with("description: Diagnose MCP servers before users do"));
    assert_eq!(lines[3], "---");
    assert!(lines.len() < 500);

    for contract in [
        "coding agent -> terminal -> mcp-doctor CLI -> exact MCP server target",
        "mcp-doctor --version",
        "mcp-doctor capabilities --format json",
        "https://github.com/EnjoyableWork/mcp-doctor/tree/main/.agents/skills/mcp-doctor",
        "https://smithery.ai/skills/enjoyable/mcp-doctor",
        "Do not run a skill installer or registry command",
        "cargo install mcp-doctor --version '=0.4.2' --locked",
        "brew install --build-from-source EnjoyableWork/tap/mcp-doctor",
        "https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.4.2",
        "Do not choose a route, run either command",
        "Continue only with `mcp-doctor 0.4.2`",
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
        "missing-CLI commands above are instructions for the user, not execution",
        "install or upgrade software",
    ] {
        assert!(skill.contains(contract), "skill should preserve {contract}");
    }
    for forbidden in [
        "allowed-tools:",
        "permissions:",
        "curl ",
        "wget ",
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
    assert_eq!(
        skill
            .matches("cargo install mcp-doctor --version '=0.4.2' --locked")
            .count(),
        1
    );
    assert_eq!(
        skill
            .matches("brew install --build-from-source EnjoyableWork/tap/mcp-doctor")
            .count(),
        1
    );

    for contract in [
        "display_name: \"MCP Doctor\"",
        "short_description: \"Diagnose MCP servers before users do—from local to production\"",
        "icon_small: \"./assets/icon.svg\"",
        "icon_large: \"./assets/icon.svg\"",
        "default_prompt: \"Use $mcp-doctor to passively diagnose this exact MCP server target: [command or endpoint].\"",
        "allow_implicit_invocation: true",
    ] {
        assert!(
            openai.contains(contract),
            "metadata should preserve {contract}"
        );
    }
    let short_description = openai
        .lines()
        .find_map(|line| {
            line.strip_prefix("  short_description: \"")
                .and_then(|value| value.strip_suffix('"'))
        })
        .expect("OpenAI metadata should have a quoted short description");
    assert!((25..=64).contains(&short_description.len()));
    assert!(!openai.contains("dependencies:"));
    assert!(!openai.contains("brand_color:"));
    assert!(!openai.contains('\t'));

    let mut icon_reader = quick_xml::Reader::from_str(&icon);
    loop {
        match icon_reader.read_event() {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {}
            Err(error) => panic!("icon should be well-formed SVG: {error}"),
        }
    }
    let lowercase_icon = icon.to_ascii_lowercase();
    assert!(icon.len() < 32_768);
    assert!(icon.contains("viewBox=\"400 200 800 800\""));
    assert!(icon.contains("fill=\"none\""));
    assert!(icon.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    assert_eq!(lowercase_icon.matches("http:").count(), 1);
    assert!(icon.contains("<linearGradient"));
    assert_eq!(icon.matches("<path ").count(), 4);
    for forbidden in [
        "<script",
        "<image",
        "<rect",
        "<circle",
        "<ellipse",
        "<polygon",
        "<polyline",
        "<foreignobject",
        "<!doctype",
        "href=",
        "https:",
        "data:",
        "/users/",
        "adobe",
    ] {
        assert!(
            !lowercase_icon.contains(forbidden),
            "icon contains {forbidden}"
        );
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
    let evidence = repository_file("docs/assurance/v0.3.2-agent-skill.md");
    let readme = repository_file("README.md");

    for contract in [
        "coding agent -> terminal -> mcp-doctor CLI -> exact MCP server target",
        "mcp-doctor-agent-skill-v0.4.2.tar.gz",
        "SHA256SUMS",
        "https://smithery.ai/skills/enjoyable/mcp-doctor",
        "https://github.com/EnjoyableWork/mcp-doctor/tree/main/.agents/skills/mcp-doctor",
        "Third-party installation convenience backed by the canonical GitHub directory",
        "Do not install both\nthe Smithery and manual copies",
        CURRENT_SKILL_SHA256,
        CURRENT_OPENAI_SHA256,
        CURRENT_ICON_SHA256,
        PUBLISHED_V042_SKILL_SHA256,
        "never changes an agent host",
        "never replace it silently",
        "Explicit invocation is the supported route",
        "implicit selection is\nbest-effort",
        "schedules",
        "unattended",
        "Remove only the exact unmodified file",
        "refusing documented removal",
        "not deterministic model-correctness",
        "evaluations/agent-skill-v1.md",
        "behavioral instructions remain self-contained",
        "mcp-doctor-chatgpt-skill-v0.4.2.zip",
        "scripts/package-chatgpt-skill.sh",
        "standalone skill bundle, not\na plugin",
        "https://agentskills.io/specification",
        "https://learn.chatgpt.com/docs/build-skills",
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
        "assurance/v0.3.2-agent-skill.md",
        "The last verified route is Codex CLI\n`0.147.0`",
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
    assert!(!readme.contains("## Use with coding agents"));
    assert!(!readme.contains("### Install the optional Agent Skill"));
    assert!(readme.contains("<a href=\"#agent-skill\">Agent Skill</a>"));
    let readme_skill = readme
        .split_once("## Agent Skill")
        .and_then(|(_, remainder)| remainder.split_once("## Quick start"))
        .map(|(section, _)| section)
        .expect("README should keep a bounded Agent Skill section");
    assert!(
        readme_skill.lines().count() <= 18,
        "README Agent Skill section should route setup without becoming a manual"
    );
    for contract in [
        "Install the CLI above first",
        "[ChatGPT upload bundle](docs/agents.md#build-the-chatgpt-upload-zip)",
        "`$skill-installer`",
        "~/.claude/skills/mcp-doctor",
        ".claude/skills/mcp-doctor",
        "Other Agent Skills hosts",
    ] {
        assert!(
            readme_skill.contains(contract),
            "README Agent Skill section should preserve {contract}"
        );
    }
    assert!(readme.contains(
        "| Use `mcp-doctor` with a coding agent | [Coding-agent guide](docs/agents.md) |"
    ));
    assert!(readme.contains(
        "| Evaluate the Agent Skill on a registry or host | [Agent Skill evaluation contract](docs/evaluations/agent-skill-v1.md) |"
    ));
    assert!(readme.contains(
        "[Open `mcp-doctor` on Smithery](https://smithery.ai/skills/enjoyable/mcp-doctor)"
    ));
    assert!(readme.contains(
        "[canonical skill directory](https://github.com/EnjoyableWork/mcp-doctor/tree/main/.agents/skills/mcp-doctor)"
    ));
    assert!(readme.contains("GitHub remains the canonical source and release authority"));
    assert!(!guide.contains("universal agent support"));

    for contract in [
        V032_SKILL_SHA256,
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
fn chatgpt_bundle_has_exact_metadata_assets_and_deterministic_packaging() {
    let packager = repository_file("scripts/package-chatgpt-skill.sh");
    let verifier = repository_file("scripts/verify-chatgpt-skill.sh");

    for contract in [
        "mcp-doctor-chatgpt-skill-v${chatgpt_package_version}.zip",
        "SOURCE_DATE_EPOCH",
        "zip -X -q -9",
        "SKILL.md",
        "agents/openai.yaml",
        "assets/icon.svg",
        "verify-chatgpt-skill.sh",
    ] {
        assert!(
            packager.contains(contract),
            "ChatGPT packager should preserve {contract}"
        );
    }
    for contract in [
        "mcp-doctor-chatgpt-skill-v${chatgpt_verify_version}.zip",
        "unexpected file, directory, or wrapper folder",
        "compressed or expanded byte limit",
        "cmp --silent",
        "symbolic link",
        "SKILL.md",
        "agents/openai.yaml",
        "assets/icon.svg",
    ] {
        assert!(
            verifier.contains(contract),
            "ChatGPT verifier should preserve {contract}"
        );
    }

    #[cfg(unix)]
    {
        if Command::new("zip").arg("-v").output().is_err()
            || Command::new("unzip").arg("-v").output().is_err()
        {
            return;
        }
        let first = tempfile::tempdir().expect("first ZIP root should be disposable");
        let second = tempfile::tempdir().expect("second ZIP root should be disposable");
        for output in [first.path(), second.path()] {
            let status = Command::new("bash")
                .arg(repository_root().join("scripts/package-chatgpt-skill.sh"))
                .arg(RELEASE_VERSION)
                .arg(output)
                .env("SOURCE_DATE_EPOCH", "1787529600")
                .status()
                .expect("ChatGPT skill packager should execute");
            assert!(status.success());
        }
        let archive = format!("mcp-doctor-chatgpt-skill-v{RELEASE_VERSION}.zip");
        assert_eq!(
            fs::read(first.path().join(&archive)).unwrap(),
            fs::read(second.path().join(&archive)).unwrap()
        );
        let entries = Command::new("unzip")
            .args(["-Z1", first.path().join(&archive).to_str().unwrap()])
            .output()
            .expect("ChatGPT skill ZIP should be inspectable");
        assert!(entries.status.success());
        assert_eq!(
            String::from_utf8(entries.stdout).unwrap(),
            concat!(
                "SKILL.md\n",
                "agents/\n",
                "agents/openai.yaml\n",
                "assets/\n",
                "assets/icon.svg\n"
            )
        );
    }
}

#[test]
fn portable_eval_corpus_is_complete_safe_and_outside_the_skill_artifact() {
    let source = repository_file("tests/fixtures/agent-skill/evals.json");
    let corpus: SkillEvalCorpus =
        serde_json::from_str(&source).expect("Agent Skill evals should be strict JSON");
    assert_eq!(corpus.skill_name, "mcp-doctor");
    assert_eq!(corpus.evals.len(), 14);
    assert_eq!(
        corpus.evals.iter().map(|case| case.id).collect::<Vec<_>>(),
        (1..=14).collect::<Vec<_>>()
    );

    for case in &corpus.evals {
        assert!(
            !case.prompt.trim().is_empty(),
            "eval {} needs a prompt",
            case.id
        );
        assert!(
            !case.expected_output.trim().is_empty(),
            "eval {} needs an expected output",
            case.id
        );
        assert!(
            case.assertions.len() >= 3,
            "eval {} needs objective assertions",
            case.id
        );
        assert!(
            case.assertions
                .iter()
                .all(|assertion| !assertion.trim().is_empty()),
            "eval {} has an empty assertion",
            case.id
        );
        for file in &case.files {
            let path = Path::new(file);
            assert!(
                !path.is_absolute(),
                "eval {} uses an absolute path",
                case.id
            );
            assert!(
                path.components()
                    .all(|component| matches!(component, Component::Normal(_))),
                "eval {} uses a nonportable input path",
                case.id
            );
            assert!(
                repository_root().join(path).is_file(),
                "eval {} input {file} should exist",
                case.id
            );
        }
    }

    for coverage in [
        "selected without an explicit skill mention",
        "skill was not selected",
        "byte-identical passive inspection commands",
        "version mismatch",
        "malformed capability output",
        "missing passive capability",
        "non-JSON reporter output",
        "report-untrusted.json",
        "synthetic-secret.txt",
    ] {
        assert!(
            source.contains(coverage),
            "eval corpus should cover {coverage}"
        );
    }
    for forbidden in [
        "mcp-doctor check --",
        "mcp-doctor break --",
        "mcp-doctor reject --",
        "--allow-side-effects",
        "--allow-credentials-to",
        "--allow-private-network",
        "curl ",
        "wget ",
        "/Users/",
    ] {
        assert!(
            !source.contains(forbidden),
            "eval corpus contains {forbidden}"
        );
    }

    let untrusted = repository_file("tests/fixtures/agent-skill/report-untrusted.json");
    let report = support::parse_and_validate_report(untrusted.as_bytes());
    assert_eq!(report["schema_version"], "mcp-doctor.report/v1");
    assert_eq!(untrusted.matches("eval-report-canary-7c2e").count(), 2);

    let contract = repository_file("docs/evaluations/agent-skill-v1.md");
    for required in [
        "mcp-doctor/\n|-- SKILL.md",
        "|   `-- openai.yaml",
        "`-- assets/\n    `-- icon.svg",
        "evidence deliberately outside the installable skill directory",
        "does not prove host selection or execution",
        "Run each case once",
        "`pass`, `fail`, or `not_observed`",
        "Do not award partial credit or derive a cross-vendor score or rank",
        "Publish a vendor claim only for the exact identity and cases that passed",
    ] {
        assert!(
            contract.contains(required),
            "eval contract should preserve {required}"
        );
    }
    assert!(!contract.contains("ChatGPT passed"));
    assert!(!contract.contains("Smithery passed"));
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
        "mcp-doctor 0.4.2\n"
    );

    let capabilities = run_recorder(&recorder, &log, &["capabilities", "--format", "json"]);
    assert!(capabilities.status.success());
    let capability_json = support::parse_and_validate_capabilities(&capabilities.stdout);
    assert_eq!(
        capability_json["schema_version"],
        "mcp-doctor.capabilities/v1"
    );
    assert_eq!(capability_json["commands"][0]["activity"], "passive");
    assert!(
        capability_json["protocol_support"]
            .as_array()
            .unwrap()
            .iter()
            .any(|support| support["command"] == "inspect"
                && support["transport"] == "streamable_http")
    );

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

    exercise_posix_recorder_stop_modes(&recorder, temporary.path());
}

#[cfg(unix)]
fn exercise_posix_recorder_stop_modes(recorder: &Path, root: &Path) {
    let version_log = root.join("version-mismatch.log");
    let version =
        run_recorder_with_mode(recorder, &version_log, &["--version"], "version-mismatch");
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        "mcp-doctor 9.9.9-eval\n"
    );
    assert_eq!(
        fs::read_to_string(version_log).unwrap(),
        "mcp-doctor --version\n"
    );

    let malformed_log = root.join("malformed-capabilities.log");
    let version = run_recorder_with_mode(
        recorder,
        &malformed_log,
        &["--version"],
        "malformed-capabilities",
    );
    assert!(version.status.success());
    let malformed = run_recorder_with_mode(
        recorder,
        &malformed_log,
        &["capabilities", "--format", "json"],
        "malformed-capabilities",
    );
    assert!(malformed.status.success());
    assert!(serde_json::from_slice::<serde_json::Value>(&malformed.stdout).is_err());
    assert_eq!(
        fs::read_to_string(malformed_log).unwrap(),
        concat!(
            "mcp-doctor --version\n",
            "mcp-doctor capabilities --format json\n"
        )
    );

    let passive_log = root.join("passive-unavailable.log");
    let version = run_recorder_with_mode(
        recorder,
        &passive_log,
        &["--version"],
        "passive-unavailable",
    );
    assert!(version.status.success());
    let capabilities = run_recorder_with_mode(
        recorder,
        &passive_log,
        &["capabilities", "--format", "json"],
        "passive-unavailable",
    );
    assert!(capabilities.status.success());
    let capability_json = support::parse_and_validate_capabilities(&capabilities.stdout);
    assert_eq!(capability_json["commands"][0]["activity"], "active");
    assert_eq!(
        fs::read_to_string(passive_log).unwrap(),
        concat!(
            "mcp-doctor --version\n",
            "mcp-doctor capabilities --format json\n"
        )
    );

    let report_log = root.join("non-json-report.log");
    let version = run_recorder_with_mode(recorder, &report_log, &["--version"], "non-json-report");
    assert!(version.status.success());
    let capabilities = run_recorder_with_mode(
        recorder,
        &report_log,
        &["capabilities", "--format", "json"],
        "non-json-report",
    );
    assert!(capabilities.status.success());
    support::parse_and_validate_capabilities(&capabilities.stdout);
    let report = run_recorder_with_mode(
        recorder,
        &report_log,
        &[
            "inspect",
            "--format",
            "json",
            "--",
            "./synthetic-mcp-server",
            "--stdio",
        ],
        "non-json-report",
    );
    assert_eq!(report.status.code(), Some(1));
    assert!(serde_json::from_slice::<serde_json::Value>(&report.stdout).is_err());
    assert_eq!(
        fs::read_to_string(report_log).unwrap(),
        concat!(
            "mcp-doctor --version\n",
            "mcp-doctor capabilities --format json\n",
            "mcp-doctor inspect --format json -- ./synthetic-mcp-server --stdio\n"
        )
    );

    let invalid_log = root.join("invalid-mode.log");
    let invalid = run_recorder_with_mode(recorder, &invalid_log, &["--version"], "unknown");
    assert_eq!(invalid.status.code(), Some(70));
    assert!(!invalid_log.exists());
}

#[cfg(unix)]
fn run_recorder(recorder: &Path, log: &Path, arguments: &[&str]) -> std::process::Output {
    let mut command = Command::new("bash");
    command
        .arg(recorder)
        .args(arguments)
        .current_dir(log.parent().expect("recorder log should have a parent"))
        .env("MCP_DOCTOR_AGENT_RECORDER_LOG", log);
    command
        .output()
        .expect("Agent Skill recorder should execute")
}

#[cfg(unix)]
fn run_recorder_with_mode(
    recorder: &Path,
    log: &Path,
    arguments: &[&str],
    mode: &str,
) -> std::process::Output {
    let mut command = Command::new("bash");
    command
        .arg(recorder)
        .args(arguments)
        .current_dir(log.parent().expect("recorder log should have a parent"))
        .env("MCP_DOCTOR_AGENT_RECORDER_LOG", log)
        .env("MCP_DOCTOR_AGENT_RECORDER_MODE", mode);
    command
        .output()
        .expect("Agent Skill recorder mode should execute")
}
