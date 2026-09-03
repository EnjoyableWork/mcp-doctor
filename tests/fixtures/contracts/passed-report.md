<!-- mcp-doctor.markdown/v1 -->
# mcp-doctor diagnostic report

| Field | Value |
| --- | --- |
| Product | `mcp-doctor 0.4.2` |
| Report contract | `mcp-doctor.report/v1` |
| Markdown contract | `mcp-doctor.markdown/v1` |
| Selected protocol revision | `2026-07-28` |
| Negotiated protocol revision | Not present |
| Outcome | `passed` |
| Exit | `0` (`success`) |
| Limit profile | `default` |

## Summary

| Measure | Count |
| --- | ---: |
| Checks | 1 |
| Required | 1 |
| Optional | 0 |
| Performed | 1 |
| Skipped | 0 |
| Passed | 1 |
| Warned | 0 |
| Incomplete | 0 |
| Failed | 0 |
| Required skipped | 0 |
| Info findings | 0 |
| Warning findings | 0 |
| Error findings | 0 |
| Critical findings | 0 |

## Protocol selection

No passive selection evidence is present.

## Primary diagnosis

None.

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

### 1. `protocol.revision`

- Requirement: `required`
- State: `performed`
- Outcome: `passed`
- Findings: None.
