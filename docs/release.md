# Release and installation integrity

This source tree represents the `mcp-doctor` `0.3.3` release candidate and its
intended canonical release tag `v0.3.3`.
GitHub Releases determines whether a version has completed public
publication. crates.io and the
`EnjoyableWork/tap/mcp-doctor` Homebrew formula must install the exact source
package held by the corresponding release. The sections below retain the
first-release record and the reusable later-release procedure.

## v0.1.0 first-release record

### Artifacts

The `v0.1.0` GitHub Release contains exactly these seven assets:

```text
SHA256SUMS
mcp-doctor-0.1.0.crate
mcp-doctor.rb
mcp-doctor-v0.1.0-aarch64-unknown-linux-gnu.spdx.json
mcp-doctor-v0.1.0-aarch64-unknown-linux-gnu.tar.gz
mcp-doctor-v0.1.0-x86_64-unknown-linux-gnu.spdx.json
mcp-doctor-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
```

The two native archives are built and tested on matching public GNU/Linux
hosts. Each has a target SPDX 2.3 SBOM. Every asset, including `SHA256SUMS`,
has GitHub build-provenance attestation tied to the tag, release workflow, and
source commit.

Cargo provides source installation on macOS ARM64, GNU/Linux ARM64 and x64,
and Windows x64. Homebrew builds the immutable Cargo source on macOS ARM64 and
GNU/Linux ARM64 and x64. The project does not issue macOS or Windows binaries
in this release, and it does not publish a WinGet package. Those native binary
channels require platform signing and a new version.

### Verify a GitHub download

Download the archive, matching checksum manifest, and attestation from the
[`v0.1.0` release](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.1.0):

```bash
sha256sum --check --ignore-missing SHA256SUMS
gh attestation verify \
  mcp-doctor-v0.1.0-x86_64-unknown-linux-gnu.tar.gz \
  --repo EnjoyableWork/mcp-doctor
gh release verify v0.1.0 --repo EnjoyableWork/mcp-doctor
```

On macOS, use `shasum -a 256 --check SHA256SUMS` for checksum verification.
Review the matching `.spdx.json` document for the archive's packaged software
inventory.

### Install the exact source release

Cargo builds the registry copy while preserving the committed lockfile:

```bash
cargo install mcp-doctor --version '=0.1.0' --locked
```

Homebrew builds the same immutable source package referenced by the published
formula:

```bash
brew install --build-from-source EnjoyableWork/tap/mcp-doctor
brew test EnjoyableWork/tap/mcp-doctor
```

The release-channel workflow independently compares the crates.io download and
the tap formula byte for byte with the immutable GitHub assets. It then
installs each represented channel on its native hosts and runs a passive
diagnostic against a synthetic MCP server. The smoke requires a successful
discovery and catalog report and proves that no tool call was authorized.

The [`v0.1.0` channel-verification run](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31413131715)
passed its immutable identity gate and all nine installed native smokes. The
registry package has SHA-256
`4ebd55311c86533d1d0bb34a223060f551ea8aaeb287de666b51b31b05ceb36d`,
identical to the canonical GitHub asset. The one-time first-publication token
was removed locally and confirmed revoked server-side after publication.

### Publication procedure

1. Work from clean `main` with version `0.1.0`, release notes, and the Rust
   `1.97.1` toolchain pin in agreement.
2. Run the normal quality gates and `.github/workflows/release-preflight.yml`.
   The preflight packages the Cargo source twice, generates the formula twice,
   builds each GNU/Linux archive twice, compares the bytes, validates both
   SPDX documents, and performs every applicable installed smoke without
   release credentials.
3. Recheck the MCP revision, crates.io identity, current `main`, and enabled
   GitHub Release immutability. Run
   `scripts/verify-release-repository-controls.sh <exact-main-commit>` on the
   controlled operator host.
4. Create annotated tag `v0.1.0` at that exact `main` commit and push only the
   tag. `.github/workflows/release.yml` builds the source and Linux artifacts,
   creates and verifies their attestations and checksums, validates the exact
   draft bytes, and publishes only after all checks pass.
5. Require the release API to report `immutable: true`, run
   `gh release verify`, download all seven assets, and compare them with the
   workflow payload before publishing another channel.
6. Publish the exact `.crate` with a short-lived crates.io token limited to
   first publication of `mcp-doctor`. Download it from crates.io and require
   byte equality with the GitHub asset, then revoke the token.
7. Copy the exact `mcp-doctor.rb` release asset to
   `EnjoyableWork/homebrew-tap/Formula/mcp-doctor.rb`; do not regenerate or
   hand-edit it after release. Require the public raw file to match.
8. Run `.github/workflows/release-channels.yml` for `0.1.0`. It is read-only
   and credential-free. Record its native matrix and the immutable release in
   `PROJECT.md`, then open the dated [adoption checkpoint](adoption.md).

## Releases after v0.1.0

The manual crates.io and tap handoffs above were the bounded first-publication
procedure, not the permanent release process. The completed `MCPD-008A` path
below governs every later version. Do not repeat the manual token or formula
copy procedure, and do not create another public version unless these controls
remain verified.

The retained subsequent-release contract is:

1. A reviewed release change and intentionally created annotated stable tag
   remain the release authority. Automation never chooses a version.
2. The generalized GitHub workflow validates the version, source commit,
   successful preflight, release notes, provenance, and exact immutable assets
   before any downstream write.
3. crates.io Trusted Publishing is bound to the exact repository, workflow,
   and protected release environment. GitHub OIDC supplies a short-lived
   publication credential; no crates.io token is stored.
4. The separate Homebrew tap accepts only the exact verified formula through a
   protected tap-owned workflow or narrowly installed GitHub App with a
   short-lived token. A broad personal access token is prohibited.
5. The credential-free release-channel workflow compares the public Cargo and
   Homebrew bytes with the canonical GitHub assets, installs them on every
   represented native host, and runs the installed diagnostic smoke journey
   for the version's advertised command and revision matrix.

`MCPD-008A` passed the required nonpublishing end-to-end rehearsal, including
rejected authorization and byte-mismatch cases, before any version after
`v0.1.0` was allowed. [PROJECT.md](../PROJECT.md) records the exact live
evidence. The first later release confirms the retained path against public
channels; it does not authorize weakening these gates.

## v0.2.0 retained-path verification

[`v0.2.0`](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.2.0)
is the first release published through the retained subsequent-release path.
[PR 14](https://github.com/EnjoyableWork/mcp-doctor/pull/14) produced exact
release commit
[`b0805a8f685e46814e358de368e2a270c21704af`](https://github.com/EnjoyableWork/mcp-doctor/commit/b0805a8f685e46814e358de368e2a270c21704af).
The exact-commit [native CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31528649356)
and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31528649333)
passed before the [protected release workflow](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31529740214)
made the GitHub Release immutable and published the byte-identical
[crates.io package](https://crates.io/crates/mcp-doctor/0.2.0) with short-lived
OIDC authority.

The tap-owned [verification and publication workflow](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/31530330361)
copied the exact release formula in commit
[`a57736ea1a7abf73eeff9a8278af11110247bd20`](https://github.com/EnjoyableWork/homebrew-tap/commit/a57736ea1a7abf73eeff9a8278af11110247bd20).
The credential-free [channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31530466930)
then passed all ten jobs: immutable GitHub, crates.io, and Homebrew byte
identity; two GNU/Linux archive smokes; four native Cargo smokes; and three
native Homebrew smokes. This is the completion evidence for the retained path,
not permission to weaken it for a later release.

## v0.3.0 retained-path verification

[`v0.3.0`](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.3.0)
published the completed optional compatibility, contract-artifact, report-file,
offline-aggregation, and structured protocol-rejection work without changing
the retained release authority or making an M4 assurance claim. Protected
[PR 54](https://github.com/EnjoyableWork/mcp-doctor/pull/54) passed
first-attempt exact-head
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31746397550) and
[release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31746397557),
then merged as exact release source
[`d9b96bbeb84baccb8e5c890e9c655a559a12a474`](https://github.com/EnjoyableWork/mcp-doctor/commit/d9b96bbeb84baccb8e5c890e9c655a559a12a474).
First-attempt exact-`main`
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31754685159) and
[release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31754685137)
passed before the signed annotated tag was intentionally pushed.

The protected [release workflow](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31755736570)
made the GitHub Release immutable, attested and re-downloaded all seven exact
assets, and published the byte-identical
[crates.io package](https://crates.io/crates/mcp-doctor/0.3.0) through OIDC.
The tap-owned [publication workflow](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/31756253855)
verified the release identity and committed the exact formula as
[`2b62e11902c7461cddbc0b96075e3745fdf6f260`](https://github.com/EnjoyableWork/homebrew-tap/commit/2b62e11902c7461cddbc0b96075e3745fdf6f260).
The credential-free [channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31756413098)
then passed all ten jobs on their first attempts: immutable GitHub, crates.io,
and Homebrew identity; two GNU/Linux archive installs; four native Cargo
installs; and three native Homebrew installs. The release used no test, job,
workflow, integrity, generation, validation, or publication retry.

## v0.3.1 coordinated security-patch verification

`v0.3.1` is the coordinated security patch for authority-bearing local file
reads and report artifact publication. It binds selected scenario, custom-CA,
snapshot, and aggregate inputs to the exact no-follow regular-file handle that
is validated and consumed, and it fails closed if the selected path no longer
identifies that file. The trust file is also read and validated before
credential environment resolution, DNS, or connection activity.

The same complete native identity now binds every opened JSON, JUnit, and
aggregate stage to publication: the stage path must still identify the opened
handle before linking, the destination must identify it immediately after,
and cleanup or rollback removes only an identity-owned path. A foreign stage
or destination is never accepted or deleted.

The release preserves every existing endpoint, credential, tool, effect,
side-effect, schema, byte, redaction, and cleanup gate. It does not add a retry,
fallback, broader protocol claim, new installation channel, or new native
binary. Published versions `0.2.0` and `0.3.0` are affected by the
authority-file issue, and `0.3.0` is affected by the report-publication issue.
Users should upgrade to `0.3.1` or later.

Exact release commit
[`d4db369`](https://github.com/EnjoyableWork/mcp-doctor/commit/d4db369a2789f7b6f89b2daad4adc1b6f4900f7e)
passed first-attempt corrected-source
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31984532369),
[CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31984532095),
and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31984532388).
The signed annotated
[`v0.3.1` tag](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.3.1)
then drove the protected
[release workflow](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31985219134),
which published an immutable seven-asset GitHub Release and the byte-identical
[crates.io package](https://crates.io/crates/mcp-doctor/0.3.1) through OIDC.
The tap-owned
[publication workflow](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/31985523936)
committed only the verified formula, and the credential-free
[ten-job channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31985595470)
passed every represented Cargo, Homebrew, and GitHub archive install on its
first attempt.

The coordinated
[authority-file advisory](https://github.com/EnjoyableWork/mcp-doctor/security/advisories/GHSA-92m2-749h-2gv5)
and
[report-publication advisory](https://github.com/EnjoyableWork/mcp-doctor/security/advisories/GHSA-8r6p-qf9j-vpvx)
were published together only after those installed-channel gates completed.
The non-sensitive merge, correction, release, and restoration record is
[retained separately](assurance/mcpd-034-security-release.md).

## v0.3.2 Agent Skill payload contract

`v0.3.2` adds one instruction-only Agent Skill companion to the existing seven
release assets. Its exact eight-asset payload is the `.crate`, Homebrew formula,
two represented GNU/Linux archives, their two SPDX documents, the versioned
`mcp-doctor-agent-skill-v0.3.2.tar.gz`, and `SHA256SUMS`.

The companion archive contains only `mcp-doctor/SKILL.md` at mode `0644`.
Deterministic packaging binds it byte-for-byte to the repository-owned
`.agents/skills/mcp-doctor/SKILL.md`; the release workflow attests the archive,
includes its digest in `SHA256SUMS`, and verifies the archive again after draft
and immutable publication downloads. Historical verification compares the
archive to the canonical skill and guide inside that same release's `.crate`,
not a later checkout. It is documentation, not an executable or an MCP server.

Cargo, Homebrew, and binary-archive installation do not install or update the
skill. Every represented installed CLI smoke instead verifies that top-level
`mcp-doctor --help` points to the exact `v0.3.2` coding-agent guide. Users then
download, verify, install, discover, and remove the companion explicitly under
the [coding-agent guide](agents.md). The skill grants no terminal permission or
active target authority and refuses `check`, `break`, and `reject`.

Protected [PR 95](https://github.com/EnjoyableWork/mcp-doctor/pull/95) merged as
exact release source
[`d117cf4`](https://github.com/EnjoyableWork/mcp-doctor/commit/d117cf4c7cbbd5bfb6dd43c01af2607ae64cc1d2).
That exact `main` commit passed first-attempt
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31995534219),
[CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31995534040),
and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31995534224).
The required nonpublishing
[release/OIDC rehearsal](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31996320198),
[wrong-workflow rejection](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31996322032),
tap [no-write rehearsal](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/31996325240),
and [existing-channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31996323697)
also passed before the tag, with clean before-and-after credential inventories.

The signed annotated
[`v0.3.2` tag](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.3.2)
drove the protected
[release workflow](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31996837111),
which published and independently reverified the immutable eight-asset GitHub
Release and the byte-identical
[crates.io package](https://crates.io/crates/mcp-doctor/0.3.2). The portable
Agent Skill archive has SHA-256
`21c3ad8dba319339060c02523aed049282ada790cbecb691f4f270297b456341`,
and its canonical `SKILL.md` has SHA-256
`f7ee6903c839a268648bf8114e75817396a78f7b08f38a424541fe4b0c483a51`.

The tap-owned
[publication workflow](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/31997316851)
committed the exact formula as
[`b3bfd0d`](https://github.com/EnjoyableWork/homebrew-tap/commit/b3bfd0d084ee5fdaf6553ee6d3c225cd5ad7d302).
The credential-free
[channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31997406753)
then passed all ten jobs: immutable release, registry, formula, provenance, and
Agent Skill identity; two GitHub archive installs; four native Cargo installs;
three native Homebrew installs; and the installed top-level help route.

Protected [PR 96](https://github.com/EnjoyableWork/mcp-doctor/pull/96) then
advanced the current-channel rehearsal default to `0.3.2` and merged as exact
[`9f3a838`](https://github.com/EnjoyableWork/mcp-doctor/commit/9f3a838751856bd20d670053071b6d537f430d37).
That exact protected `main` commit passed first-attempt
[CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31999383802),
[CodeQL](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31999383050),
and [release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31999383921).
The defaulted `0.3.2`
[release/OIDC rehearsal](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32000204735),
[wrong-workflow rejection](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32000204694),
tap [no-write rehearsal](https://github.com/EnjoyableWork/homebrew-tap/actions/runs/32000204757),
and [ten-job channel verifier](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/32000204919)
then passed from the protected source and tap heads. Clean before-and-after
inventories found no Cargo registry token, release-workflow secret reference,
or stored source or tap release-environment secret. This is the completed
current-channel rehearsal for `v0.3.2`.

## v0.3.3 bounded-work security candidate

The `v0.3.3` candidate coordinates two independent CWE-400 corrections. Its
request-scoped SSE decoder consumes accepted bytes incrementally instead of
reparsing the accumulated body after every fragment. Its Draft 2020-12 schema
path charges preliminary traversal, meta-validation, compilation, reference
fan-out, instance access, collection and equality work, and the complete
pattern-by-text product to `schema_evaluation_steps`; exhaustion stops locally
before an affected tool call.

Pattern evaluation uses the validator's bounded linear-time engine. Patterns
requiring backtracking-only constructs such as look-around or backreferences
receive the typed `unsupported_linear_pattern` diagnostic rather than entering
an execution path whose actual work cannot be charged. Existing protocol,
transport, authorization, redaction, cleanup, and reporter contracts remain
unchanged.

The candidate retains the eight-asset release shape introduced by `v0.3.2` and
reissues the canonical instruction-only Agent Skill with exact `0.3.3`
identity. That version-bound skill has no new host-support claim until its
separate exact-version observation passes. The first tag workflow made the
eight-asset GitHub Release immutable, then failed because GitHub had not yet
made its asynchronously generated release attestation available. The
attestation appeared later, but that later state is not a green rerun and the
failed workflow remains the accepted failure record. Corrected protected
source, the explicit partial-release recovery, crates.io and Homebrew handoffs,
installed-channel verification, and coordinated advisory publication remain
required.

### GitHub-controlled sequence

Every later release keeps three deliberately separate write boundaries:

1. `.github/workflows/release.yml` accepts only a canonical stable version
   newer than `0.1.0`, an annotated tag at exact current `main`, matching Cargo
   metadata and notes, and successful CI and release preflight for that commit.
   It rejects a version older than any stable crates.io release and serializes
   every tag and rehearsal through one release concurrency group. It rechecks
   `main` and the annotated tag immediately before publication, then builds,
   attests, byte-checks, makes the GitHub Release immutable, and re-downloads
   the exact immutable bytes.
2. GitHub creates the release attestation asynchronously after publication.
   Before approving the separately protected OIDC job, the operator observes
   one exact `release` attestation for the annotated tag object through the
   repository-attestations API. That job repeats one typed availability read,
   cryptographically verifies the release attestation and every asset's
   provenance, and recreates the exact source handoff from the release tag.
   Only then does the same workflow and protected `release` environment
   exchange GitHub OIDC identity through the
   full-SHA-pinned official `rust-lang/crates-io-auth-action` and run
   `cargo publish --locked`. The temporary token is masked and revoked by the
   Action. The same version may be observed only during recovery from a partial
   run, where its public byte must already equal the immutable asset; a
   different byte fails and is never republished.
3. An operator dispatches the tap-owned `Publish verified mcp-doctor formula`
   workflow in `EnjoyableWork/homebrew-tap` with the exact version and
   `publish` mode. Its read-only job independently requires the annotated tag,
   immutable release, release attestation, asset attestations, checksums,
   package hash, and formula contents. Only then can its protected `release`
   job receive that repository's short-lived `GITHUB_TOKEN` with
   `contents: write`; it copies only `Formula/mcp-doctor.rb` and fails if tap
   `main` moved.

After both downstream bytes are public, dispatch
`.github/workflows/release-channels.yml` with the exact stable version. It has
no write or OIDC permission and no secret input. A later version is not
complete until its registry and formula bytes match the immutable release and
all represented installed smokes pass.

The tap handoff is intentionally operator-dispatched instead of using a
cross-repository personal token. If unattended cross-repository initiation is
ever justified, replace that manual dispatch only with a narrowly installed
GitHub App whose short-lived token can invoke the one tap workflow; do not add
a PAT or give the source repository write access to the tap.

If the sole tag-triggered workflow attempt fails after the release becomes
immutable but before crates.io publication, do not rerun that workflow or
alter the tag or release. Preserve the failure and correct the release path on
new reviewed `main`. After the correction passes its complete local,
protected, rehearsal, and credential-inventory gates, dispatch `release.yml`
from exact `main` with only `recovery_version` set to the immutable version.
The recovery validator accepts only one failed attempt for that exact annotated
tag and source commit, one available release attestation, exact immutable
release state, and an absent crates.io version. Its protected job checks out
the immutable tag, repeats every release, asset, provenance, package, and
handoff check, and either publishes that exact source byte once through OIDC or
fails closed. This recovery is a new reviewed path for a partial release, not
an accepted green rerun of failed source.

### Trusted publisher and environment bindings

The crates.io publisher must contain exactly this GitHub identity:

| Field | Required value |
| --- | --- |
| Repository owner | `EnjoyableWork` |
| Repository | `mcp-doctor` |
| Workflow filename | `release.yml` |
| Environment | `release` |

The `mcp-doctor` `release` environment permits only `main` for the explicit
nonpublishing rehearsal or reviewed partial-release recovery, and stable
`v*.*.*` tags for initial publication. The tap's
separate `release` environment permits only tap `main`. Both environments
have a required-reviewer gate and store no secret. The current one-maintainer
organization allows its administrator to bypass that gate, so this is an
intentional single-maintainer control, not independent two-person approval;
normal releases should use the recorded approval path rather than bypass it.

### Required nonpublishing rehearsal

Before any later tag is allowed, run these workflows from their exact default
branches without changing `Cargo.toml`, creating a tag, or publishing a byte:

1. Dispatch `Publish verified immutable release` with rehearsal version
   currently represented identically across GitHub Releases, Cargo, and Homebrew
   (`0.3.2` at this review) and leave `recovery_version` empty. It reuses that
   existing immutable release, compares
   the real Cargo and Homebrew bytes, rejects synthetic provenance and mutated
   byte fixtures, obtains and revokes one short-lived token through the
   authorized workflow and environment, and proves the same workflow is
   rejected without the environment. No publish command exists in the authorization job.
   An older version is not a valid current-channel rehearsal after the rolling
   Homebrew formula advances.
2. Dispatch `Verify crates.io workflow authorization boundary`. Approve the
   same environment and require crates.io to reject it because its workflow
   filename is not `release.yml`.
3. Dispatch the tap's `Publish verified mcp-doctor formula` workflow with
   version `0.1.0` and `rehearse` mode. This retained no-write authorization
   fixture intentionally reuses the first immutable handoff; the write-capable
   job is structurally skipped, while immutable, provenance, checksum, formula,
   and negative mismatch checks still run. Current rolling bytes are covered by
   the source-repository rehearsal and the channel verifier.
4. Dispatch `Verify published release channels` for the same selected version.
   This confirms the generalized verifier remains credential-free and all
   existing public channel bytes and installed diagnostic smokes still pass.

The dated M4 supply-chain operator audit is a separate historical-evidence
gate. When it is required, `scripts/verify-supply-chain-controls.sh`
authenticates the canonical `v0.3.0` Homebrew formula at its recorded immutable
full tap commit; it does not require rolling `homebrew-tap/main` to remain at
that historical commit. The repeat-release audit, current-version rehearsal,
and channel verifier above own current rolling formula state. Neither boundary
may substitute for the other, and a failed audit is corrected on new reviewed
source rather than rerun unchanged.

The verification operator profile remains read-only. GitHub's REST
`Get a repository` response exposes merge-related settings only to credentials
with both `Contents: read` and `Contents: write`, so the audit must not infer
that an omitted `allow_auto_merge` field is a disabled setting and must not add
write authority merely to make that field appear. It verifies the same
repository's `autoMergeAllowed` value through the read-only GraphQL repository
field instead. A missing, malformed, mismatched, or `true` field fails closed.
The REST response continues to own repository identity, visibility, archive,
default-branch, and security-update state.

`PROJECT.md` records the initial four successful run links, exact
environment-policy readback, and trusted-publisher readback. Repeat and record
this rehearsal after any workflow, environment, publisher, or authority change;
a workflow file or local test alone is not completion evidence.

### Credential inventory gate

Before and after the rehearsal, inventory credential names and configuration
without printing values. Every repository, environment, and organization
secret endpoint must return a successfully parsed typed inventory; a permission
error, missing response, malformed shape, or unavailable selected-repository
mapping fails closed and cannot emit the success record. Run this inventory
with the same exact read-only verification credential used by the operator
audit rather than widening another session. Acceptance requires:

- no Cargo credential for crates.io in the operator's active Cargo home;
- no repository, environment, or organization Actions secret used by either
  release path;
- no classic or fine-grained personal access token referenced by either
  workflow;
- the exceptional verification-operator token retains the exact canonical
  read-only profile and must not gain `Contents: write` for a REST projection;
- only the ephemeral crates.io token returned to the exact `release.yml` job,
  automatically revoked when that job ends; and
- only the tap repository's per-run `GITHUB_TOKEN`, with `contents: write`
  limited to the approved copy job.

The operator's authenticated GitHub session is administration authority, not a
release credential: workflows must not read, copy, or depend on it.

## Failure and correction

Do not publish a draft when a package, checksum, SBOM, attestation, byte
comparison, or installed diagnostic smoke fails. A draft can be repaired and
reverified because it is not public. After publication, never replace an
asset, move the tag, or overwrite downstream bytes. Correct any defect with a
new version and preserve the evidence explaining what it supersedes.
