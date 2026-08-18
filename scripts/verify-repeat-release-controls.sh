#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <exact mcp-doctor main commit> <exact homebrew-tap main commit>" >&2
  exit 2
fi

repeat_release_source_commit=$1
repeat_release_tap_commit=$2
repeat_release_api_version=2026-03-10
repeat_release_organization=EnjoyableWork
repeat_release_source_repository=EnjoyableWork/mcp-doctor
repeat_release_tap_repository=EnjoyableWork/homebrew-tap

for repeat_release_commit in \
  "${repeat_release_source_commit}" \
  "${repeat_release_tap_commit}"; do
  if [[ ! "${repeat_release_commit}" =~ ^[0-9a-f]{40}$ ]]; then
    echo "repeat-release control commits must be full lowercase Git SHAs" >&2
    exit 2
  fi
done
if ! gh auth status --hostname github.com >/dev/null 2>&1; then
  echo "GitHub CLI must be authenticated as a repository administrator" >&2
  exit 1
fi

repeat_release_api() {
  gh api "$1" -H "X-GitHub-Api-Version: ${repeat_release_api_version}"
}

verify_repeat_release_repository() {
  local repository=$1
  local expected_commit=$2
  local repository_state current_main

  repository_state=$(repeat_release_api "repos/${repository}")
  if [[ "$(jq -r '.visibility' <<<"${repository_state}")" != public ]]; then
    echo "repeat-release repositories must remain public" >&2
    exit 1
  fi
  current_main=$(
    repeat_release_api "repos/${repository}/commits/main" | jq -r '.sha'
  )
  if [[ "${current_main}" != "${expected_commit}" ]]; then
    echo "repeat-release control evidence is not for exact current main" >&2
    exit 1
  fi
}

verify_repeat_release_repository \
  "${repeat_release_source_repository}" \
  "${repeat_release_source_commit}"
verify_repeat_release_repository \
  "${repeat_release_tap_repository}" \
  "${repeat_release_tap_commit}"

if [[ "$(
  repeat_release_api "repos/${repeat_release_source_repository}/immutable-releases" \
    | jq -r '.enabled'
)" != true ]]; then
  echo "mcp-doctor release immutability must remain enabled" >&2
  exit 1
fi

verify_repeat_release_environment() {
  local repository=$1
  local expected_policies=$2
  local environment_state environment_policies repeat_release_workflow_policy

  environment_state=$(
    repeat_release_api "repos/${repository}/environments/release"
  )
  jq -e '
    .name == "release" and
    .deployment_branch_policy.protected_branches == false and
    .deployment_branch_policy.custom_branch_policies == true and
    any(.protection_rules[];
      .type == "required_reviewers" and
      .prevent_self_review == false and
      (.reviewers | length) >= 1
    ) and
    any(.protection_rules[]; .type == "branch_policy")
  ' <<<"${environment_state}" >/dev/null

  environment_policies=$(
    repeat_release_api \
      "repos/${repository}/environments/release/deployment-branch-policies"
  )
  jq -e --argjson expected "${expected_policies}" '
    (.branch_policies | map({name, type}) | sort_by(.type, .name)) ==
    ($expected | sort_by(.type, .name))
  ' <<<"${environment_policies}" >/dev/null

  if [[ "$(
    repeat_release_api "repos/${repository}/actions/secrets" | jq -r '.total_count'
  )" != 0 ]] || [[ "$(
    repeat_release_api "repos/${repository}/environments/release/secrets" \
      | jq -r '.total_count'
  )" != 0 ]]; then
    echo "repeat-release repositories and environments must not store Actions secrets" >&2
    exit 1
  fi

  repeat_release_workflow_policy=$(
    repeat_release_api "repos/${repository}/actions/permissions/workflow"
  )
  jq -e '
    .default_workflow_permissions == "read" and
    .can_approve_pull_request_reviews == false
  ' <<<"${repeat_release_workflow_policy}" >/dev/null
}

verify_repeat_release_environment \
  "${repeat_release_source_repository}" \
  '[{"name":"main","type":"branch"},{"name":"v*.*.*","type":"tag"}]'
verify_repeat_release_environment \
  "${repeat_release_tap_repository}" \
  '[{"name":"main","type":"branch"}]'

repeat_release_source_id=$(
  repeat_release_api "repos/${repeat_release_source_repository}" | jq -r '.id'
)
repeat_release_tap_id=$(
  repeat_release_api "repos/${repeat_release_tap_repository}" | jq -r '.id'
)
if ! repeat_release_organization_secrets=$(
  repeat_release_api \
    "orgs/${repeat_release_organization}/actions/secrets?per_page=100" \
    2>/dev/null
); then
  echo "organization Actions secret inventory could not be verified" >&2
  exit 1
fi
if ! jq -e '
  (.secrets | type) == "array" and
  all(.secrets[];
    (.name | type) == "string" and
    (.visibility | type) == "string"
  )
' <<<"${repeat_release_organization_secrets}" >/dev/null 2>&1; then
  echo "organization Actions secret inventory could not be verified" >&2
  exit 1
fi
repeat_release_organization_secret_rows=$(
  jq -r '.secrets[] | [.name, .visibility] | @tsv' \
    <<<"${repeat_release_organization_secrets}"
) || {
  echo "organization Actions secret inventory could not be verified" >&2
  exit 1
}
while IFS=$'\t' read -r repeat_release_secret_name repeat_release_secret_visibility; do
  [[ -n "${repeat_release_secret_name}" ]] || continue
  case "${repeat_release_secret_visibility}" in
    all)
      echo "an organization Actions secret is available to the release repositories" >&2
      exit 1
      ;;
    selected)
      if ! repeat_release_selected_repositories=$(
        repeat_release_api \
          "orgs/${repeat_release_organization}/actions/secrets/${repeat_release_secret_name}/repositories" \
          2>/dev/null
      ) || ! jq -e '
        (.repositories | type) == "array" and
        all(.repositories[]; (.id | type) == "number")
      ' <<<"${repeat_release_selected_repositories}" >/dev/null 2>&1; then
        echo "organization Actions secret selection could not be verified" >&2
        exit 1
      fi
      repeat_release_selected_ids=$(
        jq -r '.repositories[].id' \
          <<<"${repeat_release_selected_repositories}"
      ) || {
        echo "organization Actions secret selection could not be verified" >&2
        exit 1
      }
      if grep -F -x -e "${repeat_release_source_id}" -e "${repeat_release_tap_id}" \
        <<<"${repeat_release_selected_ids}" >/dev/null; then
        echo "an organization Actions secret is selected for a release repository" >&2
        exit 1
      fi
      ;;
    private) ;;
    *)
      echo "organization Actions secret visibility is not recognized" >&2
      exit 1
      ;;
  esac
done <<<"${repeat_release_organization_secret_rows}"

if [[ -n "${CARGO_REGISTRY_TOKEN:-}" ]] ||
  [[ -n "${CARGO_REGISTRIES_CRATES_IO_TOKEN:-}" ]]; then
  echo "a crates.io token is present in the operator environment" >&2
  exit 1
fi

repeat_release_cargo_home=${CARGO_HOME:-${HOME}/.cargo}
for repeat_release_credential_file in \
  "${repeat_release_cargo_home}/credentials.toml" \
  "${repeat_release_cargo_home}/credentials"; do
  [[ -f "${repeat_release_credential_file}" ]] || continue
  if awk '
    /^\[[^]]+\][[:space:]]*$/ { section = $0 }
    section == "[registry]" && /^[[:space:]]*token[[:space:]]*=/ { found = 1 }
    section == "[registries.crates-io]" && /^[[:space:]]*token[[:space:]]*=/ { found = 1 }
    END { exit(found ? 0 : 1) }
  ' "${repeat_release_credential_file}"; then
    echo "a crates.io token remains in the operator Cargo credential store" >&2
    exit 1
  fi
done

repeat_release_source_workflow=$(
  repeat_release_api \
    "repos/${repeat_release_source_repository}/contents/.github/workflows/release.yml?ref=${repeat_release_source_commit}" \
    | jq -r '.content' \
    | base64 --decode
)
repeat_release_tap_workflow=$(
  repeat_release_api \
    "repos/${repeat_release_tap_repository}/contents/.github/workflows/publish-mcp-doctor.yml?ref=${repeat_release_tap_commit}" \
    | jq -r '.content' \
    | base64 --decode
)
for repeat_release_workflow in \
  "${repeat_release_source_workflow}" \
  "${repeat_release_tap_workflow}"; do
  if grep -E -q 'secrets\.|vars\.|PERSONAL_ACCESS_TOKEN|(^|[^A-Z])PAT([^A-Z]|$)' \
    <<<"${repeat_release_workflow}"; then
    echo "a repeat-release workflow references stored or personal credentials" >&2
    exit 1
  fi
done

if [[ "${repeat_release_source_workflow}" != *'rust-lang/crates-io-auth-action@c6f97d42243bad5fab37ca0427f495c86d5b1a18'* ]] ||
  [[ "${repeat_release_tap_workflow}" != *'permissions:'* ]] ||
  [[ "${repeat_release_tap_workflow}" != *'contents: write'* ]]; then
  echo "repeat-release workflows do not match the reviewed authority contract" >&2
  exit 1
fi

printf 'Verified clean repeat-release controls for exact mcp-doctor and tap main commits.\n'
