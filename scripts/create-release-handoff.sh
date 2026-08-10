#!/usr/bin/env bash

set -euo pipefail

release_handoff_script_directory="$({ cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd; })"

if [[ $# -ne 5 ]]; then
  echo "usage: $0 <asset directory> <stable version> <release commit> <verified|synthetic-rehearsal> <output manifest>" >&2
  exit 2
fi

release_handoff_assets=$1
release_handoff_version=$2
release_handoff_commit=$3
release_handoff_evidence=$4
release_handoff_output=$5

"${release_handoff_script_directory}/validate-release-version.sh" \
  published \
  "v${release_handoff_version}" \
  "${release_handoff_version}" >/dev/null

if [[ ! "${release_handoff_commit}" =~ ^[[:xdigit:]]{40}$ ]]; then
  echo "release handoff commit must be a full Git commit SHA" >&2
  exit 1
fi

case "${release_handoff_evidence}" in
  verified)
    release_handoff_environment=release
    release_handoff_immutable=true
    release_handoff_provenance=true
    ;;
  synthetic-rehearsal)
    release_handoff_environment=synthetic-rehearsal
    release_handoff_immutable=false
    release_handoff_provenance=false
    ;;
  *)
    echo "release handoff evidence must be verified or synthetic-rehearsal" >&2
    exit 2
    ;;
esac
if [[ -e "${release_handoff_output}" || -L "${release_handoff_output}" ]]; then
  echo "release handoff output already exists" >&2
  exit 1
fi

"${release_handoff_script_directory}/verify-published-release.sh" \
  "${release_handoff_assets}" \
  "${release_handoff_version}"

release_handoff_assets=$(cd -- "${release_handoff_assets}" && pwd)
release_handoff_package="mcp-doctor-${release_handoff_version}.crate"
release_handoff_formula=mcp-doctor.rb

release_handoff_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print tolower($1) }'
  else
    shasum -a 256 "$1" | awk '{ print tolower($1) }'
  fi
}

release_handoff_package_hash=$(
  release_handoff_sha256 "${release_handoff_assets}/${release_handoff_package}"
)
release_handoff_formula_hash=$(
  release_handoff_sha256 "${release_handoff_assets}/${release_handoff_formula}"
)
release_handoff_commit_lower=$(
  printf '%s\n' "${release_handoff_commit}" | tr '[:upper:]' '[:lower:]'
)

release_handoff_parent=$(dirname -- "${release_handoff_output}")
release_handoff_name=$(basename -- "${release_handoff_output}")
mkdir -p -- "${release_handoff_parent}"
release_handoff_parent=$(cd -- "${release_handoff_parent}" && pwd)
release_handoff_output="${release_handoff_parent}/${release_handoff_name}"
release_handoff_temp=$(mktemp "${release_handoff_parent}/.mcp-doctor-release-handoff.XXXXXX")

cleanup_release_handoff() {
  if [[ -n "${release_handoff_temp:-}" && -f "${release_handoff_temp}" ]]; then
    rm -f -- "${release_handoff_temp}"
  fi
}
trap cleanup_release_handoff EXIT

jq -n \
  --arg version "${release_handoff_version}" \
  --arg tag "v${release_handoff_version}" \
  --arg commit "${release_handoff_commit_lower}" \
  --arg evidence "${release_handoff_evidence}" \
  --arg environment "${release_handoff_environment}" \
  --argjson immutable "${release_handoff_immutable}" \
  --argjson provenance "${release_handoff_provenance}" \
  --arg package "${release_handoff_package}" \
  --arg package_hash "${release_handoff_package_hash}" \
  --arg formula "${release_handoff_formula}" \
  --arg formula_hash "${release_handoff_formula_hash}" \
  '{
    schema: "mcp-doctor.release-handoff/v1",
    evidence: $evidence,
    source_repository: "EnjoyableWork/mcp-doctor",
    source_workflow: ".github/workflows/release.yml",
    source_environment: $environment,
    version: $version,
    tag: $tag,
    release_commit: $commit,
    immutable: $immutable,
    provenance_verified: $provenance,
    cargo: {name: $package, sha256: $package_hash},
    homebrew: {name: $formula, sha256: $formula_hash}
  }' >"${release_handoff_temp}"

mv -- "${release_handoff_temp}" "${release_handoff_output}"
release_handoff_temp=
