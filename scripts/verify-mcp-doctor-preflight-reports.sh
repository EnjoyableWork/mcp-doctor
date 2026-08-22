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

if ! report_outcome=$(jq -er '
  if
    .schema_version == "mcp-doctor.report/v1" and
    .schema_stability == "stable" and
    (.outcome == "passed" or .outcome == "failed" or .outcome == "incomplete") and
    (.exit_code | type) == "number"
  then .outcome
  else error("unexpected report contract")
  end
' "$json_report" 2>/dev/null); then
  echo "the mcp-doctor JSON report has an unexpected contract" >&2
  exit 1
fi

if ! report_exit=$(jq -er '.exit_code' "$json_report" 2>/dev/null); then
  echo "the mcp-doctor JSON report has an unexpected contract" >&2
  exit 1
fi

case "$report_outcome:$report_exit" in
  passed:0)
    badge_message=pass
    badge_color=brightgreen
    exit_meaning=success
    ;;
  failed:1)
    badge_message=fail
    badge_color=red
    exit_meaning=unsuccessful_result
    ;;
  incomplete:3)
    badge_message=incomplete
    badge_color=lightgrey
    exit_meaning=incomplete_evidence
    ;;
  *)
    echo "the mcp-doctor JSON outcome and exit do not agree" >&2
    exit 1
    ;;
esac

if [[ $(grep -Fc '<testsuites ' "$junit_report") -ne 1 ]] ||
  ! grep -Fqx "report_outcome=$report_outcome" "$junit_report" ||
  ! grep -Fqx "exit_code=$report_exit" "$junit_report"; then
  echo "the mcp-doctor JUnit report has an unexpected contract" >&2
  exit 1
fi

markdown_marker=
if ! IFS= read -r markdown_marker <"$markdown_report" ||
  [[ "$markdown_marker" != '<!-- mcp-doctor.markdown/v1 -->' ]]; then
  echo "the mcp-doctor Markdown report has an unexpected contract" >&2
  exit 1
fi

printf -v markdown_outcome '| Outcome | `%s` |' "$report_outcome"
printf -v markdown_exit '| Exit | `%s` (`%s`) |' "$report_exit" "$exit_meaning"
if ! grep -Fqx "$markdown_outcome" "$markdown_report" ||
  ! grep -Fqx "$markdown_exit" "$markdown_report"; then
  echo "the mcp-doctor Markdown report disagrees with the JSON result" >&2
  exit 1
fi

if ! jq -e \
  --arg message "$badge_message" \
  --arg color "$badge_color" '
  keys == ["color", "label", "message", "schemaVersion"] and
  (.schemaVersion | type) == "number" and .schemaVersion == 1 and
  .label == "mcp-doctor" and
  .message == $message and
  .color == $color
' "$badge_report" >/dev/null 2>&1; then
  echo "the mcp-doctor badge report disagrees with the fixed outcome mapping" >&2
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
