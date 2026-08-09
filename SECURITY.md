# Security policy

`mcp-doctor` starts local processes, parses untrusted protocol messages and
schemas, and will eventually connect to remote MCP endpoints. Please report
security problems privately.

## Reporting a vulnerability

Use the repository's
[private vulnerability reporting](https://github.com/EnjoyableWork/mcp-doctor/security/advisories/new)
form. Do not open a public issue for a suspected vulnerability.

Include only the minimum evidence needed to reproduce the problem:

- the affected version or commit;
- operating system, architecture, transport, and MCP revision;
- a synthetic reproducer or redacted structural description;
- the observed impact and whether a process, network target, or tool call was
  involved; and
- any temporary mitigation you have verified.

Do not submit credentials, authentication headers, private endpoints, real
tool results, customer data, complete configuration files, raw server logs, or
an exploit against a system you do not own or have permission to test. If a
secret was exposed, revoke or rotate it before reporting.

Maintainers will acknowledge the report, assess scope, coordinate a correction,
and agree on disclosure timing with the reporter. Timing depends on severity
and the safety of releasing details; avoid public disclosure until a fix or
explicitly agreed mitigation is available.

## Supported versions

There is no supported public release yet. Until the first release, fixes land
on the default branch without a compatibility or backport guarantee. This
section will list supported release lines when packages are published.

## Security boundary

Security-sensitive invariants include:

- no implicit active tool execution;
- literal process arguments without a shell;
- bounded process, message, schema, network, and generation work;
- complete child-process cleanup and reap;
- redaction of credentials and untrusted values from reports and errors;
- no default external schema retrieval; and
- explicit remote-target, redirect, proxy, and authentication policy.

A defect in any of these boundaries should be treated as a potential security
issue even when it appears to be only a diagnostic-quality problem.
