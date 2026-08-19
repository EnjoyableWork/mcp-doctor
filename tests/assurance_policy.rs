use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

const CONTROLS: [&str; 24] = [
    "OSPS-AC-01.01",
    "OSPS-AC-02.01",
    "OSPS-AC-03.01",
    "OSPS-AC-03.02",
    "OSPS-BR-01.01",
    "OSPS-BR-01.03",
    "OSPS-BR-03.01",
    "OSPS-BR-03.02",
    "OSPS-BR-07.01",
    "OSPS-DO-01.01",
    "OSPS-DO-02.01",
    "OSPS-GV-02.01",
    "OSPS-GV-03.01",
    "OSPS-LE-02.01",
    "OSPS-LE-02.02",
    "OSPS-LE-03.01",
    "OSPS-LE-03.02",
    "OSPS-QA-01.01",
    "OSPS-QA-01.02",
    "OSPS-QA-02.01",
    "OSPS-QA-04.01",
    "OSPS-QA-05.01",
    "OSPS-QA-05.02",
    "OSPS-VM-02.01",
];

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repository_file(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(repository_root().join(path))
        .unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()))
}

fn repository_json(path: impl AsRef<Path>) -> Value {
    let path = path.as_ref();
    serde_json::from_str(&repository_file(path))
        .unwrap_or_else(|error| panic!("could not parse {}: {error}", path.display()))
}

fn proposal_field(control: &str, suffix: &str) -> String {
    format!(
        "{}_{}",
        control.to_ascii_lowercase().replace(['-', '.'], "_"),
        suffix
    )
}

#[test]
fn canonical_assurance_scope_is_exact_and_publication_is_verified() {
    let canonical = repository_json(".github/assurance-controls.json");
    let badge = &canonical["osps"]["badgeapp"];

    assert_eq!(
        canonical["schema_version"],
        "mcp-doctor.assurance-controls/v1"
    );
    assert_eq!(canonical["reviewed_on"], "2026-08-15");
    assert_eq!(canonical["repository"], "EnjoyableWork/mcp-doctor");
    assert_eq!(canonical["organization"], "EnjoyableWork");
    assert_eq!(canonical["osps"]["version"], "v2026.02.19");
    assert_eq!(canonical["osps"]["level"], 1);
    assert_eq!(canonical["osps"]["controls_total"], 24);
    assert_eq!(canonical["osps"]["met"], 24);
    assert_eq!(canonical["osps"]["not_applicable"], 0);
    let expected_controls = CONTROLS
        .iter()
        .copied()
        .map(Value::from)
        .collect::<Vec<_>>();
    assert_eq!(
        canonical["osps"]["controls"],
        Value::Array(expected_controls)
    );
    assert_eq!(badge["publication_state"], "verified");
    assert_eq!(badge["achieved_at"], "2026-08-15T22:14:15.614Z");

    let project_id = badge["project_id"]
        .as_u64()
        .expect("BadgeApp project ID should be a positive integer");
    assert!(project_id > 0);
    assert_eq!(
        badge["project_url"],
        format!("https://www.bestpractices.dev/en/projects/{project_id}")
    );
    assert_eq!(
        badge["project_json_url"],
        format!("https://www.bestpractices.dev/projects/{project_id}.json")
    );
    assert_eq!(
        badge["baseline_badge_url"],
        format!("https://www.bestpractices.dev/projects/{project_id}/baseline")
    );
    assert_eq!(
        badge["baseline_entry_url"],
        format!("https://www.bestpractices.dev/en/projects/{project_id}/baseline-1")
    );

    assert_eq!(canonical["slsa"]["version"], "v1.2");
    assert_eq!(canonical["slsa"]["build_level"], 2);
    assert_eq!(
        canonical["slsa"]["predicate_type"],
        "https://slsa.dev/provenance/v1"
    );
    assert_eq!(canonical["slsa"]["release"]["tag"], "v0.3.0");
    assert_eq!(canonical["slsa"]["release"]["immutable"], true);
    assert_eq!(canonical["slsa"]["verifier"]["version"], "2.97.0");
    assert_eq!(canonical["slsa"]["verifier"]["release_immutable"], true);
    assert_eq!(
        canonical["slsa"]["verifier"]["assets"]
            .as_array()
            .expect("reviewed verifier assets should be an array")
            .len(),
        2
    );
    assert_eq!(
        canonical["slsa"]["verifier"]["assets"][0]["host"],
        "aarch64-apple-darwin"
    );
    assert_eq!(
        canonical["slsa"]["verifier"]["assets"][1]["host"],
        "x86_64-unknown-linux-gnu"
    );
    assert_eq!(canonical["maintenance"]["cadence"], "at_least_annually");
    assert_eq!(
        canonical["maintenance"]["next_scheduled_review_by"],
        "2027-08-15"
    );
    assert_eq!(
        canonical["maintenance"]["event_triggers"]
            .as_array()
            .expect("event triggers should be an array")
            .len(),
        7
    );
}

#[test]
fn badgeapp_proposal_has_only_reviewable_met_answers_linked_to_the_crosswalk() {
    let proposal = repository_json(".bestpractices.json");
    let crosswalk = "https://github.com/EnjoyableWork/mcp-doctor/blob/main/docs/assurance/osps-v2026.02.19-level-1.md";

    let status_fields = proposal
        .as_object()
        .expect("proposal should be an object")
        .keys()
        .filter(|key| key.starts_with("osps_") && key.ends_with("_status"))
        .count();
    assert_eq!(status_fields, 24);

    for control in CONTROLS {
        assert_eq!(proposal[proposal_field(control, "status")], "Met");
        assert_eq!(
            proposal[proposal_field(control, "justification")],
            format!("Dated evidence and scope: {crosswalk}")
        );
    }
}

#[test]
fn osps_crosswalk_covers_every_level_one_control_without_overclaiming() {
    let crosswalk = repository_file("docs/assurance/osps-v2026.02.19-level-1.md");

    for control in CONTROLS {
        assert_eq!(
            crosswalk.matches(&format!("`{control}`")).count(),
            1,
            "crosswalk should contain exactly one assessment row for {control}"
        );
    }
    for contract in [
        "Assessment date | 2026-08-15 UTC",
        "24 `Met`; 0 `N/A`; 0 `Unmet`",
        "official-hosted self-assessment",
        "not an independent certification",
        "next scheduled review due by\n2027-08-15",
        "corrected or removed from the README immediately",
    ] {
        assert!(
            crosswalk.contains(contract),
            "OSPS crosswalk should preserve {contract}"
        );
    }
    for forbidden in [
        "independently certified",
        "regulatory compliant",
        "OSPS Level 2: Met",
        "OSPS Level 3: Met",
    ] {
        assert!(!crosswalk.contains(forbidden));
    }
}

#[test]
fn slsa_crosswalk_lists_every_canonical_release_asset_and_l2_requirement() {
    let crosswalk = repository_file("docs/assurance/slsa-v1.2-build-l2.md");
    let community = repository_json(".github/community-license-controls.json");
    let assets = community["release_license_contract"]["assets"]
        .as_array()
        .expect("canonical release assets should be an array");

    assert_eq!(assets.len(), 7);
    for asset in assets {
        let name = asset["name"].as_str().expect("asset name should be text");
        let digest = asset["sha256"]
            .as_str()
            .expect("asset digest should be text");
        assert!(crosswalk.contains(&format!("`{name}`")));
        assert!(crosswalk.contains(&format!("`{digest}`")));
    }
    for contract in [
        "SLSA `v1.2` Build L2",
        "Build L1 producer: follow a consistent build process.",
        "Build L1 producer: use a platform meeting Build L1.",
        "Build L1 producer: distribute provenance to consumers.",
        "Build L1 platform: automatically describe builder, process, and top-level input.",
        "Build L2 producer: use a hosted platform meeting Build L2.",
        "Build L2 platform: generate and sign the provenance itself.",
        "Build L2 consumer: validate provenance authenticity.",
        "https://slsa.dev/provenance/v1",
        "--deny-self-hosted-runners",
        "It does not cover the crates.io upload operation",
    ] {
        assert!(
            crosswalk.contains(contract),
            "SLSA crosswalk should preserve {contract}"
        );
    }
    assert!(!crosswalk.contains("Build L3 | Meets"));
    assert!(!crosswalk.contains("mcp-doctor is SLSA certified"));
}

#[test]
fn verifier_is_exact_bounded_and_value_minimizing() {
    let verifier = repository_file("scripts/verify-assurance-evidence.sh");

    for contract in [
        "GitHub CLI does not match the exact reviewed assurance verifier release",
        "https://github.com/cli/cli/releases/download/$assurance_gh_tag/$assurance_gh_archive",
        "grep -Fx \"$assurance_gh_archive_sha256  $assurance_gh_archive\"",
        "--retry 0",
        "--max-filesize",
        "--signer-workflow \"$assurance_signer\"",
        "--signer-digest \"$assurance_release_source\"",
        "--source-ref \"refs/tags/$assurance_release_tag\"",
        "--source-digest \"$assurance_release_source\"",
        "--cert-oidc-issuer https://token.actions.githubusercontent.com",
        "--deny-self-hosted-runners",
        "--predicate-type https://slsa.dev/provenance/v1",
        "the exact assurance verifier has no reviewed asset for this host",
        "badgeapp_project=%s release=%s assets=%s result=PASS",
    ] {
        assert!(
            verifier.contains(contract),
            "assurance verifier should preserve {contract}"
        );
    }
    for forbidden in [
        "--retry 1",
        "--retry 5",
        "--insecure",
        "--owner EnjoyableWork",
    ] {
        assert!(!verifier.contains(forbidden));
    }

    // The executable verifier intentionally supports only its reviewed macOS
    // and GNU/Linux hosts. Windows still proves the static safety contract
    // above without depending on an incidental Bash installation.
    #[cfg(not(windows))]
    {
        let invalid = std::process::Command::new("bash")
            .arg(repository_root().join("scripts/verify-assurance-evidence.sh"))
            .arg("--unknown")
            .output()
            .expect("invalid verifier invocation should run");
        assert_eq!(invalid.status.code(), Some(2));
        assert!(invalid.stdout.is_empty());
        assert!(String::from_utf8_lossy(&invalid.stderr).contains("usage:"));
    }
}

#[test]
fn readme_publishes_only_the_scoped_achieved_result() {
    let canonical = repository_json(".github/assurance-controls.json");
    let project_id = canonical["osps"]["badgeapp"]["project_id"]
        .as_u64()
        .expect("BadgeApp project ID should be an integer");
    let readme = repository_file("README.md");
    let badge = format!(
        "<a href=\"https://www.bestpractices.dev/en/projects/{project_id}/baseline-1\"><img alt=\"OpenSSF OSPS Baseline v2026.02.19 Level 1\" src=\"https://www.bestpractices.dev/projects/{project_id}/baseline\"></a>"
    );

    assert!(readme.contains(&badge));
    assert!(readme.contains("official-hosted, scoped self-assessment"));
    assert!(readme.contains("SLSA `v1.2` Build L2 evaluation"));
    assert!(!readme.contains("independently certified"));
    assert!(!readme.contains("SLSA certified"));
}
