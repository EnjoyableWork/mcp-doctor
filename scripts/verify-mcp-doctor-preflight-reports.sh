#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 4 || ! -f "$1" || ! -f "$2" || ! -f "$3" || ! -f "$4" ]]; then
  echo "the four expected mcp-doctor reports were not published" >&2
  exit 1
fi

json_report=$1
junit_report=$2
markdown_report=$3
badge_report=$4

if ! jq -e '
  .schema_version == "mcp-doctor.report/v1" and
  .schema_stability == "stable" and
  (.outcome == "passed" or .outcome == "failed" or .outcome == "incomplete") and
  (.exit_code | type) == "number"
' "$json_report" >/dev/null 2>&1; then
  echo "the mcp-doctor JSON report has an unexpected contract" >&2
  exit 1
fi
if ! grep -Fq '<testsuites ' "$junit_report"; then
  echo "the mcp-doctor JUnit report has an unexpected contract" >&2
  exit 1
fi
markdown_marker=
if ! IFS= read -r markdown_marker <"$markdown_report" ||
  [[ "$markdown_marker" != '<!-- mcp-doctor.markdown/v1 -->' ]]; then
  echo "the mcp-doctor Markdown report has an unexpected contract" >&2
  exit 1
fi
if ! jq -e '
  (.schemaVersion | type) == "number" and .schemaVersion == 1 and
  .label == "mcp-doctor" and
  (.message == "pass" or .message == "fail" or .message == "incomplete") and
  (.color | type) == "string"
' "$badge_report" >/dev/null 2>&1; then
  echo "the mcp-doctor badge report has an unexpected contract" >&2
  exit 1
fi

for report in "$json_report" "$junit_report" "$markdown_report" "$badge_report"; do
  if grep -Fq \
    -e 'synthetic-private-revision-never-report-7f2c' \
    -e 'synthetic-private-ci-stderr-never-report-7f2c' \
    -e '/Users/' \
    -e '/home/runner/' \
    -e 'C:\Users\' \
    "$report"; then
    echo "an mcp-doctor report crossed the safe publication boundary" >&2
    exit 1
  fi
done
