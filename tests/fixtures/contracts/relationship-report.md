<!-- mcp-doctor.markdown/v1 -->
# mcp-doctor diagnostic report

| Field | Value |
| --- | --- |
| Product | `mcp-doctor 0.4.1` |
| Report contract | `mcp-doctor.report/v1` |
| Markdown contract | `mcp-doctor.markdown/v1` |
| Selected protocol revision | `2026-07-28` |
| Negotiated protocol revision | Not present |
| Outcome | `failed` |
| Exit | `1` (`unsuccessful_result`) |
| Limit profile | `default` |

## Summary

| Measure | Count |
| --- | ---: |
| Checks | 3 |
| Required | 3 |
| Optional | 0 |
| Performed | 2 |
| Skipped | 1 |
| Passed | 0 |
| Warned | 0 |
| Incomplete | 0 |
| Failed | 2 |
| Required skipped | 1 |
| Info findings | 0 |
| Warning findings | 0 |
| Error findings | 1 |
| Critical findings | 1 |

## Protocol selection

No passive selection evidence is present.

## Primary diagnosis

- Check: `discovery.catalogs`
- `MCP-CATALOG-001` at `tools`: Correct the value at the reported structural location, then rerun inspect.

## Independent safety findings

- `MCP-SAFETY-001` in `transport.stdio` at `process`: Make the server and descendants exit when STDIN closes or termination is requested.

## Causal skips

- `runtime.tools` (`required`) was skipped: a required prerequisite did not pass. Blocked by `discovery.catalogs` (`MCP-CATALOG-001` at `tools`).

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

### 1. `transport.stdio`

- Requirement: `required`
- State: `performed`
- Outcome: `failed`

#### Finding 1: `MCP-SAFETY-001`

- Severity: `critical`
- Protocol revision: `2026-07-28`
- Location: `process`
- What: The managed target could not be fully cleaned up.
- Why: A surviving process can keep consuming resources or running after inspection.
- Evidence: None.
- Expected: The managed process tree must terminate and be reaped before mcp-doctor returns.
- Corrective action: Make the server and descendants exit when STDIN closes or termination is requested.
- Reference: mcp-doctor bounded local STDIO safety contract
- Primary diagnosis: `false`
- Independent safety finding: `true`

### 2. `discovery.catalogs`

- Requirement: `required`
- State: `performed`
- Outcome: `failed`

#### Finding 1: `MCP-CATALOG-001`

- Severity: `error`
- Protocol revision: `2026-07-28`
- Location: `tools`
- What: An advertised MCP catalog does not match its protocol contract.
- Why: Clients cannot reliably discover or use a capability with this structure.
- Evidence: `rule=expected_shape`, `expected=array`, `observed=string`
- Expected: Each advertised catalog response and item must match the selected MCP revision.
- Corrective action: Correct the value at the reported structural location, then rerun inspect.
- Reference: selected MCP revision catalog contracts
- Primary diagnosis: `true`
- Independent safety finding: `false`

### 3. `runtime.tools`

- Requirement: `required`
- State: `skipped`
- Skip reason: `prerequisite_failed`
- Explanation: a required prerequisite did not pass
- Blocked by check: `discovery.catalogs`
