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
generator, snapshot, diff, aggregate, Markdown artifact, badge artifact, and
capability contract versions; stdout and artifact reporter availability;
`mcp-doctor.exit/v1`;
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
| Badge artifact | `--badge-report <PATH>` | Shields endpoint badges backed by one diagnostic outcome |

Human, JSON, and JUnit are stdout selections. Markdown and badge reports are
artifact-only: neither is a `--format` value, and neither changes the selected
stdout projection or process exit.

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

### Badge artifacts

The badge artifact is deterministic `mcp-doctor.badge/v1` JSON for the
[Shields endpoint format](https://shields.io/badges/endpoint-badge). Its entire
surface is fixed and derived only from the typed overall outcome:

| `schemaVersion` | `label` | Diagnostic outcome | `message` | `color` |
| ---: | --- | --- | --- | --- |
| `1` | `mcp-doctor` | `passed` | `pass` | `brightgreen` |
| `1` | `mcp-doctor` | `failed` | `fail` | `red` |
| `1` | `mcp-doctor` | `incomplete` | `incomplete` | `lightgrey` |

The object has exactly those four fields. It contains no score, grade, count,
product or protocol version, target, path, identifier, timestamp, URL, dynamic
label, or untrusted text. It is a compact projection of one run, not a
certification, verification, conformance result, or promise about another run.

Every exit code follows `mcp-doctor.exit/v1`. A command may emit only a subset:

| Exit | Stable meaning | In practice |
| ---: | --- | --- |
| `0` | `success` | Passed or completed |
| `1` | `unsuccessful_result` | Failed diagnostic, actionable diff, or failed aggregate |
| `2` | `invalid_invocation_or_input` | Rejected options, capability request, or input |
| `3` | `incomplete_evidence` | No pass/fail conclusion |
| `4` | `internal_or_output_failure` | Internal, render, write, publish, or cleanup failure |

The command-specific sections define the exact outcome behind each code.

Keep the stdout report while also writing all four artifact projections from
the same diagnostic run with explicit new-file destinations:

```bash
mcp-doctor inspect \
  --json-report artifacts/mcp-doctor.json \
  --junit-report artifacts/mcp-doctor.xml \
  --markdown-report artifacts/mcp-doctor.md \
  --badge-report artifacts/mcp-doctor-badge.json \
  -- node ./dist/server.js --stdio
```

`--json-report`, `--junit-report`, `--markdown-report`, and `--badge-report`
also apply to `check`, `break`, and `reject`. Each parent directory must already
exist, each destination must be distinct and not already exist, and `-` is not
a file destination. `mcp-doctor` validates those conditions before target or
network activity, runs the diagnostic once, and renders every requested report
from one immutable redacted result in fixed JSON, JUnit, Markdown, badge order
under one shared aggregate output bound.

A failed or incomplete diagnostic still publishes every requested file when
reporting succeeds and retains exit `1` or `3`; a render, write, publication,
rollback, or cleanup failure cannot report success, removes every
identity-owned partial artifact, and exits `4`.

To render the file through a badge service, publish the fixed JSON with your
ordinary artifact hosting and point Shields at that public file. For example,
after separately hosting it at
`https://artifacts.example/mcp-doctor-badge.json`:

```markdown
![mcp-doctor](https://img.shields.io/endpoint?url=https%3A%2F%2Fartifacts.example%2Fmcp-doctor-badge.json)
```

`mcp-doctor` writes only the local artifact. It does not publish files, operate
a hosted endpoint, contact Shields, or grant the badge any authority beyond the
single diagnostic run that produced it.

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

1. install exact released `mcp-doctor 0.4.0` with Cargo's locked source
   contract;
2. run noninteractive passive `inspect` against one explicit STDIO target and
   exact MCP `2026-07-28` revision;
3. write versioned JSON, JUnit, Markdown, and badge reports to fixed
   repository-local paths and upload all four under one deterministic artifact
   name for seven days; and
4. leave the diagnostic process exit authoritative while the `always()` report
   verification and upload steps still run.

The checked-in target is unmistakably synthetic and repository-owned. To use
the workflow in an MCP server repository, copy it and replace only the
synthetic build step, the executable and literal arguments after `--`, and the
`pull_request.paths` entries with that repository's server-owned paths. Keep
the exact `mcp-doctor` version, immutable action commits, explicit
`contents: read` permission, fixed report destinations, report verification,
and unconditional upload behavior under review when updating the copy.

The exact `0.4.0` binary advertises `json`, `junit`, `markdown`, and `badge`
under the diagnostic command's `artifact_reporters` capability. The safe
boundary check requires the corresponding `mcp-doctor.report/v1`, JUnit,
`mcp-doctor.markdown/v1`, and `mcp-doctor.badge/v1` shapes before upload and
scans every report for the same fixed redaction sentinels and local-path
classes. A separate capability check verifies those compiled contracts before
the target process starts. A missing or different contract stops the job
without running the target or claiming that the unsupported artifacts exist.

The example grants no tool-call, side-effect, credential, private-network,
cleartext, target-discovery, production-target, or external-schema authority.
Do not add any of those gates merely to make a CI target start. Select a
synthetic or repository-owned server that can run within the documented
constrained STDIO environment; diagnose production separately and explicitly.

For pull requests, exit `0` keeps the job successful. Exit `1`, `2`, `3`, or
`4` fails the diagnostic step and therefore the job even when all four reports
are uploaded successfully afterward. `actions/upload-artifact` is only the CI
carrier: `mcp-doctor` remains responsible for producing the four immutable,
redacted projections from one run and never receives provider credentials.

The provider-native job conclusion remains the merge-enforcement authority.
The fixed `badge.json` is only the provider-neutral public badge input; this
starter neither publishes nor hosts it. Publishing requires a separately
reviewed consumer-owned HTTPS path that limits updates to trusted
default-branch runs and defines scoped evidence, freshness, failure,
revocation, and cleanup behavior. Private and air-gapped projects can retain
only native status and the four review artifacts without selecting any public
endpoint.

This repository runs the passing fixture only when the workflow or its owned
fixture contract changes. Maintainers can manually dispatch either `passing`
`diagnosed`, or `incomplete` to reproduce the acceptance evidence without
contacting a real server. The diagnosed and incomplete dispatches are
intentionally red: each non-success result must remain the job conclusion while
the named report artifact remains downloadable.
