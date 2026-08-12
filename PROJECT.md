# mcp-doctor project plan

This is the living source for product scope, delivery status, ordered work,
decisions, risks, and release gates.

| Control | Current state |
| --- | --- |
| Document state | Active |
| Product state | The passive local STDIO MVP, pinned current-revision compatibility matrix, bounded local and Streamable HTTP `check`, deterministic `break`, stable schema-backed JSON, and JUnit-compatible projection pass local, hosted, immutable-release, and represented installed-channel evidence |
| Current milestone | M4 — enterprise assurance and adoption; `MCPD-015` is Done and `MCPD-016` is In progress |
| Overall status | M0 through M3 are Done; immutable `v0.1.0` and `v0.2.0` channels remain verified; protected `main` retains the `MCPD-013` controls; the scoped `MCPD-014` repository-security baseline and dated `MCPD-015` public community, repository-inventory, channel, and license contract pass with their explicit limitations; and `MCPD-016` is activating a closed dependency, Action, untrusted-workflow, tracked-artifact, and exact-distribution supply-chain contract without changing published bytes |
| Current focus | Land and activate the `MCPD-016` canonical controls, verify a grouped review-only dependency proposal, enable selected full-SHA Actions policy, run the fork and artifact negative exercises, and authenticate exact `main` plus immutable `v0.2.0`, Cargo, and Homebrew bytes |
| Public release | `mcp-doctor` `v0.2.0` — immutable GitHub Release, crates.io, and `EnjoyableWork/tap/mcp-doctor` verified |
| Last reviewed | 2026-08-12 |
| Next review trigger | `MCPD-016` activation or exact-main evidence review; any public-repository, community-route, official-channel, source/package/archive/formula license, security-policy, supported-line, entitlement, scan result, ruleset, merge-setting, administrator-boundary, required-context, GitHub-capability, Action, workflow, dependency proposal, tracked artifact, voluntary-usage, trusted-publisher, tap-authority, release-pipeline, testing-tool, safety-boundary, or assurance-evidence change |

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

M3 was activated by the dated `DEC-027` owner decision after M2 and
`MCPD-008A` completed. Independent adoption evidence remained useful
prioritization input rather than a prerequisite whose external timing could
indefinitely block planned feature work. Each M3 ticket satisfied its
predecessor, resolved its design and safety decisions, preserved the north
star, and passed its acceptance evidence before the expanded `v0.2.0` release
closed M3 on 2026-08-11. Voluntary feedback may still reprioritize, narrow,
defer, or cancel later work.

M4 became eligible only after the expanded M3 release was independently
verified. It does not delay or reopen M3, and it does not turn a
self-assessment into a warranty, independent certification,
regulatory-compliance claim, or support SLA.

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

## Product category and comparative evaluation

`DEC-033` defines a durable way to assess the intended product, an exact
implementation, or an immutable release without turning this plan into a
scoreboard. `mcp-doctor` is a **safety-bounded MCP server-author diagnostic
preflight**: its primary job is to find the earliest actionable failing layer,
explain a safe correction, and produce reproducible evidence for people and
automation. Interactive exploration, official protocol conformance, LLM-based
quality evaluation, and broad vulnerability scanning are adjacent categories,
not identities that silently broaden this one.

This section retains the category, dimensions, weights, scoring method, and a
seed comparison set. It deliberately does not retain a current score, ranking,
winner, or market-dominance claim. A requested evaluation is a new, dated
assessment against then-current primary evidence. Its result stays outside
`PROJECT.md` unless a separately accepted ticket calls for a durable snapshot.

### Category and excellence posture

The postures below are product intentions, not claims about shipped behavior.
Acceptance evidence in the ticket board and exact artifacts remains the only
authority for what exists.

| Goal category | Product areas | Intended posture |
| --- | --- | --- |
| Excel | Causal diagnosis and remediation; safety and containment; deterministic runtime evidence | Be the tool a server author trusts first for a noninteractive preflight: identify the root layer, preserve independent safety failures, recommend a concrete correction, reproduce the result, and leave no surprising execution or residue |
| Be strong | Protocol and contract correctness; CI and machine interoperability; adoption UX and integration reach; release and project assurance | Meet the full accepted revision, transport, reporter, platform, installation, and public-proof contracts with evidence that is easy to consume and independently repeat |
| Complement deliberately | General security-vulnerability detection; interactive debugging; official conformance; LLM behavior evaluation | Diagnose structural and runtime safety failures inside the accepted threat model, exchange evidence with specialists when a real consumer exists, and avoid relabeling ordinary findings or aspirational integrations as scanner, conformance, or evaluation coverage |

### Weighted evaluation dimensions

Every full assessment uses all dimensions and the fixed weights below. A
specialist view may highlight a subset, but it must still show the unmodified
full score so omitted weaknesses cannot be normalized away.

| Dimension | Weight | Evaluation question |
| --- | ---: | --- |
| Causal diagnosis and remediation | 18 | Does the result identify the earliest actionable layer, preserve independent failures, explain causal skips, and give a safe correction using report-sufficient evidence? |
| Protocol and contract correctness | 15 | Are revision, transport, lifecycle, schema, capability, and rejection rules explicit and proven against the claimed MCP contract? |
| Runtime testing and reproducibility | 15 | Can passive and expressly authorized active behavior be exercised deterministically with bounded, replayable cases and reliable cleanup across claimed platforms? |
| Safety and containment | 17 | Are execution, network, credentials, untrusted values, resources, redaction, process trees, and performed-versus-skipped claims controlled by tested fail-closed boundaries? |
| CI and machine interoperability | 10 | Do stable vendor-neutral results, exit semantics, compatibility fixtures, and loss-aware CI projections work without rerunning or exposing the target? |
| Adoption UX and integration reach | 10 | Is the preflight installable, noninteractive, actionable from its ordinary output, available on claimed platforms, and usable by common developer and automation workflows? |
| Security-vulnerability detection | 10 | Within an explicit threat model, does the product detect meaningful security conditions with safe validation, measured accuracy, and useful interoperability rather than security-flavored labels? |
| Release and project assurance | 5 | Do reproducible builds, authenticated artifacts, dependency policy, repository controls, maintenance practice, and narrowly evidenced public claims support trust in the delivered tool? |
| **Total** | **100** | Full intended-category assessment |

The capability score for each row is `weight * rating / 5`; the total is the
sum of all eight rows. Ratings may use half points and follow one scale:

| Rating | Meaning |
| ---: | --- |
| 0 | Absent, contradicted, or no positive evidence in the selected assessment scope |
| 1 | Minimal, claim-only, or unsuitable for dependable use |
| 2 | Basic and useful only in a narrow path with important gaps |
| 3 | Credible and usable for the declared scope |
| 4 | Strong, well-evidenced, and differentiated |
| 5 | Category-leading for the declared scope with direct comparative evidence |

Unknown and unverifiable capabilities receive no positive credit in an
implemented or released assessment; the evaluator records them as unknown
rather than claiming they are absent. Popularity, feature count, and prose
claims are not substitutes for exercised behavior or report quality.

### Scope, proof maturity, and safety gate

Each assessment chooses and headlines exactly one capability scope:

- **Intended:** the accepted README destination and `PROJECT.md` plan. This
  scores design completeness and positioning and must say that it is not a
  shipped-capability assessment.
- **Implemented:** one exact commit plus code, fixtures, local tests, and any
  hosted checks tied to it. Planned behavior receives no capability credit.
- **Released:** one immutable version and only behavior reproduced from its
  published artifacts. Later source changes receive no credit.

A comparative assessment applies the same scope and evidence cutoff to every
candidate. If comparable evidence cannot be obtained, show the cells as
unknown and do not issue an aggregate ranking. Record the strongest applicable
proof maturity separately from the capability rating:

| Grade | Evidence maturity |
| --- | --- |
| `P` | Accepted plan or documented product intention only |
| `L` | Exact source implementation with local, reproducible test evidence |
| `H` | Exact source verified by project-hosted automation on a named host |
| `R` | Exact immutable release artifact reproduced on a claimed platform |
| `I` | Exact behavior independently reproduced, or sustained use independently evidenced |

Proof maturity does not add points. It prevents an intended design score from
being presented as implementation, release, adoption, or dominance evidence.

Apply the safety gate before assigning an assessment band. Evidence of
surprising or insufficiently authorized execution, secret or raw-untrusted
value disclosure, an unbounded critical path, orphaned managed processes,
false success or hidden causal skips, or unsupported security/trust claims
classifies the candidate as unsafe for recommended use. Preserve the raw
arithmetic for diagnosis, but cap the reported total at 49 until the gate is
cleared.

| Score | Capability assessment band |
| ---: | --- |
| 90–100 | Category-leading candidate; requires a clear safety gate and direct comparative evidence |
| 80–89 | Strong and differentiated |
| 70–79 | Credible with major gaps |
| 60–69 | Partial category fit |
| 0–59 | Insufficient for the category promise |

These are capability bands, not market-adoption or market-dominance bands.
Assess adoption separately with dated evidence such as verified package and
artifact use, independent repeat usage, durable third-party integrations,
external reproduction, and maintainer responsiveness. Stars, forks, downloads,
company size, or a capability total alone must not be translated into a
dominance claim.

### Dynamic assessment procedure

When an evaluation is requested:

1. Record the date, question, capability scope, evidence cutoff, exact commit
   or release, target user, and comparison cohort.
2. Revalidate every candidate's identity, ownership, activity, license,
   releases, documented scope, and primary evidence; search for material new
   entrants before retaining or changing the seed set.
3. Use the same safe scenarios and declared MCP revision, transport, platform,
   and output expectations where head-to-head execution is authorized and
   possible. Record incomparable surfaces instead of inventing parity.
4. For every dimension, record the rating, weighted points, proof grade,
   concise rationale, primary sources, unknowns, and evidence age.
5. Apply the safety gate, calculate the total and capability band, and state
   the highest-impact gaps or differentiators. A 5 requires direct comparative
   evidence, not confidence in a roadmap.
6. Report adoption and market position as a separate qualitative assessment;
   distinguish direct competitors, reference tools, and complementary
   specialists.
7. Return the date-stamped assessment without writing its scores or ranking
   into this file. Re-run rather than reuse it after a material release,
   protocol change, competitor change, security incident, or evidence expiry.

### Seed comparison set

This is a discovery roster, not an endorsement, exhaustive market map, feature
claim, or ranking. Similar names do not establish shared identity or scope.
Every evaluation must revalidate these projects from primary sources and add,
remove, or recategorize material entrants before scoring.

| Cohort | Seed projects | Why compare |
| --- | --- | --- |
| Direct and similarly named diagnostics | [DestiLabs `mcp-doctor`](https://github.com/destilabs/mcp-doctor), [`realwigu/mcp-doctor`](https://github.com/realwigu/mcp-doctor), [`Jiansen/mcp-doctor`](https://github.com/Jiansen/mcp-doctor), and [Stephen Wilson `MCP-Doctor`](https://github.com/stephenywilson/MCP-Doctor) | Discover projects positioned around MCP diagnosis, health, readiness, or server-author recommendations; verify actual overlap dynamically |
| Official reference tools | [MCP Inspector](https://github.com/modelcontextprotocol/inspector) and [MCP Conformance](https://github.com/modelcontextprotocol/conformance) | Compare interactive debugging and protocol-conformance evidence without treating either official reference as the same product category |
| Broader testing and evaluation | [MCPJam Inspector](https://github.com/MCPJam/inspector) | Compare server testing, CI, and evaluation workflows that overlap parts of the intended developer journey |
| Security specialists | [Cisco MCP Scanner](https://github.com/cisco-ai-defense/mcp-scanner), [Snyk Agent Scan](https://github.com/snyk/agent-scan), and [MCP Server Audit](https://github.com/ModelContextProtocol-Security/mcpserver-audit) | Compare explicit vulnerability and security-analysis depth, threat models, validation safety, and evidence exchange while preserving the deliberate specialist boundary |

## Growth model

| Milestone | Exit gate | State |
| --- | --- | --- |
| M0 | Clean checkout builds; help/version work; format, Clippy, tests, dependency policy, and hosted CI pass | Done |
| M1 | One built-binary `inspect` journey identifies the earliest actionable failing layer and report-only correction in redacted human and experimental JSON for synthetic fixtures and a representative real-server compatibility matrix without calling a tool | Done |
| M2 | One immutable passive-MVP release installs and passes its diagnostic smoke journey through every advertised channel; its least-privilege repeat-release path is rehearsed | Done |
| M3 | Every retained expansion is explicitly authorized and bounded; inherited safety and stable CI output remain intact; one expanded immutable release passes every retained journey | Done |
| M4 | The `DEC-034`-locked OSPS `v2026.02.19` Level 1 controls pass with dated public evidence and official self-assessment proof; every canonical M3 GitHub Release asset is separately evaluated against SLSA `v1.2` Build L2, or a superseding decision explicitly replaces the target before proof | Ready |

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
| Output | M1 delivered redacted human output plus experimental `mcp-doctor.report/v1alpha1` JSON with the same diagnosis and result; `MCPD-012` promoted that contract to stable `mcp-doctor.report/v1` and added a JUnit-compatible projection in `v0.2.0` without changing the passive M1 activity boundary |
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

Human, JSON, and JUnit reporters derive from the same immutable result. Reports reject
an empty check set, duplicate check IDs, and findings for a different revision;
they sort checks and findings canonically. Every finding includes its code,
code-owned severity, selected revision, trusted structural location, static
message and impact, safe expectation, corrective next step, versioned
reference, and typed evidence. Arbitrary values, identifiers, paths, payloads, headers,
arguments, results, and logs cannot enter the ordinary result model;
observations retain only a safe JSON type or `[REDACTED]` and a byte count.

The current CLI exposes stable `mcp-doctor.report/v1` through
`inspect --format json`, `check --format json`, and `break --format json`.
The schema-backed envelope includes stability, revision, primary diagnosis,
independent findings, exact limits, derived summary, performed/skipped checks,
causal `blocked_by` evidence, findings, outcome, and exit code. The committed
[Draft 2020-12 schema](schemas/mcp-doctor.report.v1.schema.json), compatibility
fixtures, and built-binary journeys enforce the stable contract.

#### MCPD-012 stable reporting decision

`DEC-032` resolves `OPEN-07`. `MCPD-012` promotes the shared redacted
machine-result contract to `mcp-doctor.report/v1` and adds one conservative
JUnit-compatible XML projection. The published `v0.1.0` artifacts remain
experimental JSON-only; stable JSON and JUnit are published unchanged in
`v0.2.0` after the expanded release gates passed.

Stable `v1` JSON is the authoritative, vendor-neutral automation contract. Its
committed Draft 2020-12 schema and fixtures must cover every retained passive,
reviewed active, generated, STDIO, and HTTP journey. Existing required field
names and types, finding-code meanings and code-owned severities, check and skip
semantics, primary and independent diagnosis, outcome, exit status, redaction,
and causal relationships remain compatible throughout `v1`. Consumers must
ignore unknown optional fields and handle a previously unknown finding code
from its safe reported severity and description. Adding an optional field or a
new finding code is compatible; removing or renaming a field, changing a field
type, or changing an existing code, severity, outcome, exit, skip, redaction, or
causal meaning requires a new major report version.

JUnit is a projection of that same immutable result, not a second diagnostic
model. It maps each diagnostic check to one test case and represents performed
success, failure, warning detail, and skipped or incomplete evidence without
inventing source locations or dropping the safe finding code, structural
location, expectation, and remediation. The process exit status remains the CI
gate because consuming a JUnit artifact does not itself fail every CI job. The
projection must be deterministic, bounded, correctly escaped, value-free, and
produced without rerunning a target; representative independent CI consumers
must accept the selected common subset before compatibility is documented.
Stable JSON plus exit status remains the portable fallback when a CI system
does not render JUnit.

The selected common subset is one `testsuites`, one `testsuite`, and one
`testcase` per diagnostic check, with only `failure`, `skipped`, and
`system-out` case children. Counts and zero durations are deterministic;
attributes and text are XML 1.0 escaped; invalid XML scalar values become
U+FFFD. The checked-in fixture and ordinary tests use an independently
implemented strict XML parser. On 2026-08-11, the same fixture was also passed
directly to the Jenkins JUnit plug-in at
`67a81935603ce6740d5036f23f867ada49bd5cb3` and GitLab's JUnit parser at
`7f38b981fe5d1895345f265b70773e98927b0893`; both imported three cases as one
success, one failure, and one skip. The exact environments, scope, and update
rules are recorded in [the JUnit compatibility evidence](tests/junit/README.md).
This is representative evidence, not a universal JUnit or every-CI guarantee.

`mcp-doctor` remains a security-bounded MCP diagnostic preflight, not a general
security scanner. SARIF is deferred until an accepted security-analysis scope
has vulnerability-oriented rules, stable repository artifact or source
locations, an intended code-scanning consumer, and a reviewed upload-permission
threat model. Protocol, transport, authorization, limit, redaction, and cleanup
findings must not be relabeled as source-code vulnerabilities merely to enter a
security-alert interface.

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
| Discovery and reporting | 32 advertised revisions; 10,000 catalog items; 256 report findings; 4 MiB rendered report |
| Schema and instance | Schema 1 MiB; instance 1 MiB; 100,000 schema nodes; depth 64; local `$ref` depth 32; 100,000 evaluated schema-location/instance-location pairs; 100 collected validation errors |
| Active and generation work | 100 cases; 8 MiB aggregate active inputs; 256 generation attempts; 64 retained candidates; 100,000 generation steps |
| Network activity | Zero redirects; zero retries; concurrency 1 |

Revision selection stops and reports a limit finding at the 33rd advertised
value, even if the source could continue indefinitely. Report construction
rejects more than 256 findings, and every reporter fails safely with exit 4
instead of retaining output beyond 4 MiB. Limit construction rejects zero safety bounds,
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
3. Human and the then-experimental `mcp-doctor.report/v1alpha1` JSON output derive from the same
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
event/time limits, passive-versus-active authority, and reporter causal parity
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
schema and generator version. Human, stable JSON, and JUnit reports retain only
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
pass locally. The exact hosted and published-artifact evidence that closes M3
is recorded with `MCPD-012` below.

### MCPD-012 stable reporting and expanded-release evidence

The current source renders human, stable JSON, or JUnit output from one
immutable redacted `DiagnosticReport`. The committed
[`mcp-doctor.report/v1` schema](schemas/mcp-doctor.report.v1.schema.json)
requires the existing diagnosis, independent-safety, limit, summary, check,
skip, outcome, and exit fields while permitting compatible optional fields and
new finding codes. Unit tests prove accepted compatible additions and rejected
missing required fields. Passive, reviewed active, generated, STDIO, and HTTP
built-binary JSON journeys validate their reports against that schema.

The [JUnit projection](src/contract/report.rs) preserves one case per check,
failure and skip classification, safe finding detail, primary and independent
markers, causal `blocked_by` evidence, report outcome, and exit metadata. The
same 4 MiB renderer bound applies to human, JSON, and JUnit output; overflow
returns a safe internal failure without partial report output. Golden fixtures,
escaping and invalid-scalar cases, independent strict parsing, redaction tests,
and built-binary local/remote journeys pass the complete locked local gate. The
[consumer review](tests/junit/README.md) records successful imports by pinned
Jenkins and GitLab parsers. No target rerun, SARIF path, security-scanner
positioning, external schema retrieval, or new runtime dependency is added.

Local acceptance on 2026-08-11 is 162 tests across the complete all-target,
all-feature suite, formatting and warning-denying Clippy through the disposable
POSIX gate, and a passing all-feature locked `cargo deny` review. The 91-file
Cargo package includes the stable schema, verifies as 1.3 MiB uncompressed and
281.1 KiB compressed, produces byte-stable generated channel inputs across two
runs, installs and passes the source-channel smoke, and completes
`cargo publish --dry-run`.

[PR 14](https://github.com/EnjoyableWork/mcp-doctor/pull/14) merged the
unchanged candidate as exact `main` commit
[`b0805a8`](https://github.com/EnjoyableWork/mcp-doctor/commit/b0805a8f685e46814e358de368e2a270c21704af).
The exact-commit [native CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31528649356)
and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31528649333)
passed. The [protected release workflow](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31529740214)
published the [immutable `v0.2.0` GitHub Release](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.2.0)
and a byte-identical [crates.io package](https://crates.io/crates/mcp-doctor/0.2.0)
through short-lived OIDC authority. The tap-owned
[verification and publication workflow](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/31530330361)
copied the exact formula in commit
[`a57736e`](https://github.com/EnjoyableWork/homebrew-tap/commit/a57736ea1a7abf73eeff9a8278af11110247bd20).
The credential-free [ten-job channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31530466930)
then proved canonical byte identity and installed smokes for every represented
GitHub archive, Cargo, and Homebrew host. `MCPD-012`, D-08, and M3 are Done as
of 2026-08-11.

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
| D-08 | Bounded diagnostic expansion release | M3 | Done | `MCPD-009` through `MCPD-012` pass their bounded local and built-binary journeys; [PR 14](https://github.com/EnjoyableWork/mcp-doctor/pull/14), exact-commit [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31528649356) and [preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31528649333), the [immutable `v0.2.0` release](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.2.0), authenticated [Cargo](https://crates.io/crates/mcp-doctor/0.2.0) and [Homebrew](https://github.com/EnjoyableWork/homebrew-tap/commit/a57736ea1a7abf73eeff9a8278af11110247bd20) byte handoffs, and the [ten-job installed-channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31530466930) pass |
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
| MCPD-012 | Stabilize machine reports and CI integration, then publish and independently verify the retained M3 journeys | M3 | Done | `MCPD-011` | The committed [`mcp-doctor.report/v1` schema](schemas/mcp-doctor.report.v1.schema.json), compatibility fixtures, and local/remote built-binary journeys validate stable JSON; the bounded [JUnit projection](src/contract/report.rs) and [pinned Jenkins/GitLab imports](tests/junit/README.md) preserve safe findings, skips, outcome, and exit without target re-execution; the complete locked local gate, exact-commit [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31528649356) and [preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31528649333), [immutable publication](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31529740214), authenticated Cargo and [tap](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/31530330361) handoffs, and [ten-job installed-channel verification](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31530466930) pass for exact `v0.2.0` commit `b0805a8f685e46814e358de368e2a270c21704af` |
| MCPD-013 | Protect the default branch and define a contributor-compatible merge policy | M4 | Done | `MCPD-012` | The [canonical ruleset](.github/rulesets/main.json), [public](scripts/verify-main-protection-public.sh) and [administrative](scripts/verify-main-protection-admin.sh) verifiers, [bootstrap PR 16](https://github.com/EnjoyableWork/mcp-doctor/pull/16), normal protected [PR 17](https://github.com/EnjoyableWork/mcp-doctor/pull/17), rejected destructive-path exercises, the closed [emergency record](docs/assurance/mcpd-013-emergency-exercise.md) with [PR 18](https://github.com/EnjoyableWork/mcp-doctor/pull/18), and final protected [closure PR 19](https://github.com/EnjoyableWork/mcp-doctor/pull/19) prove the selected approval, aggregate-check, merge, bypass, signing, deletion, and non-fast-forward policy; public and non-disclosing evidence boundaries remain explicit |
| MCPD-014 | Establish vulnerability disclosure and live repository-security controls | M4 | Done | `MCPD-013` | The accepted [security policy](SECURITY.md), [canonical control projection](.github/security-controls.json), and [non-disclosing verifier](scripts/verify-security-controls.sh) define the supported-version, response, disclosure, entitled-control, limitation, and clean-baseline contract; protected [PR 20](https://github.com/EnjoyableWork/mcp-doctor/pull/20), [PR 21](https://github.com/EnjoyableWork/mcp-doctor/pull/21), and final [closure PR 22](https://github.com/EnjoyableWork/mcp-doctor/pull/22), representative and exact-`main` CodeQL, both exact-`main` aggregate workflows, and the final scoped verifier pass provide the dated completion evidence below |
| MCPD-015 | Verify the public contribution, community, repository, and licensing contract | M4 | Done | `MCPD-014` | The [canonical projection](.github/community-license-controls.json), [public guide](docs/project-scope.md), [credential-free verifier](scripts/verify-community-license.sh), [tap PR 3](https://github.com/EnjoyableWork/homebrew-tap/pull/3), protected [source PR 23](https://github.com/EnjoyableWork/mcp-doctor/pull/23), exact-`main` hosted gates, and the dated pass below prove recognized public workflows, complete repository inventory, HTTPS-only official channels, and exact source, package, archive, and formula license evidence with explicit auxiliary-asset limitations |
| MCPD-016 | Harden dependency maintenance and the CI, artifact, and distribution supply chains | M4 | In progress | `MCPD-015` | Reviewed non-auto-merged dependency update proposals preserve exact direct requirements and pass maintenance/provenance/graph checks; full-SHA action inventory, fork and permission policy, tracked-artifact rejection, authenticated distribution verification, negative exercises, and operator audit pass against exact `main` and the immutable release |
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
| [`quick-xml` `=0.41.0`](https://crates.io/crates/quick-xml/0.41.0) | `MCPD-012` — Done | Development-only strict parsing of the conservative JUnit XML subset and escaping boundary; defaults are disabled, no optional feature is selected, and no quick-xml code or transitive is linked into the release binary |
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

#### MCPD-012 quick-xml adoption review — 2026-08-11

The standard library has no strict XML parser, and a hand-written test parser
would be weak evidence for XML nesting, attributes, escaping, and invalid-scalar
behavior produced by the new JUnit projection. Development-only
[`quick-xml`](https://github.com/tafia/quick-xml) `0.41.0`, released 2026-06-29,
provides a small pull parser for only the checked-in synthetic output. It does
not parse target traffic, production data, or runtime configuration.

The established public repository was active, unarchived, and maintained by
multiple owners at review. The crate is MIT-licensed, declares Rust 1.79, has no
build script or native linkage, and forbids unsafe Rust. Default features are
empty and remain disabled. The selected graph adds only `quick-xml`; its sole
normal dependency is the already locked `memchr`, so there is no new transitive
package, duplicate version, release-binary size, startup, or runtime cost.
Version `0.41.0` also includes bounded-namespace and quadratic-attribute fixes.
The upstream has no dedicated security policy; its active issue and release
paths plus RustSec and `cargo-deny` monitoring are accepted for this narrow,
removable test-only role. An advisory, ownership or provenance change,
unexplained inactivity, build/unsafe/feature expansion, or disappearance of the
independent-parser need triggers removal or full re-review.

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
| `quick-xml` | `MCPD-012` — selected 2026-08-11 | Adopted development dependency | Strictly parse the checked-in common JUnit subset and escaping cases in ordinary Rust tests; keep all features disabled and out of the runtime graph |
| `tempfile` | `MCPD-002` — Done | Adopted development dependency | Reuse the reviewed disposable-root boundary; do not add another temporary-resource package for convenience |
| `cargo-deny` `0.20.2` | `MCPD-003` — Done; delivery hardened by `MCPD-016` | Adopted development/CI tool | Keep advisory, license, ban, and source checks locked; fetch only the exact `x86_64-unknown-linux-musl` archive through HTTPS, require reviewed SHA-256 `9f12ed4c49936e09b48bf862b595cde2fe64fcbd9d74dfacac6131ca824c8d5f` and exact layout before execution, and re-review tool or policy changes. The former Action downloaded this executable without checking its digest and is removed rather than retained as a full-SHA false assurance. |
| `assert_cmd` | 2026-08-10 review | Not adopted | The existing standard-library built-binary harness already controls arguments, environment, time, output, and process fixtures; reconsider only if duplicated orchestration becomes harder to review safely |
| `insta` | 2026-08-10 review | Not adopted | Current human/JSON/JUnit golden files and catalog fixtures remain small and explicit; reconsider only when direct snapshots become materially harder to review |
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
| Pinned Jenkins and GitLab JUnit parsers | `MCPD-012` — evaluated 2026-08-11 | Diagnostic only; not adopted | Exercise the canonical fixture in disposable upstream environments to verify representative CI import; retain exact source/image evidence, but do not add Maven, Ruby, either CI product, or their dependency graphs to normal development or CI |

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
| MCPD-012 | Complete `MCPD-012` under `DEC-032`: promote the redacted authoritative machine-result contract to schema-backed `mcp-doctor.report/v1`; add one deterministic, bounded, correctly escaped JUnit-compatible projection of the same result without rerunning a target; preserve safe findings, causal skips, outcomes, and exit semantics across every retained local and remote journey; and verify the selected common JUnit subset with representative independent CI consumers. Keep SARIF and general security-scanner positioning out of scope. Then publish one protected immutable expanded version with authenticated artifacts and installed smokes for every represented channel. Do not retain an M3 feature that its owning ticket deferred or cancelled. |
| MCPD-013 | Complete `MCPD-013` under `DEC-034` and `DEC-035`: revalidate the locked M4 versions, then protect `main` with the canonical contributor-compatible public ruleset and repository merge settings; add the two non-skipping aggregate required gates; implement credential-free drift verification for the public rule projection plus non-disclosing authenticated readback for hidden bypass state; and prove the normal pull-request path, rejected direct update, deletion and force-push paths, and bounded pull-request-only emergency bypass lifecycle. Do not require an unavailable reviewer or unproven commit-signing path, change immutable release bytes, or begin later assurance tickets. |
| MCPD-014 | Complete `MCPD-014`: establish recognized supported-version, security-contact, private-reporting, response, and coordinated-disclosure guidance; enable and read back the entitled dependency, code-scanning, secret-prevention, and private-reporting controls; document unavailable features exactly; and verify a non-disclosing clean baseline. Do not publish a complete-baseline claim. |
| MCPD-015 | Complete `MCPD-015`: verify public contribution, conduct, support, defect-reporting, repository-inventory, official-channel, inbound-license, source-license, and released-asset license contracts across every in-scope repository and distribution channel. Avoid nominal reviewers, owners, or controls, and do not begin supply-chain changes. |
| MCPD-016 | Complete `MCPD-016`: automate grouped dependency update proposals without auto-merge and verify they preserve exact direct requirements plus the accepted maintenance, provenance, feature, transitive-graph, license, advisory, Rust/platform, and behavioral review; inventory and verify every selected Action at a reviewed full commit SHA; prove untrusted workflows are read-only and secretless; reject generated executables and unreviewable binary artifacts; and authenticate the immutable release, Cargo package, and Homebrew formula without changing published bytes. |
| MCPD-017 | Complete `MCPD-017`: define and verify strong-MFA, lowest-default-access, manual-grant, repository-creation, installed-application, automation-credential, ownership-continuity, and recovery controls using aggregate non-sensitive evidence. Any live organization mutation or private recovery confirmation requires explicit owner authority and must not expose identities or recovery material. |
| MCPD-018 | Complete `MCPD-018` under `DEC-034`: confirm that the activation-locked OSPS `v2026.02.19`, BadgeApp baseline series `v2026.02.19`, and SLSA `v1.2` proof routes remain current and available or stop for a superseding decision; publish the two dated and scoped crosswalks; complete the official baseline-1 self-assessment only after every applicable control passes; verify its public record, JSON, badge, and exact-`main` evidence; and verify every canonical M3 GitHub Release asset against Build L2 using its exact digest and constrained signed provenance. Define annual and event-driven review and removal triggers. Never imply independent certification, regulatory compliance, higher OSPS levels, channel-wide or future-artifact SLSA coverage, or paid platform signing. |

## M4 enterprise assurance boundary

M4 turns existing project, release, repository, and organization practices into
dated, scoped, independently inspectable adoption evidence. It is a
post-release assurance milestone, not another product release and not a reason
to rewrite, replace, or weaken the immutable M3 artifacts.

### Locked assurance target and proof

`DEC-034` selects the following exact first-M4 target, revalidated against
official primary sources on 2026-08-11:

- [OpenSSF OSPS Baseline `v2026.02.19`](https://baseline.openssf.org/versions/2026-02-19)
  Level 1, including all 24 controls in the map below;
- the official BadgeApp baseline series displaying OSPS `v2026.02.19`, as
  recorded by its pinned
  [version configuration](https://github.com/ossf/best-practices-badge/blob/16340332a377c483d82ca4634eaa6799af9bab64/app/lib/baseline_config.rb);
  and
- the approved [SLSA `v1.2`](https://slsa.dev/spec/v1.2/) Build L2
  requirements.

The `MCPD-013` activation recheck on 2026-08-11 passed. The official OSPS
current-version pointer and exact Level 1 page still identify `v2026.02.19` and
all 24 planned controls; BadgeApp's pinned configuration and its then-current
configuration at commit
[`73db726`](https://github.com/ossf/best-practices-badge/blob/73db726e5bc898903995ad63e471ff6f820086e2/app/lib/baseline_config.rb)
both select `v2026.02.19` without a transition; the official public assessment
route remains available; and the approved SLSA specification pointer still
resolves to `v1.2`, whose Build L2 proof contract remains hosted-build signed
provenance with consumer-side authenticity verification. This clears the
pre-activation drift gate only; it is not an achieved assurance result.

Before `MCPD-013` changes live repository state, it must recheck the official
current-version pointers, exact BadgeApp series, and proof availability. Any
different current version, unavailable exact series, withdrawn proof, or
material issuer change blocks M4 activation until a superseding decision
updates the scope and control map. Activation locks one internally consistent
set; M4 never silently floats, mixes framework versions, or treats a mutable
service deployment as a versioned standard. A superseding or withdrawn target
before `MCPD-018` publishes proof causes the same stop-and-decide gate. A later
change after publication triggers immediate claim review, correction, or
removal under the public-proof policy.

| Proof target | Exact evidence contract |
| --- | --- |
| OSPS Level 1 | `docs/assurance/osps-v2026.02.19-level-1.md` names the date, exact `main`, assessed repository, organization and release boundary, every control, status, applicability reasoning, non-sensitive evidence, limitations, and review triggers. Every applicable control must pass; an N/A needs explicit framework-compatible reasoning. |
| BadgeApp baseline-1 | The official public assessment must display `v2026.02.19`, reach 100% through `Met` or justified N/A answers, and link its justifications to the project crosswalk. Proof comprises the stable assessment ID and HTML page, its public project JSON, and the official baseline badge SVG linking back to that assessment, all re-read on exact `main`. The result is an official-hosted self-assessment, not independent certification. |
| SLSA Build L2 | `docs/assurance/slsa-v1.2-build-l2.md` lists every asset in the immutable canonical M3 GitHub Release with its SHA-256 digest and maps every `v1.2` L2 requirement. One exactly reviewed and pinned `gh` release verifies each public attestation against the exact repository, signer workflow, tag ref, and source commit; retained structural evidence confirms the subject digest, GitHub-hosted builder identity, and `predicateType` is `https://slsa.dev/provenance/v1`. The artifact-specific result does not cover registry or Homebrew operations, dependencies, unlisted or future artifacts, or a project-wide certification. |

BadgeApp account creation, human OAuth, acceptance of proposed answers, and
publication of the assessment are explicit owner actions at `MCPD-018`.
Automation may prepare evidence and verify public output, but it may not assert
answers or publish a badge for the owner.

### Accepted default-branch policy

The 2026-08-11 read-only baseline found a public repository whose one admin is
its only collaborator, with no ruleset or legacy branch protection on `main`,
all three merge methods enabled, and passing current CI and release-preflight
jobs. CodeQL default setup and secret-scanning controls were not configured.
This records the gap; it is not achieved protection, review, or scanner
evidence.

Later on 2026-08-11, [bootstrap PR 16](https://github.com/EnjoyableWork/mcp-doctor/pull/16)
passed both new aggregate gates and landed before the repository enabled
squash-only merge settings and active public ruleset
[`20718365`](https://github.com/EnjoyableWork/mcp-doctor/rules/20718365).
The first credential-free readback failed closed because GitHub materialized a
disabled dismissal restriction and empty required-reviewer list that were not
yet explicit in the submitted canonical projection. The authenticated readback
independently confirmed an empty bypass list. Normal protected
[PR 17](https://github.com/EnjoyableWork/mcp-doctor/pull/17) canonicalized those
inactive fields, remained blocked until both required aggregates passed, and
then merged without review or bypass, matching the accepted single-maintainer
policy.

The same credential-free readback also established that GitHub's current
unauthenticated repository response exposes `default_branch` but omits every
merge-setting field, while authenticated readback exposes those settings.
`DEC-036` therefore refines only the `DEC-035` verification boundary: the
credential-free verifier proves the default branch plus public configured and
effective rules, and the non-disclosing authenticated verifier proves both the
canonical repository merge projection and exact empty bypass state. The
selected merge, approval, check, target, deletion, non-fast-forward, signing,
and emergency values are unchanged. Missing authenticated merge fields now
fail closed rather than being misrepresented as public evidence.

`DEC-035` fixes the `MCPD-013` implementation contract:

| Choice | Accepted policy |
| --- | --- |
| Scope and enforcement | Check in the normalized canonical ruleset as `.github/rulesets/main.json`; activate one public repository ruleset for only `refs/heads/main`; require a pull request, linear history, strict required status checks, and resolved conversations; block deletion and non-fast-forward updates. The pull-request rule, not a push allowlist, prevents direct commits. Under `DEC-036`, a credential-free verifier compares the publicly visible default branch plus configured and effective live rules with the canonical file. GitHub omits repository merge settings and bypass actors from credential-free REST readback, so a separate authenticated owner check must verify the exact canonical merge projection and empty bypass list and publish only its date, canonical hash, and pass/fail result. |
| Approval count | Set `required_approving_review_count` to `0`. Do not enable code-owner, stale-approval, or last-push approval requirements while the only maintainer cannot supply an independent approval. Never describe this as peer review. A second active independent maintainer plus a successfully rehearsed normal path triggers a separate policy revision toward one approval; it does not change the count automatically. |
| Required checks | Add exact GitHub-Actions-bound contexts `Required CI` and `Required release preflight`, with strict branch-up-to-date enforcement. `Required CI` depends on dependency policy plus GNU/Linux x64, macOS ARM64, and Windows x64 format, Clippy, and test jobs. `Required release preflight` depends on deterministic source/formula generation, macOS ARM64, GNU/Linux ARM64/x64, Windows x64, and exact non-publishing payload verification. Both aggregate jobs use `needs` with `always()` and fail unless every intended dependency succeeded, so a failed, cancelled, or skipped dependency cannot turn green. Limit ordinary branch `push` triggers to `main` so a same-repository branch push cannot produce a duplicate required context alongside its pull-request run. |
| Future security gates | Dependency policy is already inside `Required CI`. `MCPD-014` configured CodeQL and secret prevention and proved CodeQL on a representative protected pull request and exact `main`, but neither is a `DEC-035` required gate because that ticket did not accept a new ruleset context or contributor-handling contract. A later exact code-scanning rule or separately named security context requires its own accepted ruleset update. A future `mcp-doctor` MCP security scanner remains product behavior rather than a repository check by default. |
| Merge method | Enable squash merge only, require linear history, and disable merge commits and rebase merges. Use the pull-request title and body for the squash commit; the existing Conventional Commits policy governs the title. Enable contributor branch-update suggestions and deletion of the head branch after merge. Keep auto-merge and merge queue disabled until measured need justifies either. |
| Standing bypass | Keep `bypass_actors` empty, including for repository administrators, GitHub Apps, and Dependabot. Because this field is hidden from public unauthenticated readback, verify it administratively at activation, after every emergency, and at each assurance review; expose no actor inventory. Administrators can still edit repository rules, so the canonical config, public projection, bounded private readback, and emergency record are the honest controls; no text may claim that GitHub makes the owner technically unable to change policy. |
| Emergency administration | When a material incident cannot wait for the normal gates, use a dedicated pull request and temporarily add only the repository-administrator role with `pull_request` bypass mode. Record a non-sensitive incident ID, reason, exact commit, rules bypassed, canonical pre-change hash, start time, and rollback owner. Never disable the ruleset, grant `always` bypass, push directly, delete `main`, or force-push. Remove the actor immediately after the one merge, re-run the credential-free public-projection verifier, authenticated empty-bypass readback, and all gates on the merged commit, and publish a non-sensitive closure record; security-sensitive detail remains private. Any additional administrator or changed GitHub capability blocks use until this actor boundary is re-decided. |
| Commit signing | Required commit signing stays off. Existing artifact provenance is not relabeled as commit signing, and `MCPD-013` does not impose an unproven contributor, Dependabot, web-squash, local-tooling, or emergency path. A later focused ticket may reconsider only after all normal and emergency actors succeed with verified signatures and its merge-method interaction is accepted. |

`MCPD-013` must read back effective layered rules, repository merge settings,
and the private empty-bypass result, not merely the submitted JSON. It must
prove one normal protected pull request, safe rejected direct-update, deletion,
and force-push paths after confirming the active target, and the temporary
pull-request-only bypass add/remove lifecycle without leaving a standing
exception. Public evidence must distinguish independently reproducible fields
from the self-attested non-disclosing bypass result. Security controls are
owned by `MCPD-014`; resolving this policy did not activate them early.

### MCPD-013 completion evidence

`MCPD-013` completed on 2026-08-11 with canonical ruleset SHA-256
`2e3377a5101c513c02bb177cbc95acc3707f77bab4c3ab8ed3e8576a3f828794`.
This is scoped repository-governance evidence, not an OSPS, SLSA, scanner,
security-baseline, certification, or complete-M4 claim.

The independently inspectable evidence is:

- [bootstrap PR 16](https://github.com/EnjoyableWork/mcp-doctor/pull/16) at
  `47a5e1c9389c1993c18aadd0ad94ec2a1039c5ea` introduced the two aggregates;
  its [Required CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31535865111/job/93927521304)
  and [Required release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31535865082/job/93929885464)
  jobs passed before activation;
- active public ruleset
  [`20718365`](https://github.com/EnjoyableWork/mcp-doctor/rules/20718365), the
  checked-in canonical projection, and the credential-free verifier expose the
  exact `main` target, pull-request, strict status-check, linear-history,
  deletion, and non-fast-forward rules without requiring GitHub credentials;
- normal protected [PR 17](https://github.com/EnjoyableWork/mcp-doctor/pull/17)
  at `6555e5624ecfd3eac706fa6420fcb4947dcf0b45` had zero reviews and no bypass,
  remained blocked until its [Required CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31537654995/job/93933333425)
  and [Required release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31537655042/job/93935386965)
  jobs passed, and then squash-merged as
  `09765f6fe13eb050de32033fc6d51b3e8b5da37f`; and
- dedicated emergency [PR 18](https://github.com/EnjoyableWork/mcp-doctor/pull/18)
  and its [public closure record](docs/assurance/mcpd-013-emergency-exercise.md)
  preserve exact commit, time, scope, rollback, empty-bypass, and post-merge
  evidence. Its squash commit
  `8487b47dbddb2dd1c50020b5b157d9807bc4fcd7` passed fresh
  [Required CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31539153287/job/93938063807)
  and [Required release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31539153316/job/93940246247)
  jobs after bypass removal.

Final protected [closure PR 19](https://github.com/EnjoyableWork/mcp-doctor/pull/19)
publishes this status and evidence through the normal no-bypass path; it may
merge only after both required aggregates pass.

The dated operator and non-disclosing evidence is deliberately narrower. With
the active target and empty bypass state confirmed on
`29d83e094b1112b6c86fbcabeb93667e11e02a53`, direct update, primary-branch
deletion, and leased same-tree non-fast-forward force-update attempts were each
rejected and `main` remained exact after every attempt. The authenticated
verifier then passed with only
`date=2026-08-11`, the canonical hash above, and `result=PASS`; it disclosed no
merge-setting payload, actor inventory, or identity. The emergency record
separately discloses the bounded add/remove sequence, including its safe
pre-merge rollback, without exposing an administrator identity. These
self-attested operations support but do not replace the reproducible public
ruleset projection.

### Accepted vulnerability-disclosure and repository-security policy

`DEC-037` fixes the `MCPD-014` contract. The checked-in
[`SECURITY.md`](SECURITY.md) is the recognized policy, and
`.github/security-controls.json` is the canonical machine-readable projection
for the live repository controls and the limits of their evidence. GitHub's
private vulnerability reporting form is the sole public security contact;
maintainers do not publish a personal address or ask reporters to disclose a
suspected vulnerability in an issue, pull request, discussion, or support
request.

| Choice | Accepted policy |
| --- | --- |
| Supported versions | Support only the latest published minor line, currently `0.2.x`. `0.1.x` is unsupported, and `main` is development-only without a release or backport guarantee. A report about an older version remains welcome, but upgrade may be the resolution. Publishing a new minor line requires this table and the canonical projection to change in the same reviewed release work. |
| Contact and route | Direct reporters to the repository's [private vulnerability reporting form](https://github.com/EnjoyableWork/mcp-doctor/security/advisories/new). Ask for only minimal synthetic or structurally redacted evidence, require authorization for any tested system, and direct a reporter to rotate an exposed secret before reporting it. There is no bounty or compensation promise. |
| Response targets | Aim to acknowledge within 3 business days, give an initial assessment or status within 7 calendar days, and update at least every 14 calendar days until closure. These are targets rather than an SLA or guaranteed remediation date; active exploitation, credential exposure, and immediate harm receive priority. |
| Coordinated disclosure | Keep the report private while scope, correction or mitigation, and timing are coordinated. Generally target disclosure within 90 days of acknowledgement, preferably after a fix or verified mitigation; shorten for active exploitation or extend by mutual agreement when safe correction or upstream coordination needs more time. Publish a GitHub Security Advisory with affected/fixed versions, mitigation, and agreed credit when appropriate, and request a CVE when warranted. |
| Entitled dependency controls | Keep the dependency graph and vulnerability alerts enabled, keep automated security fixes enabled, and enable Dependabot security updates. The existing weekly version-update proposals remain review-only and are not evidence that a dependency is safe. |
| Entitled code scanning | Enable CodeQL default setup for only GitHub Actions and Rust with the default query suite, standard runner, weekly schedule, and remote threat model. Require successful analyses of exact `main` for both languages and no open code-scanning alert before the baseline passes. This repository analysis is not a general MCP server security scanner. |
| Entitled secret prevention | Enable secret scanning and push protection, require the repository-visible alert endpoint to be reachable with secret values hidden, and require no open repository-visible alert before the baseline passes. GitHub [documents that enablement scans the repository's full Git history](https://docs.github.com/en/code-security/how-tos/secure-your-secrets/detect-secret-leaks/enable-secret-scanning), but its scan-history readback requires Advanced Security and is unavailable on this GitHub Free repository; the baseline therefore does not attest backfill completion. Values and finding details never enter public evidence. |
| Private reporting | Enable private vulnerability reporting and verify both the API setting and the recognized root policy path. A future advisory remains private until coordinated publication. |
| Non-disclosing baseline | `scripts/verify-security-controls.sh` performs authenticated, bounded readback against the canonical projection, exact default-branch CodeQL commit, readable dependency graph, and empty repository-visible open-alert responses. It writes API bodies only to a mode-`0700` temporary directory, suppresses API errors, removes the directory on exit, and emits only UTC date, canonical SHA-256, and `PASS` or `FAIL`. A failure is investigated privately and blocks ticket completion. |
| Merge-gate boundary | Do not add CodeQL or secret scanning to the `DEC-035` required ruleset in this ticket. Default setup passed both exact `main` and a representative protected pull request, but this does not establish a required-context or contributor-handling contract; `Required CI` already includes locked dependency policy. Any later required security context needs a separate accepted ruleset update. |

The initial authenticated read-only review on 2026-08-11 found the organization
on GitHub Free. Vulnerability alerts and automated security fixes were already
reachable, while Dependabot security updates, CodeQL default setup, secret
scanning, push protection, and private vulnerability reporting still required
activation. CodeQL identified GitHub Actions and Rust as the applicable
languages. Alert bodies were neither printed nor retained. This is a
pre-activation gap record, not a clean baseline or achieved assurance result.

Protected bootstrap [PR 20](https://github.com/EnjoyableWork/mcp-doctor/pull/20)
landed the contract as exact `main`
`7097b683fc6619447b31db0b55db12467626e446` after both required aggregates
passed. The repository then enabled vulnerability alerts, automated security
fixes, Dependabot security updates, secret scanning, push protection, private
vulnerability reporting, and CodeQL default setup. The initial CodeQL write
omitted its language override because GitHub's REST write schema rejected
`rust`; the generated exact-`main`
[CodeQL run 31545582099](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31545582099)
nevertheless detected and passed GitHub Actions and Rust, after which readback
reported the canonical two languages, default suite, standard runner, weekly
schedule, and remote threat model. A local status-only diagnostic also passed
the policy path, private route, dependency graph and controls, exact-commit
analyses, and empty repository-visible dependency, code, and value-hidden
secret alert responses without printing any alert body. This is activation
evidence rather than the final baseline by itself.

The scoped GitHub Free baseline cannot verify or enable Secret Protection
validity checks, non-provider and generic patterns, AI generic-secret
detection, delegated push-protection bypass, or Enterprise public-leak
monitoring. Its Advanced-Security-only secret-scan history endpoint returns an
unavailable result even though the repository-visible alert and prevention
controls are enabled, so `DEC-038` excludes backfill-completion attestation
instead of manufacturing it. Provider-routed partner alerts for public
repositories are not visible to repository administrators and are also
excluded from the clean-baseline evidence. The canonical projection records
each limitation explicitly. A plan, product-entitlement, feature, scanner,
supported-version, or GitHub API change fails closed and triggers review rather
than silently widening the claim.

This work contributes only the `OSPS-BR-07.01` and `OSPS-VM-02.01` rows assigned
to `MCPD-014`. It does not prove all OSPS Level 1 controls, BadgeApp status,
SLSA, independent certification, regulatory compliance, a complete M4
baseline, or a security warranty.

### MCPD-014 completion evidence

`MCPD-014` completed on 2026-08-11 with canonical security-control SHA-256
`d379f2c86b9571da14cdb9c51cfc83075f098688a4660aecb67eb60fa385e66a`.
This is a dated, scoped repository-security baseline, not a product
security-scanner, complete-M4, framework-conformance, certification,
regulatory-compliance, or warranty claim.

The protected and independently inspectable evidence is:

- bootstrap [PR 20](https://github.com/EnjoyableWork/mcp-doctor/pull/20)
  introduced the policy, canonical projection, verifier, and regression tests,
  then squash-merged as exact `main`
  `7097b683fc6619447b31db0b55db12467626e446` without bypass after both
  required aggregates passed;
- the initial exact-`main`
  [CodeQL run 31545582099](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31545582099)
  passed GitHub Actions and Rust after activation;
- protected correction [PR 21](https://github.com/EnjoyableWork/mcp-doctor/pull/21)
  aligned the verifier with GitHub's observed CodeQL category and GitHub Free
  secret-scan-history surfaces. Its representative
  [CodeQL run 31546161736](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31546161736),
  [Required CI run 31546164626](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31546164626),
  and [Required release preflight 31546164631](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31546164631)
  passed before the normal no-bypass squash merge; and
- on resulting exact `main`
  `7f777b32e88356cea8f0212ec9bfa61a7373907b`, fresh
  [CodeQL run 31547028561](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31547028561)
  passed GitHub Actions and Rust, while
  [Required CI run 31547028549](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31547028549)
  and [Required release preflight 31547028600](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31547028600)
  passed their complete matrices.

After those exact-commit analyses completed, the unchanged authenticated
verifier emitted only
`date=2026-08-11 canonical_sha256=d379f2c86b9571da14cdb9c51cfc83075f098688a4660aecb67eb60fa385e66a result=PASS`.
That pass read back the recognized policy path, private reporting, dependency
controls and SPDX dependency graph, canonical CodeQL setup and exact-commit
analyses, enabled secret prevention, and zero open repository-visible
dependency, code, and value-hidden secret alert responses. No alert body,
count payload, secret value, credential source, or finding detail was retained
as public evidence. The unavailable scan-history, provider-only alert, paid
feature, future product-scanner, and complete-M4 exclusions above remain part
of the result rather than being treated as passes.

Final protected [closure PR 22](https://github.com/EnjoyableWork/mcp-doctor/pull/22)
publishes this completed status through the normal no-bypass path. The PR is
also the durable record for fresh exact-`main` CodeQL, aggregate-workflow,
security-control, and branch-protection readback after its merge.

### Accepted community, repository, channel, and license contract

`DEC-039` fixes the `MCPD-015` boundary. The checked-in
[project-scope guide](docs/project-scope.md) is its human-readable inventory,
`.github/community-license-controls.json` is the canonical machine-readable
projection, and `scripts/verify-community-license.sh` is the credential-free
verifier. The contract covers public community and license evidence only; it
does not change dependencies, workflows, repository authority, release bytes,
or the later supply-chain and complete-assurance boundaries.

| Choice | Accepted policy |
| --- | --- |
| Repository inventory | Enumerate every public `EnjoyableWork` repository. `mcp-doctor` is the primary in-scope source and policy repository; `homebrew-tap` is an in-scope supporting distribution codebase only for its `mcp-doctor` policy links, MIT repository license, formula, and release handoff. `courtside-mcp`, `enjoyable-mcp`, and `mcp-sync` are active separate products recorded to prevent hidden scope, not silently covered by this assessment. A new, missing, archived, forked, relicensed, renamed, or otherwise unclassified public repository fails verification for review. |
| Policy ownership and delegation | Keep contribution, conduct, support, defect, and security policies in the primary repository. The tap README identifies its supporting role and routes `mcp-doctor` changes and reports to those exact HTTPS policies rather than duplicating files that can drift. [Tap PR 3](https://github.com/EnjoyableWork/homebrew-tap/pull/3) landed that route as commit [`8d5421a`](https://github.com/EnjoyableWork/homebrew-tap/commit/8d5421abed22e46b43de35f0876bc65edcd6e0d6) without changing a formula, workflow, credential, or release. |
| Public discussion and defects | GitHub Issues is the single public project discussion mechanism. The recognized bug and feature forms collect actionable, safety-bounded evidence; blank issues remain disabled; `SUPPORT.md` names the current release, direct forms, scope, and sensitive-data boundary. No chat room, social account, personal contact, or separate tap tracker is an official support promise. |
| Conduct reports | Use GitHub's private repository **Report content** action, whose live enablement is verified, for a concern attached to project content. Use GitHub Support when the action is unavailable or the concern is immediate or platform-wide. Do not depend on an unpublished personal address, nominal reviewer, or named owner, and do not disclose another person's private information in a public issue. Vulnerabilities retain the distinct private route in `SECURITY.md`. |
| Contributions and inbound license | `CONTRIBUTING.md` remains the recognized workflow and defines ticket, safety, test, pull-request, and dependency expectations. Contributions use the same inbound and outbound OSI-approved MIT terms; no CLA or mandatory DCO sign-off is claimed, while voluntary sign-off remains allowed. |
| Official channels | The canonical set is the HTTPS source repository, issue tracker, private vulnerability form, exact `v0.2.0` GitHub Release, crates.io `0.2.0` package, tap formula, and third-party `docs.rs` documentation mirror. The mirror is not release authority. HTTP, alternate registries or mirrors, personal contacts, and unlisted communication services are not official project channels. |
| Source and released-asset licenses | Require exact MIT metadata and root license hash for source. The immutable `0.2.0` Cargo package and both native archives must contain that exact license; crates.io must report `MIT`; the released and tap formulas must be byte-identical and declare `MIT`; and the tap must contain the same license. Enumerate all seven release assets by exact name, size, and digest so auxiliary SBOM and checksum metadata cannot be mistaken for another software distribution. |
| SPDX and metadata limitation | The immutable SPDX documents use `CC0-1.0` for the document and `NOASSERTION` for the root package license. They are not used as MIT evidence; package metadata and embedded license files supply that proof. `SHA256SUMS` is metadata accompanying the licensed release set. This limitation is explicit rather than rewriting immutable assets or manufacturing an assertion. |
| Credential-free verification | Use only bounded direct public HTTPS reads with no credential, proxy, ambient curl configuration, cookie, or `.netrc` source. Compare the complete live public-repository inventory, primary repository and recognized-community state, exact policy files, tap delegation and license files, official channels, immutable release metadata, tag source, crates.io metadata and package bytes, embedded archive licenses, formula declaration, and SPDX document-license limitation. Emit only UTC date, canonical SHA-256, verified source ref, and `PASS` or `FAIL`. |

The 2026-08-12 pre-activation review found five public organization
repositories and the two-repository project boundary above. GitHub recognized
the primary README, MIT license, contribution guide, code of conduct, and pull
request template; issues and private content reporting were enabled; and both
structured issue forms existed even though GitHub's community-profile API does
not enumerate issue forms. Source, Cargo, both native archives, the release
formula, and the tap already had exact MIT evidence. Three truthful operating
gaps remained: the conduct policy pointed to a nonexistent public maintainer
contact, support still called the released project pre-release, and the tap did
not route its `mcp-doctor` users to the canonical policies. The tap gap is now
repaired through tap PR 3; source PR 23 repaired the other two and added the
complete contract and verifier through the normal protected path. The dated
exact-`main` evidence below now closes the ticket.

This work contributes only the nine OSPS Level 1 rows assigned to `MCPD-015`
in the planning map below. It does not claim the full baseline, an official
self-assessment, independent certification, authenticated distribution, or a
security or supply-chain warranty.

### MCPD-015 completion evidence

`MCPD-015` completed on 2026-08-12 with canonical community-and-license
SHA-256
`ae1898c2f6af70578d3c61810377ce57b6ee5f694b0e5db8e7bcd015de67daa9`.
This is a dated public community, repository, channel, and license result for
the two in-scope repositories and exact `v0.2.0` distribution set. It is not a
complete OSPS result, independent certification, authenticated supply-chain
result, product security-scanner result, regulatory-compliance claim, or
warranty.

The protected and independently reproducible evidence is:

- [`homebrew-tap` PR 3](https://github.com/EnjoyableWork/homebrew-tap/pull/3)
  classified the tap as the supporting `mcp-doctor` distribution codebase and
  delegated contribution, defect, support, conduct, and security requests to
  the canonical source policies. It squash-merged as
  [`8d5421a`](https://github.com/EnjoyableWork/homebrew-tap/commit/8d5421abed22e46b43de35f0876bc65edcd6e0d6)
  without changing a formula, workflow, dependency, credential, or release;
- protected [source PR 23](https://github.com/EnjoyableWork/mcp-doctor/pull/23)
  added the canonical projection, public scope guide, corrected policies,
  bounded verifier, and four focused regression tests. Its exact head
  `ca26052da9de610da91fe206fb0be1862f4c37e9` passed the
  [Required CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31564150762/job/94013030028),
  [Required release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31564150768/job/94014462121),
  and [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31564148438)
  checks before a normal no-bypass squash merge as exact `main`
  [`6f1bed2`](https://github.com/EnjoyableWork/mcp-doctor/commit/6f1bed224aa27c468b64c19b99288122e401a96a);
- the complete locked local gate passed 175 tests across all targets and
  features, warning-denying Clippy, and formatting through the disposable
  environment; `cargo deny --all-features --locked check`, ShellCheck, JSON
  parsing, and dirty-tree package inventory also passed without a dependency,
  workflow, release, artifact, or supply-chain change; and
- on exact `main` `6f1bed224aa27c468b64c19b99288122e401a96a`, fresh
  [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31564865655)
  passed Actions and Rust,
  [Required CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31564866151/job/94015001184)
  passed all three native hosts, and
  [Required release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31564866091/job/94016103246)
  passed deterministic generation, four source-install and represented-artifact
  hosts, and the exact non-publishing payload.

The unchanged verifier then emitted only
`date=2026-08-12 canonical_sha256=ae1898c2f6af70578d3c61810377ce57b6ee5f694b0e5db8e7bcd015de67daa9 source_sha=6f1bed224aa27c468b64c19b99288122e401a96a result=PASS`.
That credential-free pass resolved `main` once; compared all five live public
organization repositories; verified the recognized primary community files,
issues, private content reporting, and exact policy bytes; verified the tap
delegation and MIT license; read the immutable release and annotated tag;
downloaded and hashed all seven exact release assets; checked the embedded
Cargo and native-archive licenses, formula declaration and equality, checksum
manifest, crates.io metadata and byte identity, and SPDX `CC0-1.0` plus
`NOASSERTION` limitation; and reached the HTTPS documentation mirror. It used
no GitHub credential, proxy, ambient curl configuration, cookie, or `.netrc`
source and retained no downloaded evidence after the bounded run.

Final protected [closure PR 24](https://github.com/EnjoyableWork/mcp-doctor/pull/24)
publishes this status and evidence through the normal no-bypass path. It may
merge only after both required aggregates pass; after merge, its public timeline
is the durable record for the final exact-`main` verifier, CodeQL, CI,
release-preflight, and protection readback.

This closure makes `MCPD-016` Ready but does not begin its dependency,
automation, artifact, authentication, or distribution-supply-chain changes.

### Accepted dependency, automation, artifact, and distribution supply-chain contract

`DEC-040` fixes the `MCPD-016` boundary. The checked-in
`.github/supply-chain-controls.json` is the canonical projection,
`scripts/verify-source-artifacts.sh` is the source-tree gate, and
`scripts/verify-supply-chain-controls.sh` is the bounded authenticated
exact-main and exact-release operator audit. This work verifies how source,
automation, and the already published distribution are maintained; it does not
change or republish any immutable `v0.2.0` byte.

| Choice | Accepted policy |
| --- | --- |
| Dependency proposals | Dependabot opens separate grouped weekly version and security proposals for Cargo and GitHub Actions. Rebasing an open proposal is not merging it. Repository auto-merge remains disabled, Dependabot cannot approve pull requests, and no workflow receives merge authority. Every proposal is review input and may be split, rejected, or closed when one member obscures causality or fails policy. |
| Dependency review | Preserve exact direct `=x.y.z` requirements, committed `Cargo.lock`, reviewed crates.io sources, and `--locked` commands. Before merge, record old and new exact identities, release notes and behavior, upstream maintenance and security response, ownership/provenance, manifest and lockfile diff, selected features, transitive and duplicate graph, licenses and sources, advisories, unsafe and build-script changes, minimum Rust and supported-platform impact, cost, and focused/native behavior. Required CI resolves the exact feature graph, runs the direct-requirement regression, runs `cargo-deny`, and retains the three native quality journeys; those automated results do not replace the human upstream review. |
| Selected Actions | GitHub Actions is repository-enabled only for GitHub-owned Actions plus the exact external Homebrew, Anchore, and Rust-project repositories in the canonical allowlist. The live setting requires every checked-in Action selection to use a full 40-character commit SHA. The canonical inventory closes that direct set at seven and records the one nested `actions/attest` implementation selected by the composite provenance Action; an unlisted direct or nested selection, moved tag, unverified selected commit, archived repository, or license-byte drift fails review. Dependabot may propose a new full SHA, but the inventory and upstream review must change in the same pull request before it can pass. GitHub-managed dynamic Dependabot and CodeQL default-setup workflows are separately inventoried provider services, not repository-selected Action refs; the live workflow inventory fails on any additional checked-in or dynamic path. |
| Untrusted code | Only `CI` and `Release preflight` execute a pull request's code. They use only GitHub-hosted ephemeral runners, top-level `contents: read`, non-persisted checkout credentials, no environment, stored secret, secret reference, privileged asset, write permission, `pull_request_target`, `workflow_run`, or untrusted metadata interpolation into shell. Explicit Homebrew and Anchore token inputs are empty. The ephemeral read-only `GITHUB_TOKEN` used by GitHub to fetch a public pull request is not publication or repository-write authority. Fork approval limits compute abuse but is not treated as a security boundary. |
| Standalone CI executable | The former full-SHA `cargo-deny-action` still fetched a mutable release executable without checking a digest, so its commit pin did not authenticate the executed tool. Required CI instead downloads only `cargo-deny` `0.20.2` for `x86_64-unknown-linux-musl` over bounded HTTPS, requires exact SHA-256 `9f12ed4c49936e09b48bf862b595cde2fe64fcbd9d74dfacac6131ca824c8d5f` and the five-entry reviewed archive layout, then executes the reported exact version from a disposable runner path. The upstream release is mutable, so any API digest, asset, tag, layout, version, or selected-tool change blocks for renewed review. |
| Source artifacts | Source control admits reviewable regular UTF-8 text source only. Executable mode is reserved for shebang-bearing `scripts/*.sh`; generated executables, libraries, packages, archives, NUL-bearing or non-UTF-8 content, disallowed ASCII controls, executable/archive/document/binary-media signatures, Git LFS pointers, symlinks, and control-character paths fail. There are no binary exceptions. The negative rehearsal proves a normal reviewable tree passes and generated ELF, NUL-bearing, non-UTF-8, and extension-disguised executable cases fail in a disposable repository. Release and testing-tool bytes stay outside Git history. |
| Published distribution | Authenticate only canonical immutable `v0.2.0`: require the annotated tag object and source commit, immutable non-draft GitHub Release, exact seven-name/size/digest asset set, GitHub release verification, and every asset's signed attestation constrained to the source repository, `release.yml`, tag ref, and source commit. The crates.io package must be unyanked, MIT-declared, and byte-identical to the attested release crate. Current tap `main` must remain the reviewed commit and its formula must be byte-identical to the attested release formula, name the exact crates.io package URL and digest, and declare MIT. Verification downloads to a mode-private disposable root and performs no publish, release, tag, formula, or package write. |
| Live policy and evidence | The operator audit requires exact clean local and remote `main`, selected-Action and SHA-pinning settings, read-only default token, no approval authority, the recorded fork policy, no repository or applicable organization Actions secret, closed upstream Action identities and licenses, the digest-recorded standalone tool, reviewable exact source tree, and authenticated distribution equality. It emits only UTC date, canonical SHA-256, exact source SHA, release tag, and `PASS` or `FAIL`; API bodies and downloaded artifacts are deleted. A failure is investigated privately and blocks completion. |

The 2026-08-12 pre-activation audit found the existing weekly Cargo and Action
version groups, exact direct requirements, locked graph, full-SHA direct Action
uses, read-only workflow default, two secretless pull-request workflows,
read-only fork policy, empty repository Actions-secret inventory, immutable
release attestations, and byte-identical Cargo and Homebrew handoffs already in
place. It also found three gaps that prevent an achieved claim: security updates
were not explicitly grouped, repository policy still allowed any Action and did
not enforce SHA pinning, and the full-SHA `cargo-deny` Action downloaded its
executable without digest verification. No Dependabot proposal exists yet for
this repository, so a real grouped proposal and its review remain required
evidence rather than being replaced by configuration inspection.

The direct and nested checked-in Action review on 2026-08-12 resolved every recorded tag
to its exact canonical commit. All eight repositories were public, active, and
unarchived; every selected commit had GitHub's verified-signature result; and
the exact MIT, BSD-2-Clause, Apache-2.0, or dual MIT/Apache-2.0 license bytes in
the canonical inventory matched. Roles and inputs were narrowed: checkout never
persists credentials, artifacts are named same-run handoffs retained for one
day, Homebrew token inputs are empty, Anchore runs exact Syft `1.50.0` with no
dependency snapshot or implicit upload, provenance delegates only to the
recorded nested SHA, and crates.io authority exists only in protected OIDC
publication jobs. Several Node Actions execute generated JavaScript bundles;
the public source commit, upstream tag, license, focused configuration, and full
pin make changes reviewable but do not independently reproduce or prove those
bundles benign. That limitation remains in the canonical result.
GitHub's separately inventoried dynamic Dependabot and CodeQL workflows do not
take their internal Action refs from this repository; current official GitHub
documentation also states that public-Action restriction policies do not govern
CodeQL default setup. `MCPD-014` continues to verify the latter through its
configured setup, exact-main analyses, and clean alert surface instead of
mislabeling provider-selected tags as project full-SHA pins.

This activation contract contributes the five OSPS `v2026.02.19` Level 1 rows
assigned below, but it is not completion evidence. The implementation must land
through protected `main`; the selected-Action and SHA settings must then be
activated; a real grouped dependency proposal and a fork read/write negative
exercise must be reviewed; exact-main CodeQL and both aggregates must pass; and
the unchanged operator audit must authenticate the immutable release, Cargo,
and Homebrew bytes before a later protected closure may mark `MCPD-016` Done.

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
| [OpenSSF OSPS Baseline `v2026.02.19` Level 1](https://baseline.openssf.org/versions/2026-02-19) | Project-wide M4 gate against the exact activation-locked version under `DEC-034` | Dated and scoped self-assessment; never independent certification or regulatory compliance | Official [BadgeApp](https://www.bestpractices.dev/) `v2026.02.19` baseline-1 public record, JSON, and badge linked to the exact project crosswalk |
| [SLSA `v1.2` Build L2](https://slsa.dev/spec/v1.2/build-track-basics) | Required evaluation of every asset in only the canonical immutable M3 GitHub Release | Artifact-specific result under `v1.2`; never a project-wide, channel-operation, dependency, unlisted-artifact, or future-release claim | Digest-matched signed provenance, constrained public verification, and the exact requirement crosswalk; no certification-like project badge |
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
| DEC-015 | Permit only trusted structural context and typed redacted evidence in ordinary reports | Accepted | 2026-08-09 | Human, JSON, and JUnit output cannot retain arbitrary values; `MCPD-007` introduced `v1alpha1`, and `MCPD-012` preserves the boundary while stabilizing `v1` and projecting JUnit |
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
| DEC-032 | Resolve `OPEN-07` with stable vendor-neutral JSON and a JUnit-compatible CI projection | Accepted | 2026-08-11 | `MCPD-012` promotes the shared redacted result to schema-backed `mcp-doctor.report/v1`, permits only compatible optional-field and new-code additions within `v1`, and renders one bounded JUnit projection without target re-execution; JSON and exit status remain authoritative and portable, while SARIF and general security-scanner positioning remain deferred pending a real security-analysis consumer and threat model |
| DEC-033 | Retain a result-free weighted product and market evaluation method | Accepted | 2026-08-11 | `PROJECT.md` fixes the intended category, excellence posture, eight dimensions totaling 100, rating and evidence rules, safety cap, assessment bands, dynamic procedure, and revalidated seed comparison set without retaining a current score, ranking, or dominance claim |
| DEC-034 | Resolve `OPEN-08` with one activation-locked assurance version set and exact proof routes | Accepted | 2026-08-11 | First-M4 targets are OSPS `v2026.02.19` Level 1, BadgeApp's baseline series for that exact version, and SLSA `v1.2` Build L2; checked-in dated crosswalks, the official public self-assessment record/JSON/badge, and digest-matched constrained provenance verification form the proof, while any pre-publication version or issuer drift blocks for a superseding decision instead of silently floating |
| DEC-035 | Resolve `OPEN-09` with a usable single-maintainer default-branch policy | Accepted | 2026-08-11 | `main` requires a pull request with zero approvals, resolved conversations, strict non-skipping `Required CI` and `Required release preflight` gates, squash-only linear history, and deletion/force-push protection; there is no standing bypass or signing requirement, and emergency administration is one recorded temporary pull-request-only administrator bypass followed by immediate removal, public-projection verification, and non-disclosing authenticated empty-bypass readback |
| DEC-036 | Refine the `DEC-035` verification boundary to match GitHub's live observable fields | Accepted | 2026-08-11 | Credential-free readback verifies `default_branch` plus the configured and effective public rules; a bounded authenticated readback verifies the exact canonical merge settings and empty hidden bypass list while emitting only date, canonical hash, and pass/fail; all selected protection values remain unchanged |
| DEC-037 | Support the latest release line through private coordinated disclosure and every entitled repository-security control | Accepted | 2026-08-11 | `0.2.x` is the sole supported line; GitHub private vulnerability reporting is the security contact; explicit non-SLA response and generally 90-day disclosure targets govern reports; GitHub Free dependency, CodeQL, secret-scanning, push-protection, and private-reporting controls form one non-disclosing clean baseline while paid and unobservable surfaces remain named exclusions and no product-scanner or complete-M4 claim is made |
| DEC-038 | Refine the `MCPD-014` clean baseline to GitHub Free's observable security surfaces | Accepted | 2026-08-11 | Require exact-main successful CodeQL categories for Actions and Rust plus empty repository-visible dependency, code, and value-hidden secret alert responses; name Advanced-Security-only secret-scan history and provider-only alerts as exclusions, because an enabled protection or empty visible-alert response cannot honestly attest an unavailable backfill-completion or provider-routed surface |
| DEC-039 | Use one primary policy repository and one explicitly delegated distribution repository with exact public license evidence | Accepted | 2026-08-12 | Inventory every public organization repository; keep `mcp-doctor` community and defect policy canonical in its source repository; make `homebrew-tap` delegate the in-scope formula surface to it; require reachable HTTPS-only official channels plus exact MIT source, Cargo package, native archive, and formula evidence; retain the immutable SPDX `NOASSERTION` limitation; and fail a bounded credential-free verifier on drift without beginning `MCPD-016` supply-chain work |
| DEC-040 | Close dependency, Action, untrusted-workflow, source-artifact, and published-distribution maintenance under one reviewable supply-chain contract | Accepted | 2026-08-12 | Group Cargo and Action version and security proposals without merge authority; require the complete dependency review and exact graph; allow only inventoried full-SHA Actions; keep pull-request code on secretless read-only hosted jobs; fetch standalone executables only by reviewed digest; reject generated executables and unreviewable binary source artifacts; and authenticate immutable `v0.2.0`, Cargo, and Homebrew equality without changing published bytes |

## Open decisions

`OPEN-04` through `OPEN-06` are accepted as `DEC-028` through `DEC-030`,
`DEC-031` records the implemented `MCPD-011` generation boundary, `OPEN-07` is
accepted as `DEC-032`, and `OPEN-08` and `OPEN-09` are accepted as `DEC-034`
and `DEC-035`. `DEC-033` separately records the dynamic comparative evaluation
method, and `DEC-036` refines only the live-verification boundary discovered
during `MCPD-013`. `DEC-037` records the completed `MCPD-014` disclosure and
security-control contract, and `DEC-038` narrows only its live verification to
the surfaces GitHub actually exposes on the current plan rather than treating
an unavailable history endpoint as a pass. `DEC-039` fixes the active
community, repository, channel, and license boundary, while `DEC-040` fixes the
active dependency, automation, artifact, and distribution supply-chain
boundary. Neither later decision widens the completed security baseline or
constitutes the complete M4 assessment.
Resolution makes an owning future ticket ready once its predecessor passes; it
does not claim that proposed behavior already exists. The remaining entry
belongs to its listed later ticket.

| ID | Decision needed | Needed by | Default if unresolved |
| --- | --- | --- | --- |
| OPEN-10 | Organization membership, application, automation-credential, owner-continuity, and private recovery boundary | `MCPD-017` | Lowest default access, deliberate grants, strong MFA, scoped automation, explicit residual-risk acceptance, and non-disclosing recovery evidence |

## Risk register

| ID | Risk | Impact | Mitigation and escalation trigger | State |
| --- | --- | --- | --- | --- |
| RISK-01 | A diagnostic invokes a mutating tool unexpectedly | Critical | Passive default plus `DEC-029`/`DEC-031` exact configuration, effects, tool, seed/case, and side-effect gates with consent and rejection tests; any implicit, mismatched, wildcard, annotation-derived, or continued call blocks every later release | Mitigated for exact `v0.2.0` reviewed `check`, generated `break`, and remote active paths by local rejection tests, hosted exact-commit gates, and installed release smokes; every future active boundary must reprove the authority contract |
| RISK-02 | A timed-out server or descendant remains running | Critical | Managed process tree, shutdown bounds, termination, reap, and resistant-child fixtures; any surviving PID blocks release | Mitigated for exact `v0.2.0` by the hosted native process matrix and retained reviewed/generated resistant-descendant cleanup journeys; every future process boundary and exact artifact must retain it |
| RISK-03 | Secrets or raw production values reach output | High | Structural redaction and sentinel tests across errors, reports, debug surfaces, fixtures, and the `DEC-028` environment-only secret boundary; any observed name or value blocks release | Mitigated for exact `v0.2.0`: target and argument secret rejection, structural-only reproduction, and human, stable JSON, and JUnit redaction pass local and hosted exact-commit evidence; the risk remains open for every later boundary and artifact |
| RISK-04 | Protocol evolution makes diagnostics incorrect | High | Revision-specific rules and fixtures with explicit unsupported outcomes; a new release triggers contract review | Open |
| RISK-05 | Pathological schema or output exhausts resources | High | Depth, bytes, errors, cases, time, and reference limits; an unbounded input path blocks release | Mitigated for exact `v0.2.0` across passive, reviewed active, synthesis, schema, instance, aggregate-input, case, and report work by local and hosted gates; later boundaries and artifacts require their own evidence |
| RISK-06 | Remote diagnosis enables SSRF or credential leakage | Critical | `DEC-030` fixes exact target gates, IANA-based address classification and pinning, verified TLS, credential-to-endpoint consent, direct zero-redirect/retry connections, finite headers/bodies, and value-free reports; any bypass, peer drift, implicit network source, or secret output blocks completion | Mitigated for exact `v0.2.0` bounded passive, reviewed, and generated activity through the retained HTTP transport and exact-authority network journeys in hosted native evidence; every future multi-origin boundary must reprove it |
| RISK-07 | Generated cases are irreproducible or exceed authorized scope | High | Versioned stable seed selection, ordered generation, structural evidence, exact tool/effect/target gates, and finite cases, candidates, inputs, work, and concurrency; mismatch blocks active testing | Mitigated for exact `v0.2.0` by fixed-seed fixtures, local and HTTP authorization rejection, structural redaction, every generation limit, sequential execution, and hosted exact-commit verification; future generator changes reopen it |
| RISK-08 | A passing report creates false confidence after skipped checks | High | Per-check performed/skipped state and non-ambiguous summary; any hidden skip blocks release | Mitigated for exact `v0.2.0` by hosted human, schema-valid stable JSON, and JUnit causal-skip and authorization journeys; every future reporter or check-state change must preserve the invariant |
| RISK-09 | Broad protocol, transport, and reporting scope delays a usable slice | High | M1 ends at passive `inspect`, M2 publishes it, and M3 stays an ordered set of bounded vertical tickets; any broad feature becoming a prerequisite for an earlier completed slice escalates | Mitigated by the ordered plan and `DEC-027`; voluntary evidence may reprioritize work, but its absence neither authorizes breadth nor blocks scoped work |
| RISK-10 | The public identity is unavailable, ambiguous, or confused with an existing command before publication | High | `DEC-008` retains the product and executable under EnjoyableWork, accepts the cross-ecosystem collision, defines a Cargo-package fallback, and requires exact official-channel guidance plus an immediate pre-publication registry recheck | Mitigated for the first release: the preferred `mcp-doctor` crate identity is published under the exact EnjoyableWork source and metadata; future channel guidance must preserve the distinction |
| RISK-11 | A release channel installs bytes not represented by the immutable release | Critical | `MCPD-008` proves exact package/formula equality, checksums, attestations, and native installed smokes for the first release; `MCPD-008A` makes those checks preconditions for every later downstream write; any mismatch requires a new version | Mitigated for `v0.1.0` and `v0.2.0` by byte-identical Cargo and Homebrew handoffs, rejected mismatch cases, authenticated assets, and successful native channel verification; every future release must retain the same immutable-byte gates |
| RISK-12 | An unprotected default branch permits direct, destructive, or insufficiently reviewed changes | High | `DEC-035` fixes the zero-approval PR, strict aggregate-check, squash-only, no-standing-bypass, deletion/force-push, public-projection drift verification, authenticated hidden-state readback, and bounded emergency contract; any unverified bypass or destructive path blocks M4 | Mitigated for the 2026-08-11 `MCPD-013` scope by the active public ruleset, canonical merge settings, normal protected merge, rejected direct/deletion/non-fast-forward paths, closed emergency exercise, post-removal gates, credential-free projection pass, and non-disclosing empty-bypass pass. An administrator can still change repository policy; ruleset, required-context, merge-setting, administrator-boundary, or GitHub-capability drift reopens the risk and requires both verifiers and, where applicable, a new exercise |
| RISK-13 | A contributor publicly exposes a vulnerability, credential, or unsafe diagnostic because reporting and prevention controls are incomplete | High | `MCPD-014` verifies private reporting, safe guidance, entitled scanning and prevention controls, limitations, and a non-disclosing baseline; any public sensitive report or hidden finding blocks M4 | Mitigated for the scoped 2026-08-11 `MCPD-014` surfaces by the recognized policy and private route, enabled entitled dependency and secret-prevention controls, representative and exact-`main` CodeQL, zero open repository-visible alerts, and non-disclosing pass. A public sensitive report, hidden finding, supported-line or policy drift, disabled or changed control, failed exact-`main` analysis, entitlement change, newly observable surface, or stale baseline reopens the risk; scan-history, provider-only, paid-feature, product-scanner, and complete-M4 evidence remain explicitly outside this result |
| RISK-14 | Mutable automation, privileged untrusted code, or unauthenticated distribution compromises the project or its releases | Critical | `MCPD-008A` limits repeat publication to reviewed full-SHA automation, OIDC or narrowly scoped short-lived authority, immutable-byte preconditions, and negative authorization tests; `MCPD-016` audits the complete CI and distribution boundary; any drift or credential exposure blocks publication and M4 | The first release removed and revoked its one-time credential, and `v0.2.0` exercised the exact OIDC and tap authorities with authenticated immutable-byte handoffs and a clean credential inventory. `MCPD-016` is now In progress with a closed full-SHA Action inventory, read-only secretless fork policy, authenticated exact-distribution verifier, and negative source-artifact gate; live selected-Action activation, grouped-proposal/fork exercises, and exact-main evidence remain required before mitigation expands to the complete ticket scope. |
| RISK-15 | Organization-owner loss or over-broad long-lived credentials become an undocumented recovery dependency | High | `MCPD-017` verifies strong MFA, lowest access, application and credential scope, owner continuity, and private recovery evidence; unresolved access or recovery assumptions block M4 | Deferred with M4 |
| RISK-16 | A stale, unofficial, or over-broad assurance claim misleads adopters | High | `DEC-034` locks exact version and proof routes and makes drift a stop-and-decide gate; `MCPD-018` binds every claim to exact scope, date, official proof, public evidence, and removal triggers; missing, stale, withdrawn, or ambiguous proof blocks or removes the claim | Policy resolved; proof remains deferred with M4 |
| RISK-17 | Technically correct findings become an undifferentiated failure list that does not help a developer repair a server or earn repeat use | High | Every MVP failure identifies the expected earliest actionable layer, preserves independent safety failures, links downstream skips to their cause, and includes safe what, where, why, expectation, remediation, and versioned-rule evidence; report-only cases, maintainer trials, and voluntary feedback record unclear findings, false findings, time to value, and repeat use | M1 report sufficiency passes locally and hosted; the checkpoint closed with zero independent reports and no adoption claim, while future feedback may reprioritize later product work |
| RISK-18 | Latest-only protocol support excludes too much of the reachable ecosystem for a useful first release | High | `DEC-024` requires a controlled official/independent matrix spanning at least two languages: complete selected current-revision success permits broad positioning, narrower credible reach requires readiness/migration positioning and a separate compatibility ticket, and no credible independent pass blocks completion without silently adding legacy behavior | Four selected current-revision servers across four languages passed locally and hosted before M2 release; future protocol revisions reopen the risk |
| RISK-19 | An unnecessary, stale, compromised, or silently widened dependency executes in the product, developer environment, or CI supply chain | Critical | Default to no addition; require an owning need and dated maintenance/provenance/security/graph review; use exact direct requirements, a committed lockfile, narrow features, reviewed sources, `cargo-deny`, non-automatic update approval, and a regression check; removal, unexplained upstream inactivity, ownership change, advisory, new build script/unsafe surface, or unreviewable lockfile growth triggers escalation | Mitigated through M3 by the MCPD-003 policy, exact locked graph, hosted dependency checks, and dated focused `quick-xml` test-only adoption review. `MCPD-016` now adds explicit grouped version/security proposals, a complete review record, exact feature-graph gate, closed Action/tool inventory, and digest-verified `cargo-deny` delivery; a real proposal review and exact-main hosted/operator evidence remain open. |
| RISK-20 | Users cannot find a real project route or receive incompatible license terms because repository, community, channel, or artifact scope drifts | High | `DEC-039` inventories every public organization repository, centralizes reachable community and defect routes, explicitly delegates the tap, and verifies HTTPS official channels plus exact source, package, archive, and formula license evidence without credentials; any new unclassified repository, unavailable route, stale policy, license mismatch, or unexplained asset blocks M4 | Mitigated for the dated 2026-08-12 `MCPD-015` scope by both repository changes, recognized community state, exact-main hosted gates, and the credential-free five-repository and exact-release pass. Any repository inventory, route, channel, license, release-set, package, formula, or GitHub/crates.io surface drift reopens the risk; the immutable SPDX limitation and later supply-chain and complete-assurance work remain explicit |

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

### Expanded M3 release complete

M3 is complete only when:

- every retained passive, reviewed active, generated, STDIO, and Streamable
  HTTP journey passes through the built `v0.2.0` release without widening
  target, tool, side-effect, secret, or network authority;
- human, schema-valid `mcp-doctor.report/v1` JSON, and bounded JUnit output
  derive from one result and agree on primary diagnosis, independent safety
  findings, causal skips, overall outcome, and exit semantics;
- the selected JUnit subset remains deterministic, correctly escaped, and
  accepted by the pinned representative Jenkins and GitLab consumers, while
  stable JSON plus process exit remains the vendor-neutral fallback;
- the complete locked local gate and hosted native CI and release preflight
  pass on the exact release commit;
- the intentionally tagged release is immutable and every asset is
  checksummed and authenticated before Cargo or Homebrew publication; and
- Cargo, Homebrew, and represented GNU/Linux archives match the canonical
  GitHub bytes and pass their applicable installed passive smoke on every
  represented host.

No SARIF, general security-scanner claim, older-revision compatibility,
unsigned macOS or Windows native binary, or deferred/cancelled M3 feature is an
implicit prerequisite. Exact publication and installed-channel evidence must
be linked from `MCPD-012` and D-08 before either is marked Done.

### M4 enterprise assurance

M4 is complete only when:

- `MCPD-013` through `MCPD-018` are done in order and D-09 links their durable
  public and non-disclosing verification evidence;
- every Level 1 control in OSPS `v2026.02.19` as locked by `DEC-034`, or an
  explicitly accepted superseding decision, passes with evidence or exact
  applicability reasoning for every row;
- the public self-assessment states the framework version, level, assessed
  repositories and organization and release boundaries, assessment date,
  limitations, self-assessed status, evidence links, and review triggers;
- the official BadgeApp assessment displays `v2026.02.19`, has achieved
  baseline-1 through supported `Met` or justified N/A answers, and its public
  page, JSON, and official badge link from the README are verified on exact
  `main`;
- every asset in the canonical immutable M3 GitHub Release meets the SLSA
  `v1.2` Build L2 requirements and its exact digest, provenance predicate,
  builder, signer workflow, tag, and source commit are publicly reproducibly
  verified, while registry and Homebrew operations, dependencies, unlisted
  artifacts, and future releases remain explicitly outside that claim;
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
