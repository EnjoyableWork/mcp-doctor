#!/usr/bin/env bash

set -euo pipefail

# Only disposable hosted macOS runners may change their preinstalled links.
if [[ $# -ne 0 || "${GITHUB_ACTIONS:-}" != true ||
  "${RUNNER_ENVIRONMENT:-}" != github-hosted || "${RUNNER_OS:-}" != macOS ]]; then
  printf 'Homebrew preparation requires a GitHub-hosted macOS runner\n' >&2
  exit 2
fi
for homebrew_command in brew readlink rm; do
  if ! command -v "$homebrew_command" >/dev/null 2>&1; then
    printf 'required Homebrew preparation command is unavailable: %s\n' "$homebrew_command" >&2
    exit 2
  fi
done

export HOMEBREW_NO_AUTO_UPDATE=1
export LC_ALL=C
# Preserve path bytes, stripping only the command's own newline and sentinel.
homebrew_prefix="$(brew --prefix && printf '.')"
homebrew_prefix="${homebrew_prefix%$'\n.'}"
if [[ ${#homebrew_prefix} -gt 4096 || "$homebrew_prefix" != /* ||
  "$homebrew_prefix" == / || "$homebrew_prefix" == */ ||
  "$homebrew_prefix" == *//* || "$homebrew_prefix" == */./* ||
  "$homebrew_prefix" == */../* || "$homebrew_prefix" == */. ||
  "$homebrew_prefix" == */.. || "$homebrew_prefix" == *[![:print:]]* ||
  ! -d "$homebrew_prefix" ]]; then
  printf 'Homebrew prefix must be a bounded normalized absolute directory\n' >&2
  exit 2
fi

homebrew_installed_formulas="$(brew list --formula -1)"
while IFS= read -r homebrew_formula; do
  if [[ "$homebrew_formula" == openssl@1.1 ]]; then
    # This old runner keg otherwise blocks openssl@3's dependency link step.
    brew unlink openssl@1.1
    break
  fi
done <<<"$homebrew_installed_formulas"

homebrew_openssl="${homebrew_prefix}/bin/openssl"
homebrew_legacy_target="${homebrew_prefix}/opt/openssl@1.1/bin/openssl"
if [[ -L "$homebrew_openssl" ]]; then
  # Preserve target newlines so only the byte-identical legacy target matches.
  homebrew_openssl_target="$(readlink -n "$homebrew_openssl" && printf '.')"
  homebrew_openssl_target="${homebrew_openssl_target%.}"
  if [[ "$homebrew_openssl_target" == "$homebrew_legacy_target" ]]; then
    # Runner-created opt links can survive keg unlink, even with no keg left.
    rm -- "$homebrew_openssl"
    if [[ -e "$homebrew_openssl" || -L "$homebrew_openssl" ]]; then
      printf 'legacy Homebrew executable link remains after removal\n' >&2
      exit 1
    fi
  fi
fi
