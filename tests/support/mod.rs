#![allow(
    dead_code,
    reason = "shared integration support grows with later built-binary journeys"
)]

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

pub struct TestEnvironment {
    _root: TempDir,
    root_path: PathBuf,
    user_root: PathBuf,
}

impl TestEnvironment {
    pub fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("mcp-doctor-test-")
            .tempdir()
            .expect("disposable test root should be created");
        let root_path = root.path().to_owned();
        let user_root = root_path.join("user");

        for directory in [
            user_root.join(".cache"),
            user_root.join(".config"),
            user_root.join(".local/share"),
            user_root.join(".local/state"),
            user_root.join("AppData/Local"),
            user_root.join("AppData/Roaming"),
            root_path.join("runtime"),
            root_path.join("tmp"),
        ] {
            fs::create_dir_all(&directory).unwrap_or_else(|error| {
                panic!(
                    "synthetic directory {} should be created: {error}",
                    directory.display()
                )
            });
        }

        Self {
            _root: root,
            root_path,
            user_root,
        }
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_mcp-doctor"));
        command.env_clear();

        for (name, value) in self.environment() {
            command.env(name, value);
        }

        command
    }

    pub fn assert_command_is_isolated(&self, command: &Command) {
        let configured: BTreeMap<OsString, OsString> = command
            .get_envs()
            .filter_map(|(name, value)| value.map(|value| (name.to_owned(), value.to_owned())))
            .collect();

        for (name, expected) in self.environment() {
            let actual = configured
                .get(OsStr::new(name))
                .unwrap_or_else(|| panic!("{name} should be set for the CLI process"));
            assert_eq!(actual, expected.as_os_str());

            if Self::is_location(name) {
                self.assert_path_is_isolated(name, Path::new(actual));
            }
        }
    }

    fn environment(&self) -> [(&'static str, PathBuf); 17] {
        [
            ("APPDATA", self.user_root.join("AppData/Roaming")),
            ("CFFIXED_USER_HOME", self.user_root.clone()),
            ("HOME", self.user_root.clone()),
            ("LOCALAPPDATA", self.user_root.join("AppData/Local")),
            ("MCP_DOCTOR_TEST_MODE", PathBuf::from("1")),
            ("MCP_DOCTOR_TEST_ROOT", self.root_path.clone()),
            ("NO_COLOR", PathBuf::from("1")),
            ("TEMP", self.root_path.join("tmp")),
            ("TMP", self.root_path.join("tmp")),
            ("TMPDIR", self.root_path.join("tmp")),
            ("TZ", PathBuf::from("UTC")),
            ("USERPROFILE", self.user_root.clone()),
            ("XDG_CACHE_HOME", self.user_root.join(".cache")),
            ("XDG_CONFIG_HOME", self.user_root.join(".config")),
            ("XDG_DATA_HOME", self.user_root.join(".local/share")),
            ("XDG_RUNTIME_DIR", self.root_path.join("runtime")),
            ("XDG_STATE_HOME", self.user_root.join(".local/state")),
        ]
    }

    fn is_location(name: &str) -> bool {
        !matches!(name, "MCP_DOCTOR_TEST_MODE" | "NO_COLOR" | "TZ")
    }

    fn assert_path_is_isolated(&self, name: &str, path: &Path) {
        assert!(path.is_absolute(), "{name} should be an absolute path");
        assert!(
            path.starts_with(&self.root_path),
            "{name} should remain inside the disposable test root"
        );
    }
}

impl Default for TestEnvironment {
    fn default() -> Self {
        Self::new()
    }
}
