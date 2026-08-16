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

## Maintainer advisory lifecycle

Maintainers apply the public policy above through this non-disclosing lifecycle:

1. Keep each suspected vulnerability, reproducer, discussion, and correction
   private while its impact is unknown.
2. Reproduce it with synthetic evidence and determine the exact affected
   released versions. A provisional severity or version range is not a public
   claim.
3. If the evidence does not reproduce or does not cross a security boundary,
   record the rationale privately and close the draft instead of publishing it.
   Open an ordinary public issue only after it cannot disclose a vulnerability.
4. If a confirmed defect affects only unreleased source, correct and verify it
   privately, record the release-scope decision, and normally close the draft
   after the correction rather than publishing an advisory for unaffected
   users.
5. Correct a confirmed released vulnerability privately and verify a
   deterministic regression plus the normal release gates. Do not expose the
   correction in a public branch, issue, pull request, or progress document
   before coordinated disclosure.
6. Coordinate the public merge, patched release, and advisory publication so a
   supported safe version is available before or with disclosure when
   practical. Active exploitation or an independently necessary mitigation may
   require earlier notice.
7. Before publication, replace provisional data with exact affected and fixed
   versions, impact, severity, mitigation, credit, and the CVE decision.
8. Publish the advisory when users have a safe upgrade or verified mitigation,
   then link the public advisory from release notes and the non-sensitive
   completion record in `PROJECT.md`.

Keep independent root causes in separate draft advisories even when their fixes
share one coordinated release. Active advisory identifiers, exploit details,
private branches, and correction status do not belong in public project
documentation.

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
