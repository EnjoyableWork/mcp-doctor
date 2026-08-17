#!/usr/bin/env bash

set -euo pipefail

agent_package_script_directory="$({ cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd; })"
agent_package_repository_root="$({ cd -- "${agent_package_script_directory}/.." && pwd; })"

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <stable version> <output directory>" >&2
  exit 2
fi

agent_package_version=$1
agent_package_output_directory=$2
agent_package_epoch=${SOURCE_DATE_EPOCH:-}

if [[ ! "${agent_package_version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Agent Skill version must be a stable semantic version" >&2
  exit 1
fi
if [[ ! "${agent_package_epoch}" =~ ^[0-9]+$ ]]; then
  echo "SOURCE_DATE_EPOCH must be the release commit timestamp" >&2
  exit 1
fi
if ! tar --version 2>/dev/null | head -n 1 | grep -F 'GNU tar' >/dev/null; then
  echo "deterministic Agent Skill archives require GNU tar" >&2
  exit 1
fi

"${agent_package_script_directory}/verify-agent-skill.sh" \
  "${agent_package_version}" >/dev/null

mkdir -p -- "${agent_package_output_directory}"
agent_package_output_directory="$({ cd -- "${agent_package_output_directory}" && pwd; })"
agent_package_archive="mcp-doctor-agent-skill-v${agent_package_version}.tar.gz"
agent_package_archive_path="${agent_package_output_directory}/${agent_package_archive}"
if [[ -e "${agent_package_archive_path}" || -L "${agent_package_archive_path}" ]]; then
  echo "Agent Skill archive already exists" >&2
  exit 1
fi

agent_package_temp_parent=${TMPDIR:-/tmp}
agent_package_stage_prefix="${agent_package_temp_parent%/}/mcp-doctor-agent-skill-package."
agent_package_stage=$(mktemp -d "${agent_package_stage_prefix}XXXXXX")
agent_package_archive_temp=$(
  mktemp "${agent_package_output_directory}/.${agent_package_archive}.XXXXXX"
)
cleanup_agent_package() {
  if [[ "${agent_package_stage}" != "${agent_package_stage_prefix}"* ]]; then
    echo "refusing to remove an unexpected Agent Skill packaging path" >&2
    return 1
  fi
  if [[ -d "${agent_package_stage}" ]]; then
    rm -rf -- "${agent_package_stage}"
  fi
  if [[ -f "${agent_package_archive_temp}" ]]; then
    rm -f -- "${agent_package_archive_temp}"
  fi
}
trap cleanup_agent_package EXIT

mkdir -m 0755 -- "${agent_package_stage}/mcp-doctor"
install -m 0644 \
  "${agent_package_repository_root}/.agents/skills/mcp-doctor/SKILL.md" \
  "${agent_package_stage}/mcp-doctor/SKILL.md"

COPYFILE_DISABLE=1 tar \
  --sort=name \
  --format=ustar \
  --mtime="@${agent_package_epoch}" \
  --owner=0 \
  --group=0 \
  --numeric-owner \
  -C "${agent_package_stage}" \
  -cf - \
  mcp-doctor \
  | gzip -n -9 >"${agent_package_archive_temp}"

mv -- "${agent_package_archive_temp}" "${agent_package_archive_path}"
agent_package_archive_temp=
"${agent_package_script_directory}/verify-agent-skill.sh" \
  "${agent_package_version}" \
  "${agent_package_archive_path}" >/dev/null
printf '%s\n' "${agent_package_archive_path}"
