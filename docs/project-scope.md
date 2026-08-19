# Project repositories, community, and license scope

Reviewed: 2026-08-12

This document identifies the public repositories and channels that make up
`mcp-doctor`. The machine-readable contract is
[`.github/community-license-controls.json`](../.github/community-license-controls.json),
and `scripts/verify-community-license.sh` checks it without credentials.

## In-scope repositories

| Repository | Project role | Status and intent |
| --- | --- | --- |
| [`EnjoyableWork/mcp-doctor`](https://github.com/EnjoyableWork/mcp-doctor) | Primary product source, community policy, public discussion, defect intake, and canonical releases | Active and in scope in full |
| [`EnjoyableWork/homebrew-tap`](https://github.com/EnjoyableWork/homebrew-tap) | Supporting Homebrew distribution codebase | Active; only the `mcp-doctor` policy links, repository license, formula, and release handoff are in scope |

The complete public EnjoyableWork inventory also contains
[`mcp-sync`](https://github.com/EnjoyableWork/mcp-sync). It is an active,
separate product rather than an `mcp-doctor` codebase or distribution channel.
Classifying it here prevents hidden repository scope; it does not assess or
extend `mcp-doctor` policies to that project. A new or unclassified public
organization repository makes the verifier fail for review.

## Community and defect routes

The source repository owns the canonical public project policies:

- [contribution process](../CONTRIBUTING.md), including the same inbound and
  outbound MIT terms;
- [code of conduct](../CODE_OF_CONDUCT.md) and GitHub's private repository
  content-report route;
- [support and defect-reporting guide](../SUPPORT.md), backed by structured
  [bug and feature forms](https://github.com/EnjoyableWork/mcp-doctor/issues/new/choose);
- public project discussion in the
  [issue tracker](https://github.com/EnjoyableWork/mcp-doctor/issues); and
- [private vulnerability reporting](https://github.com/EnjoyableWork/mcp-doctor/security/advisories/new)
  under the separate [security policy](../SECURITY.md).

The tap README delegates `mcp-doctor` requests to these same routes. This is a
real operating path, not a named but unreachable mailbox: GitHub issues are
enabled, blank issues are disabled in favor of the forms, and repository
content reporting is enabled for private conduct reports. GitHub Support is
the fallback when the repository content-report action is unavailable or the
concern is platform-wide.

## Official channels

| Channel | HTTPS location | Authority |
| --- | --- | --- |
| Source and policies | [`EnjoyableWork/mcp-doctor`](https://github.com/EnjoyableWork/mcp-doctor) | Canonical source repository |
| Public discussion and defects | [GitHub Issues](https://github.com/EnjoyableWork/mcp-doctor/issues) | Canonical public project discussion |
| Vulnerabilities | [Private vulnerability reporting](https://github.com/EnjoyableWork/mcp-doctor/security/advisories/new) | Canonical confidential security intake |
| Releases | [`v0.2.0` GitHub Release](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.2.0) | Canonical immutable release record |
| Rust package | [`mcp-doctor` `0.2.0` on crates.io](https://crates.io/crates/mcp-doctor/0.2.0) | Verified package distribution |
| Homebrew | [`EnjoyableWork/tap/mcp-doctor`](https://github.com/EnjoyableWork/homebrew-tap/blob/main/Formula/mcp-doctor.rb) | Verified formula distribution |
| Generated API documentation | [`docs.rs`](https://docs.rs/crate/mcp-doctor/latest) | Third-party documentation mirror, not a release authority |

No HTTP, alternate registry, mirror, social account, chat room, or personal
contact is an official `mcp-doctor` project channel.

## License evidence

The project source and accepted contributions use the OSI-approved MIT License.
The exact root [`LICENSE`](../LICENSE) is named by the Cargo manifest and has
SHA-256
`32a82b79c71a3a633dc51fcb306f0d4768551aaff7c8862f67a5997a5f75faea`.
The published `0.2.0` Cargo package and both GNU/Linux archives contain that
exact file. crates.io reports the package license as `MIT`. The released and
tap-owned Homebrew formula declares `license "MIT"`, and the tap repository
contains the same exact license file.

The two immutable SPDX documents use `CC0-1.0` as their document data license
and retain `NOASSERTION` for the root package's declared and concluded license.
They are therefore not used as proof of the software's MIT license. The exact
package metadata and embedded license files provide that proof. `SHA256SUMS`
is release metadata accompanying the same licensed release set.

This scoped evidence does not authenticate the supply chain by itself, complete
an OSPS assessment, certify the project, or change immutable release bytes.
The machine-readable
[supply-chain controls](../.github/supply-chain-controls.json) and focused
[assurance records](assurance/) separately authenticate their bounded scopes.
No complete assurance claim follows without current dated evidence for every
applicable control.
