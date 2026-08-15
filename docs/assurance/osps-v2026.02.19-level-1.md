# OpenSSF OSPS Baseline `v2026.02.19` Level 1

This is `mcp-doctor`'s dated, scoped self-assessment against all 24 Level 1
controls in the [OpenSSF OSPS Baseline `v2026.02.19`](https://baseline.openssf.org/versions/2026-02-19).
The [BadgeApp baseline-1 record](https://www.bestpractices.dev/en/projects/14096/baseline-1)
is an official-hosted self-assessment, reviewed and published by the owner. It
is not an independent certification,
regulatory-compliance result, warranty, or claim about OSPS Level 2 or Level 3.

| Field | Assessed value |
| --- | --- |
| Assessment date | 2026-08-15 UTC |
| Repository | [`EnjoyableWork/mcp-doctor`](https://github.com/EnjoyableWork/mcp-doctor) |
| Source snapshot | Exact `main` commit [`21b189f9fd9ed97f1fcaf9d47c75b4f120678689`](https://github.com/EnjoyableWork/mcp-doctor/commit/21b189f9fd9ed97f1fcaf9d47c75b4f120678689) |
| Release boundary | Canonical immutable GitHub Release [`v0.3.0`](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.3.0), source commit `d9b96bbeb84baccb8e5c890e9c655a559a12a474` |
| Organization boundary | Organization-wide access controls for `EnjoyableWork`; repository-specific credential evidence only for `EnjoyableWork/mcp-doctor` and `EnjoyableWork/homebrew-tap` |
| Result | 24 `Met`; 0 `N/A`; 0 `Unmet` |
| Official record | BadgeApp project [`14096`](https://www.bestpractices.dev/en/projects/14096/baseline-1), achieved 2026-08-15 22:14:15 UTC; its [public JSON](https://www.bestpractices.dev/projects/14096.json) and [provider badge](https://www.bestpractices.dev/projects/14096/baseline) are recorded in [`.github/assurance-controls.json`](../../.github/assurance-controls.json) |

The exact-version gate was rechecked on 2026-08-15. The official OSPS current
version remained `v2026.02.19`; its Level 1 checklist still contained these 24
controls. BadgeApp still selected `v2026.02.19` with no transition at upstream
commit [`2a9478f`](https://github.com/ossf/best-practices-badge/blob/2a9478f85b9cbcc404ac15cb1d0ccd75ed43cc16/app/lib/baseline_config.rb),
and the official public assessment route remained available. The checked-in
[proposal](../../.bestpractices.json) gives the owner the same 24 answers for
review; it is not itself the official result.

## Assessment method

Every control below is applicable. `Met` means the requirement was matched to
non-sensitive repository, release, or bounded live evidence. A checked-in file
alone was not treated as proof of an effective GitHub setting. Existing live
verifiers were re-run where their current authority remained available:

- the public `main`-protection verifier passed on 2026-08-15 with canonical
  SHA-256 `2e3377a5101c513c02bb177cbc95acc3707f77bab4c3ab8ed3e8576a3f828794`;
- the repository-security verifier passed on 2026-08-15 with canonical SHA-256
  `d3ecae509ded466c373e6a679a503c2c94a4cf508346ea680dd54926a2354730`;
- the credential-free community and license verifier passed exact `main`
  `21b189f9fd9ed97f1fcaf9d47c75b4f120678689` on 2026-08-15 with canonical
  SHA-256 `41361ad8db72283147783b7c582ab1560d7607e6b6ec80880a5e5c24e5aba873`;
- the `MCPD-017` non-disclosing organization verifier passed on 2026-08-15 with
  canonical SHA-256 `8f3b2c3db5f81a174d93bbcdaa8341e816b15c6ae059329fe5d61925c91a8111`.
  Its one-run credential was then revoked and rejected on reuse, so this
  assessment reuses that same-day bounded result instead of recreating
  standing authority; and
- the supply-chain control history, exact source policy, immutable release,
  and all seven `v0.3.0` asset attestations were rechecked. The separate
  [SLSA evaluation](slsa-v1.2-build-l2.md) records the artifact-level result.

Private factor identities, member or App mappings, recovery material,
credentials, security findings, and secret values are intentionally absent.

## Level 1 control crosswalk

| Control | Requirement and applicability | Status | Evidence |
| --- | --- | --- | --- |
| `OSPS-AC-01.01` | Sensitive authoritative-repository access must require MFA. The organization and public source repository are in scope. | Met | The verified [organization projection](../../.github/organization-controls.json) requires secure MFA and records zero noncompliant members or outside collaborators; the same-day non-disclosing live pass above proves the effective bounded state. |
| `OSPS-AC-02.01` | New collaborators must receive manual permissions or the lowest default privilege. The organization collaborator boundary is in scope. | Met | The verified organization projection fixes default repository permission to `none`, manual assignment, no pending invitations, and no non-owner direct administrators; the same-day live pass checked the effective state. |
| `OSPS-AC-03.01` | Direct commits to the primary branch must be prevented. `main` is the primary branch. | Met | The canonical [ruleset](../../.github/rulesets/main.json) requires pull requests and status checks with no standing bypass. The public verifier matched configured and effective rules on 2026-08-15; earlier bounded direct-update exercises were rejected. |
| `OSPS-AC-03.02` | Primary-branch deletion must require explicit sensitive-action handling. `main` is in scope. | Met | The same ruleset blocks deletion and the public verifier matched that rule on 2026-08-15; the bounded deletion exercise was rejected without changing `main`. |
| `OSPS-BR-01.01` | CI/CD use of untrusted metadata must sanitize or validate it. Pull-request and release metadata paths are in scope. | Met | The [supply-chain projection](../../.github/supply-chain-controls.json), full-SHA workflow inventory, least-privilege workflows, source-policy tests, and protected exact-head/exact-`main` evidence prove closed expected-value handling instead of shell-evaluated metadata. |
| `OSPS-BR-01.03` | Untrusted code snapshots must not reach privileged CI/CD credentials or assets. Public pull requests are in scope. | Met | Pull-request workflows retain only `contents: read`, no stored or OIDC credentials, and no persisted checkout authority. Disposable fork PR 29 independently proved absent credentials and a rejected write before closing unmerged. |
| `OSPS-BR-03.01` | Listed official project URIs must use encrypted delivery. All official channels are applicable. | Met | The [community and license projection](../../.github/community-license-controls.json) contains only HTTPS official channels. The credential-free verifier reached and revalidated them on 2026-08-15. |
| `OSPS-BR-03.02` | Official distribution channels must resist adversary-in-the-middle substitution using authenticated channels. GitHub Releases, crates.io, and Homebrew are applicable. | Met | HTTPS distribution, a signed annotated tag, immutable GitHub Release, SHA-256 manifests, GitHub artifact attestations, crates.io OIDC publication, and the tap's exact-source-digest handoff bind `v0.3.0`; represented installed-channel verification passed. |
| `OSPS-BR-07.01` | The project must prevent accidental unencrypted secrets in version control. The public repository is in scope. | Met | The [security projection](../../.github/security-controls.json) requires secret scanning and push protection. The live verifier confirmed both enabled and zero open secret-scanning alerts on 2026-08-15 without disclosing values. |
| `OSPS-DO-01.01` | A released project must document all basic user functionality. `v0.3.0` is released. | Met | The [README](../../README.md) gives installation, quick-start, command, revision, transport, reporter, safety, and CI guidance; the release and installed-channel journeys prove the documented binary boundary. |
| `OSPS-DO-02.01` | A released project must explain how to report defects. `v0.3.0` is released. | Met | [Contributing guidance](../../CONTRIBUTING.md), [support guidance](../../SUPPORT.md), and the public bug/feature issue forms provide defect routes; the community verifier checked their exact public presence. |
| `OSPS-GV-02.01` | An active project must provide public discussion for changes and usage obstacles. The project is active. | Met | Public [GitHub Issues](https://github.com/EnjoyableWork/mcp-doctor/issues), structured issue forms, and the contribution guide provide the recorded discussion and defect route. |
| `OSPS-GV-03.01` | An active project must explain contribution. The project is active. | Met | [`CONTRIBUTING.md`](../../CONTRIBUTING.md), the pull-request template, and the code of conduct define the process and were revalidated by the community verifier. |
| `OSPS-LE-02.01` | Active source must use an OSI- or FSF-conformant license. The source is in scope. | Met | `Cargo.toml` declares MIT and the exact [MIT license](../../LICENSE) is OSI-approved; the source license hash is fixed in the community projection. |
| `OSPS-LE-02.02` | Released software assets must use an OSI- or FSF-conformant license. Canonical `v0.3.0` software assets are in scope. | Met | The `.crate` and both GNU/Linux archives contain the project MIT license; the formula declares MIT. The SPDX documents use CC0-1.0 as document data license and do not silently assert a project package conclusion. |
| `OSPS-LE-03.01` | The active source license must be in the repository's standard license location. | Met | The exact MIT text is in root [`LICENSE`](../../LICENSE), with its digest bound by the community projection and verifier. |
| `OSPS-LE-03.02` | Released software assets must include or accompany the applicable license. | Met | The canonical `.crate` and both archives contain `LICENSE`; formula and SPDX treatment is recorded per asset in the community projection and rechecked against immutable bytes. |
| `OSPS-QA-01.01` | Active source must be publicly readable at a static URL. | Met | [`https://github.com/EnjoyableWork/mcp-doctor`](https://github.com/EnjoyableWork/mcp-doctor) is public, unarchived, and was read without credentials during the 2026-08-15 evidence run. |
| `OSPS-QA-01.02` | Version control history must publicly record each change, actor, and time. | Met | The public [commit history](https://github.com/EnjoyableWork/mcp-doctor/commits/main/) exposes commit identifiers, attributed authors, timestamps, and reviewed pull-request links; protected merges retain that history. |
| `OSPS-QA-02.01` | Where supported, the repository must list direct language dependencies. Cargo supports this control. | Met | [`Cargo.toml`](../../Cargo.toml) lists every direct runtime and development crate at an exact version with explicit features; committed [`Cargo.lock`](../../Cargo.lock) fixes the resolved application graph, and policy tests reject drift. |
| `OSPS-QA-04.01` | A multi-repository project must list its codebases. `mcp-doctor` uses a source repository and a supporting tap repository. | Met | The public [project scope](../project-scope.md) and community projection classify `mcp-doctor`, the in-scope `homebrew-tap` distribution surface, and separate organization products without conflating their claims. |
| `OSPS-QA-05.01` | Active version control must not contain generated executable artifacts. The complete tracked tree is applicable. | Met | `scripts/verify-source-artifacts.sh` accepted only reviewable source and no generated executable; its disposable negative rehearsal rejects executable headers and executable-mode files outside reviewed scripts. |
| `OSPS-QA-05.02` | Active version control must not contain unreviewable binary artifacts. The complete tracked tree is applicable. | Met | The same source gate bounds files and rejects NUL-bearing, invalid UTF-8, disguised executable, and unreviewable binary content; exact-head and exact-`main` CI exercise it. |
| `OSPS-VM-02.01` | Active project documentation must name security contacts. The project is active. | Met | [`SECURITY.md`](../../SECURITY.md) names the private vulnerability-reporting route, supported line, response and disclosure expectations; the live verifier confirmed private reporting enabled on 2026-08-15. |

## Scope and limitations

This result covers the public `mcp-doctor` repository, the stated
organization-access boundary, the supporting tap surfaces named above, and the
canonical immutable `v0.3.0` release where a control requires released assets.
It does not assess separate organization products, private repositories,
unlisted or future artifacts, paid GitHub features, Apple or Windows signing,
the security of every dependency, a general MCP security scanner, or any OSPS
level above Level 1.

## Maintenance and removal

Revalidate at least annually, with the next scheduled review due by
2027-08-15, and immediately after any framework-version or requirement change;
BadgeApp version, issuer, route, or record change; repository-security,
ruleset, organization-access, recovery, workflow, Action, dependency, release
pipeline, public repository, channel, license, asset, digest, attestation, or
evidence change. A missing, stale, withdrawn, broken, or over-broad result must
be corrected or removed from the README immediately; a green rerun cannot
override contrary evidence.
