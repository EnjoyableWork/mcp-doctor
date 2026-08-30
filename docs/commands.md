# Diagnostic commands

Choose the least-active diagnostic that answers your question:

| Command | Activity | Use it to |
| --- | --- | --- |
| **`inspect`** | Does not call tools | Validate discovery, definitions, and schemas |
| **`check`** | Calls selected tools | Replay reviewed known inputs and validate results |
| **`break`** | Calls one selected tool | Search deterministic schema-valid edge cases |
| **`reject`** | Calls one selected tool | Verify exact schema-invalid argument rejection |
| **`diff`** | Reads two local files | Compare explicitly captured advertised contracts |

`aggregate` and `capabilities` do not contact a target; see
[Automation and CI](automation.md).

> [!CAUTION]
> `check`, `break`, and `reject` execute real tool calls. Use disposable data
> and a test environment. Finding a tool does not give `mcp-doctor` permission
> to call it.

## Passive `inspect`

Omitting `--protocol-version`, or selecting `auto`, performs bounded passive
revision selection. STDIO `auto` may consume the discovery bound, fully stop
and reap the first process, then start the exact command once more for legacy
initialization. An explicit `2026-07-28`, `2025-11-25`, or `2025-06-18` is a
hard pin: it runs only that lifecycle and, for STDIO, starts the target once.
Neither mode calls a tool.

Inspect a local STDIO server without calling any tools. Put the command you
already use to start the server after `--`:

```bash
mcp-doctor inspect -- node ./dist/server.js --stdio
```

For Streamable HTTP, pass the endpoint URL:

```bash
mcp-doctor inspect https://mcp.example.com/mcp
```

See [MCP revision support](protocol-support.md) for the finite `auto` paths,
explicit hard pins, the exact support matrix, and schema-dialect behavior. See the
[safety model](safety.md) for network gates, cleanup, and hard limits.

In exact mode, a well-formed JSON-RPC error on the selected revision's first
lifecycle method is an `MCP-PROTOCOL-006` revision-layer diagnosis, because
catalog validity is not yet known. In `auto`, a recognized modern response is
conclusive; a non-modern STDIO error or the exact Streamable HTTP `400` legacy
signal may enter the one legacy path described in the revision contract.
Errors from later capability-advertised catalog methods are `MCP-CATALOG-004`
findings at the exact method response. Human, JSON, JUnit, and Markdown reports
expose only structural selection evidence, the fixed error kind, and a
standard JSON-RPC code when one applies; they never retain error prose, data,
bodies, or application-defined numeric codes.

## Single-tool reviewed `check` scenarios

For one exact tool, `check` accepts a regular file containing strict
`mcp-doctor.scenario/v1alpha1` JSON. The scenario declares its effect
classification and contains 1–100 cases that run sequentially in array order:

```json
{
  "schema_version": "mcp-doctor.scenario/v1alpha1",
  "tool": "search",
  "safety": { "effects": "read_only" },
  "target_env": ["UPSTREAM_TOKEN"],
  "cases": [
    {
      "id": "basic",
      "arguments": { "query": "MCP", "token": null },
      "secret_refs": { "/token": "TOOL_TOKEN" },
      "expect": {
        "result": "success",
        "structured_output_schema": {
          "type": "object",
          "required": ["items"]
        }
      }
    }
  ]
}
```

Case IDs aid review; reports use numeric indexes. `target_env` copies only
same-named invoking-process variables into the constrained target environment.
Each `secret_refs` entry maps an RFC 6901 pointer at an existing `null`
argument placeholder to an invoking-process environment variable. Names use
`[A-Za-z_][A-Za-z0-9_]*`, and argument secrets must be UTF-8 JSON strings.
Missing values, invalid pointers, non-null destinations, duplicates, unknown
fields, and unsupported schemas fail before target start. There is no
interpolation, `.env` or file loading, command execution, prompt, keychain, or
secret-store lookup.

Every run repeats the exact tool through `--allow-tool`; `side_effecting` also
requires `--allow-side-effects`, and annotations grant no authority. Arguments
must match the advertised input schema before a call. Completed results are
checked against `success` or `tool_error`, the advertised output schema, and
any scenario output schema. Ordinary mismatches and tool rejections allow later
cases; transport, protocol, cleanup, authorization, and exhausted limits stop
them. `input_required` makes the case and an otherwise successful report
incomplete without supplying input or retrying the call.

Run the reviewed scenario:

```bash
mcp-doctor check \
  --scenario path/to/scenario.json \
  --allow-tool search \
  -- node ./dist/server.js --stdio
```

## Bounded multi-tool `check` workflows

For a reviewed cross-tool path under exact MCP `2026-07-28`, `check` also
accepts the separately versioned
[`mcp-doctor.scenario/v2alpha1`](../schemas/mcp-doctor.scenario.v2alpha1.schema.json)
contract. It supports one finite sequence, not a general orchestration language:

```json
{
  "schema_version": "mcp-doctor.scenario/v2alpha1",
  "steps": [
    {
      "id": "locate",
      "tool": "lookup",
      "safety": { "effects": "read_only" },
      "arguments": { "query": "MCP" },
      "captures": { "resource_id": "/resource/id" },
      "expect": { "result": "success" }
    },
    {
      "id": "verify",
      "tool": "read",
      "safety": { "effects": "read_only" },
      "arguments": { "id": null },
      "argument_refs": { "/id": "resource_id" },
      "expect": { "result": "success" }
    }
  ]
}
```

Each of the 1–100 ordered steps names one exact tool, declares `read_only` or
`side_effecting`, supplies object arguments, and expects `success` or
`tool_error`. `captures` may select only bounded values from a successful,
schema-valid `structuredContent` using RFC 6901 pointers. `argument_refs` may
copy only an earlier named capture into an existing `null` placeholder. Capture
names are unique across the workflow. The same environment-only `target_env`
and `secret_refs` rules apply; secret and capture destinations cannot overlap.

Repeat `--allow-tool <exact-name>` exactly once for every distinct tool in the
document, with no omissions, duplicates, extras, or wildcard. If any main or
cleanup step is `side_effecting`, also pass `--allow-side-effects`. All selected
tools are discovered and locally schema-validated before the first call;
discovery and annotations never grant authority.

```console
mcp-doctor check \
  --scenario workflow.json \
  --allow-tool lookup \
  --allow-tool read \
  -- your-server
```

Main steps run sequentially at concurrency one. The first failure, incomplete
result, missing capture, unsafe response, transport loss, or exhausted safety
limit causally skips later main steps. A contiguous suffix marked
`"cleanup": true` may still run when its earlier captures and remaining bounds
are available; cleanup steps must expect success and cannot capture values. A
cleanup failure is an independent Critical finding and stops later calls.

Reports use only numeric `runtime.workflow.step[n]` and
`runtime.workflow.cleanup[n]` checks. They never retain step IDs, tool or
capture names, pointers, arguments, results, endpoints, credentials,
environment names, or captured values. Arguments and individual captures are
limited to 1 MiB; active arguments and retained captures are each limited to
8 MiB in aggregate, in addition to the existing message, schema-work, process,
network, time, report, zero-retry, and concurrency-one limits.

Workflow scenarios do not support loops, branches, expressions,
interpolation, scripts, dynamic tool selection, LLM planning, automatic input
responses, retries, or concurrency. They are rejected before target preparation
when a legacy protocol revision is selected. Use `v1alpha1` unchanged for
single-tool legacy `check` runs; `break` and `reject` retain their own exact
single-tool contracts.

## Generated `break` cases

`break` derives schema-valid object inputs only from one bounded local Draft
2020-12 tool schema. Matching `--tool` and `--allow-tool` values select and
authorize it; the run also declares `read_only` or `side_effecting`, 1–100
cases, and an unsigned 64-bit seed. A `side_effecting` run also requires `--allow-side-effects`.
Annotations, wildcards, patterns, and discovery never grant authority.

```bash
mcp-doctor break \
  --tool search \
  --allow-tool search \
  --effects read_only \
  --cases 50 \
  --seed 4242 \
  -- node ./dist/server.js --stdio
```

`mcp-doctor.generator/v1` builds a finite candidate set, validates every
candidate, and runs selected cases sequentially at concurrency one. Caps are
256 attempts, 64 retained candidates, 100,000 synthesis steps, 1 MiB per input,
and 8 MiB across active inputs, alongside the schema, reference, validation,
transport, response, and total-run limits. Invalid, externally referenced,
unsatisfiable, or over-limit schemas stop before `tools/call`.

Reports retain the generator version, case seed, serialized byte count, and
fixed structural JSON counts—never member names or values. Case `n` uses the
base seed plus `n` with unsigned wraparound; replay it with the same tool,
schema, reported seed, and `--cases 1`. Tool errors do not hide later cases;
`input_required`, unsafe failures, and cleanup follow `check` stop rules.

Generated runs accept no target-environment or argument-secret sources, fetch
no schemas, select no other tool, and cannot change the local executable or
widen the authorized endpoint. Ordinary reports never contain raw generated
arguments or tool results.

## Schema-invalid `reject` cases

`reject` tests one exact tool on STDIO or Streamable HTTP under MCP
`2026-07-28`. Matching `--tool` and `--allow-tool` values, an explicit
`read_only` or `side_effecting` classification, and an unsigned 64-bit seed are
required. A side-effecting run additionally requires `--allow-side-effects`;
tool annotations never grant authority.

```bash
mcp-doctor reject \
  --tool search \
  --allow-tool search \
  --effects read_only \
  --seed 4242 \
  -- node ./dist/server.js --stdio
```

An expected rejection is not an execution safeguard: a defective server may
still run a schema-invalid call. Use disposable data and a test target even for
a `read_only` run.

The command considers seven mutations in a fixed order: omitted arguments,
wrong root type, one omitted required property, one wrong property type, a
forbidden null, one invalid enum value, and one unexpected property. It starts
from bounded locally valid candidates and transmits a mutation only after the
same local validator proves exactly one structural mismatch. Mutations that do
not apply to the advertised schema are reported as skipped. Invalid,
externally referenced, unsatisfiable, or over-limit schemas stop before any
`tools/call`.

A case passes only for a matching, structurally valid JSON-RPC error whose code
is exactly `-32602` and whose message is a string. `mcp-doctor` never matches or
retains that prose. A malformed or different error fails the case; any result,
including `isError: true` or `input_required`, is a critical unsafe acceptance
and stops later calls. Reports retain only the generator version, seed,
mutation kind, and structural input counts—not arguments, results, error data,
or error messages.

## Contract snapshots and offline diffs

Contract snapshots are explicit developer artifacts, not ordinary reports.
Create one during passive inspection of the revision selected by default
`auto`, or with an explicit `2026-07-28`, `2025-11-25`, or `2025-06-18` hard
pin, by naming the same new path twice:

```bash
mcp-doctor inspect \
  --snapshot build-a.contract.json \
  --allow-sensitive-snapshot build-a.contract.json \
  -- node ./dist/server.js --stdio

mcp-doctor inspect \
  --protocol-version 2025-11-25 \
  --snapshot legacy.contract.json \
  --allow-sensitive-snapshot legacy.contract.json \
  -- node ./dist/server.js --stdio
```

The repeated path acknowledges sensitivity. A snapshot can expose proprietary
API shape, internal names, and allowed values:

- **Retained:** the selected revision and, for the two legacy revisions, its
  exact matching negotiated identity; normalized capabilities; tool and prompt
  names; resource URIs and templates; prompt argument names; tool behavior
  hints; JSON Schema property names; and validation values such as `const`,
  `enum`, patterns, and bounds. Legacy snapshots retain only fixed presence
  booleans for logging and completions and, for MCP `2025-11-25`, the supported
  task capability booleans.
- **Excluded:** descriptions, titles, defaults, examples, comments, server
  identity and instructions, pagination and cache metadata, experimental
  capability values, transport endpoints, DNS and peer data, credentials and
  source names, headers, arguments, results, logs, and stderr.

Treat snapshots like source containing private interfaces: set a retention
period and review them before sharing. `mcp-doctor` derives the artifact from
the same bounded discovery conversation, makes no second run or list request,
creates one new regular file (`0600` on Unix), and has no overwrite or force
mode. It can still capture a complete catalog when an artifact-retained local
schema shape makes the ordinary report fail, letting an ordinal such as
`tools[73]` resolve inside the artifact. Transport, protocol,
external-reference, resource-bound, unrepresentable, incomplete-catalog, and
cleanup failures write no snapshot.

Compare two artifacts without a target:

```bash
mcp-doctor diff before.contract.json after.contract.json
mcp-doctor diff --format json before.contract.json after.contract.json
```

`diff` reads exactly two bounded regular files without starting a process,
opening a connection, retrieving a schema, or calling a tool. Both artifacts
must identify the same supported revision; cross-revision, selected/negotiated
identity mismatch, and incompatible revision-specific artifacts are rejected
without coercion, comparison, or value reflection. It normalizes catalog and
set-like schema ordering, validates ordinal maps but ignores them for semantic
matching, and emits stable human or
`mcp-doctor.contract-diff/v1alpha1` codes for additions, removals, capability
changes, required inputs, and finite syntactic narrowing or widening rules.

For MCP `2025-11-25`, an omitted schema dialect retains that revision's Draft
2020-12 default. For MCP `2025-06-18`, omission is recorded as ambiguous and a
changed schema receives `review_required` unless both artifacts explicitly
declare supported Draft 2020-12 semantics. Other schema and behavior-hint
changes are also `review_required`; this is not general JSON Schema implication,
cross-revision inference, universal compatibility, protocol conformance, or a
health score. Unchanged or documented-compatible diffs exit `0`, potentially
breaking or review-required diffs exit `1`, and invalid, over-limit, or
unreadable artifacts exit `2`.

The artifact contracts are published with the source as
[`mcp-doctor.contract-snapshot/v1alpha1`](../schemas/mcp-doctor.contract-snapshot.v1alpha1.schema.json)
and
[`mcp-doctor.contract-diff/v1alpha1`](../schemas/mcp-doctor.contract-diff.v1alpha1.schema.json).

## Findings you can act on

Every finding includes a stable code, severity, MCP revision, safe field
location, and performed or skipped state. Active reports retain the declared
case index or generated seed and a structural input shape—enough to repeat a
failure without secrets or raw production data. Related problems identify the
first actionable cause, skip only dependent checks with a reason, and keep
independent checks running.

`MCP-SCHEMA-005` is reserved for bounded local validator evidence that could
not be completed after preliminary schema gates passed. It safely records
`meta_validation` or `compile_construction`, `schema_evaluation_steps`, count,
maximum, and structural location. The check remains performed with outcome
`incomplete`; a true contract or safety failure outranks it without removing
the evidence.

### Tool-description quality

Passive `inspect` reports `MCP-QUALITY-001` as a warning at
`tools[index].description` when an otherwise inspectable tool omits its
description or supplies a blank string. The correction is to provide a concise
description of what the tool does and when to select it.

`MCP-QUALITY-003` reports the same structural location when a non-blank
description normalizes to the tool name or to one fixed placeholder: `todo`,
`tbd`, `tool`, `description`, or `placeholder`. The finding retains none of the
name, description, normalized text, or matched placeholder. Its correction is
to replace the placeholder or repeated name with what the tool does and when
to select it. `MCP-QUALITY-001` remains authoritative for an absent or blank
description.

`MCP-QUALITY-004` reports a later uniquely named tool whose otherwise usable
description is exactly equal to an earlier eligible description after `A1
normalization v1`. Eligibility requires a structurally valid string name that
occurs exactly once in the accepted catalog prefix, a description classified
as usable by the rules above, and a nonempty normalized value. Invalid,
missing, or nonunique names and missing, blank, non-string, placeholder, or
name-only descriptions keep their existing authoritative findings and do not
also receive `MCP-QUALITY-004`. A punctuation-only description whose
normalization is empty is excluded.

Eligible tools are finalized in global `tools` catalog order across pages. The
first eligible occurrence of one exact normalized value is canonical and is
not warned. Every later eligible occurrence receives one warning at
`tools[index].description`, in that same order, with the canonical global index
in `first_matching_tool_index`. If a later accepted entry makes a name
nonunique, every accepted entry with that name is excluded before canonical
selection and warnings are finalized. An entry beyond the `catalog_items`
bound cannot create a match, become canonical, or affect name uniqueness.

The stable JSON evidence is:

```json
{
  "kind": "rule_violation",
  "rule": "reused_normalized_tool_description",
  "first_matching_tool_index": 0
}
```

The correction is to distinguish what the later tool does, when it should and
should not be selected, and how it differs from the tool at
`first_matching_tool_index`. One tool receives at most one description-quality
finding: `MCP-QUALITY-001`, `MCP-QUALITY-003`, and
existing catalog findings remain authoritative before reuse eligibility is
considered.

A non-string description remains `MCP-CATALOG-001` without an additional
quality warning.

`A1 normalization v1` defines blank deterministically as an empty string or a
string containing only these Unicode scalar values:

- `U+0009`–`U+000D`
- `U+0020`
- `U+0085`
- `U+00A0`
- `U+1680`
- `U+2000`–`U+200A`
- `U+2028`
- `U+2029`
- `U+202F`
- `U+205F`
- `U+3000`

For the `MCP-QUALITY-003` and `MCP-QUALITY-004` comparisons, `A1 normalization
v1` trims boundary ASCII whitespace, collapses each internal ASCII-whitespace
run to one space, removes ASCII punctuation, and lowercases ASCII letters. All
non-ASCII scalar values remain scalar-for-scalar significant: mcp-doctor does
not transliterate, locale-fold, or infer semantic similarity. Reuse detection
compares only exact normalized values; these passive rules do not grade
near-duplicate or semantically similar prose, readability, jargon, token
efficiency, sentiment, or general language quality.

Reuse analysis runs only after the transport conversation completes
successfully and processes the complete accepted catalog prefix. If
`catalog_items` is exceeded, any admitted warning already proven within that
prefix remains, `MCP-LIMIT-001` identifies `catalog_items`, dependent checks
remain skipped, the exit is nonzero, and the report does not claim complete
catalog coverage. If `report_findings` is exceeded, analysis still processes
the complete accepted prefix; the existing global priority policy retains a
deterministic subset without allowing quality warnings to displace security or
higher-priority findings, and `MCP-LIMIT-001` identifies `report_findings` with
the established nonzero exit. Neither limit changes `MCP-QUALITY-004` from
warning severity. A run containing only admitted quality warnings still exits
successfully.

If a later STDIO or Streamable HTTP page exchange fails, the transport finding
remains primary, delivered response prefixes are discarded, catalog and schema
checks remain causally skipped, and no `MCP-QUALITY-004` is emitted from the
discarded prefix.

The rules do not use locale, runtime whitespace tables, or an LLM. Human, JSON,
JUnit, and Markdown findings retain only fixed product vocabulary, code,
warning severity, selected revision, indexed field location, numeric catalog
ordinal, and (for `MCP-QUALITY-004`) `first_matching_tool_index`. Badge and
aggregate projections retain only their existing summary information. Tool
names, raw or normalized descriptions, catalog and schema values, server
stderr, credentials, environment data, and local paths are not retained or
rendered. The shared implementation applies to passive STDIO and Streamable
HTTP inspection for every supported revision without an extra request, tool
call, dependency, retry, fallback, target launch, network activity, or
execution authority.

### Required-input description quality

After an advertised tool input schema passes its structural, dialect,
reference, meta-schema, compilation, and work-budget gates, passive `inspect`
reports `MCP-QUALITY-002` as a warning for each direct property that is named by
the root `required` array but lacks a usable direct `description`. A description
is usable when it is a string that is not blank under the finite `A1
normalization v1` whitespace set above.

Each finding is located at
`tools[index].inputSchema.properties[index].description`. It retains only the
code, warning severity, selected revision, ordinal location, and fixed
corrective prose. Tool names, property names, descriptions, defaults, examples,
schemas, and reference targets are not retained or rendered. The correction is
to describe the accepted value and any important constraints for the required
input.

The rule inspects only the direct property annotation. A local-reference
wrapper therefore needs its own description; a description on the referenced
target is not inherited for this diagnostic. Optional properties and names in
`required` that have no direct `properties` entry are not diagnosed. The rule
does not infer intent, follow external references, or inspect nested required
properties.

An invalid, unsupported, externally referenced, incomplete, or over-limit
schema receives its prerequisite schema diagnosis instead of a partial
required-input quality result for that schema. The scan reuses the existing
schema-work and report-finding budgets, applies to passive STDIO and Streamable
HTTP for every supported revision, and adds no request, tool call, dependency,
or LLM evaluation.

### Credential-literal input safety

After an advertised tool input schema passes its existing local schema gates,
passive `inspect` reports `MCP-SECURITY-001` as an error when a direct property
has both:

- an identifier containing one exact normalized credential segment:
  `password`, `passwd`, `secret`, `token`, `api-key`, `apikey`,
  `access-token`, `private-key`, or `credential`; and
- a non-empty string literal in `default`, `const`, `examples`, or `enum`.

Identifier normalization is finite and ASCII-only. Punctuation and non-ASCII
scalars delimit segments, ASCII letters are lowercased, and camel-case or
acronym-to-word transitions delimit segments. Each single-segment term matches
one complete normalized segment; `api-key`, `access-token`, and `private-key`
match exact adjacent segments. The rule does not use entropy, prose, locale,
semantic inference, an LLM, or a broader secret scanner. It does not diagnose
nested properties or a credential-like property that advertises no literal.

Each finding is located at
`tools[index].inputSchema.properties[index].keyword` and retains only the
error code, severity, selected revision, ordinal location, keyword class, and
non-empty string-literal count. Property names, matched segments, literals,
schema fragments, and tool metadata are not retained or rendered. An invalid,
unsupported, externally referenced, incomplete, or over-limit input schema
receives its prerequisite schema finding instead of a partial credential
diagnosis for that schema.

The credential finding remains independent of an earlier actionable diagnosis
from another tool. The correction is to remove the literal from the advertised
schema and obtain the credential through authorized server runtime
configuration. The shared passive rule applies to every supported revision and
transport without reading credentials, resolving environment values, fetching
references, sending another request, or calling a tool.

```text
PRIMARY DIAGNOSIS · schema

MCP-SCHEMA-004  error  tools[3].inputSchema.required

Why:
  `required` is a string, so clients cannot interpret the advertised input.

Expected:
  an array of unique property names

Fix:
  change `required` to an array, then run `mcp-doctor` again

Checks skipped because of this issue:
  tool/runtime
```

Human, stable JSON, and JUnit reports share one immutable result, so CI cannot
hide a failure, choose another primary issue, or turn a skip into a pass.
