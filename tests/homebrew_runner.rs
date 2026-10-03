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
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output};

    struct BrewFixture {
        root: tempfile::TempDir,
    }

    impl BrewFixture {
        fn new() -> Self {
            for path in ["/usr/bin/readlink", "/bin/rm"] {
                let metadata =
                    fs::metadata(path).expect("the declared filesystem command should exist");
                assert!(metadata.is_file() && metadata.permissions().mode() & 0o111 != 0);
            }
            let root = tempfile::tempdir().expect("disposable Homebrew fixture should exist");
            fs::create_dir_all(root.path().join("homebrew/bin"))
                .expect("the disposable prefix should exist");
            let brew = root.path().join("brew");
            fs::write(
                &brew,
                r#"#!/bin/bash
set -euo pipefail
printf '%s\n' "$*" >>"$HOMEBREW_FIXTURE_LOG"
test "$HOMEBREW_NO_AUTO_UPDATE" = 1
case "${1:-}" in
  --prefix)
    test "$#" -eq 1
    printf '%s\n' "${HOMEBREW_FIXTURE_REPORTED_PREFIX-$HOMEBREW_FIXTURE_PREFIX}"
    exit "${HOMEBREW_FIXTURE_PREFIX_STATUS:-0}"
    ;;
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
            for (name, source) in [
                (
                    "readlink",
                    r#"#!/bin/bash
set -euo pipefail
test "$HOMEBREW_FIXTURE_PREFIX" = "$HOMEBREW_FIXTURE_ROOT/homebrew"
test "$#" -eq 2
test "$1" = -n
test "$2" = "$HOMEBREW_FIXTURE_PREFIX/bin/openssl"
printf 'readlink -n bin/openssl\n' >>"$HOMEBREW_FIXTURE_LOG"
if [[ "${HOMEBREW_FIXTURE_READLINK_STATUS:-0}" != 0 ]]; then
  exit "$HOMEBREW_FIXTURE_READLINK_STATUS"
fi
exec /usr/bin/readlink -n "$2"
"#,
                ),
                (
                    "rm",
                    r#"#!/bin/bash
set -euo pipefail
test "$HOMEBREW_FIXTURE_PREFIX" = "$HOMEBREW_FIXTURE_ROOT/homebrew"
test "$#" -eq 2
test "$1" = --
test "$2" = "$HOMEBREW_FIXTURE_PREFIX/bin/openssl"
printf 'rm -- bin/openssl\n' >>"$HOMEBREW_FIXTURE_LOG"
if [[ "${HOMEBREW_FIXTURE_RM_STATUS:-0}" != 0 ]]; then
  exit "$HOMEBREW_FIXTURE_RM_STATUS"
fi
if [[ "${HOMEBREW_FIXTURE_RM_NOOP:-0}" == 1 ]]; then
  exit 0
fi
exec /bin/rm -- "$2"
"#,
                ),
            ] {
                let path = root.path().join(name);
                fs::write(&path, source).expect("confined filesystem command should be writable");
                fs::set_permissions(path, fs::Permissions::from_mode(0o755))
                    .expect("confined filesystem command should be executable");
            }
            Self { root }
        }

        fn prefix(&self) -> PathBuf {
            self.root.path().join("homebrew")
        }

        fn executable(&self) -> PathBuf {
            self.prefix().join("bin/openssl")
        }

        fn legacy_target(&self) -> PathBuf {
            self.prefix().join("opt/openssl@1.1/bin/openssl")
        }

        fn create_legacy_link(&self, live: bool) {
            let target = self.legacy_target();
            if live {
                fs::create_dir_all(target.parent().unwrap()).unwrap();
                fs::write(&target, b"synthetic legacy executable").unwrap();
            }
            symlink(target, self.executable()).expect("the synthetic legacy link should exist");
        }

        fn command(&self, formulas: &str) -> Command {
            // The cleared environment and stub-only PATH cannot reach host Homebrew;
            // filesystem wrappers reject every path except the fixture's exact link.
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
                .env("HOMEBREW_FIXTURE_ROOT", self.root.path())
                .env("HOMEBREW_FIXTURE_PREFIX", self.prefix())
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
        assert_eq!(
            fixture.calls(),
            "--prefix\nlist --formula -1\nunlink openssl@1.1\n"
        );
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
            assert_eq!(fixture.calls(), "--prefix\nlist --formula -1\n");
        }
    }

    #[test]
    fn failed_inventory_cannot_authorize_unlink_from_partial_output() {
        let fixture = BrewFixture::new();
        let output = run(fixture
            .command("openssl@1.1")
            .env("HOMEBREW_FIXTURE_LIST_STATUS", "19"));
        assert_eq!(output.status.code(), Some(19));
        assert_eq!(fixture.calls(), "--prefix\nlist --formula -1\n");
    }

    #[test]
    fn failed_unlink_stops_without_retry_or_other_changes() {
        let fixture = BrewFixture::new();
        let output = run(fixture
            .command("openssl@1.1")
            .env("HOMEBREW_FIXTURE_UNLINK_STATUS", "23"));
        assert_eq!(output.status.code(), Some(23));
        assert_eq!(
            fixture.calls(),
            "--prefix\nlist --formula -1\nunlink openssl@1.1\n"
        );
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
    fn missing_preparation_commands_fail_before_evidence_work() {
        for name in ["brew", "readlink", "rm"] {
            let fixture = BrewFixture::new();
            fs::remove_file(fixture.root.path().join(name)).unwrap();
            let output = run(&mut fixture.command("openssl@1.1"));
            assert_eq!(output.status.code(), Some(2));
            assert!(fixture.calls().is_empty());
        }
    }

    #[test]
    fn successful_keg_unlink_cannot_leave_the_exact_live_opt_link() {
        let fixture = BrewFixture::new();
        fixture.create_legacy_link(true);
        let output = run(&mut fixture.command("openssl@1.1"));
        assert!(output.status.success());
        assert!(fs::symlink_metadata(fixture.executable()).is_err());
        assert_eq!(
            fs::read(fixture.legacy_target()).unwrap(),
            b"synthetic legacy executable"
        );
        assert_eq!(
            fixture.calls(),
            "--prefix\nlist --formula -1\nunlink openssl@1.1\nreadlink -n bin/openssl\nrm -- bin/openssl\n"
        );
    }

    #[test]
    fn exact_dangling_legacy_link_is_removed_even_with_no_installed_keg() {
        let fixture = BrewFixture::new();
        fixture.create_legacy_link(false);
        let output = run(&mut fixture.command("openssl@3"));
        assert!(output.status.success());
        assert!(fs::symlink_metadata(fixture.executable()).is_err());
        assert_eq!(
            fixture.calls(),
            "--prefix\nlist --formula -1\nreadlink -n bin/openssl\nrm -- bin/openssl\n"
        );
    }

    #[test]
    fn other_live_and_dangling_links_and_regular_files_are_preserved() {
        for live in [true, false] {
            let fixture = BrewFixture::new();
            let target = fixture.prefix().join("opt/openssl@3/bin/openssl");
            if live {
                fs::create_dir_all(target.parent().unwrap()).unwrap();
                fs::write(&target, b"synthetic current executable").unwrap();
            }
            symlink(&target, fixture.executable()).unwrap();
            let output = run(&mut fixture.command("openssl@1.1"));
            assert!(output.status.success());
            assert_eq!(fs::read_link(fixture.executable()).unwrap(), target);
            assert!(!fixture.calls().contains("rm --"));
        }
        let fixture = BrewFixture::new();
        fs::write(fixture.executable(), b"synthetic unrelated file").unwrap();
        let output = run(&mut fixture.command("openssl@1.1"));
        assert!(output.status.success());
        assert_eq!(
            fs::read(fixture.executable()).unwrap(),
            b"synthetic unrelated file"
        );
        assert!(!fixture.calls().contains("readlink"));
        assert!(!fixture.calls().contains("rm --"));
    }

    #[test]
    fn relative_and_newline_suffixed_legacy_targets_do_not_match_literally() {
        for relative in [true, false] {
            let fixture = BrewFixture::new();
            let target = if relative {
                PathBuf::from("../opt/openssl@1.1/bin/openssl")
            } else {
                PathBuf::from(format!("{}\n", fixture.legacy_target().display()))
            };
            symlink(&target, fixture.executable()).unwrap();
            let output = run(&mut fixture.command("openssl@1.1"));
            assert!(output.status.success());
            assert_eq!(fs::read_link(fixture.executable()).unwrap(), target);
            assert!(!fixture.calls().contains("rm --"));
        }
    }

    #[test]
    fn false_success_removal_fails_the_absence_postcondition() {
        let fixture = BrewFixture::new();
        fixture.create_legacy_link(false);
        let output = run(fixture
            .command("openssl@1.1")
            .env("HOMEBREW_FIXTURE_RM_NOOP", "1"));
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(
            fs::read_link(fixture.executable()).unwrap(),
            fixture.legacy_target()
        );
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("remains after removal")
        );
        assert_eq!(fixture.calls().matches("rm --").count(), 1);
    }

    #[test]
    fn failed_prefix_readlink_and_removal_stop_without_retry() {
        for (name, status) in [
            ("HOMEBREW_FIXTURE_PREFIX_STATUS", 17),
            ("HOMEBREW_FIXTURE_READLINK_STATUS", 27),
            ("HOMEBREW_FIXTURE_RM_STATUS", 29),
        ] {
            let fixture = BrewFixture::new();
            fixture.create_legacy_link(true);
            let output = run(fixture.command("openssl@1.1").env(name, status.to_string()));
            assert_eq!(output.status.code(), Some(status));
            assert_eq!(
                fs::read_link(fixture.executable()).unwrap(),
                fixture.legacy_target()
            );
            assert_eq!(fixture.calls().matches("--prefix\n").count(), 1);
            assert!(fixture.calls().matches("rm --").count() <= 1);
        }
    }

    #[test]
    fn invalid_prefixes_stop_before_inventory_or_filesystem_commands() {
        let fixture = BrewFixture::new();
        let valid = fixture.prefix().display().to_string();
        for prefix in [
            String::new(),
            "relative".to_owned(),
            "/".to_owned(),
            format!("{valid}/"),
            format!("{valid}//bin"),
            format!("{valid}/./bin"),
            format!("{valid}/../homebrew"),
            format!("{valid}/."),
            format!("{valid}/.."),
            format!("{valid}\n"),
            format!("{valid}\nother"),
            format!("{valid}\rother"),
            format!("{valid}/missing"),
            format!("/{}", "x".repeat(4096)),
        ] {
            let fixture = BrewFixture::new();
            let output = run(fixture
                .command("openssl@1.1")
                .env("HOMEBREW_FIXTURE_REPORTED_PREFIX", prefix));
            assert_eq!(output.status.code(), Some(2));
            assert_eq!(fixture.calls(), "--prefix\n");
        }
    }
}
