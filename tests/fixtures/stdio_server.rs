use std::env;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
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
        Some("report-single-run") => report_single_run(&remaining),
        Some("protocol-unsupported") => protocol_unsupported(),
        Some("layered-protocol-failure") => layered_protocol_failure(),
        Some("catalog-invalid") => catalog_invalid(),
        Some("catalog-blocks-schema") => catalog_blocks_schema(),
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
        Some("snapshot-correlation") => snapshot_correlation(),
        Some("snapshot-invalid-shape") => snapshot_invalid_shape(),
        Some("snapshot-started-marker") => snapshot_started_marker(&remaining),
        Some("legacy-success") => legacy_success(),
        Some("legacy-report-single-run") => legacy_report_single_run(&remaining),
        Some("legacy-ambiguous-schema") => legacy_ambiguous_schema(),
        Some("legacy-schema-external") => legacy_schema_external(),
        Some("legacy-schema-depth-limit") => legacy_schema_depth_limit(),
        Some("legacy-malformed-capability") => legacy_malformed_capability(),
        Some("legacy-mismatch") => legacy_mismatch(),
        Some("legacy-malformed") => legacy_malformed(),
        Some("legacy-timeout") => legacy_timeout(),
        Some("legacy-oversized") => legacy_oversized(),
        Some("active-success") => active_success(),
        Some("active-one-success") => active_one_success(),
        Some("active-report-single-run") => active_report_single_run(&remaining),
        Some("active-output-instance-depth") => active_output_instance_depth(),
        Some("active-output-evaluation-limit") => active_output_evaluation_limit(),
        Some("active-mismatch-continue") => active_mismatch_continue(&remaining),
        Some("active-input-required") => active_input_required(),
        Some("active-tool-rejection") => active_tool_rejection(&remaining),
        Some("active-no-calls") => active_no_calls(),
        Some("active-advertised-schema-invalid") => active_advertised_schema_invalid(),
        Some("active-advertised-output-depth") => active_advertised_output_depth(),
        Some("active-discovery-contract-invalid") => active_discovery_contract_invalid(),
        Some("active-tools-contract-invalid") => active_tools_contract_invalid(),
        Some("active-invalid-result") => active_invalid_result(),
        Some("active-tool-not-found") => active_tool_not_found(),
        Some("active-revision-limit") => active_revision_limit(),
        Some("active-catalog-finding-overflow") => active_catalog_finding_overflow(),
        Some("active-crash") => active_crash(),
        Some("active-oversize") => active_oversize(),
        Some("active-resistant-child") => active_resistant_child(&remaining),
        Some("active-started-marker") => active_started_marker(&remaining),
        Some("break-success") => break_success(&remaining),
        Some("break-report-single-run") => break_report_single_run(&remaining),
        Some("break-tool-error") => break_tool_error(&remaining),
        Some("break-impossible") => break_impossible(),
        Some("break-schema-external") => break_schema_external(),
        Some("break-oversized-input") => break_oversized_input(),
        Some("break-aggregate-input") => break_aggregate_input(),
        Some("break-generation-steps") => break_generation_steps(),
        Some("break-resistant-child") => break_resistant_child(&remaining),
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

fn report_single_run(arguments: &[OsString]) -> ExitCode {
    if !claim_single_run(arguments.first()) {
        return ExitCode::from(2);
    }
    catalog_valid()
}

fn protocol_unsupported() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    read_request(&mut input, 1, "server/discover", None);
    write_result(
        1,
        json!({
            "resultType": "complete",
            "supportedVersions": [
                "2025-11-25",
                "synthetic-private-revision-never-report-7f2c"
            ],
            "capabilities": {},
            "ttlMs": 0,
            "cacheScope": "private"
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn legacy_success() -> ExitCode {
    eprintln!("synthetic-private-legacy-stderr-never-report-7f2c");
    let mut input = io::BufReader::new(io::stdin().lock());
    let revision = read_initialize(&mut input);
    let capabilities = if revision == "2025-11-25" {
        json!({
            "tools": {"listChanged": false},
            "logging": {"synthetic": "synthetic-private-legacy-log-never-report-7f2c"},
            "completions": {"synthetic": "synthetic-private-completion-never-report-7f2c"},
            "experimental": {"synthetic": {}},
            "tasks": {
                "list": {"synthetic": "synthetic-private-task-never-report-7f2c"},
                "cancel": {},
                "requests": {"tools": {"call": {"synthetic": true}}}
            }
        })
    } else {
        json!({"tools": {"listChanged": false}})
    };
    write_result(
        1,
        json!({
            "protocolVersion": revision,
            "capabilities": capabilities,
            "serverInfo": {"name": "synthetic-legacy", "version": "1.0.0"},
            "instructions": "synthetic instructions never rendered"
        }),
    );
    read_initialized(&mut input);
    read_legacy_request(&mut input, 2, "tools/list", None);
    let input_schema = json!({
        "type": "object",
        "properties": {"query": {"type": "string"}},
        "additionalProperties": false
    });
    let output_schema = json!({
        "type": "object",
        "properties": {"ok": {"type": "boolean"}}
    });
    write_result(
        2,
        json!({
            "tools": [{
                "name": "synthetic.passive",
                "inputSchema": input_schema,
                "outputSchema": output_schema
            }],
            "nextCursor": "synthetic-private-legacy-cursor-never-report-7f2c"
        }),
    );
    read_legacy_request(
        &mut input,
        3,
        "tools/list",
        Some("synthetic-private-legacy-cursor-never-report-7f2c"),
    );
    write_result(3, json!({"tools": []}));
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn legacy_report_single_run(arguments: &[OsString]) -> ExitCode {
    if !claim_single_run(arguments.first()) {
        return ExitCode::from(2);
    }
    legacy_success()
}

fn legacy_schema_external() -> ExitCode {
    legacy_tool_schema(json!({
        "$ref": "https://synthetic.invalid/legacy-private-schema-never-report-7f2c"
    }))
}

fn legacy_schema_depth_limit() -> ExitCode {
    let mut schema = json!({"type": "string"});
    for _ in 0..65 {
        schema = json!({"not": schema});
    }
    legacy_tool_schema(schema)
}

fn legacy_tool_schema(input_schema: Value) -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    let revision = read_initialize(&mut input);
    write_result(
        1,
        json!({
            "protocolVersion": revision,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "synthetic-legacy", "version": "1.0.0"}
        }),
    );
    read_initialized(&mut input);
    read_legacy_request(&mut input, 2, "tools/list", None);
    write_result(
        2,
        json!({
            "tools": [{
                "name": "synthetic.legacy-bounded",
                "inputSchema": input_schema
            }]
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn legacy_malformed_capability() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    let revision = read_initialize(&mut input);
    write_result(
        1,
        json!({
            "protocolVersion": revision,
            "capabilities": {
                "logging": "synthetic-private-malformed-capability-never-report-7f2c"
            },
            "serverInfo": {"name": "synthetic-legacy", "version": "1.0.0"}
        }),
    );
    read_initialized(&mut input);
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn legacy_ambiguous_schema() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    let revision = read_initialize(&mut input);
    write_result(
        1,
        json!({
            "protocolVersion": revision,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "synthetic-legacy", "version": "1.0.0"}
        }),
    );
    read_initialized(&mut input);
    read_legacy_request(&mut input, 2, "tools/list", None);
    write_result(
        2,
        json!({
            "tools": [{
                "name": "synthetic.ambiguous-schema",
                "inputSchema": {"type": "object"}
            }]
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn legacy_mismatch() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    let revision = read_initialize(&mut input);
    let mismatched = if revision == "2025-11-25" {
        "2025-06-18"
    } else {
        "2025-11-25"
    };
    write_result(
        1,
        json!({
            "protocolVersion": mismatched,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "synthetic-legacy", "version": "1.0.0"}
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn legacy_malformed() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    let _ = read_initialize(&mut input);
    write_result(
        1,
        json!({
            "protocolVersion": {"private": REDACTION_SENTINEL},
            "capabilities": {},
            "serverInfo": {"name": "synthetic-legacy", "version": "1.0.0"}
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn legacy_timeout() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    let _ = read_initialize(&mut input);
    wait_forever()
}

fn legacy_oversized() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    let _ = read_initialize(&mut input);
    let mut stdout = io::stdout().lock();
    write_repeated(&mut stdout, b'x', MIB + 1);
    stdout.flush().expect("STDOUT should flush");
    wait_forever()
}

fn layered_protocol_failure() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    read_request(&mut input, 1, "server/discover", None);
    write_result(
        1,
        json!({
            "resultType": "synthetic-private-result-never-report-7f2c",
            "supportedVersions": ["2025-11-25"],
            "capabilities": {},
            "ttlMs": 0,
            "cacheScope": "private"
        }),
    );
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

fn catalog_blocks_schema() -> ExitCode {
    serve_single_catalog_value(
        "tools",
        "tools/list",
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "tools": "synthetic-private-tools-never-report-7f2c"
        }),
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

fn snapshot_correlation() -> ExitCode {
    const EXCLUDED: &str = "synthetic-snapshot-excluded-never-persist-36";

    let mut input = io::BufReader::new(io::stdin().lock());
    read_request(&mut input, 1, "server/discover", None);
    write_result(
        1,
        json!({
            "resultType": "complete",
            "supportedVersions": ["2026-07-28"],
            "capabilities": {
                "tools": {},
                "syntheticExcludedExtension": {"value": EXCLUDED}
            },
            "instructions": EXCLUDED,
            "ttlMs": 0,
            "cacheScope": "private"
        }),
    );
    read_request(&mut input, 2, "tools/list", None);
    let tools = (0..100)
        .map(|ordinal| {
            let identity = 99 - ordinal;
            let required = if ordinal == 73 {
                Value::String("synthetic_invalid_required_shape_36".to_owned())
            } else {
                Value::Array(Vec::new())
            };
            json!({
                "name": format!("synthetic.tool.{identity:03}"),
                "description": EXCLUDED,
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "synthetic_field": {
                            "type": "string",
                            "description": EXCLUDED,
                            "default": EXCLUDED
                        }
                    },
                    "required": required
                }
            })
        })
        .collect::<Vec<_>>();
    write_result(
        2,
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "syntheticExcludedMetadata": EXCLUDED,
            "tools": tools
        }),
    );
    ExitCode::SUCCESS
}

fn snapshot_invalid_shape() -> ExitCode {
    serve_single_catalog_value(
        "tools",
        "tools/list",
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "tools": [{
                "name": "synthetic.invalid-shape",
                "description": "synthetic-invalid-shape-description-never-persist-36",
                "inputSchema": 7
            }]
        }),
    )
}

fn snapshot_started_marker(arguments: &[OsString]) -> ExitCode {
    let Some(marker) = arguments.first().map(PathBuf::from) else {
        return ExitCode::from(2);
    };
    fs::write(marker, b"started").expect("the synthetic start marker should be writable");
    catalog_valid()
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

fn active_success() -> ExitCode {
    assert_eq!(
        env::var("ACTIVE_TARGET_SECRET").ok().as_deref(),
        Some(REDACTION_SENTINEL),
        "the explicitly allowed target environment value should be present"
    );
    assert!(
        env::var_os("MCP_DOCTOR_UNLISTED_SECRET").is_none(),
        "an unlisted target environment value leaked"
    );
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin(&mut input);

    let first = read_active_call(&mut input, 3);
    assert_active_arguments(&first, 0, true);
    write_result(
        3,
        json!({
            "resultType": "complete",
            "content": [{"type": "text", "text": REDACTION_SENTINEL}],
            "structuredContent": {"ok": true},
            "isError": false
        }),
    );

    let second = read_active_call(&mut input, 4);
    assert_active_arguments(&second, 1, true);
    write_result(
        4,
        json!({
            "resultType": "complete",
            "content": [{"type": "text", "text": REDACTION_SENTINEL}],
            "structuredContent": {"ok": false},
            "isError": true
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_one_success() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin(&mut input);
    let request = read_active_call(&mut input, 3);
    assert_active_arguments(&request, 0, false);
    write_result(
        3,
        json!({
            "resultType": "complete",
            "content": [],
            "structuredContent": {"ok": true},
            "isError": false
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_report_single_run(arguments: &[OsString]) -> ExitCode {
    if !claim_single_run(arguments.first()) {
        return ExitCode::from(2);
    }
    active_one_success()
}

fn active_output_instance_depth() -> ExitCode {
    let mut structured = json!({"value": true});
    for _ in 0..70 {
        structured = json!({"nested": structured});
    }
    active_single_structured_result(structured)
}

fn active_output_evaluation_limit() -> ExitCode {
    active_single_structured_result(json!({"values": vec![Value::Null; 100_001]}))
}

fn active_single_structured_result(structured: Value) -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin(&mut input);
    let request = read_active_call(&mut input, 3);
    assert_active_arguments(&request, 0, false);
    write_result(
        3,
        json!({
            "resultType": "complete",
            "content": [],
            "structuredContent": structured,
            "isError": false
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_mismatch_continue(arguments: &[OsString]) -> ExitCode {
    let Some(marker) = arguments.first().map(PathBuf::from) else {
        return ExitCode::from(2);
    };
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin(&mut input);

    let first = read_active_call(&mut input, 3);
    assert_active_arguments(&first, 0, false);
    write_result(
        3,
        json!({
            "resultType": "complete",
            "content": [],
            "structuredContent": {"ok": true},
            "isError": false
        }),
    );

    let second = read_active_call(&mut input, 4);
    assert_active_arguments(&second, 1, false);
    write_result(
        4,
        json!({
            "resultType": "complete",
            "content": [],
            "structuredContent": {"ok": REDACTION_SENTINEL},
            "isError": false
        }),
    );

    let third = read_active_call(&mut input, 5);
    assert_active_arguments(&third, 2, false);
    fs::write(marker, b"third reviewed case called")
        .expect("the continuation marker should be writable");
    write_result(
        5,
        json!({
            "resultType": "complete",
            "content": [],
            "structuredContent": {"ok": true},
            "isError": false
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_input_required() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin(&mut input);
    let first = read_active_call(&mut input, 3);
    assert_active_arguments(&first, 0, false);
    write_result(
        3,
        json!({
            "resultType": "input_required",
            "inputRequests": {
                "synthetic": {
                    "type": "elicitation",
                    "message": REDACTION_SENTINEL,
                    "schema": {"type": "boolean"}
                }
            },
            "requestState": REDACTION_SENTINEL
        }),
    );

    let second = read_active_call(&mut input, 4);
    assert_active_arguments(&second, 1, false);
    assert!(
        second["params"].get("inputResponses").is_none()
            && second["params"].get("requestState").is_none(),
        "mcp-doctor must not continue the input-required round"
    );
    write_result(
        4,
        json!({
            "resultType": "complete",
            "content": [],
            "structuredContent": {"ok": true},
            "isError": false
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_crash() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin(&mut input);
    let request = read_active_call(&mut input, 3);
    assert_active_arguments(&request, 0, false);
    ExitCode::from(7)
}

fn active_tool_rejection(arguments: &[OsString]) -> ExitCode {
    let Some(marker) = arguments.first().map(PathBuf::from) else {
        return ExitCode::from(2);
    };
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin(&mut input);
    let first = read_active_call(&mut input, 3);
    assert_active_arguments(&first, 0, false);
    write_error(3);
    let second = read_active_call(&mut input, 4);
    assert_active_arguments(&second, 1, false);
    fs::write(marker, b"later reviewed case called")
        .expect("the rejection continuation marker should be writable");
    write_result(
        4,
        json!({
            "resultType": "complete",
            "content": [],
            "structuredContent": {"ok": true},
            "isError": false
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_no_calls() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin(&mut input);
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_advertised_schema_invalid() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    read_request(&mut input, 1, "server/discover", None);
    write_discovery_response(json!({"tools": {}}));
    read_request(&mut input, 2, "tools/list", None);
    write_result(
        2,
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "tools": [{
                "name": "synthetic.reviewed",
                "inputSchema": {"type": "object"},
                "outputSchema": {
                    "type": "object",
                    "properties": {
                        "value": {"$ref": "https://invalid.example/secret-schema"}
                    }
                }
            }]
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_advertised_output_depth() -> ExitCode {
    let mut nested = json!({"type": "boolean"});
    for _ in 0..70 {
        nested = json!({"not": nested});
    }
    let mut input = io::BufReader::new(io::stdin().lock());
    read_request(&mut input, 1, "server/discover", None);
    write_discovery_response(json!({"tools": {}}));
    read_request(&mut input, 2, "tools/list", None);
    write_result(
        2,
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "tools": [{
                "name": "synthetic.reviewed",
                "inputSchema": {"type": "object"},
                "outputSchema": nested
            }]
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_discovery_contract_invalid() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    read_request(&mut input, 1, "server/discover", None);
    write_result(
        1,
        json!({
            "resultType": "complete",
            "supportedVersions": ["2026-07-28"],
            "capabilities": {"tools": {"listChanged": REDACTION_SENTINEL}},
            "ttlMs": 0,
            "cacheScope": "private"
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_tools_contract_invalid() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    read_request(&mut input, 1, "server/discover", None);
    write_discovery_response(json!({"tools": {}}));
    read_request(&mut input, 2, "tools/list", None);
    write_result(
        2,
        json!({
            "resultType": "complete",
            "ttlMs": -1,
            "cacheScope": REDACTION_SENTINEL,
            "tools": [{
                "name": "synthetic.reviewed",
                "inputSchema": {"type": "object"}
            }]
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_invalid_result() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin(&mut input);
    let request = read_active_call(&mut input, 3);
    assert_active_arguments(&request, 0, false);
    write_result(
        3,
        json!({
            "resultType": "complete",
            "structuredContent": {"ok": true},
            "isError": false
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_tool_not_found() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    read_request(&mut input, 1, "server/discover", None);
    write_discovery_response(json!({"tools": {}}));
    read_request(&mut input, 2, "tools/list", None);
    write_result(
        2,
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "tools": [{
                "name": "synthetic.other",
                "inputSchema": {"type": "object"}
            }]
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_revision_limit() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    read_request(&mut input, 1, "server/discover", None);
    let mut versions = vec![Value::String("2026-07-28".to_owned())];
    versions.extend(vec![
        Value::String("synthetic-private-revision".to_owned());
        32
    ]);
    write_result(
        1,
        json!({
            "resultType": "complete",
            "supportedVersions": versions,
            "capabilities": {"tools": {}},
            "ttlMs": 0,
            "cacheScope": "private"
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_catalog_finding_overflow() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    read_request(&mut input, 1, "server/discover", None);
    write_discovery_response(json!({"tools": {}}));
    read_request(&mut input, 2, "tools/list", None);
    write_result(
        2,
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "tools": vec![Value::Null; 300]
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_oversize() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin(&mut input);
    let request = read_active_call(&mut input, 3);
    assert_active_arguments(&request, 0, false);
    let mut stdout = io::stdout().lock();
    write_repeated(&mut stdout, b'x', MIB + 1);
    stdout.flush().expect("STDOUT should flush");
    wait_forever()
}

fn active_resistant_child(arguments: &[OsString]) -> ExitCode {
    let Some(marker) = arguments.first().map(PathBuf::from) else {
        return ExitCode::from(2);
    };
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin(&mut input);
    let request = read_active_call(&mut input, 3);
    assert_active_arguments(&request, 0, false);
    let descendant =
        Command::new(env::current_exe().expect("the fixture path should be available"))
            .arg("descendant")
            .arg(marker)
            .spawn()
            .expect("the resistant active descendant should start");
    write_result(
        3,
        json!({
            "resultType": "complete",
            "content": [],
            "structuredContent": {"ok": true},
            "isError": false
        }),
    );
    wait_with_child(descendant)
}

fn active_started_marker(arguments: &[OsString]) -> ExitCode {
    let Some(marker) = arguments.first().map(PathBuf::from) else {
        return ExitCode::from(2);
    };
    fs::write(marker, b"target started unexpectedly")
        .expect("the target-start marker should be writable");
    wait_forever()
}

fn break_success(arguments: &[OsString]) -> ExitCode {
    let Some(marker) = arguments.first().map(PathBuf::from) else {
        return ExitCode::from(2);
    };
    let Some(case_count) = arguments
        .get(1)
        .and_then(|value| value.to_str())
        .and_then(|value| value.parse::<usize>().ok())
    else {
        return ExitCode::from(2);
    };
    let mut input = io::BufReader::new(io::stdin().lock());
    generated_begin(&mut input, generated_boundary_schema());
    let mut observed = Vec::with_capacity(case_count);
    for index in 0..case_count {
        let request = read_generated_call(&mut input, i64::try_from(index).unwrap_or(i64::MAX) + 3);
        let arguments = request["params"]["arguments"]
            .as_object()
            .expect("generated arguments should remain an object");
        assert!(
            arguments
                .get("synthetic_private_query_never_report_7f2c")
                .and_then(Value::as_str)
                .is_some_and(|value| (1..=8).contains(&value.len()))
        );
        assert!(
            arguments
                .get("synthetic_private_limit_never_report_7f2c")
                .and_then(Value::as_i64)
                .is_some_and(|value| (1..=5).contains(&value))
        );
        if let Some(flags) = arguments.get("synthetic_private_flags_never_report_7f2c") {
            let flags = flags
                .as_array()
                .expect("generated flags should be an array");
            assert!((1..=2).contains(&flags.len()));
            assert!(flags.iter().all(Value::is_boolean));
        }
        assert!(arguments.keys().all(|key| matches!(
            key.as_str(),
            "synthetic_private_query_never_report_7f2c"
                | "synthetic_private_limit_never_report_7f2c"
                | "synthetic_private_flags_never_report_7f2c"
        )));
        observed.push(Value::Object(arguments.clone()));
        write_result(
            i64::try_from(index).unwrap_or(i64::MAX) + 3,
            json!({
                "resultType": "complete",
                "content": [{"type": "text", "text": REDACTION_SENTINEL}],
                "structuredContent": {"ok": true},
                "isError": false
            }),
        );
    }
    assert_eof(&mut input);
    fs::write(
        marker,
        serde_json::to_vec(&observed).expect("generated observations should serialize"),
    )
    .expect("the generated observation marker should be writable");
    ExitCode::SUCCESS
}

fn break_report_single_run(arguments: &[OsString]) -> ExitCode {
    if !claim_single_run(arguments.first()) {
        return ExitCode::from(2);
    }
    break_success(arguments.get(1..).unwrap_or_default())
}

fn claim_single_run(marker: Option<&OsString>) -> bool {
    let Some(marker) = marker.map(PathBuf::from) else {
        return false;
    };
    let Ok(mut file) = OpenOptions::new().write(true).create_new(true).open(marker) else {
        return false;
    };
    file.write_all(b"one target run")
        .and_then(|()| file.sync_all())
        .is_ok()
}

fn break_tool_error(arguments: &[OsString]) -> ExitCode {
    let Some(marker) = arguments.first().map(PathBuf::from) else {
        return ExitCode::from(2);
    };
    let mut input = io::BufReader::new(io::stdin().lock());
    generated_begin(&mut input, generated_boundary_schema());
    let first = read_generated_call(&mut input, 3);
    assert!(first["params"]["arguments"].is_object());
    write_result(
        3,
        json!({
            "resultType": "complete",
            "content": [{"type": "text", "text": REDACTION_SENTINEL}],
            "isError": true
        }),
    );
    let second = read_generated_call(&mut input, 4);
    assert!(second["params"]["arguments"].is_object());
    fs::write(marker, b"later generated case called")
        .expect("the generated continuation marker should be writable");
    write_result(
        4,
        json!({
            "resultType": "complete",
            "content": [],
            "structuredContent": {"ok": true},
            "isError": false
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn break_impossible() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    generated_begin(&mut input, json!({"type": "object", "not": {}}));
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn break_schema_external() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    generated_begin(
        &mut input,
        json!({"$ref": "https://synthetic.invalid/private-schema-never-report"}),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn break_oversized_input() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    generated_begin(
        &mut input,
        json!({
            "type": "object",
            "properties": {
                "synthetic_private_value_never_report": {
                    "type": "string",
                    "minLength": MIB
                }
            },
            "required": ["synthetic_private_value_never_report"]
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn break_aggregate_input() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    generated_begin(
        &mut input,
        json!({
            "type": "object",
            "properties": {
                "synthetic_private_value_never_report": {
                    "type": "string",
                    "minLength": 100_000,
                    "maxLength": 100_000
                }
            },
            "required": ["synthetic_private_value_never_report"],
            "additionalProperties": false
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn break_generation_steps() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    generated_begin(
        &mut input,
        json!({
            "type": "object",
            "properties": {
                "synthetic_private_values_never_report": {
                    "type": "array",
                    "items": {"type": "null"},
                    "minItems": 100_001
                }
            },
            "required": ["synthetic_private_values_never_report"],
            "additionalProperties": false
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn break_resistant_child(arguments: &[OsString]) -> ExitCode {
    let Some(marker) = arguments.first().map(PathBuf::from) else {
        return ExitCode::from(2);
    };
    let mut input = io::BufReader::new(io::stdin().lock());
    generated_begin(&mut input, generated_boundary_schema());
    let request = read_generated_call(&mut input, 3);
    assert!(request["params"]["arguments"].is_object());
    let descendant =
        Command::new(env::current_exe().expect("the fixture path should be available"))
            .arg("descendant")
            .arg(marker)
            .spawn()
            .expect("the resistant generated descendant should start");
    write_result(
        3,
        json!({
            "resultType": "complete",
            "content": [],
            "structuredContent": {"ok": true},
            "isError": false
        }),
    );
    wait_with_child(descendant)
}

fn generated_boundary_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "synthetic_private_query_never_report_7f2c": {"type": "string", "minLength": 1, "maxLength": 8},
            "synthetic_private_limit_never_report_7f2c": {"type": "integer", "minimum": 1, "maximum": 5},
            "synthetic_private_flags_never_report_7f2c": {
                "type": "array",
                "items": {"type": "boolean"},
                "minItems": 1,
                "maxItems": 2
            }
        },
        "required": [
            "synthetic_private_query_never_report_7f2c",
            "synthetic_private_limit_never_report_7f2c"
        ],
        "additionalProperties": false
    })
}

fn generated_begin(input: &mut impl BufRead, input_schema: Value) {
    read_request(input, 1, "server/discover", None);
    write_discovery_response(json!({"tools": {}}));
    read_request(input, 2, "tools/list", None);
    write_result(
        2,
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "tools": [{
                "name": "synthetic.generated",
                "annotations": {
                    "readOnlyHint": false,
                    "destructiveHint": true
                },
                "inputSchema": input_schema,
                "outputSchema": {
                    "type": "object",
                    "properties": {"ok": {"type": "boolean"}},
                    "required": ["ok"],
                    "additionalProperties": false
                }
            }]
        }),
    );
}

fn read_generated_call(input: &mut impl BufRead, expected_id: i64) -> Value {
    let mut request = String::new();
    let read = input
        .read_line(&mut request)
        .expect("the generated request should be readable");
    assert!(read > 0, "the generated request should not be empty");
    let value: Value =
        serde_json::from_str(&request).expect("the generated request should be JSON");
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["id"], expected_id);
    assert_eq!(value["method"], "tools/call");
    assert_eq!(value["params"]["name"], "synthetic.generated");
    assert_eq!(
        value["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"],
        "2026-07-28"
    );
    assert!(!request.contains("initialize"));
    value
}

fn active_begin(input: &mut impl BufRead) {
    read_request(input, 1, "server/discover", None);
    write_discovery_response(json!({"tools": {}}));
    read_request(input, 2, "tools/list", None);
    write_result(
        2,
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "tools": [{
                "name": "synthetic.reviewed",
                "annotations": {
                    "readOnlyHint": false,
                    "destructiveHint": true
                },
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "sequence": {"type": "integer"},
                        "secret": {"type": "string"}
                    },
                    "required": ["sequence"],
                    "additionalProperties": false
                },
                "outputSchema": {
                    "type": "object",
                    "properties": {"ok": {"type": "boolean"}},
                    "required": ["ok"],
                    "additionalProperties": false
                }
            }]
        }),
    );
}

fn read_active_call(input: &mut impl BufRead, expected_id: i64) -> Value {
    let mut request = String::new();
    let read = input
        .read_line(&mut request)
        .expect("the active request should be readable");
    assert!(read > 0, "the active request should not be empty");
    let value: Value = serde_json::from_str(&request).expect("the active request should be JSON");
    assert!(
        value["jsonrpc"] == "2.0" && value["id"] == expected_id && value["method"] == "tools/call",
        "the expected ordered tools/call request should be sent"
    );
    assert!(
        value["params"]["name"] == "synthetic.reviewed",
        "only the exactly reviewed tool should be called"
    );
    assert_eq!(
        value["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"],
        "2026-07-28"
    );
    assert!(!request.contains("initialize"));
    value
}

fn assert_active_arguments(request: &Value, sequence: i64, secret: bool) {
    let arguments = &request["params"]["arguments"];
    assert!(
        arguments["sequence"].as_i64() == Some(sequence),
        "reviewed cases must preserve their declared order"
    );
    if secret {
        assert!(
            arguments["secret"].as_str() == Some(REDACTION_SENTINEL),
            "the argument secret should resolve into its exact null placeholder"
        );
    } else {
        assert!(
            arguments.get("secret").is_none(),
            "undeclared argument data must not be injected"
        );
    }
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

fn read_initialize(input: &mut impl BufRead) -> String {
    let mut request = String::new();
    let read = input
        .read_line(&mut request)
        .expect("the initialize request should be readable");
    assert!(read > 0, "the initialize request should not be empty");
    let value: Value = serde_json::from_str(&request).expect("initialize should be JSON");
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["id"], 1);
    assert_eq!(value["method"], "initialize");
    assert!(value["params"]["capabilities"].is_object());
    assert_eq!(value["params"]["clientInfo"]["name"], "mcp-doctor");
    let revision = value["params"]["protocolVersion"]
        .as_str()
        .expect("initialize must select a string revision");
    assert!(matches!(revision, "2025-11-25" | "2025-06-18"));
    assert!(!request.contains("server/discover"));
    revision.to_owned()
}

fn read_initialized(input: &mut impl BufRead) {
    let mut request = String::new();
    let read = input
        .read_line(&mut request)
        .expect("the initialized notification should be readable");
    assert!(read > 0, "the initialized notification should not be empty");
    let value: Value = serde_json::from_str(&request).expect("initialized should be JSON");
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["method"], "notifications/initialized");
    assert!(value.get("id").is_none());
    assert!(value.get("params").is_none());
}

fn read_legacy_request(
    input: &mut impl BufRead,
    expected_id: i64,
    expected_method: &str,
    expected_cursor: Option<&str>,
) {
    let mut request = String::new();
    let read = input
        .read_line(&mut request)
        .expect("the legacy request should be readable");
    assert!(read > 0, "the legacy request should not be empty");
    let value: Value = serde_json::from_str(&request).expect("legacy request should be JSON");
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["id"], expected_id);
    assert_eq!(value["method"], expected_method);
    assert_eq!(
        value["params"].get("cursor").and_then(Value::as_str),
        expected_cursor
    );
    assert!(value["params"].get("_meta").is_none());
    assert!(!request.contains("tools/call"));
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

fn write_error(id: i64) {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(
        &mut stdout,
        &json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32000,
                "message": REDACTION_SENTINEL,
                "data": {"secret": REDACTION_SENTINEL}
            }
        }),
    )
    .expect("the error response should be writable");
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
