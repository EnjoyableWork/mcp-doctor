# mcp-doctor project plan

This is the living source for product scope, delivery status, ordered work,
decisions, risks, and release gates.

| Control | Current state |
| --- | --- |
| Document state | Active |
| Product state | Bounded local STDIO discovery, catalog, and JSON Schema 2020-12 diagnosis complete; full earliest-layer/report parity, real-server compatibility, and network transport remain unimplemented |
| Current milestone | M1 — passive local MVP in progress |
| Overall status | M0 plus `MCPD-004` through `MCPD-006` pass locally; the next hosted PR must confirm the expanded passive matrix on every native CI host |
| Current focus | `MCPD-007` — ready for the next goal |
| Public release | None |
| Last reviewed | 2026-08-10 |
| Next review trigger | The next hosted native transport matrix; the `MCPD-007` identity, compatibility, real-server, earliest-layer, and reporter review; the post-M2 adoption checkpoint; a change to the M1 safety boundary; M4 activation; or assurance-framework, issuer-proof, security, release-pipeline, organization-access, or evidence drift |

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
| M2 — Public MVP release | `MCPD-008` | The passive MVP installed and independently verified through every advertised channel |
| M3 — Evidence-led expansion | `MCPD-009` → `MCPD-010` → `MCPD-011` → `MCPD-012` | Only adoption-justified active, remote, adversarial, or CI capabilities, followed by one independently verified expanded release |
| M4 — Enterprise assurance and adoption | `MCPD-013` → `MCPD-014` → `MCPD-015` → `MCPD-016` → `MCPD-017` → `MCPD-018` | Contributor-compatible governance, repository and organization controls, supply-chain evidence, and a public scoped assurance baseline |

Signed native macOS and Windows artifacts are a later candidate, not part of
the first public release. They require an accepted funding and signing decision
plus native installed evidence.

M3 does not activate automatically when M2 ships. Its ordering is the default
only after the M2 adoption checkpoint records whether users retained the
passive MVP, what it found, where it produced false or unclear findings, and
which next capability removes the most common remaining blocker. A dated
decision must first accept or revise the M3 boundary and ticket board. Only
retained tickets form its required order; defer, supersede, or cancel the rest
with a recorded rationale when the evidence points elsewhere.

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
| M1 | One built-binary `inspect` journey identifies the earliest actionable failing layer and report-only correction in redacted human and experimental JSON for synthetic fixtures and a representative real-server compatibility matrix without calling a tool | In progress |
| M2 | One immutable passive-MVP release installs and passes its diagnostic smoke journey through every advertised channel; an adoption baseline is ready to collect | Proposed |
| M3 | Post-M2 evidence justifies every retained expansion; retained journeys preserve inherited safety and stable CI output; one expanded immutable release passes every retained journey | Proposed |
| M4 | The selected current OpenSSF OSPS Baseline Level 1 controls pass with dated public evidence and official self-certification proof; exact release-artifact provenance is separately evaluated against the selected current SLSA Build L2 requirements | Proposed |

Each milestone must leave the preceding milestone working. Expansion does not
justify weakening cleanup, redaction, determinism, or active-execution consent.
Assurance work may verify an immutable release but must never rewrite it or
publish a broader claim than its evidence supports.

## Distribution contract

The product, repository, Cargo package, and installed executable currently use
the name `mcp-doctor`. Registry availability was checked during initialization,
but availability alone is insufficient: an
[existing independently owned project](https://github.com/realwigu/mcp-doctor)
publishes a scoped npm package that also installs an `mcp-doctor` command.
`MCPD-007` must record whether to accept and clearly distinguish that collision
or change identity before release;
`MCPD-008` then rechecks every selected registry immediately before
publication.

The first zero-cost release target is:

| Channel | Initial contract |
| --- | --- |
| GitHub Releases | Immutable release with native GNU/Linux ARM64/x64 archives, `SHA256SUMS`, target SPDX SBOMs, provenance, source package, and Homebrew formula |
| crates.io | The exact release source package, installed and smoke-tested on supported macOS, GNU/Linux, and Windows hosts |
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
| Active activity | Outside M1; `check` and `break` remain proposed M3 expansion and always require explicit authorization |
| Schemas | JSON Schema 2020-12 under bounded local evaluation; no external retrieval by default |
| Findings | Earliest actionable layer and primary finding or findings, independent safety findings, typed code, severity, safe location/context, causally linked performed/skipped state, overall outcome, safe expectation, remediation, and specification reference |
| Output | Redacted human report plus public experimental `mcp-doctor.report/v1alpha1` JSON with the same primary diagnosis, independent findings, causal skips, and result; stabilization and additional CI formats remain M3 |
| Process policy | Literal executable/arguments, constrained environment, bounded I/O and time, full cleanup and reap |
| Test data | Synthetic fixture servers in the default suite plus a controlled, non-mutating compatibility matrix of official SDK examples and independent implementations |
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
`MCPD-004`. `MCPD-005` wires the transport findings and `MCPD-006` wires the
catalog and schema findings below. Earliest-layer designation and complete
human/agent report parity remain `MCPD-007` work.

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
| `MCP-SCHEMA-001` | Error | An advertised JSON Schema contract is invalid |
| `MCP-SCHEMA-002` | Error | An advertised schema declares a dialect outside the M1 Draft 2020-12 contract |
| `MCP-SCHEMA-003` | Error | An advertised schema would require prohibited external reference retrieval |

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
message, safe expectation, corrective next step, versioned reference, and
typed evidence. Arbitrary values, identifiers, paths, payloads, headers,
arguments, results, and logs cannot enter the ordinary result model;
observations retain only a safe JSON type or `[REDACTED]` and a byte count.

The internal JSON envelope is `mcp-doctor.report/v1alpha1` and includes the
revision, exact limits, derived summary, performed/skipped checks, findings,
outcome, and exit code. It is fixture-tested but not exposed by the CLI and is
not yet a public compatibility promise; `MCPD-007` exposes it as experimental
and `MCPD-012` owns the stable machine format.

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
| Reserved active and network work | 100 cases; zero redirects; zero retries; concurrency 1 |

Revision selection stops and reports a limit finding at the 33rd advertised
value, even if the source could continue indefinitely. Report construction
rejects more than 256 findings. Limit construction rejects zero safety bounds,
a stage longer than the total,
a message larger than stdout, a stream larger than the combined-output cap, a
schema or instance larger than a message, `$ref` depth above schema depth, or
concurrency above the case budget. Redirect and retry counts alone may be zero.

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
deterministic. Native hosted confirmation remains part of the next pull request,
and `MCPD-007` still owns real-server reach, primary-layer selection, causal
skips, and public experimental JSON.

Local acceptance on 2026-08-10 is 38 unit tests, six CLI tests, and 18
built-binary STDIO tests (62 total) through the disposable locked gate, plus a
clean `cargo-deny` advisory, license, ban, and source review. The source package
contains 45 files, verifies from its staged contents, and compresses to 94.7
KiB. The locked all-target graph checks for GNU/Linux x64 and Windows x64; the
local macOS ARM64 release binary is 7,602,960 bytes. These size measurements are
review evidence, not a stable artifact promise. Native Linux and Windows
execution and the complete hosted matrix are not claimed by this local run.

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

### M2 adoption checkpoint

After the passive MVP is independently installed and verified, M3 remains
Proposed until a dated review records:

- attempts from at least five independent server authors or independently
  maintained implementations, or the inability to recruit them as a negative
  signal rather than invented adoption;
- time and steps from installation to the first useful report;
- actionable defects found, false or unclear findings, and unsupported-version
  outcomes without retaining private server data;
- whether the author could choose the corrective next step from the ordinary
  report alone or still needed raw traffic, stderr, source, or another tool;
- whether any adopter kept the command in a repeat workflow or CI; and
- which unserved job, if any, justifies active scenarios, HTTP, adversarial
  generation, or another direction next.

This checkpoint is product evidence, not telemetry permission. Use consented,
non-sensitive reports, public issues, reproducible synthetic cases, or aggregate
counts. Do not collect endpoints, credentials, payloads, identities, or private
diagnostic output.

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
| D-03 | Local and hosted quality baseline | M0 | Done | POSIX and PowerShell gates, dependency policy, least-privilege three-OS workflow, Dependabot, and community/security surfaces pass locally; the [hosted run on merged `main` `f788e76`](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31337295110) passes dependency policy plus GNU/Linux x64, macOS ARM64, and Windows x64 gates |
| D-04 | Versioned diagnostic result contract | M1 | Done | [Typed contract modules](src/contract), [synthetic contract fixtures](tests/fixtures/contracts), and focused revision, limit, finding, redaction, skip, outcome, exit, and reporter tests |
| D-05 | Bounded STDIO process and message boundary | M1 | Done | [Managed STDIO transport](src/transport/stdio.rs), [synthetic fixture server](tests/fixtures/stdio_server.rs), and [nine built-binary journeys](tests/stdio.rs) prove literal arguments, constrained environment, passive discovery, simultaneous bounds, redaction, graceful and forced process-tree cleanup, and distinct transport failures; the full 48-test suite passes locally |
| D-06 | Adoption-ready passive `inspect` journey | M1 | In progress | `MCPD-006` catalog/schema diagnosis and bounded fixture matrix pass locally; earliest-layer and report-only-correction fixtures, public experimental JSON, real-server compatibility evidence, identity review, and complete native journeys remain `MCPD-007` |
| D-07 | Immutable passive MVP release | M2 | Proposed | Release, registry, tap, provenance, installed native smoke evidence, and a dated adoption checkpoint |
| D-08 | Evidence-led diagnostic expansion release | M3 | Proposed | Post-M2 product evidence, retained expansion journeys, stable CI reports, and independently verified release artifacts |
| D-09 | Evidence-backed enterprise assurance baseline | M4 | Proposed | Verified repository, organization, community, licensing, and supply-chain controls; complete OSPS Level 1 crosswalk; official self-certification proof; and exact-artifact SLSA evaluation |

## Ticket board

| ID | Outcome | Milestone | Status | Depends on | Acceptance evidence |
| --- | --- | --- | --- | --- | --- |
| MCPD-001 | Establish the product promise, operating model, safety priorities, delivery sequence, decisions, and risks | M0 | Done | — | Root product and project contracts are internally consistent and link correctly |
| MCPD-002 | Bootstrap one Rust 2024 binary with truthful help/version output and isolated built-binary tests | M0 | Done | `MCPD-001` | Locked build, format, Clippy, five tests, help, version, metadata, self-contained package, and installed package smoke pass |
| MCPD-003 | Add disposable local gates, dependency policy, least-privilege cross-platform CI, maintenance automation, and community/security entry points | M0 | Done | `MCPD-002` | POSIX and PowerShell gates, `cargo-deny`, Actionlint, ShellCheck, YAML parsing, links, packaging, and identity checks pass locally; the [first hosted matrix on merged `main` `f788e76`](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31337295110) passes |
| MCPD-004 | Define supported MCP revision behavior, typed findings, limits, exit semantics, and redacted report contract | M1 | Done | `MCPD-003` | [Contract implementation](src/contract), [synthetic snapshots and cases](tests/fixtures/contracts), and 24 focused contract tests prove the accepted compatibility, severity, limit, redaction, performed/skipped, deterministic report, outcome, and exit decisions |
| MCPD-005 | Implement the bounded STDIO process and message boundary with guaranteed cleanup | M1 | Done | `MCPD-004` | [Nine built-binary cases](tests/stdio.rs) cover empty-capability success without a follow-up request, literal arguments, constrained environment, malformed and redacted output, every I/O limit, timeout, early exit, missing process, and resistant-descendant cleanup; focused framing, protocol, budget, report, and cross-target compile checks also pass |
| MCPD-006 | Diagnose discovered tools, prompts, resources, and JSON Schema contracts without implicit tool execution | M1 | Done | `MCPD-005` | [Versioned catalog/schema adapter](src/contract/catalog.rs), [static catalog fixtures](tests/fixtures/catalogs), and [built-binary STDIO journeys](tests/stdio.rs) prove valid, invalid, complex, duplicate, paginated, redacted, no-retrieval, and exact bounded cases with safe expectations and remediation; the complete local locked gate and cross-target checks pass |
| MCPD-007 | Make passive `inspect` identify the earliest actionable failing layer, remain report-sufficient for humans and agents, and prove its real-server reach and release identity | M1 | Ready | `MCPD-006` | Built-binary human and experimental JSON journeys agree on the primary layer and findings, independent failures, causal skips, and corrective next step; report-only fixtures prove actionability; pinned official examples and independent servers across at least two languages prove claimed compatibility; identity and revision-release reviews are recorded |
| MCPD-008 | Publish and independently verify the first immutable passive-MVP release through GitHub, Cargo, and Homebrew | M2 | Proposed | `MCPD-007` | Every artifact and public channel installs the same version and passes the passive diagnostic smoke journey; the adoption checkpoint is opened with a dated baseline |
| MCPD-009 | Add explicit, budgeted, seed-reproducible `check` scenarios and result-schema validation when M2 evidence justifies active testing | M3 | Proposed | `MCPD-008` and the M2 adoption checkpoint | Selected-tool consent, deterministic generation, crash, silent failure, and output mismatch journeys |
| MCPD-010 | Add a bounded Streamable HTTP transport with explicit remote-target and credential policy when M2 evidence justifies remote diagnosis | M3 | Proposed | `MCPD-009` | Local HTTP fixtures prove headers, redirects, auth redaction, TLS/error, timeout, and response limits |
| MCPD-011 | Add the bounded adversarial `break` command for authorized tools when M2 evidence justifies generated pressure | M3 | Proposed | `MCPD-010` | Schema-derived cases are deterministic, limited, reproducible, and cannot widen target scope |
| MCPD-012 | Stabilize machine reports and CI integration, then publish and independently verify the retained M3 journeys | M3 | Proposed | `MCPD-011` | Stable versioned JSON plus one accepted CI format preserve findings and exits; every expanded-release artifact and channel passes its applicable installed smoke journey |
| MCPD-013 | Protect the default branch and define a contributor-compatible merge policy | M4 | Proposed | `MCPD-012` | A live public ruleset, credential-free verifier, normal protected pull request, rejected direct-update/deletion exercises, and documented emergency path prove the selected approval, check, bypass, signing, deletion, and non-fast-forward policy |
| MCPD-014 | Establish vulnerability disclosure and live repository-security controls | M4 | Proposed | `MCPD-013` | The recognized security policy, private route, supported-version and response contract, enabled entitled security features, non-disclosing verifier, and recorded clean baseline prove the scoped controls and limitations without exposing findings |
| MCPD-015 | Verify the public contribution, community, repository, and licensing contract | M4 | Proposed | `MCPD-014` | Public workflows and recognized community files, complete in-scope repository inventory, HTTPS-only official channels, and exact source, package, archive, and formula license evidence pass a credential-free verifier |
| MCPD-016 | Harden dependency maintenance and the CI, artifact, and distribution supply chains | M4 | Proposed | `MCPD-015` | Grouped updates, full-SHA action inventory, fork and permission policy, tracked-artifact rejection, authenticated distribution verification, negative exercises, and operator audit pass against exact `main` and the immutable release |
| MCPD-017 | Establish organization access, credential, ownership, and recovery policy | M4 | Proposed | `MCPD-016` | Strong MFA, lowest-default access, deliberate grants and repository creation, scoped applications and automation, explicit owner-continuity decision, private recovery exercise, and a non-disclosing live verifier pass |
| MCPD-018 | Self-assess, publish, and maintain the enterprise assurance baseline | M4 | Proposed | `MCPD-017` | Every selected OSPS Level 1 control has public evidence or exact applicability reasoning; the official assessment and badge are verified on exact `main`; exact M3 artifacts receive a correctly scoped SLSA Build L2 evaluation; and claim-review and removal triggers are documented |

## Canonical goal objectives

Use the matching objective when beginning an eligible main-story ticket:

| Ticket | Objective |
| --- | --- |
| MCPD-002 | Complete `MCPD-002`: bootstrap the original Rust 2024 `mcp-doctor` binary with truthful help/version behavior, a committed lockfile, isolated built-binary tests, and verified package metadata. Do not begin protocol behavior. Finish when every ticket acceptance check passes and durable evidence is recorded. |
| MCPD-003 | Complete `MCPD-003`: establish disposable POSIX and Windows quality gates, dependency policy, least-privilege cross-platform CI, scheduled maintenance, and public contribution, support, conduct, and private-security entry points. Do not begin protocol behavior or release automation. Finish when local evidence is recorded and the first hosted matrix passes. |
| MCPD-004 | Complete `MCPD-004`: define the supported MCP revision matrix, diagnostic findings, limits, redaction, performed/skipped semantics, and exit/report contracts using synthetic fixtures. Do not start a real process or network transport. Finish when decisions and focused tests make the contract unambiguous. |
| MCPD-005 | Complete `MCPD-005`: implement a literal-argument, constrained-environment STDIO boundary with bounded messages, output and time plus guaranteed process-tree cleanup and reap. Do not call tools or begin HTTP support. Finish when the full transport failure matrix passes. |
| MCPD-006 | Complete `MCPD-006`: diagnose advertised tools, prompts, resources, and bounded JSON Schema 2020-12 contracts without implicit tool execution or external schema retrieval. Give each failure a safe expectation and corrective next step. Finish when valid and invalid fixture catalogs produce deterministic, redacted, actionable findings. |
| MCPD-007 | Complete `MCPD-007`: wire the passive local journey through the built binary; identify the earliest actionable failing layer while preserving independent safety failures and causally linked skips; expose equivalent redacted human and experimental `mcp-doctor.report/v1alpha1` JSON output; prove report-only actionability; test pinned official SDK examples and independent implementations in at least two languages; and close the public identity and supported-revision release reviews. Do not call tools, add HTTP, or claim official conformance. Finish when a human or agent can use the report alone to determine what failed, where, why, what was expected, and what to change next. |
| MCPD-008 | Complete `MCPD-008`: publish and independently verify the first immutable passive-MVP version through GitHub Releases, crates.io, and source-built Homebrew, with deterministic packages, checksums, SPDX SBOMs, attestations, and installed passive diagnostic smokes for every represented channel. Open the dated, non-sensitive M2 adoption checkpoint; do not add active or remote behavior. |
| MCPD-009 | Complete `MCPD-009` only after the M2 adoption review justifies active testing: add explicit selected-tool scenarios with fixed budgets, deterministic seeds, reproducible structural cases, and output-schema validation. Never broaden the authorized target. Finish when active success and failure journeys pass without secret output or orphaned processes. |
| MCPD-010 | Complete `MCPD-010` only after the M2 adoption review justifies remote diagnosis: add bounded Streamable HTTP diagnosis under an accepted redirect, SSRF, proxy, authentication, TLS, header, and redaction policy. Do not begin adversarial generation. Finish when deterministic local remote-server fixtures prove the full network boundary. |
| MCPD-011 | Complete `MCPD-011` only after the M2 adoption review justifies generated pressure: generate bounded deterministic boundary cases only for explicitly authorized tools, record reproducible seeds and structural inputs, and enforce schema and scenario limits. Finish when generation cannot widen target or execution scope. |
| MCPD-012 | Complete `MCPD-012`: stabilize the redacted machine-result contract and accepted CI reporter across every retained local and remote journey, then publish one protected immutable expanded version with authenticated artifacts and installed smokes for every represented channel. Do not retain an M3 feature that the adoption review deferred or cancelled. |
| MCPD-013 | Complete `MCPD-013`: protect the default branch with a contributor-compatible public ruleset, deliberate approval, check, bypass, merge, deletion, non-fast-forward, and commit-signing choices; implement credential-free drift verification; and prove normal, rejected, and bounded emergency paths. Do not change immutable release bytes or begin later assurance tickets. |
| MCPD-014 | Complete `MCPD-014`: establish recognized supported-version, security-contact, private-reporting, response, and coordinated-disclosure guidance; enable and read back the entitled dependency, code-scanning, secret-prevention, and private-reporting controls; document unavailable features exactly; and verify a non-disclosing clean baseline. Do not publish a complete-baseline claim. |
| MCPD-015 | Complete `MCPD-015`: verify public contribution, conduct, support, defect-reporting, repository-inventory, official-channel, inbound-license, source-license, and released-asset license contracts across every in-scope repository and distribution channel. Avoid nominal reviewers, owners, or controls, and do not begin supply-chain changes. |
| MCPD-016 | Complete `MCPD-016`: automate grouped dependency updates; inventory and verify every selected Action at a reviewed full commit SHA; prove untrusted workflows are read-only and secretless; reject generated executables and unreviewable binary artifacts; and authenticate the immutable release, Cargo package, and Homebrew formula without changing published bytes. |
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
| DEC-008 | Use direct package and executable identity `mcp-doctor` | Working assumption | 2026-08-09 | `MCPD-007` must resolve the known public command-name collision and search ambiguity; registry availability alone cannot authorize publication |
| DEC-009 | First release uses Linux archives, Cargo source, and source-built Homebrew | Accepted | 2026-08-09 | Signed native macOS/Windows and WinGet remain later funded scope |
| DEC-010 | Track the ordered delivery plan in this repository | Accepted | 2026-08-09 | Hosted issues may supplement but do not replace milestone and decision truth |
| DEC-011 | Use an `inspect`, `check`, and `break` command family | Accepted | 2026-08-09 | `inspect` ships as the passive MVP; `check` and `break` remain M3 candidates whose explicit authorization and finite budgets cannot be weakened by adoption pressure |
| DEC-012 | Add a post-expansion M4 for evidence-backed enterprise assurance and adoption | Accepted | 2026-08-09 | M2 publishes the first passive release; M4 follows the independently verified M3 expansion release and orders governance, disclosure, security, community, licensing, supply-chain, organization, OSPS, and exact-artifact SLSA evidence without implying certification or regulatory compliance |
| DEC-013 | Support only modern MCP `2026-07-28` and recognize but never initialize the four earlier official revisions | Accepted | 2026-08-09 | M1 has one exact protocol contract; legacy-only and unknown advertisements fail with safe structured evidence rather than implicit fallback |
| DEC-014 | Make stable finding codes own severity and derive check state, overall outcome, and exit status from one result model | Accepted | 2026-08-09 | Callers cannot downgrade a code, turn a skip into a pass, or supply an inconsistent summary or exit code |
| DEC-015 | Permit only trusted structural context and typed redacted evidence in ordinary reports | Accepted | 2026-08-09 | Human and JSON output cannot retain arbitrary values; `MCPD-007` exposes `v1alpha1` as explicitly experimental and `MCPD-012` owns stabilization |
| DEC-016 | Adopt the finite M1 default limit profile recorded in the MCPD-004 contract | Accepted | 2026-08-09 | Every later boundary must enforce and report these simultaneous caps or explicitly revise the contract with evidence |
| DEC-017 | Make one-command passive local preflight the first adoption wedge | Accepted | 2026-08-09 | M1 must answer startup, supported protocol, advertised-contract, bound, and cleanup health without tool execution; interactive inspection, official conformance, security scanning, and LLM evaluation stay outside the MVP |
| DEC-018 | Publish the passive MVP before active or remote expansion | Accepted | 2026-08-09 | M2 distributes and independently verifies `inspect`; active scenarios, HTTP, and adversarial generation cannot become first-release prerequisites |
| DEC-019 | Gate M3 expansion on dated adoption evidence | Accepted | 2026-08-09 | M3 stays Proposed until the M2 checkpoint records real attempts, time to value, useful and false findings, repeat use, compatibility, and the next demonstrated user blocker |
| DEC-020 | Make earliest-actionable-layer diagnosis and report-only correction the project-wide north star | Accepted | 2026-08-09 | Every journey prioritizes causal diagnosis over check count, preserves independent safety failures, marks dependent skips, and gives humans and agents the same sufficient evidence; breadth without this behavior cannot satisfy a milestone |
| DEC-021 | Run each local target as a directly launched, minimally provisioned managed process tree | Accepted | 2026-08-09 | Arguments are literal; only platform launch variables are inherited; batch targets that require a shell are rejected; bounded pipes, Unix process groups, Windows Job Objects, forced termination, and wait completion form one transport boundary |
| DEC-022 | Validate M1 tool schemas as local JSON Schema Draft 2020-12 with a maintained validator and no retrieval features | Accepted | 2026-08-10 | `jsonschema` is locked with default features disabled and a rejecting retriever; exact MIT-0 and Zlib transitive licenses are reviewed with crate-scoped exceptions; unsupported dialects and external references receive typed findings instead of fallback or I/O |

## Open decisions

| ID | Decision needed | Needed by | Default if unresolved |
| --- | --- | --- | --- |
| OPEN-04 | Scenario file format and secret-reference boundary | `MCPD-009` | Versioned local file with literals prohibited from ordinary output; no secret store |
| OPEN-05 | Safety annotations or confirmations required before active tool calls | `MCPD-009` | Explicit tool allowlist and per-run active acknowledgement |
| OPEN-06 | Streamable HTTP redirect, proxy, private-address, and authentication contract | `MCPD-010` | No redirects, no inherited proxy, explicit headers, and remote target shown before activity |
| OPEN-07 | Stable machine-output version and first additional CI reporter | `MCPD-012` | Retain experimental `v1alpha1` until evidence supports a stable version; evaluate JUnit versus SARIF from the intended consumer |
| OPEN-08 | Exact OSPS, BadgeApp, and SLSA versions and proof mechanisms at M4 activation | `MCPD-013` | Use the then-current official versions; planning baseline is OSPS `v2026.02.19` Level 1 and SLSA `v1.2` Build L2, with a documented update if either is superseded |
| OPEN-09 | Default-branch approval count, required checks, merge methods, bypass, emergency administration, and commit-signing policy | `MCPD-013` | Prevent direct updates and deletion with strict current checks and no standing bypass; do not require an unavailable independent reviewer or unproven signature path |
| OPEN-10 | Organization membership, application, automation-credential, owner-continuity, and private recovery boundary | `MCPD-017` | Lowest default access, deliberate grants, strong MFA, scoped automation, explicit residual-risk acceptance, and non-disclosing recovery evidence |
| OPEN-11 | Public package, executable, repository, and search identity after the known `mcp-doctor` command collision | `MCPD-007` | `MCPD-008` remains blocked; do not publish an identity whose ownership and distinction are unresolved |
| OPEN-12 | Whether real-server evidence supports a broad `2026-07-28` MVP audience or requires explicitly narrow migration positioning or a later compatibility ticket | `MCPD-007` | Keep DEC-013 unchanged, state latest-only support prominently, and do not infer or add a legacy handshake |

## Risk register

| ID | Risk | Impact | Mitigation and escalation trigger | State |
| --- | --- | --- | --- | --- |
| RISK-01 | A diagnostic invokes a mutating tool unexpectedly | Critical | Passive default, explicit selected-tool scenarios, and consent tests; any implicit call blocks the passive MVP and every later release | Mitigated locally through `MCPD-006`; open until the complete native M1 journey passes |
| RISK-02 | A timed-out server or descendant remains running | Critical | Managed process tree, shutdown bounds, termination, reap, and resistant-child fixtures; any surviving PID blocks release | Mitigated locally through `MCPD-005`; open until the hosted native M1 matrix passes |
| RISK-03 | Secrets or raw production values reach output | High | Structural redaction and sentinel tests across errors, reports, debug surfaces, and fixtures; any observed value blocks release | Open — all milestones |
| RISK-04 | Protocol evolution makes diagnostics incorrect | High | Revision-specific rules and fixtures with explicit unsupported outcomes; a new release triggers contract review | Open |
| RISK-05 | Pathological schema or output exhausts resources | High | Depth, bytes, errors, cases, time, and reference limits; an unbounded input path blocks release | Output and passive schema paths are mitigated locally through `MCPD-006`; native hosted confirmation remains an M1 gate |
| RISK-06 | Remote diagnosis enables SSRF or credential leakage | Critical | Explicit M3 network policy and local fixtures before HTTP implementation; unclear proxy/address behavior blocks `MCPD-010` | Deferred with M3 |
| RISK-07 | Generated cases are irreproducible or exceed authorized scope | High | Stable seed, ordered generation, structural evidence, and target allowlist; mismatch blocks active testing | Deferred with M3 |
| RISK-08 | A passing report creates false confidence after skipped checks | High | Per-check performed/skipped state and non-ambiguous summary; any hidden skip blocks release | Open — M1 gate |
| RISK-09 | Broad protocol, transport, and reporting scope delays a usable slice | High | M1 ends at passive `inspect`, M2 publishes it, and the adoption checkpoint gates all active and remote M3 scope; any M3 feature becoming an MVP prerequisite escalates | Mitigated by plan |
| RISK-10 | The public identity is unavailable, ambiguous, or confused with an existing command before publication | High | `MCPD-007` records the command/search collision decision and `MCPD-008` rechecks every registry; unresolved ambiguity blocks publication | Open — M2 gate |
| RISK-11 | A release channel installs bytes not represented by the immutable release | Critical | Exact package/formula equality, checksums, attestations, and native installed smokes; any mismatch requires a new version | Deferred with M2 |
| RISK-12 | An unprotected default branch permits direct, destructive, or insufficiently reviewed changes | High | `MCPD-013` requires an enforced public ruleset, drift verifier, rejected-path exercises, and a bounded emergency process; any unverified bypass or destructive path blocks M4 | Deferred with M4 |
| RISK-13 | A contributor publicly exposes a vulnerability, credential, or unsafe diagnostic because reporting and prevention controls are incomplete | High | `MCPD-014` verifies private reporting, safe guidance, entitled scanning and prevention controls, limitations, and a non-disclosing baseline; any public sensitive report or hidden finding blocks M4 | Deferred with M4 |
| RISK-14 | Mutable automation, privileged untrusted code, or unauthenticated distribution compromises the project or its releases | Critical | `MCPD-016` inventories full-SHA Actions, proves fork and permission isolation, rejects unsafe tracked artifacts, and authenticates every in-scope channel; any drift or credential exposure blocks M4 | Deferred with M4 |
| RISK-15 | Organization-owner loss or over-broad long-lived credentials become an undocumented recovery dependency | High | `MCPD-017` verifies strong MFA, lowest access, application and credential scope, owner continuity, and private recovery evidence; unresolved access or recovery assumptions block M4 | Deferred with M4 |
| RISK-16 | A stale, unofficial, or over-broad assurance claim misleads adopters | High | `MCPD-018` binds every claim to exact version, scope, date, official proof, public evidence, and removal triggers; missing, stale, withdrawn, or ambiguous proof blocks or removes the claim | Deferred with M4 |
| RISK-17 | Technically correct findings become an undifferentiated failure list that does not help a developer repair a server or earn repeat use | High | Every MVP failure identifies the expected earliest actionable layer, preserves independent safety failures, links downstream skips to their cause, and includes safe what, where, why, expectation, remediation, and versioned-rule evidence; report-only cases, real-server trials, and the M2 checkpoint record unclear findings, false findings, time to value, and repeat use before expansion | Open — M1/M2 gate |
| RISK-18 | Latest-only protocol support excludes too much of the reachable ecosystem for a useful first release | High | `MCPD-007` tests pinned official examples and independent implementations, records unsupported outcomes, and chooses honest broad or migration-specific positioning without silently adding legacy behavior | Open — M1 gate |

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
- the controlled compatibility matrix passes its pinned official SDK examples
  and independent implementations across at least two languages, with every
  unsupported revision reported honestly; and
- the public identity and release-positioning decisions are closed, current
  usage and safety documentation is accurate, and the complete M1 native and
  hosted gates pass.

M2 is complete only when the exact passive-MVP version is immutable, installs
and passes its smoke journey through every advertised channel, and opens the
dated non-sensitive adoption checkpoint. Publication proves availability and
artifact integrity; it does not by itself prove adoption or authorize M3.

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
