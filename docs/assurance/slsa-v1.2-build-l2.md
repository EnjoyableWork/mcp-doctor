# SLSA `v1.2` Build L2 evaluation for `v0.3.0`

This dated evaluation applies SLSA's approved
[`v1.2` Build L2](https://slsa.dev/spec/v1.2/build-track-basics) requirements
separately to every asset in `mcp-doctor`'s canonical immutable
[`v0.3.0` GitHub Release](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.3.0).
It is an artifact-specific evaluation, not a certification, project-wide SLSA
badge, or claim about registry, Homebrew, dependency, unlisted, or future
artifacts.

| Field | Evaluated value |
| --- | --- |
| Evaluation date | 2026-08-15 UTC |
| SLSA target | `v1.2`, Build track, Level 2 |
| Repository | `EnjoyableWork/mcp-doctor` |
| Release | `v0.3.0`, immutable, published 2026-08-14 00:02:39 UTC |
| Annotated tag object | `6d3032426c4d9a7d624eb771fbbc30fe7605801b` |
| Exact source commit | [`d9b96bbeb84baccb8e5c890e9c655a559a12a474`](https://github.com/EnjoyableWork/mcp-doctor/commit/d9b96bbeb84baccb8e5c890e9c655a559a12a474) |
| Signer workflow | `.github/workflows/release.yml` at `refs/tags/v0.3.0` |
| Hosted build | GitHub Actions run [`31755736570`, attempt 1](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31755736570/attempts/1) |
| Provenance predicate | `https://slsa.dev/provenance/v1` |
| Consumer verifier | GitHub CLI `2.97.0`, acquired from exact reviewed release `v2.97.0` by immutable host-asset size and SHA-256 |
| Result | All 7 canonical release assets meet the evaluated Build L2 requirements |

## Exact artifact results

Each downloaded byte count and SHA-256 digest matched the immutable release
inventory before provenance verification. Each row then passed the same
constrained signature, identity, source, builder, predicate, and subject checks.

| Canonical release asset | Bytes | SHA-256 | Build L2 |
| --- | ---: | --- | --- |
| `mcp-doctor-0.3.0.crate` | 480,469 | `f27ef1bfbe3eeed2f365065d44a95d5952f795c7d01c05a98372d770dc7953af` | Meets |
| `mcp-doctor-v0.3.0-aarch64-unknown-linux-gnu.spdx.json` | 511,408 | `a68971007458c160b3d8aa0b2e238664a5db3652d77ba63cf17957170c38b321` | Meets |
| `mcp-doctor-v0.3.0-aarch64-unknown-linux-gnu.tar.gz` | 4,957,507 | `06c7887fc0087384ac81af693ad66222d4a5d5c91373efa2adb93a033474c672` | Meets |
| `mcp-doctor-v0.3.0-x86_64-unknown-linux-gnu.spdx.json` | 511,153 | `a28f05cc5b179396bb56051b4f4f768c21422945f21e636f42bad29dbbea4346` | Meets |
| `mcp-doctor-v0.3.0-x86_64-unknown-linux-gnu.tar.gz` | 5,033,486 | `e44b26e87dc68dc001ed0fb8cef8094e95c9e04240a13b47b0b6363112a83d08` | Meets |
| `mcp-doctor.rb` | 1,844 | `25ac053224e8eb687b8ca12323eca98610f4be6f47ef05f43530e7082cafbfd8` | Meets |
| `SHA256SUMS` | 641 | `3ae870041d6a58bd88b606d8abf95b1dbd5e735ebeb812aa09f7444eecf2fa92` | Meets |

## Requirement crosswalk

| SLSA `v1.2` Build requirement | Result | Evidence and reasoning |
| --- | --- | --- |
| Build L1 producer: follow a consistent build process. | Meets | The reviewed release workflow is the single build definition for all seven subjects. The exact tag, source commit, run, attempt, workflow path, and expected asset inventory are fixed and public; deterministic release-preflight checks ran before publication. |
| Build L1 producer: use a platform meeting Build L1. | Meets | Every subject's verified provenance names GitHub Actions and the exact release workflow as builder and build type; the certificate fixes the runner environment to `github-hosted`. |
| Build L1 producer: distribute provenance to consumers. | Meets | GitHub stores the signed attestations with the public repository and makes them retrievable for every exact subject through the public attestation route used by `gh attestation verify`. |
| Build L1 platform: automatically describe builder, process, and top-level input. | Meets | Every verified SLSA provenance statement contains the workflow builder ID, GitHub Actions workflow build type, exact workflow repository/path/ref, and resolved Git dependency at source commit `d9b96bbeb84baccb8e5c890e9c655a559a12a474`. |
| Build L2 producer: use a hosted platform meeting Build L2. | Meets | The certificate and predicate both identify `github-hosted`, never a self-hosted runner. GitHub documents artifact attestations as providing SLSA v1.0 Build L2; this evaluation separately applies the unchanged hosted signed-provenance and consumer-verification requirements in SLSA `v1.2`. |
| Build L2 platform: generate and sign the provenance itself. | Meets | GitHub Actions issued the OIDC identity and GitHub's artifact-attestation service signed the in-toto statement. Verified certificate fields bind the repository, exact signer workflow and digest, exact source ref and digest, GitHub-hosted runner, and run invocation. No repository signing key is stored in or exposed to the workflow. |
| Build L2 consumer: validate provenance authenticity. | Meets | GitHub CLI `2.97.0` verified the Sigstore bundle and certificate for each downloaded subject while constraining repository, signer workflow, signer digest, source ref, source digest, OIDC issuer, predicate type, and non-self-hosted runner. Structural checks then matched each subject name and digest plus the expected builder, workflow build type, dependency, and invocation. |

SLSA `v1.2` describes Build L2 as signed provenance generated by a hosted build
platform and requires the consumer to validate authenticity. The
[verification guidance](https://slsa.dev/spec/v1.2/verifying-artifacts) also
calls for the consumer to match the subject, signature, trusted builder, build
type, and external parameters. Those are the exact properties constrained or
inspected here. No Build L3 claim follows: this evaluation does not assert the
stronger build-isolation and provenance-secret protections required there.

## Verification procedure

The verifier identity is pinned in
[`.github/assurance-controls.json`](../../.github/assurance-controls.json).
GitHub CLI `2.97.0` is an immutable, verified-source release of the official,
GitHub-maintained `cli/cli` project, published 2026-07-31 from verified commit
`55dbb4dc6b7edb10b48e3d7fc5bccd32318d1b55`. Its 1,950-byte checksum manifest
has SHA-256 `61905c69ec8660f310814ec98395cdd0c2d07aabf024c597ec45813984a02334`.
The verifier accepts only the 13,845,290-byte macOS ARM64 archive with SHA-256
`a58b8fd77b417a38f47a0b54d1370c59b0fcdb324ccc9ca002b0998f7c4c999e` or
the 14,770,812-byte GNU/Linux x64 archive with SHA-256
`a2c9b8497e1f85b1ad0dfcb78b5a622e098801b8e461e459e88e1ee12f018112`.
It checks the manifest, archive layout, regular executable, and reported
version before use and never retries acquisition.

This is a manual assurance verifier, not a product runtime, build, or Rust
dependency. `cli/cli` is active, MIT-licensed, has a public GitHub security
reporting route and regular release/issue activity, and showed no ownership or
provenance change requiring escalation in the dated review. The selected
prebuilt Go CLI introduces no Cargo feature, duplicate crate, build script,
`unsafe` Rust, minimum Rust version, or product binary/startup/runtime cost;
its transitive Go build graph is upstream's prebuilt-binary boundary rather
than code compiled by this repository. Execution is limited to public GitHub
metadata and seven attestation verifications in a disposable root on the two
reviewed verifier hosts. Any release, source, maintainer, security-route,
license, advisory, archive, platform, or version change requires the complete
testing-tool review again.

After downloading each exact release URL without a retry, the verification used
the following constraints (with `$asset` set to one row above):

```bash
gh attestation verify "$asset" \
  --repo EnjoyableWork/mcp-doctor \
  --signer-workflow EnjoyableWork/mcp-doctor/.github/workflows/release.yml \
  --signer-digest d9b96bbeb84baccb8e5c890e9c655a559a12a474 \
  --source-ref refs/tags/v0.3.0 \
  --source-digest d9b96bbeb84baccb8e5c890e9c655a559a12a474 \
  --cert-oidc-issuer https://token.actions.githubusercontent.com \
  --deny-self-hosted-runners \
  --predicate-type https://slsa.dev/provenance/v1 \
  --format json
```

For all seven results, the verified certificate and statement agreed on these
structural facts:

- certificate subject alternative name and builder ID:
  `https://github.com/EnjoyableWork/mcp-doctor/.github/workflows/release.yml@refs/tags/v0.3.0`;
- certificate OIDC issuer: `https://token.actions.githubusercontent.com`;
- workflow, signer, source, and build-config digest:
  `d9b96bbeb84baccb8e5c890e9c655a559a12a474`;
- source and workflow ref: `refs/tags/v0.3.0`;
- runner environment: `github-hosted`;
- invocation:
  `https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31755736570/attempts/1`;
- build type: `https://actions.github.io/buildtypes/workflow/v1`;
- resolved Git dependency: exact source commit and tag above; and
- one subject whose name and SHA-256 matched the downloaded asset row.

The repeatable public procedure is implemented by
[`scripts/verify-assurance-evidence.sh`](../../scripts/verify-assurance-evidence.sh).
Its final output retains only the date, canonical configuration hash, exact
source commit, assessment ID, release tag, asset count, and `PASS`/`FAIL`.

## Scope, maintenance, and removal

This evaluation covers only the seven named GitHub Release assets as immutable
bytes. It does not cover the crates.io upload operation, the Homebrew tap update
or install operation, dependency provenance, source control as a SLSA Source
track result, unlisted files, another release, future artifacts, or signing on
macOS or Windows. It is not a claim that the entire project or every channel is
SLSA certified.

Revalidate at least annually, next due by 2027-08-15, and after any SLSA
version or requirement, GitHub attestation service, verifier, repository,
workflow, Action, dependency, runner, release pipeline, asset inventory,
digest, tag, source commit, or provenance change. Any mismatch blocks the
artifact-specific claim and requires immediate correction or removal; an
eventual rerun does not erase failed evidence.
