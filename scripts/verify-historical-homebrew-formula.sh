#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 8 ]]; then
  printf 'usage: %s repository commit formula-path expected-formula expected-formula-sha256 source-url package-sha256 api-version\n' \
    "$0" >&2
  exit 2
fi

historical_repository=$1
historical_tap_commit=$2
historical_formula_path=$3
historical_expected_formula=$4
historical_expected_formula_sha=$5
historical_source_url=$6
historical_package_sha=$7
historical_api_version=$8

if [[ ! "$historical_repository" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]] ||
  [[ ! "$historical_tap_commit" =~ ^[0-9a-f]{40}$ ]] ||
  [[ "$historical_formula_path" != Formula/mcp-doctor.rb ]] ||
  [[ ! "$historical_expected_formula_sha" =~ ^[0-9a-f]{64}$ ]] ||
  [[ ! "$historical_package_sha" =~ ^[0-9a-f]{64}$ ]] ||
  [[ ! "$historical_source_url" =~ ^https://github\.com/[^[:space:]]+$ ]] ||
  [[ ! "$historical_api_version" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] ||
  [[ ! -f "$historical_expected_formula" ]] ||
  [[ -L "$historical_expected_formula" ]]; then
  printf 'historical Homebrew verification input is invalid\n' >&2
  exit 2
fi

for historical_command in awk base64 cmp gh grep jq mktemp rm tr wc; do
  if ! command -v "$historical_command" >/dev/null 2>&1; then
    printf 'required historical Homebrew verifier command is unavailable\n' >&2
    exit 2
  fi
done

historical_hash() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print $1 }'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{ print $1 }'
  else
    printf 'a SHA-256 implementation is required\n' >&2
    return 2
  fi
}

umask 077
historical_temp_parent=${TMPDIR:-/tmp}
historical_temp_root="$(
  mktemp -d "${historical_temp_parent%/}/mcp-doctor-historical-homebrew.XXXXXX"
)"
historical_temp_prefix="${historical_temp_parent%/}/mcp-doctor-historical-homebrew."

historical_cleanup() {
  if [[ "$historical_temp_root" != "$historical_temp_prefix"* ]]; then
    printf 'refusing to remove unexpected historical Homebrew verifier path\n' >&2
    return 1
  fi
  if [[ -d "$historical_temp_root" ]]; then
    rm -rf -- "$historical_temp_root"
  fi
}
trap historical_cleanup EXIT

historical_formula_json="$historical_temp_root/formula.json"
historical_formula="$historical_temp_root/mcp-doctor.rb"
historical_endpoint="repos/$historical_repository/contents/$historical_formula_path?ref=$historical_tap_commit"

if ! GH_PROMPT_DISABLED=1 GH_PAGER=cat gh api \
  -H "X-GitHub-Api-Version: $historical_api_version" \
  "$historical_endpoint" >"$historical_formula_json" 2>/dev/null ||
  ! jq -er 'select(.type == "file" and .encoding == "base64") | .content' \
    "$historical_formula_json" | tr -d '\n' | \
    base64 --decode >"$historical_formula" 2>/dev/null ||
  [[ "$(wc -c <"$historical_formula" | tr -d '[:space:]')" -gt 65536 ]] ||
  ! cmp -s "$historical_expected_formula" "$historical_formula" ||
  [[ "$(historical_hash "$historical_formula")" != \
    "$historical_expected_formula_sha" ]] ||
  ! grep -F "url \"$historical_source_url\"" "$historical_formula" >/dev/null ||
  ! grep -F "sha256 \"$historical_package_sha\"" "$historical_formula" >/dev/null ||
  ! grep -F 'license "MIT"' "$historical_formula" >/dev/null; then
  printf 'historical Homebrew formula verification failed\n' >&2
  exit 1
fi
