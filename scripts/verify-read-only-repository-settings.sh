#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 2 ]]; then
  printf 'usage: %s owner/repository api-version\n' "$0" >&2
  exit 2
fi

readonly_repository=$1
readonly_api_version=$2

if [[ ! "$readonly_repository" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]] ||
  [[ ! "$readonly_api_version" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
  printf 'read-only repository verification input is invalid\n' >&2
  exit 2
fi

for readonly_command in gh jq mktemp rm tr wc; do
  if ! command -v "$readonly_command" >/dev/null 2>&1; then
    printf 'required read-only repository verifier command is unavailable\n' >&2
    exit 2
  fi
done

readonly_owner=${readonly_repository%%/*}
readonly_name=${readonly_repository#*/}
readonly_query="query(\$owner: String!, \$name: String!) {
  repository(owner: \$owner, name: \$name) {
    nameWithOwner
    autoMergeAllowed
  }
}"

umask 077
readonly_temp_parent=${TMPDIR:-/tmp}
readonly_temp_root="$(
  mktemp -d "${readonly_temp_parent%/}/mcp-doctor-read-only-repository.XXXXXX"
)"
readonly_temp_prefix="${readonly_temp_parent%/}/mcp-doctor-read-only-repository."

readonly_cleanup() {
  if [[ "$readonly_temp_root" != "$readonly_temp_prefix"* ]]; then
    printf 'refusing to remove unexpected read-only repository verifier path\n' >&2
    return 1
  fi
  if [[ -d "$readonly_temp_root" ]]; then
    rm -rf -- "$readonly_temp_root"
  fi
}
trap readonly_cleanup EXIT

readonly_response="$readonly_temp_root/repository.json"
if ! GH_PROMPT_DISABLED=1 GH_PAGER=cat gh api graphql \
  -H "X-GitHub-Api-Version: $readonly_api_version" \
  -f "query=$readonly_query" \
  -F "owner=$readonly_owner" \
  -F "name=$readonly_name" >"$readonly_response" 2>/dev/null ||
  [[ "$(wc -c <"$readonly_response" | tr -d '[:space:]')" -gt 65536 ]] ||
  ! jq -e --arg repository "$readonly_repository" '
    ((.errors // []) | length == 0) and
    .data.repository.nameWithOwner == $repository and
    .data.repository.autoMergeAllowed == false
  ' "$readonly_response" >/dev/null 2>&1; then
  printf 'read-only repository settings verification failed\n' >&2
  exit 1
fi
