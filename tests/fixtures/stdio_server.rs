use std::env;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, ExitCode, Stdio};
use std::thread;

use serde_json::Value;
use serde_json::json;

const MIB: usize = 1024 * 1024;
const REDACTION_SENTINEL: &str = "synthetic-secret-payload-7f2c";
const DRAFT_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";
const DESCENDANT_READY: &[u8] = b"descendant-ready\n";

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
        Some("tool-description-quality") => tool_description_quality(),
        Some("tool-description-non-string") => tool_description_non_string(),
        Some("tool-description-finding-limit") => tool_description_finding_limit(),
        Some("schema-invalid") => schema_invalid(),
        Some("schema-unsupported-pattern") => schema_unsupported_pattern(),
        Some("schema-external") => schema_external(&remaining),
        Some("schema-depth-limit") => schema_depth_limit(),
        Some("schema-node-limit") => schema_node_limit(),
        Some("schema-ref-depth-limit") => schema_ref_depth_limit(),
        Some("schema-evaluation-limit") => schema_evaluation_limit(),
        Some("schema-validator-work-limit") => schema_validator_work_limit(),
        Some("schema-error-limit") => schema_error_limit(),
        Some("catalog-item-limit") => catalog_item_limit(),
        Some("report-finding-limit") => report_finding_limit(),
        Some("report-finding-exact") => report_finding_exact(),
        Some("snapshot-correlation") => snapshot_correlation(),
        Some("snapshot-invalid-shape") => snapshot_invalid_shape(),
        Some("snapshot-started-marker") => snapshot_started_marker(&remaining),
        Some("legacy-success") => legacy_success(),
        Some("legacy-tool-description-quality") => legacy_tool_description_quality(),
        Some("legacy-report-single-run") => legacy_report_single_run(&remaining),
        Some("legacy-ambiguous-schema") => legacy_ambiguous_schema(),
        Some("legacy-schema-external") => legacy_schema_external(),
        Some("legacy-schema-depth-limit") => legacy_schema_depth_limit(),
        Some("legacy-malformed-capability") => legacy_malformed_capability(),
        Some("legacy-mismatch") => legacy_mismatch(),
        Some("legacy-malformed") => legacy_malformed(),
        Some("legacy-timeout") => legacy_timeout(),
        Some("legacy-oversized") => legacy_oversized(),
        Some("legacy-active-success") => legacy_active_success(),
        Some("legacy-active-revision-mismatch") => legacy_active_revision_mismatch(),
        Some("legacy-active-no-tools") => legacy_active_no_tools(),
        Some("legacy-active-task-required") => legacy_active_task_required(),
        Some("legacy-active-tool-error") => legacy_active_tool_error(),
        Some("legacy-active-invalid-result") => legacy_active_invalid_result(),
        Some("legacy-active-schema-external") => legacy_active_schema_external(),
        Some("legacy-active-url-elicitation") => legacy_active_url_elicitation(),
        Some("legacy-active-server-request") => legacy_active_server_request(),
        Some("legacy-active-unexpected-request") => legacy_active_unexpected_request(),
        Some("legacy-active-2025-06-schema") => legacy_active_2025_06_schema(&remaining),
        Some("legacy-break-success") => legacy_break_success(&remaining),
        Some("legacy-break-2025-06-missing-dialect") => legacy_break_2025_06_missing_dialect(),
        Some("active-success") => active_success(),
        Some("active-one-success") => active_one_success(),
        Some("active-report-single-run") => active_report_single_run(&remaining),
        Some("active-output-instance-depth") => active_output_instance_depth(),
        Some("active-output-evaluation-limit") => active_output_evaluation_limit(),
        Some("active-input-evaluation-limit") => active_input_evaluation_limit(),
        Some("active-input-pattern-evaluation-limit") => active_input_pattern_evaluation_limit(),
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
        Some("workflow-read-only") => workflow_read_only(),
        Some("workflow-mutation") => workflow_mutation(),
        Some("workflow-main-failure-cleanup") => workflow_main_failure_cleanup(),
        Some("workflow-schema-mismatch-cleanup") => workflow_schema_mismatch_cleanup(),
        Some("workflow-input-required-cleanup") => workflow_input_required_cleanup(),
        Some("workflow-missing-capture") => workflow_missing_capture(),
        Some("workflow-cleanup-failure") => workflow_cleanup_failure(),
        Some("workflow-call-timeout") => workflow_call_timeout(),
        Some("workflow-disconnect") => workflow_disconnect(),
        Some("break-success") => break_success(&remaining),
        Some("break-report-single-run") => break_report_single_run(&remaining),
        Some("break-tool-error") => break_tool_error(&remaining),
        Some("break-impossible") => break_impossible(),
        Some("break-schema-external") => break_schema_external(),
        Some("break-oversized-input") => break_oversized_input(),
        Some("break-shared-schema-input") => break_shared_schema_input(),
        Some("break-generation-steps") => break_generation_steps(),
        Some("break-resistant-child") => break_resistant_child(&remaining),
        Some("reject-success") => reject_success(&remaining),
        Some("reject-unsafe-success") => reject_unsafe_success(),
        Some("reject-wrong-error") => reject_wrong_error(),
        Some("reject-malformed-error") => reject_malformed_error(),
        Some("reject-wrong-id") => reject_wrong_id(),
        Some("reject-clean-exit") => reject_clean_exit(),
        Some("reject-crash") => reject_crash(),
        Some("reject-timeout") => reject_timeout(),
        Some("reject-oversize") => reject_oversize(),
        Some("reject-schema-invalid") => reject_schema_invalid(),
        Some("reject-schema-external") => reject_schema_external(),
        Some("reject-oversized-input") => reject_oversized_input(),
        Some("reject-impossible") => reject_impossible(),
        Some("reject-passive") => reject_passive(&remaining),
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

    let descendant = spawn_ready_descendant(marker, "resistant");

    write_success_response();
    wait_with_child(descendant)
}

fn descendant(arguments: &[OsString]) -> ExitCode {
    let Some(marker) = arguments.first().map(PathBuf::from) else {
        return ExitCode::from(2);
    };
    let mut readiness = OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(marker)
        .expect("the descendant readiness marker should be created");
    readiness
        .lock()
        .expect("the descendant readiness marker should be locked");
    readiness
        .write_all(DESCENDANT_READY)
        .expect("the descendant readiness marker should be writable");
    readiness
        .flush()
        .expect("the descendant readiness marker should flush");

    let mut stdout = io::stdout().lock();
    stdout
        .write_all(DESCENDANT_READY)
        .expect("the descendant readiness acknowledgement should be writable");
    stdout
        .flush()
        .expect("the descendant readiness acknowledgement should flush");
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
    eprintln!("synthetic-private-ci-stderr-never-report-7f2c");
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
                "description": "A synthetic passive tool.",
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
                "description": "A synthetic legacy bounded tool.",
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
                "description": "A synthetic ambiguous-schema tool.",
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

#[derive(Clone, Copy)]
enum LegacyActiveBehavior {
    Complete,
    ToolError,
    InvalidResult,
    TaskRequired,
    UrlElicitation,
    ServerRequest,
    UnexpectedRequest,
}

fn legacy_active_success() -> ExitCode {
    legacy_active_reviewed(LegacyActiveBehavior::Complete)
}

fn legacy_active_revision_mismatch() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    let revision = read_initialize(&mut input);
    let mismatched = if revision == "2025-11-25" {
        "2025-06-18"
    } else {
        assert_eq!(revision, "2025-06-18");
        "2025-11-25"
    };
    write_result(
        1,
        json!({
            "protocolVersion": mismatched,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "synthetic-legacy-active", "version": "1.0.0"}
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn legacy_active_no_tools() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    let revision = read_initialize(&mut input);
    assert!(matches!(revision.as_str(), "2025-11-25" | "2025-06-18"));
    write_result(
        1,
        json!({
            "protocolVersion": revision,
            "capabilities": {},
            "serverInfo": {"name": "synthetic-legacy-active", "version": "1.0.0"}
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn legacy_active_task_required() -> ExitCode {
    legacy_active_reviewed(LegacyActiveBehavior::TaskRequired)
}

fn legacy_active_tool_error() -> ExitCode {
    legacy_active_reviewed(LegacyActiveBehavior::ToolError)
}

fn legacy_active_invalid_result() -> ExitCode {
    legacy_active_reviewed(LegacyActiveBehavior::InvalidResult)
}

fn legacy_active_schema_external() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    legacy_active_begin(
        &mut input,
        "synthetic.reviewed",
        false,
        json!({
            "type": "object",
            "properties": {
                "sequence": {"$ref": "https://invalid.example/private-schema"}
            }
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn legacy_active_url_elicitation() -> ExitCode {
    legacy_active_reviewed(LegacyActiveBehavior::UrlElicitation)
}

fn legacy_active_server_request() -> ExitCode {
    legacy_active_reviewed(LegacyActiveBehavior::ServerRequest)
}

fn legacy_active_unexpected_request() -> ExitCode {
    legacy_active_reviewed(LegacyActiveBehavior::UnexpectedRequest)
}

fn legacy_active_reviewed(behavior: LegacyActiveBehavior) -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    legacy_active_begin(
        &mut input,
        "synthetic.reviewed",
        matches!(behavior, LegacyActiveBehavior::TaskRequired),
        json!({
            "type": "object",
            "properties": {"sequence": {"type": "integer"}},
            "required": ["sequence"],
            "additionalProperties": false
        }),
    );
    if matches!(behavior, LegacyActiveBehavior::TaskRequired) {
        assert_eof(&mut input);
        return ExitCode::SUCCESS;
    }
    let call = read_legacy_call(&mut input, 3, "synthetic.reviewed");
    assert_eq!(call["params"]["arguments"]["sequence"], 0);
    match behavior {
        LegacyActiveBehavior::Complete => write_result(
            3,
            json!({
                "content": [{"type": "text", "text": REDACTION_SENTINEL}],
                "structuredContent": {"ok": true},
                "isError": false
            }),
        ),
        LegacyActiveBehavior::ToolError => write_result(
            3,
            json!({
                "content": [{"type": "text", "text": REDACTION_SENTINEL}],
                "structuredContent": {"ok": true},
                "isError": true
            }),
        ),
        LegacyActiveBehavior::InvalidResult => write_result(
            3,
            json!({
                "content": {"private": REDACTION_SENTINEL},
                "structuredContent": {"ok": true},
                "isError": false
            }),
        ),
        LegacyActiveBehavior::UrlElicitation => write_json_frame(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "error": {
                "code": -32042,
                "message": REDACTION_SENTINEL,
                "data": {
                    "elicitations": [{
                        "mode": "url",
                        "elicitationId": REDACTION_SENTINEL,
                        "url": "https://synthetic.invalid/private-action?secret=synthetic-secret-payload-7f2c",
                        "message": REDACTION_SENTINEL
                    }]
                }
            }
        })),
        LegacyActiveBehavior::ServerRequest => write_json_frame(json!({
            "jsonrpc": "2.0",
            "id": "synthetic-server-request-never-report-7f2c",
            "method": "elicitation/create",
            "params": {
                "mode": "url",
                "message": REDACTION_SENTINEL,
                "elicitationId": REDACTION_SENTINEL,
                "url": "https://synthetic.invalid/private-action?secret=synthetic-secret-payload-7f2c"
            }
        })),
        LegacyActiveBehavior::UnexpectedRequest => write_json_frame(json!({
            "jsonrpc": "2.0",
            "id": "synthetic-server-request-never-report-7f2c",
            "method": "ping",
            "params": {"secret": REDACTION_SENTINEL}
        })),
        LegacyActiveBehavior::TaskRequired => unreachable!(),
    }
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn legacy_break_success(arguments: &[OsString]) -> ExitCode {
    let Some(case_count) = arguments
        .first()
        .and_then(|value| value.to_str())
        .and_then(|value| value.parse::<usize>().ok())
    else {
        return ExitCode::from(2);
    };
    let mut input = io::BufReader::new(io::stdin().lock());
    legacy_active_begin(
        &mut input,
        "synthetic.generated",
        false,
        generated_boundary_schema(),
    );
    for index in 0..case_count {
        let id = i64::try_from(index).unwrap_or(i64::MAX) + 3;
        let request = read_legacy_call(&mut input, id, "synthetic.generated");
        assert!(request["params"]["arguments"].is_object());
        write_result(
            id,
            json!({
                "content": [],
                "structuredContent": {"ok": true},
                "isError": false
            }),
        );
    }
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn legacy_active_2025_06_schema(arguments: &[OsString]) -> ExitCode {
    let Some(mode) = arguments.first().and_then(|value| value.to_str()) else {
        return ExitCode::from(2);
    };
    let mut input_schema = json!({
        "$schema": DRAFT_2020_12,
        "type": "object",
        "properties": {"sequence": {"type": "integer"}},
        "required": ["sequence"],
        "additionalProperties": false
    });
    let mut output_schema = Some(json!({
        "$schema": DRAFT_2020_12,
        "type": "object",
        "properties": {"ok": {"type": "boolean"}},
        "required": ["ok"],
        "additionalProperties": false
    }));
    match mode {
        "input-missing" => {
            input_schema
                .as_object_mut()
                .expect("the synthetic input schema is an object")
                .remove("$schema");
        }
        "input-malformed" => input_schema["$schema"] = json!([]),
        "input-wrong" => {
            input_schema["$schema"] = json!("https://synthetic.invalid/unsupported-dialect")
        }
        "input-vocabulary" => {
            input_schema["$vocabulary"] =
                json!({"https://synthetic.invalid/private-vocabulary": true});
        }
        "input-standard-vocabulary" => {
            input_schema["$vocabulary"] = json!({
                "https://json-schema.org/draft/2020-12/vocab/core": true,
                "https://json-schema.org/draft/2020-12/vocab/applicator": true,
                "https://json-schema.org/draft/2020-12/vocab/validation": true
            });
        }
        "input-external" => {
            input_schema["properties"]["sequence"] =
                json!({"$ref": "https://synthetic.invalid/private-schema"});
        }
        "input-depth" => {
            let mut nested = json!({"type": "integer"});
            for _ in 0..70 {
                nested = json!({"not": nested});
            }
            input_schema["properties"]["sequence"] = nested;
        }
        "output-omitted" => output_schema = None,
        "output-missing" => {
            output_schema
                .as_mut()
                .and_then(Value::as_object_mut)
                .expect("the synthetic output schema is an object")
                .remove("$schema");
        }
        "output-malformed" => {
            output_schema.as_mut().expect("output schema exists")["$schema"] = json!({});
        }
        "output-wrong" => {
            output_schema.as_mut().expect("output schema exists")["$schema"] =
                json!("https://synthetic.invalid/unsupported-dialect");
        }
        "output-mismatch" => {}
        _ => return ExitCode::from(2),
    }

    let mut input = io::BufReader::new(io::stdin().lock());
    legacy_active_begin_with_schemas(
        &mut input,
        "synthetic.reviewed",
        false,
        input_schema,
        output_schema,
        false,
    );
    if matches!(
        mode,
        "input-standard-vocabulary" | "output-omitted" | "output-mismatch"
    ) {
        let call = read_legacy_call(&mut input, 3, "synthetic.reviewed");
        assert_eq!(call["params"]["arguments"]["sequence"], 0);
        write_result(
            3,
            json!({
                "content": [],
                "structuredContent": if mode == "output-mismatch" {
                    json!({"ok": "synthetic-private-result-never-report-7f2c"})
                } else {
                    json!({"ok": true})
                },
                "isError": false
            }),
        );
    }
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn legacy_break_2025_06_missing_dialect() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    legacy_active_begin_with_schemas(
        &mut input,
        "synthetic.generated",
        false,
        generated_boundary_schema(),
        Some(json!({
            "$schema": DRAFT_2020_12,
            "type": "object",
            "properties": {"ok": {"type": "boolean"}},
            "required": ["ok"],
            "additionalProperties": false
        })),
        false,
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn legacy_active_begin(
    input: &mut impl BufRead,
    tool_name: &str,
    task_required: bool,
    input_schema: Value,
) {
    legacy_active_begin_with_schemas(
        input,
        tool_name,
        task_required,
        input_schema,
        Some(json!({
            "type": "object",
            "properties": {"ok": {"type": "boolean"}},
            "required": ["ok"],
            "additionalProperties": false
        })),
        true,
    );
}

fn legacy_active_begin_with_schemas(
    input: &mut impl BufRead,
    tool_name: &str,
    task_required: bool,
    mut input_schema: Value,
    mut output_schema: Option<Value>,
    add_required_dialect: bool,
) {
    let revision = read_initialize(input);
    assert!(matches!(revision.as_str(), "2025-11-25" | "2025-06-18"));
    if revision == "2025-06-18" && add_required_dialect {
        input_schema
            .as_object_mut()
            .expect("a synthetic tool input schema is an object")
            .insert(
                "$schema".to_owned(),
                Value::String(DRAFT_2020_12.to_owned()),
            );
        if let Some(output_schema) = output_schema.as_mut() {
            output_schema
                .as_object_mut()
                .expect("a synthetic tool output schema is an object")
                .insert(
                    "$schema".to_owned(),
                    Value::String(DRAFT_2020_12.to_owned()),
                );
        }
    }
    write_result(
        1,
        json!({
            "protocolVersion": revision,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "synthetic-legacy-active", "version": "1.0.0"}
        }),
    );
    read_initialized(input);
    read_legacy_request(input, 2, "tools/list", None);
    let mut tool = json!({
        "name": tool_name,
        "inputSchema": input_schema,
    });
    if let Some(output_schema) = output_schema {
        tool["outputSchema"] = output_schema;
    }
    if task_required {
        tool["execution"] = json!({"taskSupport": "required"});
    }
    write_result(2, json!({"tools": [tool]}));
}

fn read_legacy_call(input: &mut impl BufRead, expected_id: i64, tool_name: &str) -> Value {
    let mut request = String::new();
    let read = input
        .read_line(&mut request)
        .expect("the legacy active call should be readable");
    assert!(read > 0, "the legacy active call should not be empty");
    let value: Value = serde_json::from_str(&request).expect("the legacy call should be JSON");
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["id"], expected_id);
    assert_eq!(value["method"], "tools/call");
    assert_eq!(value["params"]["name"], tool_name);
    assert!(value["params"].get("_meta").is_none());
    value
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

fn tool_description_quality() -> ExitCode {
    serve_single_catalog_value(
        "tools",
        "tools/list",
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "tools": tool_description_quality_tools()
        }),
    )
}

fn legacy_tool_description_quality() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    let revision = read_initialize(&mut input);
    write_result(
        1,
        json!({
            "protocolVersion": revision,
            "capabilities": {"tools": {"listChanged": false}},
            "serverInfo": {"name": "synthetic-legacy-quality", "version": "1.0.0"}
        }),
    );
    read_initialized(&mut input);
    read_legacy_request(&mut input, 2, "tools/list", None);
    write_result(2, json!({"tools": tool_description_quality_tools()}));
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn tool_description_quality_tools() -> Vec<Value> {
    const SENTINEL: &str = "synthetic-private-tool-description-never-report-61";
    let schema = || {
        json!({
            "$schema": DRAFT_2020_12,
            "type": "object",
            "additionalProperties": false
        })
    };
    vec![
        json!({
            "name": format!("{SENTINEL}-missing"),
            "inputSchema": schema()
        }),
        json!({
            "name": format!("{SENTINEL}-empty"),
            "description": "",
            "inputSchema": schema()
        }),
        json!({
            "name": format!("{SENTINEL}-blank"),
            "description": "\u{0009}\u{000A}\u{000B}\u{000C}\u{000D}\u{0020}\u{0085}\u{00A0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200A}\u{2028}\u{2029}\u{202F}\u{205F}\u{3000}",
            "inputSchema": schema()
        }),
        json!({
            "name": format!("{SENTINEL}-usable"),
            "description": format!("Use the {SENTINEL} tool for a synthetic bounded operation."),
            "inputSchema": schema()
        }),
    ]
}

fn tool_description_non_string() -> ExitCode {
    serve_single_catalog_value(
        "tools",
        "tools/list",
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "tools": [{
                "name": "synthetic-private-non-string-description-never-report-61",
                "description": {"synthetic-private-value-never-report-61": true},
                "inputSchema": {
                    "$schema": DRAFT_2020_12,
                    "type": "object"
                }
            }]
        }),
    )
}

fn tool_description_finding_limit() -> ExitCode {
    let tools = (0..300)
        .map(|index| {
            json!({
                "name": format!("synthetic-private-quality-limit-{index}"),
                "inputSchema": {
                    "$schema": DRAFT_2020_12,
                    "type": "object"
                }
            })
        })
        .collect::<Vec<_>>();
    serve_single_catalog_value(
        "tools",
        "tools/list",
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "tools": tools
        }),
    )
}

fn schema_invalid() -> ExitCode {
    serve_single_catalog(
        "tools",
        "tools/list",
        include_str!("catalogs/invalid-schemas.json"),
    )
}

fn schema_unsupported_pattern() -> ExitCode {
    let result = single_tool_result(json!({
        "type": "object",
        "properties": {
            "synthetic-private-property-never-report-7f2c": {
                "type": "string",
                "pattern": "^(?!private)"
            }
        }
    }));
    serve_single_catalog_value("tools", "tools/list", result)
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
            "description": "A synthetic external-reference tool.",
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
            "description": "A synthetic deeply nested schema tool.",
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

fn schema_validator_work_limit() -> ExitCode {
    let properties = (0..4_096)
        .map(|index| (format!("synthetic_{index:04}"), json!({"type": "string"})))
        .collect::<serde_json::Map<_, _>>();
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
            "description": "A synthetic bounded tool.",
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

fn active_input_evaluation_limit() -> ExitCode {
    let branches = (0..64)
        .map(|value| json!({"const": value}))
        .collect::<Vec<_>>();
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin_with_input_schema(
        &mut input,
        json!({
            "type": "object",
            "properties": {
                "sequence": {
                    "type": "array",
                    "items": {"allOf": branches}
                }
            },
            "required": ["sequence"],
            "additionalProperties": false
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn active_input_pattern_evaluation_limit() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin_with_input_schema(
        &mut input,
        json!({
            "type": "object",
            "properties": {
                "sequence": {
                    "type": "string",
                    "pattern": "^(a{1000})+$"
                }
            },
            "required": ["sequence"],
            "additionalProperties": false
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
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
    let descendant = spawn_ready_descendant(marker, "resistant active");
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

fn workflow_read_only() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    workflow_begin(&mut input);
    let lookup = read_workflow_call(&mut input, 3, "synthetic.workflow.lookup");
    assert_eq!(lookup["params"]["arguments"]["query"], REDACTION_SENTINEL);
    write_workflow_result(3, json!({"resource": {"id": REDACTION_SENTINEL}}), false);

    let read = read_workflow_call(&mut input, 4, "synthetic.workflow.read");
    assert_eq!(read["params"]["arguments"]["id"], REDACTION_SENTINEL);
    assert!(read["params"]["arguments"].get("expectedVersion").is_none());
    write_workflow_result(4, json!({"value": REDACTION_SENTINEL, "version": 1}), false);
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn workflow_mutation() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    workflow_begin(&mut input);
    workflow_lookup(&mut input, 3);

    let mutate = read_workflow_call(&mut input, 4, "synthetic.workflow.mutate");
    assert_eq!(mutate["params"]["arguments"]["id"], REDACTION_SENTINEL);
    assert_eq!(mutate["params"]["arguments"]["value"], REDACTION_SENTINEL);
    write_workflow_result(4, json!({"version": 2}), false);

    let read = read_workflow_call(&mut input, 5, "synthetic.workflow.read");
    assert_eq!(read["params"]["arguments"]["id"], REDACTION_SENTINEL);
    assert_eq!(read["params"]["arguments"]["expectedVersion"], 2);
    write_workflow_result(5, json!({"value": REDACTION_SENTINEL, "version": 2}), false);

    workflow_cleanup(&mut input, 6, false);
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn workflow_main_failure_cleanup() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    workflow_begin(&mut input);
    workflow_lookup(&mut input, 3);

    let mutate = read_workflow_call(&mut input, 4, "synthetic.workflow.mutate");
    assert_eq!(mutate["params"]["arguments"]["id"], REDACTION_SENTINEL);
    write_workflow_result(4, json!({"version": 2}), true);

    workflow_cleanup(&mut input, 5, false);
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn workflow_schema_mismatch_cleanup() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    workflow_begin(&mut input);
    workflow_lookup(&mut input, 3);

    let mutate = read_workflow_call(&mut input, 4, "synthetic.workflow.mutate");
    assert_eq!(mutate["params"]["arguments"]["id"], REDACTION_SENTINEL);
    write_workflow_result(4, json!({"version": REDACTION_SENTINEL}), false);

    workflow_cleanup(&mut input, 5, false);
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn workflow_input_required_cleanup() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    workflow_begin(&mut input);
    workflow_lookup(&mut input, 3);

    let mutate = read_workflow_call(&mut input, 4, "synthetic.workflow.mutate");
    assert_eq!(mutate["params"]["arguments"]["id"], REDACTION_SENTINEL);
    write_result(
        4,
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

    workflow_cleanup(&mut input, 5, false);
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn workflow_missing_capture() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    workflow_begin(&mut input);
    let lookup = read_workflow_call(&mut input, 3, "synthetic.workflow.lookup");
    assert_eq!(lookup["params"]["arguments"]["query"], REDACTION_SENTINEL);
    write_workflow_result(3, json!({"resource": {}}), false);
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn workflow_cleanup_failure() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    workflow_begin(&mut input);
    workflow_lookup(&mut input, 3);

    let mutate = read_workflow_call(&mut input, 4, "synthetic.workflow.mutate");
    assert_eq!(mutate["params"]["arguments"]["id"], REDACTION_SENTINEL);
    write_workflow_result(4, json!({"version": 2}), false);

    workflow_cleanup(&mut input, 5, true);
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn workflow_call_timeout() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    workflow_begin(&mut input);
    let _ = read_workflow_call(&mut input, 3, "synthetic.workflow.lookup");
    wait_forever()
}

fn workflow_disconnect() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    workflow_begin(&mut input);
    let _ = read_workflow_call(&mut input, 3, "synthetic.workflow.lookup");
    ExitCode::from(7)
}

fn workflow_lookup(input: &mut impl BufRead, id: i64) {
    let lookup = read_workflow_call(input, id, "synthetic.workflow.lookup");
    assert_eq!(lookup["params"]["arguments"]["query"], REDACTION_SENTINEL);
    write_workflow_result(id, json!({"resource": {"id": REDACTION_SENTINEL}}), false);
}

fn workflow_cleanup(input: &mut impl BufRead, id: i64, fail: bool) {
    let cleanup = read_workflow_call(input, id, "synthetic.workflow.cleanup");
    assert_eq!(cleanup["params"]["arguments"]["id"], REDACTION_SENTINEL);
    write_workflow_result(id, json!({"removed": !fail}), fail);
}

fn write_workflow_result(id: i64, structured: Value, is_error: bool) {
    write_result(
        id,
        json!({
            "resultType": "complete",
            "content": [{"type": "text", "text": REDACTION_SENTINEL}],
            "structuredContent": structured,
            "isError": is_error
        }),
    );
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

fn break_shared_schema_input() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    // Compilation and validation are each below 100,000 steps in isolation,
    // but one 90,000-byte string exceeds the operation budget when those
    // phases are correctly combined. The tool must never be called.
    generated_begin(
        &mut input,
        json!({
            "type": "object",
            "properties": {
                "synthetic_private_value_never_report": {
                    "type": "string",
                    "minLength": 90_000
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
    let descendant = spawn_ready_descendant(marker, "resistant generated");
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

fn reject_success(arguments: &[OsString]) -> ExitCode {
    let Some(marker) = arguments.first().map(PathBuf::from) else {
        return ExitCode::from(2);
    };
    let mut input = io::BufReader::new(io::stdin().lock());
    reject_begin(&mut input);
    for (ordinal, id) in (3_i64..=9).enumerate() {
        let call = read_active_call(&mut input, id);
        let arguments = call["params"].get("arguments");
        match ordinal {
            0 => assert!(arguments.is_none(), "the first case omits arguments"),
            1 => assert!(arguments.is_some_and(Value::is_array)),
            2 => assert!(
                arguments
                    .and_then(Value::as_object)
                    .is_some_and(|value| !value.contains_key("sequence"))
            ),
            3 => assert!(arguments.and_then(Value::as_object).is_some_and(|value| {
                value
                    .get("sequence")
                    .is_some_and(|value| !value.is_number())
                    || value.get("secret").is_some_and(|value| !value.is_string())
            })),
            4 => assert!(
                arguments
                    .and_then(Value::as_object)
                    .is_some_and(|value| value.values().any(Value::is_null))
            ),
            5 => assert!(
                arguments
                    .and_then(Value::as_object)
                    .and_then(|value| value.get("synthetic_private_mode_never_report_7f2c"))
                    .and_then(Value::as_str)
                    .is_some_and(|value| !matches!(value, "safe" | "strict"))
            ),
            6 => {
                assert!(
                    arguments
                        .and_then(Value::as_object)
                        .is_some_and(|value| value.keys().any(|name| !matches!(
                            name.as_str(),
                            "sequence" | "secret" | "synthetic_private_mode_never_report_7f2c"
                        )))
                )
            }
            _ => unreachable!(),
        }
        write_invalid_params(id);
    }
    fs::write(marker, b"7").expect("the reject call-count marker should be writable");
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn reject_unsafe_success() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    reject_begin(&mut input);
    let _ = read_active_call(&mut input, 3);
    write_result(
        3,
        json!({
            "resultType": "input_required",
            "content": [{"type": "text", "text": REDACTION_SENTINEL}],
            "isError": true
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn reject_wrong_error() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    reject_begin(&mut input);
    let _ = read_active_call(&mut input, 3);
    write_error(3);
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn reject_malformed_error() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    reject_begin(&mut input);
    let _ = read_active_call(&mut input, 3);
    write_json_frame(json!({
        "jsonrpc": "2.0",
        "id": 3,
        "error": {"code": -32602, "message": [REDACTION_SENTINEL]}
    }));
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn reject_wrong_id() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    reject_begin(&mut input);
    let _ = read_active_call(&mut input, 3);
    write_invalid_params(4);
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn reject_clean_exit() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    reject_begin(&mut input);
    let _ = read_active_call(&mut input, 3);
    ExitCode::SUCCESS
}

fn reject_crash() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    reject_begin(&mut input);
    let _ = read_active_call(&mut input, 3);
    ExitCode::from(7)
}

fn reject_timeout() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    reject_begin(&mut input);
    let _ = read_active_call(&mut input, 3);
    wait_forever()
}

fn reject_oversize() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    reject_begin(&mut input);
    let _ = read_active_call(&mut input, 3);
    let mut stdout = io::stdout().lock();
    write_repeated(&mut stdout, b'x', MIB + 1);
    stdout.flush().expect("STDOUT should flush");
    wait_forever()
}

fn reject_schema_invalid() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin_with_input_schema(
        &mut input,
        json!({
            "type": "object",
            "properties": [REDACTION_SENTINEL]
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn reject_schema_external() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin_with_input_schema(
        &mut input,
        json!({
            "type": "object",
            "properties": {"secret": {"$ref": "https://invalid.example/private"}}
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn reject_oversized_input() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin_with_input_schema(
        &mut input,
        json!({
            "type": "object",
            "properties": {
                "synthetic_private_value_never_report_7f2c": {
                    "type": "string",
                    "minLength": MIB
                }
            },
            "required": ["synthetic_private_value_never_report_7f2c"]
        }),
    );
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn reject_impossible() -> ExitCode {
    let mut input = io::BufReader::new(io::stdin().lock());
    active_begin_with_input_schema(&mut input, json!({"type": "object", "not": {}}));
    assert_eof(&mut input);
    ExitCode::SUCCESS
}

fn reject_passive(arguments: &[OsString]) -> ExitCode {
    let Some(marker) = arguments.first().map(PathBuf::from) else {
        return ExitCode::from(2);
    };
    let mut input = io::BufReader::new(io::stdin().lock());
    reject_begin(&mut input);
    assert_eof(&mut input);
    fs::write(marker, b"0").expect("the passive reject marker should be writable");
    ExitCode::SUCCESS
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
    active_begin_with_input_schema(
        input,
        json!({
            "type": "object",
            "properties": {
                "sequence": {"type": "integer"},
                "secret": {"type": "string"}
            },
            "required": ["sequence"],
            "additionalProperties": false
        }),
    );
}

fn workflow_begin(input: &mut impl BufRead) {
    read_request(input, 1, "server/discover", None);
    write_discovery_response(json!({"tools": {}}));
    read_request(input, 2, "tools/list", None);
    write_result(
        2,
        json!({
            "resultType": "complete",
            "ttlMs": 0,
            "cacheScope": "private",
            "tools": [
                {
                    "name": "synthetic.workflow.lookup",
                    "inputSchema": {
                        "type": "object",
                        "properties": {"query": {"type": "string"}},
                        "required": ["query"],
                        "additionalProperties": false
                    },
                    "outputSchema": {
                        "type": "object",
                        "properties": {
                            "resource": {
                                "type": "object",
                                "properties": {"id": {"type": "string"}},
                                "additionalProperties": false
                            }
                        },
                        "required": ["resource"],
                        "additionalProperties": false
                    }
                },
                {
                    "name": "synthetic.workflow.mutate",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "value": {"type": "string"}
                        },
                        "required": ["id", "value"],
                        "additionalProperties": false
                    },
                    "outputSchema": {
                        "type": "object",
                        "properties": {"version": {"type": "integer"}},
                        "required": ["version"],
                        "additionalProperties": false
                    }
                },
                {
                    "name": "synthetic.workflow.read",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "expectedVersion": {"type": "integer"}
                        },
                        "required": ["id"],
                        "additionalProperties": false
                    },
                    "outputSchema": {
                        "type": "object",
                        "properties": {
                            "value": {"type": "string"},
                            "version": {"type": "integer"}
                        },
                        "required": ["value", "version"],
                        "additionalProperties": false
                    }
                },
                {
                    "name": "synthetic.workflow.cleanup",
                    "inputSchema": {
                        "type": "object",
                        "properties": {"id": {"type": "string"}},
                        "required": ["id"],
                        "additionalProperties": false
                    },
                    "outputSchema": {
                        "type": "object",
                        "properties": {"removed": {"type": "boolean"}},
                        "required": ["removed"],
                        "additionalProperties": false
                    }
                }
            ]
        }),
    );
}

fn reject_begin(input: &mut impl BufRead) {
    active_begin_with_input_schema(
        input,
        json!({
            "type": "object",
            "properties": {
                "sequence": {"type": "integer", "minimum": 1, "maximum": 5},
                "secret": {"type": "string"},
                "synthetic_private_mode_never_report_7f2c": {
                    "type": "string",
                    "enum": ["safe", "strict"]
                }
            },
            "required": ["sequence"],
            "additionalProperties": false
        }),
    );
}

fn active_begin_with_input_schema(input: &mut impl BufRead, input_schema: Value) {
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

fn read_workflow_call(input: &mut impl BufRead, expected_id: i64, expected_tool: &str) -> Value {
    let mut request = String::new();
    let read = input
        .read_line(&mut request)
        .expect("the workflow request should be readable");
    assert!(read > 0, "the workflow request should not be empty");
    let value: Value = serde_json::from_str(&request).expect("the workflow request should be JSON");
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["id"], expected_id);
    assert_eq!(value["method"], "tools/call");
    assert_eq!(value["params"]["name"], expected_tool);
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
    assert_eq!(value["params"]["capabilities"], json!({}));
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
    write_json_frame(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    }));
}

fn write_json_frame(value: Value) {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &value).expect("the response should be writable");
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

fn write_invalid_params(id: i64) {
    write_json_frame(json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": -32602,
            "message": REDACTION_SENTINEL,
            "data": {"secret": REDACTION_SENTINEL}
        }
    }));
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

fn spawn_ready_descendant(marker: PathBuf, description: &str) -> Child {
    let mut child = Command::new(env::current_exe().expect("the fixture path should be available"))
        .arg("descendant")
        .arg(marker)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("the {description} descendant should start"));
    let mut acknowledgement = vec![0_u8; DESCENDANT_READY.len()];
    child
        .stdout
        .take()
        .expect("the descendant readiness pipe should exist")
        .read_exact(&mut acknowledgement)
        .unwrap_or_else(|_| panic!("the {description} descendant should acknowledge readiness"));
    assert_eq!(acknowledgement, DESCENDANT_READY);
    child
}

fn wait_forever() -> ! {
    loop {
        thread::park();
    }
}

fn wait_with_child(mut child: Child) -> ! {
    let status = child
        .wait()
        .expect("the resistant descendant should remain observable");
    panic!("the resistant descendant exited before cleanup with {status}")
}
