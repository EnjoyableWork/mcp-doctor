#!/usr/bin/env bash

set -euo pipefail

release_control_repository=EnjoyableWork/mcp-doctor
release_control_version=0.1.0
release_control_commit=${1:-}
release_control_api_version=2026-03-10

if [[ ! "${release_control_commit}" =~ ^[0-9a-f]{40}$ ]]; then
  echo "usage: $0 <exact-current-main-commit>" >&2
  exit 2
fi
if ! gh auth status --hostname github.com >/dev/null 2>&1; then
  echo "GitHub CLI must be authenticated as a repository administrator" >&2
  exit 1
fi

repository_visibility=$(
  gh api "repos/${release_control_repository}" \
    -H "X-GitHub-Api-Version: ${release_control_api_version}" \
    --jq '.visibility'
)
if [[ "${repository_visibility}" != public ]]; then
  echo "release repository must be public" >&2
  exit 1
fi

current_main=$(
  gh api "repos/${release_control_repository}/commits/main" \
    -H "X-GitHub-Api-Version: ${release_control_api_version}" \
    --jq '.sha'
)
if [[ "${current_main}" != "${release_control_commit}" ]]; then
  echo "expected commit is not the exact current main commit" >&2
  exit 1
fi

immutable_releases=$(
  gh api "repos/${release_control_repository}/immutable-releases" \
    -H "X-GitHub-Api-Version: ${release_control_api_version}" \
    --jq '.enabled'
)
if [[ "${immutable_releases}" != true ]]; then
  echo "repository release immutability is not enabled" >&2
  exit 1
fi

if git ls-remote --exit-code --tags \
  "https://github.com/${release_control_repository}.git" \
  "refs/tags/v${release_control_version}" >/dev/null 2>&1; then
  echo "release tag already exists" >&2
  exit 1
fi
if gh release view "v${release_control_version}" \
  --repo "${release_control_repository}" >/dev/null 2>&1; then
  echo "release already exists" >&2
  exit 1
fi

registry_response=$(mktemp "${TMPDIR:-/tmp}/mcp-doctor-registry.XXXXXX")
cleanup_release_control() {
  if [[ -f "${registry_response}" ]]; then
    rm -f -- "${registry_response}"
  fi
}
trap cleanup_release_control EXIT

registry_status=$(
  curl --silent --show-error --location --retry 0 \
    --connect-timeout 10 --max-time 60 \
    --output "${registry_response}" \
    --write-out '%{http_code}' \
    --header 'User-Agent: mcp-doctor-release-control/0.1 (+https://github.com/EnjoyableWork/mcp-doctor)' \
    'https://crates.io/api/v1/crates/mcp-doctor'
)
case "${registry_status}" in
  404) ;;
  200)
    jq -e \
      --arg version "${release_control_version}" \
      '.crate.repository == "https://github.com/EnjoyableWork/mcp-doctor" and
       any(.versions[]; .num == $version)' \
      "${registry_response}" >/dev/null
    ;;
  *)
    echo "crates.io package identity could not be verified" >&2
    exit 1
    ;;
esac

printf 'Verified release controls for %s at %s.\n' \
  "${release_control_repository}" \
  "${release_control_commit}"
