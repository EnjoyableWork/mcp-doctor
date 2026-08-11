#!/usr/bin/env bash
set -euo pipefail

script_dir="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
repository_root="$(dirname -- "$script_dir")"
canonical_path="${1:-$repository_root/.github/rulesets/main.json}"

if [[ $# -gt 1 ]]; then
  printf 'usage: %s [canonical-config]\n' "$0" >&2
  exit 2
fi

for required_command in curl jq; do
  if ! command -v "$required_command" >/dev/null 2>&1; then
    printf 'required command is unavailable: %s\n' "$required_command" >&2
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
  .default_branch == "main" and
  (.api_version | type == "string" and length > 0)
' "$canonical_path" >/dev/null; then
  printf 'canonical main-protection configuration is invalid\n' >&2
  exit 2
fi

repository="$(jq -er '.repository' "$canonical_path")"
api_version="$(jq -er '.api_version' "$canonical_path")"
ruleset_name="$(jq -er '.ruleset.name' "$canonical_path")"
default_branch="$(jq -er '.default_branch' "$canonical_path")"
config_hash="$(canonical_hash)"

umask 077
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/mcp-doctor-public-protection.XXXXXX")"
cleanup() {
  rm -rf -- "$work_dir"
}
trap cleanup EXIT

repo_path="$work_dir/repository.json"
rulesets_path="$work_dir/rulesets.json"
ruleset_path="$work_dir/ruleset.json"
effective_path="$work_dir/effective.json"

fetch_public_json() {
  local api_path="$1"
  local destination="$2"

  if ! env \
    -u GH_TOKEN \
    -u GITHUB_TOKEN \
    -u GH_ENTERPRISE_TOKEN \
    -u GITHUB_ENTERPRISE_TOKEN \
    curl --disable \
      --fail \
      --silent \
      --show-error \
      --proto '=https' \
      --tlsv1.2 \
      --connect-timeout 10 \
      --max-time 30 \
      --max-filesize 1048576 \
      --max-redirs 0 \
      --noproxy '*' \
      --header 'Accept: application/vnd.github+json' \
      --header "X-GitHub-Api-Version: $api_version" \
      --header 'User-Agent: mcp-doctor-main-protection-verifier/1' \
      --output "$destination" \
      "https://api.github.com/$api_path"; then
    printf 'credential-free GitHub readback failed\n' >&2
    return 1
  fi

  if [[ ! -s "$destination" ]] || (( $(wc -c <"$destination") > 1048576 )); then
    printf 'credential-free GitHub readback exceeded its response boundary\n' >&2
    return 1
  fi
  jq -e . "$destination" >/dev/null 2>&1
}

normalize_ruleset() {
  local source_path="$1"
  local destination="$2"

  jq -S '
    def normalize_rule:
      if .type == "pull_request" then
        {
          type,
          parameters: {
            allowed_merge_methods: (.parameters.allowed_merge_methods | sort),
            dismiss_stale_reviews_on_push: .parameters.dismiss_stale_reviews_on_push,
            dismissal_restriction: {
              allowed_actors: (.parameters.dismissal_restriction.allowed_actors | sort_by(.type, .id)),
              enabled: .parameters.dismissal_restriction.enabled
            },
            require_code_owner_review: .parameters.require_code_owner_review,
            require_last_push_approval: .parameters.require_last_push_approval,
            required_approving_review_count: .parameters.required_approving_review_count,
            required_review_thread_resolution: .parameters.required_review_thread_resolution,
            required_reviewers: (.parameters.required_reviewers | sort_by(.reviewer.type, .reviewer.id))
          }
        }
      elif .type == "required_status_checks" then
        {
          type,
          parameters: {
            do_not_enforce_on_create: .parameters.do_not_enforce_on_create,
            required_status_checks: (
              .parameters.required_status_checks | sort_by(.context, .integration_id)
            ),
            strict_required_status_checks_policy: .parameters.strict_required_status_checks_policy
          }
        }
      elif has("parameters") then
        {type, parameters}
      else
        {type}
      end;

    {
      name,
      target,
      enforcement,
      conditions: {
        ref_name: {
          include: (.conditions.ref_name.include | sort),
          exclude: (.conditions.ref_name.exclude | sort)
        }
      },
      rules: (.rules | map(normalize_rule) | sort_by(.type))
    }
  ' "$source_path" >"$destination"
}

normalize_rule_list() {
  local source_path="$1"
  local destination="$2"

  jq -S '
    def normalize_rule:
      if .type == "pull_request" then
        {
          type,
          parameters: {
            allowed_merge_methods: (.parameters.allowed_merge_methods | sort),
            dismiss_stale_reviews_on_push: .parameters.dismiss_stale_reviews_on_push,
            dismissal_restriction: {
              allowed_actors: (.parameters.dismissal_restriction.allowed_actors | sort_by(.type, .id)),
              enabled: .parameters.dismissal_restriction.enabled
            },
            require_code_owner_review: .parameters.require_code_owner_review,
            require_last_push_approval: .parameters.require_last_push_approval,
            required_approving_review_count: .parameters.required_approving_review_count,
            required_review_thread_resolution: .parameters.required_review_thread_resolution,
            required_reviewers: (.parameters.required_reviewers | sort_by(.reviewer.type, .reviewer.id))
          }
        }
      elif .type == "required_status_checks" then
        {
          type,
          parameters: {
            do_not_enforce_on_create: .parameters.do_not_enforce_on_create,
            required_status_checks: (
              .parameters.required_status_checks | sort_by(.context, .integration_id)
            ),
            strict_required_status_checks_policy: .parameters.strict_required_status_checks_policy
          }
        }
      elif has("parameters") then
        {type, parameters}
      else
        {type}
      end;

    map(normalize_rule) | sort_by(.type)
  ' "$source_path" >"$destination"
}

validate_ruleset_shape() {
  local source_path="$1"

  jq -e '
    def exact_keys($expected): (keys | sort) == ($expected | sort);
    (.rules | length == 5) and
    (
      [.rules[].type] | sort == [
        "deletion",
        "non_fast_forward",
        "pull_request",
        "required_linear_history",
        "required_status_checks"
      ]
    ) and
    (
      .rules[] | select(.type == "pull_request") | .parameters |
      exact_keys([
        "allowed_merge_methods",
        "dismiss_stale_reviews_on_push",
        "dismissal_restriction",
        "require_code_owner_review",
        "require_last_push_approval",
        "required_approving_review_count",
        "required_review_thread_resolution",
        "required_reviewers"
      ])
    ) and
    (
      .rules[] | select(.type == "required_status_checks") | .parameters |
      exact_keys([
        "do_not_enforce_on_create",
        "required_status_checks",
        "strict_required_status_checks_policy"
      ])
    ) and
    all(
      .rules[] | select(
        .type == "deletion" or
        .type == "non_fast_forward" or
        .type == "required_linear_history"
      );
      exact_keys(["type"])
    )
  ' "$source_path" >/dev/null 2>&1
}

fetch_public_json "repos/$repository" "$repo_path"
fetch_public_json "repos/$repository/rulesets?includes_parents=true&per_page=100" "$rulesets_path"

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
  printf 'public ruleset projection does not match the canonical scope\n' >&2
  exit 1
fi

fetch_public_json "repos/$repository/rulesets/$ruleset_id?includes_parents=true" "$ruleset_path"
fetch_public_json "repos/$repository/rules/branches/$default_branch?per_page=100" "$effective_path"

canonical_ruleset_source="$work_dir/canonical-ruleset-source.json"
jq '.ruleset' "$canonical_path" >"$canonical_ruleset_source"

if ! validate_ruleset_shape "$canonical_ruleset_source" ||
  ! validate_ruleset_shape "$ruleset_path" ||
  ! jq -e 'has("bypass_actors") | not' "$ruleset_path" >/dev/null 2>&1; then
  printf 'public ruleset projection has an unexpected shape\n' >&2
  exit 1
fi

canonical_repo_projection="$work_dir/canonical-repository.json"
live_repo_projection="$work_dir/live-repository.json"
canonical_ruleset_projection="$work_dir/canonical-ruleset.json"
live_ruleset_projection="$work_dir/live-ruleset.json"
canonical_rules_projection="$work_dir/canonical-rules.json"
effective_rules_projection="$work_dir/effective-rules.json"

jq -S '{default_branch}' "$canonical_path" >"$canonical_repo_projection"
jq -S '{default_branch}' "$repo_path" >"$live_repo_projection"

normalize_ruleset "$canonical_ruleset_source" "$canonical_ruleset_projection"
normalize_ruleset "$ruleset_path" "$live_ruleset_projection"

if ! cmp -s "$canonical_repo_projection" "$live_repo_projection" ||
  ! cmp -s "$canonical_ruleset_projection" "$live_ruleset_projection"; then
  printf 'public main-protection projection has drifted from the canonical configuration\n' >&2
  exit 1
fi

if ! jq -e \
  --arg repository "$repository" \
  --argjson ruleset_id "$ruleset_id" '
    length == 5 and
    all(.[];
      .ruleset_source_type == "Repository" and
      .ruleset_source == $repository and
      .ruleset_id == $ruleset_id
    )
  ' "$effective_path" >/dev/null 2>&1; then
  printf 'effective main rules include an unexpected layer or source\n' >&2
  exit 1
fi

jq '.ruleset.rules' "$canonical_path" >"$work_dir/canonical-rules-source.json"
normalize_rule_list "$work_dir/canonical-rules-source.json" "$canonical_rules_projection"
normalize_rule_list "$effective_path" "$effective_rules_projection"

if ! cmp -s "$canonical_rules_projection" "$effective_rules_projection"; then
  printf 'effective main rules have drifted from the canonical configuration\n' >&2
  exit 1
fi

printf 'date=%s canonical_sha256=%s result=PASS\n' \
  "$(date -u +%F)" \
  "$config_hash"
