#!/usr/bin/env bash

set -Eeuo pipefail

assurance_script_directory="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
assurance_repository_root="$(dirname -- "$assurance_script_directory")"
assurance_canonical="$assurance_repository_root/.github/assurance-controls.json"
assurance_proposal="$assurance_repository_root/.bestpractices.json"
assurance_community="$assurance_repository_root/.github/community-license-controls.json"
assurance_readme="$assurance_repository_root/README.md"
assurance_source_ref=main

if [[ $# -eq 2 ]] && [[ "$1" == --source-ref ]]; then
  assurance_source_ref=$2
elif [[ $# -ne 0 ]]; then
  printf 'usage: %s [--source-ref main|40-hex-commit]\n' "$0" >&2
  exit 2
fi
if [[ "$assurance_source_ref" != main ]] &&
  [[ ! "$assurance_source_ref" =~ ^[0-9a-f]{40}$ ]]; then
  printf 'source ref must be main or a full lowercase Git commit SHA\n' >&2
  exit 2
fi

for assurance_command in awk cmp curl date dirname env grep jq mkdir mktemp rm tar uname wc; do
  if ! command -v "$assurance_command" >/dev/null 2>&1; then
    printf 'required assurance verifier command is unavailable\n' >&2
    exit 2
  fi
done

assurance_hash() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print $1 }'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{ print $1 }'
  else
    printf 'a SHA-256 implementation is required\n' >&2
    return 2
  fi
}

assurance_crosswalk_url='https://github.com/EnjoyableWork/mcp-doctor/blob/main/docs/assurance/osps-v2026.02.19-level-1.md'

if ! jq -e --arg crosswalk_url "$assurance_crosswalk_url" \
  --slurpfile proposal "$assurance_proposal" '
  . as $canonical |
  $proposal[0] as $answers |
  .schema_version == "mcp-doctor.assurance-controls/v1" and
  .reviewed_on == "2026-08-15" and
  .repository == "EnjoyableWork/mcp-doctor" and
  .organization == "EnjoyableWork" and
  (.assessed_main_commit | test("^[0-9a-f]{40}$")) and
  .osps.framework == "OpenSSF OSPS Baseline" and
  .osps.version == "v2026.02.19" and
  .osps.level == 1 and
  .osps.controls_total == 24 and
  .osps.met == 24 and
  .osps.not_applicable == 0 and
  .osps.crosswalk == "docs/assurance/osps-v2026.02.19-level-1.md" and
  .osps.proposal == ".bestpractices.json" and
  (.osps.upstream_source_commit | test("^[0-9a-f]{40}$")) and
  (.osps.badgeapp_source_commit | test("^[0-9a-f]{40}$")) and
  .osps.badgeapp.project_id > 0 and
  .osps.badgeapp.publication_state == "verified" and
  .osps.badgeapp.project_url ==
    ("https://www.bestpractices.dev/en/projects/" + (.osps.badgeapp.project_id | tostring)) and
  .osps.badgeapp.project_json_url ==
    ("https://www.bestpractices.dev/projects/" + (.osps.badgeapp.project_id | tostring) + ".json") and
  .osps.badgeapp.baseline_badge_url ==
    ("https://www.bestpractices.dev/projects/" + (.osps.badgeapp.project_id | tostring) + "/baseline") and
  .osps.badgeapp.baseline_entry_url ==
    ("https://www.bestpractices.dev/en/projects/" + (.osps.badgeapp.project_id | tostring) + "/baseline-1") and
  .osps.badgeapp.achieved_at == "2026-08-15T22:14:15.614Z" and
  (.osps.controls | length) == 24 and
  (.osps.controls | unique | length) == 24 and
  (.osps.controls | sort) == [
    "OSPS-AC-01.01", "OSPS-AC-02.01", "OSPS-AC-03.01", "OSPS-AC-03.02",
    "OSPS-BR-01.01", "OSPS-BR-01.03", "OSPS-BR-03.01", "OSPS-BR-03.02",
    "OSPS-BR-07.01", "OSPS-DO-01.01", "OSPS-DO-02.01", "OSPS-GV-02.01",
    "OSPS-GV-03.01", "OSPS-LE-02.01", "OSPS-LE-02.02", "OSPS-LE-03.01",
    "OSPS-LE-03.02", "OSPS-QA-01.01", "OSPS-QA-01.02", "OSPS-QA-02.01",
    "OSPS-QA-04.01", "OSPS-QA-05.01", "OSPS-QA-05.02", "OSPS-VM-02.01"
  ] and
  ([ $answers | to_entries[] | select(.key | test("^osps_[a-z]{2}_[0-9]{2}_[0-9]{2}_status$")) ] | length) == 24 and
  all(.osps.controls[];
    (ascii_downcase | gsub("[-.]"; "_")) as $field |
    $answers[($field + "_status")] == "Met" and
    $answers[($field + "_justification")] == ("Dated evidence and scope: " + $crosswalk_url)
  ) and
  .slsa.version == "v1.2" and
  .slsa.build_level == 2 and
  .slsa.crosswalk == "docs/assurance/slsa-v1.2-build-l2.md" and
  .slsa.predicate_type == "https://slsa.dev/provenance/v1" and
  .slsa.release.version == "0.3.0" and
  .slsa.release.tag == "v0.3.0" and
  (.slsa.release.tag_object | test("^[0-9a-f]{40}$")) and
  (.slsa.release.source_commit | test("^[0-9a-f]{40}$")) and
  .slsa.release.workflow == ".github/workflows/release.yml" and
  .slsa.release.run_id == 31755736570 and
  .slsa.release.run_attempt == 1 and
  .slsa.release.published_at == "2026-08-14T00:02:39Z" and
  .slsa.release.immutable == true and
  .slsa.verifier.name == "GitHub CLI" and
  .slsa.verifier.version == "2.97.0" and
  .slsa.verifier.repository == "cli/cli" and
  .slsa.verifier.tag == "v2.97.0" and
  .slsa.verifier.source_commit == "55dbb4dc6b7edb10b48e3d7fc5bccd32318d1b55" and
  .slsa.verifier.source_commit_verified == true and
  .slsa.verifier.release_immutable == true and
  .slsa.verifier.checksums_asset == "gh_2.97.0_checksums.txt" and
  .slsa.verifier.checksums_asset_bytes == 1950 and
  (.slsa.verifier.checksums_asset_sha256 | test("^[0-9a-f]{64}$")) and
  (.slsa.verifier.assets | length) == 2 and
  (.slsa.verifier.assets | map(.host) | sort) == [
    "aarch64-apple-darwin",
    "x86_64-unknown-linux-gnu"
  ] and
  all(.slsa.verifier.assets[];
    (.archive | test("^gh_2\\.97\\.0_[A-Za-z0-9_.-]+$")) and
    (.bytes | type == "number" and . > 0) and
    (.sha256 | test("^[0-9a-f]{64}$")) and
    (.directory | test("^gh_2\\.97\\.0_[A-Za-z0-9_.-]+$"))
  ) and
  ([.slsa.verifier.assets[] | select(.host == "aarch64-apple-darwin")][0] == {
    "host": "aarch64-apple-darwin",
    "archive": "gh_2.97.0_macOS_arm64.zip",
    "bytes": 13845290,
    "sha256": "a58b8fd77b417a38f47a0b54d1370c59b0fcdb324ccc9ca002b0998f7c4c999e",
    "directory": "gh_2.97.0_macOS_arm64"
  }) and
  ([.slsa.verifier.assets[] | select(.host == "x86_64-unknown-linux-gnu")][0] == {
    "host": "x86_64-unknown-linux-gnu",
    "archive": "gh_2.97.0_linux_amd64.tar.gz",
    "bytes": 14770812,
    "sha256": "a2c9b8497e1f85b1ad0dfcb78b5a622e098801b8e461e459e88e1ee12f018112",
    "directory": "gh_2.97.0_linux_amd64"
  }) and
  .maintenance.cadence == "at_least_annually" and
  .maintenance.next_scheduled_review_by == "2027-08-15" and
  (.maintenance.event_triggers | length) == 7 and
  .maintenance.failure_action == "correct_or_remove_public_claim_and_badge_immediately"
' "$assurance_canonical" >/dev/null; then
  printf 'canonical assurance configuration or BadgeApp proposal is invalid\n' >&2
  exit 2
fi

assurance_repository="$(jq -er '.repository' "$assurance_canonical")"
assurance_badge_id="$(jq -er '.osps.badgeapp.project_id' "$assurance_canonical")"
assurance_release_tag="$(jq -er '.slsa.release.tag' "$assurance_canonical")"
assurance_source_sha=unresolved
assurance_asset_count=0
assurance_date="$(date -u +%F)"
assurance_config_hash="$(assurance_hash "$assurance_canonical")"

assurance_failure() {
  trap - ERR
  printf 'date=%s canonical_sha256=%s source_sha=%s badgeapp_project=%s release=%s assets=%s result=FAIL\n' \
    "$assurance_date" "$assurance_config_hash" "$assurance_source_sha" \
    "$assurance_badge_id" "$assurance_release_tag" "$assurance_asset_count"
  exit 1
}
trap assurance_failure ERR

umask 077
assurance_temp_parent=${TMPDIR:-/tmp}
assurance_temp_root="$(mktemp -d "${assurance_temp_parent%/}/mcp-doctor-assurance-evidence.XXXXXX")"
assurance_temp_prefix="${assurance_temp_parent%/}/mcp-doctor-assurance-evidence."

assurance_cleanup() {
  if [[ "$assurance_temp_root" != "$assurance_temp_prefix"* ]]; then
    printf 'refusing to remove unexpected assurance verifier path\n' >&2
    return 1
  fi
  if [[ -d "$assurance_temp_root" ]]; then
    rm -rf -- "$assurance_temp_root"
  fi
}
trap assurance_cleanup EXIT

assurance_curl() {
  env \
    -u ALL_PROXY -u all_proxy \
    -u CURL_HOME \
    -u HTTPS_PROXY -u https_proxy \
    -u HTTP_PROXY -u http_proxy \
    -u NETRC \
    -u NO_PROXY -u no_proxy \
    curl --disable --fail --silent --show-error --location \
      --proto '=https' \
      --proto-redir '=https' \
      --proxy '' \
      --retry 0 \
      --connect-timeout 10 \
      --max-time 120 \
      --max-redirs 3 \
      --header 'Authorization:' \
      --header 'Cookie:' \
      --user-agent 'mcp-doctor-assurance-verifier/1 (+https://github.com/EnjoyableWork/mcp-doctor)' \
      "$@"
}

case "$(uname -s):$(uname -m)" in
  Darwin:arm64)
    assurance_verifier_host=aarch64-apple-darwin
    ;;
  Linux:x86_64)
    assurance_verifier_host=x86_64-unknown-linux-gnu
    ;;
  *)
    printf 'the exact assurance verifier has no reviewed asset for this host\n' >&2
    exit 2
    ;;
esac

assurance_gh_version="$(jq -er '.slsa.verifier.version' "$assurance_canonical")"
assurance_gh_tag="$(jq -er '.slsa.verifier.tag' "$assurance_canonical")"
assurance_gh_archive="$(
  jq -er --arg host "$assurance_verifier_host" \
    '.slsa.verifier.assets[] | select(.host == $host) | .archive' \
    "$assurance_canonical"
)"
assurance_gh_archive_bytes="$(
  jq -er --arg host "$assurance_verifier_host" \
    '.slsa.verifier.assets[] | select(.host == $host) | .bytes' \
    "$assurance_canonical"
)"
assurance_gh_archive_sha256="$(
  jq -er --arg host "$assurance_verifier_host" \
    '.slsa.verifier.assets[] | select(.host == $host) | .sha256' \
    "$assurance_canonical"
)"
assurance_gh_directory="$(
  jq -er --arg host "$assurance_verifier_host" \
    '.slsa.verifier.assets[] | select(.host == $host) | .directory' \
    "$assurance_canonical"
)"
assurance_gh_root="$assurance_temp_root/gh"
assurance_gh_archive_path="$assurance_temp_root/$assurance_gh_archive"
assurance_gh_archive_list="$assurance_temp_root/gh-archive.list"
assurance_gh_checksums="$assurance_temp_root/gh-checksums.txt"
mkdir -p -- "$assurance_gh_root"

assurance_curl --max-filesize "$assurance_gh_archive_bytes" \
  --output "$assurance_gh_archive_path" \
  "https://github.com/cli/cli/releases/download/$assurance_gh_tag/$assurance_gh_archive"
[[ "$(wc -c <"$assurance_gh_archive_path" | awk '{ print $1 }')" == "$assurance_gh_archive_bytes" ]]
[[ "$(assurance_hash "$assurance_gh_archive_path")" == "$assurance_gh_archive_sha256" ]]

assurance_gh_checksums_bytes="$(jq -er '.slsa.verifier.checksums_asset_bytes' "$assurance_canonical")"
assurance_gh_checksums_sha256="$(jq -er '.slsa.verifier.checksums_asset_sha256' "$assurance_canonical")"
assurance_gh_checksums_name="$(jq -er '.slsa.verifier.checksums_asset' "$assurance_canonical")"
assurance_curl --max-filesize "$assurance_gh_checksums_bytes" \
  --output "$assurance_gh_checksums" \
  "https://github.com/cli/cli/releases/download/$assurance_gh_tag/$assurance_gh_checksums_name"
[[ "$(wc -c <"$assurance_gh_checksums" | awk '{ print $1 }')" == "$assurance_gh_checksums_bytes" ]]
[[ "$(assurance_hash "$assurance_gh_checksums")" == "$assurance_gh_checksums_sha256" ]]
grep -Fx "$assurance_gh_archive_sha256  $assurance_gh_archive" \
  "$assurance_gh_checksums" >/dev/null

tar -tf "$assurance_gh_archive_path" >"$assurance_gh_archive_list"
if grep -Ev "^${assurance_gh_directory}/(LICENSE|bin/gh|share/man/man1/[A-Za-z0-9_.-]+)$" \
  "$assurance_gh_archive_list" >/dev/null; then
  assurance_failure
fi
[[ "$(grep -Fxc "$assurance_gh_directory/bin/gh" "$assurance_gh_archive_list")" == 1 ]]
tar -xf "$assurance_gh_archive_path" -C "$assurance_gh_root"
assurance_gh="$assurance_gh_root/$assurance_gh_directory/bin/gh"
[[ -f "$assurance_gh" && ! -L "$assurance_gh" && -x "$assurance_gh" ]]
assurance_actual_gh_version="$($assurance_gh version | awk 'NR == 1 { print $3 }')"
if [[ "$assurance_actual_gh_version" != "$assurance_gh_version" ]]; then
  printf 'GitHub CLI does not match the exact reviewed assurance verifier release\n' >&2
  assurance_failure
fi
if ! "$assurance_gh" auth status --hostname github.com >/dev/null 2>&1; then
  printf 'GitHub CLI must be authenticated for public attestation verification\n' >&2
  assurance_failure
fi

assurance_api_get() {
  local endpoint=$1
  local destination=$2
  GH_PROMPT_DISABLED=1 GH_PAGER=cat "$assurance_gh" api "$endpoint" \
    >"$destination" 2>/dev/null
}

assurance_source_json="$assurance_temp_root/source.json"
assurance_api_get "repos/$assurance_repository/commits/$assurance_source_ref" "$assurance_source_json"
assurance_source_sha="$(jq -er '.sha' "$assurance_source_json")"
if [[ ! "$assurance_source_sha" =~ ^[0-9a-f]{40}$ ]]; then
  assurance_failure
fi

assurance_source_index=0
for assurance_relative_file in \
  .bestpractices.json \
  .github/assurance-controls.json \
  .github/community-license-controls.json \
  .github/workflows/release.yml \
  README.md \
  docs/assurance/osps-v2026.02.19-level-1.md \
  docs/assurance/slsa-v1.2-build-l2.md \
  scripts/verify-assurance-evidence.sh; do
  assurance_source_index=$((assurance_source_index + 1))
  assurance_remote_file="$assurance_temp_root/source-$assurance_source_index"
  GH_PROMPT_DISABLED=1 GH_PAGER=cat "$assurance_gh" api \
    -H 'Accept: application/vnd.github.raw+json' \
    "repos/$assurance_repository/contents/$assurance_relative_file?ref=$assurance_source_sha" \
    >"$assurance_remote_file" 2>/dev/null
  cmp -s "$assurance_repository_root/$assurance_relative_file" "$assurance_remote_file"
done

assurance_osps_home="$assurance_temp_root/osps-home.html"
assurance_osps_version="$assurance_temp_root/osps-version.html"
assurance_badgeapp_config="$assurance_temp_root/badgeapp-config.rb"
assurance_curl --max-filesize 2097152 --output "$assurance_osps_home" \
  https://baseline.openssf.org/
assurance_curl --max-filesize 4194304 --output "$assurance_osps_version" \
  https://baseline.openssf.org/versions/2026-02-19
assurance_curl --max-filesize 65536 --output "$assurance_badgeapp_config" \
  https://raw.githubusercontent.com/ossf/best-practices-badge/main/app/lib/baseline_config.rb
grep -F 'versions/2026-02-19' "$assurance_osps_home" >/dev/null
grep -F "CURRENT_VERSION = 'v2026.02.19'" "$assurance_badgeapp_config" >/dev/null
grep -F 'IN_TRANSITION = false' "$assurance_badgeapp_config" >/dev/null
while IFS= read -r assurance_control; do
  grep -F "$assurance_control" "$assurance_osps_version" >/dev/null
done < <(jq -er '.osps.controls[]' "$assurance_canonical")

assurance_project_url="$(jq -er '.osps.badgeapp.project_url' "$assurance_canonical")"
assurance_project_json_url="$(jq -er '.osps.badgeapp.project_json_url' "$assurance_canonical")"
assurance_badge_url="$(jq -er '.osps.badgeapp.baseline_badge_url' "$assurance_canonical")"
assurance_entry_url="$(jq -er '.osps.badgeapp.baseline_entry_url' "$assurance_canonical")"
assurance_project_html="$assurance_temp_root/project.html"
assurance_entry_html="$assurance_temp_root/baseline-entry.html"
assurance_project_json="$assurance_temp_root/project.json"
assurance_badge_svg="$assurance_temp_root/baseline.svg"
assurance_curl --max-filesize 2097152 --output "$assurance_project_html" "$assurance_project_url"
assurance_curl --max-filesize 2097152 --output "$assurance_entry_html" "$assurance_entry_url"
assurance_curl --max-filesize 2097152 --output "$assurance_project_json" "$assurance_project_json_url"
assurance_curl --max-filesize 262144 --output "$assurance_badge_svg" "$assurance_badge_url"

if ! jq -e \
  --argjson id "$assurance_badge_id" \
  --arg crosswalk_url "$assurance_crosswalk_url" \
  --slurpfile canonical "$assurance_canonical" '
  . as $project |
  $canonical[0].osps.controls as $controls |
  .id == $id and
  .name == "mcp-doctor" and
  .repo_url == "https://github.com/EnjoyableWork/mcp-doctor" and
  .badge_percentage_baseline_1 == 100 and
  .achieved_baseline_1_at == $canonical[0].osps.badgeapp.achieved_at and
  all($controls[];
    . as $control |
    $project[($control + "_status")] == "Met" and
    ($project[($control + "_justification")] | contains($crosswalk_url))
  )
' "$assurance_project_json" >/dev/null; then
  assurance_failure
fi
grep -F 'v2026.02.19' "$assurance_entry_html" >/dev/null
grep -F 'baseline v2026.02.19: 1' "$assurance_badge_svg" >/dev/null
grep -F "<a href=\"$assurance_entry_url\"><img alt=\"OpenSSF OSPS Baseline v2026.02.19 Level 1\" src=\"$assurance_badge_url\"></a>" \
  "$assurance_readme" >/dev/null

assurance_release_json="$assurance_temp_root/release.json"
GH_PROMPT_DISABLED=1 GH_PAGER=cat "$assurance_gh" release view "$assurance_release_tag" \
  --repo "$assurance_repository" \
  --json isImmutable,tagName,targetCommitish,publishedAt,assets \
  >"$assurance_release_json" 2>/dev/null

if ! jq -e --slurpfile community "$assurance_community" '
  . as $release |
  $community[0].release_license_contract as $expected |
  .isImmutable == true and
  .tagName == $expected.tag and
  .targetCommitish == "main" and
  .publishedAt == $expected.published_at and
  (.assets | length) == ($expected.assets | length) and
  all($expected.assets[];
    . as $asset |
    any($release.assets[];
      .name == $asset.name and
      .size == $asset.bytes and
      .digest == ("sha256:" + $asset.sha256)
    )
  )
' "$assurance_release_json" >/dev/null; then
  assurance_failure
fi

assurance_tag_ref="$assurance_temp_root/tag-ref.json"
assurance_tag_object_json="$assurance_temp_root/tag-object.json"
assurance_api_get "repos/$assurance_repository/git/ref/tags/$assurance_release_tag" "$assurance_tag_ref"
assurance_tag_object="$(jq -er '.slsa.release.tag_object' "$assurance_canonical")"
assurance_release_source="$(jq -er '.slsa.release.source_commit' "$assurance_canonical")"
jq -e --arg tag_object "$assurance_tag_object" \
  '.object.type == "tag" and .object.sha == $tag_object' \
  "$assurance_tag_ref" >/dev/null
assurance_api_get "repos/$assurance_repository/git/tags/$assurance_tag_object" "$assurance_tag_object_json"
jq -e --arg source "$assurance_release_source" \
  '.object.type == "commit" and .object.sha == $source and .verification.verified == true' \
  "$assurance_tag_object_json" >/dev/null

assurance_signer="$assurance_repository/.github/workflows/release.yml"
assurance_signer_uri="https://github.com/$assurance_signer@refs/tags/$assurance_release_tag"
assurance_invocation="https://github.com/$assurance_repository/actions/runs/31755736570/attempts/1"
assurance_asset_root="$assurance_temp_root/assets"
mkdir -p -- "$assurance_asset_root"

while IFS=$'\t' read -r assurance_asset_name assurance_asset_bytes assurance_asset_sha256; do
  if [[ ! "$assurance_asset_name" =~ ^[A-Za-z0-9._+-]+$ ]] ||
    [[ ! "$assurance_asset_bytes" =~ ^[1-9][0-9]*$ ]] ||
    [[ ! "$assurance_asset_sha256" =~ ^[0-9a-f]{64}$ ]]; then
    assurance_failure
  fi
  assurance_asset_count=$((assurance_asset_count + 1))
  assurance_asset_path="$assurance_asset_root/$assurance_asset_name"
  assurance_result_path="$assurance_asset_root/$assurance_asset_name.attestation.json"
  assurance_curl --max-filesize "$assurance_asset_bytes" --output "$assurance_asset_path" \
    "https://github.com/$assurance_repository/releases/download/$assurance_release_tag/$assurance_asset_name"
  assurance_actual_bytes="$(wc -c <"$assurance_asset_path" | awk '{ print $1 }')"
  [[ "$assurance_actual_bytes" == "$assurance_asset_bytes" ]]
  [[ "$(assurance_hash "$assurance_asset_path")" == "$assurance_asset_sha256" ]]

  GH_PROMPT_DISABLED=1 GH_PAGER=cat "$assurance_gh" attestation verify "$assurance_asset_path" \
    --repo "$assurance_repository" \
    --signer-workflow "$assurance_signer" \
    --signer-digest "$assurance_release_source" \
    --source-ref "refs/tags/$assurance_release_tag" \
    --source-digest "$assurance_release_source" \
    --cert-oidc-issuer https://token.actions.githubusercontent.com \
    --deny-self-hosted-runners \
    --predicate-type https://slsa.dev/provenance/v1 \
    --format json >"$assurance_result_path" 2>/dev/null

  if ! jq -e \
    --arg name "$assurance_asset_name" \
    --arg sha256 "$assurance_asset_sha256" \
    --arg signer "$assurance_signer_uri" \
    --arg source "$assurance_release_source" \
    --arg tag_ref "refs/tags/$assurance_release_tag" \
    --arg invocation "$assurance_invocation" '
    length > 0 and
    any(.[];
      .verificationResult as $result |
      $result.signature.certificate.subjectAlternativeName == $signer and
      $result.signature.certificate.issuer == "https://token.actions.githubusercontent.com" and
      $result.signature.certificate.githubWorkflowSHA == $source and
      $result.signature.certificate.githubWorkflowRef == $tag_ref and
      $result.signature.certificate.buildSignerDigest == $source and
      $result.signature.certificate.runnerEnvironment == "github-hosted" and
      $result.signature.certificate.sourceRepositoryDigest == $source and
      $result.signature.certificate.sourceRepositoryRef == $tag_ref and
      $result.signature.certificate.runInvocationURI == $invocation and
      ($result.verifiedTimestamps | length) > 0 and
      $result.statement._type == "https://in-toto.io/Statement/v1" and
      $result.statement.predicateType == "https://slsa.dev/provenance/v1" and
      any($result.statement.subject[];
        .name == $name and .digest.sha256 == $sha256
      ) and
      $result.statement.predicate.buildDefinition.buildType ==
        "https://actions.github.io/buildtypes/workflow/v1" and
      $result.statement.predicate.buildDefinition.externalParameters.workflow == {
        "path": ".github/workflows/release.yml",
        "ref": $tag_ref,
        "repository": "https://github.com/EnjoyableWork/mcp-doctor"
      } and
      any($result.statement.predicate.buildDefinition.resolvedDependencies[];
        .uri == ("git+https://github.com/EnjoyableWork/mcp-doctor@" + $tag_ref) and
        .digest.gitCommit == $source
      ) and
      $result.statement.predicate.runDetails.builder.id == $signer and
      $result.statement.predicate.runDetails.metadata.invocationId == $invocation
    )
  ' "$assurance_result_path" >/dev/null; then
    assurance_failure
  fi
done < <(
  jq -er '.release_license_contract.assets[] | [.name, (.bytes | tostring), .sha256] | @tsv' \
    "$assurance_community"
)

[[ "$assurance_asset_count" == 7 ]]

trap - ERR
printf 'date=%s canonical_sha256=%s source_sha=%s badgeapp_project=%s release=%s assets=%s result=PASS\n' \
  "$assurance_date" "$assurance_config_hash" "$assurance_source_sha" \
  "$assurance_badge_id" "$assurance_release_tag" "$assurance_asset_count"
