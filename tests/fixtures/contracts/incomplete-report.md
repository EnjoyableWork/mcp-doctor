<!-- mcp-doctor.markdown/v1 -->
# mcp-doctor diagnostic report

| Field | Value |
| --- | --- |
| Product | `mcp-doctor 0.4.2` |
| Report contract | `mcp-doctor.report/v1` |
| Markdown contract | `mcp-doctor.markdown/v1` |
| Selected protocol revision | `2026-07-28` |
| Negotiated protocol revision | Not present |
| Outcome | `incomplete` |
| Exit | `3` (`incomplete_evidence`) |
| Limit profile | `default` |

## Summary

| Measure | Count |
| --- | ---: |
| Checks | 1 |
| Required | 1 |
| Optional | 0 |
| Performed | 1 |
| Skipped | 0 |
| Passed | 0 |
| Warned | 0 |
| Incomplete | 1 |
| Failed | 0 |
| Required skipped | 0 |
| Info findings | 0 |
| Warning findings | 0 |
| Error findings | 1 |
| Critical findings | 0 |

## Protocol selection

No passive selection evidence is present.

## Primary diagnosis

- Check: `schema.contracts`
- `MCP-SCHEMA-005` at `tools[0].inputSchema`: Provide a minimized, wholly synthetic reproducer through the private project support route, then rerun with a release that can complete validation within the same bound.

## Independent safety findings

None.

## Causal skips

None.

## Effective limits

- Profile: `default`
- Time: `startup_ms=10000`, `discovery_ms=10000`, `request_ms=30000`, `response_ms=30000`, `shutdown_grace_ms=2000`, `total_ms=120000`
- I/O: `message_bytes=1048576`, `stdout_bytes=8388608`, `stderr_bytes=1048576`, `aggregate_output_bytes=8388608`, `message_count=1024`
- Network: `endpoint_bytes=8192`, `resolution_addresses=16`, `resolution_count=1`, `trust_bytes=1048576`, `trust_certificates=32`
- Request fields: `request_fields=64`, `request_field_name_bytes=256`, `request_field_value_bytes=8192`, `request_fields_bytes=32768`
- Response fields: `response_fields=96`, `response_field_name_bytes=256`, `response_field_value_bytes=16384`, `response_fields_bytes=65536`
- Discovery: `protocol_revisions=32`, `catalog_items=10000`, `report_findings=256`, `report_bytes=4194304`
- Schema: `schema_bytes=1048576`, `instance_bytes=1048576`, `schema_nodes=100000`, `schema_depth=64`, `schema_ref_depth=32`, `schema_evaluation_steps=100000`, `validation_errors=100`
- Generation: `active_cases=100`, `generation_attempts=256`, `generation_candidates=64`, `generation_steps=100000`
- Activity: `redirects=0`, `retries=0`, `concurrency=1`

## Checks

### 1. `schema.contracts`

- Requirement: `required`
- State: `performed`
- Outcome: `incomplete`

#### Finding 1: `MCP-SCHEMA-005`

- Severity: `error`
- Protocol revision: `2026-07-28`
- Location: `tools[0].inputSchema`
- What: mcp-doctor could not complete local schema validation within its work bound.
- Why: Schema validity remains unknown, so reporting it as valid or invalid would be misleading.
- Evidence: `phase=compile_construction`, `limit=schema_evaluation_steps`, `unit=count`, `observed=100001`, `maximum=100000`
- Expected: Local Draft 2020-12 validation must complete within the fixed schema-work bound or report incomplete evidence.
- Corrective action: Provide a minimized, wholly synthetic reproducer through the private project support route, then rerun with a release that can complete validation within the same bound.
- Reference: mcp-doctor bounded local JSON Schema Draft 2020-12 validation contract
- Primary diagnosis: `true`
- Independent safety finding: `false`
