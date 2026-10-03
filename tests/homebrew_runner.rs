const PREFLIGHT: &str = include_str!("../.github/workflows/release-preflight.yml");
const CHANNELS: &str = include_str!("../.github/workflows/release-channels.yml");

#[test]
fn formula_jobs_prepare_only_macos_after_homebrew_setup_and_before_install() {
    let homebrew_channel = CHANNELS
        .split_once("\n  homebrew:\n")
        .expect("installed channels should have a Homebrew job")
        .1;
    for (workflow, invocation) in [
        (PREFLIGHT, "run: scripts/prepare-homebrew-runner.sh"),
        (
            homebrew_channel,
            "run: '\"$RUNNER_TEMP/mcp-doctor-prepare-homebrew-runner.sh\"'",
        ),
    ] {
        let setup = workflow
            .find("- name: Set up current stable Homebrew")
            .expect("Homebrew setup should exist");
        let preparation = workflow
            .find("- name: Prepare preinstalled Homebrew links on the macOS runner")
            .expect("runner preparation should exist");
        let installation = workflow
            .find("brew install --build-from-source")
            .expect("formula installation should exist");
        assert!(setup < preparation && preparation < installation);
        let preparation_step = &workflow[preparation..]
            .split_once("\n\n")
            .expect("preparation should be a separate step")
            .0;
        assert!(preparation_step.contains("if: runner.os == 'macOS'"));
        assert!(preparation_step.contains(invocation));
    }
}

#[test]
fn historical_channel_preserves_current_preparation_before_replacing_checkout() {
    let homebrew_channel = CHANNELS.split_once("\n  homebrew:\n").unwrap().1;
    let exact_source = homebrew_channel.find("ref: ${{ github.sha }}").unwrap();
    let verification = homebrew_channel
        .find("- name: Verify declared runner tools")
        .unwrap();
    let preservation = homebrew_channel
        .find("- name: Preserve the current Homebrew runner preparation")
        .unwrap();
    let historical_checkout = homebrew_channel
        .find("- name: Check out the immutable release tag")
        .unwrap();
    assert!(exact_source < verification && verification < preservation);
    assert!(preservation < historical_checkout);
    let preservation_step = &homebrew_channel[preservation..historical_checkout];
    assert!(preservation_step.contains("if: runner.os == 'macOS'"));
    assert!(preservation_step.contains("install -m 0755 scripts/prepare-homebrew-runner.sh"));
    assert!(preservation_step.contains("\"$RUNNER_TEMP/mcp-doctor-prepare-homebrew-runner.sh\""));
}

#[cfg(unix)]
mod unix {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::process::{Command, Output};

    struct BrewFixture {
        root: tempfile::TempDir,
    }

    impl BrewFixture {
        fn new() -> Self {
            let root = tempfile::tempdir().expect("disposable Homebrew fixture should exist");
            let brew = root.path().join("brew");
            fs::write(
                &brew,
                r#"#!/bin/bash
set -euo pipefail
printf '%s\n' "$*" >>"$HOMEBREW_FIXTURE_LOG"
test "$HOMEBREW_NO_AUTO_UPDATE" = 1
case "${1:-}" in
  list)
    test "$#" -eq 3
    test "$2" = --formula
    test "$3" = -1
    printf '%s\n' "$HOMEBREW_FIXTURE_FORMULAS"
    exit "${HOMEBREW_FIXTURE_LIST_STATUS:-0}"
    ;;
  unlink)
    test "$#" -eq 2
    test "$2" = openssl@1.1
    exit "${HOMEBREW_FIXTURE_UNLINK_STATUS:-0}"
    ;;
  *) exit 97 ;;
esac
"#,
            )
            .expect("stub brew should be writable");
            fs::set_permissions(&brew, fs::Permissions::from_mode(0o755))
                .expect("stub brew should be executable");
            Self { root }
        }

        fn command(&self, formulas: &str) -> Command {
            // The cleared environment and stub-only PATH cannot reach host Homebrew.
            let mut command = Command::new("/bin/bash");
            command
                .arg(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("scripts/prepare-homebrew-runner.sh"),
                )
                .current_dir(self.root.path())
                .env_clear()
                .env("PATH", self.root.path())
                .env("GITHUB_ACTIONS", "true")
                .env("RUNNER_ENVIRONMENT", "github-hosted")
                .env("RUNNER_OS", "macOS")
                .env("HOMEBREW_FIXTURE_FORMULAS", formulas)
                .env("HOMEBREW_FIXTURE_LOG", self.root.path().join("calls"));
            command
        }

        fn calls(&self) -> String {
            let path = self.root.path().join("calls");
            if path.exists() {
                fs::read_to_string(path).expect("stub call evidence should be readable")
            } else {
                String::new()
            }
        }
    }

    fn run(command: &mut Command) -> Output {
        command.output().expect("runner preparation should execute")
    }

    #[test]
    fn installed_old_keg_is_unlinked_once_with_literal_arguments() {
        let fixture = BrewFixture::new();
        let output = run(&mut fixture.command("openssl@3\nopenssl@1.1\nopenssl@1.1\nrust"));
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
        assert_eq!(fixture.calls(), "list --formula -1\nunlink openssl@1.1\n");
    }

    #[test]
    fn absent_and_unrelated_formulas_leave_links_unchanged() {
        for formulas in [
            "",
            "openssl@3\nrust",
            "openssl@1.10\nsynthetic/tap/openssl@1.1",
        ] {
            let fixture = BrewFixture::new();
            let output = run(&mut fixture.command(formulas));
            assert!(output.status.success());
            assert_eq!(fixture.calls(), "list --formula -1\n");
        }
    }

    #[test]
    fn failed_inventory_cannot_authorize_unlink_from_partial_output() {
        let fixture = BrewFixture::new();
        let output = run(fixture
            .command("openssl@1.1")
            .env("HOMEBREW_FIXTURE_LIST_STATUS", "19"));
        assert_eq!(output.status.code(), Some(19));
        assert_eq!(fixture.calls(), "list --formula -1\n");
    }

    #[test]
    fn failed_unlink_stops_without_retry_or_other_changes() {
        let fixture = BrewFixture::new();
        let output = run(fixture
            .command("openssl@1.1")
            .env("HOMEBREW_FIXTURE_UNLINK_STATUS", "23"));
        assert_eq!(output.status.code(), Some(23));
        assert_eq!(fixture.calls(), "list --formula -1\nunlink openssl@1.1\n");
    }

    #[test]
    fn local_self_hosted_and_other_platform_contexts_reject_before_brew() {
        for (name, value) in [
            ("GITHUB_ACTIONS", ""),
            ("RUNNER_ENVIRONMENT", "self-hosted"),
            ("RUNNER_OS", "Linux"),
        ] {
            let fixture = BrewFixture::new();
            let output = run(fixture.command("openssl@1.1").env(name, value));
            assert_eq!(output.status.code(), Some(2));
            assert!(fixture.calls().is_empty());
        }
        let fixture = BrewFixture::new();
        let output = run(fixture.command("openssl@1.1").arg("openssl@3"));
        assert_eq!(output.status.code(), Some(2));
        assert!(fixture.calls().is_empty());
    }

    #[test]
    fn missing_action_provided_brew_fails_before_evidence_work() {
        let fixture = BrewFixture::new();
        fs::remove_file(fixture.root.path().join("brew")).expect("stub brew should be removable");
        let output = run(&mut fixture.command("openssl@1.1"));
        assert_eq!(output.status.code(), Some(2));
        assert!(fixture.calls().is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("required action-provided command")
        );
    }
}
