#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 2 ]]; then
  printf 'usage: %s <release archive> <absent SPDX JSON output>\n' "$0" >&2
  exit 2
fi

sbom_input=$1
sbom_output=$2
sbom_script_directory="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
sbom_input_max_bytes=50000000
sbom_output_max_bytes=10000000
sbom_output_limit_blocks=19532
sbom_generation_max_seconds=120
sbom_generation_kill_grace_seconds=5
sbom_generation_max_processors=4

if [[ ! -f "$sbom_input" ]] || [[ -L "$sbom_input" ]]; then
  printf 'SBOM input must be a regular release archive\n' >&2
  exit 2
fi
if [[ -e "$sbom_output" ]] || [[ -L "$sbom_output" ]]; then
  printf 'SBOM output must be absent\n' >&2
  exit 2
fi
if [[ ! -d "$(dirname -- "$sbom_output")" ]]; then
  printf 'SBOM output parent must exist\n' >&2
  exit 2
fi

for sbom_command in chmod install mktemp timeout tr wc; do
  if ! command -v "$sbom_command" >/dev/null 2>&1; then
    printf 'required SBOM generator command is unavailable: %s\n' \
      "$sbom_command" >&2
    exit 2
  fi
done
sbom_input_bytes="$(wc -c <"$sbom_input" | tr -d '[:space:]')"
if [[ "$sbom_input_bytes" -eq 0 ]] ||
  [[ "$sbom_input_bytes" -gt "$sbom_input_max_bytes" ]]; then
  printf 'SBOM input is outside its byte bound\n' >&2
  exit 2
fi

umask 077
sbom_temp_parent=${TMPDIR:-/tmp}
case "$sbom_temp_parent" in
  /*) ;;
  *)
    printf 'SBOM temporary parent must be absolute\n' >&2
    exit 2
    ;;
esac
sbom_temp_root="$(mktemp -d "${sbom_temp_parent%/}/mcp-doctor-sbom.XXXXXX")"
sbom_temp_prefix="${sbom_temp_parent%/}/mcp-doctor-sbom."

sbom_cleanup() {
  if [[ "$sbom_temp_root" != "$sbom_temp_prefix"* ]]; then
    printf 'refusing to remove unexpected SBOM generator path\n' >&2
    return 1
  fi
  if [[ -d "$sbom_temp_root" ]]; then
    rm -rf -- "$sbom_temp_root"
  fi
}
trap sbom_cleanup EXIT

mkdir -p -- "$sbom_temp_root/home" "$sbom_temp_root/tmp"
"$sbom_script_directory/install-syft.sh" "$sbom_temp_root/tools" >/dev/null
sbom_config="$sbom_temp_root/syft.yaml"
printf '%s\n' \
  'cpp:' \
  '  vcpkg-allow-git-clone: false' \
  'golang:' \
  '  search-local-mod-cache-licenses: false' \
  '  search-local-vendor-licenses: false' \
  '  search-remote-licenses: false' \
  '  use-packages-lib: false' \
  'java:' \
  '  use-maven-local-repository: false' \
  '  use-network: false' \
  'javascript:' \
  '  search-remote-licenses: false' \
  'python:' \
  '  search-remote-licenses: false' >"$sbom_config"
chmod 0600 "$sbom_config"

sbom_temporary_output="$sbom_temp_root/sbom.spdx.json"
sbom_temporary_stderr="$sbom_temp_root/syft.stderr"
sbom_timeout="$(command -v timeout)"
if ! (
  ulimit -f "$sbom_output_limit_blocks"
  exec env -i \
    GOMAXPROCS="$sbom_generation_max_processors" \
    HOME="$sbom_temp_root/home" \
    LANG=C \
    LC_ALL=C \
    NO_COLOR=1 \
    SYFT_CHECK_FOR_APP_UPDATE=false \
    TMPDIR="$sbom_temp_root/tmp" \
    "$sbom_timeout" \
      --signal=TERM \
      --kill-after="${sbom_generation_kill_grace_seconds}s" \
      "${sbom_generation_max_seconds}s" \
      "$sbom_temp_root/tools/syft" scan "file:$sbom_input" \
        --output spdx-json \
        --config "$sbom_config"
) >"$sbom_temporary_output" 2>"$sbom_temporary_stderr"; then
  printf 'Syft failed to generate the release SBOM\n' >&2
  exit 1
fi

sbom_bytes="$(wc -c <"$sbom_temporary_output" | tr -d '[:space:]')"
if [[ "$sbom_bytes" -eq 0 ]] ||
  [[ "$sbom_bytes" -gt "$sbom_output_max_bytes" ]]; then
  printf 'generated release SBOM is outside its byte bound\n' >&2
  exit 1
fi

install -m 0644 "$sbom_temporary_output" "$sbom_output"
