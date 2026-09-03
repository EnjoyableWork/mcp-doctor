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
capability contract versions; status commands, representations, stream,
schema, and output limits; stdout and artifact reporter availability;
`mcp-doctor.exit/v1`; named hard limit profiles and their exact scoped time
ceilings; and compile-time process-tree and file-identity capabilities.
Capability discovery reports only fixed compiled facts.

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

## Live status for wrappers

Use `--status jsonl` when a wrapper, editor, coding agent, or CI job needs to
observe a target-facing command while it is running:

```bash
mcp-doctor inspect --status jsonl --format json \
  -- node ./dist/server.js --stdio \
  >mcp-doctor-report.json 2>mcp-doctor-status.jsonl
```

Keep the streams separate. Stdout remains the selected final report;
stderr is exclusively LF-terminated `mcp-doctor.status/v1` records once the
CLI has parsed `jsonl`. The status stream is off by default. `plain` is intended
for noninteractive human logs and may coexist with ordinary safe CLI error
prose. CLI syntax failures that occur before status selection is accepted are
not part of the JSONL contract.

Parse each JSONL line independently and act only on the fixed `event`, `phase`,
ceiling, ordinal, and exit fields. The record schema permits unknown optional
fields within `v1`; ignore them. Never infer success from process liveness, a
phase event, EOF, or an absent terminal event. Only `completed` reports the
known process exit after target cleanup and requested report publication, and
the stable report remains the diagnostic evidence.

Before starting a target, read `.status` and
`.diagnostic_time_ceiling_profiles` from the compiled capability manifest.
The latter exposes exact milliseconds and a `scope` for startup, discovery,
one request, one response, cleanup grace, and the transport total for both
`default` and `slow-start`. The total begins at STDIO startup or HTTP target
preparation and continues through transport cleanup. Every entry deliberately
sets `whole_process_exit_guarantee` to `false`: input preparation, report
publication, and runtime shutdown are not covered by `total`. An outer
watchdog should select the intended profile first, honor those phase scopes,
or use the command's advertised default when it has no selector, and add its
own bounded allowance for the wrapper and uncovered local work.
Killing a run at the watchdog remains an incomplete wrapper observation, not a
diagnostic conclusion.

Status emission is itself finite: 512 bytes per record, 128 records, 65,536
aggregate bytes, one bounded write-all operation and one flush per record, and
no retry after a write or flush error. A failed status sink still allows target
cleanup to finish and yields exit `4`; wrappers should continue draining stdout
and wait for the process rather than treating a closed stderr pipe as
cancellation. See the
[status schema](../schemas/mcp-doctor.status.v1.schema.json) and
[command event contract](commands.md#live-diagnostic-status).

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
| `3` | `incomplete_evidence` | No pass/fail conclusion, including a clean caught Unix STDIO interruption |
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

A caught Unix STDIO `SIGINT` or `SIGTERM` is intentionally different from a
completed incomplete diagnostic: no stdout report or requested artifact is
published. Successful whole-tree cleanup and artifact rollback return exit `3`;
any cleanup or output failure returns exit `4`. With `--status jsonl`, the
terminal exit-3 record carries `completion_reason: "interrupted"`. Wrappers
should use that field instead of inferring interruption from EOF or an
operating-system signal status. The compiled `interruption` capability exposes
the fixed 2,000 ms graceful phase, 2,000 ms forced-reap phase, and 4,000 ms
combined ceiling.

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

The repository publishes a tested, least-permission GitHub Actions starter as
one small bundle:

- the [`MCP Doctor` diagnostic workflow](../.github/workflows/mcp-doctor.yml);
- the [sticky-comment publisher](../.github/workflows/mcp-doctor-comment.yml);
- the [compiled-capability verifier](../scripts/verify-mcp-doctor-preflight-capabilities.sh);
  and
- the [report-boundary verifier](../scripts/verify-mcp-doctor-preflight-reports.sh).

Copy all four files to the same paths in the server repository. The diagnostic
workflow demonstrates six parts of the CI contract in one job:

1. download the exact immutable `mcp-doctor 0.4.1` GNU/Linux x64 archive,
   enforce its reviewed byte sizes, SHA-256 digest, and four-entry layout, and
   smoke-test the extracted binary without installing Rust or compiling its
   dependency graph;
2. run noninteractive passive `inspect` against one explicit STDIO target and
   exact MCP `2026-07-28` revision;
3. write versioned JSON, JUnit, Markdown, and badge reports to fixed
   repository-local paths;
4. verify the reports before appending the Markdown diagnosis to the GitHub job
   summary or uploading all four under one deterministic artifact name for
   seven days;
5. derive and upload one non-archive result descriptor capped at 2 KiB for the
   comment boundary; and
6. leave the diagnostic process exit authoritative while the report verifier
   and verified-only publication steps still run.

The two workflow files are separate by design. The diagnostic executes the
proposed server with only a read token. The comment publisher is loaded from
the trusted default branch only after that run and is the only workflow given
comment-write permission. GitHub recommends this `workflow_run` pattern for
[privilege separation](https://docs.github.com/en/actions/reference/security/secure-use#mitigating-the-risks-of-untrusted-code-checkout),
and warns privileged workflows not to check out or execute proposed code. A
single `pull_request` workflow would put the write token beside the proposed
build and server; a `pull_request_target` workflow that checks out and executes
the proposed head has the same unsafe boundary. Keep both files for this
YAML-only integration. A separately reviewed GitHub App could own the comment
outside consumer CI, but that is a different architecture rather than a safe
one-file equivalent.

The checked-in target is unmistakably synthetic and repository-owned. The
workflow marks the only two consumer-specific blocks with `ADAPT` comments. To
use the bundle in an MCP server repository:

1. replace `ADAPT 1/2` with that repository's deterministic runtime setup,
   locked dependency installation, and server build steps; and
2. replace only the literal executable and arguments after `--` at `ADAPT 2/2`
   with one repository-owned STDIO server target.

The remaining download, integrity, capability, report, summary, artifact, and
comment-descriptor steps are provider integration, not build-system examples.
The starter deliberately runs on every pull request so a copied required check
does not silently remain pending because a project-specific path filter was
missed.

Commit all four files together. On the installation pull request, the
read-only diagnostic can run from the proposed workflow, but no sticky comment
is expected: GitHub loads the `workflow_run` publisher only after that file
exists on the default branch. Merge the installation, then the next pull
request receives both **Enjoyable Work / MCP Doctor** and the sticky comment.

Keep the exact `mcp-doctor` version, GNU/Linux target, archive and binary byte
sizes, SHA-256 digest, exact archive layout, immutable action commits, explicit
`contents: read` permission, fixed report destinations, capability and report
verification, and verified-only summary and upload conditions under review
when updating the copy. The archive was built and smoke-tested on the same
`ubuntu-24.04` runner generation used by the starter; this copy does not claim
compatibility with older GNU C Library hosts. Keep the workflow display name as
`Enjoyable Work` and the job display name as `MCP Doctor`; GitHub renders the
check as the deliberate company/product pair **Enjoyable Work / MCP Doctor**.
Commands, paths, artifacts, and configuration remain lowercase. The job
summary and PR comment both use the fixed call to action **Add MCP Doctor to
another project**.

The exact `0.4.1` binary advertises `json`, `junit`, `markdown`, and `badge`
under the diagnostic command's `artifact_reporters` capability. The safe
boundary check requires the corresponding `mcp-doctor.report/v1`, JUnit,
`mcp-doctor.markdown/v1`, and `mcp-doctor.badge/v1` shapes before upload and
scans every report for the same fixed redaction sentinels and local-path
classes. A separate capability check verifies those compiled contracts before
the target process starts. A missing or different contract stops the job
without running the target or claiming that the unsupported artifacts exist.
If report verification fails, neither the Markdown summary nor any artifact is
published. The descriptor contains only a version, the closed
`passed`/`failed`/`incomplete` outcome, and bounded integer counts. It contains
no report prose, identifier, location, argument, result, stderr, or other
untrusted display value.

The example grants no tool-call, side-effect, credential, private-network,
cleartext, target-discovery, production-target, or external-schema authority.
Do not add any of those gates merely to make a CI target start. Select a
synthetic or repository-owned server that can run within the documented
constrained STDIO environment; diagnose production separately and explicitly.

For pull requests, exit `0` keeps the job successful. A completed failed or
incomplete diagnostic exits `1` or `3`; it fails the diagnostic step and job
while all four reports are still verified, summarized, and uploaded. Invalid
input or an internal/output failure exits `2` or `4`; it also fails the job,
and the verifier blocks both carriers when no valid complete report set exists.
`actions/upload-artifact` and `$GITHUB_STEP_SUMMARY` are only CI carriers:
`mcp-doctor` remains responsible for producing the four immutable, redacted
projections from one run and never receives provider credentials.

The comment publisher is a separate `workflow_run` workflow that GitHub loads
only from the repository's default branch. It starts after a completed
`Enjoyable Work` workflow's `MCP Doctor` pull-request job and never checks out
or executes pull-request code. Its one job receives only `actions: read` and
`contents: read` plus `pull-requests: write`; the diagnostic remains read-only.
Before using the write token, the publisher:

1. verifies the exact repository, default branch, workflow ID, workflow path,
   run, attempt, source event, and one associated same-repository pull request
   through GitHub's API;
2. requires the producer workflow and both repository-owned report verifiers
   at the tested head to have the same Git blob identities as the trusted
   default-branch files; an unavailable or mismatched contract publishes only a
   fixed **Summary withheld** state and fails the publisher;
3. requires the pull request to remain open at the tested head and ignores a
   stale or superseded run;
4. accepts at most one non-expired `mcp-doctor-comment.json` artifact from that
   exact run, caps it at 4 KiB, binds its run, repository, head repository, and
   head SHA, downloads it without archive extraction, and validates every key,
   enum, integer bound, count relation, and outcome/job conclusion mapping;
5. renders only fixed local Markdown plus those bounded counts; and
6. creates or updates one `github-actions[bot]` comment identified by an exact
   hidden marker. It fails closed on duplicate owned comments or a pull request
   with 100 or more existing comments rather than guessing across an unbounded
   comment history.

The visible comment is intentionally brief:

```markdown
## 🩺 MCP Doctor

✅ **Passed** · 5 passed · 0 failed · 0 warnings · 1 skipped

**Mode: Passive** · Inspects the server without calling tools.

View full check · Explore MCP Doctor modes · Add MCP Doctor to another project

Commit abc1234 · attempt 1 · CI presentation, not certification
```

The links point to the exact Actions run, the CI-mode guide, and this starter.
A later run updates the same comment instead of adding another one. If the
latest run has no valid descriptor, the publisher replaces an earlier result
with a fixed **No
structurally validated summary** state, links to the run, and fails its own
publication job. A producer contract that cannot be verified against the
trusted default branch instead receives the fixed **Summary withheld** state.
The publisher never copies `report.md` or another artifact-selected string into
the comment.

The provider-native job conclusion remains the merge-enforcement authority.
Comment creation or update is a separate presentation result and cannot turn a
failed or incomplete diagnosis green. Repositories that do not allow timeline
comments may omit the publisher and retain the same diagnostic check, job
summary, and artifacts.

### MCP Doctor CI modes

The PR comment identifies the current CI coverage as **Passive** and links to
this section. **Passive**, **Standard**, and **Full** are human-facing CI
coverage labels, not values accepted by an `mcp-doctor --mode` option:

| Mode | Commands composed in repository YAML | Coverage and exposure |
| --- | --- | --- |
| **Passive** | `inspect` | Starts the selected server and validates lifecycle, advertised capabilities, catalogs, descriptions, and schemas. It makes no tool calls. This is the safe starter and recommended check on every pull request. |
| **Standard** | Passive plus one or more reviewed `check` scenarios | Replays repository-owned known inputs against exact selected tools and validates their results. Start with deterministic `read_only` tools or disposable test data. Each scenario declares its effects, and every tool still requires its exact matching `--allow-tool`. |
| **Full** | Standard plus targeted `break` and `reject` runs | Exercises the broadest reviewed active suite: known workflows, deterministic schema-valid edge cases, and schema-invalid rejection behavior for exact selected tools. Use only an isolated test server and disposable data. Side-effecting work requires the separate `--allow-side-effects` gate. |

Selecting a deeper level means adding the corresponding explicit steps,
repository-owned scenarios, exact tools, effects, cases, and seeds to that
repository's YAML. There is intentionally no generic one-line switch that
discovers or authorizes tools. **Full** never means “call every tool”: it means
the repository has deliberately composed all applicable command families for
the finite test surface it reviewed. Keep active jobs separate from the
passive baseline so their additional authority, trigger, target, and evidence
remain visible. See [Diagnostic commands](commands.md) for the exact `check`,
`break`, and `reject` contracts before adding them.

This YAML-only publisher is intentionally a presentation surface, not an
independent attestation, certification, security result, conformance claim, or
merge authority. The read-only diagnostic still builds and executes code from
the proposed branch. The same-repository comment path therefore assumes branch
writers are inside the repository's CI trust boundary; exact producer-contract
matching prevents a pull request from directly replacing the workflow or
verifiers, but it is not an operating-system sandbox for the proposed build or
server. A service that needs independently trusted results for arbitrary
contributors requires separately hosted isolated execution and an authenticated
result, such as a reviewed GitHub App architecture.

The fixed `badge.json` is only the provider-neutral public badge input; this
starter neither publishes nor hosts it. Publishing requires a separately
reviewed consumer-owned HTTPS path that limits updates to trusted
default-branch runs and defines scoped evidence, freshness, failure,
revocation, and cleanup behavior. Private and air-gapped projects can retain
only native status and the four review artifacts without selecting any public
endpoint. The checked-in starter targets GitHub.com; its current artifact
Actions are not a GitHub Enterprise Server compatibility claim.

### Public and private repositories

The install paths, command, passive safety boundary, check result, same-repository
sticky comment, job summary, and four report formats are identical in public
and private GitHub.com repositories. GitHub changes who can see the carriers
and how runner usage is accounted for:

| Surface | Public repository | Private repository |
| --- | --- | --- |
| Pull-request check, same-repository sticky comment, and job summary | Visible with the public pull request and Actions run | Visible only to people who can view the repository and pull request |
| Uploaded report artifact | Publicly retrievable while retained | Requires repository read access; API clients also need appropriate Actions access |
| GitHub workflow status badge | Can be embedded on public pages | [Cannot be accessed externally](https://docs.github.com/en/actions/how-tos/monitor-workflows/add-a-status-badge) |
| Standard GitHub-hosted runner | [Free and unlimited](https://docs.github.com/en/actions/reference/runners/github-hosted-runners) | Consumes the account's included minutes and storage, then follows its [Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions) settings |
| Product-led discovery | The branded comment, check, summary link, artifact, and any justified public badge can lead other maintainers back to `mcp-doctor` | The same surfaces support internal adoption, but do not create public discovery |

The diagnostic workflow requests only `contents: read` and no secrets. A
private server whose build needs private packages or other credentials
therefore needs a separately reviewed build credential path; do not pass those
values into the diagnosed server environment. Fork pull-request runs keep that
restricted posture: they never receive the publisher's write token. Such runs
may be disabled or require approval
according to the repository or organization [Actions policy](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository).
The checked-in publisher intentionally posts only for an exact
same-repository head and does not comment on fork pull requests. Forks retain
the read-only check, job summary, and report artifacts after any required
approval. Repository automation receives a comment only when GitHub represents
its head as the same repository and every other publisher check passes; no
separate Dependabot compatibility claim is made here. An organization or
enterprise can disable Actions, restrict the three commit-pinned Action uses,
or prevent the requested write permission. These are GitHub policy and access
differences, not differences in the `mcp-doctor` diagnostic.

A public server repository can use GitHub's native workflow badge after the
copied workflow runs the real server on a trusted default-branch event. That
badge reports the workflow result, not a broader assurance claim. After adding
a reviewed default-branch `push` trigger and obtaining a representative run,
replace `OWNER`, `REPOSITORY`, and `DEFAULT_BRANCH` in this optional README
link:

```markdown
[![MCP Doctor](https://github.com/OWNER/REPOSITORY/actions/workflows/mcp-doctor.yml/badge.svg?branch=DEFAULT_BRANCH&event=push)](https://github.com/OWNER/REPOSITORY/actions/workflows/mcp-doctor.yml)
```

The checked-in starter intentionally does not add that trigger or badge: it
diagnoses this repository's synthetic server fixture, so presenting its
result as server health would be misleading. Private repositories should rely
on the check and job summary for authorized readers; GitHub does not serve
their workflow badges to external sites.

A manual dispatch repeats the same passing synthetic target. The comment
publisher does not run for that dispatch because it is not associated with a
pull request. Built-binary and policy tests separately prove that failed and
incomplete diagnostics retain their non-success exits while verified reports
remain eligible for the summary and report carrier.
