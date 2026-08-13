#!/usr/bin/env bash
set -euo pipefail

script_dir="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
repository_root="$(dirname -- "$script_dir")"
canonical_path="${1:-$repository_root/.github/security-controls.json}"

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
  .schema_version == "mcp-doctor.github-security-controls/v1" and
  .api_version == "2026-03-10" and
  .repository == "EnjoyableWork/mcp-doctor" and
  .repository_visibility == "public" and
  .organization_plan == "free" and
  .default_branch == "main" and
  .security_policy.path == "SECURITY.md" and
  .security_policy.private_reporting_url ==
    "https://github.com/EnjoyableWork/mcp-doctor/security/advisories/new" and
  .security_policy.supported_release_lines == ["0.3.x"] and
  .security_policy.unsupported_release_lines == ["0.2.x and earlier"] and
  .controls.vulnerability_alerts == true and
  .controls.automated_security_fixes == true and
  .controls.dependabot_security_updates == "enabled" and
  .controls.private_vulnerability_reporting == true and
  .controls.code_scanning_default_setup == {
    "state": "configured",
    "languages": ["actions", "rust"],
    "query_suite": "default",
    "runner_type": "standard",
    "schedule": "weekly",
    "threat_model": "remote"
  } and
  .controls.secret_scanning == "enabled" and
  .controls.secret_scanning_push_protection == "enabled" and
  .controls.secret_scanning_non_provider_patterns == "disabled" and
  .controls.secret_scanning_validity_checks == "disabled" and
  .clean_baseline == {
    "require_readable_dependency_graph": true,
    "require_successful_default_branch_codeql_analyses": ["actions", "rust"],
    "require_repository_visible_secret_alert_endpoint": true,
    "require_zero_open_dependabot_alerts": true,
    "require_zero_open_code_scanning_alerts": true,
    "require_zero_open_secret_scanning_alerts": true
  } and
  (.unavailable_features | map(.feature)) == [
    "secret_scanning_validity_checks",
    "secret_scanning_scan_history_readback",
    "secret_scanning_non_provider_and_generic_patterns",
    "ai_generic_secret_detection",
    "delegated_push_protection_bypass",
    "enterprise_public_leak_monitoring"
  ] and
  (.excluded_evidence | map(.surface)) == [
    "partner_only_public_repository_secret_alerts",
    "mcp_doctor_product_security_scanner",
    "complete_m4_assurance_baseline"
  ]
' "$canonical_path" >/dev/null 2>&1; then
  printf 'canonical security-control configuration is invalid\n' >&2
  exit 2
fi

repository="$(jq -er '.repository' "$canonical_path")"
organization="${repository%%/*}"
api_version="$(jq -er '.api_version' "$canonical_path")"
default_branch="$(jq -er '.default_branch' "$canonical_path")"
config_hash="$(canonical_hash)"
verification_date="$(date -u +%F)"

report_failure() {
  printf 'date=%s canonical_sha256=%s result=FAIL\n' \
    "$verification_date" \
    "$config_hash"
  exit 1
}

umask 077
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/mcp-doctor-security-controls.XXXXXX")"
cleanup() {
  rm -rf -- "$work_dir"
}
trap cleanup EXIT

repository_path="$work_dir/repository.json"
organization_path="$work_dir/organization.json"
default_branch_path="$work_dir/default-branch.json"
policy_path="$work_dir/policy.json"
private_reporting_path="$work_dir/private-reporting.json"
code_setup_path="$work_dir/code-setup.json"
code_analyses_path="$work_dir/code-analyses.json"
dependabot_alerts_path="$work_dir/dependabot-alerts.json"
code_alerts_path="$work_dir/code-alerts.json"
secret_alerts_path="$work_dir/secret-alerts.json"
sbom_path="$work_dir/sbom.json"

api_get() {
  local endpoint="$1"
  local destination="$2"

  GH_PROMPT_DISABLED=1 GH_PAGER=cat gh api \
    -H "X-GitHub-Api-Version: $api_version" \
    "$endpoint" >"$destination" 2>/dev/null
}

if ! api_get "repos/$repository" "$repository_path" ||
  ! api_get "orgs/$organization" "$organization_path" ||
  ! api_get "repos/$repository/commits/$default_branch" "$default_branch_path" ||
  ! api_get "repos/$repository/contents/SECURITY.md?ref=$default_branch" "$policy_path" ||
  ! api_get "repos/$repository/private-vulnerability-reporting" "$private_reporting_path" ||
  ! api_get "repos/$repository/code-scanning/default-setup" "$code_setup_path" ||
  ! api_get "repos/$repository/code-scanning/analyses?ref=refs/heads/$default_branch&tool_name=CodeQL&per_page=100" "$code_analyses_path" ||
  ! api_get "repos/$repository/dependabot/alerts?state=open&per_page=1" "$dependabot_alerts_path" ||
  ! api_get "repos/$repository/code-scanning/alerts?state=open&per_page=1" "$code_alerts_path" ||
  ! api_get "repos/$repository/secret-scanning/alerts?state=open&hide_secret=true&per_page=1" "$secret_alerts_path" ||
  ! api_get "repos/$repository/dependency-graph/sbom" "$sbom_path" ||
  ! GH_PROMPT_DISABLED=1 GH_PAGER=cat gh api \
    -H "X-GitHub-Api-Version: $api_version" \
    "repos/$repository/vulnerability-alerts" >/dev/null 2>&1 ||
  ! GH_PROMPT_DISABLED=1 GH_PAGER=cat gh api \
    -H "X-GitHub-Api-Version: $api_version" \
    "repos/$repository/automated-security-fixes" >/dev/null 2>&1; then
  report_failure
fi

default_branch_sha="$(jq -er '.sha' "$default_branch_path" 2>/dev/null)" || report_failure
required_languages="$(
  jq -c '.clean_baseline.require_successful_default_branch_codeql_analyses' \
    "$canonical_path" 2>/dev/null
)" || report_failure

if ! jq -e --arg repository "$repository" --arg branch "$default_branch" '
    .full_name == $repository and
    .visibility == "public" and
    .default_branch == $branch and
    .security_and_analysis.dependabot_security_updates.status == "enabled" and
    .security_and_analysis.secret_scanning.status == "enabled" and
    .security_and_analysis.secret_scanning_push_protection.status == "enabled" and
    .security_and_analysis.secret_scanning_non_provider_patterns.status == "disabled" and
    .security_and_analysis.secret_scanning_validity_checks.status == "disabled"
  ' "$repository_path" >/dev/null 2>&1 ||
  ! jq -e '.plan.name == "free"' "$organization_path" >/dev/null 2>&1 ||
  ! jq -e '.path == "SECURITY.md" and .type == "file" and .size > 0' \
    "$policy_path" >/dev/null 2>&1 ||
  ! jq -e '.enabled == true' "$private_reporting_path" >/dev/null 2>&1 ||
  ! jq -e '
    .state == "configured" and
    (.languages | sort) == ["actions", "rust"] and
    .query_suite == "default" and
    .runner_type == "standard" and
    .schedule == "weekly" and
    .threat_model == "remote"
  ' "$code_setup_path" >/dev/null 2>&1 ||
  ! jq -e --arg sha "$default_branch_sha" --argjson required "$required_languages" '
    . as $analyses |
    all($required[];
      . as $language |
      any($analyses[];
        .category == ("/language:" + $language) and
        .ref == "refs/heads/main" and
        .commit_sha == $sha and
        (.error // "") == ""
      )
    )
  ' "$code_analyses_path" >/dev/null 2>&1 ||
  ! jq -e 'length == 0' "$dependabot_alerts_path" >/dev/null 2>&1 ||
  ! jq -e 'length == 0' "$code_alerts_path" >/dev/null 2>&1 ||
  ! jq -e 'length == 0' "$secret_alerts_path" >/dev/null 2>&1 ||
  ! jq -e '
    .sbom.spdxVersion == "SPDX-2.3" and
    (.sbom.documentNamespace | type == "string" and length > 0) and
    (.sbom.packages | type == "array" and length > 0)
  ' "$sbom_path" >/dev/null 2>&1; then
  report_failure
fi

printf 'date=%s canonical_sha256=%s result=PASS\n' \
  "$verification_date" \
  "$config_hash"
