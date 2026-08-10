#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <release archive> <expected version> <fixture executable>" >&2
  exit 2
fi

smoke_archive=$1
smoke_version=$2
smoke_fixture=$3

if [[ ! -f "${smoke_archive}" || -L "${smoke_archive}" ]]; then
  echo "release archive is missing or symbolic" >&2
  exit 1
fi

smoke_archive="$(cd -- "$(dirname -- "${smoke_archive}")" && pwd)/$(basename -- "${smoke_archive}")"
smoke_temp_parent=${TMPDIR:-/tmp}
smoke_extract_prefix="${smoke_temp_parent%/}/mcp-doctor-release-archive."
smoke_extract_root=$(mktemp -d "${smoke_extract_prefix}XXXXXX")

cleanup_smoke_extract_root() {
  if [[ "${smoke_extract_root}" != "${smoke_extract_prefix}"* ]]; then
    echo "refusing to remove an unexpected archive staging path" >&2
    return 1
  fi
  if [[ -d "${smoke_extract_root}" ]]; then
    rm -rf -- "${smoke_extract_root}"
  fi
}
trap cleanup_smoke_extract_root EXIT

archive_members=$(tar -tzf "${smoke_archive}" | LC_ALL=C sort)
expected_members=$(printf '%s\n' Cargo.lock LICENSE README.md mcp-doctor | LC_ALL=C sort)
if [[ "${archive_members}" != "${expected_members}" ]]; then
  echo "release archive contains an unexpected member set" >&2
  exit 1
fi

tar -xzf "${smoke_archive}" -C "${smoke_extract_root}"
for required_file in Cargo.lock LICENSE README.md mcp-doctor; do
  if [[ ! -f "${smoke_extract_root}/${required_file}" || -L "${smoke_extract_root}/${required_file}" ]]; then
    echo "release archive is missing a required regular file" >&2
    exit 1
  fi
done
if [[ ! -x "${smoke_extract_root}/mcp-doctor" ]]; then
  echo "release archive executable is not executable" >&2
  exit 1
fi

"$(dirname -- "$0")/smoke-installed.sh" \
  "${smoke_extract_root}/mcp-doctor" \
  "${smoke_version}" \
  "${smoke_fixture}"
