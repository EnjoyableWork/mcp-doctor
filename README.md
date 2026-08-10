<h1 align="center">mcp-doctor</h1>

<p align="center">
  <strong>Diagnose, test, and break your MCP servers before your users do.</strong>
</p>

<p align="center">
  Find protocol, schema, and runtime problems in local or remote MCP servers,
  with clear reports you can trust.
</p>

<p align="center">
  <a href="https://github.com/EnjoyableWork/mcp-doctor/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/EnjoyableWork/mcp-doctor/actions/workflows/ci.yml/badge.svg"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <img alt="Built with Rust" src="https://img.shields.io/badge/built_with-Rust-dea584.svg?logo=rust&amp;logoColor=white">
  <img alt="MCP transports: STDIO and Streamable HTTP" src="https://img.shields.io/badge/MCP-STDIO_%2B_HTTP-6f42c1.svg">
</p>

<p align="center">
  <a href="#the-promise">The promise</a> ·
  <a href="#why-mcp-doctor">Why mcp-doctor?</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#inspect-check-break">Inspect. Check. Break.</a> ·
  <a href="#bring-it-into-ci">CI</a> ·
  <a href="#safe-by-default">Safety</a>
</p>

```console
$ mcp-doctor inspect -- node ./dist/weather-server.js

  mcp-doctor  weather-server · STDIO

  PASS  protocol       MCP 2026-07-28 supported
  PASS  discovery      8 tools · 2 resources · 1 prompt
  FAIL  tool/schema    weather_forecast.inputSchema.required
        expected an array of unique strings, found a string
  SKIP  tool/runtime   passive inspection; no tools called

  1 failed · 18 passed · 8 skipped                         exit 1
```

## The promise

`mcp-doctor` is the open-source Rust CLI from
[EnjoyableWork](https://github.com/EnjoyableWork) that checks your MCP server
before users depend on it. It finds the problems it can safely reach, explains
what went wrong, tells you what to fix, and gives people and AI agents the same
trustworthy report.

It does not stop after the first problem. It keeps checking anything that can
still run safely. If one failure blocks later checks, it shows the first issue
to fix and marks only those checks as skipped. Unrelated problems and serious
safety failures stay visible.

## Why mcp-doctor?

A successful connection only proves that a server answered once. It does not
prove that its messages follow MCP rules, its tools are usable, its tools
handle bad input, or its failures can be repeated.

`mcp-doctor` puts those checks into one repeatable report. It shows what ran,
what failed, what was skipped, and what to do next—without calling a tool
unless you ask it to.

| What it checks | What it finds |
| --- | --- |
| **Protocol** | Broken JSON-RPC messages, framing, version handling, methods, and feature claims |
| **Tools and features** | Bad tool, resource, or prompt definitions; duplicate names; and results that change between discovery runs |
| **Schemas** | Invalid or unsafe JSON Schema and input rules that clients cannot use |
| **Results** | Tool output that does not match `outputSchema` or claims success without the promised data |
| **Runtime** | Timeouts, crashes, early exits, bad output, oversized messages, and failed shutdown |
| **Repeatability** | Failures that change between runs, with the seed and input shape needed to run them again |

## Quick start

Install with Homebrew or Cargo:

```bash
# macOS or Linux
brew install --build-from-source EnjoyableWork/tap/mcp-doctor

# Any supported Rust host
cargo install mcp-doctor --version '=0.1.0' --locked
```

Or download a native GNU/Linux archive from
[GitHub Releases](https://github.com/EnjoyableWork/mcp-doctor/releases/latest).

Release archives, the exact Cargo package, the Homebrew formula, checksums,
SPDX SBOMs, and build attestations are published together. See the
[release guide](docs/release.md) to verify what you install.

Inspect a local STDIO server by placing its executable and arguments after
`--`:

```bash
mcp-doctor inspect -- node ./dist/server.js --stdio
```

Inspect a Streamable HTTP endpoint by URL:

```bash
mcp-doctor inspect https://mcp.example.com/mcp
```

The default report is made for people. Add `--format json` to get the same
findings as experimental, versioned JSON with secrets removed.

## Inspect. Check. Break.

Choose how much activity the target allows:

| Command | Activity | Use it to |
| --- | --- | --- |
| **`inspect`** | Does not call tools | Connect, list what the server offers, and check its definitions and schemas |
| **`check`** | Calls selected tools | Run known inputs from a scenario you wrote and check the results |
| **`break`** | Tries generated edge cases | Search one selected tool for failures you can repeat |

```bash
# Passive: discover and validate without calling a tool
mcp-doctor inspect -- node ./dist/server.js --stdio

# Active: replay one reviewed scenario
mcp-doctor check --scenario path/to/scenario -- node ./dist/server.js --stdio

# Active: run 50 deterministic edge cases against one selected tool
mcp-doctor break --tool search --cases 50 --seed 4242 -- node ./dist/server.js --stdio
```

> [!CAUTION]
> `check` and `break` execute real tool calls. Use disposable data and a test
> environment. Finding a tool does not give `mcp-doctor` permission to call it.

## Findings you can act on

Every finding includes a stable code, severity, MCP version, safe field
location, and whether the check ran or was skipped. Active failures also keep
the seed and input shape needed to repeat the case without revealing secrets
or raw production data.

When problems are connected, the report points to the first one you can fix.
It skips only the checks that depend on that problem and tells you why. It
keeps running unrelated checks and reports their problems too.

```text
PRIMARY DIAGNOSIS · schema

MCP-SCHEMA-004  error  tools[3].inputSchema.required

Why:
  `required` is a string, so clients cannot interpret the advertised input.

Expected:
  an array of unique property names

Fix:
  change `required` to an array, then run `mcp-doctor` again

Checks skipped because of this issue:
  tool/runtime
```

The human and JSON reports use the same findings. This prevents CI from hiding
a failure, choosing a different main issue, or turning a skipped check into a
pass.

The passive STDIO path is checked against pinned official TypeScript and Go
servers and independent Dart and PHP servers. See the
[compatibility evidence](tests/compatibility/README.md) for the exact scope and
results.

## Bring it into CI

Run the same check in a pull request. Its exit code can block the build:

```yaml
- name: Diagnose MCP server
  run: >-
    mcp-doctor inspect
    --format json
    --
    ./target/release/my-mcp-server --stdio
```

A required check that fails returns a non-zero status. Reports are repeatable
and hide secrets, so they work well in logs and saved build results.

## Safe by default

- `inspect` lists and checks what the server offers; it never calls a tool.
- Every active run names the scenario or tool, target, case limit, and seed.
- Hard limits cover time, data size, messages, schema work, test cases,
  redirects, retries, and parallel work.
- Normal output hides headers, credentials, tool inputs, raw results, and server
  logs.
- Local server commands run directly, not through a shell. Before exiting,
  `mcp-doctor` closes every child process, stops it if needed, and waits for it
  to end.

## Contributing

Contributions are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md), ask
for help through [SUPPORT.md](SUPPORT.md), and report suspected vulnerabilities
privately as described in [SECURITY.md](SECURITY.md).

## License

Licensed under the [MIT License](LICENSE).
