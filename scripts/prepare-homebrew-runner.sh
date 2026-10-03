#!/usr/bin/env bash

set -euo pipefail

# Only disposable hosted macOS runners may change their preinstalled links.
if [[ $# -ne 0 || "${GITHUB_ACTIONS:-}" != true ||
  "${RUNNER_ENVIRONMENT:-}" != github-hosted || "${RUNNER_OS:-}" != macOS ]]; then
  printf 'Homebrew preparation requires a GitHub-hosted macOS runner\n' >&2
  exit 2
fi
if ! command -v brew >/dev/null 2>&1; then
  printf 'required action-provided command is unavailable: brew\n' >&2
  exit 2
fi

export HOMEBREW_NO_AUTO_UPDATE=1
homebrew_installed_formulas="$(brew list --formula -1)"
while IFS= read -r homebrew_formula; do
  if [[ "$homebrew_formula" == openssl@1.1 ]]; then
    # This old runner keg otherwise blocks openssl@3's dependency link step.
    brew unlink openssl@1.1
    break
  fi
done <<<"$homebrew_installed_formulas"
