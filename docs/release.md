# Release and installation integrity

The first public release is `mcp-doctor` `0.1.0`, tagged `v0.1.0`. GitHub
Releases is its canonical immutable record. crates.io and the
`EnjoyableWork/tap/mcp-doctor` Homebrew formula must install the exact source
package held by that release.

## What is published

The GitHub Release contains exactly these seven assets:

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

## Verify a GitHub download

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

## Install the exact source release

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

## Publication procedure

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

The manual crates.io and tap handoffs above are the bounded first-publication
procedure, not the intended permanent release process. `MCPD-008A` owns the
transition after the first crate exists. Until its acceptance evidence is
recorded, do not assume that pushing another tag publishes either downstream
channel, and do not create another public version.

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
   represented native host, and runs the passive smoke journey.

`MCPD-008A` must pass a nonpublishing end-to-end rehearsal, including rejected
authorization and byte-mismatch cases, before any version after `v0.1.0` is
tagged. The first later release confirms the retained path against public
channels; it does not authorize weakening these gates.

## Failure and correction

Do not publish a draft when a package, checksum, SBOM, attestation, byte
comparison, or installed passive smoke fails. A draft can be repaired and
reverified because it is not public. After publication, never replace an
asset, move the tag, or overwrite downstream bytes. Correct any defect with a
new version and preserve the evidence explaining what it supersedes.
