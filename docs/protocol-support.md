# MCP revision support

Passive `mcp-doctor inspect` defaults to bounded `--protocol-version auto`.
It selects only a mutually supported revision compiled into this binary and
uses the transport-defined modern-or-legacy era path below. An explicit
supported revision remains a strict hard pin. Active commands keep MCP
`2026-07-28` as their sole implicit revision; supported legacy activity still
requires an exact `--protocol-version`.

## Select a revision

```bash
# Explicitly request the same bounded passive default
mcp-doctor inspect \
  --protocol-version auto \
  -- node ./dist/server.js

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

The compiled modern set is exactly MCP `2026-07-28`. `auto` sends one
`server/discover`. A valid modern result selects `2026-07-28` only when its
bounded `supportedVersions` contains that revision. A recognized modern error
is conclusive and never enters legacy initialization; because this binary has
no second compiled modern revision, an `UnsupportedProtocolVersionError`
without a mutual revision fails at `protocol.revision` without retransmission
or sequential guessing.

For STDIO, a non-modern well-formed JSON-RPC error, clean pre-response exit, or
the exact discovery deadline is finite legacy-era evidence. `mcp-doctor` first
closes, terminates when needed, and reaps that process tree. It may then start
the byte-for-byte selected command once more and send one `initialize` offering
`2025-11-25`; only an exact `2025-11-25` response or supported `2025-06-18`
counter-offer is accepted. Invalid framing or JSON, start or I/O failure,
resource or total limit, and cleanup failure are terminal and never authorize
the second launch. The two non-overlapping phases share one original total
deadline and the existing cumulative byte, message, stdout, stderr, output,
and finding budgets.

For Streamable HTTP, `auto` prepares the canonical endpoint, network and
credential gates, trust, bounded DNS answer set, and peer authority once. It
enters legacy initialization only after exact HTTP `400 Bad Request` with an
empty bounded body or a body that is not a recognized modern JSON-RPC error.
The legacy request uses the same endpoint, pinned address set, peer checks, and
credential authority without re-resolution. A recognized modern error, any
other status, redirect, timeout, TLS, trust, peer, framing, encoding, body,
limit, or cleanup failure is terminal. There are zero application retries and
zero redirects, and both eras share the original deadline and aggregate
budgets.

Legacy inspection performs only `initialize`, one
`notifications/initialized`, and capability-advertised `tools/list`,
`prompts/list`, `resources/list`, and `resources/templates/list` operations. It
does not call tools, list retained tasks, read resources, get prompts, or answer
server requests. Reports add value-free mode, selected revision when
established, fixed path, process-launch, lifecycle-request, notification, and
fallback counts. They discard error prose and data, bodies, server identity and
instructions, commands and paths, endpoints and network values, credentials,
environment names and values, catalog identifiers, cursors, and stderr.

An explicit revision bypasses `auto`: it sends one selected lifecycle, never
probes another era, never retries, and never falls back. A well-formed JSON-RPC
error on its first `server/discover` or `initialize` is `MCP-PROTOCOL-006` at
the exact lifecycle response. A later error from an advertised fixed catalog
method is `MCP-CATALOG-004` at that method's response. Reports retain only a
value-free error kind and, for the five standard JSON-RPC errors, the standard
numeric code; messages, data, and application-defined codes are discarded.

Explicit MCP `2025-11-25` and `2025-06-18` `check` and `break` preserve every
active authorization gate, call only immediate tools, never start tasks or
answer server requests, and leave required additional input incomplete without
retrying. For active MCP `2025-06-18`, every advertised input schema and every advertised output schema that `mcp-doctor` interprets
must declare the exact supported Draft 2020-12 URI; ambiguity stops before
generation or `tools/call`.

## Support matrix

**Legend:** ✅ = supported; ❌ = not supported. Passive `inspect` can select a
supported legacy entry through bounded `auto` or an exact hard pin. Active
legacy entries require an exact `--protocol-version`. Each status cell includes
an invisible `mcp-doctor-support=supported|unsupported` source token for agents
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

Across supported revisions, schema evaluation uses a bounded linear-time
regular-expression subset. Patterns requiring backtracking-only constructs,
including look-around and backreferences, receive the typed
`unsupported_linear_pattern` diagnostic at their structural location. This is
a deliberate resource-safety subset of Draft 2020-12 pattern syntax; all
accepted patterns retain the same local, no-retrieval validation boundary.
Before matching, the translated pattern's bounded structural complexity,
including counted-repetition expansion and character-class ranges, is charged
against all potentially inspected instance text under
`schema_evaluation_steps`.
