# Release and installation integrity

This source tree represents `mcp-doctor` `0.3.0` and its canonical release tag
`v0.3.0`. GitHub Releases determines whether a version has completed public
publication. crates.io and the `EnjoyableWork/tap/mcp-doctor` Homebrew formula
must install the exact source package held by the corresponding release. The
sections below retain the first-release record and the reusable later-release
procedure.

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

### GitHub-controlled sequence

Every later release keeps three deliberately separate write boundaries:

1. `.github/workflows/release.yml` accepts only a canonical stable version
   newer than `0.1.0`, an annotated tag at exact current `main`, matching Cargo
   metadata and notes, and successful CI and release preflight for that commit.
   It rejects a version older than any stable crates.io release and serializes
   every tag and rehearsal through one release concurrency group. It rechecks
   `main` and the annotated tag immediately before publication, then builds,
   attests, byte-checks, and makes the GitHub Release immutable.
2. Only after those public bytes and attestations verify, the same workflow and
   protected `release` environment exchange GitHub OIDC identity through the
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

### Trusted publisher and environment bindings

The crates.io publisher must contain exactly this GitHub identity:

| Field | Required value |
| --- | --- |
| Repository owner | `EnjoyableWork` |
| Repository | `mcp-doctor` |
| Workflow filename | `release.yml` |
| Environment | `release` |

The `mcp-doctor` `release` environment permits only `main` for the explicit
nonpublishing rehearsal and stable `v*.*.*` tags for publication. The tap's
separate `release` environment permits only tap `main`. Both environments
have a required-reviewer gate and store no secret. The current one-maintainer
organization allows its administrator to bypass that gate, so this is an
intentional single-maintainer control, not independent two-person approval;
normal releases should use the recorded approval path rather than bypass it.

### Required nonpublishing rehearsal

Before any later tag is allowed, run these workflows from their exact default
branches without changing `Cargo.toml`, creating a tag, or publishing a byte:

1. Dispatch `Publish verified immutable release` with rehearsal version
   `0.1.0`. It reuses the existing immutable release, compares the real Cargo
   and Homebrew bytes, rejects synthetic provenance and mutated byte fixtures,
   obtains and revokes one short-lived token through the authorized workflow
   and environment, and proves the same workflow is rejected without the
   environment. No publish command exists in the authorization job.
2. Dispatch `Verify crates.io workflow authorization boundary`. Approve the
   same environment and require crates.io to reject it because its workflow
   filename is not `release.yml`.
3. Dispatch the tap's `Publish verified mcp-doctor formula` workflow with
   version `0.1.0` and `rehearse` mode. The write-capable job is structurally
   skipped; immutable, provenance, checksum, formula, and negative mismatch
   checks still run.
4. Dispatch `Verify published release channels` for `0.1.0`. This confirms the
   generalized verifier remains credential-free and all existing public
   channel bytes and installed diagnostic smokes still pass.

`PROJECT.md` records the initial four successful run links, exact
environment-policy readback, and trusted-publisher readback. Repeat and record
this rehearsal after any workflow, environment, publisher, or authority change;
a workflow file or local test alone is not completion evidence.

### Credential inventory gate

Before and after the rehearsal, inventory credential names and configuration
without printing values. Acceptance requires:

- no Cargo credential for crates.io in the operator's active Cargo home;
- no repository, environment, or organization Actions secret used by either
  release path;
- no classic or fine-grained personal access token referenced by either
  workflow;
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
