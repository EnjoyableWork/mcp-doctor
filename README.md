<h1 align="center">🩺 mcp-doctor</h1>

<p align="center">
  <strong>Diagnose, test, and break your MCP servers before your users do.</strong>
</p>

<p align="center">
  Find protocol, schema, and runtime problems in local or remote MCP servers,
  with clear reports you can trust.
</p>

<p align="center">
  <a href="https://github.com/EnjoyableWork/mcp-doctor/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/EnjoyableWork/mcp-doctor/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://docs.rs/crate/mcp-doctor/latest"><img alt="crates.io version" src="https://img.shields.io/crates/v/mcp-doctor.svg?logo=rust&amp;logoColor=white"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <a href="https://www.bestpractices.dev/en/projects/14096/baseline-1"><img alt="OpenSSF OSPS Baseline v2026.02.19 Level 1" src="https://www.bestpractices.dev/projects/14096/baseline"></a>
  <img alt="MCP transports: STDIO and Streamable HTTP" src="https://img.shields.io/badge/MCP-STDIO_%2B_HTTP-6f42c1.svg">
</p>

<p align="center">
  <a href="#install">Install</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#use-with-coding-agents">Coding agents</a> ·
  <a href="#choose-a-diagnostic">Commands</a> ·
  <a href="#mcp-revision-support">MCP support</a> ·
  <a href="#documentation">Documentation</a> ·
  <a href="#assurance">Assurance</a>
</p>

A diagnosis you can act on:

> **Your weather server starts correctly**
>
> `mcp-doctor` found 8 tools, 2 resources, and 1 prompt.
>
> **First thing to fix**
>
> The `weather_forecast` tool describes its required inputs incorrectly. Define
> the required fields as a list, then run the diagnosis again.
>
> **Safe by default**
>
> No tools were called and no server data was changed.

## Install

| Channel | Platforms | Install |
| --- | --- | --- |
| Homebrew | macOS, GNU/Linux | `brew install EnjoyableWork/tap/mcp-doctor` |
| Cargo | macOS, GNU/Linux, Windows | `cargo install mcp-doctor` |
| GitHub Releases | GNU/Linux (ARM64, x64) | [Download the latest archive](https://github.com/EnjoyableWork/mcp-doctor/releases/latest) |

Every immutable release includes SHA-256 checksums, SPDX SBOMs, and build
provenance. See the [release guide](docs/release.md) for exact-version installs
and artifact verification.

## Quick start

Passive `inspect` defaults to bounded `--protocol-version auto`. For a local
STDIO target, `auto` may consume the discovery bound, fully reap the first
process, and start the exact command one more time for legacy initialization.
Use an explicit supported revision when one lifecycle and, for STDIO, one
process launch is required.

Inspect a local STDIO server without calling any tools. Put the command you
already use to start the server after `--`:

```bash
mcp-doctor inspect -- node ./dist/server.js --stdio
```

For Streamable HTTP, pass the endpoint URL:

```bash
mcp-doctor inspect https://mcp.example.com/mcp
```

`inspect` selects a mutually supported compiled revision, validates the MCP
conversation, checks advertised features and schemas, reports the earliest
actionable failure, and cleans up. It never calls a tool.

## Use with coding agents

Install `mcp-doctor` first, then optionally install its portable Agent Skill so
a coding agent starts with the same passive, exact-target workflow. The agent
uses your terminal to run the CLI; `mcp-doctor` is not configured as an MCP
server. [Install and verify the skill](docs/agents.md).

## Why mcp-doctor?

A successful connection proves only that a server answered once. It does not
prove that messages follow MCP rules, schemas are usable, results match their
contracts, failures are repeatable, or shutdown is safe.

`mcp-doctor` keeps checking anything that can still run safely. When one
failure blocks dependent work, it names the first issue to fix and marks only
those checks as skipped. Independent problems and safety failures stay visible
in the same human- and agent-readable result.

| What it checks | What it finds |
| --- | --- |
| **Protocol** | Broken JSON-RPC, framing, revision handling, methods, and feature claims |
| **Agent selection** | Missing or blank tool descriptions that make a tool hard to choose reliably |
| **Schemas** | Invalid or unsafe JSON Schema and unusable input rules |
| **Results** | Output that violates `outputSchema` or a promised success shape |
| **Runtime** | Timeouts, crashes, oversized output, early exits, and failed cleanup |
| **Repeatability** | Deterministic evidence needed to run a failure again |

## Choose a diagnostic

Start with the least-active command that answers your question:

| Command | Activity | Use it to |
| --- | --- | --- |
| **`inspect`** | Passive | Validate discovery, definitions, and schemas |
| **`check`** | Reviewed calls | Replay known inputs and validate results |
| **`break`** | Generated calls | Search one tool for repeatable edge-case failures |
| **`reject`** | Invalid calls | Verify one tool rejects malformed arguments |
| **`diff`** | Local files | Compare two explicitly captured contracts |
| **`aggregate`** | Local files | Combine stable diagnostic reports conservatively |
| **`capabilities`** | Compiled facts | Select or defer work without touching a target |

> [!CAUTION]
> `check`, `break`, and `reject` execute real tool calls. Use disposable data
> and a test environment. Finding a tool never grants permission to call it.

See the [diagnostic command guide](docs/commands.md) for reviewed scenarios,
multi-tool workflows, generated cases, rejection checks, and contract diffs.

## MCP revision support

**Legend:** ✅ = supported; ❌ = not supported. Passive `inspect` defaults to
bounded `auto`; an explicit revision remains a hard pin. Active commands keep
MCP `2026-07-28` as their sole implicit revision and require an exact option
for supported legacy activity. Each status includes an invisible
`mcp-doctor-support=supported|unsupported` source token for agents reading the
Markdown.

| MCP revision | `inspect` | Snapshot | Same-revision `diff` | `check` | `break` | `reject` |
| --- | --- | --- | --- | --- | --- | --- |
| `2026-07-28` | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> |
| `2025-11-25` | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ❌ <!-- mcp-doctor-support=unsupported --> |
| `2025-06-18` | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ✅ <!-- mcp-doctor-support=supported --> | ❌ <!-- mcp-doctor-support=unsupported --> |
| `2025-03-26` | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> |
| `2024-11-05` | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> |
| `2024-10-07` | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> |
| Unknown | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> | ❌ <!-- mcp-doctor-support=unsupported --> |

Read the [full revision contract](docs/protocol-support.md) for bounded passive
selection, exact pins, schema-dialect rules, dated usage context, and
compatibility evidence. For automation,
`mcp-doctor capabilities --format json` remains authoritative.

## Documentation

| I want to… | Read |
| --- | --- |
| Diagnose through a coding agent | [Coding-agent guide](docs/agents.md) |
| Run scenarios, generated cases, rejection checks, or diffs | [Diagnostic commands](docs/commands.md) |
| Select an MCP revision or understand compatibility evidence | [MCP revision support](docs/protocol-support.md) |
| Produce JSON/JUnit, interpret exits, aggregate reports, or configure CI | [Automation and CI](docs/automation.md) |
| Understand execution, network, cleanup, redaction, and hard limits | [Safety model](docs/safety.md) |
| Verify or publish release artifacts | [Release guide](docs/release.md) |
| Report a suspected vulnerability | [Security policy](SECURITY.md) |

## Assurance

As of 2026-08-15, `mcp-doctor` has an
[official-hosted, scoped self-assessment](https://www.bestpractices.dev/en/projects/14096/baseline-1)
for all 24 OpenSSF OSPS Baseline `v2026.02.19` Level 1 controls. The
[dated crosswalk](docs/assurance/osps-v2026.02.19-level-1.md) records the exact
scope and limitations. This is a project self-assessment, not an independent
certification or regulatory compliance claim.

Every named asset in the immutable `v0.3.0` GitHub Release also passed a scoped
[SLSA `v1.2` Build L2 evaluation](docs/assurance/slsa-v1.2-build-l2.md) against
its exact digest and signed provenance. That result does not cover registry or
Homebrew operations, dependencies, unlisted assets, or future releases.

## Safe by default

- `inspect` never calls a tool; active commands require exact redundant
  authorization and a declared effect boundary.
- Processes, network activity, schema work, messages, reports, and cleanup are
  bounded. Local commands bypass the shell and managed children are reaped.
- Remote connections use verified direct HTTPS by default. Private,
  cleartext-loopback, and credentialed targets require exact separate gates.
- Human, JSON, and JUnit output share one redacted result without raw arguments,
  results, credentials, headers, stderr, or server logs.

The [safety model](docs/safety.md) contains the complete operational boundary.

## Contributing

Contributions are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md), ask
for help through [SUPPORT.md](SUPPORT.md), follow the
[Code of Conduct](CODE_OF_CONDUCT.md), and report suspected vulnerabilities
privately through [SECURITY.md](SECURITY.md). The
[project scope](docs/project-scope.md) identifies every `mcp-doctor` repository
and official distribution or community channel.

## License

Licensed under the [MIT License](LICENSE).
