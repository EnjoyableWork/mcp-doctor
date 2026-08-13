# Security policy

`mcp-doctor` starts local processes, parses untrusted protocol messages and
schemas, and can connect to explicitly authorized remote MCP endpoints. Please
report suspected security problems privately.

## Security contact and private reporting

Contact the `mcp-doctor` repository maintainers through GitHub's
[private vulnerability reporting form](https://github.com/EnjoyableWork/mcp-doctor/security/advisories/new).
This is the project's recognized security contact and confidential reporting
route. Do not open a public issue, pull request, discussion, or support request
for a suspected vulnerability.

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

## Response targets

The maintainers aim to:

- acknowledge a private report within 3 business days;
- provide an initial assessment or status within 7 calendar days; and
- provide an update at least every 14 calendar days until the report is
  resolved or otherwise closed.

These are response targets, not a service-level agreement or a guaranteed
remediation deadline. Active exploitation, credential exposure, or immediate
user harm receives priority. Complexity, reporter availability, release
safety, and upstream coordination can affect timing.

## Coordinated disclosure

Keep the report and reproducer private while the maintainers assess scope,
develop a correction or mitigation, and coordinate disclosure timing with the
reporter. The project generally targets public disclosure within 90 days of
acknowledgement, preferably after a fix or verified mitigation is available.
The date may be shortened for active exploitation or extended by mutual
agreement when a safe correction or upstream coordination needs more time.

When appropriate, the maintainers will publish a GitHub Security Advisory that
identifies affected and fixed versions, mitigations, and reporter credit, and
will request a CVE through GitHub when warranted. The project does not operate
a bug-bounty program and cannot promise payment or other compensation.

## Supported versions

Only the latest published minor release line receives security fixes. Reports
about older versions are welcome, but the resolution may be to upgrade rather
than receive a backport.

| Version | Security support |
| --- | --- |
| `0.3.x` | Supported |
| `0.2.x` and earlier | Unsupported |
| `main` | Development only; no release or backport guarantee |

This table changes when a new supported release line is published. A tag,
branch, package, or archive not listed as supported is unsupported.

## Safe research boundary

Test only systems you own or are explicitly authorized to assess. Prefer
synthetic data, disposable local servers, and the smallest reproducer. Do not
access other users' data, degrade availability, persist access, perform social
engineering, or test production endpoints without their owner's explicit
permission. Stop if testing exposes credentials, private data, or an
unexpected third-party system; contain the impact and report privately.

Following this guidance does not grant access or waive applicable law or the
terms of any third-party service.

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
