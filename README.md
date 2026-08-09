<h1 align="center">mcp-doctor</h1>

<p align="center">
  <strong>Diagnose, test, and break your MCP servers before your users do.</strong>
</p>

<p align="center">
  Protocol conformance, schema validation, runtime pressure, and reproducible
  reports for local and remote MCP servers.
</p>

<p align="center">
  <a href="https://github.com/EnjoyableWork/mcp-doctor/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/EnjoyableWork/mcp-doctor/actions/workflows/ci.yml/badge.svg"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <img alt="Built with Rust" src="https://img.shields.io/badge/built_with-Rust-dea584.svg?logo=rust&amp;logoColor=white">
  <img alt="MCP transports: STDIO and Streamable HTTP" src="https://img.shields.io/badge/MCP-STDIO_%2B_HTTP-6f42c1.svg">
</p>

<p align="center">
  <a href="#why-mcp-doctor">Why mcp-doctor?</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#inspect-check-break">Inspect. Check. Break.</a> ·
  <a href="#bring-it-into-ci">CI</a> ·
  <a href="#safe-by-default">Safety</a>
</p>

```console
$ mcp-doctor inspect -- node ./dist/weather-server.js

  mcp-doctor  weather-server · STDIO

  PASS  protocol       MCP 2026-07-28 negotiated
  PASS  discovery      8 tools · 2 resources · 1 prompt
  FAIL  tool/schema    weather_forecast.inputSchema.required
        expected an array of unique strings, found a string
  SKIP  tool/runtime   passive inspection; no tools called

  1 failed · 18 passed · 8 skipped                         exit 1
```

## Why mcp-doctor?

A green handshake only proves that a server answered once. It does not prove
that its messages are valid, its advertised contracts are usable, its tools
survive bad inputs, or its failures can be reproduced.

`mcp-doctor` turns those invisible contracts into one deterministic diagnostic
result. It shows exactly what ran, what failed, and what was skipped—without
silently calling a tool.

| It examines | It surfaces |
| --- | --- |
| **Protocol** | Invalid JSON-RPC envelopes, framing, negotiation, methods, and capability use |
| **Catalogs** | Malformed tools, resources, prompts, duplicate names, and unstable discovery results |
| **Schemas** | Invalid or pathological JSON Schema, unsafe references, and unusable input contracts |
| **Results** | Tool output that disagrees with `outputSchema` or reports success without the promised structure |
| **Runtime** | Timeouts, crashes, early exits, malformed output, oversized messages, and failed shutdown |
| **Reproducibility** | Non-deterministic failures, with the seed and structural input needed to run them again |

## Quick start

Install with Homebrew or Cargo:

```bash
# macOS or Linux
brew install EnjoyableWork/tap/mcp-doctor

# Any supported Rust host
cargo install mcp-doctor --locked
```

Or download a native GNU/Linux archive from
[GitHub Releases](https://github.com/EnjoyableWork/mcp-doctor/releases/latest).

Inspect a local STDIO server by placing its executable and arguments after
`--`:

```bash
mcp-doctor inspect -- node ./dist/server.js --stdio
```

Inspect a Streamable HTTP endpoint by URL:

```bash
mcp-doctor inspect https://mcp.example.com/mcp
```

The report is human-readable by default. Add `--format json` for the same
findings as a stable, redacted machine result.

## Inspect. Check. Break.

Choose how much activity the target allows:

| Command | Activity | Use it to |
| --- | --- | --- |
| **`inspect`** | Passive | Negotiate the protocol, discover capabilities, and validate advertised catalogs and schemas |
| **`check`** | Explicit scenario | Run known inputs against only the tools named by a user-authored scenario and validate their results |
| **`break`** | Bounded adversarial | Generate reproducible boundary cases for one explicitly selected tool |

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
> environment. `mcp-doctor` never treats a reachable tool as authorization to
> invoke it.

## Findings you can act on

Every finding includes a stable code, severity, protocol revision, safe
location, and performed-or-skipped state. Active failures also retain the seed
and structural input required to reproduce the case without printing secrets
or raw production values.

```text
MCP-SCHEMA-004  error  tools[3].inputSchema.required

`required` must be an array of unique property names.

Reproduce:
  mcp-doctor break --tool weather_forecast --seed 4242 --case 17 -- <server>
```

Human and machine output come from the same result model, so a CI report cannot
quietly omit a failure or turn a skipped check into a pass.

## Bring it into CI

Run the same diagnostic journey in a pull request and let its exit status gate
the build:

```yaml
- name: Diagnose MCP server
  run: >-
    mcp-doctor inspect
    --format json
    --
    ./target/release/my-mcp-server --stdio
```

A required check that fails returns a non-zero status. Reports stay
deterministic and redacted, making them suitable for logs and retained build
artifacts.

## Safe by default

- `inspect` performs discovery and structural validation only; it never calls
  a tool.
- Active runs name their scenario or tool, target, case budget, and seed
  explicitly.
- Time, bytes, messages, schema work, cases, redirects, retries, and concurrency
  are bounded.
- Headers, credentials, tool arguments, raw results, and server logs are
  redacted from ordinary output.
- Local commands are passed as literal arguments, never shell source, and every
  child process is closed, terminated, and reaped before exit.

## Contributing

Contributions are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md), ask
for help through [SUPPORT.md](SUPPORT.md), and report suspected vulnerabilities
privately as described in [SECURITY.md](SECURITY.md).

## License

Licensed under the [MIT License](LICENSE).
