# Support

## Usage and development questions

Review [README.md](README.md), the focused documentation, and existing GitHub
issues before opening a new question. This source tree represents `0.3.3`; a
version is publicly available only when its canonical GitHub Release and
channel evidence exist. Only behavior proved by code, tests, and release
evidence should be treated as available. Public GitHub issues may discuss
proposals, but they are not support or availability claims.

For a reproducible bug, use the structured
[bug-report form](https://github.com/EnjoyableWork/mcp-doctor/issues/new?template=01-bug-report.yml).
Include the `mcp-doctor` version or commit, operating system and architecture,
transport, MCP revision, exact safe-to-share command shape, expected result,
and redacted actual outcome.

For a new diagnostic or workflow, use the
[feature-request form](https://github.com/EnjoyableWork/mcp-doctor/issues/new?template=02-feature-request.yml)
and describe the user problem, safety implications, and evidence that would
prove success.

## Keep reports safe

Use synthetic examples wherever possible. Do not post credentials, headers,
private URLs, environment values, complete configuration files, customer data,
raw tool results, or unreviewed server output.

Suspected vulnerabilities and accidentally exposed secrets belong in the
private process described by [SECURITY.md](SECURITY.md), never in a public issue.

## Scope

The project provides best-effort community support. Published support promises,
protocol revisions, platforms, and installation channels will be listed only
after their release evidence exists. General MCP server implementation support
outside a reproducible `mcp-doctor` behavior is out of scope for the issue
tracker.
