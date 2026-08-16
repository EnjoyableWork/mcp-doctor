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

These are the only accepted selections; `slow-start` is the compiled hard
maximum and there are no individual overrides, project configuration, or
disable-limit mode. Every byte, message, page, schema, case, generation,
report, redirect, retry, concurrency, and cleanup-capacity limit is identical
between the two selections.

A profile changes patience only. It never permits a process, private or
cleartext network destination, credential, tool, side effect, redirect, retry,
fallback, schema retrieval, or extra request. Human and stable JSON reports
identify the selection and its effective numeric limits; JUnit records the
same selection while preserving the diagnostic result and exit semantics.
`mcp-doctor capabilities` advertises the two names and exactly which commands
accept them. An invalid name is rejected before target preparation.

## Safety boundaries

- `inspect` checks advertised contracts without calling a tool.
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
- Legacy HTTP session IDs come only from initialization, stay bounded and
  run-local, repeat exactly, and receive one bounded teardown. Session loss
  never reinitializes or downgrades; teardown failure stays visible.
- Reports hide headers, credentials, tool inputs, raw results, and server logs.
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
