#!/usr/bin/env bash

set -euo pipefail

release_handoff_verify_script_directory="$({ cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd; })"

if [[ $# -ne 5 ]]; then
  echo "usage: $0 <handoff manifest> <stable version> <verified|synthetic-rehearsal> <Cargo package> <Homebrew formula>" >&2
  exit 2
fi

release_handoff_manifest=$1
release_handoff_version=$2
release_handoff_evidence=$3
release_handoff_package=$4
release_handoff_formula=$5

case "${release_handoff_evidence}" in
  verified | synthetic-rehearsal) ;;
  *)
    echo "release handoff evidence must be verified or synthetic-rehearsal" >&2
    exit 2
    ;;
esac

"${release_handoff_verify_script_directory}/validate-release-version.sh" \
  published \
  "v${release_handoff_version}" \
  "${release_handoff_version}" >/dev/null

for release_handoff_file in \
  "${release_handoff_manifest}" \
  "${release_handoff_package}" \
  "${release_handoff_formula}"; do
  if [[ ! -f "${release_handoff_file}" || -L "${release_handoff_file}" ]]; then
    echo "release handoff inputs must be regular, non-symbolic-link files" >&2
    exit 1
  fi
done

release_handoff_package_name="mcp-doctor-${release_handoff_version}.crate"
if [[ "$(basename -- "${release_handoff_package}")" != "${release_handoff_package_name}" ]]; then
  echo "release handoff Cargo filename does not match the version" >&2
  exit 1
fi
if [[ "$(basename -- "${release_handoff_formula}")" != mcp-doctor.rb ]]; then
  echo "release handoff formula must be named mcp-doctor.rb" >&2
  exit 1
fi

jq -e \
  --arg version "${release_handoff_version}" \
  --arg tag "v${release_handoff_version}" \
  --arg evidence "${release_handoff_evidence}" \
  --arg package "${release_handoff_package_name}" \
  '
    type == "object" and
    keys == [
      "cargo", "evidence", "homebrew", "immutable", "provenance_verified",
      "release_commit", "schema", "source_environment",
      "source_repository", "source_workflow", "tag", "version"
    ] and
    .schema == "mcp-doctor.release-handoff/v1" and
    .evidence == $evidence and
    .source_repository == "EnjoyableWork/mcp-doctor" and
    .source_workflow == ".github/workflows/release.yml" and
    .version == $version and
    .tag == $tag and
    (.release_commit | test("^[0-9a-f]{40}$")) and
    (if $evidence == "verified" then
       .source_environment == "release" and
       .immutable == true and
       .provenance_verified == true
     else
       .source_environment == "synthetic-rehearsal" and
       .immutable == false and
       .provenance_verified == false
     end) and
    .cargo.name == $package and
    (.cargo.sha256 | test("^[0-9a-f]{64}$")) and
    .homebrew.name == "mcp-doctor.rb" and
    (.homebrew.sha256 | test("^[0-9a-f]{64}$"))
  ' "${release_handoff_manifest}" >/dev/null

release_handoff_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print tolower($1) }'
  else
    shasum -a 256 "$1" | awk '{ print tolower($1) }'
  fi
}

release_handoff_expected_package_hash=$(jq -r '.cargo.sha256' "${release_handoff_manifest}")
release_handoff_expected_formula_hash=$(jq -r '.homebrew.sha256' "${release_handoff_manifest}")
release_handoff_actual_package_hash=$(release_handoff_sha256 "${release_handoff_package}")
release_handoff_actual_formula_hash=$(release_handoff_sha256 "${release_handoff_formula}")

if [[ "${release_handoff_actual_package_hash}" != "${release_handoff_expected_package_hash}" ]]; then
  echo "Cargo handoff bytes do not match the verified immutable release" >&2
  exit 1
fi
if [[ "${release_handoff_actual_formula_hash}" != "${release_handoff_expected_formula_hash}" ]]; then
  echo "Homebrew handoff bytes do not match the verified immutable release" >&2
  exit 1
fi
