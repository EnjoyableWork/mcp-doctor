# mcp-doctor repository guidance

## Product and sources of truth

`mcp-doctor` is a safety-bounded Rust CLI for diagnosing Model Context
Protocol servers. Write the product name as lowercase `mcp-doctor`; format
commands, paths, protocol values, configuration keys, and filenames as code.

The public project has three repository authorities:

- [README.md](README.md) is the north-star product page. It describes the
  intended finished experience and should not become a progress diary.
- Repository policy and focused documentation define durable product, safety,
  protocol, contribution, and release contracts. They must not become a
  roadmap, ticket board, decision log, or status mirror.
- Code, tests, and published artifacts prove implemented and released
  behavior. When prose conflicts with evidence, correct the prose in the same
  change.

Do not describe a protocol revision, transport, diagnostic, platform, output
format, or installation channel as implemented unless its acceptance evidence
exists.

### Public context boundary

Everything committed to this open-source repository must be understandable and
reviewable using the repository and publicly accessible sources alone. Private
planning systems may coordinate priorities and assignments, but they are not a
product, implementation, review, or evidence authority.

- Do not put private tracker names, organization-only URLs, private issue IDs,
  internal documents, meeting notes, private conversations, or inaccessible
  evaluation material in source, comments, tests, fixtures, documentation,
  templates, commit messages, or release notes.
- Do not put coding-agent control prompts, hidden prompt mechanics, model
  reasoning, chat transcripts, runtime goal instructions, or token-budget
  instructions in repository guidance or use them to explain an implementation.
  Public MCP prompt fixtures and repository-published agent usage documentation
  are allowed when they are themselves reviewed, testable product surfaces.
- Source comments must explain the invariant, safety boundary, protocol rule, or
  non-obvious tradeoff directly. Never make a contributor retrieve an external
  discussion to learn why the code is correct.
- A privately coordinated decision that affects the repository must be
  translated into a self-contained public contract, focused change rationale,
  and acceptance evidence before or with its implementation. Omit private
  provenance rather than citing it.
- Link a GitHub issue, pull request, specification, advisory, release, or other
  source only when the referenced material is publicly accessible and useful to
  verify the statement. Durable behavior belongs in code, tests, and focused
  documentation rather than only in an issue discussion.
- Suspected vulnerabilities may remain private while coordinated under
  [SECURITY.md](SECURITY.md). Public repository text must rely only on the
  resulting advisory, release evidence, or another deliberately published
  non-sensitive record.

The project-wide north star is a safe, noninteractive server-author preflight
that identifies the earliest actionable failing layer, explains it precisely,
suggests a corrective action, and emits evidence both a human and an AI agent
can trust. Prefer causal clarity and report sufficiency over check count. When a
failure blocks dependent work, designate its layer and primary finding or
findings consistently in human and machine output, mark downstream checks as
causally skipped, and keep independent safety failures prominent.

Public assurance language is never aspirational. A security or trust statement,
badge, conformance level, or framework alignment may describe only achieved,
dated, scoped, and currently verified evidence. Follow the public-proof and
removal rules under [Documentation and release claims](#documentation-and-release-claims)
even when the README otherwise describes the finished product destination.

## Priorities

In descending order:

1. Avoid surprising execution or damage to user and production systems.
2. Bound and clean up every process, network, parsing, and generation path.
3. Produce correct, deterministic, reproducible diagnostics.
4. Keep secrets and untrusted values out of reports, logs, and test failures.
5. Make the earliest actionable failure and corrective next step clear without
   requiring raw traffic, stderr, source code, or a browser.
6. Make support claims explicit by protocol revision, transport, and platform.
7. Deliver the smallest useful vertical slice before broadening the surface.

Speed never justifies implicit tool calls, unbounded reads, orphaned children,
external schema retrieval, secret disclosure, or an unsupported compatibility
claim.

## Delivery strategy

Rust 2024 is the selected implementation language. Begin with one installable
binary crate and cohesive internal modules. A workspace, published library,
plug-in system, daemon, GUI, or hosted service requires a demonstrated consumer
and a publicly reviewable design with focused acceptance evidence.

Private planning may select and sequence work, but every repository change must
stand on a focused, observable public outcome. State dependencies, safety
impact, verification, and any remaining limitation in the public pull request
or issue when one exists. Optional work cannot become a hidden prerequisite;
make a release dependency explicit in the public change and release contract.
Do not add a roadmap, ticket board, private-decision mirror, or progress diary
to repository documentation. Assurance work must not rewrite or broaden claims
about immutable release artifacts.

## Architecture

Keep dependency direction toward protocol-independent diagnostic logic:

```text
CLI parsing and output
          |
          v
application use cases -----> reporters
          |
          +-----> diagnostic model and rules
          |
          +-----> versioned protocol adapters
          |
          `-----> transport, process, clock, and randomness ports
```

- The diagnostic model must not depend on terminal styling, operating-system
  process APIs, or a concrete HTTP client.
- Represent findings, severities, locations, skips, and outcomes with typed
  structures. Human and machine reporters consume the same result.
- Keep real protocol-version differences explicit. Do not hide incompatible
  revisions behind conditionals scattered through unrelated modules.
- Add an abstraction only when real variation exists or it protects a critical
  invariant such as cleanup, redaction, or determinism.
- Prefer focused modules named after capabilities. Avoid `utils`, `helpers`,
  and `common` grab bags.
- Avoid `unsafe`. If it becomes necessary, isolate it, document the invariant,
  and add focused tests.

## Execution and network safety

Treat process commands, environment values, URLs, headers, schemas, messages,
logs, and tool results as untrusted.

- Passive inspection is the default. Calling a tool or generating active test
  inputs requires explicit user intent and an identified target.
- Pass executable arguments literally without invoking a shell or expanding
  variables.
- Do not inherit an unrestricted environment into a tested server. Define and
  test the minimum inheritance and explicit override contract.
- Bound startup, request, response, shutdown, and total scenario time. Bound
  stdout, stderr, individual messages, aggregate output, schema work,
  generated cases, retries, redirects, and concurrency.
- Always close input, request graceful shutdown when applicable, terminate the
  full managed process tree after the grace bound, and reap children before
  returning. Cleanup failure is a diagnostic failure, not a warning hidden
  behind success.
- Never send generated or mutating tool calls merely because a schema permits
  them. Active scenarios must select tools and record their safety assumptions.
- Single-tool `check` scenarios use `mcp-doctor.scenario/v1alpha1` JSON and
  replay declared cases in order. Current-revision multi-tool `check` workflows
  use only `mcp-doctor.scenario/v2alpha1`: a finite ordered sequence with exact
  per-step tools and effects, prior-success structural captures, references into
  existing null placeholders, and an optional contiguous cleanup suffix. Every
  run repeats `--allow-tool` exactly once for every distinct selected tool;
  `side_effecting` work also requires `--allow-side-effects`. Never accept a
  wildcard, extra or duplicate authorization, generic confirmation, discovered
  selection, capture, prior result, or server annotation as authority.
- Resolve scenario secrets only from explicitly named invoking-process
  environment variables: a same-name target-environment allowlist or an
  RFC 6901 pointer to an existing null argument placeholder. Fail before starting
  the target when a reference is invalid or missing. Do not add interpolation,
  `.env`, file, command, prompt, keychain, or secret-store sources, and never
  render source names, pointers, values, resolved arguments, or results.
- Treat `input_required` as incomplete in `check`. Do not automatically answer
  elicitation, sampling, roots, or any other server request, and do not retry
  the tool call with additional input without a separately accepted contract.
- `reject` uses only MCP `2026-07-28` and one exact `--tool` plus byte-identical
  `--allow-tool`, invoking-user `--effects`, a separate
  `--allow-side-effects` gate when needed, and one `--seed`. Consider only the
  seven ordered schema-invalid mutations documented in
  [the command guide](docs/commands.md#schema-invalid-reject-cases), start from
  bounded locally valid object
  evidence, and transmit a case only after the local validator proves exactly
  one mismatch. Skip inapplicable or unencodable work and stop before a call
  for invalid, external-reference, unsatisfiable, or over-limit schemas.
- A `reject` case passes only for a matching well-formed JSON-RPC error with
  integer code `-32602` and a string message. Never match or retain the message
  or error data. Treat any result—including `isError: true` or
  `input_required`—as critical unsafe acceptance and stop later calls; another
  error code or malformed error is a distinct active-contract failure and also
  stops.
- Follow the Streamable HTTP safety contract in
  [the command guide](docs/commands.md) and [safety guide](docs/safety.md).
  Accept one strict canonical endpoint,
  use public HTTPS by default, and require an exact matching
  `--allow-private-network` for eligible loopback/private destinations plus an
  exact `--allow-cleartext-http` for credential-free all-loopback HTTP. Resolve
  once under a 16-address cap, reject mixed or prohibited IANA address classes,
  pin the accepted set, and verify every connected peer. Never accept a
  wildcard, CIDR, suffix, ambient setting, or prior run as network authority.
- Keep remote connections direct with zero redirects and application retries.
  Ignore inherited and platform proxies, proxy credentials, trust-store
  environment overrides, cookies, caches, service binding, and alternate
  origins. TLS supports only 1.2 or 1.3 with full chain and service-identity
  verification; there is no insecure or credential-bearing cleartext fallback.
- Resolve remote bearer tokens and custom field values only from explicitly
  named invoking-process environment variables, and require an exact matching
  `--allow-credentials-to` HTTPS endpoint. Do not accept URL, command-line,
  `.netrc`, keychain, cookie, proxy, client-certificate, or inherited credential
  sources. Do not fetch OAuth metadata or start authorization, registration,
  browser, refresh, or step-up flows; report `401` and `403`
  structurally without rendering or following challenge values.
- Keep the stateless MCP `2026-07-28` POST binding as the default and sole
  implicit active HTTP binding. Passive `inspect` defaults to the finite
  transport-defined `auto` contract in
  [protocol support](docs/protocol-support.md): one modern discovery, at most
  one legacy-era transition, no modern retransmission or sequential guessing,
  and one shared deadline and aggregate budget. STDIO must reap the first tree
  before its one permitted legacy launch. Streamable HTTP must reuse the one
  prepared canonical endpoint, pinned address set, peer checks, trust, and
  credential authority without re-resolution. An explicit passive revision is
  a strict one-lifecycle hard pin. The exact-selected `check` and
  `break` may use the shared MCP `2025-11-25` and `2025-06-18` adapters but
  never negotiate, retry, or fall back. MCP `2025-06-18` activity also requires
  the exact supported Draft
  2020-12 declaration on every advertised schema interpreted for activity.
  Generate the exact protocol headers, validate and bound `x-mcp-header`
  mappings without treating them as execution authority, support bounded JSON
  and request-scoped SSE responses, and retain no URL, DNS answer, IP, header,
  certificate identity, challenge, body, credential source, or value in either
  reporter. The complete finite field, trust, body, time, and activity limits
  documented by the product are release blockers, not implementation
  suggestions.
- Do not fetch external JSON Schema references by default. Bound reference
  depth, instance size, validation work, and error collection.

## Redaction and diagnostics

- Never print authentication headers, environment values, raw tool arguments,
  raw tool results, server stderr, credential-bearing URLs, or complete
  unreviewed messages in ordinary output or errors.
- Prefer structural diagnostics: field path, type, byte count, item count,
  protocol method, and redacted identifier.
- Error types may retain safe context but must not wrap an untrusted payload in
  a `Display` or `Debug` implementation that can escape to the terminal.
- Machine-readable output follows the same redaction boundary as human output.
  A debug or evidence mode that exposes values requires a separate explicit
  threat model and is not an accidental verbosity flag.
- A reported success must distinguish checks performed from checks skipped.
  Never imply active behavior was tested when it was not authorized.
- When findings form a causal chain, identify the earliest actionable layer and
  primary finding or findings, and make each dependent skip reference that
  diagnosis. Do not repeat one cause as unrelated downstream failures. Preserve
  independent findings, especially cleanup, redaction, authorization, and
  resource-bound failures, even when another diagnosis occurs earlier.

## Protocol and schema policy

- Pin every conformance rule to an identified MCP revision or accepted
  compatibility range.
- Keep fixtures for every claimed revision and test version negotiation or
  rejection deliberately.
- MCP `2026-07-28` remains the default and sole implicit active revision.
  Passive `inspect` defaults to bounded `auto` and may select only the compiled
  current revision or, after the exact transport-specific legacy signal, one
  supported `2025-11-25` or `2025-06-18` initialization. It never retransmits a
  modern request, guesses sequential revisions, exceeds one legacy transition,
  overlaps STDIO children, re-resolves HTTP, or broadens endpoint, peer,
  credential, time, message, or output authority. An explicit passive revision
  is a hard pin with no probe, retry, fallback, or downgrade. `check` and
  `break` where the
  [protocol support matrix](docs/protocol-support.md#support-matrix) marks them
  supported may initialize legacy only through an exact explicit selection.
  Recognize `2025-03-26` and `2024-11-05` only for precise unsupported
  diagnostics. Follow that complete revision and transport matrix;
  `reject` never selects or claims a legacy revision.
- Broad current-revision positioning requires every selected official and
  independent current-revision
  case to pass across at least two languages. Narrower credible reach requires
  explicit readiness or migration language and a separately reviewed public
  compatibility change. Broad positioning remains blocked until credible
  independent evidence passes.
- Follow the supported JSON Schema dialect exactly. Reject unsupported or
  ambiguous behavior with a typed diagnostic rather than guessing.
- Do not use live network documentation as runtime behavior. Record the
  accepted protocol contract and update it through a reviewed public change
  when the specification changes.

## Dependencies

- Adding no dependency is the default. Prefer the standard library and the
  existing graph; do not add a crate or tool for a trivial helper, convenience,
  speculative future use, or functionality that the project already has.
- Every runtime, build, development, and standalone testing dependency requires
  a concrete capability that the existing graph cannot safely or reasonably
  provide and a public review record containing the required evidence. Test-only
  code still executes in contributor and CI environments and receives the same
  supply-chain scrutiny as product code.
- Before adoption, record the selected release and review its upstream
  stewardship, release and issue activity, security-response path, ownership or
  provenance changes, license compatibility, advisories, unsafe code and build
  scripts, default and selected features, transitive graph, duplicate versions,
  minimum Rust version, supported platforms, and build, binary, startup, and
  runtime cost. Popularity or a recent publish date alone is not evidence of
  reliability; a mature stable crate may release infrequently, but it must
  still have credible maintainership and a safe response path.
- Use an exact `=x.y.z` requirement for every direct registry dependency,
  including build and development dependencies. Commit `Cargo.lock` because
  this repository ships an application, and use `--locked` in normal build,
  test, package, install, and policy commands. The lockfile fixes the complete
  resolved graph; exact direct requirements make intentional upgrades visible
  in the manifest. Neither control proves that selected source is trustworthy.
- Use stable releases only. A pre-release requires an explicit public review
  and focused evidence. Git dependencies, alternate registries, unpublished
  forks, and unpinned remote test data are prohibited unless an explicit public
  review updates the source policy and records why crates.io cannot meet the
  need.
- Disable default features when the required capability can be selected
  narrowly. A feature expansion is a dependency change and receives the same
  review as a new crate.
- Dependency-update pull requests are proposals, never automatic approvals.
  Review release notes, upstream ownership and activity, the manifest and
  lockfile diff, new transitive crates and features, licenses, advisories,
  platform and Rust-version changes, and relevant behavior tests before merge.
  Prefer a focused update when a grouped change obscures causality.
- Pin standalone CI and diagnostic tools to an exact release or immutable
  revision at first use, record the version with its evidence, and run invasive
  tools in disposable or copied trees. Select versions when the focused change
  is reviewed rather than freezing speculative choices in advance.
- An exact immutable, size- and digest-pinned CI artifact may be acquired at
  most three times when a bounded attempt fails with an explicitly classified
  transient transport error or `408`, `429`, `500`, `502`, `503`, or `504`.
  Delete partial bytes between attempts and apply the same final integrity
  checks regardless of which attempt succeeds. Never retry TLS or trust
  failures, permanent responses, checksum or layout mismatches, tool execution,
  builds, tests, SBOM generation, validation, publication, jobs, or workflows;
  an eventual acquisition is availability evidence, not correctness evidence.
- Conditional tools default to rejection. Adopt one only through a focused
  public review that records a concrete measured need, the complete review
  above, its narrow use, and pull-request evidence.
- Do not implement JSON Schema validation, HTTP framing, process-tree control,
  cryptography, or fuzz-generation semantics casually when a maintained,
  reviewed implementation is safer.
- Record dependency and testing-tool introductions in the public pull request
  and keep `deny.toml` current. A new license, source, duplicate version,
  ignored advisory, or broad exception requires an explicit public review.
  Remove or replace a dependency when its need disappears or its maintenance,
  security, or reliability no longer satisfies this policy.

## Testing and verification

Tests should prove behavior at the narrowest useful layer:

- unit tests for validation, normalization, redaction, limits, deterministic
  generation, and outcome classification;
- protocol fixtures for every supported message and schema shape;
- transport tests with synthetic processes and local disposable HTTP servers;
- built-binary journeys for complete CLI behavior and exit codes; and
- native platform tests for every advertised process or release boundary.

Golden failure fixtures must record the expected earliest actionable layer and
corrective next step. Built-binary tests must prove that human and machine
reports agree on the primary diagnosis, independent findings, and causal skips,
and that an ordinary report alone contains enough safe evidence to recover the
intended correction.

Never call a real MCP server or production endpoint in the default suite.
Tests must clear inherited environment where practical and keep files, sockets,
and process fixtures inside disposable roots. Use unmistakably synthetic values
and assertions that do not print payloads when they fail.

A bug involving implicit execution, cleanup, redaction, limits, version
handling, or a false success claim requires a regression test.

### Deterministic CI policy

- Use timeouts only as product contracts or outer watchdogs that bound failed
  work. Read a wall clock only when the product contract under test explicitly
  consumes verification time, and keep the accepted range stable across
  ordinary clock transitions. Never use elapsed time, a wall-clock-derived
  expected value, a fixed sleep, or a fast polling interval as synchronization,
  readiness, or proof.
- Synchronize asynchronous and process fixtures through observable state or an
  explicit event/acknowledgement handshake. Polling may observe eventual
  operating-system state only when that state is itself the success condition
  and one outer deadline prevents an indefinite hang.
- Treat pass/fail variance on identical source as an unresolved defect. A green
  rerun is evidence of nondeterminism, not acceptance evidence; preserve the
  failed attempt, classify it in a public issue or pull-request risk record, and
  do not add an automatic test, job, or workflow retry.
- A narrowly bounded retry is permitted only for the idempotent download of one
  immutable size- and digest-pinned Syft asset documented in
  [the deterministic CI audit](docs/deterministic-ci.md), only for its
  classified transient transport failures, and only under its three-attempt
  limit. Never
  retry a test, build, integrity check, publication, complete job, or workflow
  as correctness evidence.
- Correct the lowest timing-dependent contract and add one deterministic
  regression that forces the relevant state transition or interleaving.
  Repetition and stress execution are supplemental evidence only.
- Do not make CI green by increasing a product timeout, weakening an assertion,
  skipping or quarantining a safety regression, or broadly serializing tests.
  Focused serialization is acceptable only for a demonstrated exclusive
  resource and must be documented at its call site.
- Declare every non-standard CI command and verify it before use. Acquire any
  repository-managed executable through its reviewed exact identity; never
  depend on an incidental runner-image tool.
- Keep built-binary and native-host tests for contracts that genuinely cross a
  process, filesystem, packaging, network-fixture, or operating-system boundary.
  Prove separable behavior at a narrower layer first. Workflow
  `timeout-minutes`, socket deadlines, and channel receive deadlines remain
  outer watchdogs, not fixture readiness or product-behavior evidence.

The normal substantive-change handoff checks are:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
3. `cargo test --workspace --all-targets --all-features --locked`
4. `cargo deny --all-features --locked check`
5. Any affected protocol fixture, package, release, or documentation checks

Use `scripts/check.sh` on POSIX systems and `scripts/check.ps1` on Windows for
the first three checks through a disposable user environment.

## Documentation and release claims

- Preserve the README as a polished description of the destination. Do not turn
  it or focused documentation into a delivery-status mirror. Public GitHub
  issues and pull requests may describe scoped proposals and implementation;
  only shipped evidence may be described as available behavior.
- Update public behavior, safety boundaries, and examples with the code that
  changes them.
- Use generic names and synthetic values. Never commit local user paths,
  credentials, private endpoints, or raw production output.
- GitHub Releases are the canonical immutable release record. Cargo and
  Homebrew must consume the same version and source identity.
- Do not publish a channel until its installed binary completes the release
  smoke journey on every represented native host.
- A stable macOS or Windows binary requires the signing and native verification
  contract accepted for that platform; unsigned artifacts are not a silent
  fallback.
- Publish no assurance badge or trust claim until a public evidence record
  verifies the official proof, exact framework version, scope, date, public
  evidence, and rendered destination on exact `main`.
- Correct or remove assurance language immediately when its framework, issuer,
  scope, evidence, repository controls, organization boundary, or release
  pipeline changes invalidate the claim.

## Version control and handoff

- Inspect `git status --short` before editing and preserve unrelated changes.
- Use `rg` and `rg --files` for discovery before adding parallel behavior.
- Follow Conventional Commits: `<type>[optional scope]: <imperative summary>`.
- Do not commit, push, rewrite history, create tags, publish releases, or open
  pull requests unless the user requests it.
- At handoff, state the change outcome, files changed, checks run, and any
  remaining assumption, risk, or unverified external gate.
