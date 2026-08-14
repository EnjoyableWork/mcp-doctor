# Compatibility evidence

This directory records the separately controlled real-server evidence for
`MCPD-007` and `MCPD-027`, and the deliberately narrower claim boundary for
`MCPD-028`. It answers two narrow questions: can the built
`mcp-doctor` passive STDIO journey diagnose selected, pinned MCP `2026-07-28`
servers without calling a tool, and can its explicitly selected MCP
`2025-11-25` active adapter safely run `check` and `break` against one pinned
official and one pinned independent server?

It is not the default test suite, an interoperability certification, or a claim
of official MCP conformance. Synthetic fixtures remain the authoritative way
to test failures, unsupported revisions, limits, redaction, and process
cleanup. The real-server matrix checks reach across implementations whose code
we do not control.

## Selected cases

### Passive MCP 2026-07-28

| Case | Source | Language | Exact release and commit | 2026-08-10 result |
| --- | --- | --- | --- | --- |
| `official-typescript-todos` | Official [TypeScript SDK](https://github.com/modelcontextprotocol/typescript-sdk) | TypeScript | `@modelcontextprotocol/fastify@2.0.0` at `cc4b41617ce3601b1290d67216ea0b194a3cd9ac` | Pass |
| `official-go-hello` | Official [Go SDK](https://github.com/modelcontextprotocol/go-sdk) | Go | `v1.7.0` at `bc72835f62eb94d0fb484439f886b6885b075f36` | Pass |
| `independent-dart-strict-current` | Independent [mcp_dart](https://github.com/leehack/mcp_dart) | Dart | `v2.4.0` at `b104df00e17340026c32a8742835ccbe905303ed` | Pass |
| `independent-php-simple` | Independent [MCP SDK for PHP](https://github.com/logiscape/mcp-sdk-php) | PHP | `v2.0.0` at `b3a8882b81a891014ef6374522dd983284496ca1` | Pass |

The two independent cases are credible release evidence rather than temporary
test repositories. At the dated review, both were public, active, unarchived,
MIT-licensed projects with current-revision server code and a recent tagged
release. `mcp_dart` had 114 stars and 31 forks and had been pushed on
2026-08-08; the PHP SDK had 368 stars and 52 forks and had been pushed on
2026-07-31. Those popularity counts are context, not a quality guarantee. A
future rerun must re-review ownership, activity, license, revision support, and
the selected example before updating the evidence date.

All four reports selected MCP `2026-07-28`, performed and passed the five
required checks, kept `runtime.tools` skipped as `not_authorized`, returned no
primary or independent failure, and exited 0. The built-binary unsupported-
revision and earliest-layer cases remain in
[`tests/stdio.rs`](../stdio.rs); the real-server matrix does not silently fall
back to an older initialization flow.

Under `DEC-024`, four passing selected cases across four languages—including
two independent implementations—support the scoped phrase “broad
current-revision compatibility.” That means the passive STDIO checks work for
this reviewed matrix. It does not mean every MCP server works, HTTP works,
tools were executed, or an official conformance suite passed.

Explicit passive MCP `2025-11-25` and `2025-06-18` adapters, plus active MCP
`2025-06-18`, remain covered by synthetic built-binary STDIO and Streamable
HTTP journeys in [`tests/stdio.rs`](../stdio.rs), [`tests/active.rs`](../active.rs),
[`tests/break.rs`](../break.rs), and [`tests/http.rs`](../http.rs). Legacy
selection never serves as a fallback for any case in this matrix.

### Active MCP 2025-11-25

| Case | Provenance | Language | Exact tool | Commands | 2026-08-14 result |
| --- | --- | --- | --- | --- | --- |
| `official-go-hello` | Official Go SDK | Go | `greet` | one-case `check`; three-case `break` at seed `6027` | 2/2 pass |
| `independent-php-simple` | Independent MCP SDK for PHP | PHP | `add-numbers` | one-case `check`; three-case `break` at seed `6027` | 2/2 pass |

These cases reuse the exact releases, commits, runtime images, and dependency
locks recorded above. Both tools are deterministic, read-only computations in
disposable containers. The checked-in scenarios and their SHA-256 digests are
recorded in [`matrix.json`](matrix.json), and every active invocation still
names the same exact tool through `--allow-tool`. Runtime containers have no
network, a read-only root filesystem, no Linux capabilities, no new privileges,
an ephemeral `/tmp`, and no caller configuration or Docker socket.

All four active reports selected and negotiated MCP `2025-11-25`, performed
and passed every required check, returned no primary or independent failure,
and exited `0`. The runner also verifies the exact runtime-case count, requires
the `break` generation check only for `break`, rejects selected-tool disclosure,
and fails if a labeled container remains. No invocation discovers, retries,
downgrades, falls back, starts a task, answers a server request, or uses a
server annotation as authority.

This is narrow active STDIO reach across two implementations and two languages,
not broad legacy compatibility. Synthetic fixtures remain the evidence for
Streamable HTTP, malformed responses, task-required tools, elicitation and
server-request handling, limits, redaction, and cleanup. No legacy HTTP,
installed-channel, every-server, or official-conformance claim follows from
these four successful runs.

### Active MCP 2025-06-18

Active MCP `2025-06-18` requires an exact Draft 2020-12 declaration on every
advertised schema that authorizes or validates activity. Missing, malformed,
unsupported, external, ambiguous, unsupported-vocabulary, or over-limit schemas
stop before generation or `tools/call`; passive omitted-dialect reporting stays
unchanged. Synthetic built-binary journeys cover that stricter gate over STDIO
and Streamable HTTP, and represented source-install smokes cover successful
`check` and `break` execution.

The controlled real-server matrix does not add a `2025-06-18` case in this
slice. Its retained `2025-11-25` active pairs and four current-revision passive
cases remain regression gates, not evidence of `2025-06-18` server reach. No
real-server, broad legacy, ecosystem-wide, or published-channel compatibility
claim follows from the new exact-selected source capability.

## Reproduce it

From the repository root, with Cargo, Git, Docker, and `jq` available:

```console
./scripts/compatibility.sh
```

The same command is available through the manually dispatched
`Compatibility evidence` GitHub Actions workflow. It stays outside ordinary
pull-request and push checks because it downloads and builds third-party
projects.

The runner:

1. builds the local `mcp-doctor` with the locked Rust graph;
2. clones each upstream tag and rejects it unless `HEAD` equals the reviewed
   commit;
3. verifies the upstream Go and pnpm lock hashes and installs the reviewed Dart
   and Composer locks from this directory;
4. prepares dependencies inside digest-pinned Linux containers that receive
   only a disposable work directory—never the caller's configuration or Docker
   socket;
5. launches each server with no network, a read-only root filesystem, no Linux
   capabilities, `no-new-privileges`, and an ephemeral `/tmp`;
6. passes the server command to the built CLI and accepts only a stable
   `mcp-doctor.report/v1` pass whose runtime-tool check is explicitly
   `not_authorized`; and
7. runs the two exact legacy `check`/`break` pairs with their pinned scenarios,
   tool authority, case count, and seed, accepting only exact selected and
   negotiated MCP `2025-11-25` reports whose required checks all passed;
8. verifies that every labeled container exited; and
9. removes the upstream checkouts, dependency caches, and reports.

Dependency and image preparation is the only networked phase. Runtime images
are multi-architecture Docker Official Images pinned by immutable OCI index
digest. The TypeScript case uses the upstream frozen pnpm lock and pnpm
`10.26.1`; Go uses the upstream `go.sum` with `-mod=readonly`; Dart uses
`dart pub get --enforce-lockfile` and an offline compiled executable; and PHP
installs the reviewed Composer lock with plugins, scripts, and development
packages disabled.

## Update rules

Changing an upstream release, commit, example, runtime image, package-manager
version, dependency lock, active tool, scenario, scenario digest, effect
classification, case count, or seed is a review event. Update
[`matrix.json`](matrix.json), regenerate the affected reviewed lock if needed,
run the controlled matrix, and record the new date and exact outcomes in the
same pull request. A failed official or independent current-revision case
removes the broad position until it is understood; it must not be hidden by
deselecting the case or enabling legacy fallback. A failed active legacy case
removes only the narrow active claim until it is understood; it cannot be
replaced with synthetic evidence or an unreviewed tool.
