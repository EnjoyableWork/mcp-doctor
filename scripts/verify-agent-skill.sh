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
agent_skill_openai="${agent_skill_directory}/agents/openai.yaml"
agent_skill_icon="${agent_skill_directory}/assets/icon.svg"
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
  echo "canonical Agent Skill must contain one regular non-symbolic SKILL.md" >&2
  exit 1
fi

agent_skill_observed_entries=$(
  find "${agent_skill_directory}" -mindepth 1 -maxdepth 2 -print \
    | sed "s#^${agent_skill_directory}/##" \
    | LC_ALL=C sort
)
agent_skill_portable_entries=SKILL.md
agent_skill_chatgpt_entries=$'SKILL.md\nagents\nagents/openai.yaml\nassets\nassets/icon.svg'
if [[ "${agent_skill_observed_entries}" == "${agent_skill_portable_entries}" ]]; then
  agent_skill_profile=portable
elif [[ "${agent_skill_observed_entries}" == "${agent_skill_chatgpt_entries}" ]]; then
  agent_skill_profile=chatgpt
else
  echo "canonical Agent Skill has an unexpected file or directory" >&2
  exit 1
fi
if [[ "${agent_skill_profile}" == chatgpt ]] &&
  { [[ ! -f "${agent_skill_openai}" || -L "${agent_skill_openai}" ]] ||
    [[ ! -f "${agent_skill_icon}" || -L "${agent_skill_icon}" ]]; }; then
  echo "ChatGPT Agent Skill files must be regular and non-symbolic" >&2
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
if (( $(wc -l <"${agent_skill_source}") > 500 )) ||
  (( $(wc -c <"${agent_skill_source}") > 65536 )); then
  echo "Agent Skill exceeds the portable line or byte limit" >&2
  exit 1
fi

if [[ "${agent_skill_profile}" == chatgpt ]]; then
  for agent_skill_openai_contract in \
    'interface:' \
    '  display_name: "MCP Doctor"' \
    '  short_description: "Diagnose MCP servers before users do—from local to production"' \
    '  icon_small: "./assets/icon.svg"' \
    '  icon_large: "./assets/icon.svg"' \
    '  default_prompt: "Use $mcp-doctor to passively diagnose this exact MCP server target: [command or endpoint]."' \
    'policy:' \
    '  allow_implicit_invocation: true'; do
    if ! grep -Fx -- "${agent_skill_openai_contract}" "${agent_skill_openai}" >/dev/null; then
      echo "OpenAI skill metadata is missing a required interface or policy contract" >&2
      exit 1
    fi
  done
  agent_skill_short_description=$(sed -n \
    's/^  short_description: "\(.*\)"$/\1/p' "${agent_skill_openai}")
  if (( ${#agent_skill_short_description} < 25 || ${#agent_skill_short_description} > 64 )) ||
    (( $(wc -l <"${agent_skill_openai}") != 9 )) ||
    (( $(wc -c <"${agent_skill_openai}") > 4096 )) ||
    grep -Eq $'\t|^dependencies:|^  brand_color:' "${agent_skill_openai}"; then
    echo "OpenAI skill metadata is malformed, oversized, or declares unsupported metadata" >&2
    exit 1
  fi

  if (( $(wc -c <"${agent_skill_icon}") > 32768 )) ||
    ! grep -F '<svg ' "${agent_skill_icon}" >/dev/null ||
    ! grep -F 'xmlns="http://www.w3.org/2000/svg"' "${agent_skill_icon}" >/dev/null ||
    (( $(awk '{ count += gsub(/http:/, "") } END { print count }' "${agent_skill_icon}") != 1 )) ||
    ! grep -F 'viewBox="400 200 800 800"' "${agent_skill_icon}" >/dev/null ||
    ! grep -F 'fill="none"' "${agent_skill_icon}" >/dev/null ||
    ! grep -F '<path ' "${agent_skill_icon}" >/dev/null ||
    grep -Eiq '<(script|image|rect|circle|ellipse|polygon|polyline|foreignObject|iframe|object|embed)([[:space:]>])|<!DOCTYPE|<\?xml-stylesheet|@import|https:|data:|(^|[[:space:]])(href|xlink:href)[[:space:]]*=' \
      "${agent_skill_icon}"; then
    echo "Agent Skill icon is malformed, oversized, or contains external or executable content" >&2
    exit 1
  fi
fi

# Backticks in these fixed strings are literal Markdown contract delimiters.
# shellcheck disable=SC2016
for agent_skill_contract in \
  'mcp-doctor --version' \
  'mcp-doctor capabilities --format json' \
  'https://github.com/EnjoyableWork/mcp-doctor/tree/main/.agents/skills/mcp-doctor' \
  'https://smithery.ai/skills/enjoyable/mcp-doctor' \
  'Do not run a skill installer or registry command' \
  "cargo install mcp-doctor --version '=${agent_skill_version}' --locked" \
  'brew install --build-from-source EnjoyableWork/tap/mcp-doctor' \
  "https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v${agent_skill_version}" \
  'Do not choose a route, run either command' \
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
  'missing-CLI commands above are instructions for the user, not execution' \
  'install or upgrade software'; do
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

if (( $(grep -Fxc -- "cargo install mcp-doctor --version '=${agent_skill_version}' --locked" \
  "${agent_skill_source}") != 1 )) ||
  (( $(grep -Fxc -- 'brew install --build-from-source EnjoyableWork/tap/mcp-doctor' \
    "${agent_skill_source}") != 1 )); then
  echo "Agent Skill installation guidance is missing, duplicated, or not version-bound" >&2
  exit 1
fi

agent_skill_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print tolower($1) }'
  else
    shasum -a 256 "$1" | awk '{ print tolower($1) }'
  fi
}

agent_skill_source_hash=$(agent_skill_sha256 "${agent_skill_source}")
if [[ ! -f "${agent_skill_guide}" || -L "${agent_skill_guide}" ]]; then
  echo "coding-agent guide is missing or symbolic" >&2
  exit 1
fi
if [[ "${agent_skill_profile}" == chatgpt ]]; then
  agent_skill_openai_hash=$(agent_skill_sha256 "${agent_skill_openai}")
  agent_skill_icon_hash=$(agent_skill_sha256 "${agent_skill_icon}")
  if ! grep -F -- "Source \`SKILL.md\` | SHA-256 \`${agent_skill_source_hash}\`" \
    "${agent_skill_guide}" >/dev/null ||
    ! grep -F -- "Source \`agents/openai.yaml\` | SHA-256 \`${agent_skill_openai_hash}\`" \
      "${agent_skill_guide}" >/dev/null ||
    ! grep -F -- "Source \`assets/icon.svg\` | SHA-256 \`${agent_skill_icon_hash}\`" \
      "${agent_skill_guide}" >/dev/null; then
    echo "coding-agent guide does not bind the canonical skill bundle digests" >&2
    exit 1
  fi
elif ! grep -F -- "Canonical \`SKILL.md\` | SHA-256 \`${agent_skill_source_hash}\`" \
  "${agent_skill_guide}" >/dev/null &&
  ! grep -F -- "Published \`SKILL.md\` | SHA-256 \`${agent_skill_source_hash}\`" \
    "${agent_skill_guide}" >/dev/null; then
  echo "coding-agent guide does not bind the portable SKILL.md digest" >&2
  exit 1
fi

if [[ -z "${agent_skill_archive}" ]]; then
  if [[ "${agent_skill_profile}" == chatgpt ]]; then
    printf 'Verified canonical mcp-doctor Agent Skill skill-sha256:%s metadata-sha256:%s icon-sha256:%s\n' \
      "${agent_skill_source_hash}" \
      "${agent_skill_openai_hash}" \
      "${agent_skill_icon_hash}"
  else
    printf 'Verified portable mcp-doctor Agent Skill sha256:%s\n' \
      "${agent_skill_source_hash}"
  fi
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
