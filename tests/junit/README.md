# JUnit projection compatibility evidence

This directory records the independently exercised consumer boundary for the
`mcp-doctor.report/v1` JUnit projection.
It is not a JUnit standard claim, a guarantee for every CI product or plug-in version, or a
replacement for the authoritative stable JSON report and process exit status.

## Selected common subset

The projection emits one top-level `testsuites`, one `testsuite`, and one
`testcase` per diagnostic check. It uses only these case children:

- `failure` for a performed failed check;
- `skipped` for skipped or causally blocked evidence; and
- `system-out` for the stable schema identity, report outcome and exit, check
  state, safe findings, causal links, and structural reproduction evidence.

The root, suite, and cases carry deterministic names, zero duration, and exact
test/failure/error/skip counts. XML 1.0 text and attributes are escaped; invalid
XML scalar values become U+FFFD. Warning and information findings remain in a
passing case's `system-out`. JUnit never causes another target run and never
becomes the portable gate: consumers must retain the `mcp-doctor` process exit.

The checked-in
[`failed-report.junit.xml`](../fixtures/contracts/failed-report.junit.xml)
fixture contains three cases: one success, one failure, and one skip. Ordinary
Rust tests parse this common subset independently with `quick-xml`, compare it
byte-for-byte with the typed renderer, and exercise passive, reviewed active,
generated, STDIO, and HTTP built-binary journeys.

## Independent consumer checks

On 2026-08-11, the canonical fixture was passed directly to two independent CI
parsers in disposable, networked preparation environments. Runtime parsing was
local and the checkouts and dependency caches were removed afterward.

| Consumer | Immutable source and environment | Observed import |
| --- | --- | --- |
| Jenkins JUnit plugin | [`jenkinsci/junit-plugin` `67a81935603ce6740d5036f23f867ada49bd5cb3`](https://github.com/jenkinsci/junit-plugin/commit/67a81935603ce6740d5036f23f867ada49bd5cb3) exercised with Maven `3.9.11` and Eclipse Temurin 21 from Docker image index `sha256:6fdc855a6ed81d288ca7ca37ac6ff5e9308b612485c0801d70b25a858c83d237` | `TestResult.parse`, followed by the plug-in's required `tally`, imported 3 total, 1 failed, and 1 skipped case |
| GitLab JUnit parser | [`gitlab-org/gitlab` `7f38b981fe5d1895345f265b70773e98927b0893`](https://gitlab.com/gitlab-org/gitlab/-/commit/7f38b981fe5d1895345f265b70773e98927b0893) with its locked ActiveSupport `7.2.3.1` and Nokogiri `1.19.3`, exercised under Docker Official Image `ruby:3.3.11-slim` index `sha256:6043d86f12575b1f4a5cf28b93fd664413a1a24b43c361a14d20a071617e7806` | `Gitlab::Ci::Parsers::Test::Junit` imported 3 total: 1 success, 1 failed, and 1 skipped case with no suite error |

The selected elements also match GitLab's documented JUnit report contract and
Azure Pipelines' documented JUnit `testsuites/testsuite/testcase/system-out`
path. That documentation review broadens confidence in the chosen subset but is
not recorded as an executed consumer result.

## Update rules

Changing the element mapping, classification, escaping, count, metadata, or
fixture is a compatibility review. Re-run the ordinary locked suite and at
least the two selected independent parsers at pinned commits, record the new
date and exact versions, and keep preparation outside normal CI. A consumer
failure narrows or removes the compatibility statement; it must not be hidden
by dropping safe diagnostic evidence or weakening the authoritative JSON and
exit contract.
