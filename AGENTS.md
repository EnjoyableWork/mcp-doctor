# mcp-doctor repository guidance

## Product and sources of truth

`mcp-doctor` is a safety-bounded Rust CLI for diagnosing Model Context
Protocol servers. Write the product name as lowercase `mcp-doctor`; format
commands, paths, protocol values, configuration keys, and filenames as code.

The repository has three distinct authorities:

- [README.md](README.md) is the north-star product page. It describes the
  intended finished experience and should not become a progress diary.
- [PROJECT.md](PROJECT.md) is the current delivery plan, decision record, risk
  register, ticket board, and implementation truth.
- Code, tests, and published artifacts prove implemented and released
  behavior. When prose conflicts with evidence, correct the prose in the same
  change.

Do not describe a protocol revision, transport, diagnostic, platform, output
format, or installation channel as implemented unless its acceptance evidence
exists.

The project-wide north star is a safe, noninteractive server-author preflight
that identifies the earliest actionable failing layer, explains it precisely,
suggests a corrective action, and emits evidence both a human and an AI agent
can trust. Prefer causal clarity and report sufficiency over check count. When a
failure blocks dependent work, designate its layer and primary finding or
findings consistently in human and machine output, mark downstream checks as
causally skipped, and keep independent safety failures prominent.

Public assurance language is never aspirational. A security or trust statement,
badge, conformance level, or framework alignment may describe only achieved,
dated, scoped, and currently verified evidence. Follow the M4 public-proof and
removal policy in `PROJECT.md` even when the README otherwise describes the
finished product destination.

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
and an accepted decision in [PROJECT.md](PROJECT.md).

Follow the ordered main story in `PROJECT.md`. Associate every material change
with a ticket, keep one main-story ticket in progress, and link durable
evidence before marking it done. Optional work cannot become a hidden
prerequisite; promote it into the ordered story if the release depends on it.
M4 assurance work begins only after the independently verified M3 release and
must not rewrite or broaden claims about its immutable artifacts.

When a runtime supports persistent goals, use the active ticket's objective as
the thread goal. A goal cannot waive dependencies, safety rules, open decisions,
or evidence gates. Do not assign a token budget unless the user explicitly
requests one.

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
- `check` scenarios use only `mcp-doctor.scenario/v1alpha1` JSON, name one tool,
  and replay declared cases in order. Every run requires an exact matching
  `--allow-tool`; every scenario declares `read_only` or `side_effecting`; and
  side effects also require `--allow-side-effects`. Never accept a wildcard,
  generic confirmation, discovered selection, or server annotation as authority.
- Resolve scenario secrets only from explicitly named invoking-process
  environment variables: a same-name target-environment allowlist or an
  RFC 6901 pointer to an existing null argument placeholder. Fail before starting
  the target when a reference is invalid or missing. Do not add interpolation,
  `.env`, file, command, prompt, keychain, or secret-store sources, and never
  render source names, pointers, values, resolved arguments, or results.
- Treat `input_required` as incomplete in `check`. Do not automatically answer
  elicitation, sampling, roots, or any other server request, and do not retry
  the tool call with additional input without a separately accepted contract.
- Do not follow remote redirects or resolve local/private network targets under
  an implicit policy. The Streamable HTTP ticket must define its SSRF,
  redirect, authentication, TLS, and proxy boundaries before implementation.
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
- The supported revision is MCP `2026-07-28`. Recognize the four earlier
  official handshake-based revisions for precise diagnostics, but do not send
  `initialize`, fall back to them, or imply compatibility. Follow `DEC-013` and
  the MCPD-004 matrix in `PROJECT.md`.
- Follow `DEC-024` when describing real-server reach. Broad current-revision
  positioning requires every selected official and independent current-revision
  case to pass across at least two languages. Narrower credible reach requires
  explicit readiness or migration language and a separate compatibility ticket;
  no credible independent pass blocks M1.
- Follow the supported JSON Schema dialect exactly. Reject unsupported or
  ambiguous behavior with a typed diagnostic rather than guessing.
- Do not use live network documentation as runtime behavior. Record the
  accepted protocol contract and update it through a reviewed ticket when the
  specification changes.

## Dependencies

- Adding no dependency is the default. Prefer the standard library and the
  existing graph; do not add a crate or tool for a trivial helper, convenience,
  speculative future use, or functionality that the project already has.
- Every runtime, build, development, and standalone testing dependency requires
  an owning ticket and a concrete capability that the existing graph cannot
  safely or reasonably provide. Test-only code still executes in contributor
  and CI environments and receives the same supply-chain scrutiny as product
  code.
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
- Use stable releases only. A pre-release requires an accepted decision and
  focused evidence. Git dependencies, alternate registries, unpublished forks,
  and unpinned remote test data are prohibited unless an accepted decision
  updates the source policy and records why crates.io cannot meet the need.
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
  tools in disposable or copied trees. Select versions when the owning ticket
  starts rather than freezing speculative choices in advance.
- Conditional tools default to rejection. Under `DEC-025`, the owning ticket may
  adopt one without separate owner approval only after recording a concrete
  measured need, the complete review above, its focused use, and pull-request
  evidence.
- Do not implement JSON Schema validation, HTTP framing, process-tree control,
  cryptography, or fuzz-generation semantics casually when a maintained,
  reviewed implementation is safer.
- Keep the dependency and testing-tool introduction plan in `PROJECT.md` and
  `deny.toml` current. A new license, source, duplicate version, ignored
  advisory, or broad exception requires an explicit review. Remove or replace a
  dependency when its need disappears or its maintenance, security, or
  reliability no longer satisfies this policy.

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

The normal substantive-change handoff checks are:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
3. `cargo test --workspace --all-targets --all-features --locked`
4. `cargo deny --all-features --locked check`
5. Any affected protocol fixture, package, release, or documentation checks

Use `scripts/check.sh` on POSIX systems and `scripts/check.ps1` on Windows for
the first three checks through a disposable user environment.

## Documentation and release claims

- Preserve the README as a polished description of the destination. Track the
  gap between that promise and current delivery in `PROJECT.md`.
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
- Publish no assurance badge or trust claim until its owning ticket verifies the
  official proof, exact framework version, scope, date, public evidence, and
  rendered destination on exact `main`.
- Correct or remove assurance language immediately when its framework, issuer,
  scope, evidence, repository controls, organization boundary, or release
  pipeline changes invalidate the claim.

## Version control and handoff

- Inspect `git status --short` before editing and preserve unrelated changes.
- Use `rg` and `rg --files` for discovery before adding parallel behavior.
- Follow Conventional Commits: `<type>[optional scope]: <imperative summary>`.
- Do not commit, push, rewrite history, create tags, publish releases, or open
  pull requests unless the user requests it.
- At handoff, state the ticket outcome, files changed, checks run, and any
  remaining assumption, risk, or unverified external gate.
