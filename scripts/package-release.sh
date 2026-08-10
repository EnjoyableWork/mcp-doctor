#!/usr/bin/env bash

set -euo pipefail

release_script_directory="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"
release_repository_root="$(
  cd -- "${release_script_directory}/.."
  pwd
)"

if [[ $# -ne 4 ]]; then
  echo "usage: $0 <version> <Rust target> <mcp-doctor executable> <output directory>" >&2
  exit 2
fi

release_version=$1
release_target=$2
release_executable=$3
release_output_directory=$4
release_epoch=${SOURCE_DATE_EPOCH:-}

if [[ ! "${release_version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "release version must be a stable semantic version" >&2
  exit 1
fi
case "${release_target}" in
  aarch64-unknown-linux-gnu | x86_64-unknown-linux-gnu) ;;
  *)
    echo "only represented native GNU/Linux targets may be packaged" >&2
    exit 1
    ;;
esac
if [[ ! "${release_epoch}" =~ ^[0-9]+$ ]]; then
  echo "SOURCE_DATE_EPOCH must be the release commit timestamp" >&2
  exit 1
fi
if [[ ! -x "${release_executable}" || -L "${release_executable}" ]]; then
  echo "release executable is missing, symbolic, or not executable" >&2
  exit 1
fi
if ! tar --version 2>/dev/null | head -n 1 | grep -F 'GNU tar' >/dev/null; then
  echo "deterministic release archives require GNU tar" >&2
  exit 1
fi

release_reported_version=$("${release_executable}" --version)
if [[ "${release_reported_version}" != "mcp-doctor ${release_version}" ]]; then
  echo "release executable version does not match the requested archive version" >&2
  exit 1
fi

mkdir -p -- "${release_output_directory}"
release_output_directory="$(cd -- "${release_output_directory}" && pwd)"
release_archive="mcp-doctor-v${release_version}-${release_target}.tar.gz"
release_archive_path="${release_output_directory}/${release_archive}"
if [[ -e "${release_archive_path}" ]]; then
  echo "release archive already exists" >&2
  exit 1
fi

release_temp_parent=${TMPDIR:-/tmp}
release_stage_prefix="${release_temp_parent%/}/mcp-doctor-release-package."
release_stage=$(mktemp -d "${release_stage_prefix}XXXXXX")
release_archive_temp=$(mktemp "${release_output_directory}/.${release_archive}.XXXXXX")

cleanup_release_stage() {
  if [[ "${release_stage}" != "${release_stage_prefix}"* ]]; then
    echo "refusing to remove an unexpected release staging path" >&2
    return 1
  fi
  if [[ -d "${release_stage}" ]]; then
    rm -rf -- "${release_stage}"
  fi
  if [[ -f "${release_archive_temp}" ]]; then
    rm -f -- "${release_archive_temp}"
  fi
}
trap cleanup_release_stage EXIT

install -m 0755 "${release_executable}" "${release_stage}/mcp-doctor"
install -m 0644 \
  "${release_repository_root}/Cargo.lock" \
  "${release_repository_root}/LICENSE" \
  "${release_repository_root}/README.md" \
  "${release_stage}/"

COPYFILE_DISABLE=1 tar \
  --sort=name \
  --format=ustar \
  --mtime="@${release_epoch}" \
  --owner=0 \
  --group=0 \
  --numeric-owner \
  -C "${release_stage}" \
  -cf - \
  Cargo.lock LICENSE README.md mcp-doctor \
  | gzip -n -9 >"${release_archive_temp}"

mv -- "${release_archive_temp}" "${release_archive_path}"
printf '%s\n' "${release_archive_path}"
