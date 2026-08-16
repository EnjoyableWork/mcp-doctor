# MCP revision support

`mcp-doctor` uses MCP `2026-07-28` by default. A supported legacy revision must
be selected exactly with `--protocol-version`; revision selection never
auto-detects, retries, falls back, or downgrades.

## Select a revision

```bash
# Passive legacy STDIO inspection
mcp-doctor inspect \
  --protocol-version 2025-11-25 \
  -- node ./dist/server.js

# Passive legacy Streamable HTTP inspection
mcp-doctor inspect \
  --protocol-version 2025-06-18 \
  https://mcp.example.com/mcp

# Active legacy replay remains gated by the exact scenario tool
mcp-doctor check \
  --protocol-version 2025-06-18 \
  --scenario path/to/scenario.json \
  --allow-tool search \
  -- node ./dist/server.js --stdio
```

Legacy inspection performs only `initialize`, one
`notifications/initialized`, and capability-advertised `tools/list`,
`prompts/list`, `resources/list`, and `resources/templates/list` operations. It
does not call tools, list retained tasks, read resources, get prompts, or answer
server requests.

Explicit MCP `2025-11-25` and `2025-06-18` `check` and `break` preserve every
active authorization gate, call only immediate tools, never start tasks or
answer server requests, and leave required additional input incomplete without
retrying. For active MCP `2025-06-18`, every advertised input schema and every advertised output schema that `mcp-doctor` interprets
must declare the exact supported Draft 2020-12 URI; ambiguity stops before
generation or `tools/call`.

## Support matrix

**Legend:** ✅ = supported; ❌ = not supported. A supported legacy entry still
requires exact `--protocol-version` selection. Each status cell includes an
invisible `mcp-doctor-support=supported|unsupported` source token for agents
reading the Markdown. For automation,
`mcp-doctor capabilities --format json` is the authoritative machine-readable
contract.

| MCP revision | Est. usage[^revision-usage] | `inspect` | Snapshot | Same-revision `diff` | `check` | `break` | `reject` |
| --- | ---: | --- | --- | --- | --- | --- | --- |
| `2026-07-28` | 11.2% | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> |
| `2025-11-25` | 77.4% | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ❌ <!-- mcp-doctor-support=unsupported --> |
| `2025-06-18` | 8.1% | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ❌ <!-- mcp-doctor-support=unsupported --> |
| `2025-03-26` | 1.9% | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> |
| `2024-11-05` | 1.3% | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> |
| `2024-10-07` | Under 0.1% | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> |
| Unknown | — | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> |

The ❌ state covers both recognized-but-unsupported and unknown revisions. Each
invocation still fails closed with its precise unsupported or unknown
diagnosis.

[^revision-usage]: Dated 2026-08-13 planning proxy, rounded from seven-day
    downloads of official TypeScript and Python SDK releases grouped by their
    advertised default revision. Package downloads are not unique deployments
    or runtime traffic. Sources: [npm version downloads](https://api.npmjs.org/versions/%40modelcontextprotocol%2Fsdk/last-week)
    and the [public PyPI dataset](https://github.com/ClickHouse/clickpy).

## Evidence boundary

Supported `inspect` and snapshot entries cover STDIO and Streamable HTTP;
`diff` is local-only. Current-revision active support has broad matrix evidence.
Legacy inspection, snapshot, diff, and active HTTP behavior have synthetic
evidence. MCP `2025-11-25` active STDIO additionally has narrow controlled
evidence from one pinned official Go server and one pinned independent PHP
server. MCP `2025-06-18` active STDIO has synthetic and represented
source-install evidence only. No broad legacy ecosystem claim follows.

`reject` is current-revision-only and has bounded synthetic STDIO and
Streamable HTTP evidence; it does not carry a broad real-server reach claim.
The exact pinned scope and results live in the
[compatibility evidence](../tests/compatibility/README.md).

## JSON Schema dialects

Advertised tool schemas are checked locally and without external retrieval.
MCP `2025-11-25` defaults an omitted dialect to bounded JSON Schema Draft
2020-12. Because MCP `2025-06-18` did not define a default, passive `inspect`
records an omitted dialect as ambiguous after bounded structural and reference
checks, without assigning dialect-specific semantics.

Active MCP `2025-06-18` instead requires the exact Draft 2020-12 declaration
before scenario validation, generation, or a tool call. Missing, malformed,
unsupported, external, ambiguous, unsupported-vocabulary, or over-limit
contracts fail closed. An omitted advertised output schema remains optional
rather than being inferred.
