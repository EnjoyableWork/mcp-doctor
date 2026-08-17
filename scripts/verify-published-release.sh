#!/usr/bin/env bash

set -euo pipefail

published_release_script_directory="$({ cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd; })"

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <published asset directory> <stable version>" >&2
  exit 2
fi

published_release_directory=$1
published_release_version=$2

if [[ ! "${published_release_version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "published release version must be a stable semantic version" >&2
  exit 1
fi
if [[ ! -d "${published_release_directory}" || -L "${published_release_directory}" ]]; then
  echo "published release asset directory is missing or symbolic" >&2
  exit 1
fi

published_release_directory=$(cd -- "${published_release_directory}" && pwd)
published_release_prefix="mcp-doctor-v${published_release_version}"
published_release_payload=(
  "mcp-doctor-${published_release_version}.crate"
  mcp-doctor.rb
  "${published_release_prefix}-aarch64-unknown-linux-gnu.tar.gz"
  "${published_release_prefix}-aarch64-unknown-linux-gnu.spdx.json"
  "${published_release_prefix}-x86_64-unknown-linux-gnu.tar.gz"
  "${published_release_prefix}-x86_64-unknown-linux-gnu.spdx.json"
)
IFS=. read -r published_release_major published_release_minor published_release_patch \
  <<<"${published_release_version}"
if (( 10#${published_release_major} > 0 )) ||
  (( 10#${published_release_minor} > 3 )) ||
  (( 10#${published_release_minor} == 3 && 10#${published_release_patch} >= 2 )); then
  published_release_payload+=(
    "mcp-doctor-agent-skill-v${published_release_version}.tar.gz"
  )
fi
published_release_expected=(SHA256SUMS "${published_release_payload[@]}")

published_release_observed=$(
  find "${published_release_directory}" -maxdepth 1 -type f -print \
    | sed 's#^.*/##' \
    | LC_ALL=C sort
)
published_release_sorted_expected=$(printf '%s\n' "${published_release_expected[@]}" | LC_ALL=C sort)
if [[ "${published_release_observed}" != "${published_release_sorted_expected}" ]]; then
  echo "published release is incomplete or contains unexpected files" >&2
  exit 1
fi

published_release_entries=$(
  sed -n 's/^[[:xdigit:]]\{64\}  \([^/][^/]*\)$/\1/p' \
    "${published_release_directory}/SHA256SUMS"
)
published_release_expected_entries=$(printf '%s\n' "${published_release_payload[@]}" | LC_ALL=C sort)
published_release_line_count=$(wc -l <"${published_release_directory}/SHA256SUMS" | tr -d '[:space:]')
published_release_expected_count=$(printf '%s\n' "${published_release_expected_entries}" | wc -l | tr -d '[:space:]')
if [[ "${published_release_line_count}" != "${published_release_expected_count}" ]] ||
  [[ "$(printf '%s\n' "${published_release_entries}" | LC_ALL=C sort)" != "${published_release_expected_entries}" ]]; then
  echo "SHA256SUMS does not contain the exact release payload" >&2
  exit 1
fi

(
  cd -- "${published_release_directory}"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum --check SHA256SUMS >/dev/null
  else
    shasum -a 256 --check SHA256SUMS >/dev/null
  fi
)

published_release_temp_parent=${TMPDIR:-/tmp}
published_release_temp_prefix="${published_release_temp_parent%/}/mcp-doctor-published-release."
published_release_temp=$(mktemp -d "${published_release_temp_prefix}XXXXXX")
cleanup_published_release() {
  if [[ "${published_release_temp}" != "${published_release_temp_prefix}"* ]]; then
    echo "refusing to remove an unexpected published-release path" >&2
    return 1
  fi
  if [[ -d "${published_release_temp}" ]]; then
    rm -rf -- "${published_release_temp}"
  fi
}
trap cleanup_published_release EXIT

for published_release_asset in "${published_release_payload[@]}"; do
  cp -- \
    "${published_release_directory}/${published_release_asset}" \
    "${published_release_temp}/"
done
"${published_release_script_directory}/verify-release-assets.sh" \
  "${published_release_temp}" \
  "${published_release_version}"
cmp --silent \
  "${published_release_directory}/SHA256SUMS" \
  "${published_release_temp}/SHA256SUMS"
