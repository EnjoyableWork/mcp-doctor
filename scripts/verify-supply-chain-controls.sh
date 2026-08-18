#!/usr/bin/env bash

set -euo pipefail

supply_script_directory="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
supply_repository_root="$(dirname -- "$supply_script_directory")"
supply_canonical="$supply_repository_root/.github/supply-chain-controls.json"
supply_community="$supply_repository_root/.github/community-license-controls.json"
supply_source_ref=main

if [[ $# -eq 2 ]] && [[ "$1" == --source-ref ]]; then
  supply_source_ref=$2
elif [[ $# -ne 0 ]]; then
  printf 'usage: %s [--source-ref main|40-hex-commit]\n' "$0" >&2
  exit 2
fi
if [[ "$supply_source_ref" != main ]] &&
  [[ ! "$supply_source_ref" =~ ^[0-9a-f]{40}$ ]]; then
  printf 'source ref must be main or a full lowercase Git commit SHA\n' >&2
  exit 2
fi

for supply_command in base64 cmp curl gh git jq tar; do
  if ! command -v "$supply_command" >/dev/null 2>&1; then
    printf 'required supply-chain verifier command is unavailable\n' >&2
    exit 2
  fi
done
if ! gh auth status --hostname github.com >/dev/null 2>&1; then
  printf 'GitHub CLI must be authenticated as a repository administrator\n' >&2
  exit 2
fi

supply_hash() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print $1 }'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{ print $1 }'
  else
    printf 'a SHA-256 implementation is required\n' >&2
    return 2
  fi
}

if ! jq -e '
  .schema_version == "mcp-doctor.supply-chain-controls/v1" and
  .api_version == "2026-03-10" and
  .reviewed_on == "2026-08-18" and
  .repository == "EnjoyableWork/mcp-doctor" and
  .default_branch == "main" and
  .dependency_updates.auto_merge == false and
  .dependency_updates.exact_direct_requirements == true and
  .dependency_updates.locked_graph == true and
  (.dependency_updates.ecosystems | map(.name)) == ["cargo", "github-actions"] and
  (.dependency_updates.review_dimensions | length) == 12 and
  (.direct_dependencies | length) == 16 and
  all(.direct_dependencies[];
    (.scope == "runtime" or .scope == "development") and
    (.version | test("^[0-9]+\\.[0-9]+\\.[0-9]+$")) and
    (.default_features | type == "boolean") and
    (.features | type == "array")
  ) and
  .github_actions_policy.enabled == true and
  .github_actions_policy.allowed_actions == "selected" and
  .github_actions_policy.sha_pinning_required == true and
  .github_actions_policy.github_owned_allowed == true and
  .github_actions_policy.verified_allowed == false and
  .github_actions_policy.default_workflow_permissions == "read" and
  .github_actions_policy.can_approve_pull_request_reviews == false and
  .github_actions_policy.fork_pull_request_approval_policy == "first_time_contributors" and
  .workflow_inventory.checked_in == [
    ".github/workflows/ci.yml",
    ".github/workflows/compatibility.yml",
    ".github/workflows/release-authorization-negative.yml",
    ".github/workflows/release-channels.yml",
    ".github/workflows/release-preflight.yml",
    ".github/workflows/release.yml"
  ] and
  (.workflow_inventory.provider_managed | map(.path)) == [
    "dynamic/dependabot/dependabot-updates",
    "dynamic/github-code-scanning/codeql"
  ] and
  (.untrusted_workflows | map(.path)) == [
    ".github/workflows/ci.yml",
    ".github/workflows/release-preflight.yml"
  ] and
  all(.untrusted_workflows[];
    .event == "pull_request" and
    .permissions == {"contents": "read"} and
    .github_hosted_only == true and
    .stored_secrets == false and
    .privileged_assets == false
  ) and
  (.actions | length) == 7 and
  all(.actions[];
    (.selection == "direct" or .selection == "nested") and
    (.uses | test("^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+(/[A-Za-z0-9_.-]+)?$")) and
    (.repository | test("^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$")) and
    (.tag | type == "string" and length > 0) and
    (.sha | test("^[0-9a-f]{40}$")) and
    (.license_files | length > 0) and
    all(.license_files[]; .sha256 | test("^[0-9a-f]{64}$"))
  ) and
  (.actions | map(select(.selection == "direct") | .uses) | sort) == [
    "Homebrew/actions/setup-homebrew",
    "actions/attest-build-provenance",
    "actions/checkout",
    "actions/download-artifact",
    "actions/upload-artifact",
    "rust-lang/crates-io-auth-action"
  ] and
  (.standalone_tools | length) == 2 and
  (.standalone_tools | map(.name) | sort) == ["cargo-deny", "syft"] and
  all(.standalone_tools[];
    (.version | type == "string" and length > 0) and
    (.repository | test("^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$")) and
    (.tag | type == "string" and length > 0) and
    (.tag_object | test("^[0-9a-f]{40}$")) and
    (.tag_verified | type == "boolean") and
    (.source_commit | test("^[0-9a-f]{40}$")) and
    (.source_commit_verified | type == "boolean") and
    (.release_immutable | type == "boolean") and
    (.latest_release_required | type == "boolean") and
    (.assets | length > 0) and
    all(.assets[];
      (.target | type == "string" and length > 0) and
      (.archive | type == "string" and length > 0) and
      (.bytes | type == "number" and . > 0) and
      (.sha256 | test("^[0-9a-f]{64}$"))
    )
  ) and
  ([.standalone_tools[] | select(.name == "syft")][0] as $syft |
    $syft.version == "1.51.0" and
    $syft.repository == "anchore/syft" and
    $syft.tag == "v1.51.0" and
    $syft.release_immutable == true and
    $syft.latest_release_required == true and
    ($syft.assets | map(.target) | sort) == [
      "aarch64-unknown-linux-gnu",
      "x86_64-unknown-linux-gnu"
    ]) and
  .source_artifact_policy.binary_exceptions == [] and
  .source_artifact_policy.text_encoding == "UTF-8" and
  .source_artifact_policy.disallowed_ascii_controls == true and
  .distribution_authentication.version == "0.3.0" and
  .distribution_authentication.tag == "v0.3.0" and
  .distribution_authentication.immutable == true and
  .distribution_authentication.homebrew_commit_scope ==
    "immutable_historical_handoff" and
  .distribution_authentication.homebrew_source ==
    "https://github.com/EnjoyableWork/mcp-doctor/releases/download/v0.3.0/mcp-doctor-0.3.0.crate" and
  .mapped_controls == [
    "OSPS-BR-01.01",
    "OSPS-BR-01.03",
    "OSPS-BR-03.02",
    "OSPS-QA-05.01",
    "OSPS-QA-05.02"
  ]
' "$supply_canonical" >/dev/null 2>&1; then
  printf 'canonical supply-chain configuration is invalid\n' >&2
  exit 2
fi

supply_repository="$(jq -er '.repository' "$supply_canonical")"
supply_organization=${supply_repository%%/*}
supply_api_version="$(jq -er '.api_version' "$supply_canonical")"
supply_default_branch="$(jq -er '.default_branch' "$supply_canonical")"
supply_config_hash="$(supply_hash "$supply_canonical")"
supply_date="$(date -u +%F)"
supply_source_sha=unresolved

supply_failure() {
  printf 'date=%s canonical_sha256=%s source_sha=%s result=FAIL\n' \
    "$supply_date" "$supply_config_hash" "$supply_source_sha"
  exit 1
}

umask 077
supply_temp_parent=${TMPDIR:-/tmp}
supply_temp_root="$(mktemp -d "${supply_temp_parent%/}/mcp-doctor-supply-controls.XXXXXX")"
supply_temp_prefix="${supply_temp_parent%/}/mcp-doctor-supply-controls."

supply_cleanup() {
  if [[ "$supply_temp_root" != "$supply_temp_prefix"* ]]; then
    printf 'refusing to remove unexpected supply-chain verifier path\n' >&2
    return 1
  fi
  if [[ -d "$supply_temp_root" ]]; then
    rm -rf -- "$supply_temp_root"
  fi
}
trap supply_cleanup EXIT

supply_api_get() {
  local endpoint=$1
  local destination=$2
  GH_PROMPT_DISABLED=1 GH_PAGER=cat gh api \
    -H "X-GitHub-Api-Version: $supply_api_version" \
    "$endpoint" >"$destination" 2>/dev/null
}

supply_curl() {
  env \
    -u ALL_PROXY -u all_proxy \
    -u HTTP_PROXY -u http_proxy \
    -u HTTPS_PROXY -u https_proxy \
    -u NO_PROXY -u no_proxy \
    curl --disable --fail --silent --show-error --location \
      --proto '=https' \
      --proto-redir '=https' \
      --proxy '' \
      --connect-timeout 10 \
      --max-time 60 \
      --max-redirs 3 \
      --header 'User-Agent: mcp-doctor-supply-chain-verifier/0.1 (+https://github.com/EnjoyableWork/mcp-doctor)' \
      "$@"
}

supply_repository_json="$supply_temp_root/repository.json"
supply_main_json="$supply_temp_root/main.json"
supply_source_json="$supply_temp_root/source.json"
supply_actions_permissions="$supply_temp_root/actions-permissions.json"
supply_selected_actions="$supply_temp_root/selected-actions.json"
supply_workflow_permissions="$supply_temp_root/workflow-permissions.json"
supply_fork_policy="$supply_temp_root/fork-policy.json"
supply_workflows="$supply_temp_root/workflows.json"
supply_repo_secrets="$supply_temp_root/repository-secrets.json"
supply_org_secrets="$supply_temp_root/organization-secrets.json"

if ! supply_api_get "repos/$supply_repository" "$supply_repository_json" ||
  ! supply_api_get "repos/$supply_repository/commits/$supply_default_branch" "$supply_main_json" ||
  ! supply_api_get "repos/$supply_repository/commits/$supply_source_ref" "$supply_source_json" ||
  ! supply_api_get "repos/$supply_repository/actions/permissions" "$supply_actions_permissions" ||
  ! supply_api_get "repos/$supply_repository/actions/permissions/selected-actions" "$supply_selected_actions" ||
  ! supply_api_get "repos/$supply_repository/actions/permissions/workflow" "$supply_workflow_permissions" ||
  ! supply_api_get "repos/$supply_repository/actions/permissions/fork-pr-contributor-approval" "$supply_fork_policy" ||
  ! supply_api_get "repos/$supply_repository/actions/workflows?per_page=100" "$supply_workflows" ||
  ! supply_api_get "repos/$supply_repository/actions/secrets?per_page=100" "$supply_repo_secrets" ||
  ! supply_api_get "orgs/$supply_organization/actions/secrets?per_page=100" "$supply_org_secrets"; then
  supply_failure
fi

supply_source_sha="$(jq -er '.sha' "$supply_source_json" 2>/dev/null)" || supply_failure
supply_main_sha="$(jq -er '.sha' "$supply_main_json" 2>/dev/null)" || supply_failure
supply_local_sha="$(git -C "$supply_repository_root" rev-parse HEAD 2>/dev/null)" || supply_failure

if [[ "$supply_source_sha" != "$supply_main_sha" ]] ||
  [[ "$supply_local_sha" != "$supply_main_sha" ]] ||
  [[ -n "$(git -C "$supply_repository_root" status --short 2>/dev/null)" ]]; then
  supply_failure
fi

supply_expected_patterns="$(
  jq -c '.github_actions_policy.patterns_allowed | sort' "$supply_canonical"
)" || supply_failure
supply_expected_workflows="$(
  jq -c '[
    .workflow_inventory.checked_in[],
    (.workflow_inventory.provider_managed[] | .path)
  ] | sort' "$supply_canonical"
)" || supply_failure
if ! jq -e \
  --arg repository "$supply_repository" \
  --arg branch "$supply_default_branch" '
    .full_name == $repository and
    .visibility == "public" and
    .archived == false and
    .default_branch == $branch and
    .allow_auto_merge == false and
    .security_and_analysis.dependabot_security_updates.status == "enabled"
  ' "$supply_repository_json" >/dev/null 2>&1 ||
  ! jq -e '
    .enabled == true and
    .allowed_actions == "selected" and
    .sha_pinning_required == true
  ' "$supply_actions_permissions" >/dev/null 2>&1 ||
  ! jq -e --argjson patterns "$supply_expected_patterns" '
    .github_owned_allowed == true and
    .verified_allowed == false and
    (.patterns_allowed | sort) == $patterns
  ' "$supply_selected_actions" >/dev/null 2>&1 ||
  ! jq -e '
    .default_workflow_permissions == "read" and
    .can_approve_pull_request_reviews == false
  ' "$supply_workflow_permissions" >/dev/null 2>&1 ||
  ! jq -e '.approval_policy == "first_time_contributors"' \
    "$supply_fork_policy" >/dev/null 2>&1 ||
  ! jq -e --argjson expected "$supply_expected_workflows" '
    (.workflows | map(.path) | sort) == $expected and
    all(.workflows[]; .state == "active")
  ' "$supply_workflows" >/dev/null 2>&1 ||
  ! jq -e '.total_count == 0' "$supply_repo_secrets" >/dev/null 2>&1; then
  supply_failure
fi

supply_repository_id="$(jq -er '.id' "$supply_repository_json")" || supply_failure
while IFS=$'\t' read -r supply_secret_name supply_secret_visibility; do
  [[ -n "$supply_secret_name" ]] || continue
  case "$supply_secret_visibility" in
    all)
      supply_failure
      ;;
    selected)
      supply_secret_repositories="$supply_temp_root/selected-secret-repositories.json"
      if ! supply_api_get \
        "orgs/$supply_organization/actions/secrets/$supply_secret_name/repositories?per_page=100" \
        "$supply_secret_repositories" ||
        jq -e --argjson id "$supply_repository_id" \
          'any(.repositories[]; .id == $id)' \
          "$supply_secret_repositories" >/dev/null 2>&1; then
        supply_failure
      fi
      ;;
    private) ;;
    *) supply_failure ;;
  esac
done < <(jq -r '.secrets[] | [.name, .visibility] | @tsv' "$supply_org_secrets")

"$supply_script_directory/verify-source-artifacts.sh" "$supply_main_sha" \
  >/dev/null 2>&1 || supply_failure

while IFS=$'\t' read -r supply_selection supply_uses supply_action_repository \
  supply_tag supply_sha; do
  supply_action_repo_json="$supply_temp_root/action-repository.json"
  supply_action_ref_json="$supply_temp_root/action-ref.json"
  supply_action_commit_json="$supply_temp_root/action-commit.json"
  if ! supply_api_get "repos/$supply_action_repository" "$supply_action_repo_json" ||
    ! supply_api_get "repos/$supply_action_repository/git/ref/tags/$supply_tag" \
      "$supply_action_ref_json" ||
    ! supply_api_get "repos/$supply_action_repository/commits/$supply_sha" \
      "$supply_action_commit_json" ||
    ! jq -e '.archived == false and .visibility == "public"' \
      "$supply_action_repo_json" >/dev/null 2>&1 ||
    ! jq -e --arg sha "$supply_sha" '
      .sha == $sha and .commit.verification.verified == true
    ' "$supply_action_commit_json" >/dev/null 2>&1; then
    supply_failure
  fi

  supply_ref_type="$(jq -er '.object.type' "$supply_action_ref_json")" || supply_failure
  supply_ref_object="$(jq -er '.object.sha' "$supply_action_ref_json")" || supply_failure
  if [[ "$supply_ref_type" == tag ]]; then
    supply_action_tag_json="$supply_temp_root/action-tag.json"
    if ! supply_api_get "repos/$supply_action_repository/git/tags/$supply_ref_object" \
      "$supply_action_tag_json" ||
      [[ "$(jq -er '.object.sha' "$supply_action_tag_json")" != "$supply_sha" ]]; then
      supply_failure
    fi
  elif [[ "$supply_ref_type" != commit ]] || [[ "$supply_ref_object" != "$supply_sha" ]]; then
    supply_failure
  fi

  while IFS=$'\t' read -r supply_license_path supply_license_hash; do
    supply_license_json="$supply_temp_root/action-license.json"
    supply_license_file="$supply_temp_root/action-license"
    if ! supply_api_get \
      "repos/$supply_action_repository/contents/$supply_license_path?ref=$supply_sha" \
      "$supply_license_json" ||
      ! jq -er '.content' "$supply_license_json" | tr -d '\n' | \
        base64 --decode >"$supply_license_file" 2>/dev/null ||
      [[ "$(supply_hash "$supply_license_file")" != "$supply_license_hash" ]]; then
      supply_failure
    fi
  done < <(
    jq -r --arg uses "$supply_uses" '
      .actions[] | select(.uses == $uses) |
      .license_files[] | [.path, .sha256] | @tsv
    ' "$supply_canonical"
  )

  if [[ "$supply_selection" == nested ]]; then
    supply_parent="$(
      jq -er --arg uses "$supply_uses" \
        '.actions[] | select(.uses == $uses) | .selected_by' \
        "$supply_canonical"
    )" || supply_failure
    supply_parent_repository="$(
      jq -er --arg uses "$supply_parent" \
        '.actions[] | select(.uses == $uses) | .repository' \
        "$supply_canonical"
    )" || supply_failure
    supply_parent_sha="$(
      jq -er --arg uses "$supply_parent" \
        '.actions[] | select(.uses == $uses) | .sha' \
        "$supply_canonical"
    )" || supply_failure
    supply_parent_manifest_json="$supply_temp_root/parent-action.json"
    supply_parent_manifest="$supply_temp_root/parent-action.yml"
    if ! supply_api_get \
      "repos/$supply_parent_repository/contents/action.yml?ref=$supply_parent_sha" \
      "$supply_parent_manifest_json" ||
      ! jq -er '.content' "$supply_parent_manifest_json" | tr -d '\n' | \
        base64 --decode >"$supply_parent_manifest" 2>/dev/null ||
      ! grep -F "uses: $supply_uses@$supply_sha" \
        "$supply_parent_manifest" >/dev/null; then
      supply_failure
    fi
  fi
done < <(
  jq -r '.actions[] | [.selection, .uses, .repository, .tag, .sha] | @tsv' \
    "$supply_canonical"
)

while IFS=$'\t' read -r \
  supply_tool_name supply_tool_repository supply_tool_tag \
  supply_tool_tag_object supply_tool_tag_verified supply_tool_source \
  supply_tool_source_verified supply_tool_immutable \
  supply_tool_latest_required; do
  supply_tool_repo_json="$supply_temp_root/$supply_tool_name-repository.json"
  supply_tool_ref_json="$supply_temp_root/$supply_tool_name-ref.json"
  supply_tool_tag_json="$supply_temp_root/$supply_tool_name-tag.json"
  supply_tool_commit_json="$supply_temp_root/$supply_tool_name-commit.json"
  supply_tool_release_json="$supply_temp_root/$supply_tool_name-release.json"
  if ! supply_api_get "repos/$supply_tool_repository" "$supply_tool_repo_json" ||
    ! supply_api_get "repos/$supply_tool_repository/git/ref/tags/$supply_tool_tag" \
    "$supply_tool_ref_json" ||
    ! supply_api_get "repos/$supply_tool_repository/git/tags/$supply_tool_tag_object" \
    "$supply_tool_tag_json" ||
    ! supply_api_get "repos/$supply_tool_repository/commits/$supply_tool_source" \
    "$supply_tool_commit_json" ||
    ! supply_api_get "repos/$supply_tool_repository/releases/tags/$supply_tool_tag" \
    "$supply_tool_release_json" ||
    ! jq -e '.archived == false and .disabled == false and .visibility == "public"' \
    "$supply_tool_repo_json" >/dev/null 2>&1 ||
    ! jq -e --arg object "$supply_tool_tag_object" '
      .object.type == "tag" and .object.sha == $object
    ' "$supply_tool_ref_json" >/dev/null 2>&1 ||
    ! jq -e \
      --arg source "$supply_tool_source" \
      --argjson verified "$supply_tool_tag_verified" '
        .object.type == "commit" and
        .object.sha == $source and
        .verification.verified == $verified
    ' "$supply_tool_tag_json" >/dev/null 2>&1 ||
    ! jq -e \
      --arg source "$supply_tool_source" \
      --argjson verified "$supply_tool_source_verified" '
        .sha == $source and .commit.verification.verified == $verified
      ' "$supply_tool_commit_json" >/dev/null 2>&1 ||
    ! jq -e \
      --arg tag "$supply_tool_tag" \
      --argjson immutable "$supply_tool_immutable" '
        .tag_name == $tag and
        .draft == false and
        .prerelease == false and
        .immutable == $immutable
      ' "$supply_tool_release_json" >/dev/null 2>&1; then
    supply_failure
  fi

  if [[ "$supply_tool_latest_required" == true ]]; then
    supply_tool_latest_json="$supply_temp_root/$supply_tool_name-latest.json"
    if ! supply_api_get \
      "repos/$supply_tool_repository/releases/latest" \
      "$supply_tool_latest_json" ||
      ! jq -e --arg tag "$supply_tool_tag" \
        '.tag_name == $tag and .draft == false and .prerelease == false' \
        "$supply_tool_latest_json" >/dev/null 2>&1; then
      supply_failure
    fi
  fi

  while IFS=$'\t' read -r \
    supply_tool_archive supply_tool_bytes supply_tool_sha; do
    if ! jq -e \
      --arg name "$supply_tool_archive" \
      --argjson bytes "$supply_tool_bytes" \
      --arg digest "sha256:$supply_tool_sha" '
        any(.assets[];
          .name == $name and
          .size == $bytes and
          .digest == $digest and
          .state == "uploaded"
        )
      ' "$supply_tool_release_json" >/dev/null 2>&1; then
      supply_failure
    fi
  done < <(
    jq -r --arg name "$supply_tool_name" '
      .standalone_tools[] | select(.name == $name) |
      .assets[] | [.archive, .bytes, .sha256] | @tsv
    ' "$supply_canonical"
  )

  while IFS=$'\t' read -r supply_tool_license_path supply_tool_license_hash; do
    supply_tool_license_json="$supply_temp_root/$supply_tool_name-license.json"
    supply_tool_license_file="$supply_temp_root/$supply_tool_name-license"
    if ! supply_api_get \
      "repos/$supply_tool_repository/contents/$supply_tool_license_path?ref=$supply_tool_source" \
      "$supply_tool_license_json" ||
      ! jq -er '.content' "$supply_tool_license_json" | tr -d '\n' | \
        base64 --decode >"$supply_tool_license_file" 2>/dev/null ||
      [[ "$(supply_hash "$supply_tool_license_file")" != \
        "$supply_tool_license_hash" ]]; then
      supply_failure
    fi
  done < <(
    jq -r --arg name "$supply_tool_name" '
      .standalone_tools[] | select(.name == $name) |
      .license_files[]? | [.path, .sha256] | @tsv
    ' "$supply_canonical"
  )
done < <(
  jq -r '
    .standalone_tools[] |
    [
      .name,
      .repository,
      .tag,
      .tag_object,
      (.tag_verified | tostring),
      .source_commit,
      (.source_commit_verified | tostring),
      (.release_immutable | tostring),
      (.latest_release_required | tostring)
    ] | @tsv
  ' "$supply_canonical"
)

supply_release_tag="$(jq -er '.distribution_authentication.tag' "$supply_canonical")" || supply_failure
supply_release_version="$(jq -er '.distribution_authentication.version' "$supply_canonical")" || supply_failure
supply_release_source="$(jq -er '.distribution_authentication.source_commit' "$supply_canonical")" || supply_failure
supply_release_tag_object="$(jq -er '.distribution_authentication.tag_object' "$supply_canonical")" || supply_failure
supply_release_json="$supply_temp_root/release.json"
supply_release_ref_json="$supply_temp_root/release-ref.json"
supply_release_tag_json="$supply_temp_root/release-tag.json"
if ! supply_api_get "repos/$supply_repository/releases/tags/$supply_release_tag" \
  "$supply_release_json" ||
  ! supply_api_get "repos/$supply_repository/git/ref/tags/$supply_release_tag" \
  "$supply_release_ref_json" ||
  ! supply_api_get "repos/$supply_repository/git/tags/$supply_release_tag_object" \
  "$supply_release_tag_json" ||
  ! jq -e --arg tag "$supply_release_tag" '
    .tag_name == $tag and .draft == false and .prerelease == false and
    .immutable == true
  ' "$supply_release_json" >/dev/null 2>&1 ||
  ! jq -e --arg object "$supply_release_tag_object" '
    .object.type == "tag" and .object.sha == $object
  ' "$supply_release_ref_json" >/dev/null 2>&1 ||
  ! jq -e --arg source "$supply_release_source" '
    .object.type == "commit" and .object.sha == $source
  ' "$supply_release_tag_json" >/dev/null 2>&1 ||
  ! jq -e --slurpfile contract "$supply_community" '
    (.assets | map({name, bytes: .size, sha256: (.digest | sub("^sha256:"; ""))}) | sort_by(.name)) ==
    ($contract[0].release_license_contract.assets |
      map({name, bytes, sha256}) | sort_by(.name))
  ' "$supply_release_json" >/dev/null 2>&1; then
  supply_failure
fi

supply_assets="$supply_temp_root/release-assets"
mkdir -p -- "$supply_assets"
if ! GH_PROMPT_DISABLED=1 GH_PAGER=cat gh release download "$supply_release_tag" \
  --repo "$supply_repository" --dir "$supply_assets" >/dev/null 2>&1 ||
  ! GH_PROMPT_DISABLED=1 GH_PAGER=cat gh release verify "$supply_release_tag" \
  --repo "$supply_repository" >/dev/null 2>&1; then
  supply_failure
fi

while IFS=$'\t' read -r supply_asset_name supply_asset_bytes supply_asset_sha; do
  supply_asset_path="$supply_assets/$supply_asset_name"
  if [[ ! -f "$supply_asset_path" ]] || [[ -L "$supply_asset_path" ]] ||
    [[ "$(wc -c <"$supply_asset_path" | tr -d '[:space:]')" != "$supply_asset_bytes" ]] ||
    [[ "$(supply_hash "$supply_asset_path")" != "$supply_asset_sha" ]] ||
    ! GH_PROMPT_DISABLED=1 GH_PAGER=cat gh attestation verify "$supply_asset_path" \
      --repo "$supply_repository" \
      --signer-workflow "$(jq -er '.distribution_authentication.attestation_signer_workflow' "$supply_canonical")" \
      --source-ref "refs/tags/$supply_release_tag" \
      --source-digest "$supply_release_source" \
      --format json >/dev/null 2>&1; then
    supply_failure
  fi
done < <(
  jq -r '.release_license_contract.assets[] | [.name, .bytes, .sha256] | @tsv' \
    "$supply_community"
)

if ! "$supply_script_directory/verify-published-release.sh" \
  "$supply_assets" "$supply_release_version" >/dev/null 2>&1; then
  supply_failure
fi

supply_cargo_uri="$(jq -er '.distribution_authentication.cargo_package' "$supply_canonical")" || supply_failure
supply_cargo_package="$supply_temp_root/mcp-doctor.crate"
supply_cargo_metadata="$supply_temp_root/crates.json"
if ! supply_curl --max-filesize 1000000 --output "$supply_cargo_package" \
  "$supply_cargo_uri" ||
  ! supply_curl --max-filesize 2000000 --output "$supply_cargo_metadata" \
  "https://crates.io/api/v1/crates/mcp-doctor/$supply_release_version" ||
  ! cmp -s "$supply_assets/mcp-doctor-$supply_release_version.crate" \
    "$supply_cargo_package" ||
  ! jq -e --arg version "$supply_release_version" '
    .version.num == $version and .version.yanked == false and
    .version.license == "MIT"
  ' "$supply_cargo_metadata" >/dev/null 2>&1; then
  supply_failure
fi

supply_tap_repository="$(jq -er '.distribution_authentication.homebrew_repository' "$supply_canonical")" || supply_failure
supply_tap_commit="$(jq -er '.distribution_authentication.homebrew_commit' "$supply_canonical")" || supply_failure
supply_formula_path="$(jq -er '.distribution_authentication.homebrew_formula' "$supply_canonical")" || supply_failure
supply_package_sha="$(
  jq -er --arg name "mcp-doctor-$supply_release_version.crate" \
    '.release_license_contract.assets[] | select(.name == $name) | .sha256' \
    "$supply_community"
)" || supply_failure
supply_homebrew_source="$(
  jq -er '.distribution_authentication.homebrew_source' "$supply_canonical"
)" || supply_failure
if ! "$supply_script_directory/verify-historical-homebrew-formula.sh" \
  "$supply_tap_repository" \
  "$supply_tap_commit" \
  "$supply_formula_path" \
  "$supply_assets/mcp-doctor.rb" \
  "$(jq -er '.distribution_authentication.homebrew_formula_sha256' "$supply_canonical")" \
  "$supply_homebrew_source" \
  "$supply_package_sha" \
  "$supply_api_version" >/dev/null 2>&1; then
  supply_failure
fi

printf 'date=%s canonical_sha256=%s source_sha=%s release=%s result=PASS\n' \
  "$supply_date" "$supply_config_hash" "$supply_source_sha" \
  "$supply_release_tag"
