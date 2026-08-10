#!/usr/bin/env bash

set -euo pipefail

release_rehearsal_script_directory="$({ cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd; })"

if [[ $# -ne 4 ]]; then
  echo "usage: $0 <asset directory> <stable version> <release commit> <verified|synthetic-rehearsal>" >&2
  exit 2
fi

release_rehearsal_assets=$1
release_rehearsal_version=$2
release_rehearsal_commit=$3
release_rehearsal_evidence=$4

case "${release_rehearsal_evidence}" in
  verified | synthetic-rehearsal) ;;
  *)
    echo "release rehearsal evidence must be verified or synthetic-rehearsal" >&2
    exit 2
    ;;
esac

release_rehearsal_temp_parent=${TMPDIR:-/tmp}
release_rehearsal_temp_prefix="${release_rehearsal_temp_parent%/}/mcp-doctor-release-rehearsal."
release_rehearsal_temp=$(mktemp -d "${release_rehearsal_temp_prefix}XXXXXX")

cleanup_release_rehearsal() {
  if [[ "${release_rehearsal_temp}" != "${release_rehearsal_temp_prefix}"* ]]; then
    echo "refusing to remove an unexpected release-rehearsal path" >&2
    return 1
  fi
  if [[ -d "${release_rehearsal_temp}" ]]; then
    rm -rf -- "${release_rehearsal_temp}"
  fi
}
trap cleanup_release_rehearsal EXIT

release_rehearsal_manifest="${release_rehearsal_temp}/handoff.json"
release_rehearsal_package="${release_rehearsal_assets}/mcp-doctor-${release_rehearsal_version}.crate"
release_rehearsal_formula="${release_rehearsal_assets}/mcp-doctor.rb"

"${release_rehearsal_script_directory}/create-release-handoff.sh" \
  "${release_rehearsal_assets}" \
  "${release_rehearsal_version}" \
  "${release_rehearsal_commit}" \
  "${release_rehearsal_evidence}" \
  "${release_rehearsal_manifest}"
"${release_rehearsal_script_directory}/verify-release-handoff.sh" \
  "${release_rehearsal_manifest}" \
  "${release_rehearsal_version}" \
  "${release_rehearsal_evidence}" \
  "${release_rehearsal_package}" \
  "${release_rehearsal_formula}"

expect_release_rehearsal_rejection() {
  local description=$1
  shift
  if "$@" >/dev/null 2>&1; then
    echo "release rehearsal unexpectedly accepted ${description}" >&2
    exit 1
  fi
}

if [[ "${release_rehearsal_evidence}" == verified ]]; then
  release_rehearsal_bad_manifest="${release_rehearsal_temp}/unverified-handoff.json"
  jq '.provenance_verified = false' \
    "${release_rehearsal_manifest}" >"${release_rehearsal_bad_manifest}"
  expect_release_rehearsal_rejection \
    "an out-of-order handoff without verified provenance" \
    "${release_rehearsal_script_directory}/verify-release-handoff.sh" \
    "${release_rehearsal_bad_manifest}" \
    "${release_rehearsal_version}" \
    verified \
    "${release_rehearsal_package}" \
    "${release_rehearsal_formula}"
else
  expect_release_rehearsal_rejection \
    "synthetic evidence at the verified-publication boundary" \
    "${release_rehearsal_script_directory}/verify-release-handoff.sh" \
    "${release_rehearsal_manifest}" \
    "${release_rehearsal_version}" \
    verified \
    "${release_rehearsal_package}" \
    "${release_rehearsal_formula}"
fi

release_rehearsal_bad_package="${release_rehearsal_temp}/mcp-doctor-${release_rehearsal_version}.crate"
cp -- "${release_rehearsal_package}" "${release_rehearsal_bad_package}"
printf 'mismatched Cargo handoff\n' >>"${release_rehearsal_bad_package}"
expect_release_rehearsal_rejection \
  "mismatched Cargo bytes" \
  "${release_rehearsal_script_directory}/verify-release-handoff.sh" \
  "${release_rehearsal_manifest}" \
  "${release_rehearsal_version}" \
  "${release_rehearsal_evidence}" \
  "${release_rehearsal_bad_package}" \
  "${release_rehearsal_formula}"

release_rehearsal_bad_formula="${release_rehearsal_temp}/mcp-doctor.rb"
cp -- "${release_rehearsal_formula}" "${release_rehearsal_bad_formula}"
printf '# mismatched Homebrew handoff\n' >>"${release_rehearsal_bad_formula}"
expect_release_rehearsal_rejection \
  "mismatched Homebrew bytes" \
  "${release_rehearsal_script_directory}/verify-release-handoff.sh" \
  "${release_rehearsal_manifest}" \
  "${release_rehearsal_version}" \
  "${release_rehearsal_evidence}" \
  "${release_rehearsal_package}" \
  "${release_rehearsal_bad_formula}"

printf 'release handoff rehearsal passed for v%s\n' "${release_rehearsal_version}"
