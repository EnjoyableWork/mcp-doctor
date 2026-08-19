# MCPD-036 coordinated bounded-work security-release record

This non-sensitive record closes `MCPD-036` and two distinct CWE-400
advisories on 2026-08-18. It records only public commits, pull requests,
workflows, release artifacts, advisories, aggregate repository-control state,
and value-free credential-inventory outcomes. It contains no private
reproducer, endpoint, credential, payload, schema content, tool argument,
result, or advisory-workspace material.

## Scope and outcome

`DEC-060` coordinated two independently bounded corrections in `v0.3.3`:

- request-scoped SSE now uses one incremental decoder whose scan work is
  linear in accepted bytes and whose current-line, event, payload, message,
  and aggregate state remains bounded; and
- Draft 2020-12 schema processing now charges preliminary analysis,
  meta-validation, validator construction, reference fan-out, instance
  access, collection, equality, uniqueness, and pattern work to one
  deterministic operation budget before an affected tool call.

The fixes share a release because leaving either defect in the latest channel
would not complete the bounded-work correction. They retain separate
regressions and advisories because their root causes and affected lines differ.
The release adds no protocol revision, transport, active authority, retry,
fallback, dependency-graph expansion, platform-signing claim, or broader
compatibility or assurance claim.

## Preserved failures and reviewed corrections

The first pre-tag operator audit on exact protected `main`
`f73b4511fc955b99b3d5c889a6706b1e0a96a7a1` preserved this bounded failure:
`date=2026-08-18 canonical_sha256=bdc0b1009b4bb40e0a4cbf58aaf07ae511a644548502fb386b69750150029767 source_sha=f73b4511fc955b99b3d5c889a6706b1e0a96a7a1 result=FAIL`.
It exposed a stale requirement that rolling `homebrew-tap/main` equal the
historical `v0.3.0` handoff commit. The correction separated immutable
historical Homebrew proof from rolling tap state.

The next operator audit on exact protected `main`
`1d5a2ccebec45709a5f7fadad19de5f15bc837d9` preserved a distinct failure:
`date=2026-08-18 canonical_sha256=aa247183c668479877cd8ffb99144997888f6e8b118c6f429bbef40174c5230b source_sha=1d5a2ccebec45709a5f7fadad19de5f15bc837d9 result=FAIL`.
GitHub withheld the REST merge-setting field from the exact read-only
credential, so the correction moved that assertion to the exact read-only
GraphQL field. The verification credential never received contents-write
authority, and neither failed source tree was rerun unchanged.

A subsequent organization-secret inventory received `403`, but the original
shell path lost that status and printed success. That result was rejected.
[PR #103](https://github.com/EnjoyableWork/mcp-doctor/pull/103) made the
inventory fail closed on unavailable or malformed evidence and merged as exact
release source
[`995d471`](https://github.com/EnjoyableWork/mcp-doctor/commit/995d471b0024a6d1e16b85e1778168bd27d3aebc).

Signed annotated tag `v0.3.3`, tag object
`a889b6fdba86205b33a1f9641140ca06231bbf35`, resolves to that exact commit. Its
sole tag-triggered
[release run](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32095369800)
made the eight-asset GitHub Release immutable, then failed before crates.io
because GitHub had not yet exposed the asynchronously generated release
attestation. The later attestation diagnosed the readiness race but did not
turn that attempt green. The run was not rerun, and the tag, release, and
assets were not moved, edited, or replaced.

`DEC-064` introduced a separately reviewed exact-byte recovery path.
[PR #104](https://github.com/EnjoyableWork/mcp-doctor/pull/104) merged its
byte-identical reviewed tree as
[`7e5fff3`](https://github.com/EnjoyableWork/mcp-doctor/commit/7e5fff3b7fa953a4ae371739a6046db9cd56feca).
First-attempt exact-head
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32096997746),
[CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32096995363),
and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32096997620),
then first-attempt exact-`main`
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32097796559),
[CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32097796067),
and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32097796654),
all passed. No unchanged workflow, test, build, integrity check, publication,
job, or complete workflow was retried into acceptance.

## Recovery and channel evidence

The corrected operator audit passed once on exact recovery source
`7e5fff3b7fa953a4ae371739a6046db9cd56feca`
with canonical control SHA-256
`aa247183c668479877cd8ffb99144997888f6e8b118c6f429bbef40174c5230b`.
Fail-closed before and after inventories agreed on that exact source and tap
source `b3bfd0d084ee5fdaf6553ee6d3c225cd5ad7d302`, with one bounded 82-byte
success line, empty stderr, and no retained publication credential. Four
first-attempt nonpublishing rehearsals then passed:

- the [release/OIDC path](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32098636374);
- [wrong-workflow rejection](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32098637731);
- the tap-owned [no-write path](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/32098639806); and
- the [ten-job existing-channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32098641058).

The reviewed
[recovery run](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32099327284)
accepted only exact `main`, the one failed attempt, the exact immutable tag and
release, one matching release attestation, and an absent crates.io version. It
checked out the immutable release source, repeated release, asset, provenance,
package, formula, and handoff verification, then published only the exact
crate through the existing protected OIDC boundary. The GitHub and crates.io
crate bytes are both 689,739 bytes with SHA-256
`a6fc434131ab109d7e9bf85e8cdff17cad34590c2a65ee4bf91578eb4310b4ec`.

The tap-owned
[publication](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/32099555744)
changed only `Formula/mcp-doctor.rb` in
[`4a2e2f3`](https://github.com/EnjoyableWork/homebrew-tap/commit/4a2e2f3ba88dad5a8d80cba42c3ee07c38da18bc).
Its source SHA-256
`dc1dbccfeb66e38a5b404d8183d3afdb7a21e07ecfb09f8a432655036e794096`
matches the immutable release handoff. The final credential-free
[release-channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32099683447)
passed all ten represented identity and installed-smoke jobs on its first
attempt. The final inventory agreed on exact source
`7e5fff3b7fa953a4ae371739a6046db9cd56feca` and tap
`4a2e2f3ba88dad5a8d80cba42c3ee07c38da18bc`.

The completed public channels are the immutable
[GitHub Release](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.3.3),
the byte-identical
[crates.io package](https://crates.io/crates/mcp-doctor/0.3.3), and the exact
Homebrew tap commit above.

## Closure-source verification

The first closure-source formatting check stopped on one formatter-owned test
line wrap. The source was changed to the exact formatted shape before the
focused release and ticket-state tests passed. The first complete
`scripts/check.sh` gate then stopped when one organization-policy assertion
still required the old two-ticket milestone phrase; the same stale assertion
was corrected in all three owning policy suites on new source, and their
focused tests passed. Neither failed source tree was rerun unchanged.

The corrected closure source passes `cargo fmt --all -- --check`, Actionlint
`1.7.12`, Bash syntax, ShellCheck `0.11.0`, the disposable synthetic
supply-chain-control rehearsal, the complete disposable-environment
`scripts/check.sh` gate, `cargo deny --all-features --locked check`, and
`git diff --check`. The closure changes no dependency or release byte.

## Coordinated disclosure and credential closure

The first publication request was rejected without changing advisory state
because the private workspace still had an open correction pull request. Its
head was proved already present in the public correction or superseded by the
later release-control source; only that workspace pull request was closed.
Both advisories were then published at `2026-08-18T04:50:21Z`, after every
represented channel passed:

- [GHSA-3vpj-fcvj-28pm](https://github.com/EnjoyableWork/mcp-doctor/security/advisories/GHSA-3vpj-fcvj-28pm)
  covers fragmented SSE in `>= 0.3.0, <= 0.3.2`; and
- [GHSA-jr72-f9q4-424m](https://github.com/EnjoyableWork/mcp-doctor/security/advisories/GHSA-jr72-f9q4-424m)
  covers unmetered JSON Schema work in `>= 0.1.0, <= 0.3.2`.

Both identify `>= 0.3.3` as patched. After the final exact-source and tap
inventory, the task-scoped fine-grained credential was submitted to GitHub's
credential-revocation endpoint. One bounded later observation proved GitHub
rejected it; no token value or identity was retained in project evidence.

This closure advances only the future nonpublishing release rehearsal default
and its synthetic rolling-tap control from verified `0.3.2` to verified
`0.3.3`. It does not mutate the signed tag, immutable release, downstream
bytes, advisories, product runtime, or credential boundary.
