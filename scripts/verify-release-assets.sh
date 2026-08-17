#!/usr/bin/env bash

set -euo pipefail

release_verify_script_directory="$({ cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd; })"

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <asset directory> <stable version>" >&2
  exit 2
fi

release_asset_directory=$1
release_asset_version=$2

if [[ ! "${release_asset_version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "release asset version must be a stable semantic version" >&2
  exit 1
fi
if [[ ! -d "${release_asset_directory}" || -L "${release_asset_directory}" ]]; then
  echo "release asset directory is missing or symbolic" >&2
  exit 1
fi

release_asset_directory=$(cd -- "${release_asset_directory}" && pwd)
release_asset_prefix="mcp-doctor-v${release_asset_version}"
release_cargo_package="mcp-doctor-${release_asset_version}.crate"
release_agent_skill="mcp-doctor-agent-skill-v${release_asset_version}.tar.gz"
release_targets=(
  aarch64-unknown-linux-gnu
  x86_64-unknown-linux-gnu
)
release_expected_assets=(
  "${release_cargo_package}"
  mcp-doctor.rb
)
IFS=. read -r release_asset_major release_asset_minor release_asset_patch \
  <<<"${release_asset_version}"
release_asset_has_agent_skill=false
if (( 10#${release_asset_major} > 0 )) ||
  (( 10#${release_asset_minor} > 3 )) ||
  (( 10#${release_asset_minor} == 3 && 10#${release_asset_patch} >= 2 )); then
  release_asset_has_agent_skill=true
  release_expected_assets+=("${release_agent_skill}")
fi
for release_target in "${release_targets[@]}"; do
  release_expected_assets+=(
    "${release_asset_prefix}-${release_target}.tar.gz"
    "${release_asset_prefix}-${release_target}.spdx.json"
  )
done

release_observed_assets=$(
  find "${release_asset_directory}" -maxdepth 1 -type f -print \
    | sed 's#^.*/##' \
    | LC_ALL=C sort
)
release_sorted_expected_assets=$(printf '%s\n' "${release_expected_assets[@]}" | LC_ALL=C sort)
if [[ "${release_observed_assets}" != "${release_sorted_expected_assets}" ]]; then
  echo "release asset set is incomplete or contains unexpected files" >&2
  exit 1
fi

for release_target in "${release_targets[@]}"; do
  jq -e '
    .spdxVersion == "SPDX-2.3" and
    (.documentNamespace | type == "string") and
    (.packages | type == "array" and length > 0)
  ' "${release_asset_directory}/${release_asset_prefix}-${release_target}.spdx.json" >/dev/null
done

release_temp_parent=${TMPDIR:-/tmp}
release_temp_prefix="${release_temp_parent%/}/mcp-doctor-release-verify."
release_temp=$(mktemp -d "${release_temp_prefix}XXXXXX")
cleanup_release_verify() {
  if [[ "${release_temp}" != "${release_temp_prefix}"* ]]; then
    echo "refusing to remove an unexpected release verification path" >&2
    return 1
  fi
  if [[ -d "${release_temp}" ]]; then
    rm -rf -- "${release_temp}"
  fi
}
trap cleanup_release_verify EXIT

release_generated="${release_temp}/generated"
"${release_verify_script_directory}/generate-release-channels.sh" \
  "${release_asset_directory}/${release_cargo_package}" \
  "${release_asset_version}" \
  "${release_generated}" >/dev/null
cmp --silent \
  "${release_asset_directory}/${release_cargo_package}" \
  "${release_generated}/cargo/${release_cargo_package}"
cmp --silent \
  "${release_asset_directory}/mcp-doctor.rb" \
  "${release_generated}/homebrew/Formula/mcp-doctor.rb"

if [[ "${release_asset_has_agent_skill}" == true ]]; then
  release_agent_source_stage="${release_temp}/agent-source"
  mkdir -p -- "${release_agent_source_stage}"
  tar -xzf \
    "${release_asset_directory}/${release_cargo_package}" \
    -C "${release_agent_source_stage}"
  release_agent_source_root="${release_agent_source_stage}/mcp-doctor-${release_asset_version}"
  "${release_verify_script_directory}/verify-agent-skill.sh" \
    "${release_asset_version}" \
    "${release_asset_directory}/${release_agent_skill}" \
    "${release_agent_source_root}"
fi

release_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1"
  else
    shasum -a 256 "$1"
  fi
}

(
  cd -- "${release_asset_directory}"
  for release_expected_asset in "${release_expected_assets[@]}"; do
    release_sha256 "${release_expected_asset}"
  done | LC_ALL=C sort -k2 >SHA256SUMS
)
