#!/usr/bin/env bash

set -Eeuo pipefail

rehearsal_script_directory="$(
  CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"
rehearsal_repository_root="$(dirname -- "${rehearsal_script_directory}")"
rehearsal_verifier="${rehearsal_script_directory}/verify-organization-controls.sh"
rehearsal_canonical_source="${rehearsal_repository_root}/.github/organization-controls.json"
rehearsal_live_source="${rehearsal_repository_root}/tests/fixtures/organization-controls/live-pass.json"
rehearsal_private_source="${rehearsal_repository_root}/tests/fixtures/organization-controls/private-pass.json"
rehearsal_sentinel="sentinel-private-application"
rehearsal_reference_date="2030-01-01"

for rehearsal_command in awk chmod cp grep jq mktemp rm sed tr wc; do
  if ! command -v "${rehearsal_command}" >/dev/null 2>&1; then
    printf 'required organization-control rehearsal command is unavailable: %s\n' \
      "${rehearsal_command}" >&2
    exit 2
  fi
done

rehearsal_sha256_file() {
  local rehearsal_file="$1"

  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${rehearsal_file}" | awk '{ print $1 }'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${rehearsal_file}" | awk '{ print $1 }'
  else
    printf 'a SHA-256 implementation is required\n' >&2
    return 2
  fi
}

rehearsal_temp_parent="${TMPDIR:-/tmp}"
rehearsal_work_prefix="${rehearsal_temp_parent%/}/mcp-doctor-organization-rehearsal."
umask 077
rehearsal_work_directory="$(mktemp -d "${rehearsal_work_prefix}XXXXXX")"

rehearsal_cleanup() {
  if [[ "${rehearsal_work_directory}" != "${rehearsal_work_prefix}"* ]]; then
    return 1
  fi
  if [[ -d "${rehearsal_work_directory}" ]]; then
    rm -rf -- "${rehearsal_work_directory}"
  fi
}
trap rehearsal_cleanup EXIT

rehearsal_live="${rehearsal_work_directory}/live.json"
rehearsal_private="${rehearsal_work_directory}/private.json"
rehearsal_canonical="${rehearsal_work_directory}/canonical.json"
rehearsal_inventory="${rehearsal_work_directory}/inventory.json"

cp -- "${rehearsal_live_source}" "${rehearsal_live}"
jq -S '.applications.inventory | sort_by(.app_id)' \
  "${rehearsal_live}" >"${rehearsal_inventory}"
rehearsal_inventory_sha="$(rehearsal_sha256_file "${rehearsal_inventory}")"
jq \
  --arg date "${rehearsal_reference_date}" \
  --arg inventory_sha "${rehearsal_inventory_sha}" '
    .observed_on = $date |
    .application_inventory.inventory_sha256 = $inventory_sha |
    .recovery.exercised_on = $date
  ' "${rehearsal_private_source}" >"${rehearsal_private}"
jq \
  --arg date "${rehearsal_reference_date}" \
  '
    .lifecycle = "verified" |
    .reviewed_on = $date |
    .installed_applications.approved_installation_count = 1 |
    .recovery.latest_exercise_on = $date
  ' "${rehearsal_canonical_source}" >"${rehearsal_canonical}"

rehearsal_output="${rehearsal_work_directory}/stdout"
rehearsal_error="${rehearsal_work_directory}/stderr"

rehearsal_run() {
  local rehearsal_selected_canonical="$1"
  local rehearsal_selected_live="$2"
  local rehearsal_selected_private="$3"

  MCP_DOCTOR_ORGANIZATION_FIXTURE=1 \
    "${rehearsal_verifier}" \
    --canonical "${rehearsal_selected_canonical}" \
    --private-attestation "${rehearsal_selected_private}" \
    --fixture "${rehearsal_selected_live}" \
    --verification-date "${rehearsal_reference_date}" \
    >"${rehearsal_output}" 2>"${rehearsal_error}"
}

rehearsal_require_no_disclosure() {
  if grep -Fq "${rehearsal_sentinel}" "${rehearsal_output}" "${rehearsal_error}" \
    >/dev/null 2>&1 ||
    grep -Fq "${rehearsal_inventory_sha}" "${rehearsal_output}" "${rehearsal_error}" \
      >/dev/null 2>&1; then
    printf 'organization-control verifier disclosed private fixture content\n' >&2
    exit 1
  fi
}

rehearsal_expect_failure() {
  local rehearsal_description="$1"
  local rehearsal_selected_canonical="$2"
  local rehearsal_selected_live="$3"
  local rehearsal_selected_private="$4"
  local rehearsal_expected_source="$5"
  local rehearsal_expected_hash
  local rehearsal_expected
  local rehearsal_observed
  local rehearsal_observed_source
  local rehearsal_stderr_bytes

  rehearsal_expected_hash="$(rehearsal_sha256_file "${rehearsal_selected_canonical}")"
  rehearsal_expected="mode=fixture date=${rehearsal_reference_date} canonical_sha256=${rehearsal_expected_hash} source_sha=${rehearsal_expected_source} result=FAIL"
  if rehearsal_run \
    "${rehearsal_selected_canonical}" \
    "${rehearsal_selected_live}" \
    "${rehearsal_selected_private}"; then
    printf 'organization-control verifier accepted %s\n' "${rehearsal_description}" >&2
    exit 1
  fi
  rehearsal_observed="$(tr -d '\r' <"${rehearsal_output}")"
  if [[ "${rehearsal_observed}" != "${rehearsal_expected}" ]] ||
    [[ -s "${rehearsal_error}" ]]; then
    rehearsal_observed_source="$(
      sed -n 's/^mode=fixture date=[0-9-]* canonical_sha256=[0-9a-f]* source_sha=\([^ ]*\) result=FAIL$/\1/p' \
        "${rehearsal_output}"
    )"
    rehearsal_stderr_bytes="$(wc -c <"${rehearsal_error}" | tr -d '[:space:]')"
    if [[ -z "${rehearsal_observed_source}" ]]; then
      rehearsal_observed_source="invalid-output-shape"
    fi
    printf 'organization-control verifier emitted unexpected evidence for %s (expected source %s; observed source %s; stderr bytes %s)\n' \
      "${rehearsal_description}" "${rehearsal_expected_source}" \
      "${rehearsal_observed_source}" "${rehearsal_stderr_bytes}" >&2
    exit 1
  fi
  rehearsal_require_no_disclosure
}

rehearsal_expect_failure \
  'an activation-only canonical projection' \
  "${rehearsal_canonical_source}" \
  "${rehearsal_live}" \
  "${rehearsal_private}" \
  unresolved

rehearsal_pass_hash="$(rehearsal_sha256_file "${rehearsal_canonical}")"
rehearsal_expected_pass="mode=fixture date=${rehearsal_reference_date} canonical_sha256=${rehearsal_pass_hash} source_sha=1111111111111111111111111111111111111111 result=PASS"
if ! rehearsal_run "${rehearsal_canonical}" "${rehearsal_live}" "${rehearsal_private}" ||
  [[ "$(tr -d '\r' <"${rehearsal_output}")" != "${rehearsal_expected_pass}" ]] ||
  [[ -s "${rehearsal_error}" ]]; then
  printf 'organization-control verifier rejected the conforming fixture\n' >&2
  exit 1
fi
rehearsal_require_no_disclosure

rehearsal_mutate_live() {
  local rehearsal_name="$1"
  local rehearsal_filter="$2"
  local rehearsal_mutated="${rehearsal_work_directory}/${rehearsal_name}.json"

  jq "${rehearsal_filter}" "${rehearsal_live}" >"${rehearsal_mutated}"
  printf '%s\n' "${rehearsal_mutated}"
}

rehearsal_bad_fixture_sha="$(
  rehearsal_mutate_live bad-fixture-sha \
    '.source_sha = "2222222222222222222222222222222222222222"'
)"
rehearsal_expect_failure 'a fixture impersonating a live source commit' \
  "${rehearsal_canonical}" "${rehearsal_bad_fixture_sha}" \
  "${rehearsal_private}" 2222222222222222222222222222222222222222

rehearsal_bad_2fa="$(rehearsal_mutate_live bad-2fa '.membership.members_2fa_insecure = 1')"
rehearsal_expect_failure 'an insecure MFA member' \
  "${rehearsal_canonical}" "${rehearsal_bad_2fa}" "${rehearsal_private}" \
  1111111111111111111111111111111111111111

rehearsal_bad_default="$(
  rehearsal_mutate_live bad-default '.organization.default_repository_permission = "read"'
)"
rehearsal_expect_failure 'a broad default repository permission' \
  "${rehearsal_canonical}" "${rehearsal_bad_default}" "${rehearsal_private}" \
  1111111111111111111111111111111111111111

rehearsal_bad_member_privilege="$(
  rehearsal_mutate_live bad-member-privilege \
    '.organization.member_privileges.members_can_delete_repositories = true'
)"
rehearsal_expect_failure 'a broad member privilege' \
  "${rehearsal_canonical}" "${rehearsal_bad_member_privilege}" \
  "${rehearsal_private}" 1111111111111111111111111111111111111111

rehearsal_changed_plan="$(
  rehearsal_mutate_live changed-plan '.organization.plan = "enterprise"'
)"
rehearsal_expect_failure 'an organization plan change' \
  "${rehearsal_canonical}" "${rehearsal_changed_plan}" \
  "${rehearsal_private}" 1111111111111111111111111111111111111111

rehearsal_changed_invitation_setting="$(
  rehearsal_mutate_live changed-invitation-setting \
    '.organization.outside_collaborator_invitations.observed_members_can_invite_outside_collaborators = false'
)"
rehearsal_expect_failure 'an outside-collaborator invitation setting change' \
  "${rehearsal_canonical}" "${rehearsal_changed_invitation_setting}" \
  "${rehearsal_private}" 1111111111111111111111111111111111111111

rehearsal_nonowner_admin="$(
  rehearsal_mutate_live nonowner-admin \
    '.membership.nonowner_direct_admin_identities = 1'
)"
rehearsal_expect_failure 'a non-owner repository administrator' \
  "${rehearsal_canonical}" "${rehearsal_nonowner_admin}" \
  "${rehearsal_private}" 1111111111111111111111111111111111111111

rehearsal_bad_app_scope="$(
  rehearsal_mutate_live bad-app-scope \
    '.applications.inventory[0].repository_selection = "all"'
)"
rehearsal_expect_failure 'an all-repository application' \
  "${rehearsal_canonical}" "${rehearsal_bad_app_scope}" "${rehearsal_private}" \
  1111111111111111111111111111111111111111

rehearsal_bad_app_identity="$(
  rehearsal_mutate_live bad-app-identity \
    '.applications.inventory[0].permissions.contents = "write"'
)"
rehearsal_expect_failure 'application inventory drift' \
  "${rehearsal_canonical}" "${rehearsal_bad_app_identity}" \
  "${rehearsal_private}" 1111111111111111111111111111111111111111

rehearsal_bad_token="$(
  rehearsal_mutate_live bad-token \
    '.operator_credential_type = "classic_personal_access_token"'
)"
rehearsal_expect_failure 'a classic personal access token' \
  "${rehearsal_canonical}" "${rehearsal_bad_token}" "${rehearsal_private}" \
  1111111111111111111111111111111111111111

rehearsal_bad_org_secret="$(
  rehearsal_mutate_live bad-org-secret '.organization_credentials.actions_secrets = 1'
)"
rehearsal_expect_failure 'an organization Actions secret' \
  "${rehearsal_canonical}" "${rehearsal_bad_org_secret}" "${rehearsal_private}" \
  1111111111111111111111111111111111111111

rehearsal_unreviewed_operator="${rehearsal_work_directory}/unreviewed-operator.json"
jq '.assertions.operator_credential_reviewed_for_current_verification = false' \
  "${rehearsal_private}" >"${rehearsal_unreviewed_operator}"
rehearsal_expect_failure 'an unreviewed operator credential' \
  "${rehearsal_canonical}" "${rehearsal_live}" \
  "${rehearsal_unreviewed_operator}" unresolved

rehearsal_world_readable_private="${rehearsal_work_directory}/world-readable-private.json"
cp -- "${rehearsal_private}" "${rehearsal_world_readable_private}"
chmod 0644 "${rehearsal_world_readable_private}"
rehearsal_expect_failure 'a group-or-world-readable private attestation' \
  "${rehearsal_canonical}" "${rehearsal_live}" \
  "${rehearsal_world_readable_private}" unresolved

rehearsal_bad_tap_key="$(
  rehearsal_mutate_live bad-tap-key \
    '(.repositories[] | select(.repository == "EnjoyableWork/homebrew-tap") | .deploy_keys) = 1 | (.repositories[] | select(.repository == "EnjoyableWork/homebrew-tap") | .write_deploy_keys) = 1'
)"
rehearsal_expect_failure 'a distribution write deploy key' \
  "${rehearsal_canonical}" "${rehearsal_bad_tap_key}" "${rehearsal_private}" \
  1111111111111111111111111111111111111111

rehearsal_bad_private="${rehearsal_work_directory}/bad-private.json"
jq '.assertions.recovery_exercise_passed = false' \
  "${rehearsal_private}" >"${rehearsal_bad_private}"
rehearsal_expect_failure 'an unconfirmed recovery exercise' \
  "${rehearsal_canonical}" "${rehearsal_live}" "${rehearsal_bad_private}" \
  unresolved

rehearsal_stale_canonical="${rehearsal_work_directory}/stale-canonical.json"
rehearsal_stale_private="${rehearsal_work_directory}/stale-private.json"
jq '.recovery.latest_exercise_on = "2000-01-01"' \
  "${rehearsal_canonical}" >"${rehearsal_stale_canonical}"
jq '.recovery.exercised_on = "2000-01-01"' \
  "${rehearsal_private}" >"${rehearsal_stale_private}"
rehearsal_expect_failure 'stale recovery evidence' \
  "${rehearsal_stale_canonical}" "${rehearsal_live}" \
  "${rehearsal_stale_private}" unresolved

rehearsal_stale_review="${rehearsal_work_directory}/stale-review.json"
jq '.reviewed_on = "2000-01-01"' \
  "${rehearsal_canonical}" >"${rehearsal_stale_review}"
rehearsal_expect_failure 'a stale organization-control review' \
  "${rehearsal_stale_review}" "${rehearsal_live}" \
  "${rehearsal_private}" unresolved

if ! rehearsal_cleanup; then
  printf 'organization-control rehearsal cleanup failed\n' >&2
  exit 1
fi
trap - EXIT

printf 'Organization-control verifier rehearsal passed.\n'
