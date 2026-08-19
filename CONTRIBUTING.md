# Contributing to mcp-doctor

Thank you for helping make MCP server diagnostics safer and more useful.

## Before opening a change

- Search existing GitHub issues and the
  [mcp-doctor Linear project](https://linear.app/enjoyablework/project/mcp-doctor-category-leadership-ff35a964e5d3).
- Use a public issue for ordinary bugs and feature proposals.
- Use the private route in [SECURITY.md](SECURITY.md) for suspected
  vulnerabilities, credentials, or diagnostics that could expose private
  systems.
- Never attach secrets, real authentication headers, private endpoints, user
  configuration, database content, or unredacted server output.

For a material change, identify the governing Linear issue or propose one with
a focused outcome, dependencies, safety impact, acceptance evidence, and a
deterministic definition of done. Linear is the sole authority for delivery
priority, sequencing, ownership, status, and completion; do not add a roadmap,
ticket board, decision log, or progress mirror to the repository.

## Development setup

Install a current stable Rust toolchain. The repository toolchain file selects
the required formatter and linter components.

On macOS or GNU/Linux:

```bash
./scripts/check.sh
```

On Windows:

```powershell
./scripts/check.ps1
```

The scripts run formatting, warning-free Clippy, and the complete test suite
through a disposable user environment. When `cargo-deny` is installed, also
run:

```bash
cargo deny --all-features --locked check
```

## Change expectations

- Keep one observable outcome per pull request.
- Preserve passive-by-default behavior and every existing execution limit.
- Add the narrowest useful regression test for changed behavior.
- Add a dependency only when the governing ticket demonstrates a concrete need
  that the standard library and existing graph cannot meet. Follow the
  maintenance, provenance, exact-version, feature, transitive-graph, and update
  review in [AGENTS.md](AGENTS.md), and record the decision and evidence in the
  governing Linear issue and pull request.
- Treat every Dependabot pull request as a proposal. Do not enable auto-merge.
  For each Rust dependency, standalone CI tool, or GitHub Action change, record
  the old and new exact identities plus release notes, upstream maintenance and
  ownership/provenance, selected features and graph changes, licenses and
  advisories, unsafe or build-script changes, Rust/platform impact, and focused
  behavior evidence in the pull request. A grouped proposal may be split or
  rejected when one member obscures causality or fails review.
- Use synthetic fixtures. Default tests must not call a real MCP server or
  production endpoint.
- Keep errors and assertions structural so failures cannot print untrusted
  payloads or secrets.
- Update the focused repository contract for changed product behavior or
  public claims, and update the Linear issue for delivery status, decisions,
  risks, and evidence links.
- Follow Conventional Commits: `<type>[optional scope]: <imperative summary>`.

## Pull requests

Describe the user-visible outcome, execution and data-safety impact, exact
verification performed, protocol revisions affected, and any unverified native
or release gate. Resolve review conversations and keep the branch current with
the protected default branch before merge.

Do not commit generated executables, binary libraries, archives, packages,
credentials, or copied production evidence. Reviewable source scripts remain
source, even when their executable bit is required. Release and testing-tool
artifacts are generated or fetched with an exact reviewed digest outside the
source tree.

## Licensing

Contributions are accepted under the repository's [MIT License](LICENSE), with
the same inbound and outbound terms. The project currently requires neither a
Contributor License Agreement nor a `Signed-off-by` line. A contributor may
still use a Developer Certificate of Origin sign-off voluntarily.

By submitting a contribution, you confirm that you have the right to license
it under these terms and agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
