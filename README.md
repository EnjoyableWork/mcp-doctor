<h1 align="center">🩺 mcp-doctor</h1>

<p align="center">
  <strong>Diagnose, test, and break your MCP servers before your users do.</strong>
</p>

<p align="center">
  Find protocol, schema, and runtime problems in local or remote MCP servers,
  with clear reports you can trust.
</p>

<p align="center">
  <a href="https://github.com/EnjoyableWork/mcp-doctor/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/EnjoyableWork/mcp-doctor/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://docs.rs/crate/mcp-doctor/latest"><img alt="crates.io version" src="https://img.shields.io/crates/v/mcp-doctor.svg?logo=rust&amp;logoColor=white"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <img alt="MCP transports: STDIO and Streamable HTTP" src="https://img.shields.io/badge/MCP-STDIO_%2B_HTTP-6f42c1.svg">
</p>

<p align="center">
  <a href="#the-promise">The promise</a> ·
  <a href="#why-mcp-doctor">Why mcp-doctor?</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#inspect-check-break-diff-aggregate-capabilities">Commands</a> ·
  <a href="#bring-it-into-ci">CI</a> ·
  <a href="#safe-by-default">Safety</a>
</p>

A diagnosis you can act on:

> **Your weather server starts correctly**
>
> `mcp-doctor` found 8 tools, 2 resources, and 1 prompt.
>
> **First thing to fix**
>
> The `weather_forecast` tool describes its required inputs incorrectly. Define
> the required fields as a list, then run the diagnosis again.
>
> **Safe by default**
>
> No tools were called and no server data was changed.

## The promise

`mcp-doctor` is the open-source Rust CLI from
[EnjoyableWork](https://github.com/EnjoyableWork) that checks your MCP server
before users depend on it. It finds the problems it can safely reach, explains
what went wrong, tells you what to fix, and gives people and AI agents the same
trustworthy report.

It does not stop after the first problem. It keeps checking anything that can
still run safely. If one failure blocks later checks, it shows the first issue
to fix and marks only those checks as skipped. Unrelated problems and serious
safety failures stay visible.

## Why mcp-doctor?

A successful connection only proves that a server answered once. It does not
prove that its messages follow MCP rules, its tools are usable, its tools
handle bad input, or its failures can be repeated.

`mcp-doctor` puts those checks into one repeatable report. It shows what ran,
what failed, what was skipped, and what to do next—without calling a tool
unless you ask it to.

| What it checks | What it finds |
| --- | --- |
| **Protocol** | Broken JSON-RPC messages, framing, version handling, methods, and feature claims |
| **Tools and features** | Bad tool, resource, or prompt definitions; duplicate names; and results that change between discovery runs |
| **Schemas** | Invalid or unsafe JSON Schema and input rules that clients cannot use |
| **Results** | Tool output that does not match `outputSchema` or claims success without the promised data |
| **Runtime** | Timeouts, crashes, early exits, bad output, oversized messages, and failed shutdown |
| **Repeatability** | Failures that change between runs, with the seed and input shape needed to run them again |

## Quick start

Install with Homebrew or Cargo:

```bash
# macOS or Linux
brew install --build-from-source EnjoyableWork/tap/mcp-doctor

# Any supported Rust host
cargo install mcp-doctor --version '=0.3.0' --locked
```

Or download a native GNU/Linux archive from
[GitHub Releases](https://github.com/EnjoyableWork/mcp-doctor/releases/latest).

Release archives, the exact Cargo package, the Homebrew formula, checksums,
SPDX SBOMs, and build attestations are published together. See the
[release guide](docs/release.md) to verify what you install.

Inspect a local STDIO server by placing its executable and arguments after
`--`:

```bash
mcp-doctor inspect -- node ./dist/server.js --stdio
```

Inspect a Streamable HTTP endpoint by URL:

```bash
mcp-doctor inspect https://mcp.example.com/mcp
```

All target-facing commands use MCP `2026-07-28` by default. Select a supported
legacy revision explicitly when diagnosing a server that has not migrated:

```bash
# Passive legacy STDIO inspection
mcp-doctor inspect --protocol-version 2025-11-25 -- node ./dist/server.js

# Passive legacy Streamable HTTP inspection
mcp-doctor inspect --protocol-version 2025-06-18 https://mcp.example.com/mcp

# Active legacy replay remains gated by the exact scenario tool
mcp-doctor check \
  --protocol-version 2025-11-25 \
  --scenario path/to/scenario.json \
  --allow-tool search \
  -- node ./dist/server.js --stdio
```

Revision selection never auto-detects, retries, falls back, or downgrades.
Legacy inspection performs only `initialize`, one
`notifications/initialized`, and capability-advertised `tools/list`,
`prompts/list`, `resources/list`, and `resources/templates/list` operations. It
does not call tools, list retained tasks, read resources, get prompts, or answer
server requests. Explicit MCP `2025-11-25` `check` and `break` preserve every
active authorization gate, call only immediate tools, never start tasks or
answer server requests, and leave required additional input incomplete without
retrying. MCP `2025-06-18` remains passive-only.

| MCP revision | Est. usage[^revision-usage] | `inspect` | Snapshot | Same-revision `diff` | `check` | `break` |
| --- | ---: | --- | --- | --- | --- | --- |
| `2026-07-28` | 11.2% | Default | Supported | Supported offline | Supported | Supported |
| `2025-11-25` | 77.4% | Explicit only | Explicit only | Supported offline | Explicit only | Explicit only |
| `2025-06-18` | 8.1% | Explicit only | Explicit only | Supported offline | Not supported | Not supported |
| `2025-03-26` | 1.9% | Rejected | Rejected | Rejected | Rejected | Rejected |
| `2024-11-05` | 1.3% | Rejected | Rejected | Rejected | Rejected | Rejected |
| `2024-10-07` | Under 0.1% | Rejected as unknown | Rejected | Rejected | Rejected as unknown | Rejected as unknown |
| Unknown | — | Rejected | Rejected | Rejected | Rejected | Rejected |

Supported `inspect` and snapshot entries cover STDIO and Streamable HTTP;
`diff` is local-only. Current-revision active support has broad matrix evidence.
Legacy inspection, snapshot, diff, and MCP `2025-11-25` active HTTP behavior
have synthetic evidence; MCP `2025-11-25` active STDIO additionally has narrow
controlled evidence from one pinned official Go server and one pinned
independent PHP server. No broad legacy ecosystem claim follows.

[^revision-usage]: Dated 2026-08-13 planning proxy, rounded from seven-day
    downloads of official TypeScript and Python SDK releases grouped by their
    advertised default revision. Package downloads are not unique deployments
    or runtime traffic. Sources: [npm version downloads](https://api.npmjs.org/versions/%40modelcontextprotocol%2Fsdk/last-week)
    and the [public PyPI dataset](https://github.com/ClickHouse/clickpy).

Let an editor, wrapper, server repository, or CI job check the installed
binary before it chooses a diagnostic:

```bash
mcp-doctor capabilities --format json
```

This command emits deterministic `mcp-doctor.capabilities/v1` JSON containing
the product version; exact command, transport, and MCP revision matrix;
recognized-unsupported revisions; report, scenario, generator, snapshot, diff,
aggregate, and capability contract versions; reporter availability;
`mcp-doctor.exit/v1`; named hard limit profiles; and compile-time process-tree
and file-identity capabilities. It does not inspect user configuration or host
inventory, read credentials, start a process, resolve DNS, connect to a target,
retrieve a schema, or call a tool.
An integration can classify a planned run as supported, unsupported, or unknown
before it selects a target.

Consumers request the representation exactly with
`--schema-version mcp-doctor.capabilities/v1`. There is no discovery, fallback,
or version downgrade. An unsupported request exits `2`; JSON output receives a
value-free typed error listing the supported capability schema. Within stable
`v1`, consumers ignore unknown optional fields and treat unknown command,
transport, revision, reporter, contract, or profile values as unknown. Adding
a new supported entry or optional field is compatible; removing support or
changing a required field or existing meaning requires a new capability major.
The checked-in [Draft 2020-12 schema](schemas/mcp-doctor.capabilities.v1.schema.json)
is the validation contract.

Advertised tool schemas are checked locally and without external retrieval.
MCP `2025-11-25` defaults an omitted dialect to bounded JSON Schema Draft
2020-12. Because MCP `2025-06-18` did not define a default, an omitted dialect
receives bounded structural and reference checks plus an explicit warning,
without assigning dialect-specific semantics. An exact Draft 2020-12
declaration enables full local validation; another dialect is rejected.

The default report is made for people. Add `--format json` for the stable,
schema-backed `mcp-doctor.report/v1` result, or `--format junit` for the same
checks projected into conservative JUnit XML. Both keep secrets removed.

Keep that stdout report while also writing both machine projections from the
same diagnostic run with explicit new-file destinations:

```bash
mcp-doctor inspect \
  --json-report artifacts/mcp-doctor.json \
  --junit-report artifacts/mcp-doctor.xml \
  -- node ./dist/server.js --stdio
```

`--json-report` and `--junit-report` also apply to `check` and `break`. Each
parent directory must already exist, each destination must be distinct and not
already exist, and `-` is not a file destination. `mcp-doctor` validates those
conditions before target or network activity, runs the diagnostic once, and
renders every requested report from one immutable redacted result. A failed or
incomplete diagnostic still publishes both files when reporting succeeds and
retains exit `1` or `3`; a render, write, publication, or cleanup failure cannot
report success and exits `4`.

## Inspect. Check. Break. Diff. Aggregate. Capabilities.

Choose how much activity the target allows:

| Command | Activity | Use it to |
| --- | --- | --- |
| **`inspect`** | Does not call tools | Connect, list what the server offers, and check its definitions and schemas |
| **`check`** | Calls selected tools | Run known inputs from a scenario you wrote and check the results |
| **`break`** | Tries generated edge cases | Search one selected tool for failures you can repeat |
| **`diff`** | Reads two local files only | Compare explicitly captured advertised contracts without starting or contacting a target |
| **`aggregate`** | Reads explicit local files only | Combine stable diagnostic reports conservatively without rerunning or contacting a target |
| **`capabilities`** | Uses compiled facts only | Select, skip, or defer a diagnostic without inspecting a host or target |

```bash
# Passive: discover and validate without calling a tool
mcp-doctor inspect -- node ./dist/server.js --stdio

# Active: replay one reviewed scenario
mcp-doctor check \
  --scenario path/to/scenario.json \
  --allow-tool search \
  -- node ./dist/server.js --stdio

# Active: run 50 deterministic edge cases against one selected tool
mcp-doctor break \
  --tool search \
  --allow-tool search \
  --effects read_only \
  --cases 50 \
  --seed 4242 \
  -- node ./dist/server.js --stdio

# Offline: compare two deliberately retained contract artifacts
mcp-doctor diff --format json before.contract.json after.contract.json

# Offline: combine explicit stable reports into one required JSON artifact
mcp-doctor aggregate \
  --output matrix.aggregate.json \
  reports/linux.json reports/macos.json reports/windows.json

# Compiled only: decide whether this binary supports a planned diagnostic
mcp-doctor capabilities --format json
```

> [!CAUTION]
> `check` and `break` execute real tool calls. Use disposable data and a test
> environment. Finding a tool does not give `mcp-doctor` permission to call it.

### Contract snapshots and offline diffs

Contract snapshots are explicit developer artifacts, not ordinary reports.
Create one during passive inspection of the default MCP `2026-07-28` revision,
or an explicitly selected MCP `2025-11-25` or `2025-06-18` revision, by naming
the same new path twice:

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
`tools[73]` resolve
inside the artifact. Transport, protocol, external-reference, resource-bound,
unrepresentable, incomplete-catalog, and cleanup failures write no snapshot.

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
health score. Unchanged or documented-compatible diffs exit `0`,
potentially breaking or review-required diffs exit `1`, and invalid,
over-limit, or unreadable artifacts exit `2`.

The artifact contracts are published with the source as
[`mcp-doctor.contract-snapshot/v1alpha1`](schemas/mcp-doctor.contract-snapshot.v1alpha1.schema.json)
and
[`mcp-doctor.contract-diff/v1alpha1`](schemas/mcp-doctor.contract-diff.v1alpha1.schema.json).

### Offline diagnostic aggregates

`aggregate` combines one through 32 explicit ordered, stable, redacted
`mcp-doctor.report/v1` regular files into one new
`mcp-doctor.aggregate/v1` artifact. Human stdout is the default;
`--format json` makes stdout byte-identical to the artifact.

Members are identified only by zero-based input ordinal. Known safe fields are
retained; compatible unknown optional properties are discarded, not echoed.
Every input must satisfy the embedded report schema and its summary, severity,
outcome, exit, revision, primary diagnosis, independent findings, and causal skips.
Failure outranks incomplete, which outranks pass: only all-pass input passes.
There is no waiver, score, baseline, deduplication, majority rule, or severity
override.

All-pass exits `0`, any failure exits `1`, and otherwise incomplete exits `3`.
Invalid, unreadable, aliased, over-limit input or an unsafe destination exits
`2`; render, write, publication, or cleanup failure exits `4` and leaves no
artifact.

`aggregate` never starts a process, opens a connection, resolves DNS or
credentials, retrieves a schema, discovers a target, or calls a tool. It does
not scan directories, expand globs, infer CI artifacts, or follow report paths.
Fixed limits cover each file, total input, nesting, nodes, validation work,
checks, findings, output, and operation time. Duplicate, aliased, hard-linked,
symlinked, malformed, inconsistent, or over-limit input rejects the invocation.
The destination must be a new file in an existing directory and is staged and
published without overwrite.

The stable aggregate contract is published with the source as
[`mcp-doctor.aggregate/v1`](schemas/mcp-doctor.aggregate.v1.schema.json). Its
member reports use the separate
[`mcp-doctor.report/v1`](schemas/mcp-doctor.report.v1.schema.json) contract;
offline validators should load both local schemas and must not retrieve either
at runtime.

### Reviewed `check` scenarios

`check` accepts only a regular file containing strict
`mcp-doctor.scenario/v1alpha1` JSON. One scenario names one exact tool, declares
its effect classification, and contains 1–100 cases that run sequentially in
array order:

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

### Generated `break` cases

`break` derives schema-valid object inputs only from one bounded local Draft
2020-12 tool schema. Matching `--tool` and `--allow-tool` values select and
authorize it; the run also declares `read_only` or `side_effecting`, 1–100
cases, and an unsigned 64-bit seed. A `side_effecting` run also requires `--allow-side-effects`.
Annotations, wildcards, patterns, and discovery never grant authority.

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
widen the authorized endpoint. Ordinary reports never contain raw generated arguments or tool results.

## Findings you can act on

Every finding includes a stable code, severity, MCP revision, safe field
location, and performed or skipped state. Active reports retain the declared
case index or generated seed and a structural input shape—enough to repeat a
failure without secrets or raw production data. Related problems identify the
first actionable cause, skip only dependent checks with a reason, and keep
independent checks running.

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

The passive STDIO path is checked against pinned official TypeScript and Go
servers and independent Dart and PHP servers. See the
[compatibility evidence](tests/compatibility/README.md) for the exact scope and
results. That broad real-server evidence covers MCP `2026-07-28` only. Explicit
MCP `2025-11-25` active STDIO diagnostics additionally pass the pinned official
Go and independent PHP cases; their Streamable HTTP behavior and all passive
legacy behavior remain synthetic. Neither legacy revision carries a broad
ecosystem or published installed-channel claim.

## Bring it into CI

Run the same check in a pull request. Its exit remains the gate while one run
produces stable JSON and JUnit:

```yaml
- name: Diagnose MCP server
  run: >-
    mcp-doctor inspect
    --json-report artifacts/mcp-doctor.json
    --junit-report artifacts/mcp-doctor.xml
    --
    ./target/release/my-mcp-server --stdio
```

A failed required check returns non-zero. Your CI system uploads or consumes
the explicit paths; `mcp-doctor` performs no provider-specific upload. Each
JUnit diagnostic check becomes one test case, while JSON and JUnit preserve
stdout's outcome, primary diagnosis, causal skips, and safe evidence. Reports
remain deterministic, bounded, and secret-free.

## Safe by default

- `inspect` checks advertised contracts without calling a tool.
- Active runs name and independently authorize one exact tool and target,
  declare effects and case limits, and add a seed for generation. Side effects
  require `--allow-side-effects`.
- Remote connections use direct public HTTPS, verified TLS, and pinned bounded
  resolution without redirects, retries, proxies, cookies, or caches.
- Private targets, loopback cleartext, and environment credentials each require
  an exact endpoint gate. Credentials never use HTTP or trigger OAuth or
  metadata discovery.
- Hard limits cover time, bytes, messages, schema work, cases, retries,
  redirects, and concurrency.
- Legacy HTTP session IDs come only from initialization, stay bounded and
  run-local, repeat exactly, and receive one bounded teardown. Session loss
  never reinitializes or downgrades; teardown failure stays visible.
- Reports hide headers, credentials, tool inputs, raw results, and server logs.
- Sensitive snapshots require an exact-path acknowledgement and new file;
  value-free offline diffs have no target or network surface.
- Aggregates accept only explicit bounded stable reports, discard unknown
  optional values, preserve failures, and perform no target, network,
  credential, retrieval, or tool activity.
- Capability discovery reports only fixed compiled facts under 64 KiB and
  reads no configuration, host inventory, credentials, files, process,
  network, target, retrieval, or tool data.
- Local commands bypass the shell. Before exit, `mcp-doctor` closes, stops when
  needed, and waits for every child process.

## Contributing

Contributions are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md), ask
for help through [SUPPORT.md](SUPPORT.md), follow the
[Code of Conduct](CODE_OF_CONDUCT.md), and report suspected vulnerabilities
privately as described in [SECURITY.md](SECURITY.md). The
[project scope](docs/project-scope.md) identifies every `mcp-doctor` repository
and official distribution or community channel.

## License

Licensed under the [MIT License](LICENSE).
