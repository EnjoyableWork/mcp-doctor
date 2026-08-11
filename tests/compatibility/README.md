# Current-revision compatibility evidence

This directory records the separately controlled real-server evidence for
`MCPD-007`. It answers a narrow question: can the built `mcp-doctor` passive
STDIO journey diagnose selected, pinned MCP `2026-07-28` servers without
calling a tool?

It is not the default test suite, an interoperability certification, or a claim
of official MCP conformance. Synthetic fixtures remain the authoritative way
to test failures, unsupported revisions, limits, redaction, and process
cleanup. The real-server matrix checks reach across implementations whose code
we do not control.

## Selected cases

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
this reviewed matrix. It does not mean every MCP server works, older revisions
work, HTTP works, tools were executed, or an official conformance suite passed.

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
7. removes the upstream checkouts, dependency caches, and reports.

Dependency and image preparation is the only networked phase. Runtime images
are multi-architecture Docker Official Images pinned by immutable OCI index
digest. The TypeScript case uses the upstream frozen pnpm lock and pnpm
`10.26.1`; Go uses the upstream `go.sum` with `-mod=readonly`; Dart uses
`dart pub get --enforce-lockfile` and an offline compiled executable; and PHP
installs the reviewed Composer lock with plugins, scripts, and development
packages disabled.

## Update rules

Changing an upstream release, commit, example, runtime image, package-manager
version, or dependency lock is a review event. Update
[`matrix.json`](matrix.json), regenerate the affected reviewed lock if needed,
run the controlled matrix, and record the new date and exact outcomes in the
same pull request. A failed official or independent current-revision case
removes the broad position until it is understood; it must not be hidden by
deselecting the case or enabling legacy fallback.
