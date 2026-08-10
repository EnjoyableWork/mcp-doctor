use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, ExitCode};
use std::thread;
use std::time::Duration;

use serde_json::Value;

const MIB: usize = 1024 * 1024;
const REDACTION_SENTINEL: &str = "synthetic-secret-payload-7f2c";

fn main() -> ExitCode {
    let mut arguments = env::args_os().skip(1);
    let Some(mode) = arguments.next() else {
        return ExitCode::from(2);
    };
    let remaining = arguments.collect::<Vec<_>>();

    match mode.to_str() {
        Some("success") => success(&remaining),
        Some("literal-arguments") => literal_arguments(&remaining),
        Some("environment") => environment(),
        Some("malformed") => malformed(),
        Some("oversized-message") => oversized_message(),
        Some("stdout-oversize") => stdout_oversize(),
        Some("stderr-oversize") => stderr_oversize(),
        Some("aggregate-oversize") => aggregate_oversize(),
        Some("message-count") => message_count(),
        Some("timeout") => timeout(),
        Some("early-exit") => early_exit(),
        Some("resistant-child") => resistant_child(&remaining),
        Some("descendant") => descendant(&remaining),
        _ => ExitCode::from(2),
    }
}

fn success(arguments: &[OsString]) -> ExitCode {
    let Some(marker) = arguments.first().map(PathBuf::from) else {
        return ExitCode::from(2);
    };
    let mut input = io::BufReader::new(io::stdin().lock());
    read_discover_request(&mut input);
    write_success_response();

    let mut unexpected = Vec::new();
    input
        .read_to_end(&mut unexpected)
        .expect("the remaining STDIN bytes should be readable");
    if !unexpected.is_empty() {
        fs::write(marker, b"unexpected request")
            .expect("the unexpected-request marker should be writable");
    }
    ExitCode::SUCCESS
}

fn literal_arguments(arguments: &[OsString]) -> ExitCode {
    let expected = [
        OsString::from("space value"),
        OsString::from("$MCP_DOCTOR_LITERAL"),
        OsString::from("; synthetic-command"),
        OsString::from("$(synthetic-command)"),
    ];
    assert_eq!(arguments, expected);
    respond_then_wait_for_eof();
    ExitCode::SUCCESS
}

fn environment() -> ExitCode {
    for forbidden in [
        "APPDATA",
        "CFFIXED_USER_HOME",
        "HOME",
        "LOCALAPPDATA",
        "MCP_DOCTOR_ENV_SENTINEL",
        "MCP_DOCTOR_TEST_MODE",
        "MCP_DOCTOR_TEST_ROOT",
        "NO_COLOR",
        "TEMP",
        "TMP",
        "TMPDIR",
        "TZ",
        "USERPROFILE",
        "XDG_CACHE_HOME",
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
        "XDG_RUNTIME_DIR",
        "XDG_STATE_HOME",
    ] {
        assert!(env::var_os(forbidden).is_none(), "{forbidden} leaked");
    }
    respond_then_wait_for_eof();
    ExitCode::SUCCESS
}

fn malformed() -> ExitCode {
    read_one_discover_request();
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{{\"value\":\"{REDACTION_SENTINEL}\"")
        .expect("the malformed frame should be writable");
    stdout.flush().expect("STDOUT should flush");
    wait_forever()
}

fn oversized_message() -> ExitCode {
    read_one_discover_request();
    let mut stdout = io::stdout().lock();
    write_repeated(&mut stdout, b'x', MIB + 1);
    stdout.flush().expect("STDOUT should flush");
    wait_forever()
}

fn stdout_oversize() -> ExitCode {
    read_one_discover_request();
    let mut stdout = io::stdout().lock();
    for _ in 0..9 {
        write_notification(&mut stdout, 960 * 1024);
    }
    stdout.flush().expect("STDOUT should flush");
    wait_forever()
}

fn stderr_oversize() -> ExitCode {
    read_one_discover_request();
    let mut stderr = io::stderr().lock();
    write_repeated(&mut stderr, b's', MIB + 1);
    stderr.flush().expect("STDERR should flush");
    wait_forever()
}

fn aggregate_oversize() -> ExitCode {
    read_one_discover_request();
    let mut stdout = io::stdout().lock();
    for _ in 0..8 {
        write_notification(&mut stdout, 960 * 1024);
    }
    stdout.flush().expect("STDOUT should flush");

    let mut stderr = io::stderr().lock();
    write_repeated(&mut stderr, b'a', 768 * 1024);
    stderr.flush().expect("STDERR should flush");
    wait_forever()
}

fn message_count() -> ExitCode {
    read_one_discover_request();
    let mut stdout = io::stdout().lock();
    for _ in 0..1_025 {
        writeln!(
            stdout,
            "{{\"jsonrpc\":\"2.0\",\"method\":\"synthetic/progress\"}}"
        )
        .expect("the notification should be writable");
    }
    stdout.flush().expect("STDOUT should flush");
    wait_forever()
}

fn timeout() -> ExitCode {
    read_one_discover_request();
    wait_forever()
}

fn early_exit() -> ExitCode {
    read_one_discover_request();
    ExitCode::SUCCESS
}

fn resistant_child(arguments: &[OsString]) -> ExitCode {
    let Some(marker) = arguments.first().map(PathBuf::from) else {
        return ExitCode::from(2);
    };
    read_one_discover_request();

    let descendant =
        Command::new(env::current_exe().expect("the fixture path should be available"))
            .arg("descendant")
            .arg(marker)
            .spawn()
            .expect("the resistant descendant should start");

    write_success_response();
    wait_with_child(descendant)
}

fn descendant(arguments: &[OsString]) -> ExitCode {
    let Some(marker) = arguments.first().map(PathBuf::from) else {
        return ExitCode::from(2);
    };
    thread::sleep(Duration::from_millis(3_500));
    fs::write(marker, b"survived cleanup")
        .expect("the descendant survival marker should be writable");
    wait_forever()
}

fn respond_then_wait_for_eof() {
    let mut input = io::BufReader::new(io::stdin().lock());
    read_discover_request(&mut input);
    write_success_response();

    let mut remaining = Vec::new();
    input
        .read_to_end(&mut remaining)
        .expect("STDIN should reach EOF");
    assert!(remaining.is_empty(), "only one request is permitted");
}

fn read_one_discover_request() {
    let mut input = io::BufReader::new(io::stdin().lock());
    read_discover_request(&mut input);
}

fn read_discover_request(input: &mut impl BufRead) {
    let mut request = String::new();
    let read = input
        .read_line(&mut request)
        .expect("the discovery request should be readable");
    assert!(read > 0, "the discovery request should not be empty");
    assert_eq!(request.bytes().filter(|byte| *byte == b'\n').count(), 1);

    let value: Value = serde_json::from_str(&request).expect("the request should be JSON");
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["id"], 1);
    assert_eq!(value["method"], "server/discover");
    assert_eq!(
        value["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"],
        "2026-07-28"
    );
    assert!(value["params"]["_meta"]["io.modelcontextprotocol/clientCapabilities"].is_object());
    assert!(!request.contains("tools/call"));
    assert!(!request.contains("initialize"));
}

fn write_success_response() {
    let mut stdout = io::stdout().lock();
    writeln!(
        stdout,
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{\"protocolVersion\":\"2026-07-28\",\"capabilities\":{{}}}}}}"
    )
    .expect("the discovery response should be writable");
    stdout.flush().expect("STDOUT should flush");
}

fn write_notification(output: &mut impl Write, total_bytes: usize) {
    const PREFIX: &[u8] =
        b"{\"jsonrpc\":\"2.0\",\"method\":\"synthetic/progress\",\"params\":{\"padding\":\"";
    const SUFFIX: &[u8] = b"\"}}\n";
    assert!(total_bytes > PREFIX.len() + SUFFIX.len());
    output
        .write_all(PREFIX)
        .expect("the notification prefix should be writable");
    write_repeated(output, b'p', total_bytes - PREFIX.len() - SUFFIX.len());
    output
        .write_all(SUFFIX)
        .expect("the notification suffix should be writable");
}

fn write_repeated(output: &mut impl Write, byte: u8, bytes: usize) {
    let chunk = [byte; 8 * 1024];
    let mut remaining = bytes;
    while remaining > 0 {
        let write = remaining.min(chunk.len());
        output
            .write_all(&chunk[..write])
            .expect("the synthetic output should be writable");
        remaining -= write;
    }
}

fn wait_forever() -> ! {
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

fn wait_with_child(mut child: Child) -> ! {
    loop {
        assert!(
            child
                .try_wait()
                .expect("the resistant descendant should remain observable")
                .is_none(),
            "the resistant descendant exited before cleanup"
        );
        thread::sleep(Duration::from_secs(60));
    }
}
