#!/usr/bin/env bash

set -euo pipefail

chatgpt_package_script_directory="$({ cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd; })"
chatgpt_package_repository_root="$({ cd -- "${chatgpt_package_script_directory}/.." && pwd; })"

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <stable version> <output directory>" >&2
  exit 2
fi

chatgpt_package_version=$1
chatgpt_package_output_directory=$2
chatgpt_package_epoch=${SOURCE_DATE_EPOCH:-}

if [[ ! "${chatgpt_package_version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "ChatGPT skill version must be a stable semantic version" >&2
  exit 1
fi
if [[ ! "${chatgpt_package_epoch}" =~ ^[0-9]+$ ]]; then
  echo "SOURCE_DATE_EPOCH must be the source commit timestamp" >&2
  exit 1
fi
if ! command -v zip >/dev/null 2>&1 || ! command -v unzip >/dev/null 2>&1; then
  echo "ChatGPT skill packaging requires zip and unzip" >&2
  exit 1
fi

"${chatgpt_package_script_directory}/verify-chatgpt-skill.sh" \
  "${chatgpt_package_version}" >/dev/null

mkdir -p -- "${chatgpt_package_output_directory}"
chatgpt_package_output_directory="$({ cd -- "${chatgpt_package_output_directory}" && pwd; })"
chatgpt_package_archive="mcp-doctor-chatgpt-skill-v${chatgpt_package_version}.zip"
chatgpt_package_archive_path="${chatgpt_package_output_directory}/${chatgpt_package_archive}"
if [[ -e "${chatgpt_package_archive_path}" || -L "${chatgpt_package_archive_path}" ]]; then
  echo "ChatGPT skill ZIP already exists" >&2
  exit 1
fi

chatgpt_package_temp_parent=${TMPDIR:-/tmp}
chatgpt_package_temp_prefix="${chatgpt_package_temp_parent%/}/mcp-doctor-chatgpt-skill-package."
chatgpt_package_temp=$(mktemp -d "${chatgpt_package_temp_prefix}XXXXXX")
chatgpt_package_stage="${chatgpt_package_temp}/stage"
chatgpt_package_archive_temp="${chatgpt_package_temp}/${chatgpt_package_archive}"
cleanup_chatgpt_package() {
  if [[ "${chatgpt_package_temp}" != "${chatgpt_package_temp_prefix}"* ]]; then
    echo "refusing to remove an unexpected ChatGPT skill packaging path" >&2
    return 1
  fi
  if [[ -d "${chatgpt_package_temp}" ]]; then
    rm -rf -- "${chatgpt_package_temp}"
  fi
}
trap cleanup_chatgpt_package EXIT

mkdir -m 0755 -- "${chatgpt_package_stage}"
mkdir -m 0755 -- "${chatgpt_package_stage}/agents"
mkdir -m 0755 -- "${chatgpt_package_stage}/assets"
install -m 0644 \
  "${chatgpt_package_repository_root}/.agents/skills/mcp-doctor/SKILL.md" \
  "${chatgpt_package_stage}/SKILL.md"
install -m 0644 \
  "${chatgpt_package_repository_root}/.agents/skills/mcp-doctor/agents/openai.yaml" \
  "${chatgpt_package_stage}/agents/openai.yaml"
install -m 0644 \
  "${chatgpt_package_repository_root}/.agents/skills/mcp-doctor/assets/icon.svg" \
  "${chatgpt_package_stage}/assets/icon.svg"

if chatgpt_package_timestamp=$(date -u -r "${chatgpt_package_epoch}" '+%Y%m%d%H%M.%S' 2>/dev/null); then
  :
elif chatgpt_package_timestamp=$(date -u -d "@${chatgpt_package_epoch}" '+%Y%m%d%H%M.%S' 2>/dev/null); then
  :
else
  echo "SOURCE_DATE_EPOCH is outside the supported date range" >&2
  exit 1
fi
TZ=UTC touch -t "${chatgpt_package_timestamp}" \
  "${chatgpt_package_stage}/SKILL.md" \
  "${chatgpt_package_stage}/agents" \
  "${chatgpt_package_stage}/agents/openai.yaml" \
  "${chatgpt_package_stage}/assets" \
  "${chatgpt_package_stage}/assets/icon.svg"

(
  cd -- "${chatgpt_package_stage}"
  zip -X -q -9 "${chatgpt_package_archive_temp}" \
    SKILL.md \
    agents/ \
    agents/openai.yaml \
    assets/ \
    assets/icon.svg
)

"${chatgpt_package_script_directory}/verify-chatgpt-skill.sh" \
  "${chatgpt_package_version}" \
  "${chatgpt_package_archive_temp}" >/dev/null
mv -- "${chatgpt_package_archive_temp}" "${chatgpt_package_archive_path}"
printf '%s\n' "${chatgpt_package_archive_path}"
