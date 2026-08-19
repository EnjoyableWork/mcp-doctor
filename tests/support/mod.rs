#![allow(
    dead_code,
    reason = "shared integration support grows with later built-binary journeys"
)]

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::collections::BTreeMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

const STABLE_REPORT_SCHEMA: &str = include_str!("../../schemas/mcp-doctor.report.v1.schema.json");
const STABLE_REPORT_SCHEMA_ID: &str = "https://github.com/EnjoyableWork/mcp-doctor/blob/main/schemas/mcp-doctor.report.v1.schema.json";
const STABLE_AGGREGATE_SCHEMA: &str =
    include_str!("../../schemas/mcp-doctor.aggregate.v1.schema.json");
const STABLE_CAPABILITIES_SCHEMA: &str =
    include_str!("../../schemas/mcp-doctor.capabilities.v1.schema.json");
const CONTRACT_SNAPSHOT_SCHEMA: &str =
    include_str!("../../schemas/mcp-doctor.contract-snapshot.v1alpha1.schema.json");
const CONTRACT_DIFF_SCHEMA: &str =
    include_str!("../../schemas/mcp-doctor.contract-diff.v1alpha1.schema.json");
const DESCENDANT_READY_MARKER: &[u8] = b"descendant-ready\n";

pub fn run_with_bound_file_mutation(
    command: &mut Command,
    selected_path: &Path,
    mutate: impl FnOnce() -> std::io::Result<()>,
) -> Output {
    run_with_path_mutation_gate(
        command,
        selected_path,
        "MCP_DOCTOR_INTERNAL_TEST_BOUND_FILE_PATH",
        "MCP_DOCTOR_INTERNAL_TEST_BOUND_FILE_GATE",
        "bound-file",
        mutate,
    )
}

pub fn run_with_report_publication_mutation(
    command: &mut Command,
    output_path: &Path,
    mutate: impl FnOnce() -> std::io::Result<()>,
) -> Output {
    let selected_path = canonical_report_output_path(output_path);
    run_with_path_mutation_gate(
        command,
        &selected_path,
        "MCP_DOCTOR_INTERNAL_TEST_REPORT_PUBLISH_PATH",
        "MCP_DOCTOR_INTERNAL_TEST_REPORT_PUBLISH_GATE",
        "report-publication",
        mutate,
    )
}

pub fn run_with_report_link_mutation(
    command: &mut Command,
    output_path: &Path,
    mutate: impl FnOnce() -> std::io::Result<()>,
) -> Output {
    let selected_path = canonical_report_output_path(output_path);
    run_with_path_mutation_gate(
        command,
        &selected_path,
        "MCP_DOCTOR_INTERNAL_TEST_REPORT_LINK_PATH",
        "MCP_DOCTOR_INTERNAL_TEST_REPORT_LINK_GATE",
        "report-link",
        mutate,
    )
}

fn canonical_report_output_path(output_path: &Path) -> PathBuf {
    let output_name = output_path
        .file_name()
        .expect("a report output path should have a filename");
    let output_parent = output_path.parent().unwrap_or_else(|| Path::new("."));
    fs::canonicalize(output_parent)
        .expect("the report output parent should have a canonical identity")
        .join(output_name)
}

fn run_with_path_mutation_gate(
    command: &mut Command,
    selected_path: &Path,
    selected_path_variable: &str,
    gate_variable: &str,
    gate_name: &str,
    mutate: impl FnOnce() -> std::io::Result<()>,
) -> Output {
    let listener = TcpListener::bind("127.0.0.1:0")
        .unwrap_or_else(|error| panic!("the {gate_name} event gate should bind: {error}"));
    let address = listener.local_addr().unwrap_or_else(|error| {
        panic!("the {gate_name} event gate should have an address: {error}")
    });
    command
        .env(selected_path_variable, selected_path)
        .env(gate_variable, address.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().unwrap_or_else(|error| {
        panic!("the command under the {gate_name} gate should start: {error}")
    });
    let accept_listener = listener
        .try_clone()
        .unwrap_or_else(|error| panic!("the {gate_name} event gate should clone: {error}"));
    let (accepted_sender, accepted_receiver) = mpsc::sync_channel(1);
    let acceptor = thread::spawn(move || {
        let accepted = accept_listener.accept().map(|(stream, _)| stream);
        let _ = accepted_sender.send(accepted);
    });

    let mut stream = match accepted_receiver.recv_timeout(Duration::from_secs(10)) {
        Ok(Ok(stream)) => stream,
        Ok(Err(error)) => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("the {gate_name} event gate should accept: {error}");
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = TcpStream::connect(address);
            let _ = acceptor.join();
            panic!("the command did not reach the {gate_name} event gate");
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("the {gate_name} event gate disconnected");
        }
    };
    acceptor
        .join()
        .unwrap_or_else(|_| panic!("the {gate_name} event gate should not panic"));
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap_or_else(|error| panic!("the {gate_name} readiness watchdog should set: {error}"));
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .unwrap_or_else(|error| {
            panic!("the {gate_name} acknowledgement watchdog should set: {error}")
        });
    let mut readiness = [0_u8; 1];
    if stream.read_exact(&mut readiness).is_err() || readiness != [1] {
        let _ = child.kill();
        let _ = child.wait();
        panic!("the command emitted an invalid {gate_name} readiness event");
    }

    let mutation = mutate();
    stream
        .write_all(&[2])
        .unwrap_or_else(|error| panic!("the {gate_name} acknowledgement should write: {error}"));
    let output = child.wait_with_output().unwrap_or_else(|error| {
        panic!("the command under the {gate_name} gate should return: {error}")
    });
    mutation.unwrap_or_else(|error| {
        panic!("the deterministic {gate_name} mutation should succeed: {error}")
    });
    output
}

pub fn assert_descendant_was_ready_and_terminated(path: &Path) {
    let mut marker = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("the descendant should publish its readiness marker before cleanup");
    marker
        .try_lock()
        .expect("the descendant still holds its readiness lock after cleanup");
    let mut contents = Vec::new();
    marker
        .read_to_end(&mut contents)
        .expect("the descendant readiness marker should be readable");
    assert_eq!(contents, DESCENDANT_READY_MARKER);
}

pub fn parse_and_validate_report(bytes: &[u8]) -> serde_json::Value {
    let report: serde_json::Value =
        serde_json::from_slice(bytes).expect("machine output should be one JSON report");
    validate_report_value(report)
}

pub fn parse_and_validate_markdown(bytes: &[u8]) -> String {
    let document = std::str::from_utf8(bytes)
        .expect("the Markdown report should be UTF-8")
        .to_owned();
    assert!(
        document.starts_with("<!-- mcp-doctor.markdown/v1 -->\n# mcp-doctor diagnostic report\n"),
        "the Markdown report should begin with its exact version marker and title"
    );
    assert!(document.ends_with('\n'));
    assert!(!document.contains('\r'));
    assert!(!document.contains('\u{1b}'));
    assert!(!document.contains("!["));
    let body = document
        .strip_prefix("<!-- mcp-doctor.markdown/v1 -->\n")
        .expect("the Markdown version marker should be present");
    assert!(!body.contains('<'), "Markdown should not contain raw HTML");
    assert!(!body.contains('>'), "Markdown should not contain raw HTML");
    document
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

pub fn parse_and_validate_aggregate(bytes: &[u8]) -> serde_json::Value {
    let aggregate: serde_json::Value =
        serde_json::from_slice(bytes).expect("machine output should be one JSON aggregate");
    let aggregate_schema: serde_json::Value = serde_json::from_str(STABLE_AGGREGATE_SCHEMA)
        .expect("the committed stable aggregate schema should be JSON");
    let report_schema: serde_json::Value = serde_json::from_str(STABLE_REPORT_SCHEMA)
        .expect("the committed stable report schema should be JSON");
    let registry = jsonschema::Registry::new()
        .add(
            STABLE_REPORT_SCHEMA_ID,
            jsonschema::Resource::from_contents(report_schema),
        )
        .expect("the stable report schema should register under its identifier")
        .prepare()
        .expect("the stable report schema registry should prepare without retrieval");
    let validator = jsonschema::draft202012::options()
        .with_registry(&registry)
        .build(&aggregate_schema)
        .expect("the committed aggregate schema should resolve only the local report schema");
    let errors = validator
        .iter_errors(&aggregate)
        .map(|error| error.instance_path().to_string())
        .collect::<Vec<_>>();
    assert!(
        errors.is_empty(),
        "stable aggregate schema rejected synthetic fields at {errors:?}"
    );
    aggregate
}

pub fn parse_and_validate_capabilities(bytes: &[u8]) -> serde_json::Value {
    parse_and_validate_schema(bytes, STABLE_CAPABILITIES_SCHEMA, "capabilities response")
}

pub fn parse_and_validate_contract_snapshot(bytes: &[u8]) -> serde_json::Value {
    parse_and_validate_schema(bytes, CONTRACT_SNAPSHOT_SCHEMA, "contract snapshot")
}

pub fn parse_and_validate_contract_diff(bytes: &[u8]) -> serde_json::Value {
    parse_and_validate_schema(bytes, CONTRACT_DIFF_SCHEMA, "contract diff")
}

fn parse_and_validate_schema(
    bytes: &[u8],
    schema_document: &str,
    artifact_name: &str,
) -> serde_json::Value {
    let artifact: serde_json::Value = serde_json::from_slice(bytes)
        .unwrap_or_else(|_| panic!("{artifact_name} should be one JSON document"));
    let schema: serde_json::Value = serde_json::from_str(schema_document)
        .unwrap_or_else(|_| panic!("the committed {artifact_name} schema should be JSON"));
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .unwrap_or_else(|_| panic!("the committed {artifact_name} schema should be valid"));
    let errors = validator
        .iter_errors(&artifact)
        .map(|error| error.instance_path().to_string())
        .collect::<Vec<_>>();
    assert!(
        errors.is_empty(),
        "{artifact_name} schema rejected synthetic fields at {errors:?}"
    );
    artifact
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
