#!/usr/bin/env bash

set -euo pipefail

agent_skill_script_directory="$({ cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd; })"
agent_skill_repository_root="$({ cd -- "${agent_skill_script_directory}/.." && pwd; })"

if [[ $# -lt 1 || $# -gt 3 ]]; then
  echo "usage: $0 <stable version> [companion archive [source root]]" >&2
  exit 2
fi

agent_skill_version=$1
agent_skill_archive=${2:-}
agent_skill_source_root=${3:-${agent_skill_repository_root}}
agent_skill_source="${agent_skill_source_root}/.agents/skills/mcp-doctor/SKILL.md"
agent_skill_directory="${agent_skill_source_root}/.agents/skills/mcp-doctor"
agent_skill_guide="${agent_skill_source_root}/docs/agents.md"

if [[ ! "${agent_skill_version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "agent skill version must be a stable semantic version" >&2
  exit 1
fi
if [[ ! -d "${agent_skill_source_root}" || -L "${agent_skill_source_root}" ]]; then
  echo "Agent Skill source root must be one real directory" >&2
  exit 1
fi
if [[ ! -f "${agent_skill_source}" || -L "${agent_skill_source}" ]]; then
  echo "canonical Agent Skill must be one regular non-symbolic file" >&2
  exit 1
fi

agent_skill_observed_entries=$(
  find "${agent_skill_directory}" -mindepth 1 -maxdepth 2 -print \
    | sed "s#^${agent_skill_directory}/##" \
    | LC_ALL=C sort
)
if [[ "${agent_skill_observed_entries}" != SKILL.md ]]; then
  echo "canonical Agent Skill must contain only SKILL.md" >&2
  exit 1
fi

if [[ "$(sed -n '1p' "${agent_skill_source}")" != --- ]] ||
  [[ "$(sed -n '2p' "${agent_skill_source}")" != "name: mcp-doctor" ]] ||
  [[ "$(sed -n '4p' "${agent_skill_source}")" != --- ]]; then
  echo "Agent Skill frontmatter is not the exact portable four-line form" >&2
  exit 1
fi
agent_skill_description=$(sed -n '3s/^description: //p' "${agent_skill_source}")
if [[ -z "${agent_skill_description}" ]] ||
  (( ${#agent_skill_description} > 1024 )); then
  echo "Agent Skill description is missing or exceeds 1024 characters" >&2
  exit 1
fi
if sed -n '2,3p' "${agent_skill_source}" | grep -Ev \
  '^(name|description): ' >/dev/null; then
  echo "Agent Skill frontmatter contains an unsupported field" >&2
  exit 1
fi
if (( $(wc -l <"${agent_skill_source}") > 500 )); then
  echo "Agent Skill exceeds the portable 500-line guidance" >&2
  exit 1
fi

# Backticks in these fixed strings are literal Markdown contract delimiters.
# shellcheck disable=SC2016
for agent_skill_contract in \
  'mcp-doctor --version' \
  'mcp-doctor capabilities --format json' \
  "Continue only with \`mcp-doctor ${agent_skill_version}\`" \
  'mcp-doctor inspect --format json -- <exact-command> <literal-arguments>' \
  'mcp-doctor inspect --format json <exact-endpoint>' \
  'mcp-doctor.report/v1' \
  'primary_diagnosis' \
  'independent_findings' \
  'skip_reason' \
  'blocked_by' \
  'wrap the target in `sh -c`' \
  'rerun exactly the same' \
  'Never run `check`, `break`, or `reject`' \
  'do not install or upgrade software'; do
  if ! grep -F -- "${agent_skill_contract}" "${agent_skill_source}" >/dev/null; then
    echo "Agent Skill is missing a required command or safety contract" >&2
    exit 1
  fi
done

for agent_skill_forbidden in \
  'allowed-tools:' \
  'permission:' \
  'permissions:' \
  'curl ' \
  'wget ' \
  'brew install' \
  'cargo install' \
  'mcp-doctor check --' \
  'mcp-doctor break --' \
  'mcp-doctor reject --' \
  'cat .env' \
  'source .env' \
  'printenv' \
  'set -x'; do
  if grep -F -- "${agent_skill_forbidden}" "${agent_skill_source}" >/dev/null; then
    echo "Agent Skill contains a forbidden permission, installer, active command, or disclosure pattern" >&2
    exit 1
  fi
done

agent_skill_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print tolower($1) }'
  else
    shasum -a 256 "$1" | awk '{ print tolower($1) }'
  fi
}

agent_skill_source_hash=$(agent_skill_sha256 "${agent_skill_source}")
if [[ ! -f "${agent_skill_guide}" || -L "${agent_skill_guide}" ]] ||
  ! grep -F -- "Canonical \`SKILL.md\` | SHA-256 \`${agent_skill_source_hash}\`" \
    "${agent_skill_guide}" >/dev/null; then
  echo "coding-agent guide does not bind the canonical SKILL.md digest" >&2
  exit 1
fi

if [[ -z "${agent_skill_archive}" ]]; then
  printf 'Verified canonical mcp-doctor Agent Skill sha256:%s\n' \
    "${agent_skill_source_hash}"
  exit 0
fi

agent_skill_expected_archive="mcp-doctor-agent-skill-v${agent_skill_version}.tar.gz"
if [[ "$(basename -- "${agent_skill_archive}")" != "${agent_skill_expected_archive}" ]] ||
  [[ ! -f "${agent_skill_archive}" || -L "${agent_skill_archive}" ]]; then
  echo "Agent Skill archive identity is missing, symbolic, or version-mismatched" >&2
  exit 1
fi

agent_skill_archive_entries=$(tar -tzf "${agent_skill_archive}" | LC_ALL=C sort)
if [[ "${agent_skill_archive_entries}" != $'mcp-doctor/\nmcp-doctor/SKILL.md' ]]; then
  echo "Agent Skill archive must contain only mcp-doctor/SKILL.md" >&2
  exit 1
fi
if tar -tzf "${agent_skill_archive}" | grep -E '(^/|(^|/)\.\.(/|$))' >/dev/null; then
  echo "Agent Skill archive contains an unsafe path" >&2
  exit 1
fi

agent_skill_temp_parent=${TMPDIR:-/tmp}
agent_skill_temp_prefix="${agent_skill_temp_parent%/}/mcp-doctor-agent-skill-verify."
agent_skill_temp=$(mktemp -d "${agent_skill_temp_prefix}XXXXXX")
cleanup_agent_skill_verify() {
  if [[ "${agent_skill_temp}" != "${agent_skill_temp_prefix}"* ]]; then
    echo "refusing to remove an unexpected Agent Skill verification path" >&2
    return 1
  fi
  if [[ -d "${agent_skill_temp}" ]]; then
    rm -rf -- "${agent_skill_temp}"
  fi
}
trap cleanup_agent_skill_verify EXIT

tar -xzf "${agent_skill_archive}" -C "${agent_skill_temp}"
if [[ -L "${agent_skill_temp}/mcp-doctor/SKILL.md" ]] ||
  ! cmp --silent \
    "${agent_skill_source}" \
    "${agent_skill_temp}/mcp-doctor/SKILL.md"; then
  echo "Agent Skill archive does not contain the canonical exact bytes" >&2
  exit 1
fi
if [[ "$(stat -c '%a' "${agent_skill_temp}/mcp-doctor/SKILL.md" 2>/dev/null ||
  stat -f '%Lp' "${agent_skill_temp}/mcp-doctor/SKILL.md")" != 644 ]]; then
  echo "Agent Skill archive file mode is not 0644" >&2
  exit 1
fi

printf 'Verified mcp-doctor Agent Skill archive sha256:%s canonical-sha256:%s\n' \
  "$(agent_skill_sha256 "${agent_skill_archive}")" \
  "${agent_skill_source_hash}"
