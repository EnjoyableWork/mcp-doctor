# Category audit rubric v1

**Version:** 1.0

**Frozen:** 2026-08-18

**Applies to:** Released or immutably identified MCP diagnostic, testing, and
inspection tools

This is the public scoring and evidence contract used for the dated
[`mcp-doctor` v0.4.0 standalone evaluation](v0.4.0.md). It evaluates observable
capability and proof quality. It does not score roadmap intent, private
demonstrations, popularity, marketing volume, or unpublished work, and it
cannot establish certification or category leadership.

## Evaluation independence

Every evaluation using this rubric follows this sequence:

1. Freeze the rubric version, immutable subject identities, comparison
   identities, evidence cutoff, environment, and allowed sources.
2. Prepare a neutral, row-addressable evidence pack without proposed states,
   points, totals, deltas, prior results, desired outcomes, or rankings.
3. Record whether the evaluator saw a prior total, proposed row state,
   predicted delta, target score, desired rank, competitor result, or scored
   worksheet before evidence review.
4. Record one `Full`, `Partial`, or `Zero` state, evidence, rationale, and
   limitations for every row, then lock all rows before calculating points.
5. Calculate mechanically from the locked states. A separate reviewer verifies
   row completeness and arithmetic without changing classifications.
6. Compare subjects only after every subject has a fresh, independently locked
   worksheet under the same rubric and cutoff.
7. Review safety, redaction, cleanup, determinism, causal diagnosis, protocol,
   compatibility, packaging, and authority regressions separately. A total
   cannot offset one of these failures.

If prohibited exposure occurs, stop and use an unexposed evaluator or label the
result non-independent. A non-independent result cannot support a public
comparison, rank, score badge, or leadership claim.

### Evaluator provenance and portability

A public result records the evaluator execution surface and any provider,
model, serving revision, inference settings, and run count made available by
that surface. Missing metadata is reported as unavailable, not inferred.

One language-model evaluation does not establish repeatability or portability
across models or providers. Such a result must be labeled single-evaluator and
uncalibrated. This publication clarification does not alter any v1 criterion,
weight, evidence minimum, state, or arithmetic rule.

## Scoring model

| Category | Criteria | Maximum |
| --- | ---: | ---: |
| Protocol correctness and coverage | 5 | 25 |
| Agent and LLM-facing quality | 5 | 25 |
| Security and safety | 5 | 20 |
| Developer experience and CI | 5 | 20 |
| Trust and public proof | 5 | 10 |
| **Total** | **25** | **100** |

Each criterion has exactly three states. `Full` receives the criterion maximum,
`Partial` receives only its stated value, and `Zero` receives zero. There are
no fractional, interpolated, bonus, or reviewer-adjusted points. Missing
minimum evidence resolves to `Zero`. Reviewer uncertainty resolves from
`Full` to `Partial` only when the partial condition and its evidence are
proven; otherwise it resolves to `Zero`.

For a safety clause only, a risky surface that is demonstrably absent and
unclaimed is closed by absence. That creates no bonus points. Category
subtotals are the sums of their five rows; the total is the sum of the five
subtotals.

## Evidence hierarchy

| Tier | Evidence class | Required identity |
| --- | --- | --- |
| E1 | Released-artifact observation | Installed artifact or source archive with digest, exact command, synthetic fixture, platform/runtime identity, and structural outcome |
| E2 | Immutable automated evidence | Public tests or CI tied to the exact release commit, with inspectable fixtures and passing status |
| E3 | Immutable source or policy evidence | Source, configuration, documentation, or policy at an exact commit |
| E4 | Immutable release metadata | Release notes, registry metadata, checksums, attestations, signatures, or changelog tied to the release |
| E5 | Mutable public claim | Default-branch prose, website, screenshot, badge, or registry page not pinned to the evaluated identity |
| E6 | Inference or nonpublic material | Reviewer inference, roadmap text, inaccessible evidence, private transcript, endpoint, credential, or unpublished note |

An E1 failure overrides a lower-tier success claim for runtime behavior. E2
earns credit only when public, passing, tied to the evaluated identity, and
testing the scored boundary. E3 proves static properties but cannot override a
contradictory artifact observation. E4 proves release identity and controls,
not runtime behavior. E5 is only a discovery lead until pinned. E6 earns no
credit. Newer source or tests never earn points for an older artifact.

Every nonzero row identifies the subject, criterion, state, evidence tier,
immutable source or exact command, synthetic fixture, environment, structural
outcome, skipped checks, and limitations. Evidence never retains credentials,
environment values, raw tool arguments or results, private endpoints, local
user paths, target stderr, prompts, transcripts, or unpublished material.

## P. Protocol correctness and coverage — 25 points

### P1 — Revision identity and negotiation (5 points)

- **Full — 5:** A released artifact completes `initialize` and `initialized`
  for at least one explicitly declared MCP revision; the report identifies the
  selected revision; and a separate mismatch fixture is rejected with a
  structured revision diagnostic without discovery, downgrade, fallback, or
  retry.
- **Partial — 3:** An exact declared revision completes initialization, but the
  selected revision is absent or the mismatch/no-fallback case is not publicly
  proven.
- **Zero — 0:** Initialization is not reproducible, revision behavior is
  claim-only, or the tool silently discovers, retries, downgrades, or falls
  back.
- **Minimum evidence:** E1 success and mismatch observations; E2 may corroborate
  but cannot replace E1.

### P2 — Standard transport coverage (5 points)

- **Full — 5:** The same selected revision completes a bounded initialization
  and capability-list journey over both STDIO and Streamable HTTP with correct
  framing and headers, without silently changing transport.
- **Partial — 3:** Exactly one standard transport completes the journey, or the
  second is supported only by immutable source evidence without a
  released-artifact observation.
- **Zero — 0:** Neither standard transport completes a protocol journey, or
  only a proprietary or legacy-only path is shown.
- **Minimum evidence:** E1 observations using the same synthetic behavior and
  declared revision.

### P3 — JSON-RPC lifecycle and message correctness (5 points)

- **Full — 5:** Controlled fixtures prove identifier matching, initialization
  ordering, initialized notification handling, capability-list request and
  response validation, well-formed error handling, and graceful close or
  session termination where applicable.
- **Partial — 3:** Happy initialization and listing succeed and identifiers are
  validated, but ordering, notification, error shape, or close/session behavior
  is incomplete.
- **Zero — 0:** Only reachability is proven, a malformed JSON-RPC shape is
  accepted, or identifiers are not validated.
- **Minimum evidence:** E1 controlled success and malformed-message
  observations.

### P4 — Capability inventory coverage (5 points)

- **Full — 5:** The released artifact enumerates and structurally validates
  advertised tools, resources, and prompts, including behavior when each is
  absent or unadvertised.
- **Partial — 3:** At least one but fewer than all three capability families is
  enumerated and validated.
- **Zero — 0:** No MCP capability family is enumerated, or only unvalidated raw
  payloads are rendered.
- **Minimum evidence:** E1 synthetic catalog observations; E2 fixtures may prove
  absent-capability cases.

### P5 — Schema and contract validation (5 points)

- **Full — 5:** The artifact selects the dialect required by the chosen
  revision, validates local references and structural constraints, rejects
  controlled invalid schemas, validates advertised input and output schemas,
  and separates malformed protocol errors from valid tool-level errors.
- **Partial — 3:** Schema syntax and at least one advertised input contract are
  validated, but dialect, local-reference, output-schema, or error
  classification evidence is incomplete.
- **Zero — 0:** Schemas are display-only, controlled invalid schemas pass, or
  the dialect is guessed without a typed limitation.
- **Minimum evidence:** E1 controlled catalogs plus E2 validator fixtures.

## A. Agent and LLM-facing quality — 25 points

### A1 — Tool-description hygiene (5 points)

- **Full — 5:** Deterministic, versioned rules cover all three fixed cases:
  missing or whitespace-only descriptions; descriptions equal to the
  normalized tool name or a published placeholder; and identical normalized
  descriptions reused by distinct tools.
- **Partial — 3:** One or two fixed cases are covered with published
  normalization.
- **Zero — 0:** No description-quality diagnostic exists, or it relies only on
  undisclosed model judgment or a mutable prompt.
- **Minimum evidence:** E1 controlled catalogs and E2 fixed-rule fixtures.

### A2 — Required-input documentation (5 points)

- **Full — 5:** Every required object property lacking a nonblank description
  is diagnosed recursively, including nested objects and array-item objects;
  invalid schemas are distinguished from undocumented valid contracts; only a
  structural location is reported.
- **Partial — 3:** Only top-level required properties are checked, or invalid
  schemas are not distinguished from missing documentation.
- **Zero — 0:** No required-input documentation check exists, or it renders an
  untrusted value as evidence.
- **Minimum evidence:** E1 nested synthetic schemas and E2 recursion,
  redaction, and classification fixtures.

### A3 — Context-efficiency evidence (5 points)

- **Full — 5:** The report gives a deterministic normalized catalog byte count,
  per-entry counts, and ranked contributors; warnings use numeric thresholds
  fixed by the evaluated version.
- **Partial — 3:** One or two of aggregate size, per-entry size, or ranked
  contributors are present with a fixed threshold.
- **Zero — 0:** No quantitative context evidence exists, only a subjective
  verbosity judgment is reported, or thresholds are mutable.
- **Minimum evidence:** E1 fixed-catalog observation and E2 normalization and
  threshold fixtures.

### A4 — Actionable and causal diagnostics (5 points)

- **Full — 5:** Every controlled failure identifies the earliest actionable
  layer and primary finding, stable code and severity, structural location,
  corrective next step, causally linked dependent skips, and independent
  safety failures.
- **Partial — 3:** A stable code or layer, structural location, and correction
  exist, but primary-cause selection, skip linkage, or preservation of
  independent safety failures is not proven.
- **Zero — 0:** A controlled failure yields only generic prose, raw payload or
  stderr, or unrelated failures for one cause.
- **Minimum evidence:** E1 golden failure journeys and E2 reporter-parity
  fixtures.

### A5 — Agent-consumable result contract (5 points)

- **Full — 5:** Machine output is versioned and deterministic, represents
  performed/skipped/not-applicable state explicitly, has stable exits, and
  agrees with the redacted human result and primary diagnosis.
- **Partial — 3:** Machine output and stable exits exist, but schema versioning,
  deterministic order, explicit skip state, or human/machine parity is not
  proven.
- **Zero — 0:** No machine-readable result exists, exits are unstable, or
  machine output crosses the ordinary redaction boundary.
- **Minimum evidence:** E1 repeated artifact runs and E2 schema and reporter
  parity tests.

## S. Security and safety — 20 points

### S1 — Non-surprising execution authority (4 points)

- **Full — 4:** Passive inspection is the default; no tool call, host/config
  scan, side effect, or credential use occurs beyond the exact target without
  explicit authority; active work requires exact tool selection and a separate
  side-effect gate; wildcards and ambient discovery are rejected.
- **Partial — 2:** Passive inspection is default, but an active or discovery
  path uses confirmation or authority broader than one exact target or tool.
- **Zero — 0:** A default or nominally passive run can call a tool, scan ambient
  configuration, use credentials, or mutate without exact authority.
- **Minimum evidence:** E1 negative-authority journeys and E2 CLI authorization
  fixtures.

### S2 — Credential sources and redaction (4 points)

- **Full — 4:** Credentials come only from explicitly named sources; values are
  rejected from command arguments and URLs; ordinary human, machine, error,
  and test output retains no values, source names, raw arguments, results, or
  stderr; every reporter has regression fixtures.
- **Partial — 2:** Controlled values are absent, but credential sources are
  broader than explicit names or reporter/error coverage is incomplete.
- **Zero — 0:** A credential can be supplied through a rendered command or URL,
  or a controlled value appears in ordinary output.
- **Minimum evidence:** E1 canary observations across reporters and errors plus
  E2 redaction tests.

### S3 — Resource bounds and cleanup (4 points)

- **Full — 4:** Applicable startup, request, response, shutdown, and total times
  are finite; message and aggregate streams are capped; redirects, retries,
  cases, and concurrency are finite; managed process trees are gracefully
  closed, terminated after a bound, and reaped, with cleanup failure reported.
- **Partial — 2:** A total deadline, stream cap, and cleanup path exist, but an
  aggregate bound, tree guarantee, retry/redirect bound, or cleanup-failure
  outcome is missing.
- **Zero — 0:** A core path is unbounded, or a controlled timeout can orphan a
  managed process.
- **Minimum evidence:** E1 limit and forced-cleanup journeys plus E2 boundary
  fixtures.

### S4 — Untrusted schema and network containment (4 points)

- **Full — 4:** Parsing, nesting, references, validation, and generation are
  bounded; external references are not fetched by default; remote connections
  use exact endpoints without ambient proxies, redirects, or insecure
  verification; private, cleartext, and credential access have separate exact
  gates where supported.
- **Partial — 2:** External retrieval is disabled and depth/size are bounded,
  but a validation, generation, endpoint, proxy, redirect, peer, or access gate
  remains unproven.
- **Zero — 0:** External references are fetched by default, ambient
  proxy/redirect behavior is followed, peer verification is disabled, or a
  controlled input is unbounded.
- **Minimum evidence:** E1 hostile-input and network observations plus E2 limit
  fixtures.

### S5 — Server-facing security diagnostics (4 points)

- **Full — 4:** Fixed, documented, redacted diagnostics cover credential
  literals in advertised schemas or inspected configuration; unsafe credential
  transmission or storage; and execution or side-effect authority beyond the
  selected operation.
- **Partial — 2:** One or two fixed classes have controlled fixtures and
  structural redacted output.
- **Zero — 0:** No class is diagnosed, or a diagnostic retains the sensitive
  value it detects.
- **Minimum evidence:** E1 synthetic findings and E2 redaction and
  classification fixtures.

## D. Developer experience and CI — 20 points

### D1 — Installability and native-platform reach (4 points)

- **Full — 4:** A stable canonical binary or package maps to the evaluated
  release, and its installed-binary smoke journey passes on Linux, macOS, and
  Windows.
- **Partial — 2:** A canonical artifact passes on one or two platforms, or only
  a locked source build is reproducible.
- **Zero — 0:** Installation depends on a moving branch, unidentified artifact,
  or irreproducible source procedure.
- **Minimum evidence:** E1 native installed-artifact observations with version
  and digest.

### D2 — Noninteractive CLI contract (4 points)

- **Full — 4:** The CLI is noninteractive, has complete help, selects target,
  transport, and revision explicitly, passes executable arguments literally,
  validates authority before execution, and documents stable exits.
- **Partial — 2:** A noninteractive path and stable exits exist, but explicit
  selection, literal arguments, pre-execution validation, or help is missing.
- **Zero — 0:** A GUI or prompt is required, execution relies on a shell command
  string, or no stable automation exit exists.
- **Minimum evidence:** E1 artifact CLI journeys and E3 immutable CLI docs.

### D3 — Report format coverage (4 points)

- **Full — 4:** One run emits all four report classes from the same result:
  human-readable, versioned JSON, a CI exchange format such as JUnit or SARIF,
  and a review artifact such as Markdown or HTML.
- **Partial — 2:** Two or three report classes are reproducible.
- **Zero — 0:** At most one class is reproducible, or formats disagree.
- **Minimum evidence:** E1 same-run observations and E2 reporter-parity tests.

### D4 — CI integration completeness (4 points)

- **Full — 4:** Public docs provide an immutable installation, least-permission
  noninteractive invocation, machine/report artifact publication, and explicit
  branch or pull-request behavior derived from stable exits.
- **Partial — 2:** Exactly two or three elements are publicly reproducible.
- **Zero — 0:** Fewer than two elements exist, or a badge lacks a diagnostic run
  determining its state.
- **Minimum evidence:** E1 public workflow observation and E3 immutable CI docs.

### D5 — Bounded performance and scale evidence (4 points)

- **Full — 4:** The artifact produces bounded, reproducible evidence for a
  latency distribution beyond a mean, controlled concurrency, and payload or
  catalog-size behavior, including samples, workload, concurrency, and
  deadlines.
- **Partial — 2:** One or two dimensions are measured reproducibly with numeric
  workload and bounds.
- **Zero — 0:** No dimension is measured, stress is unbounded, or timing lacks
  sample and workload identity.
- **Minimum evidence:** E1 bounded profiling observations and E2 deterministic
  statistic and limit tests.

## T. Trust and public proof — 10 points

### T1 — Release identity and integrity (2 points)

- **Full — 2:** An immutable public release has checksums and public provenance
  or a signature tying distributed artifacts to source.
- **Partial — 1:** An immutable version or tag exists, but integrity,
  provenance, signature, or source linkage is incomplete.
- **Zero — 0:** Only moving source or an unverifiable installed artifact exists.
- **Minimum evidence:** E1 artifact identity and E4 release metadata.

### T2 — Public quality and security controls (2 points)

- **Full — 2:** Public CI covers claimed revisions and native platforms, a
  public security policy provides a reporting path, and required checks have
  no unexplained failure.
- **Partial — 1:** Exactly one of claim-covering CI or a public security path is
  present.
- **Zero — 0:** Neither exists, or required checks are known failing without a
  scoped limitation.
- **Minimum evidence:** E2 immutable CI and E3/E4 security policy.

### T3 — Reproducible public evidence (2 points)

- **Full — 2:** Public synthetic fixtures and exact commands reproduce major
  protocol, failure, redaction, and report claims against the released artifact
  without private infrastructure.
- **Partial — 1:** Public tests or fixtures exist, but an artifact recipe or one
  proof class is missing.
- **Zero — 0:** Only narrative, screenshots, or inaccessible evidence exists.
- **Minimum evidence:** E1 public reproduction and E2 immutable fixtures.

### T4 — Transparent grade or badge semantics (2 points)

- **Full — 2:** A public score, grade, or badge states tool version, evaluated
  scope, date, performed and skipped checks, limitations, evidence destination,
  and review/removal policy.
- **Partial — 1:** Published criteria exist but at least one required context
  field is omitted.
- **Zero — 0:** No score/grade/badge exists, or its meaning is opaque,
  aspirational, misleading, or presented as certification.
- **Minimum evidence:** E3/E4 rendered destination tied to the release.

### T5 — Independent interoperability reach (2 points)

- **Full — 2:** Public reproducible evidence shows the artifact passing against
  at least two independent current-revision MCP servers in at least two
  implementation languages, with exact server and tool identities.
- **Partial — 1:** Evidence covers only an official/reference implementation,
  one independent implementation, or multiple implementations in one language.
- **Zero — 0:** No public exact-identity interoperability evidence exists.
- **Minimum evidence:** E1 artifact journeys and immutable server identities.

## Freshness, corrections, and removal

An evaluation is a historical snapshot and is never silently rewritten. A
comparison or leadership statement is current for at most 90 calendar days.
A new subject release, newly current MCP revision, removed artifact, changed
public control, or contradictory reproducible evidence triggers review before
the result is presented as current. A refreshed comparison resolves every
subject under one new shared cutoff; it never mixes fresh and stale totals.

An evidence correction amends only affected locked rows with a dated record and
then recalculates mechanically. A change to a weight, state condition, evidence
minimum, identity rule, arithmetic rule, or freshness window creates a new
rubric version and requires every compared subject to be rescored. A procedure
clarification that cannot change a state or points may change only publication
eligibility. If evidence disappears or a claim becomes false, remove or narrow
the current claim while retaining the dated historical record and limitation.
