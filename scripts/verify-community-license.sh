#!/usr/bin/env bash

set -Eeuo pipefail

community_script_directory="$(
  CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"
community_repository_root="$(dirname -- "${community_script_directory}")"
community_canonical_path="${community_repository_root}/.github/community-license-controls.json"
community_requested_ref="main"
community_canonical_was_set=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --source-ref)
      if [[ $# -lt 2 ]]; then
        printf 'usage: %s [--source-ref main|40-hex-commit] [canonical-config]\n' "$0" >&2
        exit 2
      fi
      community_requested_ref="$2"
      shift 2
      ;;
    --*)
      printf 'usage: %s [--source-ref main|40-hex-commit] [canonical-config]\n' "$0" >&2
      exit 2
      ;;
    *)
      if [[ "${community_canonical_was_set}" == true ]]; then
        printf 'usage: %s [--source-ref main|40-hex-commit] [canonical-config]\n' "$0" >&2
        exit 2
      fi
      community_canonical_path="$1"
      community_canonical_was_set=true
      shift
      ;;
  esac
done

if [[ "${community_requested_ref}" != "main" &&
  ! "${community_requested_ref}" =~ ^[0-9a-f]{40}$ ]]; then
  printf 'source ref must be main or one lowercase 40-hex commit\n' >&2
  exit 2
fi

for community_required_command in awk cmp curl grep jq sed tar wc; do
  if ! command -v "${community_required_command}" >/dev/null 2>&1; then
    printf 'required command is unavailable\n' >&2
    exit 2
  fi
done

community_sha256_file() {
  local community_file="$1"

  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${community_file}" | awk '{ print $1 }'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${community_file}" | awk '{ print $1 }'
  else
    printf 'a SHA-256 implementation is required\n' >&2
    return 2
  fi
}

if ! jq -e '
  .schema_version == "mcp-doctor.github-community-license-controls/v1" and
  .reviewed_on == "2026-08-13" and
  .api_version == "2026-03-10" and
  .organization == "EnjoyableWork" and
  .project_repository == "EnjoyableWork/mcp-doctor" and
  (.public_repository_inventory | length) == 3 and
  ([.public_repository_inventory[].repository] | unique | length) == 3 and
  ([.public_repository_inventory[].classification] | sort) == [
    "in_scope_distribution",
    "in_scope_primary",
    "separate_project"
  ] and
  (.public_repository_inventory | map({
    repository,
    classification,
    default_branch,
    license,
    archived,
    fork
  }) | sort_by(.repository)) == [
    {
      "repository": "EnjoyableWork/homebrew-tap",
      "classification": "in_scope_distribution",
      "default_branch": "main",
      "license": "MIT",
      "archived": false,
      "fork": false
    },
    {
      "repository": "EnjoyableWork/mcp-doctor",
      "classification": "in_scope_primary",
      "default_branch": "main",
      "license": "MIT",
      "archived": false,
      "fork": false
    },
    {
      "repository": "EnjoyableWork/mcp-sync",
      "classification": "separate_project",
      "default_branch": "main",
      "license": "MIT",
      "archived": false,
      "fork": false
    }
  ] and
  .community_contract.policy_repository == .project_repository and
  .community_contract.default_branch == "main" and
  .community_contract.public_discussion_uri ==
    "https://github.com/EnjoyableWork/mcp-doctor/issues" and
  .community_contract.issue_intake_uri ==
    "https://github.com/EnjoyableWork/mcp-doctor/issues/new/choose" and
  .community_contract.private_vulnerability_uri ==
    "https://github.com/EnjoyableWork/mcp-doctor/security/advisories/new" and
  .community_contract.content_reports_enabled == true and
  .community_contract.blank_issues_enabled == false and
  .community_contract.inbound_license == "MIT" and
  .community_contract.outbound_license == "MIT" and
  .community_contract.contributor_license_agreement == "not_required" and
  .community_contract.developer_certificate_of_origin == "optional" and
  (.community_contract.required_files | length) == 15 and
  ([.community_contract.required_files[]] | unique | length) == 15 and
  .community_contract.github_recognized_files == [
    "CODE_OF_CONDUCT.md",
    "CONTRIBUTING.md",
    "LICENSE",
    ".github/pull_request_template.md",
    "README.md"
  ] and
  .community_contract.issue_forms == [
    ".github/ISSUE_TEMPLATE/01-bug-report.yml",
    ".github/ISSUE_TEMPLATE/02-feature-request.yml"
  ] and
  (.official_channels | length) == 7 and
  ([.official_channels[].channel] | unique | length) == 7 and
  all(.official_channels[].uri; test("^https://[^/@]+(?:/|$)")) and
  (.official_channels | sort_by(.channel)) == [
    {
      "channel": "canonical_release",
      "uri": "https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.3.0"
    },
    {
      "channel": "cargo_package",
      "uri": "https://crates.io/crates/mcp-doctor/0.3.0"
    },
    {
      "channel": "homebrew_formula",
      "uri": "https://github.com/EnjoyableWork/homebrew-tap/blob/main/Formula/mcp-doctor.rb"
    },
    {
      "channel": "private_vulnerability_reporting",
      "uri": "https://github.com/EnjoyableWork/mcp-doctor/security/advisories/new"
    },
    {
      "channel": "public_discussion_and_defects",
      "uri": "https://github.com/EnjoyableWork/mcp-doctor/issues"
    },
    {
      "channel": "source",
      "uri": "https://github.com/EnjoyableWork/mcp-doctor"
    },
    {
      "channel": "third_party_documentation_mirror",
      "uri": "https://docs.rs/crate/mcp-doctor/latest"
    }
  ] and
  .source_license == {
    "spdx_expression": "MIT",
    "license_path": "LICENSE",
    "license_sha256": "32a82b79c71a3a633dc51fcb306f0d4768551aaff7c8862f67a5997a5f75faea",
    "manifest_path": "Cargo.toml"
  } and
  .tap_contract.repository == "EnjoyableWork/homebrew-tap" and
  .tap_contract.default_branch == "main" and
  (.tap_contract.reviewed_commit | test("^[0-9a-f]{40}$")) and
  .tap_contract.formula_license == "MIT" and
  .tap_contract.license_sha256 == .source_license.license_sha256 and
  .release_license_contract.version == "0.3.0" and
  .release_license_contract.tag == "v0.3.0" and
  (.release_license_contract.tag_object | test("^[0-9a-f]{40}$")) and
  (.release_license_contract.source_commit | test("^[0-9a-f]{40}$")) and
  .release_license_contract.spdx_expression == "MIT" and
  .release_license_contract.github_release_uri ==
    "https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.3.0" and
  .release_license_contract.cargo_api_uri ==
    "https://crates.io/api/v1/crates/mcp-doctor/0.3.0" and
  .release_license_contract.cargo_download_uri ==
    "https://static.crates.io/crates/mcp-doctor/mcp-doctor-0.3.0.crate" and
  (.release_license_contract.assets | length) == 7 and
  ([.release_license_contract.assets[].name] | unique | length) == 7 and
  all(.release_license_contract.assets[];
    (.bytes | type == "number" and . > 0 and . <= 8388608) and
    (.sha256 | test("^[0-9a-f]{64}$"))
  ) and
  ([.release_license_contract.assets[].classification] | sort) == [
    "package_formula",
    "release_metadata",
    "sbom_document",
    "sbom_document",
    "software_archive",
    "software_archive",
    "software_package"
  ] and
  ([.limitations[].surface] | sort) == [
    "sbom_project_package_license",
    "separate_organization_projects",
    "supply_chain_and_complete_assurance"
  ] and
  .mapped_controls == [
    "OSPS-BR-03.01",
    "OSPS-DO-02.01",
    "OSPS-GV-02.01",
    "OSPS-GV-03.01",
    "OSPS-LE-02.01",
    "OSPS-LE-02.02",
    "OSPS-LE-03.01",
    "OSPS-LE-03.02",
    "OSPS-QA-04.01"
  ]
' "${community_canonical_path}" >/dev/null 2>&1; then
  printf 'canonical community and license configuration is invalid\n' >&2
  exit 2
fi

community_canonical_hash="$(community_sha256_file "${community_canonical_path}")"
community_verification_date="$(date -u +%F)"
community_source_sha="unresolved"

community_report_failure() {
  trap - ERR
  printf 'date=%s canonical_sha256=%s source_sha=%s result=FAIL\n' \
    "${community_verification_date}" \
    "${community_canonical_hash}" \
    "${community_source_sha}"
  exit 1
}

trap community_report_failure ERR

community_temp_parent="${TMPDIR:-/tmp}"
community_work_prefix="${community_temp_parent%/}/mcp-doctor-community-license."
umask 077
community_work_directory="$(mktemp -d "${community_work_prefix}XXXXXX")"

community_cleanup() {
  if [[ "${community_work_directory}" != "${community_work_prefix}"* ]]; then
    return 1
  fi
  if [[ -d "${community_work_directory}" ]]; then
    rm -rf -- "${community_work_directory}"
  fi
}
trap community_cleanup EXIT

community_public_get() {
  local community_url="$1"
  local community_destination="$2"
  local community_maximum_bytes="$3"

  env \
    -u ALL_PROXY \
    -u all_proxy \
    -u CURL_HOME \
    -u GITHUB_AUTH_TOKEN \
    -u GITHUB_TOKEN \
    -u GH_TOKEN \
    -u HTTPS_PROXY \
    -u https_proxy \
    -u HTTP_PROXY \
    -u http_proxy \
    -u NETRC \
    curl --disable \
    --silent \
    --show-error \
    --fail \
    --location \
    --max-redirs 5 \
    --proto '=https' \
    --proto-redir '=https' \
    --proxy '' \
    --retry 0 \
    --connect-timeout 10 \
    --max-time 120 \
    --max-filesize "${community_maximum_bytes}" \
    --header 'Accept: application/vnd.github+json' \
    --header 'Authorization:' \
    --header 'Cookie:' \
    --header "X-GitHub-Api-Version: $(jq -er '.api_version' "${community_canonical_path}")" \
    --user-agent 'mcp-doctor-community-license-verifier/1 (+https://github.com/EnjoyableWork/mcp-doctor)' \
    --output "${community_destination}" \
    "${community_url}" 2>/dev/null
}

community_organization="$(jq -er '.organization' "${community_canonical_path}")"
community_project_repository="$(jq -er '.project_repository' "${community_canonical_path}")"
community_primary_name="${community_project_repository#*/}"
community_primary_api="https://api.github.com/repos/${community_project_repository}"

community_repositories_json="${community_work_directory}/repositories.json"
community_primary_json="${community_work_directory}/primary.json"
community_primary_commit_json="${community_work_directory}/primary-commit.json"
community_profile_json="${community_work_directory}/community-profile.json"
community_tap_json="${community_work_directory}/tap.json"

community_public_get \
  "https://api.github.com/orgs/${community_organization}/repos?type=public&per_page=100" \
  "${community_repositories_json}" \
  2097152
community_public_get "${community_primary_api}" "${community_primary_json}" 1048576
community_public_get \
  "${community_primary_api}/commits/${community_requested_ref}" \
  "${community_primary_commit_json}" \
  1048576
community_public_get \
  "${community_primary_api}/community/profile" \
  "${community_profile_json}" \
  1048576
community_public_get \
  "https://api.github.com/repos/$(jq -er '.tap_contract.repository' "${community_canonical_path}")" \
  "${community_tap_json}" \
  1048576

community_source_sha="$(jq -er '.sha' "${community_primary_commit_json}")"
if [[ ! "${community_source_sha}" =~ ^[0-9a-f]{40}$ ]]; then
  community_report_failure
fi
if [[ "${community_requested_ref}" != "main" &&
  "${community_source_sha}" != "${community_requested_ref}" ]]; then
  community_report_failure
fi

jq -S '[.public_repository_inventory[] | {
  repository,
  default_branch,
  license,
  archived,
  fork
}] | sort_by(.repository)' \
  "${community_canonical_path}" >"${community_work_directory}/expected-repositories.json"
jq -S '[.[] | {
  repository: .full_name,
  default_branch,
  license: (.license.spdx_id // "NOASSERTION"),
  archived,
  fork
}] | sort_by(.repository)' \
  "${community_repositories_json}" >"${community_work_directory}/actual-repositories.json"
cmp -s \
  "${community_work_directory}/expected-repositories.json" \
  "${community_work_directory}/actual-repositories.json"

jq -e \
  --arg repository "${community_project_repository}" \
  --arg branch "$(jq -er '.community_contract.default_branch' "${community_canonical_path}")" '
    .full_name == $repository and
    .visibility == "public" and
    .default_branch == $branch and
    .archived == false and
    .disabled == false and
    .fork == false and
    .has_issues == true and
    .license.spdx_id == "MIT"
  ' "${community_primary_json}" >/dev/null 2>&1

jq -e '
  .health_percentage == 100 and
  .content_reports_enabled == true and
  .files.code_of_conduct != null and
  .files.contributing != null and
  .files.license != null and
  .files.pull_request_template != null and
  .files.readme != null and
  all([
    .files.code_of_conduct.html_url,
    .files.contributing.html_url,
    .files.license.html_url,
    .files.pull_request_template.html_url,
    .files.readme.html_url
  ][]; test("^https://github\\.com/EnjoyableWork/mcp-doctor/"))
' "${community_profile_json}" >/dev/null 2>&1

jq -e \
  --arg repository "$(jq -er '.tap_contract.repository' "${community_canonical_path}")" \
  --arg branch "$(jq -er '.tap_contract.default_branch' "${community_canonical_path}")" '
    .full_name == $repository and
    .visibility == "public" and
    .default_branch == $branch and
    .archived == false and
    .disabled == false and
    .fork == false and
    .license.spdx_id == "MIT"
  ' "${community_tap_json}" >/dev/null 2>&1

community_policy_index=0
while IFS= read -r community_relative_file; do
  if [[ ! "${community_relative_file}" =~ ^[A-Za-z0-9._/-]+$ ||
    "${community_relative_file}" == /* ||
    "${community_relative_file}" == *".."* ]]; then
    community_report_failure
  fi
  community_policy_index=$((community_policy_index + 1))
  community_remote_file="${community_work_directory}/source-policy-${community_policy_index}"
  community_public_get \
    "https://raw.githubusercontent.com/${community_project_repository}/${community_source_sha}/${community_relative_file}" \
    "${community_remote_file}" \
    4194304
  cmp -s \
    "${community_repository_root}/${community_relative_file}" \
    "${community_remote_file}"
done < <(jq -er '.community_contract.required_files[]' "${community_canonical_path}")

community_license_path="${community_repository_root}/$(jq -er '.source_license.license_path' "${community_canonical_path}")"
community_expected_license_hash="$(jq -er '.source_license.license_sha256' "${community_canonical_path}")"
[[ "$(community_sha256_file "${community_license_path}")" == "${community_expected_license_hash}" ]]

community_manifest="${community_repository_root}/Cargo.toml"
grep -Fx 'license = "MIT"' "${community_manifest}" >/dev/null
grep -Fx 'repository = "https://github.com/EnjoyableWork/mcp-doctor"' "${community_manifest}" >/dev/null
grep -Fx 'homepage = "https://github.com/EnjoyableWork/mcp-doctor"' "${community_manifest}" >/dev/null
grep -Fx '    "/.github/community-license-controls.json",' "${community_manifest}" >/dev/null
grep -Fx '    "/docs/**",' "${community_manifest}" >/dev/null
grep -Fx '    "/LICENSE",' "${community_manifest}" >/dev/null

community_contributing="${community_repository_root}/CONTRIBUTING.md"
community_conduct="${community_repository_root}/CODE_OF_CONDUCT.md"
community_support="${community_repository_root}/SUPPORT.md"
community_readme="${community_repository_root}/README.md"
community_scope="${community_repository_root}/docs/project-scope.md"
community_issue_config="${community_repository_root}/.github/ISSUE_TEMPLATE/config.yml"

for community_expected_text in \
  'same inbound and outbound terms' \
  'requires neither a' \
  'Signed-off-by' \
  'right to license'; do
  grep -F "${community_expected_text}" "${community_contributing}" >/dev/null
done
for community_expected_text in \
  'private **Report content** action' \
  'Repository content reporting is enabled' \
  'https://support.github.com/contact/report-abuse' \
  '[SECURITY.md](SECURITY.md)'; do
  grep -F "${community_expected_text}" "${community_conduct}" >/dev/null
done
for community_expected_text in \
  'This source tree represents' \
  '0.3.0' \
  '01-bug-report.yml' \
  '02-feature-request.yml' \
  '[SECURITY.md](SECURITY.md)'; do
  grep -F "${community_expected_text}" "${community_support}" >/dev/null
done
if grep -F 'project is pre-release' "${community_support}" >/dev/null; then
  community_report_failure
fi
for community_expected_text in \
  '[Code of Conduct](CODE_OF_CONDUCT.md)' \
  '[project scope](docs/project-scope.md)' \
  '[MIT License](LICENSE)'; do
  grep -F "${community_expected_text}" "${community_readme}" >/dev/null
done
for community_expected_text in \
  'EnjoyableWork/mcp-doctor' \
  'EnjoyableWork/homebrew-tap' \
  'mcp-sync' \
  'private repository' \
  'NOASSERTION' \
  'does not authenticate the supply chain'; do
  grep -F "${community_expected_text}" "${community_scope}" >/dev/null
done
grep -Fx 'blank_issues_enabled: false' "${community_issue_config}" >/dev/null
grep -F 'https://github.com/EnjoyableWork/mcp-doctor/security/advisories/new' \
  "${community_issue_config}" >/dev/null
grep -F 'https://github.com/EnjoyableWork/mcp-doctor/blob/main/SUPPORT.md' \
  "${community_issue_config}" >/dev/null

while IFS= read -r community_issue_form; do
  for community_form_key in 'name:' 'description:' 'body:'; do
    grep -F "${community_form_key}" \
      "${community_repository_root}/${community_issue_form}" >/dev/null
  done
done < <(jq -er '.community_contract.issue_forms[]' "${community_canonical_path}")

community_tap_repository="$(jq -er '.tap_contract.repository' "${community_canonical_path}")"
community_tap_branch="$(jq -er '.tap_contract.default_branch' "${community_canonical_path}")"
community_tap_index=0
for community_tap_key in readme license formula; do
  community_tap_index=$((community_tap_index + 1))
  community_tap_relative_file="$(jq -er ".tap_contract.${community_tap_key}_path" "${community_canonical_path}")"
  community_tap_file="${community_work_directory}/tap-${community_tap_index}"
  community_public_get \
    "https://raw.githubusercontent.com/${community_tap_repository}/${community_tap_branch}/${community_tap_relative_file}" \
    "${community_tap_file}" \
    2097152
  [[ "$(community_sha256_file "${community_tap_file}")" == \
    "$(jq -er ".tap_contract.${community_tap_key}_sha256" "${community_canonical_path}")" ]]
done

community_tap_readme="${community_work_directory}/tap-1"
community_tap_license="${community_work_directory}/tap-2"
community_tap_formula="${community_work_directory}/tap-3"
for community_expected_text in \
  'supporting Homebrew distribution codebase' \
  'https://github.com/EnjoyableWork/mcp-doctor/blob/main/CONTRIBUTING.md' \
  'https://github.com/EnjoyableWork/mcp-doctor/issues/new/choose' \
  'https://github.com/EnjoyableWork/mcp-doctor/blob/main/SUPPORT.md' \
  'https://github.com/EnjoyableWork/mcp-doctor/blob/main/CODE_OF_CONDUCT.md' \
  'https://github.com/EnjoyableWork/mcp-doctor/security/advisories/new'; do
  grep -F "${community_expected_text}" "${community_tap_readme}" >/dev/null
done
cmp -s "${community_license_path}" "${community_tap_license}"
grep -Fx '  license "MIT"' "${community_tap_formula}" >/dev/null
grep -E '^  homepage "https://' "${community_tap_formula}" >/dev/null
grep -E '^  url "https://' "${community_tap_formula}" >/dev/null

community_version="$(jq -er '.release_license_contract.version' "${community_canonical_path}")"
community_tag="$(jq -er '.release_license_contract.tag' "${community_canonical_path}")"
community_release_json="${community_work_directory}/release.json"
community_tag_ref_json="${community_work_directory}/tag-ref.json"
community_tag_json="${community_work_directory}/tag.json"
community_crate_version_json="${community_work_directory}/crate-version.json"
community_crate_json="${community_work_directory}/crate.json"

community_public_get \
  "https://api.github.com/repos/${community_project_repository}/releases/tags/${community_tag}" \
  "${community_release_json}" \
  2097152
community_public_get \
  "https://api.github.com/repos/${community_project_repository}/git/ref/tags/${community_tag}" \
  "${community_tag_ref_json}" \
  1048576
community_tag_object="$(jq -er '.release_license_contract.tag_object' "${community_canonical_path}")"
community_public_get \
  "https://api.github.com/repos/${community_project_repository}/git/tags/${community_tag_object}" \
  "${community_tag_json}" \
  1048576
community_public_get \
  "$(jq -er '.release_license_contract.cargo_api_uri' "${community_canonical_path}")" \
  "${community_crate_version_json}" \
  2097152
community_public_get \
  "https://crates.io/api/v1/crates/${community_primary_name}" \
  "${community_crate_json}" \
  2097152

jq -e \
  --arg tag "${community_tag}" \
  --arg published "$(jq -er '.release_license_contract.published_at' "${community_canonical_path}")" '
    .tag_name == $tag and
    .draft == false and
    .prerelease == false and
    .immutable == true and
    .published_at == $published
  ' "${community_release_json}" >/dev/null 2>&1

jq -S '[.release_license_contract.assets[] | {
  name,
  size: .bytes,
  digest: ("sha256:" + .sha256)
}] | sort_by(.name)' \
  "${community_canonical_path}" >"${community_work_directory}/expected-assets.json"
jq -S '[.assets[] | {name, size, digest}] | sort_by(.name)' \
  "${community_release_json}" >"${community_work_directory}/actual-assets.json"
cmp -s \
  "${community_work_directory}/expected-assets.json" \
  "${community_work_directory}/actual-assets.json"

jq -e \
  --arg tag_object "${community_tag_object}" '
    .object.type == "tag" and .object.sha == $tag_object
  ' "${community_tag_ref_json}" >/dev/null 2>&1
jq -e \
  --arg tag "${community_tag}" \
  --arg source "$(jq -er '.release_license_contract.source_commit' "${community_canonical_path}")" '
    .tag == $tag and .object.type == "commit" and .object.sha == $source
  ' "${community_tag_json}" >/dev/null 2>&1

community_release_crate_name="mcp-doctor-${community_version}.crate"
community_release_formula_name="mcp-doctor.rb"
community_checksum_name="SHA256SUMS"

while IFS=$'\t' read -r community_asset_name community_asset_bytes community_asset_sha community_asset_class; do
  if [[ ! "${community_asset_name}" =~ ^[A-Za-z0-9._-]+$ ||
    ! "${community_asset_bytes}" =~ ^[0-9]+$ ||
    ! "${community_asset_sha}" =~ ^[0-9a-f]{64}$ ]]; then
    community_report_failure
  fi
  community_asset_file="${community_work_directory}/asset-${community_asset_name}"
  community_public_get \
    "https://github.com/${community_project_repository}/releases/download/${community_tag}/${community_asset_name}" \
    "${community_asset_file}" \
    "$((community_asset_bytes + 1))"
  [[ "$(wc -c <"${community_asset_file}" | awk '{ print $1 }')" == "${community_asset_bytes}" ]]
  [[ "$(community_sha256_file "${community_asset_file}")" == "${community_asset_sha}" ]]

  case "${community_asset_class}" in
    software_package | software_archive)
      community_member="$(jq -er --arg name "${community_asset_name}" '
        .release_license_contract.assets[] |
        select(.name == $name) |
        .license_evidence.member
      ' "${community_canonical_path}")"
      tar -tzf "${community_asset_file}" >"${community_work_directory}/archive-list" 2>/dev/null
      if grep -E '(^/|(^|/)\.\.(/|$))' "${community_work_directory}/archive-list" >/dev/null; then
        community_report_failure
      fi
      [[ "$(grep -Fxc "${community_member}" "${community_work_directory}/archive-list")" == "1" ]]
      tar -xOzf "${community_asset_file}" "${community_member}" \
        >"${community_work_directory}/asset-license" 2>/dev/null
      [[ "$(community_sha256_file "${community_work_directory}/asset-license")" == \
        "${community_expected_license_hash}" ]]
      ;;
    sbom_document)
      jq -e \
        --arg version "${community_version}" '
          .spdxVersion == "SPDX-2.3" and
          .dataLicense == "CC0-1.0" and
          ([.packages[] |
            select(.name == "mcp-doctor" and .versionInfo == $version) |
            select(
              .licenseDeclared == "NOASSERTION" and
              .licenseConcluded == "NOASSERTION"
            )] | length) == 1
        ' "${community_asset_file}" >/dev/null 2>&1
      ;;
    package_formula)
      cmp -s "${community_asset_file}" "${community_tap_formula}"
      grep -Fx '  license "MIT"' "${community_asset_file}" >/dev/null
      ;;
    release_metadata)
      ;;
    *)
      community_report_failure
      ;;
  esac
done < <(jq -er '
  .release_license_contract.assets[] |
  [.name, (.bytes | tostring), .sha256, .classification] |
  @tsv
' "${community_canonical_path}")

community_release_crate="${community_work_directory}/asset-${community_release_crate_name}"
community_release_formula="${community_work_directory}/asset-${community_release_formula_name}"
community_checksum_file="${community_work_directory}/asset-${community_checksum_name}"

cmp -s "${community_release_formula}" "${community_tap_formula}"
while IFS=$'\t' read -r community_asset_name community_asset_sha; do
  grep -Fx "${community_asset_sha}  ${community_asset_name}" \
    "${community_checksum_file}" >/dev/null
done < <(jq -er '
  .release_license_contract.assets[] |
  select(.name != "SHA256SUMS") |
  [.name, .sha256] |
  @tsv
' "${community_canonical_path}")

jq -e \
  --arg version "${community_version}" \
  --argjson size "$(jq -er --arg name "${community_release_crate_name}" '
    .release_license_contract.assets[] | select(.name == $name) | .bytes
  ' "${community_canonical_path}")" \
  --arg checksum "$(jq -er --arg name "${community_release_crate_name}" '
    .release_license_contract.assets[] | select(.name == $name) | .sha256
  ' "${community_canonical_path}")" '
    .version.num == $version and
    .version.crate == "mcp-doctor" and
    .version.license == "MIT" and
    .version.crate_size == $size and
    .version.checksum == $checksum and
    .version.dl_path == ("/api/v1/crates/mcp-doctor/" + $version + "/download") and
    .version.yanked == false
  ' "${community_crate_version_json}" >/dev/null 2>&1
jq -e \
  --arg version "${community_version}" '
    .crate.id == "mcp-doctor" and
    .crate.homepage == "https://github.com/EnjoyableWork/mcp-doctor" and
    .crate.repository == "https://github.com/EnjoyableWork/mcp-doctor" and
    .crate.max_version == $version and
    .crate.newest_version == $version and
    .crate.max_stable_version == $version
  ' "${community_crate_json}" >/dev/null 2>&1

community_static_crate="${community_work_directory}/static.crate"
community_public_get \
  "$(jq -er '.release_license_contract.cargo_download_uri' "${community_canonical_path}")" \
  "${community_static_crate}" \
  1048576
cmp -s "${community_static_crate}" "${community_release_crate}"

tar -xOzf \
  "${community_release_crate}" \
  "mcp-doctor-${community_version}/Cargo.toml.orig" \
  >"${community_work_directory}/published-manifest" 2>/dev/null
grep -Fx 'license = "MIT"' "${community_work_directory}/published-manifest" >/dev/null
grep -Fx 'repository = "https://github.com/EnjoyableWork/mcp-doctor"' \
  "${community_work_directory}/published-manifest" >/dev/null

community_docs_mirror="${community_work_directory}/docs-mirror.html"
community_public_get \
  "$(jq -er '.official_channels[] | select(.channel == "third_party_documentation_mirror") | .uri' "${community_canonical_path}")" \
  "${community_docs_mirror}" \
  4194304
grep -F 'mcp-doctor' "${community_docs_mirror}" >/dev/null

printf 'date=%s canonical_sha256=%s source_sha=%s result=PASS\n' \
  "${community_verification_date}" \
  "${community_canonical_hash}" \
  "${community_source_sha}"
