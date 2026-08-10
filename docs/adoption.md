# M2 adoption checkpoint

- Opened: 2026-08-10
- Release under review: `mcp-doctor` `v0.1.0`
- Initial baseline: zero independent adoption reports at opening
- Public collection record: [M2 adoption checkpoint issue 5](https://github.com/EnjoyableWork/mcp-doctor/issues/5)

This checkpoint asks whether the passive preflight earns repeat use before the
project adds active tool calls, remote targets, adversarial generation, or a
stable CI report. Publication proves that the tool can be installed and that
its artifacts are traceable; it does not prove adoption.

## What useful evidence looks like

Reports should cover an independently authored MCP server or implementation.
Only consented, aggregate, non-sensitive observations are recorded:

- language or SDK family and whether the server is official, independent, or
  private;
- whether installation succeeded and which release channel was used;
- whether the first actionable diagnosis was correct, unclear, or false;
- whether the report alone explained what failed, where, why, what was
  expected, and what to change next;
- approximate time to the first useful answer;
- whether the user fixed a problem and ran `mcp-doctor` again; and
- which single missing capability, if any, prevented continued use.

Do not post endpoint URLs, credentials, authentication headers, environment
values, server source, raw MCP payloads, tool arguments or results, private
names, complete stderr, or unreviewed report output. A public response should
be a safe summary. Suspected vulnerabilities belong in the private process in
[`SECURITY.md`](../SECURITY.md).

## Decision rule

The review aims for at least five independently authored servers or
implementations, with representation beyond the repository's own fixtures. The
number is a collection target, not proof by itself. The evidence must show
whether people found the report correct and actionable and whether they chose
to use it again.

M3 does not start automatically. Each proposed active, remote, adversarial, or
CI capability must solve a repeated observed problem. Weak or contradictory
evidence means narrow, defer, or cancel that capability rather than expanding
the MVP by default.

The public GitHub checkpoint issue is the collection record. Its link and the
dated baseline are added to `PROJECT.md` after the immutable release and issue
exist.
