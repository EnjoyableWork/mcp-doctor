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
  <a href="#inspect-check-break-diff">Inspect. Check. Break. Diff.</a> ·
  <a href="#bring-it-into-ci">CI</a> ·
  <a href="#safe-by-default">Safety</a>
</p>

```console
$ mcp-doctor inspect -- node ./dist/weather-server.js

  mcp-doctor  weather-server · STDIO

  PASS  protocol       MCP 2026-07-28 supported
  PASS  discovery      8 tools · 2 resources · 1 prompt
  FAIL  tool/schema    weather_forecast.inputSchema.required
        expected an array of unique strings, found a string
  SKIP  tool/runtime   passive inspection; no tools called

  1 failed · 18 passed · 8 skipped                         exit 1
```

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
cargo install mcp-doctor --version '=0.2.0' --locked
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

`inspect` uses MCP `2026-07-28` by default. Select either supported handshake
revision explicitly when diagnosing a server that has not migrated:

```bash
# Passive legacy STDIO inspection
mcp-doctor inspect --protocol-version 2025-11-25 -- node ./dist/server.js

# Passive legacy Streamable HTTP inspection
mcp-doctor inspect --protocol-version 2025-06-18 https://mcp.example.com/mcp
```

Revision selection never auto-detects, retries, falls back, or downgrades.
Legacy inspection performs only `initialize`, one
`notifications/initialized`, and capability-advertised `tools/list`,
`prompts/list`, `resources/list`, and `resources/templates/list` operations. It
does not call tools, list retained tasks, read resources, get prompts, answer
server requests, or enable legacy `check` or `break` behavior.

| MCP revision | `inspect` STDIO | `inspect` Streamable HTTP | `check` / `break` | Evidence position |
| --- | --- | --- | --- | --- |
| `2026-07-28` | Default | Default | Supported | Broad current-revision matrix |
| `2025-11-25` | Explicit only | Explicit only | Not supported | Synthetic diagnostics only |
| `2025-06-18` | Explicit only | Explicit only | Not supported | Synthetic diagnostics only |
| `2025-03-26`, `2024-11-05`, or unknown | Rejected | Rejected | Rejected | No compatibility claim |

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

## Inspect. Check. Break. Diff. Aggregate.

Choose how much activity the target allows:

| Command | Activity | Use it to |
| --- | --- | --- |
| **`inspect`** | Does not call tools | Connect, list what the server offers, and check its definitions and schemas |
| **`check`** | Calls selected tools | Run known inputs from a scenario you wrote and check the results |
| **`break`** | Tries generated edge cases | Search one selected tool for failures you can repeat |
| **`diff`** | Reads two local files only | Compare explicitly captured advertised contracts without starting or contacting a target |
| **`aggregate`** | Reads explicit local files only | Combine stable diagnostic reports conservatively without rerunning or contacting a target |

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
```

> [!CAUTION]
> `check` and `break` execute real tool calls. Use disposable data and a test
> environment. Finding a tool does not give `mcp-doctor` permission to call it.

### Contract snapshots and offline diffs

Contract snapshots are separate, explicitly requested developer artifacts—not
ordinary diagnostic reports. Create one only during passive MCP `2026-07-28`
inspection by naming the same exact new path twice:

```bash
mcp-doctor inspect \
  --snapshot build-a.contract.json \
  --allow-sensitive-snapshot build-a.contract.json \
  -- node ./dist/server.js --stdio
```

The redundant path is an acknowledgement that the artifact is sensitive. A
snapshot contains normalized capability settings, tool and prompt names,
advertised resource URIs and URI templates, prompt argument names, tool
behavior hints, JSON Schema property names, and validation-bearing values such
as `const`, `enum`, patterns, and bounds. Those fields may disclose proprietary
API shape, internal naming, or allowed values. Store the file with the same
care as source containing private interfaces, choose a retention period, and
review it before sharing. `mcp-doctor` creates one new regular file with mode
`0600` on Unix, never overwrites an existing path, and has no force option.

Descriptions, titles, defaults, examples, comments, server identity,
instructions, pagination and cache metadata, transport endpoints, DNS and peer
data, credentials and their source names, request headers, arguments, runtime
results, logs, and stderr are not retained. A snapshot requested with an
ordinary report is assembled from that exact bounded discovery conversation;
there is no second run or list request. A complete current-revision catalog can
still be written when a bounded local schema shape retained by the artifact
makes the report fail, so a redacted location such as `tools[73]` can be
resolved through the artifact-local ordinal map. Transport, protocol,
external-reference, resource-bound, unrepresentable or incomplete-catalog, and
cleanup failures write no snapshot.

Compare two artifacts without a target:

```bash
mcp-doctor diff before.contract.json after.contract.json
mcp-doctor diff --format json before.contract.json after.contract.json
```

`diff` reads exactly two bounded regular files and never starts a process,
opens a connection, retrieves a schema, or calls a tool. Catalog and set-like
schema ordering are normalized, and each snapshot's ordinal map is validated
but ignored for semantic matching. Human and
`mcp-doctor.contract-diff/v1alpha1` JSON output use stable codes for additions,
removals, capability changes, required inputs, and a finite set of syntactic
narrowing or widening rules. Other schema and behavior-hint changes are
`review_required`; `mcp-doctor` does not claim general JSON Schema implication,
universal compatibility, protocol conformance, or a health score. Unchanged or
documented-compatible diffs exit `0`, potentially breaking or review-required
diffs exit `1`, and invalid, over-limit, or unreadable artifacts exit `2`.

The artifact contracts are published with the source as
[`mcp-doctor.contract-snapshot/v1alpha1`](schemas/mcp-doctor.contract-snapshot.v1alpha1.schema.json)
and
[`mcp-doctor.contract-diff/v1alpha1`](schemas/mcp-doctor.contract-diff.v1alpha1.schema.json).

### Offline diagnostic aggregates

Use `aggregate` when separate jobs or platforms have already produced stable,
redacted `mcp-doctor.report/v1` JSON files. It accepts one through 32 explicit
ordered regular files and writes one new `mcp-doctor.aggregate/v1` JSON
artifact. Human stdout is the default; `--format json` makes stdout
byte-identical to that required artifact.

Members are identified only by zero-based input ordinal. Every known safe
report field is retained, while compatible unknown optional properties are
accepted and discarded instead of echoed. Each input must satisfy the embedded
stable report schema and consistent summary, severity, outcome, exit,
revision, primary-diagnosis, independent-finding, and causal-skip semantics.
Any failed member fails the aggregate; otherwise any incomplete member makes it
incomplete; only all-pass input passes. There is no waiver, score, baseline,
deduplication, majority rule, or severity override.

An all-pass aggregate exits `0`, any failed member exits `1`, and otherwise an
incomplete member exits `3`. Invalid, unreadable, aliased, or over-limit input
and unsafe destinations exit `2`; render, write, publication, or cleanup
failure exits `4` and leaves no aggregate artifact.

`aggregate` never starts a process, opens a connection, resolves DNS or
credentials, retrieves a schema, discovers a target, or calls a tool. It does
not scan directories, expand globs, infer CI artifacts, or follow paths found
inside a report. Per-file, total-input, nesting, node, validation-work, check,
finding, output, and operation-time limits are fixed in the artifact. Duplicate,
canonical-alias, hard-link, symbolic-link, malformed, inconsistent, or
over-limit inputs reject the whole invocation. The required destination must
be a new file in an existing directory and is staged and published without
overwrite.

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

The case ID is for reviewing the file; reports use only its numeric index.
`target_env` copies only those same-named invoking-process variables into the
otherwise constrained target environment. Each `secret_refs` key is an RFC
6901 pointer to an existing `null` argument placeholder, and its value names an
invoking-process environment variable. Missing values, invalid pointers,
non-null destinations, duplicate members, unknown fields, and unsupported
schemas fail before the target starts. There is no interpolation, `.env` or
file loading, command execution, prompt, keychain, or secret-store lookup.
Environment reference names use the portable ASCII form
`[A-Za-z_][A-Za-z0-9_]*`; argument-secret values must be valid UTF-8 because
they become JSON strings.

Every run repeats the exact scenario tool with `--allow-tool`; a
`side_effecting` scenario also requires `--allow-side-effects`. Advertised tool
annotations never grant permission. Arguments must match the advertised input
schema before a call. Completed results are checked against the expected
`success` or `tool_error`, the advertised output schema, and the optional
scenario output schema. Ordinary mismatches and tool rejections do not hide
later cases. Transport, protocol, cleanup, authorization, and exhausted-limit
failures stop later calls. `input_required` makes that case and an otherwise
successful report incomplete; `mcp-doctor` neither supplies input nor retries
that call.

### Generated `break` cases

`break` derives schema-valid object inputs only from the one selected tool's
bounded local Draft 2020-12 input schema. The invocation must name that tool
independently with matching `--tool` and `--allow-tool` values, classify it as
`read_only` or `side_effecting`, choose 1–100 cases, and supply an unsigned
64-bit seed. A `side_effecting` run also requires `--allow-side-effects`;
advertised annotations, wildcards, patterns, and discovered selection never
grant authority.

`mcp-doctor.generator/v1` builds a finite candidate set, validates every
candidate against the advertised schema, and runs selected cases sequentially
with concurrency one. Generation is capped at 256 attempts, 64 retained
candidates, 100,000 synthesis steps, 1 MiB per input, and 8 MiB across active
inputs. Schema depth, local-reference, validation, transport, response, and
total-run bounds still apply. An invalid, externally referenced,
unsatisfiable, or over-limit schema stops generation before `tools/call`.

Each case reports its generator version, case seed, serialized byte count, and
fixed structural JSON counts without retaining member names or values. Case
`n` uses the base seed plus `n` with unsigned wraparound. To replay one case,
run the same tool and advertised schema with that reported seed and
`--cases 1`. Tool errors remain findings and do not hide later cases;
`input_required`, unsafe failures, and cleanup retain the same stop rules as
`check`.

Generated runs do not accept target-environment or argument-secret sources,
fetch schemas, choose another tool, change the local executable, or widen the
exact remote endpoint authorized by the command. Ordinary human and JSON
reports never contain raw generated arguments or tool results.

## Findings you can act on

Every finding includes a stable code, severity, MCP version, safe field
location, and whether the check ran or was skipped. Active reports keep the
declared case index or generated seed and the structural input shape needed to
repeat a failure without revealing secrets or raw production data.

When problems are connected, the report points to the first one you can fix.
It skips only the checks that depend on that problem and tells you why. It
keeps running unrelated checks and reports their problems too.

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

The human, stable JSON, and JUnit reports use the same immutable findings. This
prevents CI from hiding a failure, choosing a different main issue, or turning
a skipped check into a pass.

The passive STDIO path is checked against pinned official TypeScript and Go
servers and independent Dart and PHP servers. See the
[compatibility evidence](tests/compatibility/README.md) for the exact scope and
results. That broad real-server evidence covers MCP `2026-07-28` only. Explicit
MCP `2025-11-25` and `2025-06-18` diagnostics have synthetic STDIO and
Streamable HTTP evidence; they do not yet carry a broad ecosystem or published
installed-channel claim.

## Bring it into CI

Run the same check in a pull request. The process exit remains the gate while
stable JSON and JUnit files are produced from that one run:

```yaml
- name: Diagnose MCP server
  run: >-
    mcp-doctor inspect
    --json-report artifacts/mcp-doctor.json
    --junit-report artifacts/mcp-doctor.xml
    --
    ./target/release/my-mcp-server --stdio
```

A required check that fails returns a non-zero status. Upload or consume the
explicit paths with the facilities of any CI system; `mcp-doctor` does not
perform provider-specific uploads. Each JUnit diagnostic check becomes one test
case, and JSON and JUnit retain the same outcome, primary diagnosis, causal
skips, and safe evidence as stdout. Reports are deterministic, bounded, and
hide secrets.

## Safe by default

- `inspect` lists and checks what the server offers; it never calls a tool.
- Every active run names and independently authorizes the exact tool and
  target. Reviewed scenarios declare their effect and case limit; generated
  runs also declare their effect, case limit, and seed.
- Side-effecting active runs require a separate `--allow-side-effects` gate.
- Remote connections default to direct public HTTPS with verified TLS, pinned
  bounded address resolution, and no redirect, retry, proxy, cookie, or cache.
- Private targets, loopback cleartext, and environment-sourced credentials each
  require their own exact endpoint gate; credentials never travel over HTTP or
  trigger an automatic OAuth or metadata flow.
- Hard limits cover time, data size, messages, schema work, test cases,
  redirects, retries, and parallel work.
- A legacy HTTP session identifier is accepted only from initialization,
  bounded and retained only for the run, repeated exactly on later requests,
  and followed by one bounded teardown attempt. Session loss never triggers a
  reinitialize or downgrade, and teardown failure remains independently
  visible.
- Normal output hides headers, credentials, tool inputs, raw results, and server
  logs.
- Sensitive contract snapshots require an exact-path acknowledgement and a new
  output file. Offline diffs remain value-free and have no target or network
  surface.
- Offline aggregates accept only explicit bounded stable reports, discard
  unknown optional values, preserve failures conservatively, and have no
  target, network, credential, retrieval, or tool surface.
- Local server commands run directly, not through a shell. Before exiting,
  `mcp-doctor` closes every child process, stops it if needed, and waits for it
  to end.

## Contributing

Contributions are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md), ask
for help through [SUPPORT.md](SUPPORT.md), follow the
[Code of Conduct](CODE_OF_CONDUCT.md), and report suspected vulnerabilities
privately as described in [SECURITY.md](SECURITY.md). The
[project scope](docs/project-scope.md) identifies every `mcp-doctor` repository
and official distribution or community channel.

## License

Licensed under the [MIT License](LICENSE).
