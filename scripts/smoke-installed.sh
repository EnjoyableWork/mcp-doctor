#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <mcp-doctor executable> <expected version> <fixture executable>" >&2
  exit 2
fi

smoke_executable=$1
smoke_version=$2
smoke_fixture=$3

for smoke_required_executable in "${smoke_executable}" "${smoke_fixture}"; do
  if [[ ! -x "${smoke_required_executable}" || -L "${smoke_required_executable}" ]]; then
    echo "installed smoke executable or fixture is missing, symbolic, or not executable" >&2
    exit 1
  fi
done
if ! command -v jq >/dev/null 2>&1; then
  echo "installed smoke requires jq" >&2
  exit 1
fi

smoke_executable="$(cd -- "$(dirname -- "${smoke_executable}")" && pwd)/$(basename -- "${smoke_executable}")"
smoke_fixture="$(cd -- "$(dirname -- "${smoke_fixture}")" && pwd)/$(basename -- "${smoke_fixture}")"
smoke_temp_parent=${TMPDIR:-/tmp}
smoke_root_prefix="${smoke_temp_parent%/}/mcp-doctor-release-smoke."
smoke_root=$(mktemp -d "${smoke_root_prefix}XXXXXX")
smoke_home="${smoke_root}/home"
smoke_report="${smoke_root}/report.json"
smoke_stderr="${smoke_root}/stderr.txt"
smoke_path=${PATH:?PATH must locate the fixture platform loader}

cleanup_smoke_root() {
  if [[ "${smoke_root}" != "${smoke_root_prefix}"* ]]; then
    echo "refusing to remove an unexpected installed-smoke path" >&2
    return 1
  fi
  if [[ -d "${smoke_root}" ]]; then
    rm -rf -- "${smoke_root}"
  fi
}
trap cleanup_smoke_root EXIT

mkdir -p -- "${smoke_home}"

run_mcp_doctor() {
  env -i \
    HOME="${smoke_home}" \
    LANG=C \
    LC_ALL=C \
    NO_COLOR=1 \
    PATH="${smoke_path}" \
    TEMP="${smoke_root}" \
    TMP="${smoke_root}" \
    TMPDIR="${smoke_root}" \
    TZ=UTC \
    USERPROFILE="${smoke_home}" \
    "${smoke_executable}" "$@"
}

version_output=$(run_mcp_doctor --version)
if [[ "${version_output}" != "mcp-doctor ${smoke_version}" ]]; then
  echo "installed executable reported an unexpected version" >&2
  exit 1
fi

if ! run_mcp_doctor inspect --format json -- "${smoke_fixture}" catalog-valid \
  >"${smoke_report}" 2>"${smoke_stderr}"; then
  echo "installed passive diagnostic smoke failed" >&2
  exit 1
fi
if [[ -s "${smoke_stderr}" ]]; then
  echo "installed passive diagnostic smoke wrote unexpected stderr" >&2
  exit 1
fi

jq -e '
  .schema_version == "mcp-doctor.report/v1" and
  .schema_stability == "stable" and
  .protocol_revision == "2026-07-28" and
  .primary_diagnosis == null and
  .independent_findings == [] and
  .outcome == "passed" and
  .exit_code == 0 and
  .summary.required == 5 and
  .summary.required_skipped == 0 and
  .summary.failed == 0 and
  ([.checks[] | select(.requirement == "required") |
    (.state == "performed" and .outcome == "passed")] | all) and
  ([.checks[] | select(.id == "runtime.tools")] | length) == 1 and
  ([.checks[] | select(.id == "runtime.tools") |
    (.state == "skipped" and .skip_reason == "not_authorized" and
     (has("blocked_by") | not))] | all)
' "${smoke_report}" >/dev/null
