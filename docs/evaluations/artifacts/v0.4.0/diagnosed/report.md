<!-- mcp-doctor.markdown/v1 -->
# mcp-doctor diagnostic report

| Field | Value |
| --- | --- |
| Product | `mcp-doctor 0.4.0` |
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
| Checks | 6 |
| Required | 5 |
| Optional | 1 |
| Performed | 3 |
| Skipped | 3 |
| Passed | 2 |
| Warned | 0 |
| Incomplete | 0 |
| Failed | 1 |
| Required skipped | 2 |
| Info findings | 0 |
| Warning findings | 0 |
| Error findings | 1 |
| Critical findings | 0 |

## Protocol selection

- Mode: `exact`
- Path: `exact_pin`
- Selected revision: `2026-07-28`
- Bounded work: `process_launches=1`, `lifecycle_requests=1`, `lifecycle_notifications=0`, `fallbacks=0`

## Primary diagnosis

- Check: `protocol.revision`
- `MCP-PROTOCOL-002` at `server.supportedVersions`: Add MCP 2026-07-28 support, then rerun the same diagnosis without falling back.

## Independent safety findings

None.

## Causal skips

- `discovery.catalogs` (`required`) was skipped: the protocol revision is unsupported. Blocked by `protocol.revision` (`MCP-PROTOCOL-002` at `server.supportedVersions`).
- `schema.contracts` (`required`) was skipped: the protocol revision is unsupported. Blocked by `protocol.revision` (`MCP-PROTOCOL-002` at `server.supportedVersions`).

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
- Outcome: `passed`
- Findings: None.

### 2. `protocol.envelope`

- Requirement: `required`
- State: `performed`
- Outcome: `passed`
- Findings: None.

### 3. `protocol.revision`

- Requirement: `required`
- State: `performed`
- Outcome: `failed`

#### Finding 1: `MCP-PROTOCOL-002`

- Severity: `error`
- Protocol revision: `2026-07-28`
- Location: `server.supportedVersions`
- What: The server does not support the required protocol revision.
- Why: Applying rules for a different revision could produce a false diagnosis.
- Evidence: `required_revision=2026-07-28`, `offered=2`, `recognized_legacy=1`, `unknown_date=0`, `opaque=1`
- Expected: The server must support MCP protocol revision 2026-07-28 for this diagnosis.
- Corrective action: Add MCP 2026-07-28 support, then rerun the same diagnosis without falling back.
- Reference: selected MCP revision lifecycle contract
- Primary diagnosis: `true`
- Independent safety finding: `false`

### 4. `discovery.catalogs`

- Requirement: `required`
- State: `skipped`
- Skip reason: `unsupported_revision`
- Explanation: the protocol revision is unsupported
- Blocked by check: `protocol.revision`

### 5. `schema.contracts`

- Requirement: `required`
- State: `skipped`
- Skip reason: `unsupported_revision`
- Explanation: the protocol revision is unsupported
- Blocked by check: `protocol.revision`

### 6. `runtime.tools`

- Requirement: `optional`
- State: `skipped`
- Skip reason: `not_authorized`
- Explanation: active behavior was not authorized
