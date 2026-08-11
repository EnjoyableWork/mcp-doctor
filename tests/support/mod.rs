#![allow(
    dead_code,
    reason = "shared integration support grows with later built-binary journeys"
)]

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::collections::BTreeMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const STABLE_REPORT_SCHEMA: &str = include_str!("../../schemas/mcp-doctor.report.v1.schema.json");

pub fn parse_and_validate_report(bytes: &[u8]) -> serde_json::Value {
    let report: serde_json::Value =
        serde_json::from_slice(bytes).expect("machine output should be one JSON report");
    validate_report_value(report)
}

pub fn validate_report_value(report: serde_json::Value) -> serde_json::Value {
    let schema: serde_json::Value = serde_json::from_str(STABLE_REPORT_SCHEMA)
        .expect("the committed stable report schema should be JSON");
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .expect("the committed stable report schema should follow Draft 2020-12");
    let errors = validator
        .iter_errors(&report)
        .map(|error| error.instance_path().to_string())
        .collect::<Vec<_>>();
    assert!(
        errors.is_empty(),
        "stable report schema rejected synthetic fields at {errors:?}"
    );
    report
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct JunitSummary {
    pub tests: usize,
    pub failures: usize,
    pub skipped: usize,
}

pub fn parse_and_validate_junit(bytes: &[u8]) -> (String, JunitSummary) {
    let document = std::str::from_utf8(bytes)
        .expect("JUnit output should be UTF-8")
        .to_owned();
    let mut reader = Reader::from_str(&document);
    let mut stack = Vec::<Vec<u8>>::new();
    let mut root_summary = None;
    let mut suite_summary = None;
    let mut observed = JunitSummary {
        tests: 0,
        failures: 0,
        skipped: 0,
    };
    let mut system_outputs = 0;

    loop {
        match reader
            .read_event()
            .expect("an independent XML parser should accept the JUnit output")
        {
            Event::Start(element) => {
                let name = element.name().as_ref().to_vec();
                let parent = stack.last().map(Vec::as_slice);
                match name.as_slice() {
                    b"testsuites" => {
                        assert!(parent.is_none(), "testsuites should be the root element");
                        assert!(
                            root_summary.is_none(),
                            "there should be one testsuites root"
                        );
                        root_summary = Some(junit_summary(&element));
                    }
                    b"testsuite" => {
                        assert_eq!(parent, Some(b"testsuites".as_slice()));
                        assert!(suite_summary.is_none(), "there should be one testsuite");
                        require_attribute(&element, b"name");
                        require_zero_attribute(&element, b"errors");
                        require_zero_attribute(&element, b"time");
                        suite_summary = Some(junit_summary(&element));
                    }
                    b"testcase" => {
                        assert_eq!(parent, Some(b"testsuite".as_slice()));
                        require_attribute(&element, b"classname");
                        require_attribute(&element, b"name");
                        require_zero_attribute(&element, b"time");
                        observed.tests += 1;
                    }
                    b"failure" => {
                        assert_eq!(parent, Some(b"testcase".as_slice()));
                        require_attribute(&element, b"message");
                        require_attribute(&element, b"type");
                        observed.failures += 1;
                    }
                    b"skipped" => {
                        assert_eq!(parent, Some(b"testcase".as_slice()));
                        require_attribute(&element, b"message");
                        observed.skipped += 1;
                    }
                    b"system-out" => {
                        assert_eq!(parent, Some(b"testcase".as_slice()));
                        system_outputs += 1;
                    }
                    name => panic!("unexpected element in common JUnit output: {name:?}"),
                }
                for attribute in element.attributes() {
                    attribute.expect("every JUnit attribute should parse");
                }
                stack.push(name);
            }
            Event::End(element) => {
                let expected = stack.pop().expect("every JUnit end tag has a start tag");
                assert_eq!(element.name().as_ref(), expected);
            }
            Event::GeneralRef(reference) => assert!(
                matches!(
                    reference.as_ref(),
                    b"amp" | b"lt" | b"gt" | b"apos" | b"quot"
                ),
                "JUnit output should use only predefined XML entities"
            ),
            Event::Decl(_) | Event::Text(_) => {}
            Event::Eof => break,
            event => panic!("unexpected XML event in common JUnit output: {event:?}"),
        }
    }

    assert!(stack.is_empty(), "every JUnit element should close");
    assert_eq!(system_outputs, observed.tests);
    assert_eq!(root_summary, Some(observed));
    assert_eq!(suite_summary, Some(observed));
    (document, observed)
}

fn junit_summary(element: &BytesStart<'_>) -> JunitSummary {
    require_attribute(element, b"name");
    require_zero_attribute(element, b"errors");
    require_zero_attribute(element, b"time");
    JunitSummary {
        tests: numeric_attribute(element, b"tests"),
        failures: numeric_attribute(element, b"failures"),
        skipped: numeric_attribute(element, b"skipped"),
    }
}

fn require_attribute(element: &BytesStart<'_>, name: &[u8]) {
    assert!(
        attribute_value(element, name).is_some(),
        "missing JUnit attribute"
    );
}

fn require_zero_attribute(element: &BytesStart<'_>, name: &[u8]) {
    assert_eq!(
        attribute_value(element, name).as_deref(),
        Some(b"0".as_slice())
    );
}

fn numeric_attribute(element: &BytesStart<'_>, name: &[u8]) -> usize {
    let value = attribute_value(element, name).expect("missing numeric JUnit attribute");
    std::str::from_utf8(&value)
        .expect("numeric JUnit attributes should be ASCII")
        .parse()
        .expect("numeric JUnit attributes should contain nonnegative integers")
}

fn attribute_value(element: &BytesStart<'_>, name: &[u8]) -> Option<Vec<u8>> {
    element
        .attributes()
        .map(|attribute| attribute.expect("every JUnit attribute should parse"))
        .find(|attribute| attribute.key.as_ref() == name)
        .map(|attribute| attribute.value.into_owned())
}

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
        for name in platform_launch_environment_names() {
            if let Some(value) = env::var_os(name) {
                command.env(name, value);
            }
        }

        command
    }

    pub fn artifact_path(&self, name: &str) -> PathBuf {
        self.root_path.join(name)
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

fn platform_launch_environment_names() -> &'static [&'static str] {
    #[cfg(windows)]
    {
        &["PATH", "PATHEXT", "SystemRoot", "WINDIR"]
    }

    #[cfg(not(windows))]
    {
        &["PATH"]
    }
}

impl Default for TestEnvironment {
    fn default() -> Self {
        Self::new()
    }
}
