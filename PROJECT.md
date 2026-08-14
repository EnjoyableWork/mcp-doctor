# mcp-doctor project plan

This is the living source for product scope, delivery status, ordered work,
decisions, risks, and release gates.

| Control | Current state |
| --- | --- |
| Document state | Active |
| Product state | The passive local STDIO MVP, pinned current-revision compatibility matrix, bounded local and Streamable HTTP `check`, deterministic `break`, passive legacy inspection, current-revision contract snapshots and diffs, one-run JSON/JUnit report artifacts, conservative offline aggregation, structured unsupported-version diagnosis, and stable schema-backed JSON and JUnit-compatible projections pass local, protected hosted, immutable-release, and represented installed-channel evidence in `v0.3.0`; optional compiled-only capability discovery, explicit passive legacy-revision snapshot/diff support, exact-selected MCP `2025-11-25` and `2025-06-18` `check` and `break`, and current-revision schema-invalid argument rejection pass their scoped local, clean package/install, protected exact-head, and exact-`main` source evidence without a new release or broad compatibility claim |
| Current milestone | M4 — enterprise assurance and adoption; `MCPD-016A` is Done and `MCPD-017` is In progress |
| Overall status | M0 through M3 are Done; immutable `v0.1.0`, `v0.2.0`, and `v0.3.0` channels are verified; protected `main` retains the `MCPD-013` controls; the scoped `MCPD-014` repository-security baseline and revalidated three-repository `MCPD-015` public community, channel, and license contract pass with their explicit limitations; dated `MCPD-016` evidence and the completed `MCPD-016A` correction prove the current supply-chain boundary, including direct digest-authenticated Syft acquisition, the narrowed live Action allowlist, and first-attempt exact-head and exact-`main` gates; `DEC-041` and `DEC-042` fix the organization-access policy; owner-authorized `MCPD-017` activation has applied the supported member, App, OAuth, PAT, short-lived verifier, secure-2FA, credential-inventory, deploy-key, and private-recovery controls, with closure still withheld until the protected change merges and the non-disclosing verifier passes on exact `main`; optional `MCPD-021` through `MCPD-030` are Done for resolved issues #72, #73, #64, #66, #74, #60, #61, #75, and #41 plus the independently verified `v0.3.0` release without changing the M4 gate; optional `MCPD-032` is In progress for issue #65 |
| Current focus | Complete the `MCPD-017` exact-`main` closure independently; completed optional `MCPD-019` through `MCPD-030` and in-progress `MCPD-032` remain outside the M4 dependency chain and cannot delay, redefine, or become prerequisites for `MCPD-017` or `MCPD-018`; proposed `MCPD-031` is a prepublication gate for the next release only |
| Public release | `mcp-doctor` `v0.3.0` — immutable GitHub Release, crates.io, `EnjoyableWork/tap/mcp-doctor`, and all ten represented installed-channel jobs verified |
| Last reviewed | 2026-08-14 |
| Next review trigger | Any organization authentication, member, owner, billing-manager, invitation, base-permission, member-privilege, repository-creation, application, OAuth, personal-access-token, organization credential, in-scope repository credential, deploy-key, recovery, public-repository, community-route, official-channel, source/package/archive/formula license, security-policy, supported-line, entitlement, scan result, ruleset, merge-setting, administrator-boundary, required-context, GitHub-capability, Action, workflow, dependency proposal, report destination, projection fan-out, aggregate schema or outcome policy, capability-manifest schema, command/transport/revision matrix, exit/profile identifier, tracked artifact, voluntary-usage, trusted-publisher, tap-authority, release-pipeline, testing-tool, safety-boundary, or assurance-evidence change |

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
| M4 — Enterprise assurance and adoption | `MCPD-013` → `MCPD-014` → `MCPD-015` → `MCPD-016` → `MCPD-016A` → `MCPD-017` → `MCPD-018` | Contributor-compatible governance, repository and organization controls, supply-chain evidence, and a public scoped assurance baseline |

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

`MCPD-019` is an optional compatibility ticket running alongside M4. It does
not change the M4 order, release boundary, or immutable M3 artifacts. Its
legacy behavior is available only through an explicit passive `inspect`
selection and cannot become a prerequisite for `MCPD-017` or `MCPD-018`.

`MCPD-020` is an optional developer-workflow ticket running alongside M4. It
does not change the M4 order, release boundary, ordinary diagnostic reports, or
immutable M3 artifacts. Contract snapshots are an explicitly acknowledged
sensitive artifact, and their offline comparison cannot become a prerequisite
for `MCPD-017` or `MCPD-018`.

`MCPD-021` is a completed optional reporting-workflow ticket that ran alongside
M4. It depends on the completed stable reporter boundary in `MCPD-012`,
preserves the existing stdout and diagnostic contracts, and projects one
immutable redacted result into explicit JSON and JUnit files without repeating
target activity. It did not delay, redefine, or become a prerequisite for
`MCPD-017` or `MCPD-018`.

`MCPD-022` is a completed optional offline-evidence ticket that ran alongside
M4. It depends on the completed stable report and multi-artifact boundaries in
`MCPD-012` and `MCPD-021`, combines only explicit existing redacted reports,
and adds no target, transport, credential, discovery, or tool activity. It did
not delay, redefine, or become a prerequisite for `MCPD-017` or `MCPD-018`.

`MCPD-023` was a focused optional correctness ticket that ran alongside M4. It
resolved GitHub issue #64 by recognizing only the current revision's exact,
bounded structured unsupported-version response and assigning it to the
protocol layer. It adds no fallback, retry, broader compatibility, or new
dependency and did not become a prerequisite for `MCPD-017` or `MCPD-018`.

`MCPD-024` was the intentional `v0.3.0` release track for completed optional
capabilities and the `MCPD-023` correction. It reuses the accepted protected,
immutable, OIDC, exact-byte, and independently verified release path. It does
not publish or depend on an M4 assurance claim and did not redefine the
`MCPD-017` or `MCPD-018` completion gates.

`MCPD-025` was an optional integration-discovery ticket that ran alongside M4.
It resolved issue #66 with a compiled-only command and stable manifest so an
editor, wrapper, CI job, or server repository can decide whether a planned
diagnostic is supported before any target activity. It depends on the stable
reporter boundary in `MCPD-012` and inventories only already implemented
command contracts. It adds no target, configuration, credential, process,
network, retrieval, or tool authority and did not delay, redefine, or become a
prerequisite for `MCPD-017` or `MCPD-018`.

`MCPD-026` is completed optional work for resolved GitHub issue #74 under
`DEC-051`; protected [PR 63](https://github.com/EnjoyableWork/mcp-doctor/pull/63),
merge commit [`6e0f0ac`](https://github.com/EnjoyableWork/mcp-doctor/commit/6e0f0acf096f797a12f3bf8826d8d11963007039),
and first-attempt exact-`main` evidence close its isolated legacy artifact
scope. `MCPD-027` is completed optional work for resolved GitHub issue #60
under `DEC-052`; protected [PR 78](https://github.com/EnjoyableWork/mcp-doctor/pull/78),
merge commit [`ac3d9ac`](https://github.com/EnjoyableWork/mcp-doctor/commit/ac3d9ac1c289b3329eadbe8fb1a35cca597386c4),
and first-attempt exact-`main` evidence close its active MCP `2025-11-25`
source scope. `MCPD-028` is completed optional work for resolved GitHub issue
#61 under `DEC-052`; protected [PR 80](https://github.com/EnjoyableWork/mcp-doctor/pull/80),
merge commit [`e380b26`](https://github.com/EnjoyableWork/mcp-doctor/commit/e380b26c382ea2b83fefe41c153f00baea023db2),
and first-attempt exact-`main` evidence close its exact-dialect MCP `2025-06-18`
source scope. `MCPD-029` is completed optional work for resolved issue #75
under `DEC-053`; protected [PR 82](https://github.com/EnjoyableWork/mcp-doctor/pull/82),
merge commit [`3472952`](https://github.com/EnjoyableWork/mcp-doctor/commit/3472952a521ad30fbf716c828739887835a78898),
and first-attempt exact-`main` evidence close its bounded current-revision
rejection-diagnostic scope. `MCPD-030` is completed optional deterministic-CI
policy work for resolved issue #41 under `DEC-054`; protected
[PR 83](https://github.com/EnjoyableWork/mcp-doctor/pull/83), merge commit
[`dbc19bd`](https://github.com/EnjoyableWork/mcp-doctor/commit/dbc19bd7a863c8e53651a78bd4570616a59d5e02),
and first-attempt exact-`main` evidence close its test and automation scope
without changing a product limit, runtime authority, release artifact, live
setting, or M4 gate. None is part of M4, and none may
delay, redefine, or become a prerequisite for `MCPD-017` or `MCPD-018`.
The audit assigns legacy release-only fixed polling, broad curl retry, and
source-checkout runner verification to proposed `MCPD-031`; it is not current
delivery work, but it must be resolved before another public version uses those
paths as release evidence.

`MCPD-032` is optional runtime ergonomics work for GitHub issue #65 under
`DEC-055`. It adds one invocation-local choice between the existing `default`
diagnostic limits and a finite `slow-start` profile for `inspect`, `check`, and
`break`, without adding numeric overrides, configuration, retry, target
authority, or release claims. It cannot delay, redefine, or become a
prerequisite for `MCPD-017` or `MCPD-018`.

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
| `MCP-TRANSPORT-004` | Error | The server exited before returning the pending diagnostic response |
| `MCP-PROTOCOL-001` | Info | The requested protocol revision is supported |
| `MCP-PROTOCOL-002` | Error | The server does not support the required revision |
| `MCP-PROTOCOL-003` | Error | The revision value is missing or has the wrong JSON type |
| `MCP-PROTOCOL-004` | Warning | A feature is deprecated by the selected revision |
| `MCP-LIMIT-001` | Error | A configured diagnostic safety limit is exceeded |
| `MCP-SAFETY-001` | Critical | A managed target cannot be fully cleaned up |
| `MCP-CATALOG-001` | Error | An advertised catalog response or item violates the selected revision's structural contract |
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
| `MCP-ACTIVE-006` | Error | A tool response violates the selected revision's result envelope and stops later calls |
| `MCP-ACTIVE-007` | Error | The selected tool requires task augmentation, so the immediate active path stops before `tools/call` |

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

Generation begins only after the selected revision's discovery or initialize
lifecycle, unique tool selection, and bounded local Draft 2020-12 input-schema
contract pass. It uses only that selected schema and the existing validator. It
does not accept
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

### MCPD-020 accepted contract-snapshot and offline-diff plan

GitHub issue 70 owns one optional current-revision developer artifact. The
artifact helps a server maintainer compare advertised contracts between two
builds without turning an ordinary diagnostic report into a value-bearing
inventory, rerunning either target during comparison, or claiming universal
compatibility. This ticket depends on the stable passive `MCPD-012` boundary
and runs alongside, but never delays, the active M4 story.

#### Commands and execution authority

- Snapshot creation is available only on passive current-revision `inspect`
  through both `--snapshot PATH` and an exact matching
  `--allow-sensitive-snapshot PATH`. A missing or non-identical acknowledgement,
  an explicit legacy `--protocol-version`, or an already existing output path
  fails before a process starts or a network target is prepared.
- The first slice has no overwrite option. It creates one new regular file with
  exclusive `create_new` semantics, owner-only Unix mode where supported, a
  complete bounded write and sync, and removal of only the newly created
  partial file if writing fails. Its parent must already exist.
- A snapshot is assembled in memory from the exact discovery and paginated list
  responses that produced the ordinary report. It causes no second discovery,
  list request, target run, or connection. It is persisted after the exact
  current revision is confirmed, the bounded catalog conversation completes,
  and cleanup succeeds, including when a bounded local schema shape retained by
  the artifact is the reason the ordinary report fails. Transport, protocol,
  external-reference, resource-bound, unrepresentable or incomplete-catalog,
  and cleanup failures prevent creation.
- `mcp-doctor diff [--format human|json] BEFORE AFTER` reads exactly two local
  regular files. It accepts no target, endpoint, credential, transport, schema
  retrieval, or active-tool option and performs no process, network, tool,
  prompt, resource, or schema-retrieval activity.

#### Artifact and sensitivity boundary

The checked-in `mcp-doctor.contract-snapshot/v1alpha1` schema is independent
from `mcp-doctor.report/v1`. It contains only MCP `2026-07-28`, normalized known
capabilities, tool and prompt names, resource names and advertised resource
URIs, resource-template names and URI templates, prompt argument names and
required state, the four protocol-defined tool behavior hints with omitted
values expanded to their protocol defaults, and local-reference-only input and
output JSON Schema structure. Advertised resource identifiers are contract
data; the transport endpoint is never retained.

Descriptions, titles, icons, instructions, list cursors, TTL and cache fields,
server identity, runtime results, request arguments, headers, transport URLs,
DNS and peer data, credentials and their source names, environment data, logs,
stderr, challenges, and unrelated extension content are excluded. Recursive
schema annotations `title`, `description`, `default`, `examples`, and `$comment`
are also excluded because they do not change validation; there is no stronger
value-retaining mode in this slice. Validation-bearing property names and
`const`, `enum`, pattern, format, local-reference, and numeric or size
constraint values remain because removing them would erase the advertised
contract. Documentation and
CLI help must therefore warn that snapshots can expose proprietary names,
resource identifiers, schema fields, and allowed values and require deliberate
storage, retention, and sharing.

Objects are key-sorted; catalogs are identity-sorted by tool or prompt name,
resource URI, and resource-template URI template; and set-like `required`,
`type`, `enum`, `allOf`, `anyOf`, and `oneOf` arrays are deterministically
sorted without reordering sequence-bearing schema arrays. Omitted prompt
`required`, capability settings, and tool behavior hints are expanded to their
specified defaults. No hash, digest, correlation token, health score, or
conformance percentage is introduced.

Each catalog stores a canonical contract array and a separate artifact-local
numeric mapping from the exact zero-based discovery ordinal to that contract's
canonical index. Every ordinal and contract index must occur exactly once and
remain in range. The mapping lets a maintainer resolve an ordinary location
such as `tools[42]` inside the deliberately sensitive snapshot from the same
conversation; ordinary human, JSON, and JUnit reports remain identifier-free.
Diff validates each mapping independently, never joins snapshots by ordinal,
and ignores a valid mapping during semantic comparison, so catalog reordering
updates correlation without manufacturing a contract change. Duplicate,
out-of-range, malformed, or foreign mapping references fail structurally
without echoing an identifier.

#### Offline diff and compatibility rules

Human output and schema-backed `mcp-doctor.contract-diff/v1alpha1` JSON consume
one typed result with `artifact_validation`, `normalization`,
`catalog_comparison`, and `schema_comparison` performed/skipped states. Invalid
input causally skips dependent comparison. Findings contain a stable code,
catalog kind, safe before/after ordinal where applicable, change kind, and one
of `compatible`, `potentially_breaking`, or `review_required`; they do not
reproduce names, URIs, schema keys or values, paths supplied by the user, or
snapshot content. Unchanged or documented-compatible results exit zero,
potentially breaking or review-required results exit one, and invocation,
input, limit, or I/O failures exit two.

| Code | Change | Classification |
| --- | --- | --- |
| `MCP-DIFF-001` | Tool, prompt, resource, or resource template added | Compatible |
| `MCP-DIFF-002` | Tool, prompt, resource, or resource template removed | Potentially breaking |
| `MCP-DIFF-003` | Known capability enabled | Compatible |
| `MCP-DIFF-004` | Known capability disabled | Potentially breaking |
| `MCP-DIFF-005` | Required prompt or tool input added | Potentially breaking |
| `MCP-DIFF-006` | Required prompt or tool input removed | Compatible |
| `MCP-DIFF-007` | Recognized input constraint narrowed | Potentially breaking |
| `MCP-DIFF-008` | Recognized input constraint widened | Compatible |
| `MCP-DIFF-009` | Other schema, identity-adjacent, or catalog structure changed | Review required |
| `MCP-DIFF-010` | Protocol-defined behavior hint changed | Review required |

Recognized narrowing and widening are deliberately syntactic and local: set
subset/superset changes to `type`, `enum`, and `required`; adding or removing
`const`; monotonic changes to the same minimum, maximum, length, item, and
property-count keyword; and boolean `additionalProperties` restriction or
relaxation. Tool-input changes use caller-acceptance direction. Output-schema,
combinator, conditional, reference-target, pattern, format, multiple-of,
mixed-keyword, and otherwise non-monotonic changes are `review_required`; the
diff never attempts general JSON Schema implication. Adding/removing a required
prompt argument follows the same stable required-input codes. Tool-hint changes
never grant execution authority and remain review-required.

Artifact validation uses value-free codes `MCP-SNAPSHOT-001` through
`MCP-SNAPSHOT-006` for malformed structure, unsupported artifact version,
unsupported protocol revision, exhausted bounds, prohibited external
reference, and invalid correlation respectively. Differing snapshot schema
versions or protocol revisions fail before comparison rather than being
classified as a contract change.

#### Finite implementation and acceptance evidence

Snapshot creation reuses the existing eight-MiB aggregate conversation and
10,000-item catalog bounds, one-MiB-per-schema, 100,000-node, depth-64, and
local-reference-depth-32 limits. Each diff input is a regular file of at most
eight MiB; parsing and normalization retain the same item, schema, node, depth,
and reference bounds; comparison stops at 256 findings; and either reporter is
limited to four MiB. External references remain prohibited during creation and
offline validation. The implementation reuses `serde`, `serde_json`, and the
standard library and adds no dependency.

Acceptance requires checked-in snapshot and diff schemas plus deterministic
goldens; equivalent reordered and representation-normalized snapshots with no
semantic diff; catalog, capability, prompt-required, narrowed/widened schema,
hint, and unclassified-structure cases; malformed, oversized, too-deep,
external-reference, cross-version, invalid-correlation, and existing-output
negatives; a 100-tool ordinal case whose locally invalid-schema report resolves
only through the same-run artifact; and sentinels proving all excluded data stays out
of snapshots, ordinary reports, diff output, and failures. Built-binary STDIO
and disposable HTTP journeys must prove exact opt-in before activity, one
bounded conversation, no overwrite, and an offline diff with no target or
network. POSIX and PowerShell installed smokes round-trip a snapshot and
equivalent diff on every represented source/archive/Cargo/Homebrew path. The
complete locked local gate, package contents, and hosted required checks must
pass before `MCPD-020` is Done; publication is not part of this ticket.

### MCPD-026 accepted legacy contract-snapshot and same-revision diff plan

[GitHub issue 74](https://github.com/EnjoyableWork/mcp-doctor/issues/74)
owns one optional extension of the completed `MCPD-019` passive legacy adapter
and `MCPD-020` sensitive artifact workflow. The explicitly requested isolated
worktree and goal establish ownership and work-in-progress fit. This work remains
outside the M4 dependency chain and cannot delay, redefine, or become a
prerequisite for `MCPD-017` or `MCPD-018`.

#### Artifact version, identity, and retained fields

- Extend `mcp-doctor.contract-snapshot/v1alpha1` rather than introduce a second
  artifact. Existing MCP `2026-07-28` artifacts remain valid and new current-
  revision bytes remain unchanged. Existing readers reject a new legacy
  artifact through its already-required revision field instead of coercing it;
  new readers accept the old current-revision shape.
- `protocol_revision` remains the exact selected revision. A legacy artifact
  additionally requires `negotiated_protocol_revision`, which must equal the
  selected MCP `2025-11-25` or `2025-06-18` revision. Current-revision artifacts
  omit negotiated identity because their per-request binding has no initialize
  negotiation. Missing, extra, unsupported, or mismatched identity is invalid.
- Preserve the `DEC-045` retained and excluded content boundary. In addition to
  the existing catalog-related capabilities, retain only fixed booleans for the
  revision-defined legacy `logging` and `completions` capability presence and,
  for MCP `2025-11-25` only, `tasks` presence plus its `list`, `cancel`, and
  `requests.tools.call` presence. Experimental capability names and contents,
  server identity and instructions, task contents, cursors, headers, session
  identifiers, transport and credential data, logs, and unrelated extensions
  remain excluded.
- Each legacy tool schema carries a fixed `draft_2020_12`, `ambiguous`, or
  `unsupported` artifact-local dialect state for its input schema and optional
  output schema. The field never copies an advertised URI. MCP `2025-11-25`
  maps an omitted declaration to `draft_2020_12`; MCP `2025-06-18` maps an
  omitted declaration to `ambiguous`; an exact Draft 2020-12 declaration maps
  to `draft_2020_12`; and another string maps to `unsupported`. The normalized
  sensitive schema still retains its validation-bearing local structure under
  the existing bounds. A stored state inconsistent with the revision and schema
  shape is rejected rather than repaired.

#### Capture and persistence boundary

- Snapshot preparation keeps the identical exact-path acknowledgement,
  existing-parent, destination-alias, new-file-only, owner-only Unix mode,
  bounded write, sync, cleanup, and no-overwrite behavior for all three passive
  revisions. Revision selection remains an exact CLI input and never becomes
  discovery, fallback, retry, or downgrade authority.
- Creation consumes the selected revision, the bounded conversation's
  negotiated identity, and the exact completed response vector that produced
  the ordinary report. For legacy inspection, the initialize result supplies
  capabilities and every following response must correspond to the already
  capability-gated paginated list sequence. Creation sends no additional
  initialize, initialized notification, list, task, prompt, resource, tool, or
  transport request and persists only after conversation completion and cleanup
  succeed.
- Exact selected/negotiated mismatch, incomplete or extra responses, malformed
  revision-specific capabilities, external references, exhausted bounds, and
  cleanup failure write no artifact. As in `DEC-045`, a complete bounded local
  schema shape retained by the sensitive artifact may still be captured when
  that schema is the reason the ordinary passive report is unsuccessful.

#### Offline comparison and dialect behavior

- `diff` independently reads, bounds, parses, normalizes, and validates both
  local regular files. It compares only artifacts with the same supported
  selected revision and matching revision-specific identity/field contract.
  Artifact-local ordinal maps are validated independently and never joined
  across snapshots; semantic identity remains catalog-local and revision-local.
- Add value-free `MCP-SNAPSHOT-007` for selected, negotiated, or cross-artifact
  revision-identity mismatch and `MCP-SNAPSHOT-008` for an incompatible
  revision-specific artifact contract. Both fail artifact validation, causally
  skip normalization and catalog/schema comparison, exit `2`, and reproduce no
  path, identifier, revision value, dialect URI, or schema content. An invalid
  cross-revision diff reports no chosen protocol revision rather than selecting
  or inferring one side.
- Existing capability, catalog, prompt-input, tool-hint, and schema codes retain
  their `DEC-045` meaning within one revision. Syntactic Draft 2020-12 narrowing
  and widening applies to explicit Draft 2020-12 schemas and to omitted MCP
  `2025-11-25` schemas. Any changed MCP `2025-06-18` schema whose dialect is
  ambiguous, or any changed unsupported-dialect schema, is `review_required`;
  the diff never guesses a dialect or general implication. Identical ambiguous
  schemas remain unchanged evidence.

#### Completion evidence

Acceptance requires both legacy revisions over synthetic STDIO and Streamable
HTTP, selected/negotiated identity, all retained revision-specific capabilities,
same-conversation request counters, pagination and cleanup, deterministic
same-revision equality and documented changes, cross-revision and incompatible-
artifact rejection, both omitted-dialect cases, external-reference and every
retained finite bound, no-clobber and cleanup failure, and transport, credential,
session, server, runtime, and log secret-exclusion sentinels. Existing current-
revision snapshot and diff goldens must remain byte-identical. The CLI help,
README matrix and sensitivity guidance, schemas, compiled capability manifest,
POSIX and PowerShell installed smokes, source package and packaged install,
complete locked local gates, protected exact-head hosted gates, protected merge,
issue closure, and exact-`main` durable evidence must pass without a dependency,
release, active legacy, broad compatibility, score, or assurance claim.

Pre-merge local evidence on 2026-08-14 covers both revisions over the synthetic
STDIO and credentialed Streamable HTTP fixtures, exact request/session cleanup
counters, revision-specific capability retention, omitted/explicit/unsupported
dialect behavior, same-revision reordered and changed diffs, value-free cross-
revision and incompatible-artifact rejection, external-reference and finite-
bound failures, no-clobber, cleanup failure, and excluded-data sentinels. The
new current-revision capture matches the checked-in 4,345-byte golden and a
disposable build of the pre-change commit byte for byte. `scripts/check.sh`,
`cargo deny --all-features --locked check`, source-artifact review, two matching
clean `cargo package --locked` runs with SHA-256
`e5a35d17a1f2a82eb9df487ecc8ce38d2d6b69f21318e84b995c993d05a6dfaf`, and an
exact-commit install-from-package POSIX smoke pass without a dependency change.
Protected [PR 63](https://github.com/EnjoyableWork/mcp-doctor/pull/63) exact-head
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31769989978),
[CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31769987698),
and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31769989971)
pass on the first attempt, including native PowerShell execution and all
represented source-install, archive, and SBOM gates. The protected squash
[merge](https://github.com/EnjoyableWork/mcp-doctor/commit/6e0f0acf096f797a12f3bf8826d8d11963007039)
landed the implementation. Replacement [issue #74](https://github.com/EnjoyableWork/mcp-doctor/issues/74)
is closed as completed, and first-attempt exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31771389361),
[CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31771389015),
and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31771389387)
complete the durable evidence without a dependency, release, active-legacy,
broad-compatibility, score, or assurance claim.

### MCPD-021 accepted multi-report artifact plan

[GitHub issue #72](https://github.com/EnjoyableWork/mcp-doctor/issues/72)
owns one optional reporting workflow. CI consumers need stable JSON for
automation and JUnit for native test rendering, but running a target twice is
not equivalent evidence and can repeat authorized tool calls, observe changed
state, consume more process or network work, or produce a different diagnosis.
This ticket therefore fans out one immutable redacted `DiagnosticReport`; it
does not add a diagnostic, reporter, target suite, CI-provider integration,
report merge, or upload feature.

#### CLI and result boundary

- Existing `inspect`, `check`, and `break` invocations retain `--format
  human|json|junit` as the byte-compatible stdout selector. Each additionally
  accepts at most one `--json-report PATH` and one `--junit-report PATH`; no
  destination is inferred from a repository, workspace, environment, or CI
  provider. `diff` and contract snapshots remain separate artifact contracts.
- Every requested report is rendered only after the command has constructed
  its one typed diagnostic result. The target starts, initializes, discovers,
  connects, and executes each authorized case exactly as often as the
  equivalent stdout-only invocation. Reporter selection grants no execution,
  network, credential, discovery, or tool authority.
- Passing, failed, and incomplete diagnostics write every requested artifact
  when all rendering and persistence succeeds. The diagnostic exit remains the
  process exit in that case. A render, destination, write, sync, publication,
  or cleanup failure returns internal/reporter exit `4` and can never be
  hidden behind diagnostic success, failure, or incompleteness.
- The stable JSON schema, JUnit projection, finding codes and severities,
  primary and independent diagnosis, causal skips, performed/skipped state,
  outcome, exit metadata, redaction, and per-report four-MiB bound remain
  unchanged. At most the stdout projection plus the two file projections may
  exist, and their combined rendered bytes are additionally capped at eight
  MiB.

#### Destination and persistence boundary

- Before process launch, DNS, connection, credential resolution, discovery, or
  tool execution, every requested destination must name a new file in an
  existing directory. Existing regular files, non-regular entries, missing or
  invalid parents, duplicate paths, actual filesystem aliases, and conflicts
  with an `inspect` contract-snapshot destination fail with invocation exit
  `2`. There is no overwrite or force option.
- Preparation creates only bounded, exclusive same-directory stage files, with
  owner-only mode on Unix and destination-directory ACL inheritance elsewhere.
  Rendering completes in memory before any report destination appears.
  Each complete synchronized stage is published with a no-clobber same-filesystem
  operation; if any requested publication fails, every output created by the
  invocation is removed. Ordinary failure cleans every owned stage and output,
  and a cleanup failure remains a safe visible error without rendering a path.
- Destination paths, stage names, file contents, and operating-system errors
  never enter human, JSON, or JUnit diagnostic content. Files contain only the
  existing redacted reporter bytes; raw traffic, stderr, endpoints, headers,
  credentials, environment data, arguments, results, and server-provided
  values excluded by the reporter contract remain excluded.

#### Finite implementation and acceptance evidence

The implementation reuses the standard library and existing reporters and adds
no dependency. Deterministic unit tests cover reporter ordering, per-report and
aggregate bounds, render failure, destination preparation, alias detection,
exclusive publication, rollback, and cleanup. Built-binary STDIO, disposable
HTTP, and reviewed active journeys prove one conversation or process and no
replayed `tools/call`, while passing, failed, and incomplete cases prove JSON
and JUnit parity plus exit precedence. Negative cases cover existing,
duplicate, aliased, missing-parent, non-regular, unwritable, write-failing, and
cleanup-failing destinations without leaking paths or starting a target.
Existing stdout-only goldens remain byte-identical, and POSIX and PowerShell
installed smokes exercise both artifacts. `MCPD-021` completed on 2026-08-13
through protected [PR 50](https://github.com/EnjoyableWork/mcp-doctor/pull/50),
the complete locked local gate, `cargo deny`, deterministic package and
installed-source checks, exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31721429572)
and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31721429592),
exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31722608992)
and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31722609011),
and closure of [issue #72](https://github.com/EnjoyableWork/mcp-doctor/issues/72).
No publication was part of this ticket.

### MCPD-022 accepted offline aggregate plan

[GitHub issue #73](https://github.com/EnjoyableWork/mcp-doctor/issues/73)
identifies a real report-sufficiency gap. Maintainers can produce authoritative
per-run evidence across revisions, transports, platforms, packages, and
reviewed active scenarios, but a generic CI test view or ad hoc script cannot
preserve the complete primary-diagnosis, independent-safety, causal-skip,
limit, outcome, and exit contract. A conservative offline aggregate strengthens
the north star by letting humans and agents trust one bounded matrix result
without rerunning, comparing, suppressing, or scoring diagnostics.

#### CLI and evidence boundary

- Add `mcp-doctor aggregate --output PATH [--format human|json] REPORT...` for
  one through 32 explicit ordered `mcp-doctor.report/v1` JSON inputs. Human is
  the default stdout projection; JSON stdout is byte-identical to the required
  persisted `mcp-doctor.aggregate/v1` artifact. No input or output is inferred
  from a workspace, environment, repository, glob, or CI provider.
- Address members only by zero-based input ordinal. Preserve every known safe
  member field needed to understand its revision, negotiated revision, primary
  and independent findings, limits, summary, performed and skipped checks,
  causal links, structural reproduction, evidence, outcome, and exit. Accept
  and ignore compatible unknown optional properties under the stable v1
  consumer rule rather than reflecting untrusted values; retain an unknown
  finding code only when its grammar and all known safe metadata validate.
- Validate each member against the embedded stable report schema and then
  verify semantic invariants that schema shape alone cannot prove: unique
  checks; exact recomputed summary counts; finding severity/check outcome;
  report outcome; allowed outcome/exit relationships; primary, independent,
  and causal references to retained findings; and matching protocol revisions.
  An unsupported report major, malformed member, or inconsistent claim rejects
  the entire invocation instead of yielding an aggregate.
- Aggregate outcome is `failed` with exit `1` when any member failed,
  otherwise `incomplete` with exit `3` when any member is incomplete, and
  otherwise `passed` with exit `0`. There is no waiver, baseline, severity
  override, deduplication, majority rule, score, or percentage, and member
  failures and independent safety findings are never demoted or hidden.

#### Offline safety and finite work

- Accept only explicit existing regular files and reject symlinks, zero inputs,
  more than 32 inputs, duplicate paths, canonical aliases, and hard-link aliases
  before aggregation. Read through already-open handles under four MiB per
  input and 16 MiB total; never recursively scan or follow a report-discovered
  path or reference.
- Bound JSON nesting to 64, total JSON nodes and validation work to 1,000,000,
  retained checks to 4,096, retained findings to 2,048, rendered stdout plus
  artifact bytes to eight MiB, and monotonic operation time to ten seconds.
  The work bounds remain authoritative and deterministic; tests inject the
  clock rather than sleeping or retrying.
- Perform no process launch, network access, DNS, credential resolution,
  external schema retrieval, target discovery, contract comparison, generated
  case, or `tools/call`. Input paths, output paths, operating-system errors,
  unknown optional values, environment data, endpoints, headers, credentials,
  arguments, results, logs, and any other excluded values never enter valid
  output or errors.
- Require one explicit new aggregate destination in an existing directory.
  Render completely before publication; use an exclusive same-directory stage
  with Unix owner-only mode and platform ACL inheritance elsewhere, synchronize
  it, publish without clobber, and remove every owned stage or output on failure.
  Destination/input errors return invocation exit `2`; render, write,
  publication, or cleanup failure returns internal/reporter exit `4`.

#### Finite implementation and acceptance evidence

The implementation reuses `serde`, `serde_json`, `jsonschema`, the standard
library, and the existing artifact-safety pattern and adds no dependency. Stable
Rust `1.97.1` does not expose Windows handle identity, so one Windows-only module
isolates the `GetFileInformationByHandleEx(FileIdInfo)` call over an owned
no-follow handle, compares the complete 128-bit file identifier plus volume,
and fails closed when Windows cannot supply that identity. Its two unsafe
operations are limited to the documented valid-handle/output-buffer call and
post-success initialization invariant; focused native hard-link identity tests
cover the boundary. Committed aggregate schema and goldens cover all-pass,
pass-plus-incomplete, pass-plus-fail, multiple failures, independent safety
findings, mixed revision and transport evidence, byte determinism, JSON/human
parity, compatible unknown properties, and unknown finding codes. Negative and
trap journeys cover every input, identity, schema, semantic, depth, node, check,
finding, byte, time, render, destination, race, write, publication, cleanup,
redaction, and no-activity boundary. Existing `inspect`, `check`, `break`,
`diff`, JSON, JUnit, snapshot, and multi-artifact behavior remains
byte-compatible. POSIX and PowerShell installed-source smokes aggregate stable
reports on every represented path. `MCPD-022` completed on 2026-08-13 through
protected [PR 52](https://github.com/EnjoyableWork/mcp-doctor/pull/52), the
complete locked local gate, `cargo deny`, source-artifact and supply-chain
rehearsals, deterministic package and release-channel generation, packaged-source
installation and smoke checks, corrected exact-head
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31736927318) and
[release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31736927338),
exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31737876282)
and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31737876227)
on merge commit [`983919d`](https://github.com/EnjoyableWork/mcp-doctor/commit/983919d0ffae417133f829b806e8f5a9e72082b7),
and completed closure of [issue #73](https://github.com/EnjoyableWork/mcp-doctor/issues/73).
No dependency, workflow, publication, or immutable-release change was part of
the ticket.

### MCPD-023 accepted structured protocol-rejection correction

[GitHub issue #64](https://github.com/EnjoyableWork/mcp-doctor/issues/64)
identifies a causal-classification defect in current-revision Streamable HTTP.
The MCP `2026-07-28` contract defines HTTP `400` plus JSON-RPC error code
`-32022` and structured requested/supported revision data as a protocol-version
rejection. Treating that exact signal as a generic HTTP status failure points a
server author at the wrong layer and weakens the north star.

`MCPD-023` recognizes the signal only from an identity-encoded
`application/json` body already bounded by the existing response and message
budgets. The response must be a JSON-RPC error with the matching request
identifier, code `-32022`, a string
message, the exact requested current revision, and at most 32 string supported
revisions that do not contradict the rejection by containing the requested
revision. The implementation does not interpret error prose, retain or render
the message or offered values, retry the request, initialize, downgrade, or
fall back. A malformed, mismatched, contradictory, or over-limit lookalike
remains an HTTP-contract failure.

An exact match makes the HTTP transport and JSON-RPC envelope pass, fails
`protocol.revision` with `MCP-PROTOCOL-002`, and marks dependent discovery,
schema, and active work as causally skipped. Human, stable JSON, and JUnit
reports share that primary diagnosis and one safe fixed rule value. Disposable
passive and authorized-active built-binary journeys prove one request, no
fallback or tool call, reporter parity, and sentinel redaction; focused unit
cases prove the exact-match and negative shape boundaries. The contract was
reviewed against upstream commit
[`5947545`](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/594754559cc928eae08e184c74a89508c1235fc2/schema/2026-07-28/schema.ts),
and the correction adds no dependency or new network authority.

### MCPD-024 accepted `v0.3.0` release plan

`v0.3.0` is a backward-compatible feature release for the completed optional
`MCPD-019` through `MCPD-022` capabilities plus the `MCPD-023` correctness
fix. The minor version reflects additive command and reporting surfaces while
preserving existing command, stable report-major, exit, safety, and release
contracts. The release does not claim active legacy-revision support, general
security-scanner coverage, native signing, certification, or M4 completion.

The reviewed release change updates the package version, lockfile root,
supported release line, canonical security projection, release notes, pinned
installation guidance, and a portable README diagnosis example. That example
uses only ordinary Markdown paragraphs, emphasis, inline code, and a blockquote
so GitHub and crates.io can reflow it without terminal prompts, exit codes,
screenshots, custom HTML, or fixed-width layout assumptions.

Publication follows the retained `MCPD-008A` path: protected exact-head and
exact-`main` CI and release preflight; one intentional annotated `v0.3.0` tag at
the reviewed exact `main`; immutable GitHub artifacts, checksums, SBOMs, and
attestations; crates.io OIDC Trusted Publishing; the tap-owned exact-byte
formula update; and the credential-free installed-channel verifier on every
represented host and channel. Any version, source, digest, artifact, channel,
supported-line, or verification mismatch blocks publication or requires a new
version. This release remains independent of the unfinished `MCPD-017` and
does not convert its private organization evidence into a public claim.

`MCPD-023` and `MCPD-024` completed on 2026-08-13. Protected
[PR 54](https://github.com/EnjoyableWork/mcp-doctor/pull/54) at exact head
`6611f31a5e076dc771a33799c714fe2835ec6e51` passed first-attempt
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31746397550) and
[release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31746397557),
then merged as exact release source
[`d9b96bb`](https://github.com/EnjoyableWork/mcp-doctor/commit/d9b96bbeb84baccb8e5c890e9c655a559a12a474)
and closed [issue #64](https://github.com/EnjoyableWork/mcp-doctor/issues/64).
First-attempt exact-`main`
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31754685159) and
[release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31754685137)
passed before the signed annotated tag `v0.3.0`, tag object
`6d3032426c4d9a7d624eb771fbbc30fe7605801b`, was intentionally pushed at
that source commit.

The protected [release workflow](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31755736570)
published and reverified the
[immutable GitHub Release](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.3.0),
all seven checksummed and attested assets, and the byte-identical unyanked
[crates.io package](https://crates.io/crates/mcp-doctor/0.3.0) through OIDC.
The tap-owned [publication workflow](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/31756253855)
accepted the exact release formula and committed it as
[`2b62e11`](https://github.com/EnjoyableWork/homebrew-tap/commit/2b62e11902c7461cddbc0b96075e3745fdf6f260).
Finally, the credential-free
[channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31756413098)
passed all ten jobs on their first attempts: immutable release, registry, and
formula identity; two native GNU/Linux archive installs; four native Cargo
installs; and three native Homebrew installs. No retry, republish, source
change, or M4 assurance claim was used as completion evidence.

### MCPD-025 accepted compiled capability-manifest plan

GitHub issue #66 identifies a real integration defect: the installed binary
has no machine-readable product-capability contract. Editors, wrappers, CI
jobs, and server repositories must parse `--help`, compare product versions, or
start a diagnostic to discover whether a command, transport, MCP revision,
input contract, reporter, or output contract is supported. That can turn a
tool-target incompatibility into an apparent server failure and spends target
authority before the consumer knows whether the run is meaningful. A
side-effect-free compiled manifest therefore supports the north star by making
preflight selection and honest unsupported or unknown outcomes available to
the same human and agent consumers that rely on diagnostic evidence.

`DEC-050` adds `mcp-doctor capabilities [--format human|json]
[--schema-version SCHEMA]`. The default human form is a deterministic summary;
JSON uses stable `mcp-doctor.capabilities/v1`. A successful document names the
package version, command activity classes and reporters, exact supported MCP
revisions for each target-bearing command and transport, recognized but
unsupported revisions, every current report/scenario/generator/artifact
contract, `mcp-doctor.exit/v1`, default hard limit-profile identifiers, the
64-KiB manifest-output limit, and only compile-family process-tree and
file-identity capabilities. It contains no server advertisement or annotation
and no environment value, path, endpoint, credential, runtime observation, or
host inventory.

Schema requests are exact and default only to
`mcp-doctor.capabilities/v1`; there is no discovery, retry, fallback, or
downgrade. An unsupported request exits `2` without reflecting the request.
The JSON form returns a value-free typed error under the known `v1` envelope
and lists only supported manifest schemas; the human form returns one fixed
safe error. Successful and error documents remain under 64 KiB, deterministic
for one compiled binary, and require no file read, target, process, DNS,
network, schema retrieval, configuration discovery, credential resolution, or
tool call.

Within stable `v1`, required field names, types, and existing meanings remain
compatible. New optional fields and new command, transport, revision,
reporter, contract, platform, or profile entries are compatible; consumers
must ignore unknown optional fields and classify unrecognized values as
unknown. Removing a supported entry, making an existing supported combination
unsupported, or changing a required field, existing value meaning, exit
contract, or safety boundary requires a new capability major. The manifest is
compiled from constants owned by the implementing modules, and a command
inventory regression ties it to the CLI. Every later capability change must
update the manifest, schema/golden, consumer cases, documentation, and
installed smokes in the same ticket.

Acceptance requires the checked-in Draft 2020-12 schema, deterministic golden,
tri-state consumer fixture, schema-valid forward-compatible extension case,
unknown-version JSON and human failures, environment/proxy/target-like
no-activity regressions, compile-family assertions, human and JSON output
bounds, POSIX and PowerShell installed-source smokes, the complete locked local
and dependency gates, deterministic package/source review, and protected
exact-head CI plus release preflight on GNU/Linux x64/ARM64, macOS ARM64, and
Windows x64. This ticket adds no dependency and makes no immutable-release,
channel-wide, conformance, health, security, or M4 assurance claim.

`MCPD-025` completed on 2026-08-13 through protected
[PR 58](https://github.com/EnjoyableWork/mcp-doctor/pull/58), merged as
[`c5847ee`](https://github.com/EnjoyableWork/mcp-doctor/commit/c5847ee794c227376783b2828f44ce3de34c81b9),
which closed [issue #66](https://github.com/EnjoyableWork/mcp-doctor/issues/66). Exact
implementation head `f4a96a4cf14f8642e1e66c116c934f58ab86374a` passed its
first-attempt [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31761161743)
and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31761161698).
Those runs cover all three native quality-gate hosts, dependency policy,
deterministic source packaging, Windows source installation, GNU/Linux x64 and
ARM64 plus macOS ARM64 Cargo/formula installation, represented archive and
SBOM validation, and the new installed capability smoke on every applicable
path. No retry, dependency, target activity, publication, release-channel claim, or
M4 assurance claim is part of this evidence.

### MCPD-027 accepted active legacy-adapter plan

[GitHub issue #60](https://github.com/EnjoyableWork/mcp-doctor/issues/60)
identifies the first active compatibility slice: an author who explicitly
selects MCP `2025-11-25` can inspect the server but cannot replay a reviewed
scenario or generate bounded schema-valid cases against that same revision.
[Issue #61](https://github.com/EnjoyableWork/mcp-doctor/issues/61) depends on
the same active boundary for MCP `2025-06-18` and adds a deliberately stricter
schema-dialect gate. `DEC-052` resolves `OPEN-15` for both tickets without
making either revision an implemented active capability ahead of evidence.

#### One selected adapter and unchanged authority

- One typed active-protocol adapter is selected once from the explicit CLI
  value before target preparation. It owns request sequencing and encoding,
  handshake and catalog interpretation, schema-dialect policy, tool-result
  interpretation, and whether modern HTTP request metadata is eligible. The
  scenario, exact tool/effect/side-effect authorization, environment-only
  secret resolution, deterministic generator, sequential case runner,
  redacted result model, reporters, and bounded process and network transports
  remain shared and do not branch on revisions independently.
- `check` and `break` default to MCP `2026-07-28`. `MCPD-027` added the exact
  `2025-11-25` value; `MCPD-028` adds only exact `2025-06-18` under its stricter
  schema policy. No range, alias other than the existing current
  spelling, discovery-based selection, remembered choice, retry, downgrade, or
  fallback can select an active revision.
- The current adapter retains byte- and behavior-compatible `server/discover`,
  `_meta`, catalog, modern `resultType`, HTTP routing field, and
  `x-mcp-header` behavior. A current-revision golden and all existing active
  journeys are regression gates for the refactor rather than rewritten to
  resemble the legacy wire contract.

#### Exact legacy lifecycle, schemas, and results

- A legacy run sends one `initialize` request containing the exact selected
  revision, empty client capabilities, and bounded fixed `mcp-doctor` client
  identity. It advertises no tasks, elicitation, sampling, roots, or other
  server-request capability. The response must be a valid initialize result
  with the exact negotiated revision, a valid server identity, and a tools
  capability before exactly one `notifications/initialized` notification and
  the capability-gated `tools/list` sequence. Any malformed handshake,
  mismatch, absent tools capability, or exhausted bound stops dependent work
  without notification, catalog request, generation, or call as applicable.
- MCP `2025-11-25` input and advertised output schemas use local bounded JSON
  Schema Draft 2020-12 when `$schema` is omitted. An explicit dialect must be
  the already supported exact Draft 2020-12 identity; external references and
  unsupported or over-limit behavior still fail before generation or a call.
  Tool `execution.taskSupport` is bounded and accepts only `forbidden`,
  `optional`, or `required`, with omission defaulting to `forbidden`.
  `required` produces a typed actionable no-call diagnosis because this slice
  never starts or polls a task; `optional`, `forbidden`, and omitted use the
  ordinary immediate call only.
- MCP `2025-06-18` reuses that lifecycle and legacy result adapter under
  `MCPD-028`. Every advertised input or output schema interpreted for active
  authority must declare the exact supported Draft 2020-12 URI; omitted,
  malformed, unsupported, or ambiguous dialects and unsupported vocabularies
  stop before generation or `tools/call`. The passive omitted-dialect ambiguity contract remains
  unchanged and never becomes active authority.
- A legacy immediate `tools/call` success requires an array `content`, permits
  only an object `structuredContent` when present, and permits only a boolean
  `isError` when present; it does not require or interpret modern
  `resultType`. Output validation and expected success/tool-error
  classification retain the existing finite and value-free behavior. The
  exact bounded MCP `2025-11-25` URL-elicitation-required JSON-RPC error code
  `-32042` with structurally valid URL-mode entries marks the case incomplete
  without retaining its message, data, identifiers, URLs, or other values and
  without response, navigation, retrieval, or retry. Other errors retain the
  existing rejected-call diagnosis.

#### Legacy HTTP, server requests, and reporting

- Legacy Streamable HTTP reuses the `DEC-030` and `DEC-044` direct pinned
  transport. Initialize omits `MCP-Protocol-Version`; every later POST and the
  one bounded teardown use the exact selected revision and any bounded
  visible-ASCII session identifier established by initialize. JSON and
  request-scoped SSE, including the permitted `2025-11-25` empty priming
  event, finish on the matching response. Session loss is diagnosed without
  the specification's optional reinitialization path. One bounded DELETE is
  attempted when a session exists, and its independent cleanup finding is
  preserved. Modern `Mcp-Method`, `Mcp-Name`, `Mcp-Param-*`, and
  `x-mcp-header` mappings are never emitted for a legacy request.
- A server request is never answered. A structurally recognized request for
  elicitation or other additional input stops the affected case under the
  existing incomplete semantics without retaining its method parameters or
  values; every other unexpected request is a protocol failure. No server
  request, notification, advertisement, annotation, task capability, or
  elicitation error grants tool, side-effect, credential, target, or retry
  authority.
- Stable report `v1` retains its compatible selected and optional negotiated
  revision fields. Human, JSON, and JUnit must agree on those fields, the
  earliest primary diagnosis, causal skips, independent cleanup findings,
  outcome, and exit. New typed finding or rule values are compatible additions
  only and remain value-free. The compiled capability manifest, help, README
  matrix, and installed smokes add a revision only in the same change whose
  complete acceptance evidence proves that exact command/transport pair.

#### Evidence and claim threshold

`MCPD-027` requires positive and negative built-binary `check` and `break`
journeys for MCP `2025-11-25` over STDIO and Streamable HTTP; exact lifecycle,
catalog, result, Draft 2020-12 default, task-required, URL-elicitation, server
request, authority, bound, session, header-omission, teardown, redaction,
cleanup, reporter, manifest, and deterministic-generation coverage; unchanged
current-revision goldens and journeys; POSIX and PowerShell installed-source
smokes; the complete locked and dependency gates; protected merge; issue
closure; and durable evidence. Synthetic implementation evidence permits only
an exact revision/command/transport support statement. Broad compatibility
wording additionally requires the controlled official and independent cases
across at least two languages plus represented installed-channel journeys from
`DEC-024`. `MCPD-028` independently passed the equivalent synthetic and
installed-source matrix plus every strict dialect negative before changing the
exact `2025-06-18` active source claim; it adds no real-server compatibility
case for that revision.

#### Current implementation evidence

The `MCPD-027` merged source implements the typed adapter in
[`src/contract/active_protocol.rs`](src/contract/active_protocol.rs) and keeps
scenario authority, generation, reporting, and transport limits shared. The
built-binary suites cover positive and negative MCP `2025-11-25` STDIO and
Streamable HTTP lifecycle, schema, result, task, additional-input, session,
header, teardown, redaction, reporter, and current-revision regression paths.
The compiled capability manifest and both installed-smoke scripts expose the
same exact command, transport, and revision boundary.

The completed `MCPD-028` source selects MCP `2025-06-18` through that same
adapter and adds only its exact Draft 2020-12 active-schema policy. Synthetic
built-binary journeys cover successful and failed `check` and `break` over
STDIO and Streamable HTTP; exact initialization, legacy envelopes and results,
JSON and request-scoped SSE, session and teardown behavior, no fallback,
authority, redaction, causal reporters, and the missing, malformed, wrong,
unsupported-vocabulary, external-reference, and finite-limit schema gates.
The compiled manifest, help, README, compatibility claim boundary, and POSIX
and PowerShell source-install smokes describe the same exact-selected source
capability.

On 2026-08-14, the controlled compatibility runner passed all four retained
MCP `2026-07-28` passive cases plus MCP `2025-11-25` `check` and `break`
against the digest-pinned official Go hello server and independent PHP simple
server, both locally and in its exact-head hosted
[run](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31776618162).
Those two active cases span two languages and retain exact scenario, tool,
effect, case-count, and seed authority.
`scripts/check.sh`, `cargo deny --all-features --locked check`, two clean
deterministic `cargo package --locked` runs with matching archive SHA-256
`1136795c6aed30b58b69112ff07b318176a0938ac9aec8f5292b520bd895a168`, and a
disposable macOS install from that exact package followed by
`scripts/smoke-installed.sh` also pass without a dependency change.

Protected [PR 78](https://github.com/EnjoyableWork/mcp-doctor/pull/78) at exact
implementation head
[`b410f12`](https://github.com/EnjoyableWork/mcp-doctor/commit/b410f12550877f5df8c973796320002cededabba)
passes first-attempt native
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31776590472),
[CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31776588719),
and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31776590451).
Native Windows PowerShell execution and diagnosis plus macOS ARM64, GNU/Linux
ARM64, and GNU/Linux x64 Cargo/Homebrew source installs, capability smokes,
represented native archives, and SBOM validation pass in that preflight. Final
evidence head
[`5d56205`](https://github.com/EnjoyableWork/mcp-doctor/commit/5d562053a5bec65a8bc9a9364954d0cda87fed83)
then passed exact-head
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31777527666),
[CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31777526334),
[release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31777527665),
and [compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31777531248).

The protected squash merge produced exact `main` commit
[`ac3d9ac`](https://github.com/EnjoyableWork/mcp-doctor/commit/ac3d9ac1c289b3329eadbe8fb1a35cca597386c4)
and closed [issue #60](https://github.com/EnjoyableWork/mcp-doctor/issues/60).
First-attempt exact-`main`
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31778407756),
[CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31778407549),
[release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31778407803),
and [compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31778427941)
pass on 2026-08-14. Therefore `MCPD-027` is Done for the exact source support;
no dependency or release changed, and no published-channel or ecosystem-wide
compatibility claim follows.

For `MCPD-028`, `scripts/check.sh`,
`cargo deny --all-features --locked check`, source-artifact review, ShellCheck,
PowerShell syntax validation, the retained compatibility matrix, and the POSIX
installed smoke passed locally. Two clean `cargo package --locked` runs
produced the same 146-file archive with SHA-256
`cca378b4ae3ae3d13b24f8ded9f14d01f6297088e07b5d368840aab847a42466`;
installing that exact packaged source and running its full installed smoke also
passed without a dependency change. Protected
[PR 80](https://github.com/EnjoyableWork/mcp-doctor/pull/80) at exact head
[`e311e52`](https://github.com/EnjoyableWork/mcp-doctor/commit/e311e5279c0d44f1f62ea3f9755f671a3bad7b91)
passed first-attempt exact-head
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31807179992),
[CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31807179016),
[release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31807179957),
and [compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31807297863),
including native Windows execution and represented macOS and GNU/Linux source
installs, archives, and SBOM validation.

The protected squash merge produced exact `main` commit
[`e380b26`](https://github.com/EnjoyableWork/mcp-doctor/commit/e380b26c382ea2b83fefe41c153f00baea023db2)
and closed [issue #61](https://github.com/EnjoyableWork/mcp-doctor/issues/61).
First-attempt exact-`main`
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808031576),
[CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808031251),
[release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808031581),
and [compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808063280)
pass on 2026-08-14. Therefore `MCPD-028` is Done for exact MCP `2025-06-18`
source support; no dependency, release, published-channel, real-server
compatibility, ecosystem-wide compatibility, or M4 claim follows.

## Target architecture

```text
┌──────────────────────────────────────────────────────────┐
│                       CLI boundary                       │
│        argument parsing · output · stable exit code      │
└───────────────────────────┬──────────────────────────────┘
                            v
┌──────────────────────────────────────────────────────────┐
│                  diagnostic application                  │
│       inspect · check · break · diff · aggregate          │
│          capabilities · compiled manifest                 │
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
| D-10 | Explicit passive legacy-revision diagnostics | Optional compatibility | Done | [PR 47](https://github.com/EnjoyableWork/mcp-doctor/pull/47) and its exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31674282783) prove revision-specific synthetic STDIO and Streamable HTTP behavior for both selected legacy revisions across human, JSON, and JUnit reports; broad real-server wording remains gated on the controlled two-language matrix and represented installed-channel journeys |
| D-11 | Explicit current-revision contract snapshots and offline diffs | Optional developer workflow | Done | [PR 49](https://github.com/EnjoyableWork/mcp-doctor/pull/49), the [typed artifact and diff implementation](src/contract/snapshot.rs), [snapshot schema](schemas/mcp-doctor.contract-snapshot.v1alpha1.schema.json), [diff schema](schemas/mcp-doctor.contract-diff.v1alpha1.schema.json), [bounded built-binary journeys](tests/snapshot.rs), HTTP credential-exclusion journey, goldens, README contract, and POSIX/PowerShell installed smokes implement `DEC-045`; the complete local gate, dirty-tree package/install round trip, exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31708939630), and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31708939599) pass without a dependency change |
| D-12 | One-run stable JSON and JUnit report artifacts | Optional reporting workflow | Done | `DEC-046`, protected [PR 50](https://github.com/EnjoyableWork/mcp-doctor/pull/50), the [typed one-result artifact implementation](src/report_artifacts.rs), [bounded built-binary journeys](tests/report_artifacts.rs), STDIO/HTTP/active no-replay evidence, byte-compatible stdout goldens, POSIX and PowerShell installed smokes, complete local gates, exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31721429572) and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31721429592), exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31722608992) and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31722609011), and closed issue [#72](https://github.com/EnjoyableWork/mcp-doctor/issues/72) prove one-result fan-out, explicit destinations, no overwrite, aggregate bounds, rollback, reporter-exit precedence, projection parity, and no target replay without a dependency or publication change |
| D-13 | Conservative bounded offline diagnostic-report aggregates | Optional offline evidence | Done | `DEC-047`, protected [PR 52](https://github.com/EnjoyableWork/mcp-doctor/pull/52), the [typed aggregate implementation](src/aggregate.rs), [stable aggregate schema](schemas/mcp-doctor.aggregate.v1.schema.json), [bounded built-binary journeys](tests/aggregate.rs), goldens, POSIX and PowerShell installed smokes, complete local gates, corrected exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31736927318) and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31736927338), exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31737876282) and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31737876227), and completed issue [#73](https://github.com/EnjoyableWork/mcp-doctor/issues/73) prove conservative normalized aggregation, finite offline work, native alias rejection, failure-safe publication, redaction, and zero target activity without a dependency or release change |
| D-14 | Correct structured unsupported-version causal diagnosis | Optional correctness | Done | `DEC-048`, the bounded classifier and reporter-parity journeys, protected [PR 54](https://github.com/EnjoyableWork/mcp-doctor/pull/54), first-attempt exact-head and exact-`main` CI/preflight, merge commit [`d9b96bb`](https://github.com/EnjoyableWork/mcp-doctor/commit/d9b96bbeb84baccb8e5c890e9c655a559a12a474), closed [issue #64](https://github.com/EnjoyableWork/mcp-doctor/issues/64), and verified `v0.3.0` installed artifacts prove the causal correction without replay, fallback, value reflection, or a dependency change |
| D-15 | Immutable independently verified `v0.3.0` feature release | Optional release | Done | Exact release source [`d9b96bb`](https://github.com/EnjoyableWork/mcp-doctor/commit/d9b96bbeb84baccb8e5c890e9c655a559a12a474), the [immutable GitHub Release](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.3.0), byte-identical [crates.io](https://crates.io/crates/mcp-doctor/0.3.0) and [Homebrew](https://github.com/EnjoyableWork/homebrew-tap/commit/2b62e11902c7461cddbc0b96075e3745fdf6f260) handoffs, and the first-attempt [ten-job installed-channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31756413098) pass |
| D-16 | Stable compiled product-capability discovery | Optional integration discovery | Done | `DEC-050`, protected [PR 58](https://github.com/EnjoyableWork/mcp-doctor/pull/58), merge commit [`c5847ee`](https://github.com/EnjoyableWork/mcp-doctor/commit/c5847ee794c227376783b2828f44ce3de34c81b9), exact implementation head `f4a96a4cf14f8642e1e66c116c934f58ab86374a`, first-attempt exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31761161743) and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31761161698), the [typed compiled manifest](src/capabilities.rs), [stable schema](schemas/mcp-doctor.capabilities.v1.schema.json), deterministic golden, tri-state consumer and forward-compatible fixtures, [bounded built-binary journeys](tests/capabilities.rs), README contract, represented installed smokes, and closed [issue #66](https://github.com/EnjoyableWork/mcp-doctor/issues/66) prove compiled-only discovery without target activity, a dependency, publication, or release claim |
| D-17 | Explicit legacy contract snapshots and same-revision offline diffs | Optional developer workflow | Done | `DEC-051`; protected [PR 63](https://github.com/EnjoyableWork/mcp-doctor/pull/63), exact implementation head `ff6f4223b84956fcad39b6e5f8184ddd7eaf469e`, final evidence head `2679e867bc108ca543b89a317fa1f9945eef9097`, merge commit [`6e0f0ac`](https://github.com/EnjoyableWork/mcp-doctor/commit/6e0f0acf096f797a12f3bf8826d8d11963007039), closed [issue #74](https://github.com/EnjoyableWork/mcp-doctor/issues/74), the typed implementation, schemas, current byte golden, bounded STDIO/HTTP and offline negative journeys, README/CLI/manifest assertions, represented installed smokes, complete local gate, `cargo-deny`, source review, clean deterministic package and packaged install, final exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31770718856), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31770715886), and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31770718592), plus first-attempt exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31771389361), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31771389015), and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31771389387) prove the exact passive legacy artifact extension without replay, cross-revision inference, a dependency, release, broad compatibility, or M4 claim |
| D-18 | Explicit MCP `2025-11-25` `check` and `break` | Optional compatibility | Done | `DEC-052`; protected [PR 78](https://github.com/EnjoyableWork/mcp-doctor/pull/78), implementation head [`b410f12`](https://github.com/EnjoyableWork/mcp-doctor/commit/b410f12550877f5df8c973796320002cededabba), final evidence head [`5d56205`](https://github.com/EnjoyableWork/mcp-doctor/commit/5d562053a5bec65a8bc9a9364954d0cda87fed83), merge commit [`ac3d9ac`](https://github.com/EnjoyableWork/mcp-doctor/commit/ac3d9ac1c289b3329eadbe8fb1a35cca597386c4), closed [issue #60](https://github.com/EnjoyableWork/mcp-doctor/issues/60), the typed adapter, positive and negative STDIO/HTTP journeys, current-revision regressions, controlled official Go and independent PHP cases, complete local and dependency gates, clean deterministic package/install evidence, final exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31777527666), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31777526334), [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31777527665), and [compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31777531248), plus first-attempt exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31778407756), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31778407549), [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31778407803), and [compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31778427941) pass on 2026-08-14 without a dependency, release, published-channel, ecosystem-wide compatibility, or M4 change |
| D-19 | Explicit MCP `2025-06-18` `check` and `break` | Optional compatibility | Done | `DEC-052`; protected [PR 80](https://github.com/EnjoyableWork/mcp-doctor/pull/80), exact implementation head [`e311e52`](https://github.com/EnjoyableWork/mcp-doctor/commit/e311e5279c0d44f1f62ea3f9755f671a3bad7b91), merge commit [`e380b26`](https://github.com/EnjoyableWork/mcp-doctor/commit/e380b26c382ea2b83fefe41c153f00baea023db2), closed [issue #61](https://github.com/EnjoyableWork/mcp-doctor/issues/61), shared-adapter and exact-dialect positive and no-call negative journeys over both transports, reporter and manifest parity, unchanged current and MCP `2025-11-25` regressions, complete local and dependency gates, clean deterministic package/install evidence, exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31807179992), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31807179016), [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31807179957), and [compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31807297863), plus first-attempt exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808031576), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808031251), [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808031581), and [compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808063280) pass on 2026-08-14 without a dependency, release, published-channel, real-server compatibility, ecosystem-wide compatibility, or M4 change |
| D-20 | Bounded schema-invalid tool-argument rejection diagnosis | Optional active correctness | Done | `DEC-053`; protected [PR 82](https://github.com/EnjoyableWork/mcp-doctor/pull/82), exact implementation head [`b33f3fd`](https://github.com/EnjoyableWork/mcp-doctor/commit/b33f3fd2a4fa4f34d953da632a8e599792610b8d), final evidence head [`984da68`](https://github.com/EnjoyableWork/mcp-doctor/commit/984da68997b56e1f190a5b97f23a9e7871a20717), merge commit [`3472952`](https://github.com/EnjoyableWork/mcp-doctor/commit/3472952a521ad30fbf716c828739887835a78898), closed [issue #75](https://github.com/EnjoyableWork/mcp-doctor/issues/75), complete local, package/install, exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31822239279), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31822236681), and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31822239369), plus first-attempt exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31825446998), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31825446882), and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31825447073) pass on 2026-08-14 without a dependency, publication, legacy, compatibility, or M4 claim |
| D-21 | Deterministic CI policy and timing-dependent evidence audit | Optional engineering assurance | Done | `DEC-054`, `MCPD-030`, protected [PR 83](https://github.com/EnjoyableWork/mcp-doctor/pull/83), exact implementation head [`6fb3232`](https://github.com/EnjoyableWork/mcp-doctor/commit/6fb3232782b0898c51c294efbb6c6a50296a8be5), final evidence head [`aed30fe`](https://github.com/EnjoyableWork/mcp-doctor/commit/aed30fe619d99d7bc354036e435ddf279c5c98f8), merge commit [`dbc19bd`](https://github.com/EnjoyableWork/mcp-doctor/commit/dbc19bd7a863c8e53651a78bd4570616a59d5e02), and closed [issue #41](https://github.com/EnjoyableWork/mcp-doctor/issues/41) prove [the dated tracked audit](docs/deterministic-ci.md), enforced inventories, direct cleanup evidence, and exact runner-command verification. Complete local, dependency, source-review, actionlint, verifier, deterministic package/generation, and installed-smoke evidence; first-attempt exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31830206578), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31830202365), and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31830206650); and first-attempt exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31831098275), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31831098605), and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31831098203) pass on 2026-08-14 without a retry, timeout inflation, assertion weakening, quarantine, broad serialization, dependency, product/runtime contract, release artifact, live setting, or M4 change |
| D-22 | Deterministic release-state, retry, and runner-tool correction | Optional release maintenance | Proposed | `MCPD-031` owns the eight workflow and one verifier broad curl retries, two fixed-sleep publication-state loops, and legacy release-job runner verification identified by the `MCPD-030` audit; it must replace or narrowly reclassify them under `DEC-054` and pass a nonpublishing rehearsal before any later public release, without changing immutable `v0.1.0`, `v0.2.0`, or `v0.3.0` bytes |
| D-23 | Invocation-local named diagnostic limit selection | Optional runtime ergonomics | In progress | `DEC-055` and `MCPD-032` define two finite compiled profiles for `inspect`, `check`, and `break`; completion requires pre-target rejection, unchanged cleanup/capacity/authority bounds, report and capability-manifest parity, deterministic cross-transport and installed-source evidence, protected merge, issue #65 closure, and no dependency, configuration, numeric override, release, or M4 change |

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
| MCPD-016 | Harden dependency maintenance and the CI, artifact, and distribution supply chains | M4 | Done | `MCPD-015` | Protected [activation PR 25](https://github.com/EnjoyableWork/mcp-doctor/pull/25), rejected [Cargo proposal 26](https://github.com/EnjoyableWork/mcp-doctor/pull/26), accepted [Action proposal 27](https://github.com/EnjoyableWork/mcp-doctor/pull/27), disposable fork [PR 29](https://github.com/EnjoyableWork/mcp-doctor/pull/29), [closure PR 38](https://github.com/EnjoyableWork/mcp-doctor/pull/38), and [evidence correction PR 39](https://github.com/EnjoyableWork/mcp-doctor/pull/39) bind the exact-`main` CI, CodeQL, release preflight, live readbacks, artifact negatives, and authenticated operator result below without auto-merge, stored secrets, or changed published bytes |
| MCPD-016A | Remove indirect mutable Syft acquisition from release automation | M4 | Done | `MCPD-016` | `DEC-043`, protected [PR 43](https://github.com/EnjoyableWork/mcp-doctor/pull/43), deterministic local regressions, the narrowed live selected-Action readback, the authenticated exact-`main` verifier, and first-attempt exact-head plus exact-`main` CI, CodeQL, and release-preflight evidence below prove repository-owned immutable Linux ARM64/x64 acquisition and removal of the obsolete Action without changing an immutable release byte |
| MCPD-017 | Establish organization access, credential, ownership, and recovery policy | M4 | In progress | `MCPD-016A` | `DEC-041`, `DEC-042`, the verified [canonical projection](.github/organization-controls.json), [authenticated verifier](scripts/verify-organization-controls.sh), and [credential-free negative rehearsal](scripts/rehearse-organization-controls.sh) fix and test the selected boundary; all supported live controls, the exact source-and-tap verifier credential, bounded private App and Codespaces inventories, and the independent private recovery exercise pass, while protected merge, exact-`main` verification, temporary-credential revocation, and final aggregate evidence remain completion gates |
| MCPD-018 | Self-assess, publish, and maintain the enterprise assurance baseline | M4 | Proposed | `MCPD-017` | Every selected OSPS Level 1 control has public evidence or exact applicability reasoning; the official assessment and badge are verified on exact `main`; exact M3 artifacts receive a correctly scoped SLSA Build L2 evaluation; and claim-review and removal triggers are documented |
| MCPD-019 | Add explicit passive `inspect` diagnostics for MCP `2025-11-25` and `2025-06-18` over STDIO and Streamable HTTP | Optional compatibility | Done | `MCPD-012` | `DEC-044`, [PR 47](https://github.com/EnjoyableWork/mcp-doctor/pull/47), and exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31674282783) prove exact opt-in selection, selected/negotiated reporter parity, revision-specific bounded lifecycle and catalog behavior, positive and negative built-binary journeys for both revisions and transports, and the complete locked gate without a dependency change; broad real-server wording remains withheld until the controlled official/independent two-language and represented installed-channel evidence passes |
| MCPD-020 | Add exact-opt-in sensitive contract snapshots and deterministic offline diffs for passive MCP `2026-07-28` inspection | Optional developer workflow | Done | `MCPD-012` | `DEC-045`; [PR 49](https://github.com/EnjoyableWork/mcp-doctor/pull/49), typed snapshot and diff models, checked-in schemas and goldens, 9 focused built-binary snapshot/diff journeys, the credentialed same-conversation HTTP journey, malformed/limit/correlation/overwrite/offline-isolation negatives, secret-exclusion sentinels, POSIX and PowerShell installed smokes, the complete disposable locked gate, `cargo-deny`, source-artifact review, deterministic package verification, a packaged-source install round trip, exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31708939630), and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31708939599) pass without a dependency change |
| MCPD-021 | Emit stable JSON and JUnit artifacts from one diagnostic run | Optional reporting workflow | Done | `MCPD-012` | `DEC-046`; protected [PR 50](https://github.com/EnjoyableWork/mcp-doctor/pull/50), the [artifact persistence boundary](src/report_artifacts.rs), [focused artifact tests](tests/report_artifacts.rs), STDIO/HTTP/active and installed-platform no-replay evidence, the complete local and dependency gates, exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31721429572) and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31721429592), exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31722608992) and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31722609011), and closed issue [#72](https://github.com/EnjoyableWork/mcp-doctor/issues/72) prove byte-compatible stdout plus explicit bounded JSON/JUnit files, pre-activity rejection, no overwrite, no target or tool replay, reporter parity, rollback, and exit precedence without a dependency or publication change |
| MCPD-022 | Aggregate stable diagnostic reports conservatively without target activity | Optional offline evidence | Done | `MCPD-021` | `DEC-047`; protected [PR 52](https://github.com/EnjoyableWork/mcp-doctor/pull/52), the implementation, stable schema, goldens, exhaustive offline, semantic, compatibility, alias, limit, destination, trap, redaction, and installed-platform evidence, complete locked local and dependency gates, corrected exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31736927318) and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31736927338), exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31737876282) and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31737876227), and completed issue [#73](https://github.com/EnjoyableWork/mcp-doctor/issues/73) prove the accepted conservative offline aggregate contract without target activity, a dependency, or a release change |
| MCPD-023 | Classify the exact current-revision unsupported-version response at the protocol layer | Optional correctness | Done | `MCPD-010` | `DEC-048`; focused passive, active, reporter-parity, exact-shape, malformed-shape, redaction, and no-replay evidence; protected [PR 54](https://github.com/EnjoyableWork/mcp-doctor/pull/54); first-attempt exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31746397550) and [preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31746397557); exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31754685159) and [preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31754685137); merge commit [`d9b96bb`](https://github.com/EnjoyableWork/mcp-doctor/commit/d9b96bbeb84baccb8e5c890e9c655a559a12a474); closed [issue #64](https://github.com/EnjoyableWork/mcp-doctor/issues/64); and installed `v0.3.0` artifact verification |
| MCPD-024 | Publish and independently verify completed optional capabilities as `v0.3.0` | Optional release | Done | `MCPD-023` | Signed annotated tag `v0.3.0` and immutable [release workflow](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31755736570); exact GitHub asset, crates.io OIDC, and tap-owned [publication](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/31756253855) handoffs; tap commit [`2b62e11`](https://github.com/EnjoyableWork/homebrew-tap/commit/2b62e11902c7461cddbc0b96075e3745fdf6f260); and first-attempt [ten-job installed-channel verification](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31756413098) |
| MCPD-025 | Expose a stable compiled capability manifest without target activity | Optional integration discovery | Done | `MCPD-012` | `DEC-050`; protected [PR 58](https://github.com/EnjoyableWork/mcp-doctor/pull/58); merge commit [`c5847ee`](https://github.com/EnjoyableWork/mcp-doctor/commit/c5847ee794c227376783b2828f44ce3de34c81b9); exact implementation head `f4a96a4cf14f8642e1e66c116c934f58ab86374a`; first-attempt exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31761161743) and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31761161698); typed deterministic human and `mcp-doctor.capabilities/v1` output, exact schema rejection, command/transport/revision and recognized-unsupported matrices, schema/reporter/exit/profile/platform inventories, 64-KiB bound, schema/golden/consumer/forward-compatibility/no-activity regressions, represented installed smokes, and closed [issue #66](https://github.com/EnjoyableWork/mcp-doctor/issues/66) without a dependency, target activity, publication, or release claim |
| MCPD-026 | Extend sensitive contract snapshots and offline diffs to explicit MCP `2025-11-25` and `2025-06-18` inspection | Optional developer workflow | Done | `MCPD-019`, `MCPD-020` | `DEC-051` resolves `OPEN-14`; protected [PR 63](https://github.com/EnjoyableWork/mcp-doctor/pull/63), exact implementation head `ff6f4223b84956fcad39b6e5f8184ddd7eaf469e`, final evidence head `2679e867bc108ca543b89a317fa1f9945eef9097`, merge commit [`6e0f0ac`](https://github.com/EnjoyableWork/mcp-doctor/commit/6e0f0acf096f797a12f3bf8826d8d11963007039), closed [issue #74](https://github.com/EnjoyableWork/mcp-doctor/issues/74), complete synthetic, secret-exclusion, current-byte-regression, schema, README/CLI/manifest, POSIX and native PowerShell installed/package, locked-gate, dependency, source-review, clean deterministic package, final exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31770718856), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31770715886), and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31770718592), and first-attempt exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31771389361), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31771389015), and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31771389387) evidence pass on 2026-08-14 without a dependency, target replay, active legacy behavior, release, broad compatibility, or M4 change |
| MCPD-027 | Add explicit MCP `2025-11-25` `check` and `break` over STDIO and Streamable HTTP | Optional compatibility | Done | `MCPD-019`, `MCPD-025` | `DEC-052`; protected [PR 78](https://github.com/EnjoyableWork/mcp-doctor/pull/78), implementation head [`b410f12`](https://github.com/EnjoyableWork/mcp-doctor/commit/b410f12550877f5df8c973796320002cededabba), final evidence head [`5d56205`](https://github.com/EnjoyableWork/mcp-doctor/commit/5d562053a5bec65a8bc9a9364954d0cda87fed83), merge commit [`ac3d9ac`](https://github.com/EnjoyableWork/mcp-doctor/commit/ac3d9ac1c289b3329eadbe8fb1a35cca597386c4), and closed [issue #60](https://github.com/EnjoyableWork/mcp-doctor/issues/60) prove the revision-parameterized adapter, exact initialize/result/schema/task/additional-input behavior, both transports, reporter and manifest parity, current regressions, controlled official Go and independent PHP active STDIO cases, complete local gates, clean package/install evidence, native Windows execution, represented source-install platforms, final exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31777527666), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31777526334), [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31777527665), and [compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31777531248), and first-attempt exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31778407756), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31778407549), [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31778407803), and [compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31778427941) on 2026-08-14 without a dependency, release, published-channel, ecosystem-wide compatibility, or M4 change |
| MCPD-028 | Extend explicit `check` and `break` to MCP `2025-06-18` with a no-guess schema gate | Optional compatibility | Done | `MCPD-027` | `DEC-052`; protected [PR 80](https://github.com/EnjoyableWork/mcp-doctor/pull/80), exact implementation head [`e311e52`](https://github.com/EnjoyableWork/mcp-doctor/commit/e311e5279c0d44f1f62ea3f9755f671a3bad7b91), merge commit [`e380b26`](https://github.com/EnjoyableWork/mcp-doctor/commit/e380b26c382ea2b83fefe41c153f00baea023db2), and closed [issue #61](https://github.com/EnjoyableWork/mcp-doctor/issues/61) prove reuse of the shared active adapter, exact lifecycle/result/HTTP/authorization/report/no-fallback behavior, and the exact Draft 2020-12 pre-generation and pre-call gate for every interpreted advertised input or output schema. Positive and missing, malformed, wrong, unsupported-vocabulary, external-reference, and finite-limit no-call journeys cover both transports; tasks remain unsupported; passive ambiguity, current, and MCP `2025-11-25` behavior remain unchanged. Complete local, dependency, source-review, clean package/install, POSIX/PowerShell smoke, final exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31807179992), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31807179016), [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31807179957), and [compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31807297863), plus first-attempt exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808031576), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808031251), [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808031581), and [compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808063280) pass on 2026-08-14 without a dependency, release, published-channel, real-server compatibility, ecosystem-wide compatibility, or M4 change |
| MCPD-029 | Diagnose bounded schema-invalid tool-argument rejection for the current active revision | Optional active correctness | Done | `MCPD-012` | `DEC-053`; protected [PR 82](https://github.com/EnjoyableWork/mcp-doctor/pull/82), exact implementation head [`b33f3fd`](https://github.com/EnjoyableWork/mcp-doctor/commit/b33f3fd2a4fa4f34d953da632a8e599792610b8d), final evidence head [`984da68`](https://github.com/EnjoyableWork/mcp-doctor/commit/984da68997b56e1f190a5b97f23a9e7871a20717), merge commit [`3472952`](https://github.com/EnjoyableWork/mcp-doctor/commit/3472952a521ad30fbf716c828739887835a78898), closed [issue #75](https://github.com/EnjoyableWork/mcp-doctor/issues/75), seven deterministic mutation kinds, exact authorization and invalidity gates, unsafe-success and failure distinctions, value-free reporter and capability coverage, complete local and package/install evidence, final exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31822239279), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31822236681), and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31822239369), plus first-attempt exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31825446998), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31825446882), and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31825447073) pass on 2026-08-14 without a dependency, publication, legacy, compatibility, or M4 change |
| MCPD-030 | Adopt deterministic CI policy and audit timing-dependent tests and automation | Optional engineering assurance | Done | `MCPD-003` | `DEC-054`; protected [PR 83](https://github.com/EnjoyableWork/mcp-doctor/pull/83), exact implementation head [`6fb3232`](https://github.com/EnjoyableWork/mcp-doctor/commit/6fb3232782b0898c51c294efbb6c6a50296a8be5), final evidence head [`aed30fe`](https://github.com/EnjoyableWork/mcp-doctor/commit/aed30fe619d99d7bc354036e435ddf279c5c98f8), merge commit [`dbc19bd`](https://github.com/EnjoyableWork/mcp-doctor/commit/dbc19bd7a863c8e53651a78bd4570616a59d5e02), and closed [issue #41](https://github.com/EnjoyableWork/mcp-doctor/issues/41) prove the complete tracked audit, explicit classifications, direct cleanup and HTTP state-transition evidence, current hosted runner verification, enforced inventories, preserved historical failures, and proposed `MCPD-031` ownership for release-only corrections. Complete local, dependency, source-review, actionlint, POSIX/PowerShell verifier, deterministic package/generation, and installed-smoke evidence; first-attempt exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31830206578), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31830202365), and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31830206650); and first-attempt exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31831098275), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31831098605), and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31831098203) pass on 2026-08-14 without a retry, quarantine, timeout inflation, assertion weakening, broad serialization, dependency, or product/runtime/release/live-setting/M4 change. |
| MCPD-031 | Replace legacy release polling, broad network retries, and implicit runner tools before the next public version | Optional release maintenance | Proposed | `MCPD-030`, `MCPD-024` | Before a successor to `v0.3.0` is authorized, remove or explicitly narrow the eight `curl --retry 5` workflow uses, the one verifier use, and the two fixed-five-second publication-state loops identified in [the deterministic-CI audit](docs/deterministic-ci.md), and make each release-producer or installed-channel source checkout verify its exact declared runner contract before evidence work. Observe immutable GitHub and crates.io state itself under one explicit outer deadline; retry only an exact immutable, integrity-verified acquisition and only for classified transient failures within the accepted cap; never retry publication, integrity, a job, or a workflow. Require deterministic policy regressions, a nonpublishing rehearsal, complete local gates, and first-attempt exact-head hosted evidence. Change no existing immutable artifact, M4 gate, product timeout, or live setting merely to close the audit. |
| MCPD-032 | Add invocation-local named bounded diagnostic limit profiles | Optional runtime ergonomics | In progress | `MCPD-004`, `MCPD-010`, `MCPD-025` | `DEC-055` resolves [issue #65](https://github.com/EnjoyableWork/mcp-doctor/issues/65) with exact `default` and `slow-start` selections on `inspect`, `check`, and `break`. `default` preserves the current 10-second startup/discovery, 30-second request/response, two-second cleanup, and 120-second total bounds; `slow-start` uses 30-second startup/discovery, 60-second request/response, the same two-second cleanup, and a 240-second total. All byte, message, schema, retry, redirect, concurrency, credential, network, process, and tool-authority limits remain unchanged. Reject every other value before target preparation; publish the selection and effective limits through human, JSON, JUnit, aggregate, and capability contracts; and prove deterministic behavior over STDIO, HTTP, active authorization, reporter artifacts, CLI rejection, package/install smokes, complete locked gates, protected merge, issue closure, and durable evidence. Add no individual numeric override, configuration source, unbounded or adaptive mode, retry, fallback, dependency, publication, compatibility, assurance, or M4 claim. |

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
| Syft `1.51.0` | `MCPD-008` — `1.50.0` selected 2026-08-10; acquisition and security correction accepted under `MCPD-016A` on 2026-08-12 | Adopted release tooling | Generate target SPDX 2.3 JSON only for the two represented GNU/Linux archives. Fetch the immutable `linux_amd64` and `linux_arm64` `1.51.0` release assets directly under exact byte, SHA-256, four-entry layout, version, and platform checks; discard partial bytes and allow at most three attempts only for the `DEC-043` transient acquisition classes. Never use the former Action's mutable `main/install.sh`, retry generation or validation, enable Syft network or host-cache cataloging, or broaden macOS/Windows artifact scope. |
| `Homebrew/actions/setup-homebrew` `2026.08.10.1` | `MCPD-008` — selected 2026-08-10; updated under `MCPD-016` on 2026-08-12 | Adopted release tooling | Pin immutable release commit `c8707045ccae42888fe98e86f2ee8938bc7cc193`; use only to style, audit, source-build, test, and smoke the exact formula on represented native hosts with both token inputs empty. The reviewed update removes an unused privileged sandbox-setup path, changes no selected dependency or nested Action, retains exact BSD-2-Clause license bytes, and passes every represented source/formula journey. |
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
| MCPD-016A | Complete `MCPD-016A`: replace indirect Syft acquisition with one repository-owned GNU/Linux ARM64/x64 installer that downloads only exact immutable `v1.51.0` assets, verifies reviewed size, SHA-256, layout, version, and platform before execution, retries only explicitly classified transient acquisition failures within three attempts, and deterministically rejects every permanent, trust, integrity, mapping, execution, and generation failure. Remove the obsolete Action from workflows, canonical policy, live selected-Action settings, tests, and current documentation; preserve SPDX output and release scope; and finish only when local checks plus first-attempt exact-head and exact-`main` CI, CodeQL, and release preflight pass without a workflow rerun or changed published byte. |
| MCPD-017 | Complete `MCPD-017`: define and verify strong-MFA, lowest-default-access, manual-grant, repository-creation, installed-application, automation-credential, ownership-continuity, and recovery controls using aggregate non-sensitive evidence. Any live organization mutation or private recovery confirmation requires explicit owner authority and must not expose identities or recovery material. |
| MCPD-018 | Complete `MCPD-018` under `DEC-034`: confirm that the activation-locked OSPS `v2026.02.19`, BadgeApp baseline series `v2026.02.19`, and SLSA `v1.2` proof routes remain current and available or stop for a superseding decision; publish the two dated and scoped crosswalks; complete the official baseline-1 self-assessment only after every applicable control passes; verify its public record, JSON, badge, and exact-`main` evidence; and verify every canonical M3 GitHub Release asset against Build L2 using its exact digest and constrained signed provenance. Define annual and event-driven review and removal triggers. Never imply independent certification, regulatory compliance, higher OSPS levels, channel-wide or future-artifact SLSA coverage, or paid platform signing. |
| MCPD-019 | Complete `MCPD-019` without delaying M4: add explicit `inspect --protocol-version` support for MCP `2025-11-25` and `2025-06-18` while retaining `2026-07-28` as the default and sole active revision. Implement each selected revision's exact passive lifecycle and capability-gated catalog contract over bounded STDIO and Streamable HTTP; preserve one initialized notification, session affinity, protocol headers, finite request-scoped SSE, causal session diagnostics, independent teardown findings, redaction, cleanup, and no retry or fallback. Finish the implementation when synthetic built-binary and reporter-parity evidence plus the complete locked gate pass; withhold broad real-server wording until controlled official and independent cases across two languages and represented installed channels also pass. |
| MCPD-020 | Complete `MCPD-020` without delaying M4: add exact-path, affirmatively acknowledged `mcp-doctor.contract-snapshot/v1alpha1` creation to a completed bounded passive MCP `2026-07-28` STDIO or Streamable HTTP conversation whose cleanup succeeds, including when a bounded local schema shape retained by the artifact makes the ordinary report fail, plus deterministic file-only human and `mcp-doctor.contract-diff/v1alpha1` comparison. Preserve ordinary report redaction, artifact sensitivity warnings, exclusive bounded file creation, artifact-local ordinal correlation, documented conservative compatibility rules, causal performed/skipped evidence, and zero diff-time process, network, retrieval, or tool activity. Add no dependency, no overwrite mode, no digest/token, no score, no general schema-implication claim, and no publication. Finish when schemas, goldens, exhaustive negative and secret-exclusion journeys, represented installed round trips, packaging, the complete locked gate, and hosted required checks pass. |
| MCPD-021 | Resolve GitHub issue #72 without delaying M4: preserve byte-compatible `--format` stdout behavior while adding explicit `--json-report PATH` and `--junit-report PATH` destinations to `inspect`, `check`, and `break`. Construct one immutable redacted `DiagnosticReport`, render every requested projection from it under the existing four-MiB per-report and new eight-MiB aggregate bounds, and perform no second process, connection, discovery request, credential resolution, generated case, or `tools/call`. Validate new-file destinations and snapshot conflicts before activity; reject existing, non-regular, missing-parent, duplicate, aliased, or unwritable targets; use exclusive same-directory stages with Unix owner-only mode and platform ACL inheritance elsewhere, publish without overwrite only after every render and write succeeds, roll back every owned output on failure, surface cleanup failure with exit `4`, and never render paths or operating-system values. Preserve stable JSON/JUnit semantics, diagnostic exit when all files succeed, and reporter failure precedence. Add no dependency, reporter, upload, CI-provider integration, merge feature, or immutable-release change. Finish only after deterministic unit, STDIO, HTTP, reviewed active, failure, redaction, byte-compatibility, POSIX and PowerShell installed-smoke, package, locked local, protected exact-head and exact-`main` evidence pass, the protected fix merges, issue #72 closes, and durable completion evidence is recorded. |
| MCPD-022 | Resolve GitHub issue #73 without delaying M4: add `mcp-doctor aggregate --output PATH [--format human|json] REPORT...` for one through 32 explicit ordered existing `mcp-doctor.report/v1` regular files. Perform no process, network, DNS, credential, external-schema, target, discovery, generation, or tool activity. Reject symlinks, duplicate/canonical/hard-link aliases, malformed or incompatible reports, semantic inconsistencies, exhausted four-MiB per-input, 16-MiB aggregate-input, 64-depth, one-million-node/work, 4,096-check, 2,048-finding, eight-MiB rendered-output, or ten-second monotonic bounds, and unsafe output destinations without rendering paths or untrusted values. Validate the embedded stable schema plus summaries, outcomes, exits, revisions, findings, and causal references; accept and ignore unknown optional values while retaining grammar-safe unknown finding codes and known safe metadata. Emit deterministic concise human stdout by default or JSON stdout byte-identical to the required stable `mcp-doctor.aggregate/v1` artifact; use zero-based ordinals, normalized known-safe members, fail-over-incomplete-over-pass precedence and exits `1`/`3`/`0`, exclusive owner-only same-directory staging, no-clobber publication, rollback, and visible cleanup failure. Add no dependency, retry, concurrency, waiver, score, percentage, baseline, deduplication, orchestration, upload, report-major coercion, publication, or immutable-release change. Finish only after schema/goldens, exhaustive offline/trap/redaction/compatibility/limit/destination regressions, unchanged existing behavior, POSIX and PowerShell installed-source smokes, package and complete locked gates, protected exact-head and exact-`main` evidence, protected merge, issue #73 closure, and durable completion evidence. |
| MCPD-023 | Resolve GitHub issue #64 without delaying M4: recognize only the exact bounded current-revision HTTP `400` identity-encoded `application/json` JSON-RPC `-32022` response whose identifier matches, message is a string, requested revision is exactly `2026-07-28`, and supported list contains at most 32 strings without the requested revision. Mark HTTP and envelope performed, fail `protocol.revision` with `MCP-PROTOCOL-002`, and causally skip dependent work across passive and active flows. Retain and render no server message or revision value; do not parse prose, retry, initialize, downgrade, fall back, or add a dependency. Keep malformed, mismatched, contradictory, and over-limit lookalikes as HTTP-contract failures. Finish only after focused exact/negative classifier tests, disposable one-request passive and active journeys, human/JSON/JUnit parity, redaction, complete locked local gates, protected exact-head and exact-`main` evidence, protected merge, issue closure, and durable completion evidence. |
| MCPD-024 | Publish `v0.3.0` without delaying or claiming M4: include completed `MCPD-019` through `MCPD-022` capabilities and the merged `MCPD-023` correction as a backward-compatible minor release. Update only current version, supported-line, release-note, package, installation, and portable README projections while preserving historical immutable evidence. Require the retained exact source/version/release-note guard, complete locked gates, package/install/archive smokes, protected exact-head and exact-`main` CI and release preflight, one intentional annotated exact-`main` tag, immutable GitHub artifacts with checksums/SBOMs/attestations, crates.io OIDC publication, tap-owned exact-byte formula publication, and credential-free represented installed-channel verification. Any source, digest, version, channel, supported-line, or verification mismatch blocks publication or requires a new version. |
| MCPD-025 | Resolve GitHub issue #66 without delaying M4: add `mcp-doctor capabilities [--format human|json] [--schema-version SCHEMA]` backed only by fixed compiled facts. Emit deterministic bounded human output or stable `mcp-doctor.capabilities/v1` JSON covering the exact command/transport/revision matrix, recognized-unsupported revisions, input/output/generator contracts, reporters, `mcp-doctor.exit/v1`, named hard limit profiles, product version, and compile-family process/file controls. Require exact schema selection with no discovery or fallback; return a value-free schema-valid JSON error and exit `2` for unsupported requests. Read no configuration, host inventory, file, environment value, credential, server advertisement, process, DNS, network target, external schema, or tool result. Add no dependency or release claim. Finish only after schema/golden, tri-state consumer and forward-compatibility fixtures, deterministic bounds, secret/proxy/target-like no-activity regressions, represented installed-source smokes, complete locked gates, protected exact-head CI and release preflight, protected merge, issue closure, and durable evidence. |
| MCPD-026 | Resolve GitHub issue #74 without delaying M4: extend the existing exact-path sensitive snapshot workflow only to explicit passive MCP `2025-11-25` and `2025-06-18` inspection. Capture one revision-correct artifact from the same completed bounded conversation, retain selected and negotiated identity, preserve the existing include/exclude, no-clobber, cleanup, size, depth, count, and redaction boundaries, compare only same-revision artifacts offline, and reject cross-revision or incompatible artifacts without coercion or value reflection. Preserve `2025-11-25` default Draft 2020-12 semantics and `2025-06-18` omitted-dialect ambiguity. Add no target replay, active call, retrieval, overwrite, cross-revision inference, score, dependency, release, or compatibility claim. Finish only after `OPEN-14` is accepted and the issue's complete synthetic, secret-exclusion, current-regression, installed-platform, package, locked, hosted, merge, closure, and durable-evidence gates pass. |
| MCPD-027 | Resolve GitHub issue #60 without delaying M4: establish one cohesive revision-parameterized active adapter that preserves exact current behavior, then add explicit MCP `2025-11-25` `check` and `break` over STDIO and Streamable HTTP. Require exact selected and negotiated identity, one initialize and initialized lifecycle, legacy tool result and schema rules, revision-correct HTTP headers, optional session affinity, bounded JSON/SSE, and teardown; never retry, downgrade, fall back, answer server requests, start tasks, or treat discovery as authority. Required task augmentation stops before a call, and required additional input remains incomplete and non-retried. Finish only after `OPEN-15` is accepted and all authorization, finite-work, redaction, cleanup, reporter, manifest, current-regression, compatibility, installed-platform, complete-gate, protected-merge, issue-closure, and durable-evidence checks pass. |
| MCPD-028 | Resolve GitHub issue #61 only after `MCPD-027`: extend its active adapter to explicit MCP `2025-06-18` `check` and `break` while retaining the exact lifecycle, transport, authority, redaction, cleanup, reporting, and no-fallback boundaries. Before generation or `tools/call`, require the exact supported Draft 2020-12 declaration on every advertised schema used to authorize or validate activity; omitted, malformed, unsupported, external, ambiguous, or over-limit contracts produce a typed no-call diagnosis. Do not add tasks, infer a dialect, weaken passive ambiguity reporting, copy the active stack, or broaden compatibility claims. Finish only after the issue's exact-dialect positives and negatives, both transports, reporter and manifest parity, unchanged current and `2025-11-25` journeys, installed-platform, complete-gate, protected-merge, issue-closure, and durable-evidence checks pass. |
| MCPD-029 | Resolve GitHub issue #75 without delaying M4: add a separately authorized current-revision diagnostic for a finite reviewed or deterministic set of well-formed JSON-RPC `tools/call` requests whose arguments are each locally proven schema-invalid for one safe structural mutation. Preserve exact tool, effect, side-effect, target, credential, time, byte, work, sequential-execution, cleanup, and redaction gates; classify documented revision-allowed rejection without matching server prose; distinguish unsafe success or execution, malformed response, transport failure, timeout, crash, and cleanup; and retain only a case index or seed plus a fixed mutation kind. Add no arbitrary fuzzing, load, denial-of-service pressure, automatic retry, dynamic selection, raw argument/result/error retention, legacy claim, or dependency without review. Finish only after `OPEN-16` is accepted, any overlap consumes the settled `MCPD-027` adapter, and all focused, existing-active-regression, reporter, installed-platform, complete-gate, protected-merge, issue-closure, and durable-evidence checks pass. |
| MCPD-030 | Resolve GitHub issue #41 without delaying M4: adopt `DEC-054` as the repository-wide deterministic-CI policy; audit every tracked test, script, and workflow use of clocks, elapsed time, sleeps, polling, timeouts, retries, concurrency, platform tools, and runner assumptions; classify product bounds, outer watchdogs, deterministic state or acknowledgement mechanisms, the exact digest-verified transient acquisition exception, and defects. Correct every blocking defect with a narrow forced-state or state-transition regression, and assign any nonblocking release-only correction an explicit owner and prepublication gate. Preserve failed hosted evidence and require clean exact-head acceptance without rerun. Do not increase a product timeout, weaken an assertion, quarantine a safety test, broadly serialize, add a dependency, change a release artifact or live setting, or alter `MCPD-017`, `MCPD-018`, or another product contract. Finish only when the inventory and operating guidance are current, local gates and first-attempt exact-head hosted checks pass, the protected fix merges, issue #41 closes, exact-`main` evidence passes, and durable completion evidence is recorded. |
| MCPD-031 | Before authorizing the next public version, replace the legacy release paths identified by `MCPD-030`: eight workflow and one verifier `curl --retry 5` uses, two fixed-five-second GitHub/crates.io publication-state loops, and source-checkout jobs that do not yet verify their exact declared runner contract. Observe the required immutable external state itself under one explicit outer deadline, make runner verification precede evidence work, and permit a retry only for classified transient acquisition of exact integrity-verified immutable bytes within the accepted cap. Never retry publication, integrity, a test, job, or workflow. Add deterministic policy regressions and a nonpublishing rehearsal; preserve all immutable `v0.1.0`, `v0.2.0`, and `v0.3.0` bytes and every M4 boundary. Finish only when complete local and first-attempt exact-head hosted gates pass and the next release ticket can cite the corrected path. |
| MCPD-032 | Resolve GitHub issue #65 without delaying M4: add one invocation-local `--limit-profile default|slow-start` selection to `inspect`, `check`, and `break`; keep `default` byte- and behavior-compatible except for reporting its explicit name; let `slow-start` increase only startup, discovery, request, response, and total patience to the exact finite values in `DEC-055`; and preserve the two-second cleanup bound plus every byte, count, retry, redirect, concurrency, target, process, network, credential, side-effect, protocol, and tool-authorization boundary. Reject unknown names before target preparation, retain the selected profile and complete effective values in stable reports and offline aggregates, inventory selection support in compiled capabilities, and add deterministic cross-command, transport, artifact, rejection, and represented installed-source evidence without timing-based acceptance. Add no per-field override, environment or file configuration, unbounded/adaptive mode, fallback, dependency, release, or M4 claim. Finish only when complete local and first-attempt exact-head hosted gates pass, the protected change merges, issue #65 closes, and durable completion evidence is recorded. |

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
| Supported versions | Support only the latest published minor line, currently `0.3.x`. `0.2.x` and earlier are unsupported, and `main` is development-only without a release or backport guarantee. A report about an older version remains welcome, but upgrade may be the resolution. |
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
| Repository inventory | Enumerate every public `EnjoyableWork` repository. `mcp-doctor` is the primary in-scope source and policy repository; `homebrew-tap` is an in-scope supporting distribution codebase only for its `mcp-doctor` policy links, MIT repository license, formula, and release handoff. `mcp-sync` is the active separate public product recorded to prevent hidden scope, not silently covered by this assessment. A new, missing, archived, forked, relicensed, renamed, or otherwise unclassified public repository fails verification for review. |
| Policy ownership and delegation | Keep contribution, conduct, support, defect, and security policies in the primary repository. The tap README identifies its supporting role and routes `mcp-doctor` changes and reports to those exact HTTPS policies rather than duplicating files that can drift. [Tap PR 3](https://github.com/EnjoyableWork/homebrew-tap/pull/3) landed that route as commit [`8d5421a`](https://github.com/EnjoyableWork/homebrew-tap/commit/8d5421abed22e46b43de35f0876bc65edcd6e0d6) without changing a formula, workflow, credential, or release. |
| Public discussion and defects | GitHub Issues is the single public project discussion mechanism. The recognized bug and feature forms collect actionable, safety-bounded evidence; blank issues remain disabled; `SUPPORT.md` names the current release, direct forms, scope, and sensitive-data boundary. No chat room, social account, personal contact, or separate tap tracker is an official support promise. |
| Conduct reports | Use GitHub's private repository **Report content** action, whose live enablement is verified, for a concern attached to project content. Use GitHub Support when the action is unavailable or the concern is immediate or platform-wide. Do not depend on an unpublished personal address, nominal reviewer, or named owner, and do not disclose another person's private information in a public issue. Vulnerabilities retain the distinct private route in `SECURITY.md`. |
| Contributions and inbound license | `CONTRIBUTING.md` remains the recognized workflow and defines ticket, safety, test, pull-request, and dependency expectations. Contributions use the same inbound and outbound OSI-approved MIT terms; no CLA or mandatory DCO sign-off is claimed, while voluntary sign-off remains allowed. |
| Official channels | The canonical set is the HTTPS source repository, issue tracker, private vulnerability form, exact `v0.3.0` GitHub Release, crates.io `0.3.0` package, tap formula, and third-party `docs.rs` documentation mirror. The mirror is not release authority. HTTP, alternate registries or mirrors, personal contacts, and unlisted communication services are not official project channels. |
| Source and released-asset licenses | Require exact MIT metadata and root license hash for source. The immutable `0.3.0` Cargo package and both native archives must contain that exact license; crates.io must report `MIT`; the released and tap formulas must be byte-identical and declare `MIT`; and the tap must contain the same license. Enumerate all seven release assets by exact name, size, and digest so auxiliary SBOM and checksum metadata cannot be mistaken for another software distribution. |
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

The credential-free verifier was deliberately rerun during `MCPD-016` closure
and detected real inventory drift: `courtside-mcp` and `enjoyable-mcp` no longer
appeared in either the public or authenticated all-repository organization
inventory. It failed before any broader claim was made. The current projection
and public guide now enumerate the three live public repositories—`mcp-doctor`,
`homebrew-tap`, and the separate `mcp-sync` project—at canonical SHA-256
`08f301494c59e2a28746029b2a471d43d6ceb1331d5a380ae08176e1eb4a20d8`.
This is a current-scope revalidation, not a claim about whether either missing
repository was deleted, transferred, or otherwise changed, and it does not
rewrite the historical five-repository completion pass above.

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
| Selected Actions | GitHub Actions is repository-enabled only for GitHub-owned Actions plus the exact external Homebrew and Rust-project repositories in the canonical allowlist. The live setting requires every checked-in Action selection to use a full 40-character commit SHA. `DEC-043` removes Anchore's SBOM Action after its pinned bundle was shown to acquire another executable through mutable unpinned code. The canonical inventory therefore closes the direct set at six and records the one nested `actions/attest` implementation selected by the composite provenance Action; an unlisted direct or nested selection, moved tag, unverified selected commit, archived repository, or license-byte drift fails review. Dependabot may propose a new full SHA, but the inventory and upstream review must change in the same pull request before it can pass. GitHub-managed dynamic Dependabot and CodeQL default-setup workflows are separately inventoried provider services, not repository-selected Action refs; the live workflow inventory fails on any additional checked-in or dynamic path. |
| Untrusted code | Only `CI` and `Release preflight` execute a pull request's code. They use only GitHub-hosted ephemeral runners, top-level `contents: read`, non-persisted checkout credentials, no environment, stored secret, secret reference, privileged asset, write permission, `pull_request_target`, `workflow_run`, or untrusted metadata interpolation into shell. Explicit Homebrew token inputs are empty. The ephemeral read-only `GITHUB_TOKEN` used by GitHub to fetch a public pull request is not publication or repository-write authority. Fork approval limits compute abuse but is not treated as a security boundary. |
| Standalone CI executables | The former full-SHA `cargo-deny-action` still fetched a mutable release executable without checking a digest, so its commit pin did not authenticate the executed tool. Required CI instead downloads only `cargo-deny` `0.20.2` for `x86_64-unknown-linux-musl` over bounded HTTPS, requires exact SHA-256 `9f12ed4c49936e09b48bf862b595cde2fe64fcbd9d74dfacac6131ca824c8d5f` and the five-entry reviewed archive layout, then executes the reported exact version from a disposable runner path. `MCPD-016A` applies the same boundary to current Syft `1.51.0`: only immutable GNU/Linux ARM64/x64 release assets may run, under exact size, SHA-256, four-entry layout, reported-version, and reported-platform checks. At most three 20-second attempts separated by one second may acquire the same asset after curl `6`, `7`, `18`, `28`, `52`, `55`, `56`, or `92`, or HTTP `408`, `429`, `500`, `502`, `503`, or `504`; partial bytes are deleted. TLS/trust errors, other responses, integrity or layout failures, tool execution, generation, validation, build, test, publication, job, and workflow failures never retry. The `cargo-deny` upstream release is mutable, so any API digest, asset, tag, layout, version, or selected-tool change blocks for renewed review; immutable Syft drift does the same. |
| Source artifacts | Source control admits reviewable regular UTF-8 text source only. Executable mode is reserved for shebang-bearing `scripts/*.sh`; generated executables, libraries, packages, archives, NUL-bearing or non-UTF-8 content, disallowed ASCII controls, executable/archive/document/binary-media signatures, Git LFS pointers, symlinks, and control-character paths fail. There are no binary exceptions. The negative rehearsal proves a normal reviewable tree passes and generated ELF, NUL-bearing, non-UTF-8, and extension-disguised executable cases fail in a disposable repository. Release and testing-tool bytes stay outside Git history. |
| Published distribution | Authenticate only canonical immutable `v0.3.0`: require the annotated tag object and source commit, immutable non-draft GitHub Release, exact seven-name/size/digest asset set, GitHub release verification, and every asset's signed attestation constrained to the source repository, `release.yml`, tag ref, and source commit. The crates.io package must be unyanked, MIT-declared, and byte-identical to the attested release crate. Current tap `main` must remain the reviewed commit and its formula must be byte-identical to the attested release formula, name the exact immutable GitHub Release package URL and digest, and declare MIT. Verification downloads to a mode-private disposable root and performs no publish, release, tag, formula, or package write. |
| Live policy and evidence | The operator audit requires exact clean local and remote `main`, selected-Action and SHA-pinning settings, read-only default token, no approval authority, the recorded fork policy, no repository or applicable organization Actions secret, closed upstream Action identities and licenses, the digest-recorded standalone tools including Syft's latest-release contract, reviewable exact source tree, and authenticated distribution equality. It emits only UTC date, canonical SHA-256, exact source SHA, release tag, and `PASS` or `FAIL`; API bodies and downloaded artifacts are deleted. A failure is investigated privately and blocks completion. |

The 2026-08-12 pre-activation audit found the existing weekly Cargo and Action
version groups, exact direct requirements, locked graph, full-SHA direct Action
uses, read-only workflow default, two secretless pull-request workflows,
read-only fork policy, empty repository Actions-secret inventory, immutable
release attestations, and byte-identical Cargo and Homebrew handoffs already in
place. It also found three gaps that prevent an achieved claim: security updates
were not explicitly grouped, repository policy still allowed any Action and did
not enforce SHA pinning, and the full-SHA `cargo-deny` Action downloaded its
executable without digest verification. At that pre-activation point, no
Dependabot proposal existed for this repository, so configuration inspection
could not replace a real grouped proposal and review.

The initial direct and nested checked-in Action review on 2026-08-12 resolved every recorded tag
to its exact canonical commit. All eight then-selected repositories were public, active, and
unarchived; every selected commit had GitHub's verified-signature result; and
the exact MIT, BSD-2-Clause, Apache-2.0, or dual MIT/Apache-2.0 license bytes in
the canonical inventory matched. Roles and inputs were narrowed: checkout never
persists credentials, artifacts are named same-run handoffs retained for one
day, Homebrew token inputs are empty, the then-selected Anchore Action requested exact Syft `1.50.0` with no
dependency snapshot or implicit upload, provenance delegates only to the
recorded nested SHA, and crates.io authority exists only in protected OIDC
publication jobs. Several Node Actions execute generated JavaScript bundles;
the public source commit, upstream tag, license, focused configuration, and full
pin make changes reviewable but do not independently reproduce or prove those
bundles benign. That limitation remains in the canonical result.
`MCPD-016A` later established that the Action pin did not cover its fetched
`main/install.sh` or that installer's release lookup, so `DEC-043` removes the
Action rather than retaining its full-SHA pin as false executable provenance.
GitHub's separately inventoried dynamic Dependabot and CodeQL workflows do not
take their internal Action refs from this repository; current official GitHub
documentation also states that public-Action restriction policies do not govern
CodeQL default setup. `MCPD-014` continues to verify the latter through its
configured setup, exact-main analyses, and clean alert surface instead of
mislabeling provider-selected tags as project full-SHA pins.

The first post-activation provider runs opened grouped Cargo
[PR 26](https://github.com/EnjoyableWork/mcp-doctor/pull/26) and Action
[PR 27](https://github.com/EnjoyableWork/mcp-doctor/pull/27) without approval,
merge, write-token, stored-secret, or auto-merge authority. The dated
[Cargo review](https://github.com/EnjoyableWork/mcp-doctor/pull/26#issuecomment-5268391783)
rejected exact `base64` `=0.23.1`: defaults stayed disabled and only `alloc`
was selected, so the new default-on unsafe SIMD remained off; ordinary native
behavior passed; the active two-owner upstream, dual license, unyanked package,
checksum, Rust `1.71`, platform reach, no-build-script boundary, and advisory
state were reviewed; but unsigned tag and commit provenance plus a second
`base64` line retained by `reqwest`, `hyper-util`, `pem`, and `rcgen` added cost
without a needed capability and failed the duplicate-version ban. The proposal
was closed without an ignore rule, leaving later convergence or a security
update eligible for fresh review.

The dated [Action review](https://github.com/EnjoyableWork/mcp-doctor/pull/27#issuecomment-5268400437)
accepts only `Homebrew/actions/setup-homebrew` immutable release
`2026.08.10.1` at verified commit
`c8707045ccae42888fe98e86f2ee8938bc7cc193`. The public active Homebrew
repository, all 15 intervening verified commits, unchanged 1,334-byte
BSD-2-Clause license at SHA-256
`f80329e58613ad669c0e73cb132d8060b9b2c55e339c73848068e4d1567f4627`,
zero published repository advisories, release notes, ownership, inputs, and
implementation diff were reviewed. The selected subtree removes only the
unused `setup-sandbox` input and privileged Bubblewrap setup path; it changes no
package lock, dependency, nested Action, build script, generated artifact, Rust
toolchain, or runtime requirement. Later immutable `2026.08.10.2` changes only
the upstream test workflow and has the same selected subtree. The proposed SHA
passes Linux x64 and ARM64, macOS ARM64, and Windows source/package/formula
journeys with token inputs empty. Updating both workflow uses and this canonical
inventory in PR 27 is mandatory; green behavior alone cannot bypass the closed
pin.

This contract contributes the five OSPS `v2026.02.19` Level 1 rows assigned
below. Protected [PR 25](https://github.com/EnjoyableWork/mcp-doctor/pull/25)
landed the controls; the selected-Action and SHA settings are active;
both grouped proposals above received complete reviews; disposable fork
[PR 29](https://github.com/EnjoyableWork/mcp-doctor/pull/29) proved read-only
permissions, absent repository/OIDC credentials, absent persisted checkout
authority, and a rejected write before closing unmerged; and exact-main commit
`40234363e8a1764498b524bc86c39afff0584355` first passed the operator audit.
Protected PR 27 then accepted the reviewed Action pin, and the completion
evidence below repeats every required hosted and operator gate on its exact
merged `main`.

### MCPD-016 completion evidence

`MCPD-016` completed on 2026-08-12 with canonical supply-chain SHA-256
`ea63855124cae11a0230aabc982c5c722b2154876133b7437e2c72a0a1b69ef5`.
This is a dated repository dependency, CI, tracked-source-artifact, and exact
`v0.2.0` distribution result. It is not a complete OSPS result, independent
certification, regulatory-compliance claim, reproducible proof of every
upstream Action bundle, product security-scanner result, or warranty.

The protected and independently reviewable evidence is:

- [activation PR 25](https://github.com/EnjoyableWork/mcp-doctor/pull/25)
  introduced separate grouped Cargo and Action version/security proposals,
  exact direct and feature-graph regressions, selected full-SHA repository
  policy, a closed direct/nested Action inventory, digest-authenticated
  `cargo-deny`, the source-artifact gate and negative rehearsal, and the bounded
  operator audit. It merged through all 15 checks as exact `main`
  `d145244b4da993fea17c345e2742220944af7f53`; repository auto-merge remained
  disabled, workflow default authority remained `contents: read`, and
  Dependabot retained no review-approval authority;
- grouped Cargo [PR 26](https://github.com/EnjoyableWork/mcp-doctor/pull/26)
  was rejected after its dated
  [complete review](https://github.com/EnjoyableWork/mcp-doctor/pull/26#issuecomment-5268391783)
  found an unnecessary duplicate dependency line despite exact narrow features,
  ordinary behavior, acceptable maintenance, license, advisory, Rust, and
  platform evidence. It closed without an ignore rule or merge. Grouped Action
  [PR 27](https://github.com/EnjoyableWork/mcp-doctor/pull/27) was accepted only
  after its dated
  [complete review](https://github.com/EnjoyableWork/mcp-doctor/pull/27#issuecomment-5268400437)
  and same-PR inventory update. Exact head
  `d11e8378999c057a74a18a83767179d220897897` passed
  [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31611427951),
  [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31611421352),
  and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31611427635)
  before a normal protected squash merge as
  [`5cdc032`](https://github.com/EnjoyableWork/mcp-doctor/commit/5cdc032336ca5e9cc2dba3c0052eff36be0fc83c);
- disposable fork [PR 29](https://github.com/EnjoyableWork/mcp-doctor/pull/29)
  at exact fork commit `4ba3f5121c3810c1e9dc7bd4bc0ee492afb4de93`
  passed [read-only CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31609790299).
  Its [dependency job](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31609790299/job/94157892254)
  proved only read metadata/content authority, no repository or OIDC token
  environment, no persisted checkout credential, and a rejected unauthenticated
  write to a synthetic ref before source verification continued. The draft PR
  closed unmerged and its exercise branch was removed. The personal fork retains
  only a mirror of `main` because available authority could not delete the fork;
  it has no organization setting, secret, or merge authority and is not used as
  security evidence beyond the closed run. The closure audit compared the
  immutable run head with the PR commit record and protected
  [PR 39](https://github.com/EnjoyableWork/mcp-doctor/pull/39) corrected a
  transposed recorded identifier before the final claim;
- the source gate verified 111 reviewable regular UTF-8 source files and no
  generated executable or binary artifact. The disposable rehearsal separately
  rejected an ELF executable, NUL-bearing content, invalid UTF-8, and an
  extension-disguised executable. Exact-main
  [dependency policy](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31612642595/job/94167581709)
  repeated the source, exact graph, digest-authenticated tool, advisory, license,
  ban, and source checks;
- on exact `main` `5cdc032336ca5e9cc2dba3c0052eff36be0fc83c`,
  [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31612643730)
  passed Actions and Rust,
  [Required CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31612642595/job/94168634038)
  passed dependency policy and all three native quality hosts, and
  [Required release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31612642612/job/94171302909)
  passed deterministic Cargo/formula generation, four source-install journeys,
  both represented GNU/Linux archives and SPDX documents, and exact
  non-publishing payload equality; and
- public and authenticated protection readbacks both emitted
  `date=2026-08-12 canonical_sha256=2e3377a5101c513c02bb177cbc95acc3707f77bab4c3ab8ed3e8576a3f828794 result=PASS`,
  while the scoped security readback emitted
  `date=2026-08-12 canonical_sha256=d379f2c86b9571da14cdb9c51cfc83075f098688a4660aecb67eb60fa385e66a result=PASS`.
  The unchanged operator audit emitted only
  `date=2026-08-12 canonical_sha256=ea63855124cae11a0230aabc982c5c722b2154876133b7437e2c72a0a1b69ef5 source_sha=5cdc032336ca5e9cc2dba3c0052eff36be0fc83c release=v0.2.0 result=PASS`.
  It authenticated live selected-Action and permission settings, every reviewed
  Action/tag/commit/license identity, exact `cargo-deny`, the source tree,
  immutable release and all attestations, byte-identical crates.io package, and
  byte-identical tap formula without changing a published byte.

Protected [closure PR 38](https://github.com/EnjoyableWork/mcp-doctor/pull/38)
carries the current three-repository community rebaseline and this status;
protected [evidence correction PR 39](https://github.com/EnjoyableWork/mcp-doctor/pull/39)
binds the exact fork identity found by the completion audit. Both may merge
only after both required aggregates and CodeQL pass. PR 39's public timeline is
the durable record for final exact-`main` CI, release preflight, CodeQL,
protection, security, credential-free community/license, and supply-chain
operator readbacks. `MCPD-017` entered active delivery under the activation-only
`DEC-041` and `DEC-042` contracts, but is temporarily Blocked on the
`MCPD-016A` correction below; no achieved organization-access or
private-recovery claim is implied.

### MCPD-016A Syft acquisition correction

Exact `main` `219ca52901373dea3549ac75cce7c09696ab789f` passed CI and
CodeQL after PR 42, but its
[release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31641032348)
failed only after the GNU/Linux ARM64 build, source installs, deterministic
archive, and smoke journey had passed. The full-SHA
`anchore/sbom-action` `0.24.0` downloaded Syft's unpinned
`main/install.sh`; that script performed another live release-page lookup for
the already selected `v1.50.0`, and GitHub returned `503`. The same source path
passed on the pull request and GNU/Linux x64 host. A rerun would therefore hide
the uncontrolled acquisition boundary rather than provide acceptance evidence.
This drift reopens `RISK-14` without invalidating immutable `v0.2.0` bytes or
diagnostic behavior.

The current-version review changed the original recommendation before it was
committed. Syft `1.51.0`, published 2026-08-10, superseded `1.50.0` and
remediated high `GHSA-hc8v-wwc9-vgxm` plus medium
`GHSA-qgq7-7hm3-q39j` in `go-git`; it also fixed temporary-directory cleanup.
Keeping the older binary would knowingly introduce a stale standalone tool, so
`DEC-043` selects `1.51.0` while preserving the same two-target SPDX role. The
2026-08-12 review found public, active, unarchived Apache-2.0 `anchore/syft`;
immutable non-draft, non-prerelease `v1.51.0`; annotated tag object
`57260929138ad516dd4999a5cc43b4a295d2461f` resolving to source commit
`2293641e3bd628a01bb37639318d62c0ebe89b39`; and these exact release assets:

| Target | Exact asset | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| `x86_64-unknown-linux-gnu` | `syft_1.51.0_linux_amd64.tar.gz` | 28,743,977 | `2a2e837a2c8d59ec9af5472ee22d3b04ee463c4e44476ecf993fd1e5ab6ebc7f` |
| `aarch64-unknown-linux-gnu` | `syft_1.51.0_linux_arm64.tar.gz` | 26,261,269 | `6c0466811541ea03add5213a60a1562f0851e4c0b0ecfdee1a694a9455285900` |

The tag object itself is unsigned; the resolved source commit has GitHub's
valid verified result, the GitHub Release is immutable, and the canonical
operator audit independently re-reads tag, source-verification state, license,
release, asset size, and provider digest. Anchore's active issue and release
history and private security route are credible, but its policy supports only
the latest release and explicitly excludes purposely malicious scan content.
The two published Syft advisories affect releases before `1.42.3` and the
`0.69.x` line, not `1.51.0`; any new release, advisory, ownership, or support
change therefore triggers renewed review rather than an automatic update.

The selected release uses Go `1.26.3`; its reviewed GoReleaser configuration
sets `CGO_ENABLED=0` and produces static Linux binaries for both selected
architectures. The provider's exact ARM64 release SBOM contains 276 packages
and 827 relationships, so the prebuilt tool retains a materially broader
transitive and possible `unsafe` surface than this repository independently
audits. Exact immutable identities, no native C linkage, a disposable runner,
one fixed repository-generated archive, and explicit disabling of known
network-backed and host-cache catalogers bound that residual; they do not prove
every upstream package benign or provide kernel-level network isolation. This
correction changes no Cargo dependency, feature, build script, Rust floor,
product runtime, or published byte. It removes one Node Action and its mutable
installer path while retaining a roughly 26–29 MB ephemeral download, SPDX 2.3
output, two-target scope, and separate output validation.

Repository-owned `scripts/install-syft.sh` accepts only those two GNU/Linux
assets and verifies exact size, SHA-256, four-entry archive layout, reported
version, and reported platform before execution. It deletes partial bytes and
permits no more than three attempts under the finite `DEC-043` transient
allowlist. `scripts/generate-release-sbom.sh` executes the verified binary in a
minimal environment with update checks, network-backed cataloging, host caches,
and ambient configuration disabled; bounds its input, output, error output,
processor count, and one 120-second generation attempt; keeps failed or
oversized output private to a disposable root; and never retries generation.
The offline rehearsal forces every supported mapping,
transient and permanent class, exhaustion, equal-size wrong-digest bytes, wrong
layout, version and platform, unsupported host, successful SPDX generation, and
non-retried generation and oversized-output failures without a network call or
wall-clock wait.

`MCPD-016A` is Done. Protected [PR 43](https://github.com/EnjoyableWork/mcp-doctor/pull/43)
merged the correction as exact `main`
`5a5efd1bf4a911b446c305d063455a12b00cba80`. Its exact head
`bf5dad6682ce6dd541afabcbf870406a34589cc8` passed
[CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31648861587),
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31648864252), and
[release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31648864241)
with `run_attempt: 1` for each and no workflow or job rerun. The final preflight
included native GNU/Linux ARM64 and x64 source-install, archive, direct Syft
acquisition, SPDX generation, validation, and cross-artifact payload checks.

The merge then passed exact-`main`
[CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31654117788),
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31654118100), and
[release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31654118076),
again with `run_attempt: 1` for every workflow and no rerun. Only after the
merged workflows stopped selecting the former Action, the live allowlist was
narrowed and read back as GitHub-owned Actions plus exactly
`Homebrew/actions/*@*` and `rust-lang/crates-io-auth-action@*`, with arbitrary
verified-publisher Actions disabled. The authenticated operator verifier then
emitted only
`date=2026-08-13 canonical_sha256=aa7aa82886b2e282c66c55263161ab6466eccd0632777500e4b5c45f736a6e25 source_sha=5a5efd1bf4a911b446c305d063455a12b00cba80 release=v0.2.0 result=PASS`.
It re-authenticated the live policy, every Action and standalone-tool identity,
both immutable Syft assets, and unchanged `v0.2.0` release, Cargo, and Homebrew
bytes without publishing or changing an artifact. The broader deterministic-CI
policy and timing audit remain separately tracked by issue 41.

### Accepted organization access, credential, continuity, and recovery contract

`DEC-041` resolves `OPEN-10` and fixes the `MCPD-017` boundary. The owner
accepted choices `1B`, `2A`, and `3A` on 2026-08-12: retain one real owner
rather than add a nominal privileged account; move organization administration
and automation away from classic personal access tokens toward reviewed
short-lived authority with at most 30-day exceptional fine-grained access; and
exercise a real private recovery path every six months. The checked-in
`.github/organization-controls.json` is the non-sensitive canonical projection,
`scripts/verify-organization-controls.sh` is the authenticated exact-main
verifier, and `scripts/rehearse-organization-controls.sh` proves its important
failure and non-disclosure paths without a credential.

`DEC-042` resolves `OPEN-11`, which the live activation discovered after
`DEC-041` was accepted. GitHub documents the native control that restricts
[outside-collaborator invitations to organization owners](https://docs.github.com/en/enterprise-cloud@latest/organizations/managing-organization-settings/setting-permissions-for-adding-outside-collaborators)
as a GitHub Enterprise Cloud feature. The live organization reports plan
`free`, exposes no supported control for changing that setting, and reports the
API field `members_can_invite_outside_collaborators` as `true`. The approved
version-2 projection records that value as an unavailable platform capability,
not desired authority. It fails closed unless the one member is also the one
owner, the outside-collaborator and pending-invitation counts are zero, and no
non-owner has direct repository-administrator authority. GitHub classifies a
nonmember with organization-repository access as an outside collaborator, so
the complete member, owner, and outside-collaborator inventories prove that
zero conclusion without granting the verifier access to unrelated private
repositories. Any plan, native-field, membership, invitation, or
outside-collaborator change requires review. If GitHub makes the native
restriction available, it must be enabled and this compensating contract must
be superseded.

| Choice | Accepted policy |
| --- | --- |
| Scope | Authentication, membership, member privileges, installed applications, personal-access-token policy, organization Actions credentials, ownership continuity, and recovery are organization-wide controls. Repository-specific credential assertions cover the `mcp-doctor` source repository and its `homebrew-tap` distribution repository. A separate repository's project-specific secret or variable is not silently relabeled as `mcp-doctor` assurance, but an organization-wide credential or application that can reach either in-scope repository remains in scope. Private repository names are never public evidence. |
| Strong MFA | Keep organization 2FA required and permit only GitHub's documented [secure methods](https://docs.github.com/en/organizations/keeping-your-organization-secure/managing-two-factor-authentication-for-your-organization/requiring-two-factor-authentication-in-your-organization). Require zero current members, outside collaborators, or billing managers with disabled or insecure 2FA. GitHub's API proves the organization requirement plus aggregate disabled/insecure member and outside-collaborator counts; because it does not expose the secure-method enforcement switch or billing-manager factor state, the owner separately confirms those UI-only aggregates without recording an identity or factor inventory. Any disabled enforcement or nonconforming account fails. |
| Membership and repository authority | Keep exact base repository permission `none`; add membership only through an owner-reviewed manual grant; give the lowest role required for an identified task; and retain no non-owner direct administrator. With one owner selected, any added member, owner, outside collaborator, billing manager, or pending invitation is decision drift until deliberately reviewed. Disable every GitHub Free-configurable future-member privilege. Because the native outside-collaborator invitation restriction is Enterprise Cloud-only, require the `DEC-042` compensating invariant that every repository administrator is the sole organization owner. The complete organization member, owner, and outside-collaborator inventories prove that invariant while they remain one, one, and zero; the verifier does not broaden its private-repository scope merely to re-enumerate the same identity universe. Organization owners remain capable of administration, so the policy is an auditable owner boundary rather than a claim of technical owner incapacity. |
| Installed applications | Enable GitHub's [owner-only installation restriction](https://docs.github.com/en/organizations/managing-programmatic-access-to-your-organization/limiting-oauth-app-and-github-app-access-requests-and-installations); members may submit requests but outside collaborators may not, and a request grants no authority until owner review. Every installation uses selected repositories, never automatic access to every current and future repository. A fresh private review records the exact stable App identity, requested permissions and event subscriptions, selected repositories and single-file paths, installation update time, suspension state, need, owner decision, and normalized inventory SHA-256. The organization-owner API independently verifies the aggregate count, identity, permissions, events, update time, selected-versus-all state, single-file grants, and suspension state; GitHub does not provide an owner token an API route to enumerate another App's exact selected repositories, so that mapping remains an explicit private UI-attested limitation rather than a false live-API claim. A new App, permission, event, repository or single-file selection, all-repository grant, suspension, removal, update, or changed need fails for review. Neither public configuration nor output contains the digest, an App name, or a private repository name, so it cannot confirm guesses about the private inventory. |
| Automation credentials | Normal automation uses only job-scoped `GITHUB_TOKEN`, OIDC, or one-hour GitHub App installation tokens. Under GitHub's [organization PAT policies](https://docs.github.com/en/organizations/managing-programmatic-access-to-your-organization/setting-a-personal-access-token-policy-for-your-organization), block classic PAT access to the organization. Interactive verification uses only an owner-reviewed exceptional fine-grained PAT: resource owner `EnjoyableWork`; exactly the `mcp-doctor` source and `homebrew-tap` distribution repositories; the canonical read-only organization and repository permission profile; at most 30 days; no unattended automation; and revocation when the task ends. Unrelated repositories are deliberately excluded. User-wide OAuth authority is excluded because it cannot be restricted to only `EnjoyableWork`; a GitHub App user token is excluded because it can inspect only installations of its issuing App. Organization Actions/Dependabot secrets, variables, webhooks, and self-hosted runners remain empty. The source and tap retain no Actions, Codespaces, Dependabot, or environment secret, variable, deploy key, or webhook; because GitHub requires write permission even to list repository Codespaces secrets, their zero counts receive fresh private owner review instead of expanding verifier authority. Separate-project credentials require their own private project review and cannot reach the source or tap through an organization-wide grant. |
| Owner continuity | Retain one active human owner and prohibit a shared, unattended, or nominal owner account. This deliberately accepts the residual availability risk identified by GitHub's [two-owner continuity guidance](https://docs.github.com/en/organizations/managing-peoples-access-to-your-organization-with-roles/maintaining-ownership-continuity-for-your-organization): the organization can become unavailable when that owner is unreachable. A genuinely trusted active second operator triggers a new decision rather than an automatic privilege grant. The exact aggregate owner count and accepted residual-risk assertion are rechecked without publishing identity. |
| Recovery | Every six months, and after an owner or authentication-factor change, the owner privately exercises an independent organization-account recovery path; GitHub's [account-recovery policy](https://docs.github.com/en/site-policy/other-site-policies/github-account-recovery-policy) makes existing recovery methods the decisive boundary when 2FA access is lost. If single-use material is consumed, rotate it before recording success. Evidence contains only date, aggregate owner count, scope, and `PASS`; identities, factor inventory, and recovery material are prohibited. Missing, failed, future-dated, older-than-184-day, or merely preparedness-only evidence fails. The verifier consumes a mode-private aggregate owner attestation and deletes its temporary API projections on exit. |
| Verification and maintenance | The canonical file moved from activation to verified only after the reviewed aggregate App count and latest recovery date replaced their explicit nulls; the exact App inventory and digest remain only in the private attestation, and verified configuration is not ticket-completion evidence until the protected exact-`main` run passes. The live verifier binds a clean local exact `main`, uses direct GitHub API `2026-03-10` requests with no ambient proxy or trust override, pins the API timezone to UTC and normalizes only valid UTC App timestamps, enforces finite per-response, request-count, connect, request, total-time, installation, environment, and private-inventory limits, compares every API-observable App field and in-scope credential count, and accepts only the exact two-repository organization-scoped fine-grained operator profile. It derives the zero non-owner-administrator conclusion from the complete member, owner, and outside-collaborator identity inventories rather than requesting unrelated private-repository access. It requires fresh mode-private confirmation of the operator's exact repository scope, App repository mapping, Codespaces-secret counts, and the other named UI/private limitations. It emits only UTC date, canonical SHA-256, source SHA, and `PASS` or `FAIL`. CI runs only synthetic fixtures under an explicit fixed fixture-only reference date that live mode rejects: fixture output is explicitly labeled `mode=fixture` and bound to a non-main sentinel SHA, while insecure MFA, broad defaults or member privileges, organization-plan or outside-invitation drift, a non-owner repository administrator, all-repository or changed Apps, non-UTC App timestamps, private mapping drift, unsupported operator authority, organization secrets, a tap deploy key, and missing or stale recovery all fail without exposing the synthetic private identifier. Review every 90 days and after any listed control changes. |

Activation research corrected the initial verifier assumption before it became
public evidence. GitHub documents `GET /user/installations/{installation_id}/repositories`
for a GitHub App user access token and describes `GET /user/installations` as
listing installations of the token's own GitHub App; the current classic PAT is
also rejected by that route. Therefore no single owner credential can use it to
enumerate the exact repository grants of unrelated installed Apps. The verifier
now cross-checks the owner-readable installation metadata against a bounded,
fresh private inventory and treats only the exact repository mapping as manual
evidence. This also preserves the owner's explicit requirement that verifier
authority reach only `EnjoyableWork` rather than silently substituting a
user-wide OAuth token.

The 2026-08-12 read-only activation audit found meaningful existing strengths:
organization 2FA is required; no current member or outside collaborator appeared
in either the disabled-2FA or insecure-2FA filters; base repository permission is
`none`; member repository creation is disabled; and there is one owner, no
outside collaborator, no pending invitation, and no non-owner direct
administrator. Organization and in-scope repository Actions and Dependabot
secret inventories are empty, there is no organization runner or webhook, and
the source repository has no in-scope stored credential.

The owner then explicitly authorized and completed the staged live activation.
API-supported future-member repository, Pages, team, issue, visibility,
deletion, and private-fork privileges are disabled; the GitHub App installation
control is restricted to owners; App requests are limited to members; three
retained installations use reviewed selected-repository scopes with no
all-repository or suspended installation; and the obsolete write-capable tap
deploy key was deleted after its job-scoped `GITHUB_TOKEN` replacement was
revalidated. Readback reports zero tap deploy keys. The owner confirmed
secure-method enforcement with zero billing managers, while the API reports
required organization 2FA and zero disabled or insecure members or outside
collaborators. The remaining outside-collaborator invitation field is the
GitHub Free platform limitation governed by `DEC-042`, not an unrecorded
successful toggle.

OAuth App access is restricted. Fine-grained PAT access requires owner approval
and a maximum 30-day lifetime, while classic PATs are blocked from every
EnjoyableWork API and Git-over-HTTPS resource. The broad pre-existing classic
credential may remain available to the owner's unrelated boundaries, but a live
negative readback proves that it receives `403` from the organization. The
temporary verifier credential is owned by `EnjoyableWork`, expires within 30
days, has the exact canonical read-only permission profile, reaches only the
source and tap, succeeds across the required organization and repository APIs,
and receives `404` for an unrelated private repository. It remains private,
non-automated, and scheduled for revocation after the exact-`main` run.

The bounded private App inventory and digest match all API-observable App
identity, permission, event, update-time, selection, single-file, and suspension
fields plus the owner-reviewed exact repository mappings and retain decisions.
Fresh owner UI review reports zero Codespaces secrets for both the source and
tap and no organization Codespaces secret available to either. A clean private
browser session then signed in through an existing passkey without the normal
password or 2FA path and reached the organization owner settings; the private
session was closed, no single-use material was consumed, and the aggregate
recovery evidence records only the date, one-owner count, scope, and `PASS`.
One environment secret on the separately classified public product and one
variable on a private repository remain outside the repository-specific
`mcp-doctor` claim; neither is treated as permission to ignore any
organization-wide reach.

The canonical lifecycle is now verified with the reviewed aggregate App count
and recovery date, and the mode-private attestation is complete. The live API
also exposed localized fractional App timestamps, so the verifier now requests
UTC explicitly, canonicalizes only bounded valid UTC timestamps, and rejects a
non-UTC negative. `MCPD-017` remains In progress because the changed verifier
cannot truthfully produce its exact-`main` result until this protected change
merges; the temporary credential must then be revoked and the aggregate result
recorded. This accepted policy and pre-merge evidence are not an achieved OSPS
result, complete M4 baseline, certification, regulatory-compliance claim, or
assurance statement about a separate product.

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
| DEC-041 | Resolve `OPEN-10` with strong MFA, lowest-default access, owner-reviewed short-lived authority, explicit single-owner risk, and private recovery proof | Accepted | 2026-08-12 | Retain one real human owner without adding a nominal privileged account; require manual least-privilege grants, owner-only selected-repository Apps, job tokens/OIDC/App installation tokens for normal automation, blocked classic PATs, and owner-approved exact-scope fine-grained PAT exceptions lasting at most 30 days; privately exercise independent recovery every six months and after owner or factor changes; and publish only aggregate non-sensitive verifier evidence |
| DEC-042 | Resolve `OPEN-11` when GitHub Free cannot restrict outside-collaborator invitations to owners | Accepted | 2026-08-12 | Record the native Enterprise Cloud-only restriction and the live Free-plan `true` API field as an unavailable capability rather than desired authority; compensate by requiring the sole member to be the sole owner, zero outside collaborators and pending invitations, and therefore zero non-owner direct repository administrators; prove the identity universe without granting verifier access to unrelated repositories, fail on plan, setting, or access drift, and replace the compensation with the native restriction if it becomes available |
| DEC-043 | Replace indirect mutable Syft acquisition with exact immutable assets and transient-only bounded retries | Accepted | 2026-08-12 | Remove `anchore/sbom-action`; replace the superseded `1.50.0` selection with current security-remediating Syft `1.51.0`; acquire only its GNU/Linux ARM64/x64 assets by exact immutable URL, size, SHA-256, four-entry layout, version, and platform; make no more than three 20-second attempts separated by one second only for curl `6`, `7`, `18`, `28`, `52`, `55`, `56`, or `92`, or HTTP `408`, `429`, `500`, `502`, `503`, or `504`; delete partial bytes; never retry trust, integrity, execution, generation, validation, build, test, publication, job, or workflow failure; preserve SPDX and release scope; and require first-attempt exact-head and exact-`main` evidence |
| DEC-044 | Resolve `OPEN-12` with explicit revision-selected passive legacy adapters | Accepted | 2026-08-13 | Supersede the latest-only part of `DEC-013` and `DEC-024` only for `inspect`: keep MCP `2026-07-28` as the default and sole `check`/`break` revision; permit exact `--protocol-version 2025-11-25` or `2025-06-18` selections with no discovery, negotiation fallback, retry, or downgrade; send each revision's initialize and exactly one initialized notification, then only capability-advertised tool, prompt, resource, and resource-template list operations, never the potentially value-bearing retained-task list; use revision-specific envelopes and pagination; fully validate omitted-dialect `2025-11-25` schemas as locally bounded JSON Schema 2020-12, while an omitted `2025-06-18` dialect receives bounded structural/reference checks and an explicit ambiguity warning rather than guessed semantics, with exact Draft 2020-12 declarations enabling full validation; never retrieve an external reference; and require exact selected/negotiated reporting. For legacy Streamable HTTP, omit the protocol header only on initialize, require it thereafter, retain and verify an optional bounded visible-ASCII session identifier, accept the bounded `2025-11-25` empty SSE priming event, complete a matching SSE response without waiting for EOF, diagnose session loss causally without reinitializing, and attempt one shutdown-grace-bounded DELETE teardown; successful, unsupported, and already-absent termination close safely, while every other teardown failure remains an independent safety finding. Stable report `v1` gains compatible optional revision fields; broad legacy positioning waits for the controlled official/independent two-language and installed-channel evidence. |
| DEC-045 | Resolve `OPEN-13` with exact-opt-in sensitive current-revision snapshots and conservative offline comparison | Accepted | 2026-08-13 | Permit snapshot creation only after a bounded passive MCP `2026-07-28` discovery and complete catalog conversation finish and cleanup succeeds, including when a bounded local schema shape retained by the artifact makes the ordinary report fail, and require exact matching output and acknowledgement paths; write one new bounded regular file from that same conversation with owner-only Unix mode and never overwrite; retain normalized capabilities, catalog identities, required inputs, protocol-defined tool hints, bounded local schema structure, and artifact-local ordinal correlation while excluding descriptions/defaults, transport/runtime/credential/log data, and all ordinary-report identifiers; compare two independently bounded local artifacts with deterministic human or schema-backed JSON, stable addition/removal/capability/required-input/narrowing/widening/review codes and causal performed/skipped state; reject unsupported versions, external references, malformed correlation, or exhausted bounds without echoing values; and add no target activity, retrieval, dependency, digest/token, score, general implication claim, or publication path |
| DEC-046 | Resolve issue #72 with explicit one-result JSON and JUnit artifact fan-out | Accepted | 2026-08-13 | Keep `--format` as the byte-compatible stdout selector and add only `--json-report PATH` and `--junit-report PATH` to `inspect`, `check`, and `break`; preflight new-file destinations and actual aliases before any target, network, credential, discovery, or tool activity; construct one immutable redacted result; render stdout and at most two file projections under unchanged four-MiB individual and eight-MiB combined bounds; persist complete exclusive same-directory stages, with Unix owner-only mode and platform ACL inheritance elsewhere, through no-clobber publication with all-output rollback and visible cleanup failure; preserve diagnostic exit only when every requested output succeeds; and add no rerun, retry, concurrency, overwrite, arbitrary format/path parser, new reporter, CI-provider integration, upload, report merge, dependency, or immutable release change |
| DEC-047 | Resolve issue #73 with normalized conservative offline aggregation of stable reports | Accepted | 2026-08-13 | Add one explicit ordered local-only aggregate command whose required new JSON destination and selected human/JSON stdout derive from one validated normalized set of one through 32 `mcp-doctor.report/v1` inputs; identify members only by ordinal; accept but ignore compatible unknown optional values, preserve grammar-safe unknown codes and every known safe field, verify schema plus semantic consistency, and make any failed member fail the aggregate, otherwise any incomplete member incomplete it, with no suppression, waiver, score, deduplication, baseline, or majority rule. Reject symlink and file aliases, including through a no-follow open and fail-closed complete Windows handle identity, all malformed/incompatible/inconsistent or finite-bound-exhausting input, and unsafe output before presenting evidence; stage and publish without clobber with rollback and visible cleanup failure; perform no process, network, DNS, credential, retrieval, target, discovery, generation, or tool activity; and add no dependency, CI-provider behavior, publication, or immutable-release change |
| DEC-048 | Resolve issue #64 by treating only the exact bounded `-32022` response as a protocol-version rejection | Accepted | 2026-08-13 | On current-revision Streamable HTTP, accept only an identity-encoded bounded `application/json` HTTP `400` JSON-RPC error with the matching identifier, fixed `-32022` code, string message, exact requested `2026-07-28`, and at most 32 string supported revisions that exclude the requested revision; retain none of its prose or values; mark transport and envelope passed, fail `protocol.revision` with `MCP-PROTOCOL-002`, and causally skip dependent passive or active work. Never retry, initialize, downgrade, fall back, parse arbitrary prose, or add a dependency; malformed, mismatched, contradictory, or over-limit lookalikes remain HTTP-contract failures. |
| DEC-049 | Publish completed optional capabilities as backward-compatible `v0.3.0` and advance the supported line | Accepted | 2026-08-13 | Treat `MCPD-019` through `MCPD-023` as an additive minor release; support `0.3.x` and retire `0.2.x` when publication completes; use a portable Markdown diagnosis example and exact `0.3.0` install guidance; preserve stable report-major, safety, historical evidence, and every `MCPD-008A` protected immutable-byte handoff; make no M4, scanner, active-legacy, broad legacy-compatibility, native-signing, or certification claim; and withhold completion until GitHub, crates.io, Homebrew, and represented installed-channel evidence all pass. |
| DEC-050 | Resolve issue #66 with one exact, stable, compiled-only capability response | Accepted | 2026-08-13 | Add human and stable `mcp-doctor.capabilities/v1` output from fixed source-owned constants only; cover product, command activity/reporters/contracts, exact command/transport/revision support, recognized-unsupported revisions, `mcp-doctor.exit/v1`, named hard limit profiles, a 64-KiB output bound, and compile-family process-tree/file-identity mechanisms. Require an exact schema request with no discovery, retry, fallback, or downgrade; return a value-free schema-valid `v1` error and exit `2` for unsupported JSON requests. Permit compatible optional fields and new inventory entries within `v1`, require a new major for removal or semantic breakage, and require consumers to treat unknown values as unknown. Never inspect configuration or host inventory, read files, environment values, or credentials, start a process, resolve DNS, connect, retrieve a schema, trust a server advertisement, invoke a tool, add a dependency, or make a release, conformance, health, security, or assurance claim. |
| DEC-051 | Resolve `OPEN-14` by extending the sensitive `v1alpha1` snapshot only to exact passive legacy revisions and same-revision diff | Accepted | 2026-08-13 | Keep existing current-revision artifact bytes valid and unchanged; treat `protocol_revision` as selected identity and require a matching `negotiated_protocol_revision` only for MCP `2025-11-25` and `2025-06-18`; retain only fixed revision-defined logging, completions, and applicable task-capability booleans in addition to the `DEC-045` content; attach fixed value-free dialect state to legacy tool schemas so omitted `2025-11-25` means Draft 2020-12 while omitted `2025-06-18` remains ambiguous; capture from the same completed bounded passive conversation only after cleanup; compare only artifacts with the same exact revision; reject selected/negotiated, cross-revision, or revision-contract mismatch with value-free `MCP-SNAPSHOT-007`/`008` and causal skips; preserve every sensitivity, no-clobber, cleanup, size, depth, count, correlation, redaction, and offline no-activity bound; and add no replay, active call, retrieval, overwrite, cross-revision inference, score, dependency, release, broad compatibility, or M4 claim. |
| DEC-052 | Resolve `OPEN-15` with one exact-selected revision-parameterized active adapter | Accepted | 2026-08-13 | Select one typed adapter before target preparation and keep scenario authority, generation, transports, reporting, and limits shared; preserve the current `2026-07-28` wire contract exactly; let `MCPD-027` add only explicit `2025-11-25` initialize, initialized, tools catalog, immediate call, Draft 2020-12 default, legacy result, task-required no-call, exact `-32042` incomplete, and server-request no-answer behavior; reserve `2025-06-18` for `MCPD-028` behind an exact declared Draft 2020-12 gate. Legacy HTTP reuses the bounded optional session, JSON/SSE, loss-without-reinitialize, and one-DELETE boundary while omitting every modern routing and mapped header. Reports retain exact selected/negotiated identity; manifests and public claims change only with exact acceptance evidence, and broad compatibility still requires the controlled two-language and installed-channel gate. No revision is discovered, retried, downgraded, fallen back to, or granted authority by advertisements, annotations, tasks, elicitation, or server requests. |
| DEC-053 | Resolve `OPEN-16` with an explicit current-revision `reject` diagnostic | Accepted | 2026-08-14 | Add one `reject` command, not a scenario revision, for MCP `2026-07-28` only. Require one exact `--tool`, a byte-identical `--allow-tool`, invoking-user `--effects read_only` or `side_effecting`, separate `--allow-side-effects` for the latter, and a reproducible `--seed`; discovery and annotations grant no authority. Reuse the settled active adapter and bounded local Draft 2020-12 validator to derive at most seven cases in fixed order: `missing_arguments`, `wrong_root_type`, `omitted_required_property`, `wrong_property_type`, `forbidden_null`, `invalid_enum`, and `unexpected_property`. Begin from a locally valid bounded generated instance when a mutation needs one, change one structural fact, and transmit a case only after validation proves exactly one mismatch; report an inapplicable kind as skipped, reject invalid, ambiguous, external-reference, unsatisfiable, over-limit, or unencodable mapped-header work before its call, and require at least one performed case. A pass requires only a matching well-formed JSON-RPC error with integer code `-32602` and a string message, as required for invalid tool arguments by the selected revision; retain neither message nor optional data and never match prose. Any result response, including `isError: true` or `input_required`, is critical unsafe acceptance and stops later calls; any other error code or malformed error object is a distinct active-contract failure and also stops; transport, protocol, authorization, limit, crash, and cleanup behavior retains existing causal and independent-safety rules. Extend stable report `v1` compatibly with an optional fixed `mutation_kind` beside existing generator seed and structural counts, preserve byte output for existing `check` and `break`, add the exact command/revision/transport inventory to capabilities, and add no arbitrary inputs, scenario change, fuzzing, retry, concurrency, dynamic selection, raw argument/result/error retention, legacy support, dependency, release, compatibility, security-scanner, or M4 claim. |
| DEC-054 | Treat timing-dependent CI variance and undeclared runner tools as defects | Accepted | 2026-08-14 | Product time limits and outer test or job watchdogs remain necessary to bound failure, but elapsed time, wall-clock fixture values, fixed sleeps, fast polling, runner speed, and retries are not synchronization or acceptance evidence. An explicitly audited wall-clock read is permitted only when the product contract itself consumes verification time, its pass/fail range remains stable across ordinary clock transitions, and it supplies no clock-derived expected output. Asynchronous and process fixtures must expose observable state or an explicit event/acknowledgement handshake; polling may observe eventual operating-system state only when that state is itself the success condition and one outer deadline prevents a hang. An identical-source failure followed by a pass remains unresolved, must retain the failed evidence, and cannot be accepted through rerun. Correct the lowest timing-dependent layer and add one deterministic forced-state or transition regression; repetition is supplemental only. Never increase a product timeout, weaken an assertion, skip or quarantine a safety test, retry a test/build/integrity/publication/job/workflow, or broadly serialize to obtain green CI. Focused serialization is allowed only for a documented exclusive resource. The sole general retry exception remains `DEC-043`'s at-most-three classified transient downloads of one immutable size- and digest-pinned Syft asset, with deterministic retry-policy tests and no correctness retry. Every other retry or eventual-state wait must be classified in the audit and either removed or owned by an explicit later ticket and gate before it can provide release evidence. Repository-acquired tools retain their exact reviewed identities; every other non-standard command used by CI must be declared and checked before use rather than inferred from a runner image. Built-binary and native tests remain appropriate for real process, filesystem, packaging, or platform contracts after separable behavior is proved narrowly. |
| DEC-055 | Resolve issue #65 with two named compiled diagnostic limit selections and no individual overrides | Accepted | 2026-08-14 | Preserve the existing limits as `default` and add `slow-start` only to `inspect`, `check`, and `break`. `default` keeps startup and discovery at 10 seconds, request and response at 30 seconds, cleanup at two seconds, and the complete run at 120 seconds. `slow-start` sets startup and discovery to 30 seconds, request and response to 60 seconds, and the complete run to 240 seconds while retaining the two-second cleanup bound and every byte, count, schema-work, retry, redirect, concurrency, process, network, credential, protocol, side-effect, and tool-authorization boundary. Select the typed profile before target preparation, use it consistently across STDIO and Streamable HTTP, and report its fixed name plus complete effective values in the stable human and JSON contracts, its name in JUnit evidence, its retained value in offline aggregates, and its availability in compiled capabilities. Reject every unknown value locally without target activity. Do not add numeric or per-field overrides, configuration-file or environment selection, an unbounded or adaptive profile, retry, fallback, implicit revision selection, extra target activity, dependency, release, compatibility, assurance, or M4 claim. Deterministic value and immediate-state transitions prove the contract; elapsed waiting is not acceptance evidence. |

## Open decisions

`OPEN-04` through `OPEN-06` are accepted as `DEC-028` through `DEC-030`,
`DEC-031` records the implemented `MCPD-011` generation boundary, `OPEN-07` is
accepted as `DEC-032`, `OPEN-08` and `OPEN-09` are accepted as `DEC-034` and
`DEC-035`, `OPEN-10` is accepted as `DEC-041`, and `OPEN-11` is accepted as
`DEC-042`. `DEC-043` records the approved `MCPD-016A` supply-chain correction.
`OPEN-12` is accepted as `DEC-044`, `OPEN-13` is accepted as `DEC-045`, and
`OPEN-14` is accepted as `DEC-051`.
`DEC-048` records the focused issue #64 protocol-rejection correction, and
`DEC-049` records the intentional `v0.3.0` release and supported-line advance.
`DEC-050` records the issue #66 compiled capability-discovery contract,
`OPEN-15` is accepted as `DEC-052` for the shared active legacy adapter, and
`OPEN-16` is accepted as `DEC-053` for the current-revision schema-invalid
rejection diagnostic, and `DEC-054` accepts issue #41's repository-wide
deterministic-CI and runner-tool policy.
`DEC-033` separately records
the dynamic comparative evaluation method. `DEC-036` and `DEC-038` refine live
verification boundaries discovered during `MCPD-013` and `MCPD-014` without
weakening their selected controls. `DEC-037`, `DEC-039`, `DEC-040`, and
`DEC-043` record the disclosure/security, community/license, original
supply-chain, and corrective acquisition contracts. None of these decisions constitutes the complete M4 assessment or
claims that pending `MCPD-017` live controls and private recovery evidence have
passed.

The completed decisions above remain accepted. There are no unresolved open
decisions for an active optional ticket.

Future material choices must receive new `OPEN-*` identifiers rather than
silently changing an accepted decision.

## Risk register

| ID | Risk | Impact | Mitigation and escalation trigger | State |
| --- | --- | --- | --- | --- |
| RISK-01 | A diagnostic invokes a mutating tool unexpectedly | Critical | Passive default plus `DEC-029`/`DEC-031` exact configuration, effects, tool, seed/case, and side-effect gates with consent and rejection tests; any implicit, mismatched, wildcard, annotation-derived, or continued call blocks every later release | Mitigated for exact `v0.3.0` reviewed `check`, generated `break`, and remote active paths by local rejection tests, hosted exact-commit gates, and installed release smokes; every future active boundary must reprove the authority contract |
| RISK-02 | A timed-out server or descendant remains running | Critical | Managed process tree, shutdown bounds, termination, reap, and resistant-child fixtures; any surviving PID blocks release | Mitigated for exact `v0.3.0` by the hosted native process matrix and retained reviewed/generated resistant-descendant cleanup journeys; every future process boundary and exact artifact must retain it |
| RISK-03 | Secrets or raw production values reach output | High | Structural redaction and sentinel tests across errors, reports, debug surfaces, fixtures, and the `DEC-028` environment-only secret boundary; any observed name or value blocks release | Mitigated for exact `v0.3.0`: target and argument secret rejection, structural-only reproduction, and human, stable JSON, and JUnit redaction pass local and hosted exact-commit evidence; the risk remains open for every later boundary and artifact |
| RISK-04 | Protocol evolution makes diagnostics incorrect | High | Revision-specific rules and fixtures with explicit unsupported outcomes; a new release triggers contract review | Open |
| RISK-05 | Pathological schema or output exhausts resources | High | Depth, bytes, errors, cases, time, and reference limits; an unbounded input path blocks release | Mitigated for exact `v0.3.0` across passive, reviewed active, synthesis, schema, instance, aggregate-input, case, and report work by local and hosted gates; later boundaries and artifacts require their own evidence |
| RISK-06 | Remote diagnosis enables SSRF or credential leakage | Critical | `DEC-030` fixes exact target gates, IANA-based address classification and pinning, verified TLS, credential-to-endpoint consent, direct zero-redirect/retry connections, finite headers/bodies, and value-free reports; any bypass, peer drift, implicit network source, or secret output blocks completion | Mitigated for exact `v0.3.0` bounded passive, reviewed, and generated activity through the retained HTTP transport and exact-authority network journeys in hosted native evidence; every future multi-origin boundary must reprove it |
| RISK-07 | Generated cases are irreproducible or exceed authorized scope | High | Versioned stable seed selection, ordered generation, structural evidence, exact tool/effect/target gates, and finite cases, candidates, inputs, work, and concurrency; mismatch blocks active testing | Mitigated for exact `v0.3.0` by fixed-seed fixtures, local and HTTP authorization rejection, structural redaction, every generation limit, sequential execution, and hosted exact-commit verification; future generator changes reopen it |
| RISK-08 | A passing report creates false confidence after skipped checks | High | Per-check performed/skipped state and non-ambiguous summary; any hidden skip blocks release | Mitigated for exact `v0.3.0` by hosted human, schema-valid stable JSON, and JUnit causal-skip and authorization journeys; every future reporter or check-state change must preserve the invariant |
| RISK-09 | Broad protocol, transport, and reporting scope delays a usable slice | High | M1 ends at passive `inspect`, M2 publishes it, and M3 stays an ordered set of bounded vertical tickets; any broad feature becoming a prerequisite for an earlier completed slice escalates | Mitigated by the ordered plan and `DEC-027`; voluntary evidence may reprioritize work, but its absence neither authorizes breadth nor blocks scoped work |
| RISK-10 | The public identity is unavailable, ambiguous, or confused with an existing command before publication | High | `DEC-008` retains the product and executable under EnjoyableWork, accepts the cross-ecosystem collision, defines a Cargo-package fallback, and requires exact official-channel guidance plus an immediate pre-publication registry recheck | Mitigated for the first release: the preferred `mcp-doctor` crate identity is published under the exact EnjoyableWork source and metadata; future channel guidance must preserve the distinction |
| RISK-11 | A release channel installs bytes not represented by the immutable release | Critical | `MCPD-008` proves exact package/formula equality, checksums, attestations, and native installed smokes for the first release; `MCPD-008A` makes those checks preconditions for every later downstream write; any mismatch requires a new version | Mitigated for `v0.1.0`, `v0.2.0`, and `v0.3.0` by byte-identical Cargo and Homebrew handoffs, rejected mismatch cases, authenticated assets, and successful native channel verification; every future release must retain the same immutable-byte gates |
| RISK-12 | An unprotected default branch permits direct, destructive, or insufficiently reviewed changes | High | `DEC-035` fixes the zero-approval PR, strict aggregate-check, squash-only, no-standing-bypass, deletion/force-push, public-projection drift verification, authenticated hidden-state readback, and bounded emergency contract; any unverified bypass or destructive path blocks M4 | Mitigated for the 2026-08-11 `MCPD-013` scope by the active public ruleset, canonical merge settings, normal protected merge, rejected direct/deletion/non-fast-forward paths, closed emergency exercise, post-removal gates, credential-free projection pass, and non-disclosing empty-bypass pass. An administrator can still change repository policy; ruleset, required-context, merge-setting, administrator-boundary, or GitHub-capability drift reopens the risk and requires both verifiers and, where applicable, a new exercise |
| RISK-13 | A contributor publicly exposes a vulnerability, credential, or unsafe diagnostic because reporting and prevention controls are incomplete | High | `MCPD-014` verifies private reporting, safe guidance, entitled scanning and prevention controls, limitations, and a non-disclosing baseline; any public sensitive report or hidden finding blocks M4 | Mitigated for the scoped 2026-08-11 `MCPD-014` surfaces by the recognized policy and private route, enabled entitled dependency and secret-prevention controls, representative and exact-`main` CodeQL, zero open repository-visible alerts, and non-disclosing pass. A public sensitive report, hidden finding, supported-line or policy drift, disabled or changed control, failed exact-`main` analysis, entitlement change, newly observable surface, or stale baseline reopens the risk; scan-history, provider-only, paid-feature, product-scanner, and complete-M4 evidence remain explicitly outside this result |
| RISK-14 | Mutable automation, privileged untrusted code, or unauthenticated distribution compromises the project or its releases | Critical | `MCPD-008A` limits repeat publication to reviewed full-SHA automation, OIDC or narrowly scoped short-lived authority, immutable-byte preconditions, and negative authorization tests; `MCPD-016` audits the complete CI and distribution boundary; `DEC-043` requires direct immutable, digest-authenticated Syft assets after its Action pin proved incomplete; any drift or credential exposure blocks publication and M4 | Mitigated for the corrected 2026-08-13 scope: repository-owned Linux acquisition, transient-only bounded retries, deterministic negatives, Action and live-allowlist removal, the authenticated exact-`main` verifier, and first-attempt exact-head plus exact-`main` gates passed; the same controls then published and independently verified immutable `v0.3.0` without a retry or long-lived publication credential. Any Action, standalone-tool, asset, allowlist, workflow, credential, distribution, or evidence drift reopens the risk. |
| RISK-15 | Organization-owner loss or over-broad long-lived credentials become an undocumented recovery dependency | High | `DEC-041` accepts the documented single-owner residual risk and fixes strong MFA, lowest access, application and credential scope, and private recovery requirements; `DEC-042` fails closed around GitHub Free's unavailable owner-only outside-invitation setting; `MCPD-017` remains incomplete until the protected exact-`main` verifier and temporary-credential revocation bind the completed live controls, reviewed selected-repository App inventory, exact short-lived operator authority, and fresh private recovery exercise without sensitive disclosure | Mitigated at the live pre-merge boundary: supported organization controls, restricted OAuth and PAT access, exact source-and-tap verifier scope, bounded private inventories, obsolete tap-key removal, and the independent recovery exercise pass; exact-`main` evidence and verifier-token revocation remain open under `MCPD-017` |
| RISK-16 | A stale, unofficial, or over-broad assurance claim misleads adopters | High | `DEC-034` locks exact version and proof routes and makes drift a stop-and-decide gate; `MCPD-018` binds every claim to exact scope, date, official proof, public evidence, and removal triggers; missing, stale, withdrawn, or ambiguous proof blocks or removes the claim | Policy resolved; proof remains deferred with M4 |
| RISK-17 | Technically correct findings become an undifferentiated failure list that does not help a developer repair a server or earn repeat use | High | Every MVP failure identifies the expected earliest actionable layer, preserves independent safety failures, links downstream skips to their cause, and includes safe what, where, why, expectation, remediation, and versioned-rule evidence; report-only cases, maintainer trials, and voluntary feedback record unclear findings, false findings, time to value, and repeat use | M1 report sufficiency passes locally and hosted; the checkpoint closed with zero independent reports and no adoption claim, while future feedback may reprioritize later product work |
| RISK-18 | Revision support excludes or misdiagnoses too much of the reachable ecosystem | High | `DEC-024` retains the proven current-revision matrix, `DEC-044` permits only explicit passive legacy selection, and `DEC-052` fixes exact-selected active legacy adaptation without fallback; broad wording for any new revision additionally requires controlled official and independent cases spanning at least two languages plus represented installed-channel journeys | Mitigated for the explicit MCP `2025-11-25` source scope by protected [PR 78](https://github.com/EnjoyableWork/mcp-doctor/pull/78), controlled two-language cases, represented source installs, merge [`ac3d9ac`](https://github.com/EnjoyableWork/mcp-doctor/commit/ac3d9ac1c289b3329eadbe8fb1a35cca597386c4), and first-attempt exact-`main` gates; and for the exact MCP `2025-06-18` source scope by protected [PR 80](https://github.com/EnjoyableWork/mcp-doctor/pull/80), shared-adapter synthetic journeys on both transports, strict dialect no-call negatives, native and represented source installs, merge [`e380b26`](https://github.com/EnjoyableWork/mcp-doctor/commit/e380b26c382ea2b83fefe41c153f00baea023db2), closed issue #61, and first-attempt exact-`main` [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808031576), [CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808031251), [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808031581), and [compatibility](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31808063280). MCP `2025-06-18` real-server, published-channel, and ecosystem-wide legacy positioning stay withheld |
| RISK-19 | An unnecessary, stale, compromised, or silently widened dependency executes in the product, developer environment, or CI supply chain | Critical | Default to no addition; require an owning need and dated maintenance/provenance/security/graph review; use exact direct requirements, a committed lockfile, narrow features, reviewed sources, `cargo-deny`, non-automatic update approval, and a regression check; removal, unexplained upstream inactivity, ownership change, advisory, new build script/unsafe surface, or unreviewable lockfile growth triggers escalation | Mitigated for the dated `MCPD-016` dependency scope and tightened by `MCPD-016A`: explicit grouped version and security proposals remain review-only; exact direct-requirement, feature-graph, source, license, advisory, duplicate, Action, and standalone-tool inventories fail closed; and superseded Syft `1.50.0` moves from an Action-controlled mutable installer to exact immutable, security-remediating `1.51.0` asset identities. Future dependency, Action, tool, asset, or acquisition-policy drift requires the same review. |
| RISK-20 | Users cannot find a real project route or receive incompatible license terms because repository, community, channel, or artifact scope drifts | High | `DEC-039` inventories every public organization repository, centralizes reachable community and defect routes, explicitly delegates the tap, and verifies HTTPS official channels plus exact source, package, archive, and formula license evidence without credentials; any new unclassified repository, unavailable route, stale policy, license mismatch, or unexplained asset blocks M4 | Mitigated for the current three-repository `MCPD-015` scope revalidated on 2026-08-12. The credential-free five-repository completion pass remains historical; when `courtside-mcp` and `enjoyable-mcp` disappeared from the organization inventory, the verifier failed closed and the MCPD-016 closure rebaselined the canonical inventory before restoring a pass. Any later repository, route, channel, license, release-set, package, formula, or GitHub/crates.io drift reopens the risk; the immutable SPDX limitation and later complete-assurance work remain explicit. |
| RISK-21 | A contract snapshot discloses sensitive advertised data, an offline diff contacts a target, or a compatibility label overstates what a structural comparison proves | High | `DEC-045` and `DEC-051` require an exact-path sensitivity acknowledgement, same-conversation completed passive creation, exclusive bounded regular-file creation with owner-only Unix mode, a fixed include/exclude boundary, exact legacy selected/negotiated and dialect identity, value-free same-revision diff findings, artifact-local ordinal correlation, local-only inputs, no retrieval or activity, and conservative documented classifications with review-required fallback; any ordinary-report identifier, excluded sentinel, overwrite, network/process attempt, unbounded artifact, invalid correlation, cross-revision comparison, dialect inference, or unsupported compatibility claim blocks completion | Mitigated for current and explicit passive legacy snapshot/diff source scope by `MCPD-020` and `MCPD-026`: protected [PR 63](https://github.com/EnjoyableWork/mcp-doctor/pull/63), merge [`6e0f0ac`](https://github.com/EnjoyableWork/mcp-doctor/commit/6e0f0acf096f797a12f3bf8826d8d11963007039), closed issue #74, and first-attempt exact-head plus exact-`main` native CI/preflight prove same-conversation, capability, dialect, finite-bound, no-clobber, cleanup, current-byte, redaction, installed-package, dependency, and offline-isolation behavior. No broad compatibility or immutable-release claim follows. Any snapshot boundary, acknowledgement, identity, dialect, overwrite, ordinal, offline-isolation, bound, or compatibility-classification drift reopens the risk |
| RISK-22 | Multiple requested reports replay target activity, diverge in diagnosis, overwrite files, leave misleading partial artifacts, or hide reporter failure behind a diagnostic exit | High | `DEC-046` and `MCPD-021` require one typed result, fixed JSON/JUnit file flags, pre-activity destination and alias rejection, exclusive same-directory staging with Unix owner-only mode and platform ACL inheritance elsewhere, no-clobber all-output publication, rollback and visible cleanup failure, unchanged per-report bounds, an eight-MiB aggregate rendered-output bound, reporter-failure exit `4`, byte-compatible stdout-only behavior, and deterministic STDIO/HTTP/active counter evidence; any repeated process, request, discovery, credential resolution, or `tools/call`, divergent primary/skip/outcome/exit metadata, overwritten file, leaked path, unbounded render, residue after ordinary failure, or false-success exit blocks completion | Mitigated for the `MCPD-021` scope by protected [PR 50](https://github.com/EnjoyableWork/mcp-doctor/pull/50), deterministic destination, race, rollback, aggregate-bound, redaction, reporter-exit, STDIO, HTTP, active, and installed-platform evidence, and passing exact-head plus exact-`main` CI and release preflight; any projection, destination, overwrite, bound, rollback, redaction, exit-precedence, or no-replay drift reopens the risk |
| RISK-23 | An offline aggregate turns incomplete or failed evidence into a pass, drops a causal or independent finding, reflects untrusted values or paths, double-counts an aliased report, exhausts local resources, or unexpectedly contacts or executes a target | High | `DEC-047` and `MCPD-022` require explicit ordinal inputs, stable-schema and semantic validation, normalized known-safe fields, compatible-unknown ignore behavior, conservative fail/incomplete/pass precedence, complete member evidence, regular-file and alias rejection with no-follow opens and native complete identities, finite input/depth/node/check/finding/time/output work, no-clobber rollback-safe output, value-free errors, and a command surface with no process/network/credential/retrieval/target/tool authority; any demotion, omitted retained member fact, unknown-value reflection, duplicate identity, unbounded work, target activity, partial artifact, or false-success exit blocks completion | Mitigated for the `MCPD-022` scope by protected [PR 52](https://github.com/EnjoyableWork/mcp-doctor/pull/52), deterministic outcome, retention, compatibility, native alias, limit, atomic-output, redaction, no-activity, and installed-platform evidence, and passing corrected exact-head plus exact-`main` CI and release preflight; any input, normalization, identity, precedence, bound, output, redaction, or activity-boundary drift reopens the risk |
| RISK-24 | A structured protocol-version rejection is mislabeled as transport failure, reflected unsafely, or used to trigger retry or downgrade | High | `DEC-048` and `MCPD-023` require the exact bounded current-revision `400`/`-32022` shape, matching request and requested revision, noncontradictory finite supported list, value-free rule evidence, protocol-layer primary diagnosis, causal skips, and zero replay/fallback; every malformed lookalike remains an HTTP failure | Mitigated for exact `v0.3.0` by focused exact and negative classifier tests, passive and active one-request human/JSON/JUnit/redaction journeys, protected first-attempt exact-head and exact-`main` evidence, closed issue #64, and represented installed-channel verification; any classifier, bound, redaction, causal-skip, replay, fallback, or revision-contract drift reopens the risk |
| RISK-25 | A stale or over-broad capability manifest starts an unsupported diagnostic, hides an available one, leaks ambient state, or turns server advertisements into execution authority | High | `DEC-050` requires source-owned constant reuse, CLI-inventory regression, an exact command/transport/revision matrix plus recognized-unsupported inventory, tri-state consumer cases, stable compatible schema rules, value-free exact schema rejection, fixed compile-family fields, a 64-KiB output limit, and zero configuration, host-inventory, file, environment-value, credential, process, DNS, network, retrieval, server-advertisement, or tool activity; any manifest/code drift, false support, unknown-value execution, ambient value, unbounded output, or activity blocks completion | Mitigated for the `MCPD-025` scope by protected [PR 58](https://github.com/EnjoyableWork/mcp-doctor/pull/58), the schema, golden, unit and built-binary deterministic/unknown-version/forward-compatibility/consumer/proxy/target-like/redaction evidence, represented installed smokes, and first-attempt exact-head [CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31761161743) plus [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31761161698); any manifest/code, schema, matrix, profile, bound, redaction, consumer-unknown, or zero-activity drift reopens the risk |
| RISK-26 | A schema-invalid rejection diagnostic executes a side effect, mistakes success for safe rejection, or leaks the deliberately invalid request or server response | Critical | `DEC-053` and `MCPD-029` require the existing exact tool, effect, side-effect, target, and credential gates; prove each finite case invalid for one fixed structural reason before transmission; treat only exact structural `-32602` rejection as the expectation rather than safety authority; stop on unsafe success or exhausted safety bounds; and retain only value-free reproduction and causal evidence | Mitigated for the current source scope by protected [PR 82](https://github.com/EnjoyableWork/mcp-doctor/pull/82), exact implementation and final evidence heads, closed issue #75, merge [`3472952`](https://github.com/EnjoyableWork/mcp-doctor/commit/3472952a521ad30fbf716c828739887835a78898), focused safety and redaction regressions, complete local/package/install evidence, and first-attempt exact-head plus exact-`main` CI, CodeQL, and release preflight; no published artifact or legacy claim follows, and any authority, invalidity-proof, response-classification, redaction, cleanup, or finite-work drift reopens the risk |
| RISK-27 | Scheduler-dependent fixtures, correctness retries, or incidental runner tools make identical source pass and fail or hide a safety regression | High | `DEC-054` and `MCPD-030` require a complete tracked timing/tool inventory; product bounds only in product code or explicit limit fixtures; audited product-clock input only for a contract that consumes verification time; job, socket, and channel deadlines only as outer watchdogs; observable state or event/acknowledgement fixture coordination; deterministic forced-state regressions; one exact immutable Syft acquisition exception under `DEC-043`; declared non-standard CI tools; preserved failed evidence; and first-attempt acceptance without rerun, timeout inflation, assertion weakening, quarantine, or broad serialization | Mitigated for current test and source-checkout workflow scope by protected [PR 83](https://github.com/EnjoyableWork/mcp-doctor/pull/83), merge [`dbc19bd`](https://github.com/EnjoyableWork/mcp-doctor/commit/dbc19bd7a863c8e53651a78bd4570616a59d5e02), closed issue #41, the tracked audit and enforced inventories, direct readiness/lock-release and peer-close/client-completion proofs, exact runner-command verification, and first-attempt exact-head plus exact-`main` native CI, CodeQL, and release-preflight evidence. Any timing threshold used as correctness proof, correctness rerun, undeclared runner tool, inventory drift, or weaker cleanup/state-transition evidence reopens the risk. Release-only retry, eventual-state, and legacy runner-verification mechanisms remain blocked from supporting a later publication claim until proposed `MCPD-031` corrects and rehearses them. |
| RISK-28 | A larger diagnostic profile becomes hidden execution authority, weakens cleanup or capacity limits, masks a hung server, or is reported inconsistently | High | `DEC-055` and `MCPD-032` permit only invocation-local exact named selection for three existing diagnostic commands, cap both profiles with fixed total and phase times, keep cleanup and all byte/count/retry/redirect/concurrency/authority limits identical, reject unknown names before target preparation, and require one typed profile to drive runtime, human/JSON/JUnit reports, offline aggregate retention, and compiled capability discovery. Deterministic value-equality, pre-target rejection, cross-transport immediate-state, active no-authorization, artifact-parity, and represented installed-source tests replace elapsed-time acceptance. | Open until the complete local gates, protected exact-head evidence, merge, issue closure, and durable completion record pass. Any per-field/config/environment/unbounded/adaptive mode, profile-dependent cleanup/capacity/authority change, hidden fallback, report/runtime mismatch, or timing-based acceptance reopens the risk. |

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
