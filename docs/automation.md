# Automation and CI

`mcp-doctor` exposes compiled capability discovery, stable redacted reports,
offline aggregation, and deterministic exit semantics for agents, wrappers,
editors, and CI systems.

## Compiled capability discovery

Check the installed binary before choosing a target or diagnostic:

```bash
mcp-doctor capabilities --format json
```

This command emits deterministic `mcp-doctor.capabilities/v1` JSON containing
the product version; exact command, transport, and MCP revision matrix;
recognized-unsupported revisions; report, scenario, generator, snapshot, diff,
aggregate, and capability contract versions; reporter availability;
`mcp-doctor.exit/v1`; named hard limit profiles; and compile-time process-tree
and file-identity capabilities. Capability discovery reports only fixed compiled facts.

It does not inspect user configuration or host inventory, read credentials,
start a process, resolve DNS, connect to a target, retrieve a schema, or call a
tool. An integration can classify a planned run as supported, unsupported, or unknown
before it selects a target.

Consumers request the representation exactly with
`--schema-version mcp-doctor.capabilities/v1`. There is no discovery, fallback,
or version downgrade. An unsupported request exits `2`; JSON output receives a
value-free typed error listing the supported capability schema. Within stable
`v1`, consumers ignore unknown optional fields and treat unknown command,
transport, revision, reporter, contract, or profile values as unknown. Adding
a new supported entry or optional field is compatible; removing support or
changing a required field or existing meaning requires a new capability major.
The checked-in
[Draft 2020-12 schema](../schemas/mcp-doctor.capabilities.v1.schema.json) is the
validation contract.

## Reports and exit codes

Target-facing diagnostics render the same immutable, redacted result:

| Output | Select with | Best for |
| --- | --- | --- |
| Human | Default | Reading in a terminal |
| JSON | `--format json` | Agents and automation |
| JUnit | `--format junit` | CI test interfaces |

JSON follows the stable, schema-backed `mcp-doctor.report/v1` contract. JUnit
projects the same checks into conservative XML; both machine formats keep
secrets removed.

Every exit code follows `mcp-doctor.exit/v1`. A command may emit only a subset:

| Exit | Stable meaning | In practice |
| ---: | --- | --- |
| `0` | `success` | Passed or completed |
| `1` | `unsuccessful_result` | Failed diagnostic, actionable diff, or failed aggregate |
| `2` | `invalid_invocation_or_input` | Rejected options, capability request, or input |
| `3` | `incomplete_evidence` | No pass/fail conclusion |
| `4` | `internal_or_output_failure` | Internal, render, write, publish, or cleanup failure |

The command-specific sections define the exact outcome behind each code.

Keep the stdout report while also writing both machine projections from the
same diagnostic run with explicit new-file destinations:

```bash
mcp-doctor inspect \
  --json-report artifacts/mcp-doctor.json \
  --junit-report artifacts/mcp-doctor.xml \
  -- node ./dist/server.js --stdio
```

`--json-report` and `--junit-report` also apply to `check`, `break`, and
`reject`. Each parent directory must already exist, each destination must be
distinct and not already exist, and `-` is not a file destination.
`mcp-doctor` validates those conditions before target or network activity,
runs the diagnostic once, and renders every requested report from one
immutable redacted result.

A failed or incomplete diagnostic still publishes both files when reporting
succeeds and retains exit `1` or `3`; a render, write, publication, or cleanup
failure cannot report success and exits `4`.

## Offline diagnostic aggregates

`aggregate` combines one through 32 explicit ordered, stable, redacted
`mcp-doctor.report/v1` regular files into one new
`mcp-doctor.aggregate/v1` artifact. Human stdout is the default;
`--format json` makes stdout byte-identical to the artifact.

```bash
mcp-doctor aggregate \
  --output matrix.aggregate.json \
  reports/linux.json reports/macos.json reports/windows.json
```

Members are identified only by zero-based input ordinal. Known safe fields are
retained; compatible unknown optional properties are discarded, not echoed.
Every input must satisfy the embedded report schema and its summary, severity,
outcome, exit, revision, primary diagnosis, independent findings, and causal
skips. Failure outranks incomplete, which outranks pass: only all-pass input
passes. There is no waiver, score, baseline, deduplication, majority rule, or
severity override.

All-pass exits `0`, any failure exits `1`, and otherwise incomplete exits `3`.
Invalid, unreadable, aliased, over-limit input or an unsafe destination exits
`2`; render, write, publication, or cleanup failure exits `4` and leaves no
artifact.

`aggregate` never starts a process, opens a connection, resolves DNS or
credentials, retrieves a schema, discovers a target, or calls a tool. It does
not scan directories, expand globs, infer CI artifacts, or follow report paths.
Fixed limits cover each file, total input, nesting, nodes, validation work,
checks, findings, output, and operation time. Duplicate, aliased, hard-linked,
symlinked, malformed, inconsistent, or over-limit input rejects the invocation.
The destination must be a new file in an existing directory and is staged and
published without overwrite.

The stable aggregate contract is published with the source as
[`mcp-doctor.aggregate/v1`](../schemas/mcp-doctor.aggregate.v1.schema.json). Its
member reports use the separate
[`mcp-doctor.report/v1`](../schemas/mcp-doctor.report.v1.schema.json) contract;
offline validators should load both local schemas and must not retrieve either
at runtime.

## Bring it into CI

Run the same check in a pull request. Its exit remains the gate while one run
produces stable JSON and JUnit:

```yaml
- name: Diagnose MCP server
  run: >-
    mcp-doctor inspect
    --json-report artifacts/mcp-doctor.json
    --junit-report artifacts/mcp-doctor.xml
    --
    ./target/release/my-mcp-server --stdio
```

A failed required check returns non-zero. Your CI system uploads or consumes
the explicit paths; `mcp-doctor` performs no provider-specific upload. Each
JUnit diagnostic check becomes one test case, while JSON and JUnit preserve
stdout's outcome, primary diagnosis, causal skips, and safe evidence. Reports
remain deterministic, bounded, and secret-free.
