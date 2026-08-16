use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

const AUDIT: &str = include_str!("../docs/deterministic-ci.md");
const CI_TOOLS: &str = include_str!("../.github/ci-tools.json");
const CI_TOOL_VERIFIER: &str = include_str!("../scripts/verify-ci-tools.sh");
const CI_TOOL_VERIFIER_PS1: &str = include_str!("../scripts/verify-ci-tools.ps1");
const SUPPLY_CHAIN_CONTROLS: &str = include_str!("../.github/supply-chain-controls.json");
const RUST_TOOLCHAIN: &str = include_str!("../rust-toolchain.toml");

#[test]
fn tracked_rust_tests_do_not_use_timing_as_proof_or_coordination() {
    for path in files_under("tests", &["rs"]) {
        if path.ends_with("deterministic_ci_policy.rs") {
            continue;
        }
        let source = read(&path);
        for prohibited in [
            ".elapsed()",
            "Instant::now()",
            "thread::sleep",
            "tokio::time::sleep(",
        ] {
            assert!(
                !source.contains(prohibited),
                "{} uses prohibited timing mechanism {prohibited}",
                relative(&path).display()
            );
        }
        if path.ends_with("http.rs") {
            assert_eq!(source.matches("SystemTime::now()").count(), 1);
            assert_eq!(source.matches("UNIX_EPOCH").count(), 2);
            assert!(source.contains("fn current_utc_year()"));
        } else {
            for prohibited in ["SystemTime::now()", "UNIX_EPOCH"] {
                assert!(
                    !source.contains(prohibited),
                    "{} uses an undeclared test clock {prohibited}",
                    relative(&path).display()
                );
            }
        }
    }

    for path in files_under("src", &["rs"]) {
        let source = read(&path);
        let Some((_, unit_tests)) = source.rsplit_once("#[cfg(test)]\nmod tests") else {
            continue;
        };
        for prohibited in [
            ".elapsed()",
            "SystemTime::now()",
            "UNIX_EPOCH",
            "thread::sleep",
            "tokio::time::sleep(",
        ] {
            assert!(
                !unit_tests.contains(prohibited),
                "{} unit tests use prohibited timing mechanism {prohibited}",
                relative(&path).display()
            );
        }
        if path.ends_with("src/transport/http.rs") {
            assert_eq!(unit_tests.matches("Instant::now()").count(), 2);
        } else {
            assert!(
                !unit_tests.contains("Instant::now()"),
                "{} unit tests read a scheduler-dependent instant",
                relative(&path).display()
            );
        }
    }

    assert!(AUDIT.contains("explicit descendant-ready acknowledgement"));
    assert!(AUDIT.contains("peer close"));
    assert!(AUDIT.contains("outer watchdog"));
    assert!(AUDIT.contains("sole test wall-clock read"));
}

#[test]
fn every_workflow_job_has_an_outer_watchdog() {
    for path in files_under(".github/workflows", &["yml", "yaml"]) {
        let source = read(&path);
        let jobs = source
            .lines()
            .filter(|line| line.trim_start().starts_with("runs-on:"))
            .count();
        let watchdogs = source
            .lines()
            .filter(|line| line.trim_start().starts_with("timeout-minutes:"))
            .count();
        assert!(jobs > 0, "{} contains no jobs", relative(&path).display());
        assert_eq!(
            watchdogs,
            jobs,
            "{} must give every job one outer timeout-minutes watchdog",
            relative(&path).display()
        );
        assert!(
            source.contains("concurrency:"),
            "{} must declare its run-level concurrency behavior",
            relative(&path).display()
        );
    }
}

#[test]
fn operational_sleeps_and_positive_retries_match_the_owned_inventory() {
    let mut sleeps = BTreeMap::<String, usize>::new();
    let mut retries = BTreeMap::<String, usize>::new();

    for root in ["scripts", ".github/workflows"] {
        for path in files_under(root, &["sh", "ps1", "yml", "yaml"]) {
            let source = read(&path);
            for line in source.lines().map(str::trim_start) {
                if line.starts_with("sleep ") {
                    *sleeps.entry(relative_string(&path)).or_default() += 1;
                }
                if line.contains("--retry ") && !line.contains("--retry 0") {
                    *retries.entry(relative_string(&path)).or_default() += 1;
                }
                assert!(
                    !line.contains("--retry-all-errors"),
                    "{} enables unclassified retry-all-errors",
                    relative(&path).display()
                );
            }
        }
    }

    assert_eq!(
        sleeps,
        BTreeMap::from([("scripts/install-syft.sh".to_owned(), 1)])
    );
    assert!(retries.is_empty(), "positive curl retries are prohibited");
    assert!(AUDIT.contains("`DEC-043` acquisition exception"));
    assert!(AUDIT.contains("`MCPD-031` completion"));

    let release = read(&repository_root().join(".github/workflows/release.yml"));
    let channels = read(&repository_root().join(".github/workflows/release-channels.yml"));
    let release_controls =
        read(&repository_root().join("scripts/verify-release-repository-controls.sh"));
    assert_eq!(release.matches("--retry 0").count(), 6);
    assert_eq!(channels.matches("--retry 0").count(), 3);
    assert_eq!(release_controls.matches("--retry 0").count(), 1);
    for source in [&release, &channels, &release_controls] {
        assert!(source.contains("--connect-timeout 10"));
        assert!(source.contains("--max-time "));
    }
    for source in [&release, &channels] {
        assert!(!source.contains("sleep 5"));
        assert!(!source.contains("for _ in {1..60}"));
    }
    assert!(release.contains("timeout 60 gh api"));
    assert!(release.contains("timeout 60 gh release verify"));
    assert!(release.contains(".immutable == true"));
    assert!(release.contains("cmp --silent"));
}

#[test]
fn nonstandard_ci_commands_are_declared_and_incidental_tools_are_rejected() {
    let inventory: Value =
        serde_json::from_str(CI_TOOLS).expect("CI tool inventory should be JSON");
    assert_eq!(inventory["schema_version"], "mcp-doctor.ci-tools/v1");
    assert_eq!(inventory["reviewed_on"], "2026-08-16");
    assert_eq!(inventory["rust_toolchain"]["channel"], "1.97.1");
    assert!(RUST_TOOLCHAIN.contains("channel = \"1.97.1\""));

    let declared = declared_commands(&inventory);
    let automation = automation_source();
    for command in [
        "brew",
        "cargo-deny",
        "docker",
        "gh",
        "jq",
        "node",
        "php",
        "pwsh",
        "ruby",
        "syft",
    ] {
        if contains_token(&automation, command) {
            assert!(
                declared.contains(command),
                "automation uses undeclared non-standard command {command}"
            );
        }
    }

    for command in inventory["prohibited_incidental_commands"]
        .as_array()
        .expect("prohibited commands should be an array")
    {
        let command = command
            .as_str()
            .expect("prohibited command names should be strings");
        assert!(
            !contains_token(&automation, command),
            "automation relies on prohibited incidental command {command}"
        );
    }

    let controls: Value =
        serde_json::from_str(SUPPLY_CHAIN_CONTROLS).expect("supply-chain controls should be JSON");
    let reviewed_tools = controls["standalone_tools"]
        .as_array()
        .expect("standalone tools should be an array")
        .iter()
        .map(|tool| {
            (
                tool["name"].as_str().expect("tool name should be a string"),
                tool["version"]
                    .as_str()
                    .expect("tool version should be a string"),
            )
        })
        .collect::<BTreeSet<_>>();
    let acquired = inventory["repository_acquired"]
        .as_array()
        .expect("repository-acquired tools should be an array")
        .iter()
        .map(|tool| {
            (
                tool["command"]
                    .as_str()
                    .expect("acquired command should be a string"),
                tool["version"]
                    .as_str()
                    .expect("acquired version should be a string"),
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(acquired, reviewed_tools);

    for contract in [
        "command -v jq",
        "command -v \"$ci_command\"",
        "select(.runner == $runner)",
    ] {
        assert!(CI_TOOL_VERIFIER.contains(contract));
    }
    for contract in [
        "ConvertFrom-Json",
        "Where-Object { $_.runner -eq $RunnerLabel }",
        "Get-Command -Name $command",
    ] {
        assert!(CI_TOOL_VERIFIER_PS1.contains(contract));
    }

    let current_hosted_workflows = [
        read(&repository_root().join(".github/workflows/ci.yml")),
        read(&repository_root().join(".github/workflows/compatibility.yml")),
        read(&repository_root().join(".github/workflows/release-preflight.yml")),
    ]
    .join("\n");
    assert_eq!(
        current_hosted_workflows
            .matches("persist-credentials: false\n\n      - name: Verify declared runner tools",)
            .count(),
        8
    );
    assert_eq!(
        current_hosted_workflows
            .matches("./scripts/verify-ci-tools.sh")
            .count(),
        6
    );
    assert_eq!(
        current_hosted_workflows
            .matches("./scripts/verify-ci-tools.ps1")
            .count(),
        2
    );
    assert!(AUDIT.contains("immediately after checkout"));

    let release = read(&repository_root().join(".github/workflows/release.yml"));
    assert_eq!(release.matches("uses: actions/checkout@").count(), 6);
    assert_eq!(
        release
            .matches("- name: Verify declared runner tools")
            .count(),
        6
    );
    assert_eq!(release.matches("./scripts/verify-ci-tools.sh").count(), 6);

    let channels = read(&repository_root().join(".github/workflows/release-channels.yml"));
    assert_eq!(
        channels
            .matches("- name: Check out the exact runner contract")
            .count(),
        5
    );
    assert_eq!(
        channels
            .matches("- name: Verify declared runner tools")
            .count(),
        5
    );
    assert_eq!(
        channels
            .matches("- name: Check out the immutable release tag")
            .count(),
        5
    );
    assert_eq!(channels.matches("./scripts/verify-ci-tools.sh").count(), 4);
    assert_eq!(channels.matches("./scripts/verify-ci-tools.ps1").count(), 1);
    assert!(channels.contains("test \"$GITHUB_REF\" = refs/heads/main"));
    assert!(channels.contains("ref: ${{ github.sha }}"));
}

fn declared_commands(inventory: &Value) -> BTreeSet<&str> {
    let mut commands = BTreeSet::new();
    for contract in inventory["runner_contracts"]
        .as_array()
        .expect("runner contracts should be an array")
    {
        for command in contract["commands"]
            .as_array()
            .expect("runner commands should be an array")
        {
            commands.insert(command.as_str().expect("runner command should be a string"));
        }
    }
    for section in [
        "action_provided",
        "repository_acquired",
        "generated_environment",
    ] {
        for entry in inventory[section]
            .as_array()
            .expect("tool-provider sections should be arrays")
        {
            commands.insert(
                entry["command"]
                    .as_str()
                    .expect("provided command should be a string"),
            );
        }
    }
    for entry in inventory["container_provided"]
        .as_array()
        .expect("container tools should be an array")
    {
        for command in entry["commands"]
            .as_array()
            .expect("container commands should be an array")
        {
            commands.insert(
                command
                    .as_str()
                    .expect("container command should be a string"),
            );
        }
    }
    commands
}

fn automation_source() -> String {
    ["scripts", ".github/workflows"]
        .into_iter()
        .flat_map(|root| files_under(root, &["sh", "ps1", "yml", "yaml"]))
        .map(|path| read(&path))
        .collect::<Vec<_>>()
        .join("\n")
}

fn contains_token(source: &str, token: &str) -> bool {
    source
        .split(|character: char| {
            !(character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        })
        .any(|candidate| candidate == token)
}

fn files_under(relative_root: &str, extensions: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files(
        &repository_root().join(relative_root),
        extensions,
        &mut files,
    );
    files.sort();
    files
}

fn collect_files(root: &Path, extensions: &[&str], files: &mut Vec<PathBuf>) {
    for entry in
        fs::read_dir(root).unwrap_or_else(|_| panic!("{} should be readable", root.display()))
    {
        let path = entry.expect("directory entries should be readable").path();
        if path.is_dir() {
            collect_files(&path, extensions, files);
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extensions.contains(&extension))
        {
            files.push(path);
        }
    }
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("{} should be UTF-8", path.display()))
}

fn relative(path: &Path) -> &Path {
    path.strip_prefix(repository_root())
        .expect("policy paths should remain inside the repository")
}

fn relative_string(path: &Path) -> String {
    relative(path).to_string_lossy().replace('\\', "/")
}

fn repository_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}
