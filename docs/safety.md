# Safety model

`mcp-doctor` treats every target, message, schema, value, and result as
untrusted. Passive inspection is the default; active diagnostics require exact
redundant authority.

## Bounded diagnostic patience

`inspect`, `check`, and `break` accept one invocation-local
`--limit-profile`. The default remains suitable for an untrusted target. Use
`slow-start` only when a legitimate server or constrained CI runner needs more
time to start, discover capabilities, or return a bounded response:

```bash
mcp-doctor inspect \
  --limit-profile slow-start \
  -- node ./dist/server.js --stdio
```

| Selection | Startup | Discovery | Request | Response | Cleanup grace | Total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `default` | 10 s | 10 s | 30 s | 30 s | 2 s | 120 s |
| `slow-start` | 30 s | 30 s | 60 s | 60 s | 2 s | 240 s |

The scopes are exact: startup covers target preparation or process start;
discovery covers one discovery phase; request and response each cover one
corresponding write or HTTP exchange and wait; cleanup grace covers graceful
cleanup before forced termination; and total runs from STDIO startup or HTTP
target preparation through transport cleanup. None is a measured duration,
ETA, SLO, or guaranteed whole-process exit time. Input preparation, report
rendering and publication, and runtime shutdown remain outside the total. The
compiled capability manifest exposes the same milliseconds and scope strings
and sets `whole_process_exit_guarantee` to `false` for every profile.

These are the only accepted selections; `slow-start` is the compiled hard
maximum and there are no individual overrides, project configuration, or
disable-limit mode. Every byte, message, page, schema, case, generation,
report, redirect, retry, concurrency, and cleanup-capacity limit is identical
between the two selections.

A profile changes patience only. It never grants a process, private or
cleartext network destination, credential, tool, side effect, redirect, retry,
schema retrieval, or request beyond the selected command's existing authority.
For passive `inspect auto`, both protocol-era phases share the one selected
profile; choosing `slow-start` does not add a fallback, launch, prepared target,
or lifecycle request. Human and stable JSON reports identify the selection and
its effective numeric limits; JUnit records the same selection while
preserving the diagnostic result and exit semantics. `mcp-doctor capabilities`
advertises the two names and exactly which commands accept them. An invalid
name is rejected before target preparation.

## Status-channel safety

Live status is off by default. An explicit `--status plain` or `--status jsonl`
on `inspect`, `check`, `break`, or `reject` writes only to stderr and leaves the
stdout report, artifacts, diagnostic ordering, and target activity unchanged.
Each event is written and flushed once before the named work starts. Status
does not poll the target, relay MCP progress, reset a deadline, or add a launch,
message, tool call, retry, redirect, fallback, concurrent task, or side effect.

The status model accepts only fixed product vocabulary and bounded numbers. It
never retains or renders endpoints, executables, arguments, tool or case names,
paths, environment names or values, headers, credential sources or values,
server messages, MCP progress payloads, tool inputs or results, or target
stderr. Both representations are newline-delimited and contain no terminal
control sequences. JSONL stderr is independently schema-valid on every line;
plain status can be followed by the same safe CLI error prose emitted when
status is disabled.

Status is capped at 512 bytes per event, 128 events, and 65,536 aggregate
bytes. Each record uses one bounded write-all operation and one flush; no write
or flush error is retried. If the sink closes or a bound is exceeded, later
status is suppressed, target cleanup still runs, and the command cannot return
success; exit `4` identifies the output failure. Missing `completed` status
therefore proves neither success nor failure. Only the final stable report is
diagnostic evidence.

## Safety boundaries

- `inspect` checks advertised contracts without calling a tool.
- Passive `inspect` defaults to a finite `auto` negotiation. STDIO permits at
  most two non-overlapping launches of the exact command and reaps the first
  tree before the second; Streamable HTTP prepares one endpoint and pinned
  address authority. Both paths permit at most two lifecycle requests, one
  initialized notification, one legacy transition, zero retransmissions, zero
  application retries, concurrency one, and one shared total and aggregate
  budget. An explicit revision is a one-lifecycle hard pin.
- Active runs name and independently authorize each exact tool and target,
  declare effects and bounded cases, and add a seed for generation. Side
  effects require `--allow-side-effects`.
- Remote connections use direct public HTTPS, verified TLS, and pinned bounded
  resolution without redirects, retries, proxies, cookies, or caches.
- Private targets, loopback cleartext, and environment credentials each require
  an exact endpoint gate. Credentials never use HTTP or trigger OAuth or
  metadata discovery.
- Hard limits cover time, bytes, messages, schema work, cases, retries,
  redirects, and concurrency.
- Request-scoped SSE is decoded incrementally. Accepted bytes are scanned in
  order without reparsing prior chunks; only bounded current-line,
  current-event, and JSON payload state is retained under the per-message and
  aggregate-output limits.
- `schema_evaluation_steps` is one deterministic operation budget spanning the
  preliminary schema/instance walk, Draft 2020-12 meta-validation, validator
  construction, local-reference fan-out, and actual instance access. String,
  pattern, equality, collection, combinator, and uniqueness work must fit that
  budget before the affected tool call. Preliminary structural or work excess
  is `MCP-LIMIT-001`. Exhaustion during meta-validation or validator
  construction after those gates is the typed `MCP-SCHEMA-005` tool-limitation
  evidence and makes the performed check incomplete; it never becomes a pass,
  warning, limit increase, retry, or authority to call the tool.
- Pattern evaluation uses the validator's linear-time regular-expression
  engine with fixed 100,000-byte compiled-size and DFA-cache limits. The same
  ECMA-262 translation is inspected as maintained HIR before construction;
  counted repetitions, character-class ranges, reachable pattern fan-out, and
  all potential instance text form a conservative product charged before any
  match starts. Draft 2020-12 patterns requiring backtracking features such as
  look-around or backreferences fail locally with the typed
  `unsupported_linear_pattern` rule; they are never passed to a backtracking
  engine.
- Legacy HTTP session IDs come only from initialization, stay bounded and
  run-local, repeat exactly, and receive one bounded teardown. Session loss
  never reinitializes or changes the selected revision; teardown failure stays
  visible.
- Reports hide headers, credentials, tool inputs, raw results, and server logs.
- Opt-in live status uses only fixed structural vocabulary and numeric ceilings;
  it never forwards target progress or replaces the final report.
- Sensitive snapshots require an exact-path acknowledgement and new file;
  value-free offline diffs have no target or network surface.
- Aggregates accept only explicit bounded stable reports, discard unknown
  optional values, preserve failures, and perform no target, network,
  credential, retrieval, or tool activity.
- Capability discovery reports only fixed compiled facts under 64 KiB and
  reads no configuration, host inventory, credentials, files, process,
  network, target, retrieval, or tool data.
- Local commands bypass the shell. Before exit, `mcp-doctor` closes, stops when
  needed, and waits for every child process.

See the [diagnostic command guide](commands.md) for each active authorization
contract and [SECURITY.md](../SECURITY.md) to report a suspected vulnerability
privately.
