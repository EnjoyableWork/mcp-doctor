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
recognized-unsupported revisions; passive selection default, modes, compiled
modern set, exact pins, transport paths and maxima; report, scenario,
generator, snapshot, diff, aggregate, Markdown artifact, and capability contract
versions; stdout and artifact reporter availability; `mcp-doctor.exit/v1`;
named hard limit profiles; and compile-time process-tree and file-identity
capabilities. Capability discovery reports only fixed compiled facts.

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
| Markdown artifact | `--markdown-report <PATH>` | Pull-request summaries and reviewable build artifacts |

Human, JSON, and JUnit are stdout selections. Markdown is artifact-only: it is
not a `--format` value and never changes the selected stdout projection or
process exit.

JSON follows the stable, schema-backed `mcp-doctor.report/v1` contract. JUnit
projects the same checks into conservative XML. Markdown follows the
deterministic `mcp-doctor.markdown/v1` projection and begins with
`<!-- mcp-doctor.markdown/v1 -->`; all projections keep secrets removed.
Passive `inspect` adds a typed `protocol_selection` object:
requested mode, fixed path, selected supported revision when established, and
bounded process-launch, lifecycle-request, notification, and fallback counts.
The human header, JUnit properties, and Markdown protocol-selection section
carry the same value-free evidence.
When bounded local schema meta-validation or validator construction cannot
finish after preliminary structural gates pass, the shared result retains a
performed `schema.contracts` check with outcome `incomplete` and
`MCP-SCHEMA-005`. JSON includes only the phase, fixed limit name and unit,
observed count, maximum count, and structural location. JUnit projects that
performed incomplete check as skipped/incomplete rather than a failure or pass;
Markdown identifies the same incomplete check and safe evidence. A genuine
failure in the same report still takes precedence while the incomplete evidence
remains visible.

The Markdown artifact includes the product and report contract versions,
selected and negotiated protocol revisions when present, outcome and stable
exit meaning, complete summary counts, primary diagnosis, independent safety
findings, causal skips, fixed corrective actions, checks, and the effective
limit profile and values. It uses stable headings and ordering, LF line endings,
and one final newline. Apart from its required version comment, it contains no
raw HTML, timestamps, local paths, target identifiers, untrusted values,
terminal escapes, remote images, or external assets.

Every exit code follows `mcp-doctor.exit/v1`. A command may emit only a subset:

| Exit | Stable meaning | In practice |
| ---: | --- | --- |
| `0` | `success` | Passed or completed |
| `1` | `unsuccessful_result` | Failed diagnostic, actionable diff, or failed aggregate |
| `2` | `invalid_invocation_or_input` | Rejected options, capability request, or input |
| `3` | `incomplete_evidence` | No pass/fail conclusion |
| `4` | `internal_or_output_failure` | Internal, render, write, publish, or cleanup failure |

The command-specific sections define the exact outcome behind each code.

Keep the stdout report while also writing all three artifact projections from
the same diagnostic run with explicit new-file destinations:

```bash
mcp-doctor inspect \
  --json-report artifacts/mcp-doctor.json \
  --junit-report artifacts/mcp-doctor.xml \
  --markdown-report artifacts/mcp-doctor.md \
  -- node ./dist/server.js --stdio
```

`--json-report`, `--junit-report`, and `--markdown-report` also apply to
`check`, `break`, and `reject`. Each parent directory must already exist, each
destination must be distinct and not already exist, and `-` is not a file
destination. `mcp-doctor` validates those conditions before target or network
activity, runs the diagnostic once, and renders every requested report from one
immutable redacted result in fixed JSON, JUnit, Markdown order under one shared
aggregate output bound.

A failed or incomplete diagnostic still publishes every requested file when
reporting succeeds and retains exit `1` or `3`; a render, write, publication,
rollback, or cleanup failure cannot report success, removes every
identity-owned partial artifact, and exits `4`.

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
skips. When present, passive protocol-selection evidence must also have a
coherent mode, path, selected revision, and bounded counts; it is preserved in
the member report. Typed performed-incomplete schema evidence is validated as
strictly as its check outcome, summary, primary diagnosis, and exit. Failure
outranks incomplete, which outranks pass: only all-pass input passes.
There is no waiver, score, baseline, deduplication, majority rule, or severity
override.

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

## GitHub Actions starter

The repository publishes a copyable
[least-permission preflight workflow](../.github/workflows/mcp-doctor-preflight.yml).
It demonstrates all four parts of the CI contract in one job:

1. install exact released `mcp-doctor 0.3.3` with Cargo's locked source
   contract;
2. run noninteractive passive `inspect` against one explicit STDIO target and
   exact MCP `2026-07-28` revision;
3. write versioned JSON and JUnit reports to fixed repository-local paths and
   upload both under one deterministic artifact name for seven days; and
4. leave the diagnostic process exit authoritative while the `always()` report
   verification and upload steps still run.

The checked-in target is unmistakably synthetic and repository-owned. To use
the workflow in an MCP server repository, copy it and replace only the
synthetic build step, the executable and literal arguments after `--`, and the
`pull_request.paths` entries with that repository's server-owned paths. Keep
the exact `mcp-doctor` version, immutable action commits, explicit
`contents: read` permission, fixed report destinations, report verification,
and unconditional upload behavior under review when updating the copy.

The exact `0.3.3` binary used by this released starter predates the Markdown
artifact contract, so this workflow deliberately requests only JSON and JUnit.
A copied workflow may add `--markdown-report` and upload that third file only
after its pinned binary advertises `markdown` under the diagnostic command's
`artifact_reporters` capability and `mcp-doctor.markdown/v1` under
`schema_versions.markdown_report`.

The example grants no tool-call, side-effect, credential, private-network,
cleartext, target-discovery, production-target, or external-schema authority.
Do not add any of those gates merely to make a CI target start. Select a
synthetic or repository-owned server that can run within the documented
constrained STDIO environment; diagnose production separately and explicitly.

For pull requests, exit `0` keeps the job successful. Exit `1`, `2`, `3`, or
`4` fails the diagnostic step and therefore the job even when both reports are
uploaded successfully afterward. `actions/upload-artifact` is only the CI
carrier: `mcp-doctor` remains responsible for producing the two immutable,
redacted projections from one run and never receives provider credentials.

This repository runs the passing fixture only when the workflow or its owned
fixture contract changes. Maintainers can manually dispatch either `passing`
or `diagnosed` to reproduce the acceptance evidence without contacting a real
server. The diagnosed dispatch is intentionally red: its non-success result
must remain the job conclusion while the named report artifact remains
downloadable.
