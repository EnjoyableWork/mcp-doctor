#!/usr/bin/env bash

set -euo pipefail

chatgpt_verify_script_directory="$({ cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd; })"
chatgpt_verify_repository_root="$({ cd -- "${chatgpt_verify_script_directory}/.." && pwd; })"

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 <stable version> [ChatGPT ZIP]" >&2
  exit 2
fi

chatgpt_verify_version=$1
chatgpt_verify_archive=${2:-}
chatgpt_verify_skill_root="${chatgpt_verify_repository_root}/.agents/skills/mcp-doctor"
chatgpt_verify_openai="${chatgpt_verify_skill_root}/agents/openai.yaml"
chatgpt_verify_icon="${chatgpt_verify_skill_root}/assets/icon.svg"

if [[ ! "${chatgpt_verify_version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "ChatGPT skill version must be a stable semantic version" >&2
  exit 1
fi

"${chatgpt_verify_script_directory}/verify-agent-skill.sh" \
  "${chatgpt_verify_version}" >/dev/null
if [[ ! -f "${chatgpt_verify_openai}" || -L "${chatgpt_verify_openai}" ]] ||
  [[ ! -f "${chatgpt_verify_icon}" || -L "${chatgpt_verify_icon}" ]]; then
  echo "ChatGPT skill source is missing its metadata or icon" >&2
  exit 1
fi

chatgpt_verify_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print tolower($1) }'
  else
    shasum -a 256 "$1" | awk '{ print tolower($1) }'
  fi
}

if [[ -z "${chatgpt_verify_archive}" ]]; then
  printf 'Verified ChatGPT skill source for mcp-doctor %s\n' \
    "${chatgpt_verify_version}"
  exit 0
fi

chatgpt_verify_expected_archive="mcp-doctor-chatgpt-skill-v${chatgpt_verify_version}.zip"
if [[ "$(basename -- "${chatgpt_verify_archive}")" != "${chatgpt_verify_expected_archive}" ]] ||
  [[ ! -f "${chatgpt_verify_archive}" || -L "${chatgpt_verify_archive}" ]]; then
  echo "ChatGPT skill ZIP identity is missing, symbolic, or version-mismatched" >&2
  exit 1
fi
if ! command -v unzip >/dev/null 2>&1; then
  echo "ChatGPT skill ZIP verification requires unzip" >&2
  exit 1
fi
chatgpt_verify_archive_bytes=$(stat -c '%s' "${chatgpt_verify_archive}" 2>/dev/null ||
  stat -f '%z' "${chatgpt_verify_archive}")
chatgpt_verify_expanded_bytes=$(LC_ALL=C unzip -l "${chatgpt_verify_archive}" |
  awk 'END { print $1 }')
if [[ ! "${chatgpt_verify_archive_bytes}" =~ ^[0-9]+$ ]] ||
  [[ ! "${chatgpt_verify_expanded_bytes}" =~ ^[0-9]+$ ]] ||
  (( chatgpt_verify_archive_bytes > 131072 )) ||
  (( chatgpt_verify_expanded_bytes > 131072 )); then
  echo "ChatGPT skill ZIP exceeds the compressed or expanded byte limit" >&2
  exit 1
fi

chatgpt_verify_entries=$(unzip -Z1 "${chatgpt_verify_archive}" | LC_ALL=C sort)
chatgpt_verify_expected_entries=$'SKILL.md\nagents/\nagents/openai.yaml\nassets/\nassets/icon.svg'
if [[ "${chatgpt_verify_entries}" != "${chatgpt_verify_expected_entries}" ]]; then
  echo "ChatGPT skill ZIP has an unexpected file, directory, or wrapper folder" >&2
  exit 1
fi
if unzip -Z1 "${chatgpt_verify_archive}" |
  grep -E '(^/|(^|/)\.\.(/|$)|\\)' >/dev/null; then
  echo "ChatGPT skill ZIP contains an unsafe path" >&2
  exit 1
fi

chatgpt_verify_temp_parent=${TMPDIR:-/tmp}
chatgpt_verify_temp_prefix="${chatgpt_verify_temp_parent%/}/mcp-doctor-chatgpt-skill-verify."
chatgpt_verify_temp=$(mktemp -d "${chatgpt_verify_temp_prefix}XXXXXX")
cleanup_chatgpt_verify() {
  if [[ "${chatgpt_verify_temp}" != "${chatgpt_verify_temp_prefix}"* ]]; then
    echo "refusing to remove an unexpected ChatGPT skill verification path" >&2
    return 1
  fi
  if [[ -d "${chatgpt_verify_temp}" ]]; then
    rm -rf -- "${chatgpt_verify_temp}"
  fi
}
trap cleanup_chatgpt_verify EXIT

unzip -qq "${chatgpt_verify_archive}" -d "${chatgpt_verify_temp}"
if find "${chatgpt_verify_temp}" -type l -print -quit | grep -q .; then
  echo "ChatGPT skill ZIP contains a symbolic link" >&2
  exit 1
fi
for chatgpt_verify_file in SKILL.md agents/openai.yaml assets/icon.svg; do
  if ! cmp --silent \
    "${chatgpt_verify_skill_root}/${chatgpt_verify_file}" \
    "${chatgpt_verify_temp}/${chatgpt_verify_file}"; then
    echo "ChatGPT skill ZIP does not contain the canonical exact bytes" >&2
    exit 1
  fi
  chatgpt_verify_mode=$(stat -c '%a' \
    "${chatgpt_verify_temp}/${chatgpt_verify_file}" 2>/dev/null ||
    stat -f '%Lp' "${chatgpt_verify_temp}/${chatgpt_verify_file}")
  if [[ "${chatgpt_verify_mode}" != 644 ]]; then
    echo "ChatGPT skill ZIP file mode is not 0644" >&2
    exit 1
  fi
done

printf 'Verified mcp-doctor ChatGPT skill ZIP sha256:%s\n' \
  "$(chatgpt_verify_sha256 "${chatgpt_verify_archive}")"
