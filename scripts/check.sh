#!/usr/bin/env bash

set -euo pipefail

doctor_script_directory="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"
doctor_repository_root="$(
  cd -- "${doctor_script_directory}/.."
  pwd
)"
doctor_caller_user_root="${HOME:?HOME must identify the caller Rust toolchain root}"
doctor_cargo_home="${CARGO_HOME:-${doctor_caller_user_root}/.cargo}"
doctor_rustup_home="${RUSTUP_HOME:-${doctor_caller_user_root}/.rustup}"
doctor_tool_path="${PATH:?PATH must locate the Rust toolchain}"
doctor_temp_parent="${TMPDIR:-/tmp}"
doctor_synthetic_root="$(
  mktemp -d "${doctor_temp_parent%/}/mcp-doctor-quality.XXXXXX"
)"
doctor_synthetic_prefix="${doctor_temp_parent%/}/mcp-doctor-quality."
doctor_synthetic_user_root="${doctor_synthetic_root}/user"

doctor_cleanup() {
  if [[ "${doctor_synthetic_root}" != "${doctor_synthetic_prefix}"* ]]; then
    printf 'Refusing to remove unexpected quality-gate path: %s\n' \
      "${doctor_synthetic_root}" >&2
    return 1
  fi

  if [[ -d "${doctor_synthetic_root}" ]]; then
    rm -rf -- "${doctor_synthetic_root}"
  fi
}

trap doctor_cleanup EXIT

mkdir -p -- \
  "${doctor_synthetic_user_root}/.cache" \
  "${doctor_synthetic_user_root}/.config" \
  "${doctor_synthetic_user_root}/.local/share" \
  "${doctor_synthetic_user_root}/.local/state" \
  "${doctor_synthetic_user_root}/AppData/Local" \
  "${doctor_synthetic_user_root}/AppData/Roaming" \
  "${doctor_synthetic_root}/runtime" \
  "${doctor_synthetic_root}/tmp"

doctor_run_isolated() {
  env -i \
    APPDATA="${doctor_synthetic_user_root}/AppData/Roaming" \
    CARGO_HOME="${doctor_cargo_home}" \
    CARGO_INCREMENTAL=0 \
    CARGO_TERM_COLOR=never \
    CFFIXED_USER_HOME="${doctor_synthetic_user_root}" \
    HOME="${doctor_synthetic_user_root}" \
    LANG=C \
    LC_ALL=C \
    LOCALAPPDATA="${doctor_synthetic_user_root}/AppData/Local" \
    MCP_DOCTOR_TEST_MODE=1 \
    MCP_DOCTOR_TEST_ROOT="${doctor_synthetic_root}" \
    NO_COLOR=1 \
    PATH="${doctor_tool_path}" \
    RUSTUP_HOME="${doctor_rustup_home}" \
    TEMP="${doctor_synthetic_root}/tmp" \
    TMP="${doctor_synthetic_root}/tmp" \
    TMPDIR="${doctor_synthetic_root}/tmp" \
    TZ=UTC \
    USERPROFILE="${doctor_synthetic_user_root}" \
    XDG_CACHE_HOME="${doctor_synthetic_user_root}/.cache" \
    XDG_CONFIG_HOME="${doctor_synthetic_user_root}/.config" \
    XDG_DATA_HOME="${doctor_synthetic_user_root}/.local/share" \
    XDG_RUNTIME_DIR="${doctor_synthetic_root}/runtime" \
    XDG_STATE_HOME="${doctor_synthetic_user_root}/.local/state" \
    "$@"
}

cd -- "${doctor_repository_root}"

printf 'Running quality gates through a disposable user environment.\n'
doctor_run_isolated cargo fmt --all -- --check
doctor_run_isolated cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
doctor_run_isolated cargo test --workspace --all-targets --all-features --locked
printf 'Formatting, Clippy, and tests passed.\n'
