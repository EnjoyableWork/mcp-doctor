use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, ExitCode};
use std::thread;
use std::time::Duration;

use serde_json::Value;
use serde_json::json;

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
        Some("catalog-valid") => catalog_valid(),
        Some("catalog-invalid") => catalog_invalid(),
        Some("catalog-invalid-resources") => catalog_invalid_resources(),
        Some("catalog-duplicate") => catalog_duplicate(),
        Some("catalog-repeated-cursor") => catalog_repeated_cursor(),
        Some("schema-invalid") => schema_invalid(),
        Some("schema-external") => schema_external(&remaining),
        Some("schema-depth-limit") => schema_depth_limit(),
        Some("schema-node-limit") => schema_node_limit(),
        Some("schema-ref-depth-limit") => schema_ref_depth_limit(),
        Some("schema-evaluation-limit") => schema_evaluation_limit(),
        Some("schema-error-limit") => schema_error_limit(),
        Some("catalog-item-limit") => catalog_item_limit(),
        Some("report-finding-limit") => report_finding_limit(),
        Some("report-finding-exact") => report_finding_exact(),
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

fn catalog_valid() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    read_request(&mut input, 1, "server/discover", None);
    write_discovery_response(json!({
        "tools": {},
        "prompts": {},
        "resources": {}
    }));

    read_request(&mut input, 2, "tools/list", None);
    write_fixture_result(2, include_str!("catalogs/valid-tools-page-1.json"));
    read_request(
        &mut input,
        3,
        "tools/list",
        Some("synthetic-private-cursor-never-report-7f2c"),
    );
    write_fixture_result(3, include_str!("catalogs/valid-tools-page-2.json"));
    read_request(&mut input, 4, "prompts/list", None);
    write_fixture_result(4, include_str!("catalogs/valid-prompts.json"));
    read_request(&mut input, 5, "resources/list", None);
    write_fixture_result(5, include_str!("catalogs/valid-resources.json"));
    read_request(&mut input, 6, "resources/templates/list", None);
    write_fixture_result(6, include_str!("catalogs/valid-resource-templates.json"));
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn catalog_invalid() -> ExitCode {
    serve_single_catalog(
        "prompts",
        "prompts/list",
        include_str!("catalogs/invalid-catalog.json"),
    )
}

fn catalog_invalid_resources() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    read_request(&mut input, 1, "server/discover", None);
    write_discovery_response(json!({"resources": {}}));
    read_request(&mut input, 2, "resources/list", None);
    write_fixture_result(2, include_str!("catalogs/invalid-resources.json"));
    read_request(&mut input, 3, "resources/templates/list", None);
    write_fixture_result(3, include_str!("catalogs/invalid-resource-templates.json"));
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn catalog_duplicate() -> ExitCode {
    serve_single_catalog(
        "prompts",
        "prompts/list",
        include_str!("catalogs/duplicate-catalog.json"),
    )
}

fn catalog_repeated_cursor() -> ExitCode {
    let cursor = "synthetic-private-repeated-cursor-never-report-7f2c";
    let mut input = io::BufReader::new(io::stdin().lock());
    read_request(&mut input, 1, "server/discover", None);
    write_discovery_response(json!({"prompts": {}}));
    read_request(&mut input, 2, "prompts/list", None);
    write_result(
        2,
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "prompts": [],
            "nextCursor": cursor
        }),
    );
    read_request(&mut input, 3, "prompts/list", Some(cursor));
    write_result(
        3,
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "prompts": [],
            "nextCursor": cursor
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn schema_invalid() -> ExitCode {
    serve_single_catalog(
        "tools",
        "tools/list",
        include_str!("catalogs/invalid-schemas.json"),
    )
}

fn schema_external(arguments: &[OsString]) -> ExitCode {
    let Some(reference) = arguments.first().and_then(|value| value.to_str()) else {
        return ExitCode::from(2);
    };
    let result = json!({
        "resultType": "complete",
        "ttlMs": 0,
        "cacheScope": "private",
        "tools": [{
            "name": "synthetic.external",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "synthetic-private-property-never-report-7f2c": {
                        "$ref": reference
                    }
                }
            }
        }]
    });
    serve_single_catalog_value("tools", "tools/list", result)
}

fn schema_depth_limit() -> ExitCode {
    let mut nested = json!({"type": "string"});
    for _ in 0..35 {
        nested = json!({"allOf": [nested]});
    }
    let result = json!({
        "resultType": "complete",
        "ttlMs": 0,
        "cacheScope": "private",
        "tools": [{
            "name": "synthetic.deep",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "synthetic-private-property-never-report-7f2c": nested
                }
            }
        }]
    });
    serve_single_catalog_value("tools", "tools/list", result)
}

fn schema_node_limit() -> ExitCode {
    let values = vec![Value::Null; 100_001];
    let result = single_tool_result(json!({
        "type": "object",
        "enum": values
    }));
    serve_single_catalog_value("tools", "tools/list", result)
}

fn schema_ref_depth_limit() -> ExitCode {
    let mut definitions = serde_json::Map::new();
    for index in 0..34 {
        let value = if index == 33 {
            json!({"type": "string"})
        } else {
            json!({"$ref": format!("#/$defs/node{}", index + 1)})
        };
        definitions.insert(format!("node{index}"), value);
    }
    let result = single_tool_result(json!({
        "type": "object",
        "$defs": definitions,
        "properties": {
            "synthetic-private-property-never-report-7f2c": {
                "$ref": "#/$defs/node0"
            }
        }
    }));
    serve_single_catalog_value("tools", "tools/list", result)
}

fn schema_evaluation_limit() -> ExitCode {
    let mut properties = serde_json::Map::new();
    for index in 0..25_000 {
        properties.insert(format!("synthetic{index}"), json!({"$ref": "#"}));
    }
    let result = single_tool_result(json!({
        "type": "object",
        "properties": properties
    }));
    serve_single_catalog_value("tools", "tools/list", result)
}

fn schema_error_limit() -> ExitCode {
    let invalid = (0..101)
        .map(|_| json!({"type": "synthetic-secret-type-never-report-7f2c"}))
        .collect::<Vec<_>>();
    let result = single_tool_result(json!({
        "type": "object",
        "allOf": invalid
    }));
    serve_single_catalog_value("tools", "tools/list", result)
}

fn single_tool_result(input_schema: Value) -> Value {
    json!({
        "resultType": "complete",
        "ttlMs": 0,
        "cacheScope": "private",
        "tools": [{
            "name": "synthetic.bounded",
            "inputSchema": input_schema
        }]
    })
}

fn catalog_item_limit() -> ExitCode {
    let prompts = (0..10_001)
        .map(|index| json!({"name": format!("synthetic-{index}")}))
        .collect::<Vec<_>>();
    let result = json!({
        "resultType": "complete",
        "ttlMs": 0,
        "cacheScope": "private",
        "prompts": prompts
    });
    serve_single_catalog_value("prompts", "prompts/list", result)
}

fn report_finding_limit() -> ExitCode {
    report_finding_count(300)
}

fn report_finding_exact() -> ExitCode {
    // One revision-confirmed finding plus 255 catalog findings reaches the
    // report maximum exactly without exceeding it.
    report_finding_count(255)
}

fn report_finding_count(count: usize) -> ExitCode {
    let result = json!({
        "resultType": "complete",
        "ttlMs": 0,
        "cacheScope": "private",
        "prompts": vec![Value::Null; count]
    });
    serve_single_catalog_value("prompts", "prompts/list", result)
}

fn serve_single_catalog(capability: &str, method: &str, result: &str) -> ExitCode {
    let result: Value = serde_json::from_str(result).expect("catalog fixture should be valid JSON");
    serve_single_catalog_value(capability, method, result)
}

fn serve_single_catalog_value(capability: &str, method: &str, result: Value) -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    read_request(&mut input, 1, "server/discover", None);
    write_discovery_response(json!({capability: {}}));
    read_request(&mut input, 2, method, None);
    write_result(2, result);
    assert_eof(&mut input);
    ExitCode::SUCCESS
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
    read_request(input, 1, "server/discover", None);
}

fn read_request(
    input: &mut impl BufRead,
    expected_id: i64,
    expected_method: &str,
    expected_cursor: Option<&str>,
) {
    let mut request = String::new();
    let read = input
        .read_line(&mut request)
        .expect("the discovery request should be readable");
    assert!(read > 0, "the discovery request should not be empty");
    assert_eq!(request.bytes().filter(|byte| *byte == b'\n').count(), 1);

    let value: Value = serde_json::from_str(&request).expect("the request should be JSON");
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["id"], expected_id);
    assert_eq!(value["method"], expected_method);
    assert_eq!(
        value["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"],
        "2026-07-28"
    );
    assert!(value["params"]["_meta"]["io.modelcontextprotocol/clientCapabilities"].is_object());
    assert_eq!(
        value["params"].get("cursor").and_then(Value::as_str),
        expected_cursor
    );
    assert!(!request.contains("tools/call"));
    assert!(!request.contains("initialize"));
}

fn write_success_response() {
    write_discovery_response(json!({}));
}

fn write_discovery_response(capabilities: Value) {
    write_result(
        1,
        json!({
            "resultType": "complete",
            "supportedVersions": ["2026-07-28"],
            "capabilities": capabilities,
            "ttlMs": 0,
            "cacheScope": "private"
        }),
    );
}

fn write_fixture_result(id: i64, fixture: &str) {
    let result: Value =
        serde_json::from_str(fixture).expect("catalog fixture should be valid JSON");
    write_result(id, result);
}

fn write_result(id: i64, result: Value) {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(
        &mut stdout,
        &json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result,
        }),
    )
    .expect("the response should be writable");
    stdout.write_all(b"\n").expect("the frame should terminate");
    stdout.flush().expect("STDOUT should flush");
}

fn assert_eof(input: &mut impl Read) {
    let mut remaining = Vec::new();
    input
        .read_to_end(&mut remaining)
        .expect("STDIN should reach EOF");
    assert!(remaining.is_empty(), "no active request is permitted");
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
