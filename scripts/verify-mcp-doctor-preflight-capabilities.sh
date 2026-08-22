#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 2 || ! -f "$1" || -z "$2" ]]; then
  echo "the expected mcp-doctor capability evidence was not provided" >&2
  exit 1
fi

capabilities=$1
expected_version=$2

if ! jq -e --arg expected_version "$expected_version" '
  .schema_version == "mcp-doctor.capabilities/v1" and
  .schema_stability == "stable" and
  .product == {"name": "mcp-doctor", "version": $expected_version} and
  ([.commands[] | select(.name == "inspect")] | length) == 1 and
  ([.commands[] | select(.name == "inspect")][0] |
    .activity == "passive" and
    .artifact_reporters == ["json", "junit", "markdown", "badge"] and
    .output_schema_versions == [
      "mcp-doctor.report/v1",
      "mcp-doctor.contract-snapshot/v1alpha1"
    ]) and
  .schema_versions.diagnostic_report == ["mcp-doctor.report/v1"] and
  .schema_versions.markdown_report == ["mcp-doctor.markdown/v1"] and
  .schema_versions.badge_report == ["mcp-doctor.badge/v1"]
' "$capabilities" >/dev/null 2>&1; then
  echo "the exact mcp-doctor binary lacks the required passive report contracts" >&2
  exit 1
fi
