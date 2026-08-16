#!/usr/bin/env bash

set -euo pipefail
LC_ALL=C
export LC_ALL

if [[ $# -lt 3 ]]; then
  echo "usage: $0 <future|published|rehearsal> <v-prefixed tag> <package version> [published stable versions...]" >&2
  exit 2
fi

release_version_mode=$1
release_version_tag=$2
release_package_version=$3
shift 3
release_published_versions=("$@")

stable_tag_pattern='^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'
stable_version_pattern='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'

if [[ ! "${release_version_tag}" =~ ${stable_tag_pattern} ]]; then
  echo "release tag must be a canonical stable v-prefixed semantic version" >&2
  exit 1
fi
if [[ ! "${release_package_version}" =~ ${stable_version_pattern} ]]; then
  echo "Cargo package version must be a canonical stable semantic version" >&2
  exit 1
fi

release_tag_version=${release_version_tag#v}
if [[ "${release_tag_version}" != "${release_package_version}" ]]; then
  echo "release tag and Cargo package version do not match" >&2
  exit 1
fi

release_semver_greater_than() {
  local candidate=$1
  local baseline=$2
  local candidate_major candidate_minor candidate_patch
  local baseline_major baseline_minor baseline_patch

  IFS=. read -r candidate_major candidate_minor candidate_patch <<<"${candidate}"
  IFS=. read -r baseline_major baseline_minor baseline_patch <<<"${baseline}"

  if [[ "${candidate_major}" != "${baseline_major}" ]]; then
    release_numeric_component_greater "${candidate_major}" "${baseline_major}"
    return
  fi
  if [[ "${candidate_minor}" != "${baseline_minor}" ]]; then
    release_numeric_component_greater "${candidate_minor}" "${baseline_minor}"
    return
  fi
  release_numeric_component_greater "${candidate_patch}" "${baseline_patch}"
}

release_numeric_component_greater() {
  local candidate=$1
  local baseline=$2

  if ((${#candidate} != ${#baseline})); then
    ((${#candidate} > ${#baseline}))
    return
  fi
  [[ "${candidate}" > "${baseline}" ]]
}

case "${release_version_mode}" in
  future)
    if [[ ${#release_published_versions[@]} -eq 0 ]]; then
      echo "future release validation requires the published stable version inventory" >&2
      exit 2
    fi
    if ! release_semver_greater_than "${release_package_version}" 0.1.0; then
      echo "repeat-release automation refuses v0.1.0 and earlier versions" >&2
      exit 1
    fi
    for release_published_version in "${release_published_versions[@]}"; do
      if [[ ! "${release_published_version}" =~ ${stable_version_pattern} ]]; then
        echo "published release inventory contains a noncanonical version" >&2
        exit 1
      fi
      if [[ "${release_package_version}" == "${release_published_version}" ]]; then
        continue
      fi
      if ! release_semver_greater_than \
        "${release_package_version}" \
        "${release_published_version}"; then
        echo "release version is older than an already published stable version" >&2
        exit 1
      fi
    done
    ;;
  published)
    if [[ ${#release_published_versions[@]} -ne 0 ]]; then
      echo "published verification does not accept a registry inventory" >&2
      exit 2
    fi
    if [[ "${release_package_version}" != 0.1.0 ]] &&
      ! release_semver_greater_than "${release_package_version}" 0.1.0; then
      echo "published release verification refuses versions before v0.1.0" >&2
      exit 1
    fi
    ;;
  rehearsal)
    if [[ ${#release_published_versions[@]} -ne 0 ]]; then
      echo "release rehearsal does not accept a registry inventory" >&2
      exit 2
    fi
    if [[ "${release_package_version}" != 0.1.0 ]] &&
      ! release_semver_greater_than "${release_package_version}" 0.1.0; then
      echo "release rehearsal refuses versions before v0.1.0" >&2
      exit 1
    fi
    ;;
  *)
    echo "release version mode must be future, published, or rehearsal" >&2
    exit 2
    ;;
esac

printf '%s\n' "${release_package_version}"
