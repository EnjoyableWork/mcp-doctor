#!/usr/bin/env bash
set -euo pipefail

script_dir="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
repository_root="$(dirname -- "$script_dir")"
canonical_path="${1:-$repository_root/.github/rulesets/main.json}"

if [[ $# -gt 1 ]]; then
  printf 'usage: %s [canonical-config]\n' "$0" >&2
  exit 2
fi

for required_command in gh jq; do
  if ! command -v "$required_command" >/dev/null 2>&1; then
    printf 'required command is unavailable\n' >&2
    exit 2
  fi
done

canonical_hash() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$canonical_path" | awk '{ print $1 }'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$canonical_path" | awk '{ print $1 }'
  else
    printf 'a SHA-256 implementation is required\n' >&2
    return 2
  fi
}

if ! jq -e '
  .schema_version == "mcp-doctor.github-main-protection/v1" and
  .repository == "EnjoyableWork/mcp-doctor" and
  .ruleset.bypass_actors == []
' "$canonical_path" >/dev/null 2>&1; then
  printf 'canonical main-protection configuration is invalid\n' >&2
  exit 2
fi

repository="$(jq -er '.repository' "$canonical_path")"
api_version="$(jq -er '.api_version' "$canonical_path")"
ruleset_name="$(jq -er '.ruleset.name' "$canonical_path")"
config_hash="$(canonical_hash)"
verification_date="$(date -u +%F)"

report_failure() {
  printf 'date=%s canonical_sha256=%s result=FAIL\n' \
    "$verification_date" \
    "$config_hash"
  exit 1
}

umask 077
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/mcp-doctor-admin-protection.XXXXXX")"
cleanup() {
  rm -rf -- "$work_dir"
}
trap cleanup EXIT

rulesets_path="$work_dir/rulesets.json"
ruleset_path="$work_dir/ruleset.json"

if ! GH_PROMPT_DISABLED=1 GH_PAGER=cat gh api \
  -H "X-GitHub-Api-Version: $api_version" \
  "repos/$repository/rulesets?includes_parents=true&per_page=100" \
  >"$rulesets_path" 2>/dev/null; then
  report_failure
fi

if ! ruleset_id="$(
  jq -er --arg name "$ruleset_name" '
    if length == 1 and
      .[0].name == $name and
      .[0].source_type == "Repository"
    then .[0].id
    else error("unexpected ruleset projection")
    end
  ' "$rulesets_path" 2>/dev/null
)"; then
  report_failure
fi

if ! GH_PROMPT_DISABLED=1 GH_PAGER=cat gh api \
  -H "X-GitHub-Api-Version: $api_version" \
  "repos/$repository/rulesets/$ruleset_id?includes_parents=true" \
  >"$ruleset_path" 2>/dev/null; then
  report_failure
fi

if ! jq -e '.bypass_actors == []' "$ruleset_path" >/dev/null 2>&1; then
  report_failure
fi

printf 'date=%s canonical_sha256=%s result=PASS\n' \
  "$verification_date" \
  "$config_hash"
