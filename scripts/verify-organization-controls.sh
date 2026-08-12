#!/usr/bin/env bash

set -Eeuo pipefail

organization_script_directory="$(
  CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"
organization_repository_root="$(dirname -- "${organization_script_directory}")"
organization_canonical_path="${organization_repository_root}/.github/organization-controls.json"
organization_canonical_overridden=false
organization_private_attestation=""
organization_fixture_path=""
organization_verification_mode="live"
organization_verification_date_override=""

organization_usage() {
  printf '%s\n' \
    "usage: $0 --private-attestation FILE [--canonical FILE] [--fixture FILE --verification-date YYYY-MM-DD]" >&2
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --canonical)
      if [[ $# -lt 2 ]]; then
        organization_usage
        exit 2
      fi
      organization_canonical_path="$2"
      organization_canonical_overridden=true
      shift 2
      ;;
    --private-attestation)
      if [[ $# -lt 2 ]]; then
        organization_usage
        exit 2
      fi
      organization_private_attestation="$2"
      shift 2
      ;;
    --fixture)
      if [[ $# -lt 2 ]]; then
        organization_usage
        exit 2
      fi
      organization_fixture_path="$2"
      shift 2
      ;;
    --verification-date)
      if [[ $# -lt 2 ]]; then
        organization_usage
        exit 2
      fi
      organization_verification_date_override="$2"
      shift 2
      ;;
    *)
      organization_usage
      exit 2
      ;;
  esac
done

if [[ -n "${organization_fixture_path}" ]]; then
  organization_verification_mode="fixture"
fi
if [[ "${organization_canonical_overridden}" == true ]] &&
  [[ "${organization_verification_mode}" != "fixture" ]]; then
  organization_usage
  exit 2
fi
if [[ "${organization_verification_mode}" == "fixture" ]] &&
  [[ -z "${organization_verification_date_override}" ]]; then
  organization_usage
  exit 2
fi
if [[ "${organization_verification_mode}" != "fixture" ]] &&
  [[ -n "${organization_verification_date_override}" ]]; then
  organization_usage
  exit 2
fi

if [[ -z "${organization_private_attestation}" ]]; then
  organization_usage
  exit 2
fi

for organization_command in awk chmod cp date env git jq mktemp rm sed sort stat tr wc; do
  if ! command -v "${organization_command}" >/dev/null 2>&1; then
    printf 'required organization-control verifier command is unavailable: %s\n' \
      "${organization_command}" >&2
    exit 2
  fi
done
if [[ -z "${organization_fixture_path}" ]]; then
  for organization_command in curl gh; do
    if ! command -v "${organization_command}" >/dev/null 2>&1; then
      printf 'required organization-control verifier command is unavailable: %s\n' \
        "${organization_command}" >&2
      exit 2
    fi
  done
fi

organization_sha256_file() {
  local organization_file="$1"

  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${organization_file}" | awk '{ print $1 }'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${organization_file}" | awk '{ print $1 }'
  else
    printf 'a SHA-256 implementation is required\n' >&2
    return 2
  fi
}

organization_date_epoch() {
  local organization_date_value="$1"
  local organization_parsed_date
  local organization_parsed_epoch
  local organization_parsed_value

  if organization_parsed_value="$(
    date -u -d "${organization_date_value}T00:00:00Z" '+%F %s' 2>/dev/null
  )"; then
    :
  elif organization_parsed_value="$(
    date -j -u -f '%Y-%m-%dT%H:%M:%SZ' \
      "${organization_date_value}T00:00:00Z" '+%F %s' 2>/dev/null
  )"; then
    :
  else
    return 1
  fi

  organization_parsed_date="${organization_parsed_value%% *}"
  organization_parsed_epoch="${organization_parsed_value##* }"
  if [[ "${organization_parsed_date}" != "${organization_date_value}" ]] ||
    [[ ! "${organization_parsed_epoch}" =~ ^[0-9]+$ ]]; then
    return 1
  fi
  printf '%s\n' "${organization_parsed_epoch}"
}

organization_regular_bounded_file() {
  local organization_file="$1"
  local organization_maximum_bytes="$2"
  local organization_bytes

  if [[ ! -f "${organization_file}" ]] ||
    [[ -L "${organization_file}" ]] ||
    [[ ! -r "${organization_file}" ]]; then
    return 1
  fi
  organization_bytes="$(
    wc -c <"${organization_file}" 2>/dev/null | tr -d '[:space:]'
  )" || return 1
  [[ "${organization_bytes}" =~ ^[0-9]+$ ]] &&
    ((organization_bytes > 0 && organization_bytes <= organization_maximum_bytes))
}

organization_private_file_mode_is_restricted() {
  local organization_file="$1"
  local organization_mode
  local organization_mode_value

  if organization_mode="$(stat -c '%a' "${organization_file}" 2>/dev/null)"; then
    :
  elif organization_mode="$(stat -f '%Lp' "${organization_file}" 2>/dev/null)"; then
    :
  else
    return 1
  fi
  if [[ ! "${organization_mode}" =~ ^[0-7]{3,4}$ ]]; then
    return 1
  fi
  organization_mode_value="$((8#${organization_mode}))"
  (( (organization_mode_value & 8#077) == 0 ))
}

if ! organization_regular_bounded_file "${organization_canonical_path}" 262144; then
  printf 'canonical organization-control configuration is invalid\n' >&2
  exit 2
fi

if ! jq -e '
  keys == [
    "access",
    "api_version",
    "authentication",
    "automation_credentials",
    "installed_applications",
    "lifecycle",
    "mapped_controls",
    "organization",
    "ownership_continuity",
    "private_attestation",
    "recovery",
    "reviewed_on",
    "schema_version",
    "scope",
    "verification"
  ] and
  .schema_version == "mcp-doctor.github-organization-controls/v2" and
  (.lifecycle == "activation" or .lifecycle == "verified") and
  .api_version == "2026-03-10" and
  (.reviewed_on | test("^[0-9]{4}-[0-9]{2}-[0-9]{2}$")) and
  .organization == "EnjoyableWork" and
  (.scope | keys) == [
    "distribution_repository",
    "organization_wide_controls",
    "private_repository_names_public",
    "project_repository",
    "repository_credential_scope",
    "separate_repository_credentials"
  ] and
  .scope.project_repository == "EnjoyableWork/mcp-doctor" and
  .scope.distribution_repository == "EnjoyableWork/homebrew-tap" and
  .scope.repository_credential_scope == [
    "EnjoyableWork/mcp-doctor",
    "EnjoyableWork/homebrew-tap"
  ] and
  .scope.organization_wide_controls == [
    "authentication",
    "membership",
    "member_privileges",
    "installed_applications",
    "personal_access_token_policy",
    "organization_actions_credentials",
    "ownership_continuity",
    "recovery"
  ] and
  .scope.separate_repository_credentials == "outside_mcp_doctor_assurance" and
  .scope.private_repository_names_public == false and
  .authentication.organization_two_factor_authentication_required == true and
  .authentication.secure_two_factor_methods_required == true and
  .authentication.members_with_two_factor_authentication_disabled == 0 and
  .authentication.members_with_insecure_two_factor_authentication == 0 and
  .authentication.outside_collaborators_with_two_factor_authentication_disabled == 0 and
  .authentication.outside_collaborators_with_insecure_two_factor_authentication == 0 and
  .authentication.factor_inventory_private == true and
  .access.exact_member_count == 1 and
  .access.exact_owner_count == 1 and
  .access.exact_member_count == .access.exact_owner_count and
  .access.outside_collaborator_count == 0 and
  .access.pending_invitation_count == 0 and
  .access.default_repository_permission == "none" and
  .access.manual_permission_assignment_required == true and
  .access.non_owner_direct_admin_identity_count == 0 and
  .access.member_privileges == {
    "members_can_create_repositories": false,
    "members_can_create_public_repositories": false,
    "members_can_create_private_repositories": false,
    "members_can_create_internal_repositories": false,
    "members_can_create_pages": false,
    "members_can_create_public_pages": false,
    "members_can_create_private_pages": false,
    "members_can_create_teams": false,
    "members_can_delete_issues": false,
    "members_can_delete_repositories": false,
    "members_can_change_repo_visibility": false,
    "members_can_fork_private_repositories": false
  } and
  .access.outside_collaborator_invitations == {
    "required_authority": "owners_only",
    "native_restriction_availability": "github_enterprise_cloud_only",
    "observed_organization_plan": "free",
    "observed_members_can_invite_outside_collaborators": true,
    "compensating_control": "all_repository_administrators_are_organization_owners",
    "recheck_triggers": [
      "organization_plan",
      "members_can_invite_outside_collaborators",
      "member_or_owner_count",
      "outside_collaborator_or_pending_invitation_count",
      "non_owner_direct_admin_identity_count"
    ]
  } and
  (.installed_applications | keys) == [
    "access_requests",
    "all_repository_access_allowed",
    "approved_installation_count",
    "approved_inventory_location",
    "approved_inventory_public",
    "installation_authority",
    "inventory_fields",
    "new_or_changed_installation_requires_review",
    "repository_selection",
    "review_interval_days"
  ] and
  .installed_applications.installation_authority == "owners_only" and
  .installed_applications.access_requests == "owner_review_required" and
  .installed_applications.repository_selection == "selected_only" and
  .installed_applications.all_repository_access_allowed == false and
  .installed_applications.approved_inventory_location == "private_attestation" and
  .installed_applications.approved_inventory_public == false and
  .installed_applications.inventory_fields == [
    "app_id",
    "app_slug",
    "permissions",
    "repositories",
    "repository_selection",
    "suspended"
  ] and
  .installed_applications.review_interval_days == 90 and
  .installed_applications.new_or_changed_installation_requires_review == true and
  ((.lifecycle == "activation" and
    .installed_applications.approved_installation_count == null and
    .recovery.latest_exercise_on == null) or
   (.lifecycle == "verified" and
    (.installed_applications.approved_installation_count | type == "number" and . >= 0) and
    (.recovery.latest_exercise_on | test("^[0-9]{4}-[0-9]{2}-[0-9]{2}$")))) and
  .automation_credentials.normal_automation_credentials == [
    "github_actions_job_token",
    "github_oidc",
    "github_app_installation_token"
  ] and
  .automation_credentials.classic_personal_access_token_access == "blocked" and
  .automation_credentials.fine_grained_personal_access_tokens == {
    "access": "exception_only",
    "approval": "owner_required",
    "maximum_lifetime_days": 30,
    "repository_selection": "exact",
    "permissions": "minimum",
    "automation_use": "prohibited"
  } and
  .automation_credentials.interactive_administration_credentials == [
    "approved_oauth_app_token",
    "exceptional_fine_grained_personal_access_token"
  ] and
  .automation_credentials.organization == {
    "actions_secrets": 0,
    "actions_variables": 0,
    "dependabot_secrets": 0,
    "webhooks": 0,
    "self_hosted_runners": 0
  } and
  (.automation_credentials.repositories | map(.repository)) == [
    "EnjoyableWork/mcp-doctor",
    "EnjoyableWork/homebrew-tap"
  ] and
  all(.automation_credentials.repositories[];
    (keys == [
      "actions_secrets",
      "actions_variables",
      "codespaces_secrets",
      "dependabot_secrets",
      "deploy_keys",
      "environment_secrets",
      "repository",
      "webhooks",
      "write_deploy_keys"
    ]) and
    .actions_secrets == 0 and
    .actions_variables == 0 and
    .codespaces_secrets == 0 and
    .dependabot_secrets == 0 and
    .environment_secrets == 0 and
    .deploy_keys == 0 and
    .write_deploy_keys == 0 and
    .webhooks == 0
  ) and
  .automation_credentials.separate_project_repository_credentials == {
    "state": "outside_project_assurance",
    "must_not_access_in_scope_repositories": true,
    "organization_wide_credentials_remain_in_scope": true,
    "project_specific_assurance_not_claimed": true
  } and
  .ownership_continuity == {
    "model": "single_owner_with_explicit_residual_risk",
    "shared_accounts_allowed": false,
    "residual_risk_accepted": true,
    "second_owner_policy": "redecide_when_a_trusted_active_operator_is_available",
    "nominal_owner_prohibited": true
  } and
  (.recovery | keys) == [
    "after_owner_or_authentication_factor_change",
    "cadence_months",
    "exercise",
    "latest_exercise_on",
    "maximum_age_days",
    "prohibited_public_evidence",
    "public_evidence_fields",
    "required_result"
  ] and
  .recovery.exercise == "real_private_independent_recovery_path" and
  .recovery.cadence_months == 6 and
  .recovery.maximum_age_days == 184 and
  .recovery.after_owner_or_authentication_factor_change == true and
  .recovery.required_result == "PASS" and
  .recovery.public_evidence_fields == ["date", "owner_count", "scope", "result"] and
  .recovery.prohibited_public_evidence == [
    "identities",
    "authentication_factor_inventory",
    "recovery_material"
  ] and
  .private_attestation.schema_version ==
    "mcp-doctor.github-organization-private-attestation/v1" and
  (.private_attestation | keys) == [
    "maximum_age_days",
    "required_assertions",
    "schema_version"
  ] and
  .private_attestation.maximum_age_days == 31 and
  .private_attestation.required_assertions == [
    "secure_two_factor_methods_enforced",
    "billing_manager_count",
    "billing_managers_use_secure_two_factor_methods",
    "github_app_installation_restricted_to_owners",
    "app_access_requests_receive_owner_review",
    "oauth_app_access_restrictions_enabled",
    "classic_personal_access_token_access_blocked",
    "fine_grained_personal_access_token_approval_required",
    "fine_grained_personal_access_token_maximum_lifetime_days",
    "operator_credential_reviewed_for_current_verification",
    "application_inventory_reviewed",
    "repository_credential_inventory_reviewed",
    "single_owner_residual_risk_accepted",
    "no_shared_or_unattended_human_accounts",
    "recovery_exercise_passed",
    "recovery_material_rotated_if_consumed",
    "identities_or_recovery_material_recorded"
  ] and
  .verification.script == "scripts/verify-organization-controls.sh" and
  (.verification | keys) == [
    "api_limitations",
    "limits",
    "private_attestation_required",
    "public_output_fields",
    "rehearsal",
    "script",
    "source_ref"
  ] and
  .verification.rehearsal == "scripts/rehearse-organization-controls.sh" and
  .verification.source_ref == "main" and
  .verification.private_attestation_required == true and
  .verification.limits == {
    "maximum_api_requests": 128,
    "maximum_total_seconds": 900,
    "maximum_connect_seconds": 10,
    "maximum_request_seconds": 30,
    "maximum_response_bytes": 4194304,
    "maximum_organization_repositories": 32,
    "maximum_installations": 16,
    "maximum_repositories_per_installation": 32,
    "maximum_direct_collaborators_per_repository": 99,
    "maximum_environments_per_repository": 16
  } and
  .verification.public_output_fields == [
    "date",
    "canonical_sha256",
    "source_sha",
    "result"
  ] and
  .verification.api_limitations == [
    "secure_two_factor_method_enforcement_setting",
    "billing_manager_membership_and_two_factor_state",
    "github_app_installation_authority_setting",
    "app_access_request_setting",
    "application_need_and_owner_decision",
    "oauth_app_access_restriction_setting",
    "personal_access_token_policy_settings",
    "operator_credential_need_and_lifetime",
    "private_recovery_execution"
  ] and
  .mapped_controls == ["OSPS-AC-01.01", "OSPS-AC-02.01"]
' "${organization_canonical_path}" >/dev/null 2>&1; then
  printf 'canonical organization-control configuration is invalid\n' >&2
  exit 2
fi

organization_canonical_hash="$(organization_sha256_file "${organization_canonical_path}")"
if [[ "${organization_verification_mode}" == "fixture" ]]; then
  organization_verification_date="${organization_verification_date_override}"
else
  organization_verification_date="$(date -u +%F)"
fi
if [[ ! "${organization_verification_date}" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] ||
  ! organization_date_epoch "${organization_verification_date}" >/dev/null; then
  printf 'organization-control verification date is invalid\n' >&2
  exit 2
fi
organization_source_sha="unresolved"

organization_report_failure() {
  trap - ERR
  if [[ "${organization_verification_mode}" == "fixture" ]]; then
    printf 'mode=fixture date=%s canonical_sha256=%s source_sha=%s result=FAIL\n' \
      "${organization_verification_date}" \
      "${organization_canonical_hash}" \
      "${organization_source_sha}"
  else
    printf 'date=%s canonical_sha256=%s source_sha=%s result=FAIL\n' \
      "${organization_verification_date}" \
      "${organization_canonical_hash}" \
      "${organization_source_sha}"
  fi
  exit 1
}

trap organization_report_failure ERR

organization_temp_parent="${TMPDIR:-/tmp}"
organization_work_prefix="${organization_temp_parent%/}/mcp-doctor-organization-controls."
umask 077
organization_work_directory="$(mktemp -d "${organization_work_prefix}XXXXXX")"

organization_cleanup() {
  if [[ "${organization_work_directory}" != "${organization_work_prefix}"* ]]; then
    return 1
  fi
  if [[ -d "${organization_work_directory}" ]]; then
    rm -rf -- "${organization_work_directory}"
  fi
}
trap organization_cleanup EXIT

if [[ "$(jq -er '.lifecycle' "${organization_canonical_path}" 2>/dev/null)" != "verified" ]]; then
  organization_report_failure
fi

if ! organization_regular_bounded_file "${organization_private_attestation}" 65536 ||
  ! organization_private_file_mode_is_restricted "${organization_private_attestation}"; then
  organization_report_failure
fi

organization_attestation_observed_on="$(
  jq -er '.observed_on' "${organization_private_attestation}" 2>/dev/null
)" || organization_report_failure
organization_approved_application_inventory_sha="$(
  jq -er '.application_inventory.inventory_sha256' \
    "${organization_private_attestation}" 2>/dev/null
)" || organization_report_failure
organization_recovery_exercised_on="$(
  jq -er '.recovery.exercised_on' "${organization_private_attestation}" 2>/dev/null
)" || organization_report_failure
organization_latest_recovery="$(
  jq -er '.recovery.latest_exercise_on' "${organization_canonical_path}" 2>/dev/null
)" || organization_report_failure

if ! jq -e \
  --arg schema "$(jq -er '.private_attestation.schema_version' "${organization_canonical_path}")" \
  --arg organization "$(jq -er '.organization' "${organization_canonical_path}")" \
  --arg observed_on "${organization_attestation_observed_on}" \
  --arg recovery_on "${organization_latest_recovery}" \
  --arg inventory_sha "${organization_approved_application_inventory_sha}" \
  --argjson installation_count "$(jq -er '.installed_applications.approved_installation_count' "${organization_canonical_path}")" \
  --argjson owner_count "$(jq -er '.access.exact_owner_count' "${organization_canonical_path}")" \
  --argjson token_days "$(jq -er '.automation_credentials.fine_grained_personal_access_tokens.maximum_lifetime_days' "${organization_canonical_path}")" '
    .schema_version == $schema and
    keys == [
      "application_inventory",
      "assertions",
      "observed_on",
      "organization",
      "recovery",
      "schema_version"
    ] and
    .organization == $organization and
    .observed_on == $observed_on and
    ($inventory_sha | test("^[0-9a-f]{64}$")) and
    .application_inventory == {
      "installation_count": $installation_count,
      "inventory_sha256": $inventory_sha,
      "reviewed": true
    } and
    .assertions == {
      "secure_two_factor_methods_enforced": true,
      "billing_manager_count": 0,
      "billing_managers_use_secure_two_factor_methods": true,
      "github_app_installation_restricted_to_owners": true,
      "app_access_requests_receive_owner_review": true,
      "oauth_app_access_restrictions_enabled": true,
      "classic_personal_access_token_access_blocked": true,
      "fine_grained_personal_access_token_approval_required": true,
      "fine_grained_personal_access_token_maximum_lifetime_days": $token_days,
      "operator_credential_reviewed_for_current_verification": true,
      "application_inventory_reviewed": true,
      "repository_credential_inventory_reviewed": true,
      "single_owner_residual_risk_accepted": true,
      "no_shared_or_unattended_human_accounts": true,
      "recovery_exercise_passed": true,
      "recovery_material_rotated_if_consumed": true,
      "identities_or_recovery_material_recorded": false
    } and
    .recovery == {
      "exercised_on": $recovery_on,
      "owner_count": $owner_count,
      "scope": "organization_account_recovery",
      "result": "PASS",
      "independent_path_succeeded": true,
      "recovery_material_rotated_if_consumed": true,
      "identities_or_recovery_material_recorded": false
    }
  ' "${organization_private_attestation}" >/dev/null 2>&1; then
  organization_report_failure
fi

organization_today_epoch="$(organization_date_epoch "${organization_verification_date}")" ||
  organization_report_failure
organization_observed_epoch="$(organization_date_epoch "${organization_attestation_observed_on}")" ||
  organization_report_failure
organization_recovery_epoch="$(organization_date_epoch "${organization_recovery_exercised_on}")" ||
  organization_report_failure
organization_reviewed_epoch="$(
  organization_date_epoch "$(jq -er '.reviewed_on' "${organization_canonical_path}")"
)" || organization_report_failure
organization_attestation_max_age="$((
  $(jq -er '.private_attestation.maximum_age_days' "${organization_canonical_path}") * 86400
))"
organization_recovery_max_age="$((
  $(jq -er '.recovery.maximum_age_days' "${organization_canonical_path}") * 86400
))"
organization_review_max_age="$((
  $(jq -er '.installed_applications.review_interval_days' "${organization_canonical_path}") * 86400
))"

if ((organization_observed_epoch > organization_today_epoch)) ||
  ((organization_today_epoch - organization_observed_epoch > organization_attestation_max_age)) ||
  ((organization_recovery_epoch > organization_today_epoch)) ||
  ((organization_today_epoch - organization_recovery_epoch > organization_recovery_max_age)) ||
  ((organization_reviewed_epoch > organization_today_epoch)) ||
  ((organization_today_epoch - organization_reviewed_epoch > organization_review_max_age)); then
  organization_report_failure
fi

organization_projection="${organization_work_directory}/projection.json"

if [[ -n "${organization_fixture_path}" ]]; then
  if [[ "${MCP_DOCTOR_ORGANIZATION_FIXTURE:-}" != "1" ]] ||
    ! organization_regular_bounded_file "${organization_fixture_path}" 1048576; then
    organization_report_failure
  fi
  if ! jq -e 'type == "object"' "${organization_fixture_path}" >/dev/null 2>&1; then
    organization_report_failure
  fi
  cp -- "${organization_fixture_path}" "${organization_projection}"
  organization_source_sha="$(jq -er '.source_sha' "${organization_projection}" 2>/dev/null)" ||
    organization_report_failure
  if [[ "${organization_source_sha}" != "1111111111111111111111111111111111111111" ]]; then
    organization_report_failure
  fi
else
  organization_gh() {
    env \
      -u ALL_PROXY -u all_proxy \
      -u HTTP_PROXY -u http_proxy \
      -u HTTPS_PROXY -u https_proxy \
      -u NO_PROXY -u no_proxy \
      GH_HOST=github.com GH_PROMPT_DISABLED=1 GH_PAGER=cat gh "$@"
  }

  if ! organization_gh auth status --hostname github.com >/dev/null 2>&1; then
    organization_report_failure
  fi

  organization_api_version="$(jq -er '.api_version' "${organization_canonical_path}")"
  organization_name="$(jq -er '.organization' "${organization_canonical_path}")"
  organization_token="$(
    organization_gh auth token --hostname github.com 2>/dev/null
  )" || organization_report_failure
  if [[ ! "${organization_token}" =~ ^[A-Za-z0-9_]+$ ]] ||
    ((${#organization_token} > 512)); then
    unset organization_token
    organization_report_failure
  fi
  case "${organization_token}" in
    ghp_*) organization_operator_credential_type="classic_personal_access_token" ;;
    github_pat_*) organization_operator_credential_type="fine_grained_personal_access_token" ;;
    gho_*) organization_operator_credential_type="oauth_app_token" ;;
    ghu_*) organization_operator_credential_type="github_app_user_token" ;;
    ghs_*) organization_operator_credential_type="github_app_installation_token" ;;
    *) organization_operator_credential_type="unclassified" ;;
  esac

  organization_token_config="${organization_work_directory}/curl-token.conf"
  printf 'header = "Authorization: Bearer %s"\n' \
    "${organization_token}" >"${organization_token_config}"
  chmod 0600 "${organization_token_config}"
  unset organization_token

  organization_api_request_count=0
  organization_api_request_max="$(
    jq -er '.verification.limits.maximum_api_requests' "${organization_canonical_path}"
  )"
  organization_api_started_epoch="$(date -u +%s)"
  organization_api_total_max="$(
    jq -er '.verification.limits.maximum_total_seconds' "${organization_canonical_path}"
  )"
  organization_api_deadline_epoch="$((
    organization_api_started_epoch + organization_api_total_max
  ))"
  organization_api_connect_max="$(
    jq -er '.verification.limits.maximum_connect_seconds' "${organization_canonical_path}"
  )"
  organization_api_request_seconds_max="$(
    jq -er '.verification.limits.maximum_request_seconds' "${organization_canonical_path}"
  )"
  organization_api_response_max="$(
    jq -er '.verification.limits.maximum_response_bytes' "${organization_canonical_path}"
  )"
  organization_repository_max="$(
    jq -er '.verification.limits.maximum_organization_repositories' \
      "${organization_canonical_path}"
  )"
  organization_installation_max="$(
    jq -er '.verification.limits.maximum_installations' "${organization_canonical_path}"
  )"
  organization_installation_repository_max="$(
    jq -er '.verification.limits.maximum_repositories_per_installation' \
      "${organization_canonical_path}"
  )"
  organization_direct_collaborator_max="$(
    jq -er '.verification.limits.maximum_direct_collaborators_per_repository' \
      "${organization_canonical_path}"
  )"
  organization_environment_max="$(
    jq -er '.verification.limits.maximum_environments_per_repository' \
      "${organization_canonical_path}"
  )"

  organization_api_get() {
    local organization_endpoint="$1"
    local organization_destination="$2"
    local organization_now
    local organization_remaining
    local organization_request_seconds
    local organization_connect_seconds

    organization_api_request_count="$((organization_api_request_count + 1))"
    organization_now="$(date -u +%s)"
    organization_remaining="$((organization_api_deadline_epoch - organization_now))"
    if ((organization_api_request_count > organization_api_request_max)) ||
      ((organization_remaining <= 0)); then
      return 1
    fi
    organization_request_seconds="${organization_api_request_seconds_max}"
    if ((organization_remaining < organization_request_seconds)); then
      organization_request_seconds="${organization_remaining}"
    fi
    organization_connect_seconds="${organization_api_connect_max}"
    if ((organization_request_seconds < organization_connect_seconds)); then
      organization_connect_seconds="${organization_request_seconds}"
    fi

    env \
      -u ALL_PROXY -u all_proxy \
      -u AWS_CA_BUNDLE \
      -u CURL_CA_BUNDLE -u CURL_HOME \
      -u HTTP_PROXY -u http_proxy \
      -u HTTPS_PROXY -u https_proxy \
      -u NETRC \
      -u NO_PROXY -u no_proxy \
      -u REQUESTS_CA_BUNDLE \
      -u SSL_CERT_DIR -u SSL_CERT_FILE -u SSLKEYLOGFILE \
      curl --disable \
      --silent \
      --show-error \
      --fail \
      --globoff \
      --request GET \
      --proto '=https' \
      --proxy '' \
      --retry 0 \
      --connect-timeout "${organization_connect_seconds}" \
      --max-time "${organization_request_seconds}" \
      --max-filesize "${organization_api_response_max}" \
      --config "${organization_token_config}" \
      --header 'Accept: application/vnd.github+json' \
      --header "X-GitHub-Api-Version: ${organization_api_version}" \
      --header 'User-Agent: mcp-doctor-organization-control-verifier/0.1' \
      --url "https://api.github.com/${organization_endpoint}" \
      >"${organization_destination}" 2>/dev/null
  }

  organization_api_get \
    "repos/$(jq -er '.scope.project_repository' "${organization_canonical_path}")/commits/main" \
    "${organization_work_directory}/source.json"
  organization_source_sha="$(
    jq -er '.sha' "${organization_work_directory}/source.json" 2>/dev/null
  )" || organization_report_failure
  organization_local_sha="$(git -C "${organization_repository_root}" rev-parse HEAD 2>/dev/null)" ||
    organization_report_failure
  if [[ "${organization_source_sha}" != "${organization_local_sha}" ]] ||
    [[ "$(git -C "${organization_repository_root}" branch --show-current 2>/dev/null)" != "main" ]] ||
    [[ -n "$(git -C "${organization_repository_root}" status --short 2>/dev/null)" ]]; then
    organization_report_failure
  fi

  organization_api_get "orgs/${organization_name}" \
    "${organization_work_directory}/organization.json"
  organization_api_get "orgs/${organization_name}/members?role=all&per_page=100" \
    "${organization_work_directory}/members.json"
  organization_api_get "orgs/${organization_name}/members?role=admin&per_page=100" \
    "${organization_work_directory}/owners.json"
  organization_api_get "orgs/${organization_name}/members?filter=2fa_disabled&role=all&per_page=100" \
    "${organization_work_directory}/members-disabled.json"
  organization_api_get "orgs/${organization_name}/members?filter=2fa_insecure&role=all&per_page=100" \
    "${organization_work_directory}/members-insecure.json"
  organization_api_get "orgs/${organization_name}/outside_collaborators?per_page=100" \
    "${organization_work_directory}/outside.json"
  organization_api_get "orgs/${organization_name}/outside_collaborators?filter=2fa_disabled&per_page=100" \
    "${organization_work_directory}/outside-disabled.json"
  organization_api_get "orgs/${organization_name}/outside_collaborators?filter=2fa_insecure&per_page=100" \
    "${organization_work_directory}/outside-insecure.json"
  organization_api_get "orgs/${organization_name}/invitations?per_page=100" \
    "${organization_work_directory}/invitations.json"
  organization_api_get "orgs/${organization_name}/installations?per_page=100" \
    "${organization_work_directory}/installations.json"
  organization_api_get "orgs/${organization_name}/repos?type=all&per_page=100" \
    "${organization_work_directory}/repositories.json"
  organization_api_get "orgs/${organization_name}/actions/secrets?per_page=100" \
    "${organization_work_directory}/organization-actions-secrets.json"
  organization_api_get "orgs/${organization_name}/actions/variables?per_page=100" \
    "${organization_work_directory}/organization-actions-variables.json"
  organization_api_get "orgs/${organization_name}/dependabot/secrets?per_page=100" \
    "${organization_work_directory}/organization-dependabot-secrets.json"
  organization_api_get "orgs/${organization_name}/hooks?per_page=100" \
    "${organization_work_directory}/organization-hooks.json"
  organization_api_get "orgs/${organization_name}/actions/runners?per_page=100" \
    "${organization_work_directory}/organization-runners.json"

  if ! jq -e --argjson maximum "${organization_installation_max}" '
    (.total_count | type == "number" and . >= 0 and . <= $maximum) and
    (.installations | type == "array") and
    ((.installations | length) == .total_count)
  ' "${organization_work_directory}/installations.json" >/dev/null 2>&1; then
    organization_report_failure
  fi
  if ! jq -e --argjson maximum "${organization_repository_max}" \
    'type == "array" and length <= $maximum' \
    "${organization_work_directory}/repositories.json" >/dev/null 2>&1; then
    organization_report_failure
  fi

  organization_application_lines="${organization_work_directory}/application-lines.jsonl"
  : >"${organization_application_lines}"
  while IFS= read -r organization_installation; do
    organization_installation_id="$(
      jq -er '.id' <<<"${organization_installation}" 2>/dev/null
    )" || organization_report_failure
    if [[ ! "${organization_installation_id}" =~ ^[0-9]+$ ]]; then
      organization_report_failure
    fi
    organization_installation_repositories="${organization_work_directory}/installation-${organization_installation_id}.json"
    organization_api_get \
      "user/installations/${organization_installation_id}/repositories?per_page=100" \
      "${organization_installation_repositories}"
    if ! jq -e --argjson maximum "${organization_installation_repository_max}" '
      (.total_count | type == "number" and . >= 0 and . <= $maximum) and
      (.repositories | type == "array") and
      ((.repositories | length) == .total_count)
    ' "${organization_installation_repositories}" >/dev/null 2>&1; then
      organization_report_failure
    fi
    jq -nc \
      --argjson installation "${organization_installation}" \
      --slurpfile repositories "${organization_installation_repositories}" '
        {
          app_id: $installation.app_id,
          app_slug: $installation.app_slug,
          permissions: $installation.permissions,
          repositories: ($repositories[0].repositories | map(.full_name) | sort),
          repository_selection: $installation.repository_selection,
          suspended: ($installation.suspended_at != null)
        }
      ' >>"${organization_application_lines}" 2>/dev/null
  done < <(jq -c '.installations[]' "${organization_work_directory}/installations.json")
  jq -s 'sort_by(.app_id)' "${organization_application_lines}" \
    >"${organization_work_directory}/application-inventory.json" 2>/dev/null

  organization_nonowner_admin_ids="${organization_work_directory}/nonowner-admin-ids.jsonl"
  : >"${organization_nonowner_admin_ids}"
  while IFS= read -r organization_repository_name; do
    if [[ ! "${organization_repository_name}" =~ ^EnjoyableWork/[A-Za-z0-9_.-]+$ ]] ||
      ((${#organization_repository_name} > 113)); then
      organization_report_failure
    fi
    organization_repository_key="$(
      printf '%s' "${organization_repository_name}" | organization_sha256_file /dev/stdin 2>/dev/null || true
    )"
    if [[ ! "${organization_repository_key}" =~ ^[0-9a-f]{64}$ ]]; then
      organization_report_failure
    fi
    organization_collaborators="${organization_work_directory}/collaborators-${organization_repository_key}.json"
    organization_api_get \
      "repos/${organization_repository_name}/collaborators?affiliation=direct&per_page=100" \
      "${organization_collaborators}"
    if ! jq -e --argjson maximum "${organization_direct_collaborator_max}" \
      'type == "array" and length <= $maximum and
       all(.[];
         (.id | type == "number") and
         (.role_name | type == "string") and
         (.permissions.admin | type == "boolean")
       )' \
      "${organization_collaborators}" >/dev/null 2>&1; then
      organization_report_failure
    fi
    jq -r --slurpfile owners "${organization_work_directory}/owners.json" '
      .[] | .id as $id |
      select(.role_name == "admin" or .permissions.admin == true) |
      select(([ $owners[0][].id ] | index($id)) == null) |
      $id
    ' "${organization_collaborators}" >>"${organization_nonowner_admin_ids}" 2>/dev/null
  done < <(jq -r '.[].full_name' "${organization_work_directory}/repositories.json")
  organization_nonowner_admin_count="$(
    sort -u "${organization_nonowner_admin_ids}" | sed '/^$/d' | wc -l | tr -d '[:space:]'
  )"

  organization_collect_repository_credentials() {
    local organization_repository_name="$1"
    local organization_repository_index="$2"
    local organization_prefix="${organization_work_directory}/repository-${organization_repository_index}"
    local organization_environment_secret_count=0
    local organization_environment_index=0
    local organization_environment_json
    local organization_environment_name
    local organization_environment_encoded
    local organization_environment_secrets

    organization_api_get "repos/${organization_repository_name}/actions/secrets?per_page=100" \
      "${organization_prefix}-actions-secrets.json"
    organization_api_get "repos/${organization_repository_name}/actions/variables?per_page=100" \
      "${organization_prefix}-actions-variables.json"
    organization_api_get "repos/${organization_repository_name}/codespaces/secrets?per_page=100" \
      "${organization_prefix}-codespaces-secrets.json"
    organization_api_get "repos/${organization_repository_name}/dependabot/secrets?per_page=100" \
      "${organization_prefix}-dependabot-secrets.json"
    organization_api_get "repos/${organization_repository_name}/environments?per_page=100" \
      "${organization_prefix}-environments.json"
    organization_api_get "repos/${organization_repository_name}/keys?per_page=100" \
      "${organization_prefix}-keys.json"
    organization_api_get "repos/${organization_repository_name}/hooks?per_page=100" \
      "${organization_prefix}-hooks.json"

    if ! jq -e --argjson maximum "${organization_environment_max}" '
      (.total_count | type == "number" and . >= 0 and . <= $maximum) and
      (.environments | type == "array") and
      ((.environments | length) == .total_count)
    ' "${organization_prefix}-environments.json" >/dev/null 2>&1; then
      organization_report_failure
    fi

    while IFS= read -r organization_environment_json; do
      organization_environment_name="$(
        jq -er 'select(type == "string" and length > 0 and length <= 255)' \
          <<<"${organization_environment_json}" 2>/dev/null
      )" || organization_report_failure
      organization_environment_index="$((organization_environment_index + 1))"
      organization_environment_encoded="$(
        printf '%s' "${organization_environment_name}" | jq -sRr @uri
      )"
      organization_environment_secrets="${organization_prefix}-environment-${organization_environment_index}.json"
      organization_api_get \
        "repos/${organization_repository_name}/environments/${organization_environment_encoded}/secrets?per_page=100" \
        "${organization_environment_secrets}"
      organization_environment_secret_count="$((
        organization_environment_secret_count +
          $(jq -er '.total_count' "${organization_environment_secrets}")
      ))"
    done < <(jq -c '.environments[].name' "${organization_prefix}-environments.json")

    jq -nc \
      --arg repository "${organization_repository_name}" \
      --argjson environment_secrets "${organization_environment_secret_count}" \
      --slurpfile actions_secrets "${organization_prefix}-actions-secrets.json" \
      --slurpfile actions_variables "${organization_prefix}-actions-variables.json" \
      --slurpfile codespaces_secrets "${organization_prefix}-codespaces-secrets.json" \
      --slurpfile dependabot_secrets "${organization_prefix}-dependabot-secrets.json" \
      --slurpfile deploy_keys "${organization_prefix}-keys.json" \
      --slurpfile webhooks "${organization_prefix}-hooks.json" '
        {
          repository: $repository,
          actions_secrets: $actions_secrets[0].total_count,
          actions_variables: $actions_variables[0].total_count,
          codespaces_secrets: $codespaces_secrets[0].total_count,
          dependabot_secrets: $dependabot_secrets[0].total_count,
          environment_secrets: $environment_secrets,
          deploy_keys: ($deploy_keys[0] | length),
          write_deploy_keys: ([$deploy_keys[0][] | select(.read_only == false)] | length),
          webhooks: ($webhooks[0] | length)
        }
      '
  }

  organization_repository_credential_lines="${organization_work_directory}/repository-credentials.jsonl"
  : >"${organization_repository_credential_lines}"
  organization_repository_index=0
  while IFS= read -r organization_repository_name; do
    organization_repository_index="$((organization_repository_index + 1))"
    organization_collect_repository_credentials \
      "${organization_repository_name}" "${organization_repository_index}" \
      >>"${organization_repository_credential_lines}" 2>/dev/null
  done < <(jq -r '.automation_credentials.repositories[].repository' \
    "${organization_canonical_path}")
  jq -s 'sort_by(.repository)' "${organization_repository_credential_lines}" \
    >"${organization_work_directory}/repository-credentials.json" 2>/dev/null

  jq -n \
    --arg source_sha "${organization_source_sha}" \
    --arg operator_credential_type "${organization_operator_credential_type}" \
    --argjson nonowner_admin_count "${organization_nonowner_admin_count}" \
    --slurpfile canonical "${organization_canonical_path}" \
    --slurpfile organization "${organization_work_directory}/organization.json" \
    --slurpfile members "${organization_work_directory}/members.json" \
    --slurpfile owners "${organization_work_directory}/owners.json" \
    --slurpfile members_disabled "${organization_work_directory}/members-disabled.json" \
    --slurpfile members_insecure "${organization_work_directory}/members-insecure.json" \
    --slurpfile outside "${organization_work_directory}/outside.json" \
    --slurpfile outside_disabled "${organization_work_directory}/outside-disabled.json" \
    --slurpfile outside_insecure "${organization_work_directory}/outside-insecure.json" \
    --slurpfile invitations "${organization_work_directory}/invitations.json" \
    --slurpfile application_inventory "${organization_work_directory}/application-inventory.json" \
    --slurpfile organization_actions_secrets "${organization_work_directory}/organization-actions-secrets.json" \
    --slurpfile organization_actions_variables "${organization_work_directory}/organization-actions-variables.json" \
    --slurpfile organization_dependabot_secrets "${organization_work_directory}/organization-dependabot-secrets.json" \
    --slurpfile organization_hooks "${organization_work_directory}/organization-hooks.json" \
    --slurpfile organization_runners "${organization_work_directory}/organization-runners.json" \
    --slurpfile repositories "${organization_work_directory}/repository-credentials.json" '
      {
        source_sha: $source_sha,
        organization: {
          login: $organization[0].login,
          plan: $organization[0].plan.name,
          two_factor_requirement_enabled:
            $organization[0].two_factor_requirement_enabled,
          default_repository_permission:
            $organization[0].default_repository_permission,
          member_privileges:
            ($organization[0] | {
              members_can_create_repositories,
              members_can_create_public_repositories,
              members_can_create_private_repositories,
              members_can_create_internal_repositories,
              members_can_create_pages,
              members_can_create_public_pages,
              members_can_create_private_pages,
              members_can_create_teams,
              members_can_delete_issues,
              members_can_delete_repositories,
              members_can_change_repo_visibility,
              members_can_fork_private_repositories
            }),
          outside_collaborator_invitations: {
            observed_members_can_invite_outside_collaborators:
              $organization[0].members_can_invite_outside_collaborators
          }
        },
        membership: {
          members: ($members[0] | length),
          owners: ($owners[0] | length),
          outside_collaborators: ($outside[0] | length),
          pending_invitations: ($invitations[0] | length),
          members_2fa_disabled: ($members_disabled[0] | length),
          members_2fa_insecure: ($members_insecure[0] | length),
          outside_2fa_disabled: ($outside_disabled[0] | length),
          outside_2fa_insecure: ($outside_insecure[0] | length),
          nonowner_direct_admin_identities: $nonowner_admin_count
        },
        applications: {inventory: $application_inventory[0]},
        organization_credentials: {
          actions_secrets: $organization_actions_secrets[0].total_count,
          actions_variables: $organization_actions_variables[0].total_count,
          dependabot_secrets: $organization_dependabot_secrets[0].total_count,
          webhooks: ($organization_hooks[0] | length),
          self_hosted_runners: $organization_runners[0].total_count
        },
        repositories: $repositories[0],
        operator_credential_type: $operator_credential_type
      }
    ' >"${organization_projection}" 2>/dev/null
fi

if [[ ! "${organization_source_sha}" =~ ^[0-9a-f]{40}$ ]]; then
  organization_report_failure
fi

organization_application_inventory="${organization_work_directory}/normalized-application-inventory.json"
if ! jq -S '.applications.inventory | sort_by(.app_id)' "${organization_projection}" \
  >"${organization_application_inventory}" 2>/dev/null; then
  organization_report_failure
fi
organization_application_inventory_sha="$(
  organization_sha256_file "${organization_application_inventory}"
)" || organization_report_failure

if ! jq -e \
  --slurpfile canonical "${organization_canonical_path}" \
  --arg inventory_sha "${organization_application_inventory_sha}" \
  --arg approved_inventory_sha "${organization_approved_application_inventory_sha}" '
    (.source_sha | test("^[0-9a-f]{40}$")) and
    .organization.login == $canonical[0].organization and
    .organization.plan ==
      $canonical[0].access.outside_collaborator_invitations.observed_organization_plan and
    .organization.two_factor_requirement_enabled ==
      $canonical[0].authentication.organization_two_factor_authentication_required and
    .organization.default_repository_permission ==
      $canonical[0].access.default_repository_permission and
    .organization.member_privileges == $canonical[0].access.member_privileges and
    .organization.outside_collaborator_invitations == {
      "observed_members_can_invite_outside_collaborators":
        $canonical[0].access.outside_collaborator_invitations.observed_members_can_invite_outside_collaborators
    } and
    .membership == {
      "members": $canonical[0].access.exact_member_count,
      "owners": $canonical[0].access.exact_owner_count,
      "outside_collaborators": $canonical[0].access.outside_collaborator_count,
      "pending_invitations": $canonical[0].access.pending_invitation_count,
      "members_2fa_disabled":
        $canonical[0].authentication.members_with_two_factor_authentication_disabled,
      "members_2fa_insecure":
        $canonical[0].authentication.members_with_insecure_two_factor_authentication,
      "outside_2fa_disabled":
        $canonical[0].authentication.outside_collaborators_with_two_factor_authentication_disabled,
      "outside_2fa_insecure":
        $canonical[0].authentication.outside_collaborators_with_insecure_two_factor_authentication,
      "nonowner_direct_admin_identities":
        $canonical[0].access.non_owner_direct_admin_identity_count
    } and
    (.applications.inventory | length) ==
      $canonical[0].installed_applications.approved_installation_count and
    all(.applications.inventory[];
      .repository_selection == "selected" and
      .suspended == false and
      (.app_id | type == "number") and
      (.app_slug | type == "string" and length > 0) and
      (.permissions | type == "object") and
      (.repositories | type == "array" and length > 0)
    ) and
    $inventory_sha == $approved_inventory_sha and
    .organization_credentials == $canonical[0].automation_credentials.organization and
    (.repositories | sort_by(.repository)) ==
      ($canonical[0].automation_credentials.repositories | sort_by(.repository)) and
    (.operator_credential_type == "oauth_app_token" or
     .operator_credential_type == "github_app_user_token" or
     .operator_credential_type == "github_app_installation_token" or
     .operator_credential_type == "fine_grained_personal_access_token")
  ' "${organization_projection}" >/dev/null 2>&1; then
  organization_report_failure
fi

if ! organization_cleanup; then
  organization_report_failure
fi
trap - EXIT

if [[ "${organization_verification_mode}" == "fixture" ]]; then
  printf 'mode=fixture date=%s canonical_sha256=%s source_sha=%s result=PASS\n' \
    "${organization_verification_date}" \
    "${organization_canonical_hash}" \
    "${organization_source_sha}"
else
  printf 'date=%s canonical_sha256=%s source_sha=%s result=PASS\n' \
    "${organization_verification_date}" \
    "${organization_canonical_hash}" \
    "${organization_source_sha}"
fi
