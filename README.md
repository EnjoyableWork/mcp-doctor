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
  <a href="#why-mcp-doctor">Why mcp-doctor?</a> ·
  <a href="#choose-a-diagnostic">Commands</a> ·
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

### Install the optional Agent Skill

The Agent Skill teaches a compatible coding agent how to use the CLI safely; it
does not include the CLI. Install `mcp-doctor` above, then choose one skill
route:

| Route | Best for | Start here |
| --- | --- | --- |
| Smithery | Discovery and an agent-specific install command | [Open `mcp-doctor` on Smithery](https://smithery.ai/skills/enjoyable/mcp-doctor) |
| GitHub | Reviewing the source or making a version-controlled manual install | [Open the canonical skill directory](https://github.com/EnjoyableWork/mcp-doctor/tree/main/.agents/skills/mcp-doctor) |

Smithery is an installation convenience backed by the GitHub skill directory;
GitHub remains the canonical source and release authority. Follow the
[coding-agent guide](docs/agents.md) for exact-version, update, and removal
instructions.

## Quick start

`inspect` is passive: it validates discovery, definitions, and schemas without
calling a tool.

### Diagnose a local STDIO server

Put the command you already use to start the server after `--`:

```bash
mcp-doctor inspect -- node ./dist/server.js --stdio
```

### Diagnose a remote Streamable HTTP server

```bash
mcp-doctor inspect https://mcp.example.com/mcp
```

### Select a protocol revision

Omit `--protocol-version` for bounded automatic selection, or choose one of the
supported passive values explicitly:

```bash
# Auto-select a mutually supported revision (the default)
mcp-doctor inspect \
  --protocol-version auto \
  -- node ./dist/server.js --stdio

# Pin the current revision
mcp-doctor inspect \
  --protocol-version 2026-07-28 \
  -- node ./dist/server.js --stdio

# Pin a supported legacy revision
mcp-doctor inspect \
  --protocol-version 2025-11-25 \
  -- node ./dist/server.js --stdio
```

The passive values are `auto`, `2026-07-28`, `2025-11-25`, and `2025-06-18`.
See [MCP revision support](docs/protocol-support.md) for selection behavior,
active-command support, and the complete matrix.

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
| **Agent guidance** | Missing, blank, placeholder, or name-only tool descriptions, plus required inputs without usable descriptions |
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

## Documentation

| I want to… | Read |
| --- | --- |
| Use `mcp-doctor` with a coding agent | [Coding-agent guide](docs/agents.md) |
| Evaluate the Agent Skill on a registry or host | [Agent Skill evaluation contract](docs/evaluations/agent-skill-v1.md) |
| Choose a diagnostic or run active scenarios safely | [Diagnostic commands](docs/commands.md) |
| Select an MCP revision or verify the support matrix | [MCP revision support](docs/protocol-support.md) |
| Configure CI, produce reports, or interpret exits | [Automation and CI](docs/automation.md) |
| Generate a portable Shields-compatible badge artifact | [Badge artifact contract](docs/automation.md#badge-artifacts) |
| Understand execution, network, cleanup, redaction, and hard limits | [Safety model](docs/safety.md) |
| Verify or publish release artifacts | [Release guide](docs/release.md) |
| Review the dated standalone category evaluation | [`v0.4.0` evaluation evidence](docs/evaluations/v0.4.0.md) |
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
- Human, JSON, JUnit, Markdown, and badge output share one redacted result
  without raw arguments, results, credentials, headers, stderr, or server logs.

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
