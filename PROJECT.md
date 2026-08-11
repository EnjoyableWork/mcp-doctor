# mcp-doctor project plan

This is the living source for product scope, delivery status, ordered work,
decisions, risks, and release gates.

| Control | Current state |
| --- | --- |
| Document state | Active |
| Product state | The passive local STDIO MVP and pinned current-revision compatibility matrix pass locally and in hosted evidence; bounded local and Streamable HTTP `check` plus deterministic `break` paths pass locally but are not yet in a public release; stable CI reporting and the expanded release remain unimplemented |
| Current milestone | M3 — bounded diagnostic expansion; `MCPD-011` is Done locally and `MCPD-012` remains Proposed |
| Overall status | M0, M1, and M2 pass locally and in hosted evidence; the immutable `v0.1.0` channels and least-privilege repeat-release path remain verified; `MCPD-009` through `MCPD-011` complete three locally verified M3 slices without changing published artifacts or claiming hosted active, remote, or generated evidence |
| Current focus | Preserve the completed `DEC-031` generation boundary; resolve `OPEN-07` before activating `MCPD-012` stable reporting, publication, and independent verification |
| Public release | `mcp-doctor` `v0.1.0` — immutable GitHub Release, crates.io, and `EnjoyableWork/tap/mcp-doctor` verified |
| Last reviewed | 2026-08-11 |
| Next review trigger | `MCPD-012` activation or resolution of `OPEN-07`; any voluntary usage evidence that changes M3 priority; any trusted-publisher, tap-authority, release-pipeline, dependency, testing-tool, safety-boundary, or assurance-evidence change |

## Document roles

| Artifact | Role |
| --- | --- |
| [README.md](README.md) | North-star product page describing the intended finished experience |
| [PROJECT.md](PROJECT.md) | Current implementation truth, delivery sequence, decisions, and evidence |
| [AGENTS.md](AGENTS.md) | Durable engineering, safety, testing, and handoff constraints |
| Code, tests, and releases | Authority for implemented and published behavior |

The README is not an implementation checklist. A feature described there is
not delivered until the ticket board links code and test evidence. Keep current
limitations here rather than weakening the product story.

Public assurance claims are different from product destination language. A
security or trust statement, badge, conformance level, or framework alignment
may describe only achieved, dated, scoped, and currently verified evidence. It
must be corrected or removed when that evidence becomes stale.

## Operating model

1. Work on the earliest incomplete main-story ticket whose predecessor is
   done and whose decisions and evidence are clear.
2. Keep one main-story ticket in progress. Optional work may run only when it
   cannot delay, redefine, or become a prerequisite for that story.
3. Give each material change one observable outcome and record durable evidence
   before marking it done.
4. Update affected decisions, risks, status, and documentation in the same
   change as implementation.
5. Do not jump past a blocked story ticket. Resolve its named blocker or accept
   a decision that replaces it at the same point in the sequence.
6. Treat Git as history; update this file in place instead of appending a diary.

When a hosted issue tracker becomes useful, it may own discussion and task
detail. This file remains the repository-level milestone and decision index.

## Status model

| Status | Meaning |
| --- | --- |
| Proposed | Useful outcome, but not yet eligible to start |
| Ready | Scope, predecessor, decisions, and evidence are clear |
| In progress | An owner is actively delivering the ticket |
| Blocked | A named external decision, dependency, or authority prevents progress |
| Done | Acceptance evidence exists and documentation is accurate |
| Deferred | Intentionally outside the current release |
| Superseded | Replaced by a named accepted decision and ticket |
| Cancelled | No longer intended, with the reason recorded |

## Main story

| Arc | Required order | Outcome |
| --- | --- | --- |
| M0 — Foundation | `MCPD-001` → `MCPD-002` → `MCPD-003` | Original repository contract, runnable binary, and enforced quality baseline |
| M1 — Passive local MVP | `MCPD-004` → `MCPD-005` → `MCPD-006` → `MCPD-007` | One actionable, agent-readable STDIO preflight that never invokes a tool |
| M2 — Public MVP release | `MCPD-008` → `MCPD-008A` | The passive MVP installed and independently verified through every advertised channel, followed by a rehearsed least-privilege path for every later release |
| M3 — Bounded diagnostic expansion | `MCPD-009` → `MCPD-010` → `MCPD-011` → `MCPD-012` | Explicitly authorized and bounded active, remote, adversarial, and CI capabilities, followed by one independently verified expanded release |
| M4 — Enterprise assurance and adoption | `MCPD-013` → `MCPD-014` → `MCPD-015` → `MCPD-016` → `MCPD-017` → `MCPD-018` | Contributor-compatible governance, repository and organization controls, supply-chain evidence, and a public scoped assurance baseline |

Signed native macOS and Windows artifacts are a later candidate, not part of
the first public release. They require an accepted funding and signing decision
plus native installed evidence.

`MCPD-008A` began only after the first crates.io publication completed. It did
not delay, reopen, or replace the immutable `v0.1.0` release. Its completed
controls now gate every later public version. A reviewed release change and an
intentionally created annotated tag remain human release authority; automation
may publish that approved version but may not choose one.

M3 is activated by the dated `DEC-027` owner decision now that M2 and
`MCPD-008A` are complete. Independent adoption evidence is useful prioritization
input but is not a prerequisite whose external timing can indefinitely block
planned feature work. Each M3 ticket must still satisfy its predecessor,
resolve its own design and safety decisions, preserve the north star, and pass
its acceptance evidence. Voluntary feedback may reprioritize, narrow, defer, or
cancel later work, but its absence alone does not stop the ordered story.

M4 begins only after the expanded M3 release is independently verified. It
does not delay or reopen M3, and it does not turn a self-assessment into a
warranty, independent certification, regulatory-compliance claim, or support
SLA.

## Product outcome

`mcp-doctor` gives an MCP server author a deterministic way to inspect a local
or remote server, find protocol and schema defects, reproduce runtime failures,
and distinguish what was actually tested from what was skipped.

The proof is not that the CLI can parse one response. The proof is that a valid
synthetic server passes; malformed schemas, crashes, timeouts, oversized output,
invalid results, and cleanup failures produce precise non-zero outcomes; the
same seed reproduces an active finding; and no default path surprises a real
system with tool execution.

### Product north star

> `mcp-doctor` is a safe, noninteractive server-author preflight that identifies
> the earliest actionable failing layer, explains it precisely, suggests a
> corrective action, and emits evidence both a human and an AI agent can trust.

This promise governs the MVP and every later transport, active mode, reporter,
and release. Diagnostic breadth is subordinate to causal clarity: another
check, score, or surface is useful only when it preserves or improves the path
from one command to a trustworthy corrective action.

When one failure blocks dependent work, the result identifies the earliest
actionable diagnostic layer and its primary finding or findings. Downstream
checks are skipped with an explicit reference to that blocking diagnosis
rather than repeated as unrelated symptoms. Independent failures remain
visible, and a critical cleanup, redaction, authorization, or other safety
failure is never demoted or hidden to manufacture a single simple cause.

The MVP expression of this promise is one local STDIO command.
`mcp-doctor inspect` must answer whether the server starts cleanly, speaks a
supported MCP revision, advertises structurally usable contracts, remains
within diagnostic limits, and shuts down safely — without invoking a tool.

A failed result is useful only when it tells the developer what failed, where
it failed, why it matters, what the contract expected, and what to change next.
The developer should not need to inspect raw JSON-RPC traffic, launch a browser
tool, read source code, or expose an untrusted value to act on an ordinary
finding.

The MVP is a deterministic diagnostic preflight. It is not an interactive MCP
client, a replacement for the official conformance suite, a general security
scanner, an LLM evaluation platform, or an autonomous tool caller. Those
products may consume or complement its evidence; they do not widen the MVP.

#### North-star acceptance

- **Causal accuracy:** every golden failure fixture names its expected earliest
  actionable layer; the built binary selects that layer without allowing a
  downstream symptom to outrank it.
- **Report sufficiency:** a reviewer given only the ordinary report can recover
  the safe what, where, why, expectation, and corrective next step without raw
  traffic, stderr, source, or arbitrary observed values.
- **Human-agent parity:** human and machine reports designate the same primary
  layer and findings, independent findings, causal skips, limits, outcome, and
  exit status.
- **Safety:** reaching an answer never weakens authorization, redaction,
  resource bounds, cleanup, or honest performed/skipped semantics.
- **Anti-drift:** a feature does not satisfy a milestone merely by increasing
  check count; it must preserve or improve time to a trustworthy diagnosis and
  must not turn the ordinary report into an undifferentiated failure list.

### Product principles

- **Causes before cascades:** identify the earliest actionable failing layer,
  preserve independent safety failures, and explain causal skips explicitly.
- **Passive first:** inspection may discover and validate, but active tool
  calls require explicit authorization.
- **Bounded by construction:** time, bytes, messages, schema work, cases,
  redirects, retries, concurrency, and cleanup all have enforceable limits.
- **One result model:** human and machine output report the same findings,
  redaction, skips, and outcome.
- **Versioned diagnosis:** every rule names the MCP revision or compatibility
  range it evaluates.
- **Reproducible pressure:** active cases retain the seed and structural input
  needed to reproduce a failure.
- **Evidence before claims:** support is a tested revision, transport, platform,
  and installed artifact rather than an aspiration.
- **Narrow before extensible:** build one vertical journey, then introduce an
  interface only when a second real transport, revision, or reporter needs it.

## Growth model

| Milestone | Exit gate | State |
| --- | --- | --- |
| M0 | Clean checkout builds; help/version work; format, Clippy, tests, dependency policy, and hosted CI pass | Done |
| M1 | One built-binary `inspect` journey identifies the earliest actionable failing layer and report-only correction in redacted human and experimental JSON for synthetic fixtures and a representative real-server compatibility matrix without calling a tool | Done |
| M2 | One immutable passive-MVP release installs and passes its diagnostic smoke journey through every advertised channel; its least-privilege repeat-release path is rehearsed | Done |
| M3 | Every retained expansion is explicitly authorized and bounded; inherited safety and stable CI output remain intact; one expanded immutable release passes every retained journey | In progress |
| M4 | The selected current OpenSSF OSPS Baseline Level 1 controls pass with dated public evidence and official self-certification proof; exact release-artifact provenance is separately evaluated against the selected current SLSA Build L2 requirements | Proposed |

Each milestone must leave the preceding milestone working. Expansion does not
justify weakening cleanup, redaction, determinism, or active-execution consent.
Assurance work may verify an immutable release but must never rewrite it or
publish a broader claim than its evidence supports.

## Distribution contract

The accepted public identity is MCP Doctor, the
`EnjoyableWork/mcp-doctor` repository, and the installed `mcp-doctor`
executable. An independently owned scoped npm package also installs an
`mcp-doctor` command; that cross-ecosystem executable collision is accepted.
Official metadata and installation guidance distinguish this project as the
Rust CLI from EnjoyableWork without turning the marketing page into a
competitor comparison.

`MCPD-008` rechecks every selected registry immediately before publication. If
the unscoped crates.io name remains available, the Cargo package is
`mcp-doctor`. If it is unavailable, the package becomes
`enjoyable-mcp-doctor` while the installed executable, product, repository, and
Homebrew formula remain `mcp-doctor`. Availability is not reserved or implied
before publication.

The first zero-cost release target is:

| Channel | Initial contract |
| --- | --- |
| GitHub Releases | Immutable release with native GNU/Linux ARM64/x64 archives, `SHA256SUMS`, target SPDX SBOMs, provenance, source package, and Homebrew formula |
| crates.io | `mcp-doctor` if available at publication, otherwise `enjoyable-mcp-doctor`; either exact release source package installs the `mcp-doctor` executable and is smoke-tested on supported macOS, GNU/Linux, and Windows hosts |
| Homebrew | `EnjoyableWork/tap/mcp-doctor`, built from the immutable source package and tested on supported macOS and GNU/Linux hosts |

There is no first-release WinGet package and no project-issued macOS or Windows
binary. A later signed-native release requires Developer ID signing and
notarization for macOS, Public Trust Authenticode for Windows, all represented
native archive smokes, and a new version. Never replace immutable release bytes
or move a published version tag.

## M1 boundary

| Dimension | M1 contract |
| --- | --- |
| Runtime | One Rust command-line binary |
| Transport | Local STDIO process |
| Current protocol | MCP `2026-07-28` |
| Earlier revisions | Recognize `2025-11-25`, `2025-06-18`, `2025-03-26`, and `2024-11-05` as unsupported legacy revisions; never fall back or send `initialize` |
| Default activity | `inspect`: discovery and structural validation only; no implicit tool call |
| Active activity | Outside M1; bounded reviewed `check` replay and deterministic generated `break` cases are implemented under `MCPD-009` and `MCPD-011`, and neither changes the passive M1 default |
| Schemas | JSON Schema 2020-12 under bounded local evaluation; no external retrieval by default |
| Findings | Earliest actionable layer and primary finding or findings, independent safety findings, typed code, severity, safe location/context, causally linked performed/skipped state, overall outcome, safe expectation, remediation, and specification reference |
| Output | Redacted human report plus public experimental `mcp-doctor.report/v1alpha1` JSON with the same primary diagnosis, independent findings, causal skips, and result; stabilization and additional CI formats remain M3 |
| Process policy | Literal executable/arguments, constrained environment, bounded I/O and time, full cleanup and reap |
| Test data | Synthetic fixture servers in the default suite plus a controlled, non-mutating compatibility matrix of official SDK examples and independent implementations |
| Compatibility positioning | Broad current-revision positioning only when all selected current-revision official and independent cases pass across at least two implementation languages; otherwise use explicit readiness/migration positioning and a separate compatibility ticket if at least one credible independent implementation passes; no credible independent pass blocks M1 |
| Distribution | Source checkout and packaged-artifact validation in M1; first public packages are M2 |

### MCPD-004 diagnostic contract

The revision contract is pinned to the official
[MCP `2026-07-28` release](https://github.com/modelcontextprotocol/modelcontextprotocol/releases/tag/2026-07-28)
and its tagged
[versioning rules](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/2026-07-28/docs/specification/2026-07-28/basic/versioning.mdx).
The tagged schema requires per-request protocol metadata and
`server/discover` for this revision; the tagged versioning rules classify
`2025-11-25` and earlier as handshake-based legacy behavior. MCPD-004 defines
how those facts are modeled; it does not send a discovery request or start a
server.

#### Revision matrix

| Advertised revision value | Era | Support | Required diagnostic behavior |
| --- | --- | --- | --- |
| `2026-07-28` | Modern per-request metadata | Supported | Select this exact revision |
| `2025-11-25` | Legacy `initialize` handshake | Recognized, unsupported | Report `MCP-PROTOCOL-002`; never fall back or send `initialize` |
| `2025-06-18` | Legacy `initialize` handshake | Recognized, unsupported | Report `MCP-PROTOCOL-002`; never fall back or send `initialize` |
| `2025-03-26` | Legacy `initialize` handshake | Recognized, unsupported | Report `MCP-PROTOCOL-002`; never fall back or send `initialize` |
| `2024-11-05` | Legacy `initialize` handshake | Recognized, unsupported | Report `MCP-PROTOCOL-002`; never fall back or send `initialize` |
| Any other string | Unknown | Unsupported | Treat a valid date-shaped value as safe structured metadata; otherwise retain only a redaction marker and byte count; report `MCP-PROTOCOL-002` |
| Missing or non-string value | Invalid message shape | Unsupported | Retain only structural location and redacted byte count; report `MCP-PROTOCOL-003` |

Server advertisements select `2026-07-28` if and only if that exact value is
present. An empty list, only legacy values, or only unknown values has no mutual
revision. Unknown strings are not called malformed because the official schema
defines the revision as a string without a date-pattern constraint.

#### Finding and check semantics

The code registry began with stable meanings and code-owned severities in
`MCPD-004`. `MCPD-005` wires the transport findings, `MCPD-006` wires the
catalog and schema findings below, and `MCPD-007` designates the earliest
actionable layer while preserving independent safety failures and reporter
parity.

| Code | Severity | Reserved meaning |
| --- | --- | --- |
| `MCP-TRANSPORT-001` | Error | The MCP server process could not be started |
| `MCP-TRANSPORT-002` | Error | A managed STDIO channel failed before diagnosis completed |
| `MCP-TRANSPORT-003` | Error | The server wrote an invalid bounded STDIO message |
| `MCP-TRANSPORT-004` | Error | The server exited before returning the discovery response |
| `MCP-PROTOCOL-001` | Info | The requested protocol revision is supported |
| `MCP-PROTOCOL-002` | Error | The server does not advertise the required revision |
| `MCP-PROTOCOL-003` | Error | The revision value is missing or has the wrong JSON type |
| `MCP-PROTOCOL-004` | Warning | A feature is deprecated by the selected revision |
| `MCP-LIMIT-001` | Error | A configured diagnostic safety limit is exceeded |
| `MCP-SAFETY-001` | Critical | A managed target cannot be fully cleaned up |
| `MCP-CATALOG-001` | Error | An advertised catalog response or item violates the `2026-07-28` structural contract |
| `MCP-CATALOG-002` | Error | An advertised identifier is duplicated within its catalog scope |
| `MCP-CATALOG-003` | Error | A pagination cursor repeats instead of advancing or ending the catalog |
| `MCP-SCHEMA-001` | Error | A local advertised or scenario-provided JSON Schema contract is invalid |
| `MCP-SCHEMA-002` | Error | A local schema declares a dialect outside the Draft 2020-12 contract |
| `MCP-SCHEMA-003` | Error | A schema would require prohibited external reference retrieval |
| `MCP-SCENARIO-001` | Error | A scenario violates the strict `mcp-doctor.scenario/v1alpha1` structure |
| `MCP-SCENARIO-002` | Error | A target-environment or argument secret reference cannot resolve safely |
| `MCP-SCENARIO-003` | Error | A scenario-provided local output schema is invalid |
| `MCP-GENERATION-001` | Error | Bounded schema-valid object inputs cannot be generated for the selected tool |
| `MCP-AUTH-001` | Error | `--allow-tool` does not authorize the selected exact tool |
| `MCP-AUTH-002` | Error | A `side_effecting` active run lacks `--allow-side-effects` |
| `MCP-ACTIVE-001` | Error | The exact selected tool is not advertised uniquely |
| `MCP-ACTIVE-002` | Error | An active case fails advertised input-schema validation and is not called |
| `MCP-ACTIVE-003` | Error | The server rejects an active tool call at the JSON-RPC layer |
| `MCP-ACTIVE-004` | Error | A completed result disagrees with the declared or generated expectation |
| `MCP-ACTIVE-005` | Error | `structuredContent` violates an advertised or scenario-provided output schema |
| `MCP-ACTIVE-006` | Error | A tool response violates the current-revision result envelope and stops later calls |

Each check has a stable ID, is `required` or `optional`, and is exactly one of:

- `performed`, with a canonically sorted and deduplicated finding list; or
- `skipped`, with a typed reason and no findings.

A performed check is `failed` when any finding is Error or Critical, `warning`
when it has Warning findings but no failures, and `passed` otherwise. A report
is `failed` when any performed check fails, even if another required check was
skipped. With no failure it is `incomplete` when the report declares no
required checks, no check was performed, or a required check was skipped.
Otherwise it is `passed`; optional skips and warnings remain visible but do not
change that outcome.

#### Exit and report semantics

| Exit | Meaning |
| --- | --- |
| `0` | Passed diagnostic report |
| `1` | Diagnostic failure from Error or Critical findings |
| `2` | Invalid invocation or configuration |
| `3` | Incomplete evidence because no required check was declared, no check ran, or a required check was skipped |
| `4` | Internal invariant or reporter failure |

Human and JSON reporters derive from the same immutable result. Reports reject
an empty check set, duplicate check IDs, and findings for a different revision;
they sort checks and findings canonically. Every finding includes its code,
code-owned severity, selected revision, trusted structural location, static
message and impact, safe expectation, corrective next step, versioned
reference, and typed evidence. Arbitrary values, identifiers, paths, payloads, headers,
arguments, results, and logs cannot enter the ordinary result model;
observations retain only a safe JSON type or `[REDACTED]` and a byte count.

The CLI exposes `mcp-doctor.report/v1alpha1` through `inspect --format json`
and `check --format json`.
The experimental envelope includes stability, revision, primary diagnosis,
independent findings, exact limits, derived summary, performed/skipped checks,
causal `blocked_by` evidence, findings, outcome, and exit code. It is
fixture-tested but is not a stable compatibility promise; `MCPD-012` owns the
stable machine format.

#### M1 default limit profile

These limits are finite, appear in every report, and apply simultaneously; the
first exhausted bound stops the affected work. A MiB is exactly 1,048,576
bytes. Request time bounds writing one complete request; response time bounds
waiting for and reading its complete response. Operation-specific and total
time bounds also continue to apply.

| Area | Default bounds |
| --- | --- |
| Time | Startup 10,000 ms; discovery 10,000 ms; request 30,000 ms; response 30,000 ms; shutdown grace 2,000 ms; total run 120,000 ms |
| I/O | One message 1 MiB; stdout 8 MiB; stderr 1 MiB; combined output 8 MiB; 1,024 messages |
| Discovery and reporting | 32 advertised revisions; 10,000 catalog items; 256 report findings |
| Schema and instance | Schema 1 MiB; instance 1 MiB; 100,000 schema nodes; depth 64; local `$ref` depth 32; 100,000 evaluated schema-location/instance-location pairs; 100 collected validation errors |
| Active and generation work | 100 cases; 8 MiB aggregate active inputs; 256 generation attempts; 64 retained candidates; 100,000 generation steps |
| Network activity | Zero redirects; zero retries; concurrency 1 |

Revision selection stops and reports a limit finding at the 33rd advertised
value, even if the source could continue indefinitely. Report construction
rejects more than 256 findings. Limit construction rejects zero safety bounds,
a stage longer than the total,
a message larger than stdout, a stream larger than the combined-output cap, a
schema or instance larger than a message, `$ref` depth above schema depth, or
retained generation candidates above generation attempts, or concurrency above
the case budget. Redirect and retry counts alone may be zero. Generation also
fails closed when its requested cases, candidate inputs,
aggregate active-input bytes, schema work, validation work, or synthesis steps
exceed these simultaneous bounds.

### MCPD-005 bounded STDIO boundary

`mcp-doctor inspect -- <executable> [arguments...]` now launches one local
target directly, with no shell or argument expansion. The target inherits only
`PATH` on Unix and `PATH`, `PATHEXT`, `SystemRoot`, and `WINDIR` on Windows;
Windows batch files are rejected because their platform launch path would
require a command interpreter.

The boundary begins with one newline-delimited `server/discover` request with
exact `2026-07-28` per-request metadata. `MCPD-006` may continue on the same
managed process with only the capability-gated passive list requests recorded
below. It never sends `initialize` or an implicit active request. Fixed-size
reads enforce the simultaneous M1 message, message-count, stdout, stderr,
combined-output, startup, discovery, request, response, shutdown-grace, and
total-run bounds without retaining stderr or exposing raw target, payload, or
I/O-error values.

Shutdown closes stdin first, allows the declared grace period, then terminates
the entire Unix process group or Windows Job Object and waits for managed
process and pipe completion. A drop guard preserves best-effort termination on
an unexpected control path. The feature-gated synthetic fixture is not part of
a normal installation; it covers conforming, malformed, timed-out, oversized,
early-exit, and resistant-descendant behavior through the built CLI.

Local acceptance on 2026-08-09 is 33 unit tests, six CLI tests, and nine
built-binary STDIO tests under the locked dependency graph. macOS ARM64 runs
the complete matrix; Windows x64 and GNU/Linux x64 all-target code paths also
cross-compile locally. Native execution on those two hosts remains evidence
for the next hosted pull request and is not claimed by this local run.

### MCPD-006 passive catalog and schema diagnosis

After a valid `server/discover` result selects MCP `2026-07-28`, `inspect`
issues only the list methods backed by advertised capabilities, in deterministic
order: `tools/list`, `prompts/list`, `resources/list`, and
`resources/templates/list`. Each page uses required per-request metadata and an
opaque cursor only when the prior page returned a new one. A repeated cursor,
global catalog-item excess, malformed page, or exhausted transport bound stops
the affected work. The conversation never constructs `tools/call`,
`prompts/get`, `resources/read`, or `initialize`; fixture servers assert the
exact method sequence and EOF after the last passive response.

The adapter checks complete/cacheable list envelopes, known capability setting
types, core tool, prompt, resource, and resource-template shapes, identifiers
across pages, prompt argument identifiers, and resource URI identity without
placing any identifier, cursor, URI, schema value, or payload into a report.
Locations contain only trusted MCP/schema fields, numeric array indexes, and
`[*]` for server-defined property names. Invalid values are represented by a
safe JSON type; every failure includes a static expectation, correction, and
versioned contract reference.

Tool `inputSchema` must be an object with root `type: "object"`; optional
`outputSchema` must also be an object. M1 accepts an omitted `$schema` as the
MCP Draft 2020-12 default or the exact Draft 2020-12 URI. Other explicitly
declared dialects, including Draft 7, receive `MCP-SCHEMA-002` rather than being
silently reinterpreted. Fragment-only local `$ref` and `$dynamicRef` values are
resolved under node, depth, reference-depth, and work budgets. Relative,
HTTP(S), file, and other external references receive `MCP-SCHEMA-003` before
compilation. The maintained `jsonschema` validator runs with default features
disabled and a rejecting retriever; the locked normal dependency graph contains
no HTTP or TLS client. Draft 2020-12 meta-schema errors and compilation errors
produce sanitized structural locations and never render validator error text.

The synthetic fixture matrix covers valid empty and paginated catalogs,
composition and local-reference schemas, malformed prompt/resource/template
shapes, invalid and unsupported schemas, unresolved and external references,
duplicate names, repeated cursors, and exact catalog-item, schema-node,
schema-depth, local-reference-depth, schema-work, validation-error, and
report-finding boundaries. A disposable loopback listener proves
external-reference diagnosis makes no connection. Static fixture values and
cursors are absent from reports; repeated invalid runs are byte-for-byte
deterministic. `MCPD-007` adds built-binary earliest-layer, causal-skip,
independent-cleanup, report-only, and human/JSON parity journeys over the same
bounded transport.

Local acceptance on 2026-08-10 is 40 unit tests, seven CLI tests, one
compatibility-policy test, one dependency-policy test, and 22 built-binary
STDIO tests (71 total) through the disposable locked gate. The exact local
format, Clippy, test, `cargo-deny`, ShellCheck, Bash syntax, Actionlint, JSON,
package, and diff checks pass. The source package contains 53 files and verifies
from its staged contents. Native GNU/Linux x64 and Windows x64 execution and
the complete hosted matrix are not claimed by this local run.

### MCPD-007 dated acceptance review

The controlled [compatibility matrix](tests/compatibility/README.md) passed all
four pinned MCP `2026-07-28` STDIO servers on 2026-08-10: official TypeScript
and Go examples plus independent Dart and PHP implementations. Every case
performed and passed the five required passive checks, left `runtime.tools`
skipped as `not_authorized`, exited 0, and left no target container running.
Under `DEC-024`, that evidence supports only the scoped phrase **broad
current-revision compatibility**. It is not official conformance, every-server
support, legacy support, HTTP support, or evidence that a tool was called.

The official release recheck on 2026-08-10 identified
[MCP `2026-07-28`](https://github.com/modelcontextprotocol/modelcontextprotocol/releases/tag/2026-07-28)
as the latest specification release, so the supported-revision decision did
not change. The immediate publication recheck found both Cargo names available;
the exact immutable source package subsequently claimed the preferred
[`mcp-doctor` identity](https://crates.io/crates/mcp-doctor/0.1.0). The fallback
`enjoyable-mcp-doctor` name was not used.

The conditional test-tool review resolved all three `MCPD-007` candidates
without adding a dependency or numeric release gate:

| Tool reviewed on 2026-08-10 | Decision | Measured reason |
| --- | --- | --- |
| [`cargo-nextest` `0.9.143`](https://github.com/nextest-rs/nextest/releases/tag/cargo-nextest-0.9.143) | Not adopted | The authoritative 71-test suite completes in about 12 seconds and is dominated by an intentional 10-second timeout case; there is no demonstrated partitioning, isolation, or flake-diagnosis need for another runner |
| [`cargo-llvm-cov` `0.8.7`](https://github.com/taiki-e/cargo-llvm-cov/releases/tag/v0.8.7) | Diagnostic only; not adopted | A copied-tree run reported 54.22% lines, but the required cleared environment prevents coverage state from reaching built-binary subprocesses and therefore understates exercised transport and inspection code; weakening isolation or enforcing a misleading percentage would reduce assurance |
| [`cargo-mutants` `27.1.0`](https://github.com/sourcefrog/cargo-mutants/releases/tag/v27.1.0) | Diagnostic only; not adopted | A time-bounded copied-tree subset challenged five primary-diagnosis mutations; the first run caught two, three misses produced narrow standard tests, and the rerun caught all five in about six seconds |

All three upstreams were active, unarchived, permissively licensed projects at
the dated review. Rejection remains the safer result because the measured need
did not justify permanent developer or CI code execution. The exact diagnostic
versions and outcomes above are reproducible evidence, not newly supported
project tooling.

Hosted acceptance on `main` commit `24f79f8` completed on 2026-08-10. The
[native CI run](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31363588701)
passed dependency policy plus the complete disposable gate on GNU/Linux x64,
macOS ARM64, and Windows x64. The separately dispatched
[compatibility run](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31363605095)
passed all four pinned current-revision servers on hosted GNU/Linux x64. These
results close `MCPD-007` and M1; they do not publish an artifact or begin M2.

### Golden M1 journey

Given a disposable test environment, executable fixture servers, and the
separately controlled compatibility matrix:

1. `mcp-doctor inspect` identifies a conforming STDIO server and reports
   exactly which passive checks passed without calling a tool.
2. A malformed message, invalid catalog schema, timeout, oversized response,
   early exit, and unsupported protocol each return a distinct redacted finding
   and non-zero status; each fixture selects the expected earliest actionable
   layer and states what failed, where, why it matters, what was expected, how
   to correct it, and which versioned rule applies.
3. Human and `mcp-doctor.report/v1alpha1` JSON output derive from the same
   result, designate the same primary layer and findings, retain independent
   findings and causally linked skips in deterministic order, and do not expose
   arbitrary observed values.
4. A report-only acceptance case proves the intended corrective next step is
   recoverable without raw JSON-RPC traffic, stderr, source code, or a browser.
5. The CLI closes or terminates and reaps every fixture process, including one
   that ignores graceful shutdown.
6. Pinned official SDK examples and independent implementations across at
   least two implementation languages establish which real servers the claimed
   revision and passive checks can diagnose without turning live endpoints into
   the default test suite.
7. No default test reads user configuration, reaches a production endpoint,
   invokes a tool, or exposes fixture values in a report or assertion failure.

### MCPD-008 dated release evidence

On 2026-08-10, annotated tag `v0.1.0` resolved to commit
`948d0b62546a7707d90fe6a28b6c219c229fb9f6`. The
[release workflow](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31405768056)
published the canonical immutable
[`v0.1.0` GitHub Release](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.1.0)
with exactly seven checksummed and attested assets. The exact formula was copied
without regeneration to the
[`homebrew-tap` commit](https://github.com/EnjoyableWork/homebrew-tap/commit/6044088bc8b04c24a762a69cabbe52a5b22b1e22).

An isolated detached checkout generated the Cargo package three times with Rust
and Cargo `1.97.1`; both `cargo package` runs and `cargo publish --dry-run`
matched the immutable asset SHA-256
`4ebd55311c86533d1d0bb34a223060f551ea8aaeb287de666b51b31b05ceb36d`.
The one-time `publish-new` credential then published
[`mcp-doctor` `0.1.0`](https://crates.io/crates/mcp-doctor/0.1.0). Its registry
download matched the GitHub asset byte for byte, and `cargo logout` removed the
local credential.

The credential-free
[channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31413131715)
passed its immutable identity gate and all nine installed passive smokes: Cargo
on macOS ARM64, Windows x64, and GNU/Linux ARM64/x64; source-built Homebrew on
macOS ARM64 and GNU/Linux ARM64/x64; and GitHub archives on GNU/Linux ARM64/x64.
The zero-report adoption baseline and nonblocking closing decision are recorded
in [`M2 adoption checkpoint` issue 5](https://github.com/EnjoyableWork/mcp-doctor/issues/5).
The owner confirmed server-side revocation of the one-time crates.io token after
`cargo logout` removed it locally. These results close `MCPD-008`.

`MCPD-008A` completed the retained future-release boundary without changing
`v0.1.0`: canonical future-only stable tags, immutable GitHub bytes before any
downstream write, registry-order rejection with exact-byte recovery, global
release serialization, a final main/tag authority recheck, crates.io OIDC under
the exact workflow and environment, a tap-owned short-lived write job, reusable
verified handoff manifests, and rejected provenance and byte-mismatch cases.

The 2026-08-10 GitHub administration readback shows a required-reviewer
`release` environment in both public repositories. `mcp-doctor` admits only
branch `main` and tag pattern `v*.*.*`; `homebrew-tap` admits only branch
`main`. Both use custom deployment policies, both repository and environment
secret inventories are empty, the organization Actions-secret inventory is
empty, and both repositories retain read-only default workflow permission.
GitHub reports administrator bypass enabled in this one-member organization,
so the reviewer gate provides deliberate single-maintainer friction rather
than independent approval; no two-person claim is made.

The exact repeat-release implementation merged at source commit
[`47aa41e`](https://github.com/EnjoyableWork/mcp-doctor/commit/47aa41efee5eaf1ed81f699611748aff787ed971)
and tap commit
[`dafb41a`](https://github.com/EnjoyableWork/homebrew-tap/commit/dafb41ae86968285b5ae85f3dd633cc15103131b).
The 2026-08-10 crates.io owner readback reports repository
`EnjoyableWork/mcp-doctor`, owner ID `245300066`, workflow `release.yml`, and
environment `release`, with trusted publishing required for every new version.
The live [immutable-first and authorized/missing-environment
rehearsal](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31443495330),
[wrong-workflow rejection](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31441772215),
[tap-owned no-write rehearsal](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/31444057455),
and post-tap [ten-job channel
verification](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31444142085)
all pass on those exact commits. The final
[`verify-repeat-release-controls.sh`](scripts/verify-repeat-release-controls.sh)
readback confirms current public main commits, immutable releases, protected
environment policies, read-only workflow defaults, empty applicable GitHub
secret inventories, no operator crates.io credential, and no stored or personal
credential reference. These results close `MCPD-008A`, D-07A, and M2 without a
new tag, publication, or change to immutable `v0.1.0` bytes.

### M2 adoption checkpoint conclusion

The checkpoint opened and closed on 2026-08-10 with zero independent adoption
reports. That absence remains explicit: the project makes no adoption or repeat-
use claim. `DEC-027` records the owner decision that externally timed reports,
which may take days or months to arrive, must not indefinitely prevent planned
M3 work.

Future consented, aggregate, non-sensitive reports remain useful prioritization
evidence. They may reprioritize, narrow, defer, or cancel later work, but they
are not prerequisites for a ticket whose predecessor, design decisions, safety
boundary, and acceptance evidence are otherwise ready. Do not collect endpoints,
credentials, payloads, identities, or private diagnostic output.

### MCPD-009 accepted design boundary

`MCPD-009` adds deterministic replay of reviewed cases, not generated pressure.
Seeded generation is the separate `MCPD-011` boundary. Every active command
remains noninteractive and calls exactly one tool selected independently by its
configuration and invocation.

#### Scenario contract

The only accepted initial format is JSON with the discriminator
`mcp-doctor.scenario/v1alpha1`. One file names one exact tool and contains from
one through the existing 100-case limit in array order. Each case has a unique
author-facing ID, JSON object arguments, expected `success` or `tool_error`,
and an optional local Draft 2020-12 schema for `structuredContent`.

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

Before a call, the case arguments must pass the discovered tool's advertised
input schema. A completed result must match `success` versus `tool_error`, the
advertised output schema when present, and the scenario's optional narrowing
schema. Validation uses the existing bounded, no-retrieval Draft 2020-12
boundary. Scripts, shell expansion, external references, raw-result snapshots,
and another scenario parser are outside `MCPD-009`; schema `const` and `enum`
remain available for reviewed value expectations without printing observed
values.

Cases run sequentially with concurrency one. An ordinary tool error or result
mismatch is recorded and later declared cases still run. Authorization,
transport, cleanup, or exhausted-resource failures stop remaining calls because
continuing would be unsafe or unreliable. Reports use deterministic case indexes
and structural locations; they do not copy case IDs, tool arguments, results,
schema values, or other arbitrary scenario values.

#### Secret-reference contract

Secrets come only from the invoking process environment. `target_env` is an
explicit same-name allowlist added to the existing minimal child environment.
Each `secret_refs` entry uses an RFC 6901 JSON Pointer relative to that case's
arguments and names one source environment variable. The pointer must resolve
to an existing `null` placeholder; the root, missing locations, duplicate
locations, non-null destinations, invalid variable names, and missing values
are configuration failures before the target starts or any tool is called.
Reference names use `[A-Za-z_][A-Za-z0-9_]*`; argument-secret values must be
valid UTF-8 because they become JSON strings, while allowlisted target values
remain literal operating-system strings.

There is no interpolation, inherited unrestricted environment, `.env` loading,
file or command reference, prompt, keychain integration, or project secret
store. Environment names and values, pointers, resolved arguments, and results
cannot enter ordinary human or machine output, errors, or debug formatting.

#### Active-authorization contract

Every scenario must classify its tool as `read_only` or `side_effecting`;
uncertainty is `side_effecting`, and omission is invalid. Every invocation must
also provide `--allow-tool <exact-name>`, which must match the scenario and the
discovered tool byte for byte. A `side_effecting` scenario additionally requires
`--allow-side-effects`. No wildcard, pattern, generic `--yes`, interactive
prompt, discovered-tool selection, or server annotation can substitute for
these gates; server annotations remain untrusted input.

An `input_required` result is recorded as incomplete for that case and is not
retried. `MCPD-009` does not answer elicitation, sampling, roots, or another
server-initiated request. Supporting a later round requires its own accepted
authorization, secret, redaction, budget, and result contract.

#### MCPD-009 local acceptance evidence

The built binary now exposes the accepted `check` command and strict regular-
file scenario contract. [Twenty-two active test suites](tests/active.rs) exercise
ordered success and ordinary continuation; exact, wildcard, and side-effect
authorization rejection before target start; environment-only target and
argument secrets; human and experimental JSON redaction; input-schema rejection
without a call; advertised and scenario output schemas; protocol rejection;
invalid and `input_required` results; scenario, schema, validation, report, and
transport limits; crashes; and resistant-process-tree cleanup. Fixtures assert
the exact request order, current revision metadata, absence of `initialize`, no
automatic input response, causal stop behavior, and EOF when no later call is
safe. The normal disposable quality gate and
`cargo deny --all-features --locked check` pass without a dependency change.

### MCPD-010 implemented network boundary

`DEC-030` resolved `OPEN-06`; `MCPD-010` implements that boundary and passes its
local deterministic and built-binary acceptance evidence. It does not change
the published `v0.1.0` artifacts; that ticket itself did not begin adversarial
generation, add an OAuth client, or claim hosted native remote evidence. A
future release must still run the normal native release and installed-channel
gates against its exact bytes.

The decision was reviewed on 2026-08-10 against the official MCP
[`2026-07-28` Streamable HTTP transport](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http)
and [authorization specification](https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization),
[HTTP Semantics, RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html),
[TLS BCP 195, RFC 9325](https://www.rfc-editor.org/rfc/rfc9325.html),
[service identity verification, RFC 9525](https://www.rfc-editor.org/rfc/rfc9525.html),
and the dated IANA
[IPv4](https://www.iana.org/assignments/iana-ipv4-special-registry/iana-ipv4-special-registry.xhtml)
and [IPv6](https://www.iana.org/assignments/iana-ipv6-special-registry/iana-ipv6-special-registry.xhtml)
special-purpose address registries. These sources are reviewed design evidence;
the binary never retrieves policy or registry data at runtime.

#### Transport and protocol scope

One invocation names one absolute MCP endpoint. `inspect` remains passive, and
remote `check` inherits every `DEC-028` and `DEC-029` scenario, exact-tool,
side-effect, secret, continuation, and redaction gate. Selecting a network
transport never grants authority to call a tool.

The first HTTP adapter implements only the stateless MCP `2026-07-28` binding:
one JSON-RPC request per POST, no `initialize`, no protocol session, no
standalone GET stream, and no legacy fallback. It sends the matching
`MCP-Protocol-Version`, `Mcp-Method`, and applicable `Mcp-Name` and
`Mcp-Param-*` headers. It accepts only `application/json` or request-scoped
`text/event-stream`, bounds and ignores request-related notifications, and
requires one final matching response. `subscriptions/listen`, HTTP
notifications, resumable SSE, HTTP+SSE, WebSocket, HTTP/3, h2c, upgrade, and
alternate-service behavior are outside `MCPD-010`.

#### Target and address authority

The endpoint must be a strictly parsed absolute `https` URL, except for the
loopback cleartext exception below. User information, query strings, fragments,
percent-encoded hosts, IPv6 zone identifiers, noncanonical numeric IPv4 forms,
empty hosts, and ports outside 1–65535 are rejected before DNS or any request.
DNS names are converted to their ASCII form and endpoint comparisons use one
canonical scheme, host, port, and path representation. The raw URL never enters
an ordinary report.

Public HTTPS is the default. A target is public only when every normalized DNS
answer is unicast and either outside all reviewed IANA special-purpose blocks or
covered by the most-specific applicable entry whose `Destination` and
`Globally Reachable` fields are both true. IPv4-mapped IPv6 addresses are
classified as IPv4. Loopback, RFC 1918 private-use, RFC 6598 shared, and IPv6
unique-local destinations are eligible only when
`--allow-private-network <exact-url>` independently parses to the same canonical
endpoint. No wildcard, CIDR, suffix, environment setting, configuration file,
or previous invocation can provide that authority.

Cleartext HTTP is limited to an all-loopback target and additionally requires
`--allow-cleartext-http <exact-url>` as well as the private-network gate. It can
never carry a bearer token or user-supplied header. Link-local, unspecified,
multicast, broadcast, documentation, benchmarking, discard-only, and other
non-global special-purpose destinations remain prohibited even with either
gate. This keeps cloud metadata and ambiguous special-use routes outside the
initial contract.

Name resolution runs once under the connection and total deadlines, retains at
most 16 unique answers, and fails the whole target on overflow, a prohibited
answer, or a mixture of public and private classes. The accepted addresses are
sorted and pinned for the run. Every connection uses only that set, checks its
peer address, and uses the canonical hostname for HTTP authority, TLS SNI, and
certificate verification. There is no second resolution after validation;
the connector attempts only the first sorted address family and does not race
or fall through to another family. Sequential attempts within those pinned
candidates are connection establishment, not an application retry.

#### Redirect, proxy, and connection policy

Redirects and application retries remain exactly zero. Every `3xx` response is
a typed failure; `Location` is neither followed nor rendered. Authentication
challenges, `429`, stale-header errors, connection failures, and TLS failures do
not cause an automatic replay. Each planned MCP operation is sent at most once.

Connections are direct. The adapter ignores `HTTP_PROXY`, `HTTPS_PROXY`,
`ALL_PROXY`, `NO_PROXY`, platform proxy settings, PAC, `.netrc`, DNS service
binding, and proxy credentials, and exposes no explicit proxy option in this
ticket. It does not use a cookie jar, cache, HSTS store, `Alt-Svc`, cross-origin
connection coalescing, or persistent credential store. The implemented adapter
uses HTTP/1.1 for both HTTPS and explicitly allowed loopback cleartext; HTTP/2
is outside this ticket.

#### TLS and trust policy

HTTPS requires TLS 1.2 or 1.3 under BCP 195, prefers TLS 1.3, and always verifies
the complete certificate chain, validity period, and canonical DNS name or IP
identity. There is no `--insecure`, hostname exception, expired-certificate
exception, silent downgrade, or credential-bearing cleartext fallback.

The default trust anchors come from the platform verifier. Environment trust
overrides such as `SSL_CERT_FILE` and `SSL_CERT_DIR` are ignored. A caller may
add trust for only this run with `--tls-ca-file <path>`: a bounded regular file
of at most 1 MiB containing at most 32 PEM CA certificates and no private key.
The path, certificate subjects, SANs, contents, and verifier text stay out of
ordinary output. Client certificates, private keys, PKCS#12, and mutual TLS are
outside this ticket. Any selected TLS implementation still requires the full
dependency review before adoption.

#### Authentication and header policy

`MCPD-010` supports pre-provisioned request credentials, not an OAuth client
flow. `--bearer-token-env <NAME>` reads one nonempty RFC 6750 bearer token from
an explicitly named invoking-process environment variable. Repeated
`--header-env <FIELD=NAME>` options may supply other end-to-end fields from
explicit environment variables. Either option also requires
`--allow-credentials-to <exact-url>` matching the canonical HTTPS endpoint.
Credential values are resolved and validated before the first connection and
are sent on every MCP POST only after TLS identity verification. They never
enter a URL, command-line value, redirect, proxy, cleartext request, report,
error, or debug representation.

The adapter does not fetch `resource_metadata`, authorization-server metadata,
or another URI from `WWW-Authenticate`; it does not open a browser, register a
client, request scopes, exchange or refresh tokens, use a keychain, or persist
credentials. HTTP `401` and `403` become structural authentication findings
that may record status and challenge kind, never challenge parameters, scopes,
descriptions, metadata URLs, or raw bodies. This is deliberately not a claim of
MCP Authorization client conformance; a future authorization flow requires its
own accepted multi-origin SSRF, phishing, credential-storage, callback, token,
and consent contract.

User-supplied field names must be valid HTTP tokens, unique case-insensitively,
and cannot override routing, framing, negotiation, authentication, or MCP-owned
fields. At minimum `Host`, `Authorization`, `Proxy-Authorization`, `Cookie`,
`Origin`, `Referer`, `Forwarded`, `X-Forwarded-*`, `Connection`, `TE`, `Trailer`,
`Transfer-Encoding`, `Upgrade`, `Content-*`, `Accept*`, `User-Agent`, every
`Mcp-*`, and every `Proxy-*` or `Sec-*` field are reserved. Values must be
nonempty visible ASCII plus permitted spaces or tabs and contain no control,
CR, LF, or NUL byte. The dedicated bearer option alone constructs
`Authorization: Bearer`.

The adapter owns `Host`, `Content-Length`, `Content-Type`, `Accept`,
`Accept-Encoding: identity`, `User-Agent`, and the current-revision MCP headers.
Automatic decompression is disabled, and a non-identity `Content-Encoding` is
rejected. A valid `x-mcp-header` annotation is transport mapping, not execution
authority: remote `check` applies it only after exact tool authorization and
input validation, uses the specification's safe/Base64 encoding, charges every
derived field and value to the header budgets, and never renders the annotation,
argument path, or value. Invalid or over-budget annotations exclude the tool
with a typed structural finding; the client does not retry after a header
mismatch.

#### Network bounds and diagnostic evidence

The existing message, aggregate-output, message-count, request, response, and
120-second total values apply. For this transport the existing 10-second startup
value bounds DNS, TCP, and TLS connection establishment. `MCPD-010` adds the
following typed network limits to the shared result and both reporters:

| Network area | Default bound |
| --- | --- |
| Endpoint | 8,192 UTF-8 bytes; one canonical origin and path |
| Resolution | 16 unique addresses; one resolution; pinned sequential connection attempts |
| Trust file | 1 MiB; 32 PEM CA certificates |
| Request fields | 64 fields; 256-byte name; 8 KiB encoded wire value; 32 KiB aggregate wire bytes |
| Response fields | 96 fields; 256-byte name; 16 KiB wire value; 64 KiB aggregate wire bytes |
| Body and SSE | 1 MiB per JSON message or SSE event; 8 MiB aggregate; 1,024 messages/events |
| Activity | zero redirects; zero application retries; concurrency one |

The 96-field response ceiling leaves four slots below the selected HTTP/1
parser's fixed 100-field hard stop for framing fields, so the first supported
excess remains a typed `mcp-doctor` limit rather than an unclassified parser
failure.

Before DNS or connection, human output may describe only structural target
facts such as HTTPS versus explicitly allowed loopback HTTP and which gates are
present; it does not echo the endpoint. Reports may retain the address class,
explicit/default port distinction, TLS version, HTTP status, media-type class,
and byte, field, event, or address counts. They never retain a host, IP, path,
URL, DNS answer, socket text, certificate identity, header name or value,
environment source, authentication challenge, or response body. Machine output
has the same boundary. The earliest target, resolution, TLS, HTTP, protocol, or
authentication failure becomes the primary diagnosis; dependent discovery or
case work is causally skipped, while independent limit and redaction failures
remain visible.

`MCPD-010` acceptance requires deterministic resolver and connector ports plus
disposable local HTTP and TLS servers. Built-binary and focused tests must prove
canonical URL rejection, exact gates, public/private/mixed/special address
classification, answer overflow and pinning, peer mismatch, zero redirects and
retries, ignored proxy and trust environment, positive and negative TLS identity,
bounded custom and `Mcp-Param-*` fields, credential delivery and redaction,
`401`/`403`, JSON and SSE framing, compression rejection, status/header/body/
event/time limits, passive-versus-active authority, and human/JSON causal parity
without a real MCP server, production endpoint, secret output, or network escape.

### MCPD-011 implemented generation boundary

`DEC-031` fixes `break` as deterministic, schema-valid boundary generation for
one explicitly selected tool. It does not fuzz arbitrary bytes, send inputs
that fail the advertised schema, discover a tool to call, or turn passive
inspection into active behavior.

#### Selection and execution authority

Every invocation supplies the selected exact tool independently through both
`--tool <exact-name>` and `--allow-tool <exact-name>`, declares `read_only` or
`side_effecting` through `--effects`, chooses 1–100 cases, and supplies an
unsigned 64-bit seed. A `side_effecting` run also requires
`--allow-side-effects`. Empty selections and mismatches fail before a local
target starts or a remote target resolves. Wildcard-looking or pattern-looking
values remain literal and never broaden matching, and server annotations grant
no authority.

Generation begins only after the current-revision discovery, unique tool
selection, and bounded local Draft 2020-12 input-schema contract pass. It uses
only that selected schema and the existing validator. It does not accept
target-environment or argument-secret sources, retrieve a schema, answer
elicitation or another server request, select another tool, alter the literal
local target, or change the exact `DEC-030` endpoint and credential authority.

Generated calls remain sequential with concurrency one. Each generated case
expects a completed non-error result; an ordinary tool error or mismatch is a
finding and later cases continue. `input_required` remains incomplete without
an automatic answer or retry. Transport, protocol, authorization, cleanup,
invalid-result, and exhausted-resource failures stop dependent calls under the
same active causal model as `check`.

#### Deterministic generator and reproduction contract

`mcp-doctor.generator/v1` constructs a finite candidate set from applicable
object structure, local references, declared `const`, `enum`, `default`, and
examples, common numeric, string, array, and property boundaries, and bounded
schema combinators. The existing validator is authoritative: every retained
candidate is a JSON object that passes the exact advertised schema. A schema
the generator cannot satisfy receives `MCP-GENERATION-001`; invalid,
externally referenced, or over-limit schemas retain their earlier schema or
limit diagnosis and no `tools/call` occurs.

The fixed simultaneous bounds are 256 synthesis attempts, 64 retained
candidates, 100,000 synthesis steps, 100 selected cases, 1 MiB per input, and
8 MiB across retained candidates or selected active inputs. Existing schema
node, depth, local-reference depth, validation-work, transport, response,
message, time, finding, and cleanup limits also apply. Candidate order and
selection use fixed-width arithmetic plus ordered collections so the same
generator version, advertised schema, and seed do not depend on hash order or
host word size.

Case `n` uses the base seed with wrapping addition of `n`; using one reported
case seed as the base for a one-case run selects the same input under the same
schema and generator version. Human and experimental JSON reports retain only
the generator version, case seed, serialized byte count, root kind, node and
depth counts, and fixed JSON kind/member/item counts. They do not retain object
member names, scalar values, raw arguments, raw results, tool names, targets,
or server-provided schema values. Any future algorithm change that alters
these seeded sequences must change the generator version and its exact-seed
fixture.

#### MCPD-011 local acceptance evidence

The [bounded generator](src/contract/generate.rs), shared [active diagnostic
model](src/contract/active.rs), and [`break` application](src/break_command.rs)
add no dependency. [Nine disposable built-binary generation
journeys](tests/break.rs) prove exact known-seed reproduction, structural-only
human/JSON evidence, unsigned seed wraparound, exact and side-effect authority
before target start, sequential continuation, prohibited schema retrieval,
unsatisfiable and instance/aggregate/work limits without a call, the exact
100-case ceiling, and resistant-descendant cleanup. The [disposable HTTP/TLS
suite](tests/http.rs) separately proves that generated activity retains the
same exact endpoint and tool authority and makes no unauthorized connection.
Focused generator tests pin `mcp-doctor.generator/v1` outputs and revalidate
every retained input against its advertised schema. The disposable formatting,
Clippy, and full-test gate, `cargo deny`, and dirty-tree package verification
pass locally; no hosted or exact-artifact M3 claim is made.

## Target architecture

```text
┌──────────────────────────────────────────────────────────┐
│                       CLI boundary                       │
│        argument parsing · output · stable exit code      │
└───────────────────────────┬──────────────────────────────┘
                            v
┌──────────────────────────────────────────────────────────┐
│                  diagnostic application                  │
│          inspect · check · break · aggregate             │
└──────────────┬───────────────────────┬───────────────────┘
               v                       v
┌──────────────────────────┐  ┌────────────────────────────┐
│ model and rules          │  │ replaceable boundaries     │
│ findings · limits        │  │ process · HTTP · clock     │
│ schemas · versions       │  │ randomness · reporters     │
└──────────────────────────┘  └────────────────────────────┘
```

Begin with a modular monolith. The diagnostic model owns no terminal styles,
OS paths, child handles, sockets, or concrete randomness. Real version and
transport variation should remain cohesive rather than leak through the CLI.

## Deliverables

| ID | Deliverable | Milestone | State | Evidence |
| --- | --- | --- | --- | --- |
| D-01 | Product and repository operating model | M0 | Done | [README.md](README.md), [PROJECT.md](PROJECT.md), [AGENTS.md](AGENTS.md) |
| D-02 | Rust CLI walking skeleton | M0 | Done | Manifest, lockfile, help/version source, five focused tests, package verification, and installed source-package smoke |
| D-03 | Local and hosted quality baseline | M0 | Done | POSIX and PowerShell gates, exact direct requirements with a regression test, dated dependency/testing-tool adoption policy, least-privilege three-OS workflow, Dependabot, and community/security surfaces pass locally; the [hosted run on `main` `24f79f8`](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31363588701) proves the refined dependency policy plus GNU/Linux x64, macOS ARM64, and Windows x64 gates |
| D-04 | Versioned diagnostic result contract | M1 | Done | [Typed contract modules](src/contract), [synthetic contract fixtures](tests/fixtures/contracts), and focused revision, limit, finding, redaction, skip, outcome, exit, and reporter tests |
| D-05 | Bounded STDIO process and message boundary | M1 | Done | [Managed STDIO transport](src/transport/stdio.rs), [synthetic fixture server](tests/fixtures/stdio_server.rs), and [nine built-binary journeys](tests/stdio.rs) prove literal arguments, constrained environment, passive discovery, simultaneous bounds, redaction, graceful and forced process-tree cleanup, and distinct transport failures; the full 48-test suite passes locally |
| D-06 | Adoption-ready passive `inspect` journey | M1 | Done | [Built-binary journeys](tests/stdio.rs) prove earliest-layer selection, independent safety findings, causal skips, report-only correction, redaction, and equivalent human/experimental JSON; the [four-case compatibility matrix](tests/compatibility/README.md), scoped broad current-revision position, registry/revision rechecks, and conditional test-tool decisions pass locally; [native CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31363588701) and [hosted compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31363605095) pass on `main` `24f79f8` |
| D-07 | Immutable passive MVP release | M2 | Done | The immutable GitHub release, byte-identical crates.io package and Homebrew formula, ten-job installed channel verifier, dated adoption baseline, local credential removal, and confirmed server-side token revocation pass |
| D-07A | Least-privilege repeat-release path | M2 | Done | Exact source and tap `main` commits, protected environments, crates.io publisher identity, authorized and rejected OIDC paths, immutable byte handoffs, tap-owned no-write rehearsal, ten-job channel verification, and final clean credential readback pass in linked `MCPD-008A` evidence |
| D-08 | Bounded diagnostic expansion release | M3 | In progress | `MCPD-009` through `MCPD-011` complete bounded reviewed replay, direct Streamable HTTP, and deterministic authorized generation locally; stable reporting and CI, publication, installed-channel evidence, and independent verification remain |
| D-09 | Evidence-backed enterprise assurance baseline | M4 | Proposed | Verified repository, organization, community, licensing, and supply-chain controls; complete OSPS Level 1 crosswalk; official self-certification proof; and exact-artifact SLSA evaluation |

## Ticket board

| ID | Outcome | Milestone | Status | Depends on | Acceptance evidence |
| --- | --- | --- | --- | --- | --- |
| MCPD-001 | Establish the product promise, operating model, safety priorities, delivery sequence, decisions, and risks | M0 | Done | — | Root product and project contracts are internally consistent and link correctly |
| MCPD-002 | Bootstrap one Rust 2024 binary with truthful help/version output and isolated built-binary tests | M0 | Done | `MCPD-001` | Locked build, format, Clippy, five tests, help, version, metadata, self-contained package, and installed package smoke pass |
| MCPD-003 | Add disposable local gates, dependency policy, least-privilege cross-platform CI, maintenance automation, and community/security entry points | M0 | Done | `MCPD-002` | POSIX and PowerShell gates, `cargo-deny`, exact direct-requirement enforcement, dated dependency and testing-tool reviews, Actionlint, ShellCheck, YAML parsing, links, packaging, and identity checks pass locally; the [hosted matrix on `main` `24f79f8`](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31363588701) passes the refined dependency policy and all three native hosts |
| MCPD-004 | Define supported MCP revision behavior, typed findings, limits, exit semantics, and redacted report contract | M1 | Done | `MCPD-003` | [Contract implementation](src/contract), [synthetic snapshots and cases](tests/fixtures/contracts), and 24 focused contract tests prove the accepted compatibility, severity, limit, redaction, performed/skipped, deterministic report, outcome, and exit decisions |
| MCPD-005 | Implement the bounded STDIO process and message boundary with guaranteed cleanup | M1 | Done | `MCPD-004` | [Nine built-binary cases](tests/stdio.rs) cover empty-capability success without a follow-up request, literal arguments, constrained environment, malformed and redacted output, every I/O limit, timeout, early exit, missing process, and resistant-descendant cleanup; focused framing, protocol, budget, report, and cross-target compile checks also pass |
| MCPD-006 | Diagnose discovered tools, prompts, resources, and JSON Schema contracts without implicit tool execution | M1 | Done | `MCPD-005` | [Versioned catalog/schema adapter](src/contract/catalog.rs), [static catalog fixtures](tests/fixtures/catalogs), and [built-binary STDIO journeys](tests/stdio.rs) prove valid, invalid, complex, duplicate, paginated, redacted, no-retrieval, and exact bounded cases with safe expectations and remediation; the complete local locked gate and cross-target checks pass |
| MCPD-007 | Make passive `inspect` identify the earliest actionable failing layer, remain report-sufficient for humans and agents, and prove its real-server reach and release identity | M1 | Done | `MCPD-006` | Built-binary human and experimental JSON journeys agree on primary and independent findings, causal skips, limits, summary, outcome, and correction; report-only fixtures prove actionability; four pinned official/independent servers across four languages support scoped broad current-revision compatibility; registry and revision rechecks are dated; all conditional tools are resolved under `DEC-025`; [native CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31363588701) and [hosted compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31363605095) pass on `main` `24f79f8` |
| MCPD-008 | Publish and independently verify the first immutable passive-MVP release through GitHub, Cargo, and Homebrew | M2 | Done | `MCPD-007` | [GitHub publication](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31405768056), byte-identical [crates.io](https://crates.io/crates/mcp-doctor/0.1.0) and [Homebrew](https://github.com/EnjoyableWork/homebrew-tap/commit/6044088bc8b04c24a762a69cabbe52a5b22b1e22) handoffs, the [ten-job channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31413131715), and dated baseline in [adoption issue 5](https://github.com/EnjoyableWork/mcp-doctor/issues/5) pass; the one-time credential was removed locally and confirmed revoked server-side |
| MCPD-008A | Establish a GitHub-controlled, least-privilege path for every release after `v0.1.0` | M2 | Done | `MCPD-008` | Exact source [`47aa41e`](https://github.com/EnjoyableWork/mcp-doctor/commit/47aa41efee5eaf1ed81f699611748aff787ed971) and tap [`dafb41a`](https://github.com/EnjoyableWork/homebrew-tap/commit/dafb41ae86968285b5ae85f3dd633cc15103131b) controls, protected-environment and trusted-publisher readback, the [authorized and missing-environment rehearsal](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31443495330), [wrong-workflow rejection](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31441772215), [tap no-write rehearsal](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/31444057455), [ten-job channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31444142085), and final clean credential inventory pass without republishing `v0.1.0` |
| MCPD-009 | Add explicit, budgeted, deterministic `check` scenario replay and result-schema validation | M3 | Done | `MCPD-008A` | [Twenty-two built-binary active suites](tests/active.rs), the [strict replay adapter](src/contract/active.rs), and the disposable locked gate prove the `DEC-028`/`DEC-029` contract, consent rejection, ordered continuation, redaction, bounded schemas/results/reports, crash, incomplete, limit, and cleanup paths without a new dependency |
| MCPD-010 | Add a bounded Streamable HTTP transport with explicit remote-target and credential policy | M3 | Done | `MCPD-009` | The [bounded transport](src/transport/http.rs), [typed HTTP-header contract](src/contract/http_headers.rs), [application/report integration](src/contract/mod.rs), and [ten disposable built-binary HTTP/TLS journeys](tests/http.rs) prove exact target gates, classified and pinned addresses, peer checks, direct zero-redirect/retry/proxy connections, current-revision JSON/SSE and headers, verified identity, environment-only credential delivery and redaction, passive and authorized active authority, causal report parity, and every target, field, body, event, status, TLS, time, and resource bound without a real endpoint |
| MCPD-011 | Add the bounded adversarial `break` command for authorized tools | M3 | Done | `MCPD-010` | The [bounded generator](src/contract/generate.rs), [`break` application](src/break_command.rs), [nine disposable local journeys](tests/break.rs), and [exact-authority HTTP/TLS journey](tests/http.rs) prove versioned known-seed reproduction, schema-valid structural inputs, every generation and active-input bound, sequential continuation, redaction, exact tool/effect/target gates before activity, no schema retrieval or unauthorized connection, and resistant-process-tree cleanup without a new dependency |
| MCPD-012 | Stabilize machine reports and CI integration, then publish and independently verify the retained M3 journeys | M3 | Proposed | `MCPD-011` | Stable versioned JSON plus one accepted CI format preserve findings and exits; the `MCPD-008A` path publishes every expanded-release artifact and channel, and each passes its applicable installed smoke journey |
| MCPD-013 | Protect the default branch and define a contributor-compatible merge policy | M4 | Proposed | `MCPD-012` | A live public ruleset, credential-free verifier, normal protected pull request, rejected direct-update/deletion exercises, and documented emergency path prove the selected approval, check, bypass, signing, deletion, and non-fast-forward policy |
| MCPD-014 | Establish vulnerability disclosure and live repository-security controls | M4 | Proposed | `MCPD-013` | The recognized security policy, private route, supported-version and response contract, enabled entitled security features, non-disclosing verifier, and recorded clean baseline prove the scoped controls and limitations without exposing findings |
| MCPD-015 | Verify the public contribution, community, repository, and licensing contract | M4 | Proposed | `MCPD-014` | Public workflows and recognized community files, complete in-scope repository inventory, HTTPS-only official channels, and exact source, package, archive, and formula license evidence pass a credential-free verifier |
| MCPD-016 | Harden dependency maintenance and the CI, artifact, and distribution supply chains | M4 | Proposed | `MCPD-015` | Reviewed non-auto-merged dependency update proposals preserve exact direct requirements and pass maintenance/provenance/graph checks; full-SHA action inventory, fork and permission policy, tracked-artifact rejection, authenticated distribution verification, negative exercises, and operator audit pass against exact `main` and the immutable release |
| MCPD-017 | Establish organization access, credential, ownership, and recovery policy | M4 | Proposed | `MCPD-016` | Strong MFA, lowest-default access, deliberate grants and repository creation, scoped applications and automation, explicit owner-continuity decision, private recovery exercise, and a non-disclosing live verifier pass |
| MCPD-018 | Self-assess, publish, and maintain the enterprise assurance baseline | M4 | Proposed | `MCPD-017` | Every selected OSPS Level 1 control has public evidence or exact applicability reasoning; the official assessment and badge are verified on exact `main`; exact M3 artifacts receive a correctly scoped SLSA Build L2 evaluation; and claim-review and removal triggers are documented |

## Dependency and testing-tool introduction plan

Dependencies and testing tools enter with the first ticket that demonstrates a
real need. They support that ticket rather than becoming independent side
quests, and they are not added early to prepare for hypothetical work. Runtime,
build, development, and standalone diagnostic tools all execute code within the
project's trust boundary and receive the same maintenance, provenance, license,
advisory, source, and transitive-graph review.

An adoption review must show why the standard library and existing graph are
insufficient; identify the narrow capability and selected features; and examine
upstream stewardship, release and issue activity, security response,
ownership/provenance changes, unsafe code and build scripts, Rust and platform
support, licenses, advisories, duplicate versions, and build/runtime/artifact
cost. Popularity and release recency are useful signals but are not substitutes
for credible maintainership, focused tests, or a safe replacement path.

Every direct registry requirement is exact (`=x.y.z`). The committed
`Cargo.lock` fixes the complete resolved graph and registry checksums, and
normal commands use `--locked`. These controls make changes visible and
repeatable; they do not prove that selected source is benign. Automated update
pull requests are proposals only. Each update rechecks upstream ownership and
activity, release notes, the manifest and lockfile diff, features, new
transitives, licenses, advisories, Rust/platform impact, and affected behavior
before merge. Git sources, alternate registries, pre-releases, broad features,
or advisory/license exceptions require an accepted decision.

### Current direct dependency baseline

The 2026-08-11 review found every selected direct version stable, every
declared upstream repository active and not archived, the locked graph free of
known advisories under `cargo-deny`, and all selected licenses accepted or
narrowly documented. `reqwest` and `rustls` are their current stable releases;
`base64` deliberately retains the exact stable `0.22.1` already selected by the
HTTP graph instead of adding current `0.23.1` as a duplicate. This is dated
adoption evidence, not a promise about future maintenance; an update, ownership
change, advisory, unexplained inactivity, or project-need change triggers
re-review.

| Dependency | First owning ticket | Narrow role and current review boundary |
| --- | --- | --- |
| [`base64` `=0.22.1`](https://crates.io/crates/base64/0.22.1) | `MCPD-010` — Done | Exact standard Base64 encoding for the specification's unsafe-header-value sentinel; direct defaults are disabled and only `alloc` is requested, while reusing the same exact version already required by `reqwest` avoids a second implementation or graph version |
| [`clap` `=4.6.6`](https://crates.io/crates/clap/4.6.6) | `MCPD-002` — Done | CLI parsing and generated help through the selected `derive` feature; the active clap-rs upstream, Rust floor, permissive license, graph, and built-binary CLI behavior are reviewed |
| [`serde` `=1.0.229`](https://crates.io/crates/serde/1.0.229) and [`serde_json` `=1.0.151`](https://crates.io/crates/serde_json/1.0.151) | `MCPD-004` — Done | Typed protocol/report serialization and strict JSON parsing; active serde-rs stewardship, Rust floors, permissive licenses, graph, deterministic fixtures, and redaction boundaries are reviewed |
| [`tokio` `=1.53.1`](https://crates.io/crates/tokio/1.53.1) | `MCPD-005` — Done; `net` expanded by `MCPD-010` | Only the async process, I/O, timer, macro, runtime, and network features needed by bounded STDIO and one-shot DNS/HTTP; active upstream, Rust floor, MIT license, feature graph, timing, cleanup, and cross-platform behavior are reviewed |
| [`process-wrap` `=9.1.0`](https://crates.io/crates/process-wrap/9.1.0) | `MCPD-005` — Done | Process-group, Windows Job Object, Tokio, and kill-on-drop control with defaults disabled; watchexec stewardship, Rust floor, permissive license, feature graph, and resistant-descendant cleanup are reviewed |
| [`jsonschema` `=0.49.9`](https://crates.io/crates/jsonschema/0.49.9) | `MCPD-006` — Done | Draft 2020-12 validation with defaults and retrieval features disabled plus a rejecting retriever; active upstream, Rust floor, MIT license, transitive-license exceptions, no-network graph, bounds, and no-retrieval evidence are reviewed |
| [`reqwest` `=0.13.4`](https://crates.io/crates/reqwest/0.13.4) | `MCPD-010` — Done | Maintained asynchronous HTTP framing and a rustls/platform-verifier client; defaults are disabled and only `rustls-no-provider` is selected, while the application separately fixes HTTP/1.1, direct connections, identity encoding, no redirects/retries/decompression, one origin, and pinned resolution |
| [`rustls` `=0.23.43`](https://crates.io/crates/rustls/0.23.43) | `MCPD-010` — Done | Direct selection and installation of the `ring` crypto provider plus TLS 1.2/1.3 configuration and typed TLS-error recognition; defaults are disabled and only `ring`, `std`, and `tls12` are selected |
| [`rcgen` `=0.14.9`](https://crates.io/crates/rcgen/0.14.9) | `MCPD-010` — Done | Development-only generation of a fresh disposable CA and loopback server identity, preventing checked-in leaf expiry from making native TLS evidence time-dependent; defaults are disabled and only the required `pem` and `ring` features are selected |
| [`tempfile` `=3.27.0`](https://crates.io/crates/tempfile/3.27.0) | `MCPD-002` — Done | Development-only ownership of disposable test roots; active upstream, Rust floor, permissive license, graph, cleanup, and synthetic-path isolation are reviewed |

#### MCPD-010 HTTP/TLS adoption review — 2026-08-11

The standard library does not provide verified TLS or a maintained HTTP client,
and implementing HTTP framing, certificate validation, or cryptography inside
`mcp-doctor` would violate the dependency and network-safety policy. The
selected crates keep those semantics in maintained implementations while the
application owns the stricter target, authorization, resolver, retry, proxy,
redaction, and resource boundaries. `base64` is used for one exact wire
encoding already present in that graph rather than duplicating a subtle
protocol representation locally.

Registry ownership and repository identity were checked against crates.io and
the declared upstreams. [`reqwest`](https://github.com/seanmonstar/reqwest) is
owned and maintained by Sean McArthur; `0.13.4` was released 2026-05-25.
[`rustls`](https://github.com/rustls/rustls) is published by its established
maintainers and rustls organization; `0.23.43` was released 2026-07-29 and its
[security policy](https://github.com/rustls/rustls/security/policy) covers the
current line, private reporting, fixes, regression tests, releases, and RustSec
advisories. [`base64`](https://github.com/marshallpierce/rust-base64) retains
its two established crates.io owners; `0.22.1` was released 2024-04-30 and the
repository remained active through 2026-08-05. All three repositories were
active and unarchived during review with no unexplained registry/repository
provenance change. `reqwest` and `base64` do not publish a dedicated security
policy; their active owner and issue/release paths plus RustSec monitoring are
accepted for these focused roles, and that weaker formal response surface is
an explicit replacement or re-review trigger.

The direct crates declare permissive MIT/Apache-2.0, MIT/Apache-2.0, and
Apache-2.0/ISC/MIT licenses and Rust floors 1.48, 1.85, and 1.71 respectively;
the graph's effective floor is below the repository's Rust 1.97 toolchain.
`base64` and `rustls` forbid unsafe Rust. `base64` and `reqwest` have no build
script; the selected `rustls` build script only gates an unselected nightly
`read_buf` optimization. `reqwest` contains small unsafe buffer adapters, and
the transitive `ring` `0.17.14` provider owns reviewed native C/assembly, unsafe,
and `cc` build surfaces. Platform-verifier FFI and trust packages are selected
per target. These concentrated surfaces, especially `ring`, platform trust,
and the HTTP parser, require focused TLS, malformed-response, native, advisory,
and ownership rechecks on every update.

The change adds 72 locked crates for HTTP, URL/IDNA, async connection, TLS,
cryptography, and target-specific macOS, Windows, Linux, Android, and WebAssembly
support. Default `reqwest` features such as default TLS, charset conversion,
HTTP/2, and system-proxy discovery are not selected; automatic compression is
also absent. Default `rustls` features such as `aws-lc-rs`, logging, and
post-quantum exchange are not selected. The application selects `ring`, `std`,
and TLS 1.2 support, explicitly limits the client to HTTP/1.1, disables proxy
and replay behavior at construction, and proves the runtime settings with trap
fixtures. `cargo tree --duplicates` reports only reviewed `getrandom` 0.2/0.3/
0.4, `syn` 2/3, and `windows-sys` 0.52/0.61 transitions; `deny.toml` names those
exact exceptions and scopes the additional `ring`, `rustls-webpki`, `subtle`,
`untrusted`, and root-certificate data licenses. The complete all-feature
locked graph passes advisory, ban, license, and source policy.

Static TLS leaf fixtures are not acceptable evidence because platform-enforced
validity limits would make them expire. Development-only
[`rcgen`](https://github.com/rustls/rcgen) `0.14.9`, released 2026-08-10, creates
one process-local CA and IP-identity leaf with a 397-day-or-shorter window based
on the current UTC year; the random key and certificate never enter a report or
tracked artifact. The rustls-owned repository and its two established crates.io
publishers were active and unarchived, the crate is MIT/Apache-2.0, declares
Rust 1.88, forbids unsafe Rust, and has no build script. Defaults are disabled;
only `pem` and the already selected `ring` provider are active. Its 19 additional
lock entries cover certificate encoding/time support and optional parser
metadata, while the active test tree reuses `ring` and adds only `pem`, `time`,
and `yasna` support. `rcgen` has no dedicated security policy; its narrow
test-only role, easy removal boundary, active rustls stewardship, and the same
advisory/ownership re-review triggers make that residual acceptable. No rcgen
code or transitive is linked into the release binary.

On the 2026-08-11 ARM64 macOS review host, the first graph-changing release
build with a populated prior project cache completed in 16.34 seconds; the
current optimized binary is 11,234,320 bytes, 100 `--version` starts completed
in 0.67 seconds, and all ten disposable HTTP/TLS built-binary journeys completed
in 0.97 seconds inside the isolated gate. These are cost observations, not
performance guarantees. Native hosted and exact-artifact release gates remain
mandatory before publishing this implementation.

`MCPD-009` reuses `serde_json` for its versioned JSON scenario and RFC 6901
pointer resolution, `jsonschema` for bounded local input and output validation,
and the standard library for explicit environment lookup. It adds no dependency.
No new scenario parser, interpolation engine, secret-store package, or
assertion runtime is authorized without a measured need and the complete
adoption review.

`MCPD-011` reuses the same `serde_json` values, bounded `jsonschema` validator,
typed limits, active conversation, reporters, and local/HTTP transports. Its
fixed-width deterministic selector and bounded synthesizer use the standard
library and add no dependency. Table-driven schema cases, exact-seed unit
fixtures, and disposable built-binary limit, authorization, redaction, network,
and cleanup journeys expose the required invariants directly; no measured
shrinking, corpus, crash-artifact, or parser-gap need justifies a property or
fuzz framework for this ticket.

### Testing methods and candidate tools

`Baseline` is always authoritative. `Adopted` means the owning ticket
completed the review and evidence. `Conditional` means the ticket must
evaluate the stated trigger and record adoption or rejection without adding a
dependency when ordinary Rust tests remain clearer. `Diagnostic` tools inform
test design and remediation rather than impose an arbitrary numeric gate.
Versions for future tools are selected and pinned only when their ticket starts.
The ticket owner may make that evidence-backed choice without a separate owner
approval; rejection remains the default, and any adoption must satisfy the
complete dependency policy and be visible in the pull request.

| Tool or method | First owning ticket | Commitment | Use or adoption trigger |
| --- | --- | --- | --- |
| Rust `#[test]` and `cargo test` | `MCPD-002` / `MCPD-003` — Done | Baseline | Keep `cargo test --workspace --all-targets --all-features --locked` authoritative and test at the narrowest useful layer |
| Checked-in synthetic protocol, report, catalog, and process fixtures | `MCPD-004` through `MCPD-006` — Done | Adopted method | Prefer small reviewable fixtures, disposable roots/processes/sockets, exact structural assertions, and unmistakably synthetic redaction sentinels |
| `rcgen` | `MCPD-010` — selected 2026-08-11 | Adopted development dependency | Generate a process-local CA and loopback leaf only for disposable built-binary TLS journeys; never persist its generated private key or use it for product certificate handling |
| `tempfile` | `MCPD-002` — Done | Adopted development dependency | Reuse the reviewed disposable-root boundary; do not add another temporary-resource package for convenience |
| `cargo-deny` | `MCPD-003` — Done | Adopted development/CI tool | Keep advisory, license, ban, and source checks locked; pin the CI action by full commit SHA and review tool-policy changes |
| `assert_cmd` | 2026-08-10 review | Not adopted | The existing standard-library built-binary harness already controls arguments, environment, time, output, and process fixtures; reconsider only if duplicated orchestration becomes harder to review safely |
| `insta` | 2026-08-10 review | Not adopted | Current human/JSON golden files and catalog fixtures remain small and explicit; reconsider only when direct snapshots become materially harder to review |
| `cargo-nextest` `0.9.143` | `MCPD-007` — evaluated 2026-08-10 | Not adopted | The 71-test baseline is already fast except for one intentional timeout and has no measured partitioning, isolation, or flake-diagnosis problem; authoritative `cargo test` remains clearer |
| `cargo-llvm-cov` `0.8.7` | `MCPD-007` — evaluated 2026-08-10 | Diagnostic only; not adopted | Its copied-tree result cannot observe cleared-environment built-binary subprocesses accurately; preserve the isolation boundary, convert credible gaps into tests, and add no percentage gate |
| `cargo-mutants` `27.1.0` | `MCPD-007` — evaluated 2026-08-10 | Diagnostic only; not adopted | A copied-tree five-mutant primary-diagnosis subset produced three useful test gaps and then passed 5/5; standard focused tests retain the value without permanent tool execution |
| GitHub artifact and build-provenance Actions | `MCPD-008` — selected 2026-08-10 | Adopted release tooling | Current active official GitHub Actions are pinned by full commit SHA; short-lived workflow artifacts assemble the exact payload, while GitHub attestations bind each published byte to the tag workflow and commit without a stored signing secret |
| Syft `1.50.0` through `anchore/sbom-action` `0.24.0` | `MCPD-008` — selected 2026-08-10 | Adopted release tooling | Current active Apache-2.0 Anchore tooling produces target SPDX 2.3 JSON for the two represented GNU/Linux archives; generation runs only in disposable hosted release jobs and the output contract is checked independently |
| `Homebrew/actions/setup-homebrew` `2026.08.03.2` | `MCPD-008` — selected 2026-08-10 | Adopted release tooling | The current official Homebrew action is pinned by full commit SHA and used only to style, audit, source-build, test, and smoke the exact formula on represented native hosts |
| [`rust-lang/crates-io-auth-action` `v1.0.5`](https://github.com/rust-lang/crates-io-auth-action/releases/tag/v1.0.5) at `c6f97d42243bad5fab37ca0427f495c86d5b1a18` | `MCPD-008A` — selected 2026-08-10 | Adopted and live-verified | The official Rust project Action is active, unarchived, dual MIT/Apache-2.0, Node 24 bundled, released from a verified commit, masks its 30-minute token, and revokes it in its post step; it is full-SHA-pinned in the [verified authorized and rejected OIDC paths](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31443495330), with no new Cargo dependency or stored secret |
| GitHub-native Homebrew tap update authority | `MCPD-008A` | Adopted and live-verified | The separate tap owns a manual rehearsal/publish workflow; its read-only job authenticates the annotated immutable upstream source, provenance, checksums, package hash, and formula without executing upstream code, while only an approved publish-mode job receives the tap's short-lived `contents: write` `GITHUB_TOKEN` and may copy `Formula/mcp-doctor.rb`; the protected no-write [hosted rehearsal](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/31444057455) passes, and no cross-repository PAT or source-repository tap write exists |
| `proptest` or another property framework | `MCPD-011` — evaluated 2026-08-11 | Not adopted | Exact-seed unit fixtures and table-driven valid, unsatisfiable, reference, and limit schemas expose deterministic generator invariants without a measured shrinking gap or another executable dependency |
| `cargo-fuzz` or another fuzz harness | `MCPD-010` / `MCPD-011` — evaluated 2026-08-11 | Diagnostic only; not adopted | Disposable framing, schema, generator, limit, redaction, and cleanup journeys cover the identified boundaries without an unresolved parser or crash-corpus gap; a future ticket must identify a finite target, timeout, artifact policy, and no-network execution need before adoption |

Every introduction ticket owns the complete adoption: recheck the then-current
release and upstream state, update `Cargo.toml` and `Cargo.lock` or record the
exact standalone tool version, add the first meaningful test, wire only the
necessary local/CI command, document developer use, and link durable evidence
from the ticket. An evaluated rejection is valuable evidence and remains the
default when a new package would not materially improve correctness, safety, or
diagnosis.

## Canonical goal objectives

Use the matching objective when beginning an eligible main-story ticket:

| Ticket | Objective |
| --- | --- |
| MCPD-002 | Complete `MCPD-002`: bootstrap the original Rust 2024 `mcp-doctor` binary with truthful help/version behavior, a committed lockfile, isolated built-binary tests, and verified package metadata. Do not begin protocol behavior. Finish when every ticket acceptance check passes and durable evidence is recorded. |
| MCPD-003 | Complete `MCPD-003`: establish disposable POSIX and Windows quality gates, dependency policy, least-privilege cross-platform CI, scheduled maintenance, and public contribution, support, conduct, and private-security entry points. Do not begin protocol behavior or release automation. Finish when local evidence is recorded and the first hosted matrix passes. |
| MCPD-004 | Complete `MCPD-004`: define the supported MCP revision matrix, diagnostic findings, limits, redaction, performed/skipped semantics, and exit/report contracts using synthetic fixtures. Do not start a real process or network transport. Finish when decisions and focused tests make the contract unambiguous. |
| MCPD-005 | Complete `MCPD-005`: implement a literal-argument, constrained-environment STDIO boundary with bounded messages, output and time plus guaranteed process-tree cleanup and reap. Do not call tools or begin HTTP support. Finish when the full transport failure matrix passes. |
| MCPD-006 | Complete `MCPD-006`: diagnose advertised tools, prompts, resources, and bounded JSON Schema 2020-12 contracts without implicit tool execution or external schema retrieval. Give each failure a safe expectation and corrective next step. Finish when valid and invalid fixture catalogs produce deterministic, redacted, actionable findings. |
| MCPD-007 | Complete `MCPD-007`: wire the passive local journey through the built binary; identify the earliest actionable failing layer while preserving independent safety failures and causally linked skips; expose equivalent redacted human and experimental `mcp-doctor.report/v1alpha1` JSON output; prove report-only actionability; test pinned official SDK examples and independent implementations spanning at least two languages; apply `DEC-024` to record an evidence-matched broad or readiness/migration position and block completion if no credible independent implementation passes; recheck public registry identity under `DEC-008`; and evaluate and record the conditional test-runner, coverage, and mutation-tool decisions under `DEC-025` without creating an arbitrary numeric release gate. Do not call tools, add HTTP, implement an older revision, or claim official conformance. Finish when a human or agent can use the report alone to determine what failed, where, why, what was expected, and what to change next. |
| MCPD-008 | Complete `MCPD-008`: publish and independently verify the first immutable passive-MVP version through GitHub Releases, crates.io, and source-built Homebrew, with deterministic packages, checksums, SPDX SBOMs, attestations, and installed passive diagnostic smokes for every represented channel. Open the dated, non-sensitive M2 adoption checkpoint; do not add active or remote behavior. |
| MCPD-008A | Complete `MCPD-008A` only after the first crates.io publication: turn every later release into an intentional GitHub-controlled flow. Generalize stable-tag validation without weakening the reviewed version, source, preflight, annotation, immutability, or provenance contract; bind crates.io Trusted Publishing to the exact repository, workflow, and protected release environment through OIDC; and establish tap-owned or narrowly installed short-lived GitHub authority that copies only the exact verified formula. Publish downstream only after immutable GitHub bytes verify, then run credential-free channel verification. Store no long-lived crates.io token or broad personal access token, do not republish `v0.1.0`, and do not create a new version merely to test automation. Finish when the credential inventory is clean, negative authorization and byte-identity cases fail safely, and a nonpublishing end-to-end rehearsal proves every handoff before any later tag is allowed. |
| MCPD-009 | Complete `MCPD-009`: add `mcp-doctor.scenario/v1alpha1` JSON replay for one exact tool and 1–100 ordered reviewed cases; bounded advertised and scenario-provided local output-schema validation; environment-only target and argument secret references; exact per-run `--allow-tool`; required `read_only` or `side_effecting` classification; and an additional `--allow-side-effects` gate when applicable. Do not generate inputs, trust server annotations, interpolate values, expose arguments or results, or continue `input_required`. Finish when ordinary mismatches continue to later cases, unsafe failures stop calls, and active success, rejection, crash, silent-failure, incomplete, redaction, limit, and cleanup journeys pass without secret output or orphaned processes. |
| MCPD-010 | Complete `MCPD-010` under `DEC-030`: add direct, pinned, bounded Streamable HTTP `2026-07-28` diagnosis for public HTTPS by default, with exact private, loopback-cleartext, and credential-to-endpoint gates; verified TLS; environment-only pre-provisioned credentials; current protocol and bounded `x-mcp-header` fields; zero redirects, application retries, proxies, implicit OAuth discovery, or legacy fallback; and equivalent redacted reports for passive and authorized active journeys. Do not begin adversarial generation or claim OAuth client conformance. Finish when deterministic resolver plus disposable HTTP/TLS fixtures prove every target, address, peer, header, credential, JSON/SSE, status, TLS, redaction, time, and resource boundary without network escape. |
| MCPD-011 | Complete `MCPD-011` after `MCPD-010`: generate bounded deterministic boundary cases only for explicitly authorized tools, record reproducible seeds and structural inputs, and enforce schema and scenario limits. Finish when generation cannot widen target or execution scope. |
| MCPD-012 | Complete `MCPD-012`: stabilize the redacted machine-result contract and accepted CI reporter across every retained local and remote journey, then publish one protected immutable expanded version with authenticated artifacts and installed smokes for every represented channel. Do not retain an M3 feature that its owning ticket deferred or cancelled. |
| MCPD-013 | Complete `MCPD-013`: protect the default branch with a contributor-compatible public ruleset, deliberate approval, check, bypass, merge, deletion, non-fast-forward, and commit-signing choices; implement credential-free drift verification; and prove normal, rejected, and bounded emergency paths. Do not change immutable release bytes or begin later assurance tickets. |
| MCPD-014 | Complete `MCPD-014`: establish recognized supported-version, security-contact, private-reporting, response, and coordinated-disclosure guidance; enable and read back the entitled dependency, code-scanning, secret-prevention, and private-reporting controls; document unavailable features exactly; and verify a non-disclosing clean baseline. Do not publish a complete-baseline claim. |
| MCPD-015 | Complete `MCPD-015`: verify public contribution, conduct, support, defect-reporting, repository-inventory, official-channel, inbound-license, source-license, and released-asset license contracts across every in-scope repository and distribution channel. Avoid nominal reviewers, owners, or controls, and do not begin supply-chain changes. |
| MCPD-016 | Complete `MCPD-016`: automate grouped dependency update proposals without auto-merge and verify they preserve exact direct requirements plus the accepted maintenance, provenance, feature, transitive-graph, license, advisory, Rust/platform, and behavioral review; inventory and verify every selected Action at a reviewed full commit SHA; prove untrusted workflows are read-only and secretless; reject generated executables and unreviewable binary artifacts; and authenticate the immutable release, Cargo package, and Homebrew formula without changing published bytes. |
| MCPD-017 | Complete `MCPD-017`: define and verify strong-MFA, lowest-default-access, manual-grant, repository-creation, installed-application, automation-credential, ownership-continuity, and recovery controls using aggregate non-sensitive evidence. Any live organization mutation or private recovery confirmation requires explicit owner authority and must not expose identities or recovery material. |
| MCPD-018 | Complete `MCPD-018`: confirm the current official OSPS, BadgeApp, and SLSA versions; publish a dated and scoped crosswalk for every selected OSPS Level 1 control; complete the official self-assessment and obtain its official badge only after every control passes; verify that badge and evidence on exact `main`; evaluate only the exact M3 release artifacts against SLSA Build L2; and define annual, framework-change, issuer-status, security-incident, release-pipeline-change, organization-change, and evidence-drift review and removal triggers. Never imply independent certification, regulatory compliance, higher OSPS levels, all-artifact SLSA coverage, or paid platform signing. |

## M4 enterprise assurance boundary

M4 turns existing project, release, repository, and organization practices into
dated, scoped, independently inspectable adoption evidence. It is a
post-release assurance milestone, not another product release and not a reason
to rewrite, replace, or weaken the immutable M3 artifacts.

The planning baseline is the current
[OpenSSF OSPS Baseline `v2026.02.19`](https://baseline.openssf.org/versions/2026-02-19)
Level 1 checklist and the current
[SLSA `v1.2`](https://slsa.dev/spec/v1.2/) specification. `MCPD-018` must
recheck the official current versions and proof mechanisms before publishing a
claim. A superseding version or changed issuer process triggers a documented
scope decision and an updated control map rather than silent reuse of this
snapshot.

### OSPS Level 1 planning control map

The current OSPS planning baseline has 24 Level 1 controls. Existing files or
settings do not close a row by themselves: the owning ticket must verify their
effective public or live state and link durable evidence.

| Owning work | Level 1 controls | Count | Evidence boundary |
| --- | --- | --- | --- |
| `MCPD-013` | `OSPS-AC-03.01`, `OSPS-AC-03.02` | 2 | Enforced direct-update and primary-branch deletion protection |
| `MCPD-014` | `OSPS-BR-07.01`, `OSPS-VM-02.01` | 2 | Secret prevention, security contacts, and private disclosure route |
| `MCPD-015` | `OSPS-BR-03.01`, `OSPS-DO-02.01`, `OSPS-GV-02.01`, `OSPS-GV-03.01`, `OSPS-LE-02.01`, `OSPS-LE-02.02`, `OSPS-LE-03.01`, `OSPS-LE-03.02`, `OSPS-QA-04.01` | 9 | Public project channels, contribution and defect guidance, repository inventory, and source and released-asset licensing |
| `MCPD-016` | `OSPS-BR-01.01`, `OSPS-BR-01.03`, `OSPS-BR-03.02`, `OSPS-QA-05.01`, `OSPS-QA-05.02` | 5 | Untrusted CI isolation, authenticated distribution, and executable and binary artifact policy |
| `MCPD-017` | `OSPS-AC-01.01`, `OSPS-AC-02.01` | 2 | Strong MFA, deliberate permission assignment, and lowest-default access |
| Existing evidence revalidated by `MCPD-018` | `OSPS-DO-01.01`, `OSPS-QA-01.01`, `OSPS-QA-01.02`, `OSPS-QA-02.01` | 4 | Released user guide, public source and history, and direct dependency inventory |
| **Total** | **Every Level 1 control in the planning baseline** | **24** | `MCPD-018` links evidence or exact applicability reasoning for every control before any achieved-status claim appears |

### Public proof and maintenance policy

- Every assurance statement names its exact framework version, level, assessed
  repositories and release or organization boundary, assessment date,
  limitations, and evidence.
- When an accepted target provides an official badge or conformance mark, its
  owning ticket cannot be done until that proof is issued under the provider's
  terms, linked to the public assessment, and verified on exact `main`.
- Human OAuth, public account attribution, legal assertions, paid services, and
  live organization changes are explicit owner gates. Their absence cannot be
  hidden by inventing a badge or weakening the target.
- Private settings, identities, authentication factors, vulnerability or secret
  findings, credentials, and recovery material stay private. Public evidence
  uses non-sensitive structure, aggregate results, and stable verifier output.
- Revalidate claims on their stated schedule and after a framework, issuer,
  security, release-pipeline, organization-access, or evidence change. Correct
  or remove a stale, withdrawn, broken, or over-broad claim immediately.

### Assurance target board

This board is a scope boundary, not a second executable queue. Only OSPS Level
1 and the exact-artifact SLSA evaluation are accepted M4 work. The other rows
remain candidates until a later decision defines their value, cost, owner,
evidence, and official proof.

| Target | M4 role | Permitted claim boundary | Required public proof |
| --- | --- | --- | --- |
| [OpenSSF OSPS Baseline Level 1](https://baseline.openssf.org/) | Project-wide M4 gate against the exact current version selected at activation | Dated and scoped self-assessment; never independent certification or regulatory compliance | Official [BadgeApp](https://www.bestpractices.dev/) baseline self-certification badge linked to the public assessment and project crosswalk |
| [SLSA Build L2](https://slsa.dev/spec/v1.2/build-track-basics) | Required evaluation of only the exact M3 release artifacts with qualifying signed provenance | Artifact-specific result under the exact selected SLSA version; never a project-wide, all-channel, dependency, or future-release claim | Signed provenance plus a public verification record; no certification-like project badge |
| [OpenSSF Best Practices Passing](https://www.bestpractices.dev/en/criteria) | Candidate after M4 | Public project self-certification only after every applicable criterion passes | Official Passing badge linked to its assessment |
| [NIST SSDF](https://csrc.nist.gov/Projects/ssdf/publications) | Candidate gap assessment after M4 | Dated practice-alignment crosswalk against the selected final publication; never “NIST certified” | Scoped public evidence link; no unofficial certification shield |
| [OpenChain ISO/IEC 18974](https://openchainproject.org/security-assurance) and [ISO/IEC 5230](https://openchainproject.org/license-compliance) | Organization-process candidate after M4 | Only the organization program and scope accepted through the official process; never product certification | Applicable official self-certification or conformance proof under issuer terms |

## Decision log

| ID | Decision | Status | Date | Consequence |
| --- | --- | --- | --- | --- |
| DEC-001 | Implement the product in Rust 2024 | Accepted | 2026-08-09 | Optimize for a portable native executable, explicit types, and predictable local execution |
| DEC-002 | Begin as one modular binary crate | Accepted | 2026-08-09 | Split only for a real consumer or demonstrated boundary |
| DEC-003 | Keep README as north-star and PROJECT as delivery truth | Accepted | 2026-08-09 | Public clarity does not replace implementation evidence |
| DEC-004 | Make passive inspection the default and active tool execution explicit | Accepted | 2026-08-09 | No command may silently graduate from discovery to tool invocation |
| DEC-005 | Target MCP `2026-07-28` first and decide older revisions explicitly | Accepted | 2026-08-09 | Earlier handshake behavior is not copied or inferred |
| DEC-006 | Prove STDIO before Streamable HTTP | Accepted | 2026-08-09 | M1 establishes process, protocol, result, and safety invariants before network variation |
| DEC-007 | Use bounded JSON Schema 2020-12 evaluation without external retrieval by default | Accepted | 2026-08-09 | Schema correctness cannot create implicit network access or unbounded work |
| DEC-008 | Retain the MCP Doctor product and `mcp-doctor` executable under EnjoyableWork | Accepted | 2026-08-10 | Accept the cross-ecosystem command collision and distinguish this Rust CLI through its organization and official channels; prefer the `mcp-doctor` crate when rechecked at publication, otherwise publish `enjoyable-mcp-doctor` with the same executable and product identity |
| DEC-009 | First release uses Linux archives, Cargo source, and source-built Homebrew | Accepted | 2026-08-09 | Signed native macOS/Windows and WinGet remain later funded scope |
| DEC-010 | Track the ordered delivery plan in this repository | Accepted | 2026-08-09 | Hosted issues may supplement but do not replace milestone and decision truth |
| DEC-011 | Use an `inspect`, `check`, and `break` command family | Accepted | 2026-08-09 | `inspect` ships as the passive MVP; `check` and `break` are separately authorized M3 surfaces whose explicit authority and finite budgets cannot be weakened by adoption pressure |
| DEC-012 | Add a post-expansion M4 for evidence-backed enterprise assurance and adoption | Accepted | 2026-08-09 | M2 publishes the first passive release; M4 follows the independently verified M3 expansion release and orders governance, disclosure, security, community, licensing, supply-chain, organization, OSPS, and exact-artifact SLSA evidence without implying certification or regulatory compliance |
| DEC-013 | Support only modern MCP `2026-07-28` and recognize but never initialize the four earlier official revisions | Accepted | 2026-08-09 | M1 has one exact protocol contract; legacy-only and unknown advertisements fail with safe structured evidence rather than implicit fallback |
| DEC-014 | Make stable finding codes own severity and derive check state, overall outcome, and exit status from one result model | Accepted | 2026-08-09 | Callers cannot downgrade a code, turn a skip into a pass, or supply an inconsistent summary or exit code |
| DEC-015 | Permit only trusted structural context and typed redacted evidence in ordinary reports | Accepted | 2026-08-09 | Human and JSON output cannot retain arbitrary values; `MCPD-007` exposes `v1alpha1` as explicitly experimental and `MCPD-012` owns stabilization |
| DEC-016 | Adopt the finite M1 default limit profile recorded in the MCPD-004 contract | Accepted | 2026-08-09 | Every later boundary must enforce and report these simultaneous caps or explicitly revise the contract with evidence |
| DEC-017 | Make one-command passive local preflight the first adoption wedge | Accepted | 2026-08-09 | M1 must answer startup, supported protocol, advertised-contract, bound, and cleanup health without tool execution; interactive inspection, official conformance, security scanning, and LLM evaluation stay outside the MVP |
| DEC-018 | Publish the passive MVP before active or remote expansion | Accepted | 2026-08-09 | M2 distributes and independently verifies `inspect`; active scenarios, HTTP, and adversarial generation cannot become first-release prerequisites |
| DEC-019 | Gate M3 expansion on dated adoption evidence | Superseded | 2026-08-09 | `DEC-027` replaces the hard gate after the checkpoint closed with zero independent reports; the original decision remains visible rather than being rewritten |
| DEC-020 | Make earliest-actionable-layer diagnosis and report-only correction the project-wide north star | Accepted | 2026-08-09 | Every journey prioritizes causal diagnosis over check count, preserves independent safety failures, marks dependent skips, and gives humans and agents the same sufficient evidence; breadth without this behavior cannot satisfy a milestone |
| DEC-021 | Run each local target as a directly launched, minimally provisioned managed process tree | Accepted | 2026-08-09 | Arguments are literal; only platform launch variables are inherited; batch targets that require a shell are rejected; bounded pipes, Unix process groups, Windows Job Objects, forced termination, and wait completion form one transport boundary |
| DEC-022 | Validate M1 tool schemas as local JSON Schema Draft 2020-12 with a maintained validator and no retrieval features | Accepted | 2026-08-10 | `jsonschema` is locked with default features disabled and a rejecting retriever; exact MIT-0 and Zlib transitive licenses are reviewed with crate-scoped exceptions; unsupported dialects and external references receive typed findings instead of fallback or I/O |
| DEC-023 | Admit dependencies and testing tools only at demonstrated need under one dated review and exact-resolution policy | Accepted | 2026-08-10 | Standard-library and existing-graph reuse remain the default; every direct registry requirement is exact, the complete graph is committed and used with `--locked`, and each adoption or update rechecks maintenance, provenance, security, features, graph, platform, cost, and focused evidence without treating version pinning as proof of trust |
| DEC-024 | Keep M1 on MCP `2026-07-28` and let real-server evidence determine release positioning | Accepted | 2026-08-10 | Passing all selected current-revision official and independent cases across at least two languages permits broad current-revision positioning; narrower reach with at least one credible independent pass requires explicit readiness/migration positioning and a separate compatibility ticket; no credible independent pass blocks M1 and M2 rather than adding legacy behavior implicitly |
| DEC-025 | Let each owning ticket resolve conditional testing tools from evidence | Accepted | 2026-08-10 | Rejection is the default; `MCPD-007` may adopt a runner, coverage tool, or mutation tool without separate owner approval only when a concrete measured need, complete dependency/tool review, focused use, exact pin, and pull-request evidence justify it |
| DEC-026 | After the first crate publication, publish releases through GitHub with ephemeral or narrowly scoped authority | Accepted | 2026-08-10 | `MCPD-008A` generalizes the tag workflow, uses crates.io OIDC Trusted Publishing, gives the separate tap only bounded short-lived update authority, and preserves exact immutable bytes plus independent verification; humans still approve release changes and create the annotated tag, and no long-lived crates.io token or broad personal access token is permitted |
| DEC-027 | Treat independent adoption evidence as nonblocking M3 prioritization input | Accepted | 2026-08-10 | Issue 5 closed with zero independent reports and no adoption claim; M3 may proceed when each ticket's predecessor, design decisions, safety boundary, and acceptance evidence are ready, while future feedback may still reprioritize, narrow, defer, or cancel work |
| DEC-028 | Resolve `OPEN-04` with one versioned JSON scenario and environment-only secret-reference boundary | Accepted | 2026-08-10 | `mcp-doctor.scenario/v1alpha1` names one exact tool and 1–100 ordered cases; existing JSON and bounded local-schema machinery owns parsing and expectations; only explicit target-environment names and RFC 6901 pointers to null argument placeholders may resolve invoking-process environment values; no interpolation, secondary parser, secret store, external schema, or arbitrary value reaches reports |
| DEC-029 | Resolve `OPEN-05` with redundant exact active authorization and no automatic continuation | Accepted | 2026-08-10 | Each scenario declares `read_only` or `side_effecting`, every run repeats the exact tool through `--allow-tool`, side effects also require `--allow-side-effects`, annotations and wildcards never authorize, and `input_required` remains incomplete without elicitation or another retry |
| DEC-030 | Resolve `OPEN-06` with one direct, pinned, credential-scoped Streamable HTTP endpoint | Accepted | 2026-08-10 | Public HTTPS is the default; exact endpoint gates bound eligible private, loopback-cleartext, and credential use; DNS answers are classified once and pinned; TLS identity is mandatory; headers and JSON/SSE are finite; redirects, application retries, proxies, ambient credentials, automatic OAuth discovery, legacy fallback, and value-bearing reports are prohibited |
| DEC-031 | Generate only versioned, bounded, schema-valid cases for one redundantly authorized tool | Accepted | 2026-08-11 | `break` requires matching `--tool` and `--allow-tool`, an explicit effect classification, 1–100 cases, a seed, and side-effect consent when applicable; `mcp-doctor.generator/v1` uses fixed-width deterministic selection, finite candidates and work, existing local schema validation, sequential calls, and value-free structural reproduction evidence without another dependency, target source, schema retrieval, or authority derived from discovery or annotations |

## Open decisions

`OPEN-04`, `OPEN-05`, and `OPEN-06` are accepted as `DEC-028`, `DEC-029`, and
`DEC-030`; their owning `MCPD-009` and `MCPD-010` tickets are implemented
locally. `DEC-031` records the implemented `MCPD-011` generation boundary. The
remaining entries belong to their listed later tickets.

| ID | Decision needed | Needed by | Default if unresolved |
| --- | --- | --- | --- |
| OPEN-07 | Stable machine-output version and first additional CI reporter | `MCPD-012` | Retain experimental `v1alpha1` until evidence supports a stable version; evaluate JUnit versus SARIF from the intended consumer |
| OPEN-08 | Exact OSPS, BadgeApp, and SLSA versions and proof mechanisms at M4 activation | `MCPD-013` | Use the then-current official versions; planning baseline is OSPS `v2026.02.19` Level 1 and SLSA `v1.2` Build L2, with a documented update if either is superseded |
| OPEN-09 | Default-branch approval count, required checks, merge methods, bypass, emergency administration, and commit-signing policy | `MCPD-013` | Prevent direct updates and deletion with strict current checks and no standing bypass; do not require an unavailable independent reviewer or unproven signature path |
| OPEN-10 | Organization membership, application, automation-credential, owner-continuity, and private recovery boundary | `MCPD-017` | Lowest default access, deliberate grants, strong MFA, scoped automation, explicit residual-risk acceptance, and non-disclosing recovery evidence |

## Risk register

| ID | Risk | Impact | Mitigation and escalation trigger | State |
| --- | --- | --- | --- | --- |
| RISK-01 | A diagnostic invokes a mutating tool unexpectedly | Critical | Passive default plus `DEC-029`/`DEC-031` exact configuration, effects, tool, seed/case, and side-effect gates with consent and rejection tests; any implicit, mismatched, wildcard, annotation-derived, or continued call blocks every later release | Mitigated locally for reviewed `check`, generated `break`, and exact remote active paths by `MCPD-009` through `MCPD-011`; hosted exact-artifact evidence remains required |
| RISK-02 | A timed-out server or descendant remains running | Critical | Managed process tree, shutdown bounds, termination, reap, and resistant-child fixtures; any surviving PID blocks release | Mitigated for M1 by the complete hosted native process matrix and retained locally by reviewed and generated active cleanup journeys; future exact artifacts must retain it |
| RISK-03 | Secrets or raw production values reach output | High | Structural redaction and sentinel tests across errors, reports, debug surfaces, fixtures, and the `DEC-028` environment-only secret boundary; any observed name or value blocks release | `MCPD-009` proves target/argument secret rejection and `MCPD-011` proves structural-only generated reproduction in human/JSON output; the risk remains open for every later boundary and exact release artifact |
| RISK-04 | Protocol evolution makes diagnostics incorrect | High | Revision-specific rules and fixtures with explicit unsupported outcomes; a new release triggers contract review | Open |
| RISK-05 | Pathological schema or output exhausts resources | High | Depth, bytes, errors, cases, time, and reference limits; an unbounded input path blocks release | Mitigated for passive and reviewed active paths plus `MCPD-011` candidate, synthesis, schema, instance, aggregate-input, case, and report work; later boundaries and exact artifacts require their own evidence |
| RISK-06 | Remote diagnosis enables SSRF or credential leakage | Critical | `DEC-030` fixes exact target gates, IANA-based address classification and pinning, verified TLS, credential-to-endpoint consent, direct zero-redirect/retry connections, finite headers/bodies, and value-free reports; any bypass, peer drift, implicit network source, or secret output blocks completion | Mitigated locally for bounded passive, reviewed, and generated activity through one `MCPD-010` transport and an `MCPD-011` exact-authority network journey; hosted native evidence and every future multi-origin boundary must reprove it |
| RISK-07 | Generated cases are irreproducible or exceed authorized scope | High | Versioned stable seed selection, ordered generation, structural evidence, exact tool/effect/target gates, and finite cases, candidates, inputs, work, and concurrency; mismatch blocks active testing | Mitigated locally by `DEC-031`, exact-seed fixtures, local and HTTP authorization rejection, structural redaction, every generation limit, and sequential execution evidence; hosted exact-artifact verification remains |
| RISK-08 | A passing report creates false confidence after skipped checks | High | Per-check performed/skipped state and non-ambiguous summary; any hidden skip blocks release | Mitigated for M1 by hosted human/JSON causal-skip and authorization journeys; new checks must preserve the invariant |
| RISK-09 | Broad protocol, transport, and reporting scope delays a usable slice | High | M1 ends at passive `inspect`, M2 publishes it, and M3 stays an ordered set of bounded vertical tickets; any broad feature becoming a prerequisite for an earlier completed slice escalates | Mitigated by the ordered plan and `DEC-027`; voluntary evidence may reprioritize work, but its absence neither authorizes breadth nor blocks scoped work |
| RISK-10 | The public identity is unavailable, ambiguous, or confused with an existing command before publication | High | `DEC-008` retains the product and executable under EnjoyableWork, accepts the cross-ecosystem collision, defines a Cargo-package fallback, and requires exact official-channel guidance plus an immediate pre-publication registry recheck | Mitigated for the first release: the preferred `mcp-doctor` crate identity is published under the exact EnjoyableWork source and metadata; future channel guidance must preserve the distinction |
| RISK-11 | A release channel installs bytes not represented by the immutable release | Critical | `MCPD-008` proves exact package/formula equality, checksums, attestations, and native installed smokes for the first release; `MCPD-008A` makes those checks preconditions for every later downstream write; any mismatch requires a new version | Mitigated for `v0.1.0` and the repeat-release path by byte-identical Cargo and Homebrew handoffs, rejected mismatch cases, and successful channel verification; every future release must retain the same immutable-byte gates |
| RISK-12 | An unprotected default branch permits direct, destructive, or insufficiently reviewed changes | High | `MCPD-013` requires an enforced public ruleset, drift verifier, rejected-path exercises, and a bounded emergency process; any unverified bypass or destructive path blocks M4 | Deferred with M4 |
| RISK-13 | A contributor publicly exposes a vulnerability, credential, or unsafe diagnostic because reporting and prevention controls are incomplete | High | `MCPD-014` verifies private reporting, safe guidance, entitled scanning and prevention controls, limitations, and a non-disclosing baseline; any public sensitive report or hidden finding blocks M4 | Deferred with M4 |
| RISK-14 | Mutable automation, privileged untrusted code, or unauthenticated distribution compromises the project or its releases | Critical | `MCPD-008A` limits repeat publication to reviewed full-SHA automation, OIDC or narrowly scoped short-lived authority, immutable-byte preconditions, and negative authorization tests; `MCPD-016` later audits the complete CI and distribution boundary; any drift or credential exposure blocks publication and M4 | The first release removed and revoked its one-time credential; the completed repeat-release rehearsal proves exact OIDC and tap authority plus a clean credential inventory at the merged commits, while future drift blocks publication and full assurance remains `MCPD-016` |
| RISK-15 | Organization-owner loss or over-broad long-lived credentials become an undocumented recovery dependency | High | `MCPD-017` verifies strong MFA, lowest access, application and credential scope, owner continuity, and private recovery evidence; unresolved access or recovery assumptions block M4 | Deferred with M4 |
| RISK-16 | A stale, unofficial, or over-broad assurance claim misleads adopters | High | `MCPD-018` binds every claim to exact version, scope, date, official proof, public evidence, and removal triggers; missing, stale, withdrawn, or ambiguous proof blocks or removes the claim | Deferred with M4 |
| RISK-17 | Technically correct findings become an undifferentiated failure list that does not help a developer repair a server or earn repeat use | High | Every MVP failure identifies the expected earliest actionable layer, preserves independent safety failures, links downstream skips to their cause, and includes safe what, where, why, expectation, remediation, and versioned-rule evidence; report-only cases, maintainer trials, and voluntary feedback record unclear findings, false findings, time to value, and repeat use | M1 report sufficiency passes locally and hosted; the checkpoint closed with zero independent reports and no adoption claim, while future feedback may reprioritize M3 |
| RISK-18 | Latest-only protocol support excludes too much of the reachable ecosystem for a useful first release | High | `DEC-024` requires a controlled official/independent matrix spanning at least two languages: complete selected current-revision success permits broad positioning, narrower credible reach requires readiness/migration positioning and a separate compatibility ticket, and no credible independent pass blocks completion without silently adding legacy behavior | Four selected current-revision servers across four languages passed locally and hosted before M2 release; future protocol revisions reopen the risk |
| RISK-19 | An unnecessary, stale, compromised, or silently widened dependency executes in the product, developer environment, or CI supply chain | Critical | Default to no addition; require an owning need and dated maintenance/provenance/security/graph review; use exact direct requirements, a committed lockfile, narrow features, reviewed sources, `cargo-deny`, non-automatic update approval, and a regression check; removal, unexplained upstream inactivity, ownership change, advisory, new build script/unsafe surface, or unreviewable lockfile growth triggers escalation | Mitigated locally by the MCPD-003 policy refinement; complete live update and supply-chain verification remains `MCPD-016` |

## Readiness and completion gates

### Ticket ready

A ticket is ready when it has one observable outcome, an eligible predecessor,
explicit acceptance evidence, resolved or recorded decisions, an owner, and no
conflict with the work-in-progress limit.

### Ticket done

A ticket is done when the outcome and important failure paths work; focused and
broader checks pass; safety, redaction, and protocol claims remain accurate;
public documentation is updated; and durable evidence is linked from its row.

### Passive MVP complete

M1 is complete only when:

- the built `mcp-doctor inspect -- <executable> [arguments...]` journey works on
  every accepted native host with literal arguments and a constrained
  environment;
- success proves startup, supported protocol, bounded discovery and schema
  inspection, and complete cleanup without a tool call;
- representative startup, framing, protocol, catalog, schema, bound, early-exit,
  and cleanup failures produce distinct non-zero outcomes and select the
  expected earliest actionable failing layer without hiding independent safety
  failures;
- every ordinary failure states safe what, where, why, expectation, remediation,
  and versioned-rule context without exposing an arbitrary observed value;
- human and public experimental `mcp-doctor.report/v1alpha1` JSON output retain
  the same primary layer and findings, independent findings, causally linked
  skips, summary, limits, outcome, and exit;
- report-only acceptance cases prove that the intended corrective next step is
  recoverable without raw traffic, stderr, source code, or a browser;
- the controlled compatibility matrix covers pinned official SDK examples and
  independent implementations spanning at least two languages, reports every
  unsupported revision honestly, and applies `DEC-024`: all selected
  current-revision cases must pass for broad positioning, a narrower position
  requires at least one credible independent pass plus an explicit
  readiness/migration claim and separate compatibility ticket, and no credible
  independent pass blocks completion;
- the registry identity recheck records the preferred or fallback Cargo package
  under `DEC-008`, and official metadata and installation guidance consistently
  identify the `mcp-doctor` executable as the Rust CLI from EnjoyableWork;
- the `cargo-nextest`, `cargo-llvm-cov`, and `cargo-mutants` evaluations record
  rejection or an evidence-backed, exactly pinned adoption under `DEC-025`; and
- current usage and safety documentation is accurate, and the complete M1
  native and hosted gates pass.

The first M2 publication is complete only when the exact passive-MVP version is
immutable, installs and passes its smoke journey through every advertised
channel, and opens the dated non-sensitive adoption checkpoint. M2 closes only
when `MCPD-008A` also proves the nonpublishing, least-privilege path required
before any later tag. Those gates passed on 2026-08-10, with durable evidence
linked from the ticket board. Publication proves availability and artifact
integrity; it does not by itself prove adoption. M3 authorization is the
separate `DEC-027` decision and does not weaken ticket-level design, safety, or
acceptance gates.

### M4 enterprise assurance

M4 is complete only when:

- `MCPD-013` through `MCPD-018` are done in order and D-09 links their durable
  public and non-disclosing verification evidence;
- every Level 1 control in the exact OSPS version selected at activation passes,
  including applicable conditional controls, with evidence or explicit
  applicability reasoning for every row;
- the public self-assessment states the framework version, level, assessed
  repositories and organization and release boundaries, assessment date,
  limitations, self-assessed status, evidence links, and review triggers;
- the official BadgeApp baseline assessment has achieved the selected Level 1
  status and its official badge links from the README to that assessment and is
  verified on exact `main`;
- the exact M3 GitHub release assets named by the SLSA statement meet the
  selected SLSA Build L2 requirements and their signed provenance is publicly
  reproducibly verified, while Cargo, Homebrew, dependencies, future releases,
  and any unassessed artifacts remain explicitly outside that claim;
- no public text implies higher OSPS levels, independent certification,
  regulatory compliance, NIST certification, OpenChain product certification,
  universal SLSA coverage, a warranty, an SLA, or paid native signing;
- no public evidence exposes private settings, identities, authentication
  factors, vulnerability or secret findings, credentials, or recovery material;
- annual and event-driven evidence revalidation and claim-removal triggers are
  documented and executable; and
- the immutable M3 release and all earlier product, safety, installed-channel,
  and native evidence remain unchanged and valid.

### Milestone complete

A milestone is complete when every retained ticket in its accepted boundary is
done; every removed proposal has a recorded `Deferred`, `Superseded`, or
`Cancelled` disposition and rationale; its golden journey and native matrix
pass; critical risks are mitigated or explicitly accepted; and all preceding
behavior still passes. A workflow definition alone is not hosted CI evidence,
and README prose is never release evidence.
