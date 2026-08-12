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
    serde_json::from_str(&repository_file(".github/organization-controls.json"))
        .expect("organization controls should be valid JSON")
}

#[test]
fn canonical_projection_preserves_the_accepted_activation_boundary() {
    let controls = controls();

    assert_eq!(
        controls["schema_version"],
        "mcp-doctor.github-organization-controls/v1"
    );
    assert_eq!(controls["lifecycle"], "activation");
    assert_eq!(controls["api_version"], "2026-03-10");
    assert_eq!(controls["organization"], "EnjoyableWork");
    assert_eq!(
        controls["scope"]["repository_credential_scope"],
        json!(["EnjoyableWork/mcp-doctor", "EnjoyableWork/homebrew-tap"])
    );
    assert_eq!(controls["scope"]["private_repository_names_public"], false);

    let authentication = &controls["authentication"];
    assert_eq!(
        authentication["organization_two_factor_authentication_required"],
        true
    );
    assert_eq!(authentication["secure_two_factor_methods_required"], true);
    for count in [
        "members_with_two_factor_authentication_disabled",
        "members_with_insecure_two_factor_authentication",
        "outside_collaborators_with_two_factor_authentication_disabled",
        "outside_collaborators_with_insecure_two_factor_authentication",
    ] {
        assert_eq!(authentication[count], 0, "{count} must stay empty");
    }

    let access = &controls["access"];
    assert_eq!(access["exact_member_count"], 1);
    assert_eq!(access["exact_owner_count"], 1);
    assert_eq!(access["outside_collaborator_count"], 0);
    assert_eq!(access["pending_invitation_count"], 0);
    assert_eq!(access["default_repository_permission"], "none");
    assert_eq!(access["manual_permission_assignment_required"], true);
    assert_eq!(access["non_owner_direct_admin_identity_count"], 0);
    let member_privileges = access["member_privileges"]
        .as_object()
        .expect("member privileges should be an object");
    assert_eq!(member_privileges.len(), 13);
    assert!(
        member_privileges
            .values()
            .all(|value| value == &json!(false)),
        "every non-owner member privilege must remain disabled"
    );

    let applications = &controls["installed_applications"];
    assert_eq!(applications["installation_authority"], "owners_only");
    assert_eq!(applications["access_requests"], "owner_review_required");
    assert_eq!(applications["repository_selection"], "selected_only");
    assert_eq!(applications["all_repository_access_allowed"], false);
    assert!(applications["approved_installation_count"].is_null());
    assert_eq!(
        applications["approved_inventory_location"],
        "private_attestation"
    );
    assert_eq!(applications["approved_inventory_public"], false);
    assert!(
        applications
            .as_object()
            .expect("installed applications should be an object")
            .get("approved_inventory_sha256")
            .is_none(),
        "the private inventory digest must not enter public configuration"
    );

    let credentials = &controls["automation_credentials"];
    assert_eq!(
        credentials["normal_automation_credentials"],
        json!([
            "github_actions_job_token",
            "github_oidc",
            "github_app_installation_token"
        ])
    );
    assert_eq!(
        credentials["classic_personal_access_token_access"],
        "blocked"
    );
    assert_eq!(
        credentials["fine_grained_personal_access_tokens"],
        json!({
            "access": "exception_only",
            "approval": "owner_required",
            "maximum_lifetime_days": 30,
            "repository_selection": "exact",
            "permissions": "minimum",
            "automation_use": "prohibited"
        })
    );
    assert!(
        credentials["organization"]
            .as_object()
            .expect("organization credential counts should be an object")
            .values()
            .all(|value| value == &json!(0))
    );
    let repositories = credentials["repositories"]
        .as_array()
        .expect("repository credential counts should be an array");
    assert_eq!(repositories.len(), 2);
    for repository in repositories {
        let fields = repository
            .as_object()
            .expect("repository credential entry should be an object");
        assert!(
            fields
                .iter()
                .filter(|(name, _)| name.as_str() != "repository")
                .all(|(_, value)| value == &json!(0)),
            "every in-scope stored credential count must stay empty"
        );
    }

    assert_eq!(
        controls["ownership_continuity"]["model"],
        "single_owner_with_explicit_residual_risk"
    );
    assert_eq!(
        controls["ownership_continuity"]["residual_risk_accepted"],
        true
    );
    assert_eq!(
        controls["ownership_continuity"]["shared_accounts_allowed"],
        false
    );
    assert_eq!(controls["recovery"]["cadence_months"], 6);
    assert_eq!(controls["recovery"]["maximum_age_days"], 184);
    assert!(controls["recovery"]["latest_exercise_on"].is_null());
    assert_eq!(
        controls["verification"]["limits"],
        json!({
            "maximum_api_requests": 128,
            "maximum_total_seconds": 900,
            "maximum_connect_seconds": 10,
            "maximum_request_seconds": 30,
            "maximum_response_bytes": 4_194_304,
            "maximum_organization_repositories": 32,
            "maximum_installations": 16,
            "maximum_repositories_per_installation": 32,
            "maximum_direct_collaborators_per_repository": 99,
            "maximum_environments_per_repository": 16
        })
    );
    assert_eq!(
        controls["mapped_controls"],
        json!(["OSPS-AC-01.01", "OSPS-AC-02.01"])
    );
    assert!(
        controls["private_attestation"]["required_assertions"]
            .as_array()
            .expect("private assertion inventory should be an array")
            .contains(&json!(
                "operator_credential_reviewed_for_current_verification"
            ))
    );
}

#[test]
fn project_resolves_open_10_without_claiming_live_completion() {
    let project = repository_file("PROJECT.md");

    for contract in [
        "`MCPD-016` is Done and `MCPD-017` is In progress",
        "### Accepted organization access, credential, continuity, and recovery contract",
        "accepted choices `1B`, `2A`, and `3A` on 2026-08-12",
        "| DEC-041 | Resolve `OPEN-10` with strong MFA, lowest-default access, owner-reviewed short-lived authority, explicit single-owner risk, and private recovery proof | Accepted |",
        "`OPEN-10` is accepted as `DEC-041`",
        "There are no unresolved open decisions.",
        "No live setting, installation, credential, key, or recovery path changed during",
        "`MCPD-017` remains In progress",
        "| RISK-15 | Organization-owner loss or over-broad long-lived credentials become an undocumented recovery dependency",
        "Policy accepted; activation implementation in progress",
    ] {
        assert!(
            project.contains(contract),
            "PROJECT.md should preserve the OPEN-10 resolution: {contract}"
        );
    }
    for stale_or_premature in [
        "| OPEN-10 |",
        "`MCPD-017` is Ready but has not begun",
        "| MCPD-017 | Establish organization access, credential, ownership, and recovery policy | M4 | Ready |",
        "| MCPD-017 | Establish organization access, credential, ownership, and recovery policy | M4 | Done |",
    ] {
        assert!(
            !project.contains(stale_or_premature),
            "PROJECT.md must not retain {stale_or_premature}"
        );
    }
}

#[test]
fn verifier_is_bounded_non_disclosing_and_fixture_gated() {
    let verifier = repository_file("scripts/verify-organization-controls.sh");
    let rehearsal = repository_file("scripts/rehearse-organization-controls.sh");
    let workflow = repository_file(".github/workflows/ci.yml");

    for contract in [
        "--private-attestation",
        "--verification-date",
        "MCP_DOCTOR_ORGANIZATION_FIXTURE",
        "mode=fixture",
        "organization_regular_bounded_file",
        "organization_private_file_mode_is_restricted",
        "umask 077",
        "GH_HOST=github.com",
        "-u HTTPS_PROXY -u https_proxy",
        "-u SSL_CERT_DIR -u SSL_CERT_FILE -u SSLKEYLOGFILE",
        "curl --disable",
        "--max-filesize",
        "maximum_api_requests",
        "https://api.github.com/${organization_endpoint}",
        "git -C \"${organization_repository_root}\" status --short",
        "orgs/${organization_name}/members?filter=2fa_insecure",
        "orgs/${organization_name}/installations?per_page=100",
        "approved_inventory_location",
        "operator_credential_reviewed_for_current_verification",
        "classic_personal_access_token",
        "result=FAIL",
        "result=PASS",
        "organization_verification_date_override",
    ] {
        assert!(
            verifier.contains(contract),
            "organization verifier should preserve {contract}"
        );
    }
    for forbidden in [
        "set -x",
        "--show-token",
        "organization_gh api",
        "result_json",
        "app_slug=%",
    ] {
        assert!(
            !verifier.contains(forbidden),
            "organization verifier must not contain {forbidden}"
        );
    }
    for negative in [
        "an activation-only canonical projection",
        "a fixture impersonating a live source commit",
        "an insecure MFA member",
        "a broad default repository permission",
        "a broad member privilege",
        "an all-repository application",
        "application inventory drift",
        "a classic personal access token",
        "an organization Actions secret",
        "an unreviewed operator credential",
        "a group-or-world-readable private attestation",
        "a distribution write deploy key",
        "an unconfirmed recovery exercise",
        "stale recovery evidence",
        "a stale organization-control review",
    ] {
        assert!(
            rehearsal.contains(negative),
            "organization rehearsal should reject {negative}"
        );
    }
    assert!(rehearsal.contains("rehearsal_require_no_disclosure"));
    assert!(rehearsal.contains("rehearsal_reference_date=\"2030-01-01\""));
    assert!(rehearsal.contains("grep -Fq"));
    assert!(rehearsal.contains("--verification-date \"${rehearsal_reference_date}\""));
    assert!(!rehearsal.contains(&["rehearsal", "_today"].concat()));
    assert!(!rehearsal.contains(&["rg", " -F"].concat()));
    assert!(workflow.contains("Rehearse non-disclosing organization-control verification"));
    assert!(workflow.contains("./scripts/rehearse-organization-controls.sh"));
}

#[test]
#[cfg_attr(
    windows,
    ignore = "organization-control rehearsal executes through Bash"
)]
fn fixture_date_is_explicit_and_live_verification_cannot_override_clock() {
    let repository = repository_root();
    let verifier = "scripts/verify-organization-controls.sh";
    let private_fixture = "tests/fixtures/organization-controls/private-pass.json";
    let live_fixture = "tests/fixtures/organization-controls/live-pass.json";

    let live_override = Command::new("bash")
        .arg(verifier)
        .args([
            "--private-attestation",
            private_fixture,
            "--verification-date",
            "2030-01-01",
        ])
        .current_dir(&repository)
        .output()
        .expect("live override rejection should execute");
    assert_eq!(live_override.status.code(), Some(2));
    assert!(live_override.stdout.is_empty());
    assert!(String::from_utf8_lossy(&live_override.stderr).starts_with("usage: "));

    let missing_fixture_date = Command::new("bash")
        .arg(verifier)
        .args([
            "--canonical",
            ".github/organization-controls.json",
            "--private-attestation",
            private_fixture,
            "--fixture",
            live_fixture,
        ])
        .env("MCP_DOCTOR_ORGANIZATION_FIXTURE", "1")
        .current_dir(&repository)
        .output()
        .expect("missing fixture date rejection should execute");
    assert_eq!(missing_fixture_date.status.code(), Some(2));
    assert!(missing_fixture_date.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing_fixture_date.stderr).starts_with("usage: "));

    let invalid_fixture_date = Command::new("bash")
        .arg(verifier)
        .args([
            "--canonical",
            ".github/organization-controls.json",
            "--private-attestation",
            private_fixture,
            "--fixture",
            live_fixture,
            "--verification-date",
            "2030-02-30",
        ])
        .env("MCP_DOCTOR_ORGANIZATION_FIXTURE", "1")
        .current_dir(repository)
        .output()
        .expect("invalid fixture date rejection should execute");
    assert_eq!(invalid_fixture_date.status.code(), Some(2));
    assert!(invalid_fixture_date.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&invalid_fixture_date.stderr),
        "organization-control verification date is invalid\n"
    );
}

#[test]
#[cfg_attr(
    windows,
    ignore = "organization-control rehearsal executes through Bash"
)]
fn organization_control_rehearsal_proves_pass_fail_and_redaction_paths() {
    let output = Command::new("bash")
        .arg("scripts/rehearse-organization-controls.sh")
        .current_dir(repository_root())
        .output()
        .expect("organization-control rehearsal should execute");

    assert!(
        output.status.success(),
        "organization-control rehearsal failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Organization-control verifier rehearsal passed.\n"
    );
    assert!(output.stderr.is_empty());
}
