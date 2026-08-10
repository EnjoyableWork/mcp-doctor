# Contributing to mcp-doctor

Thank you for helping make MCP server diagnostics safer and more useful.

## Before opening a change

- Search existing issues and the ticket board in [PROJECT.md](PROJECT.md).
- Use a public issue for ordinary bugs and feature proposals.
- Use the private route in [SECURITY.md](SECURITY.md) for suspected
  vulnerabilities, credentials, or diagnostics that could expose private
  systems.
- Never attach secrets, real authentication headers, private endpoints, user
  configuration, database content, or unredacted server output.

For a material change, identify the governing `MCPD-NNN` ticket or propose one
with a focused outcome, dependencies, acceptance evidence, and safety impact.

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
  review in [AGENTS.md](AGENTS.md) and record the decision in
  [PROJECT.md](PROJECT.md).
- Use synthetic fixtures. Default tests must not call a real MCP server or
  production endpoint.
- Keep errors and assertions structural so failures cannot print untrusted
  payloads or secrets.
- Update the README for changed destination behavior and `PROJECT.md` for
  current status, decisions, risks, or evidence.
- Follow Conventional Commits: `<type>[optional scope]: <imperative summary>`.

## Pull requests

Describe the user-visible outcome, execution and data-safety impact, exact
verification performed, protocol revisions affected, and any unverified native
or release gate. Resolve review conversations and keep the branch current with
the protected default branch before merge.

Do not commit generated binaries, archives, packages, credentials, or copied
production evidence. Release artifacts are generated and attested outside the
source tree.

## Licensing

Contributions are accepted under the repository's [MIT License](LICENSE), with
the same inbound and outbound terms. The project currently requires neither a
Contributor License Agreement nor a `Signed-off-by` line. A contributor may
still use a Developer Certificate of Origin sign-off voluntarily.

By submitting a contribution, you confirm that you have the right to license
it under these terms and agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
