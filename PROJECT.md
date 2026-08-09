# mcp-doctor project plan

This is the living source for product scope, delivery status, ordered work,
decisions, risks, and release gates.

| Control | Current state |
| --- | --- |
| Document state | Active |
| Product state | M0 foundation; no MCP diagnostic behavior yet |
| Current milestone | M0 — trustworthy walking skeleton |
| Overall status | Local M0 foundation passes; first hosted CI matrix remains unverified |
| Current focus | `MCPD-003` — obtain hosted cross-platform baseline evidence |
| Public release | None |
| Last reviewed | 2026-08-09 |
| Next review trigger | First hosted CI run; a change to the M1 safety boundary; M4 activation; or assurance-framework, issuer-proof, security, release-pipeline, organization-access, or evidence drift |

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
| M1 — Local diagnosis | `MCPD-004` → `MCPD-005` → `MCPD-006` → `MCPD-007` → `MCPD-008` | One safe, useful STDIO diagnostic journey with explicit active scenarios |
| M2 — Production diagnosis | `MCPD-009` → `MCPD-010` → `MCPD-011` | Streamable HTTP, bounded adversarial generation, and CI-friendly reporting |
| M3 — Public release | `MCPD-012` | One immutable version installed and proven through every advertised channel |
| M4 — Enterprise assurance and adoption | `MCPD-013` → `MCPD-014` → `MCPD-015` → `MCPD-016` → `MCPD-017` → `MCPD-018` | Contributor-compatible governance, repository and organization controls, supply-chain evidence, and a public scoped assurance baseline |

Signed native macOS and Windows artifacts are a later candidate, not part of
the first public release. They require an accepted funding and signing decision
plus native installed evidence.

M4 begins only after the first release is independently verified. It does not
delay or reopen M3, and it does not turn a self-assessment into a warranty,
independent certification, regulatory-compliance claim, or support SLA.

## Product outcome

`mcp-doctor` gives an MCP server author a deterministic way to inspect a local
or remote server, find protocol and schema defects, reproduce runtime failures,
and distinguish what was actually tested from what was skipped.

The proof is not that the CLI can parse one response. The proof is that a valid
synthetic server passes; malformed schemas, crashes, timeouts, oversized output,
invalid results, and cleanup failures produce precise non-zero outcomes; the
same seed reproduces an active finding; and no default path surprises a real
system with tool execution.

### Product principles

- **Passive first:** inspection may discover and validate, but active tool
  calls require explicit authorization.
- **Bounded by construction:** time, bytes, messages, schema work, cases,
  redirects, retries, concurrency, and cleanup all have enforceable limits.
- **One result model:** human and machine output report the same findings,
  redaction, skips, and outcome.
- **Versioned conformance:** every rule names the MCP revision or compatibility
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
| M0 | Clean checkout builds; help/version work; format, Clippy, tests, dependency policy, and hosted CI pass | In progress |
| M1 | Complete bounded STDIO journey passes the synthetic success and failure matrix on supported native hosts | Proposed |
| M2 | Local and Streamable HTTP targets share deterministic diagnostics, explicit active scenarios, and stable machine output | Proposed |
| M3 | One immutable release installs and passes its diagnostic smoke journey through every advertised channel | Proposed |
| M4 | The selected current OpenSSF OSPS Baseline Level 1 controls pass with dated public evidence and official self-certification proof; exact release-artifact provenance is separately evaluated against the selected current SLSA Build L2 requirements | Proposed |

Each milestone must leave the preceding milestone working. Expansion does not
justify weakening cleanup, redaction, determinism, or active-execution consent.
Assurance work may verify an immutable release but must never rewrite it or
publish a broader claim than its evidence supports.

## Distribution contract

The product, repository, Cargo package, and installed executable use the name
`mcp-doctor`. Registry availability was checked during initialization and must
be checked again immediately before first publication because an unpublished
name is not reserved.

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
| Earlier revisions | Decide exact handshake-based compatibility in `MCPD-004`; do not infer it |
| Default activity | `inspect`: discovery and structural validation only; no implicit tool call |
| Active activity | `check`: one explicitly selected scenario and tool target with fixed budgets |
| Schemas | JSON Schema 2020-12 under bounded local evaluation; no external retrieval by default |
| Findings | Typed code, severity, safe location/context, performed/skipped state, and overall outcome |
| Output | Redacted human report; stable machine contract is completed before M2 exits |
| Process policy | Literal executable/arguments, constrained environment, bounded I/O and time, full cleanup and reap |
| Test data | Synthetic fixture servers and disposable environments only |
| Distribution | Source checkout; public packages are M3 |

### Golden M1 journey

Given a disposable test environment and executable fixture servers:

1. `mcp-doctor inspect` identifies a conforming STDIO server and reports
   exactly which passive checks passed without calling a tool.
2. A malformed message, invalid catalog schema, timeout, oversized response,
   early exit, and unsupported protocol each return a distinct redacted finding
   and non-zero status.
3. The CLI closes or terminates and reaps every fixture process, including one
   that ignores graceful shutdown.
4. `mcp-doctor check` runs an active scenario that calls only its selected
   synthetic tool, enforces its budgets, and records enough structural input
   plus seed to reproduce failure.
5. A successful active result is validated against its advertised output
   schema; a mismatch is a diagnostic failure.
6. Repeating a scenario with the same fixture and seed produces the same case
   order and finding classification.
7. No default test reads user configuration, reaches a production endpoint, or
   exposes fixture values in a report or assertion failure.

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
| D-03 | Local and hosted quality baseline | M0 | In progress | POSIX and PowerShell gates, dependency policy, least-privilege three-OS workflow, Dependabot, and community/security surfaces pass local validation; first hosted run pending |
| D-04 | Versioned diagnostic result contract | M1 | Proposed | Types, fixtures, reporter tests, and accepted revision decisions |
| D-05 | Bounded STDIO diagnostic journey | M1 | Proposed | Transport/process tests and built-binary success/failure matrix |
| D-06 | Explicit reproducible `check` scenario runner | M1 | Proposed | Scenario contract, deterministic fixtures, and safety evidence |
| D-07 | Streamable HTTP diagnostic journey | M2 | Proposed | Remote fixtures, network-boundary tests, and native CLI journeys |
| D-08 | Immutable public release | M3 | Proposed | Release, registry, tap, provenance, and installed native smoke evidence |
| D-09 | Evidence-backed enterprise assurance baseline | M4 | Proposed | Verified repository, organization, community, licensing, and supply-chain controls; complete OSPS Level 1 crosswalk; official self-certification proof; and exact-artifact SLSA evaluation |

## Ticket board

| ID | Outcome | Milestone | Status | Depends on | Acceptance evidence |
| --- | --- | --- | --- | --- | --- |
| MCPD-001 | Establish the product promise, operating model, safety priorities, delivery sequence, decisions, and risks | M0 | Done | — | Root product and project contracts are internally consistent and link correctly |
| MCPD-002 | Bootstrap one Rust 2024 binary with truthful help/version output and isolated built-binary tests | M0 | Done | `MCPD-001` | Locked build, format, Clippy, five tests, help, version, metadata, self-contained package, and installed package smoke pass |
| MCPD-003 | Add disposable local gates, dependency policy, least-privilege cross-platform CI, maintenance automation, and community/security entry points | M0 | In progress | `MCPD-002` | POSIX and PowerShell gates, `cargo-deny`, Actionlint, ShellCheck, YAML parsing, links, packaging, and identity checks pass locally; first hosted matrix remains pending |
| MCPD-004 | Define supported MCP revision behavior, typed findings, limits, exit semantics, and redacted report contract | M1 | Proposed | `MCPD-003` | Accepted compatibility decisions plus focused model, fixture, and reporter tests |
| MCPD-005 | Implement the bounded STDIO process and message boundary with guaranteed cleanup | M1 | Proposed | `MCPD-004` | Success, malformed, timeout, oversize, early-exit, resistant-child, and redaction tests |
| MCPD-006 | Diagnose discovered tools, prompts, resources, and JSON Schema contracts without implicit tool execution | M1 | Proposed | `MCPD-005` | Catalog/schema fixtures prove valid, invalid, complex, duplicate, and bounded cases |
| MCPD-007 | Add explicit, budgeted, seed-reproducible `check` scenarios and result-schema validation | M1 | Proposed | `MCPD-006` | Selected-tool consent, deterministic generation, crash, silent failure, and output mismatch journeys |
| MCPD-008 | Prove and document the complete local golden journey and native failure matrix | M1 | Proposed | `MCPD-007` | Built-binary suite passes every M1 criterion on the accepted platform matrix |
| MCPD-009 | Add a bounded Streamable HTTP transport with explicit remote-target and credential policy | M2 | Proposed | `MCPD-008` | Local HTTP fixtures prove headers, redirects, auth redaction, TLS/error, timeout, and response limits |
| MCPD-010 | Add the bounded adversarial `break` command for authorized tools | M2 | Proposed | `MCPD-009` | Schema-derived cases are deterministic, limited, reproducible, and cannot widen target scope |
| MCPD-011 | Stabilize machine reports and CI integration across local and remote journeys | M2 | Proposed | `MCPD-010` | Versioned JSON plus one accepted CI format prove redaction, compatibility, and exit behavior |
| MCPD-012 | Publish and independently verify the first immutable GitHub, Cargo, and Homebrew release | M3 | Proposed | `MCPD-011` | Every artifact and public channel installs the same version and passes the diagnostic smoke journey |
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
| MCPD-006 | Complete `MCPD-006`: diagnose advertised tools, prompts, resources, and bounded JSON Schema 2020-12 contracts without implicit tool execution or external schema retrieval. Finish when valid and invalid fixture catalogs produce deterministic redacted findings. |
| MCPD-007 | Complete `MCPD-007`: add explicit selected-tool scenarios with fixed budgets, deterministic seeds, reproducible structural cases, and output-schema validation. Never broaden the authorized target. Finish when active success and failure journeys pass without secret output or orphaned processes. |
| MCPD-008 | Complete `MCPD-008`: prove the entire M1 STDIO journey and failure matrix through the built binary on the accepted native platform matrix, correct defects found, and publish accurate usage and safety guidance without adding M2 scope. |
| MCPD-009 | Complete `MCPD-009`: add bounded Streamable HTTP diagnosis under an accepted redirect, SSRF, proxy, authentication, TLS, header, and redaction policy. Do not begin adversarial generation. Finish when deterministic local remote-server fixtures prove the full network boundary. |
| MCPD-010 | Complete `MCPD-010`: generate bounded deterministic boundary cases only for explicitly authorized tools, record reproducible seeds and structural inputs, and enforce schema and scenario limits. Finish when generation cannot widen target or execution scope. |
| MCPD-011 | Complete `MCPD-011`: publish a versioned redacted machine-result contract and accepted CI reporter that preserve human-report findings, skipped checks, and exit semantics across STDIO and HTTP journeys. |
| MCPD-012 | Complete `MCPD-012`: publish one protected immutable version through GitHub Releases, crates.io, and source-built Homebrew, with deterministic packages, checksums, SPDX SBOMs, attestations, and installed native diagnostic smokes for every represented channel. |
| MCPD-013 | Complete `MCPD-013`: protect the default branch with a contributor-compatible public ruleset, deliberate approval, check, bypass, merge, deletion, non-fast-forward, and commit-signing choices; implement credential-free drift verification; and prove normal, rejected, and bounded emergency paths. Do not change immutable release bytes or begin later assurance tickets. |
| MCPD-014 | Complete `MCPD-014`: establish recognized supported-version, security-contact, private-reporting, response, and coordinated-disclosure guidance; enable and read back the entitled dependency, code-scanning, secret-prevention, and private-reporting controls; document unavailable features exactly; and verify a non-disclosing clean baseline. Do not publish a complete-baseline claim. |
| MCPD-015 | Complete `MCPD-015`: verify public contribution, conduct, support, defect-reporting, repository-inventory, official-channel, inbound-license, source-license, and released-asset license contracts across every in-scope repository and distribution channel. Avoid nominal reviewers, owners, or controls, and do not begin supply-chain changes. |
| MCPD-016 | Complete `MCPD-016`: automate grouped dependency updates; inventory and verify every selected Action at a reviewed full commit SHA; prove untrusted workflows are read-only and secretless; reject generated executables and unreviewable binary artifacts; and authenticate the immutable release, Cargo package, and Homebrew formula without changing published bytes. |
| MCPD-017 | Complete `MCPD-017`: define and verify strong-MFA, lowest-default-access, manual-grant, repository-creation, installed-application, automation-credential, ownership-continuity, and recovery controls using aggregate non-sensitive evidence. Any live organization mutation or private recovery confirmation requires explicit owner authority and must not expose identities or recovery material. |
| MCPD-018 | Complete `MCPD-018`: confirm the current official OSPS, BadgeApp, and SLSA versions; publish a dated and scoped crosswalk for every selected OSPS Level 1 control; complete the official self-assessment and obtain its official badge only after every control passes; verify that badge and evidence on exact `main`; evaluate only the exact M3 release artifacts against SLSA Build L2; and define annual, framework-change, issuer-status, security-incident, release-pipeline-change, organization-change, and evidence-drift review and removal triggers. Never imply independent certification, regulatory compliance, higher OSPS levels, all-artifact SLSA coverage, or paid platform signing. |

## M4 enterprise assurance boundary

M4 turns existing project, release, repository, and organization practices into
dated, scoped, independently inspectable adoption evidence. It is a
post-release assurance milestone, not a second product release and not a reason
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
| DEC-008 | Use direct package and executable identity `mcp-doctor` | Working assumption | 2026-08-09 | Recheck registry availability immediately before publication; use a publisher-qualified fallback only if needed |
| DEC-009 | First release uses Linux archives, Cargo source, and source-built Homebrew | Accepted | 2026-08-09 | Signed native macOS/Windows and WinGet remain later funded scope |
| DEC-010 | Track the ordered delivery plan in this repository | Accepted | 2026-08-09 | Hosted issues may supplement but do not replace milestone and decision truth |
| DEC-011 | Use an `inspect`, `check`, and `break` command family | Accepted | 2026-08-09 | Inspection stays passive; checks run explicit scenarios; adversarial generation requires an explicit tool and finite budget |
| DEC-012 | Add a post-release M4 for evidence-backed enterprise assurance and adoption | Accepted | 2026-08-09 | M3 remains the first-release gate; M4 orders repository governance, disclosure and security controls, community and licensing evidence, supply-chain hardening, organization access and recovery, OSPS Level 1 self-assessment with official proof, and exact-artifact SLSA evaluation without implying certification or regulatory compliance |

## Open decisions

| ID | Decision needed | Needed by | Default if unresolved |
| --- | --- | --- | --- |
| OPEN-02 | Exact earlier MCP revision compatibility window | `MCPD-004` | Support only `2026-07-28`; return a precise unsupported-version finding |
| OPEN-03 | Finding codes, severity policy, and exit-code mapping | `MCPD-004` | Stable typed codes; zero only when every required performed check passes |
| OPEN-04 | Scenario file format and secret-reference boundary | `MCPD-007` | Versioned local file with literals prohibited from ordinary output; no secret store |
| OPEN-05 | Safety annotations or confirmations required before active tool calls | `MCPD-007` | Explicit tool allowlist and per-run active acknowledgement |
| OPEN-06 | Streamable HTTP redirect, proxy, private-address, and authentication contract | `MCPD-009` | No redirects, no inherited proxy, explicit headers, and remote target shown before activity |
| OPEN-07 | Machine output versioning and first CI reporter | `MCPD-011` | Versioned JSON first; evaluate JUnit versus SARIF from the intended consumer |
| OPEN-08 | Exact OSPS, BadgeApp, and SLSA versions and proof mechanisms at M4 activation | `MCPD-013` | Use the then-current official versions; planning baseline is OSPS `v2026.02.19` Level 1 and SLSA `v1.2` Build L2, with a documented update if either is superseded |
| OPEN-09 | Default-branch approval count, required checks, merge methods, bypass, emergency administration, and commit-signing policy | `MCPD-013` | Prevent direct updates and deletion with strict current checks and no standing bypass; do not require an unavailable independent reviewer or unproven signature path |
| OPEN-10 | Organization membership, application, automation-credential, owner-continuity, and private recovery boundary | `MCPD-017` | Lowest default access, deliberate grants, strong MFA, scoped automation, explicit residual-risk acceptance, and non-disclosing recovery evidence |

## Risk register

| ID | Risk | Impact | Mitigation and escalation trigger | State |
| --- | --- | --- | --- | --- |
| RISK-01 | A diagnostic invokes a mutating tool unexpectedly | Critical | Passive default, explicit selected-tool scenarios, and consent tests; any implicit call blocks release | Open — M1 gate |
| RISK-02 | A timed-out server or descendant remains running | Critical | Managed process tree, shutdown bounds, termination, reap, and resistant-child fixtures; any surviving PID blocks release | Open — M1 gate |
| RISK-03 | Secrets or raw production values reach output | High | Structural redaction and sentinel tests across errors, reports, debug surfaces, and fixtures; any observed value blocks release | Open — all milestones |
| RISK-04 | Protocol evolution makes diagnostics incorrect | High | Revision-specific rules and fixtures with explicit unsupported outcomes; a new release triggers contract review | Open |
| RISK-05 | Pathological schema or output exhausts resources | High | Depth, bytes, errors, cases, time, and reference limits; an unbounded input path blocks release | Open — M1 gate |
| RISK-06 | Remote diagnosis enables SSRF or credential leakage | Critical | Explicit M2 network policy and local fixtures before HTTP implementation; unclear proxy/address behavior blocks `MCPD-009` | Deferred with M2 |
| RISK-07 | Generated cases are irreproducible or exceed authorized scope | High | Stable seed, ordered generation, structural evidence, and target allowlist; mismatch blocks active testing | Open — M1 gate |
| RISK-08 | A passing report creates false confidence after skipped checks | High | Per-check performed/skipped state and non-ambiguous summary; any hidden skip blocks release | Open — M1 gate |
| RISK-09 | Broad protocol, transport, and reporting scope delays a usable slice | High | Ordered STDIO-first story and one-ticket WIP limit; M2 work beginning before `MCPD-008` escalates | Mitigated by plan |
| RISK-10 | Package identity becomes unavailable before publication | Medium | Recheck immediately before publish and keep publisher-qualified fallback; collision blocks final metadata | Open — M3 gate |
| RISK-11 | A release channel installs bytes not represented by the immutable release | Critical | Exact package/formula equality, checksums, attestations, and native installed smokes; any mismatch requires a new version | Deferred with M3 |
| RISK-12 | An unprotected default branch permits direct, destructive, or insufficiently reviewed changes | High | `MCPD-013` requires an enforced public ruleset, drift verifier, rejected-path exercises, and a bounded emergency process; any unverified bypass or destructive path blocks M4 | Deferred with M4 |
| RISK-13 | A contributor publicly exposes a vulnerability, credential, or unsafe diagnostic because reporting and prevention controls are incomplete | High | `MCPD-014` verifies private reporting, safe guidance, entitled scanning and prevention controls, limitations, and a non-disclosing baseline; any public sensitive report or hidden finding blocks M4 | Deferred with M4 |
| RISK-14 | Mutable automation, privileged untrusted code, or unauthenticated distribution compromises the project or its releases | Critical | `MCPD-016` inventories full-SHA Actions, proves fork and permission isolation, rejects unsafe tracked artifacts, and authenticates every in-scope channel; any drift or credential exposure blocks M4 | Deferred with M4 |
| RISK-15 | Organization-owner loss or over-broad long-lived credentials become an undocumented recovery dependency | High | `MCPD-017` verifies strong MFA, lowest access, application and credential scope, owner continuity, and private recovery evidence; unresolved access or recovery assumptions block M4 | Deferred with M4 |
| RISK-16 | A stale, unofficial, or over-broad assurance claim misleads adopters | High | `MCPD-018` binds every claim to exact version, scope, date, official proof, public evidence, and removal triggers; missing, stale, withdrawn, or ambiguous proof blocks or removes the claim | Deferred with M4 |

## Readiness and completion gates

### Ticket ready

A ticket is ready when it has one observable outcome, an eligible predecessor,
explicit acceptance evidence, resolved or recorded decisions, an owner, and no
conflict with the work-in-progress limit.

### Ticket done

A ticket is done when the outcome and important failure paths work; focused and
broader checks pass; safety, redaction, and protocol claims remain accurate;
public documentation is updated; and durable evidence is linked from its row.

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

A milestone is complete when every ticket in its boundary is done, its golden
journey and native matrix pass, critical risks are mitigated or explicitly
accepted, and all preceding behavior still passes. A workflow definition alone
is not hosted CI evidence, and README prose is never release evidence.
