#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 1 ]]; then
  printf 'usage: %s RUNNER_LABEL\n' "$0" >&2
  exit 2
fi

ci_runner_label=$1
ci_inventory=.github/ci-tools.json

if [[ ! -f "$ci_inventory" ]]; then
  printf 'CI tool inventory is unavailable\n' >&2
  exit 2
fi
if ! command -v jq >/dev/null 2>&1; then
  printf 'required declared runner command is unavailable: jq\n' >&2
  exit 2
fi
if ! jq -e --arg runner "$ci_runner_label" '
  .schema_version == "mcp-doctor.ci-tools/v1" and
  ([.runner_contracts[] | select(.runner == $runner)] | length == 1) and
  ([.runner_contracts[] | select(.runner == $runner) | .commands[]] |
    length > 0 and
    all(type == "string" and test("^[a-z0-9][a-z0-9-]*$")))
' "$ci_inventory" >/dev/null; then
  printf 'CI tool inventory has no valid exact runner contract\n' >&2
  exit 2
fi

while IFS= read -r ci_command; do
  if ! command -v "$ci_command" >/dev/null 2>&1; then
    printf 'required declared runner command is unavailable: %s\n' \
      "$ci_command" >&2
    exit 2
  fi
done < <(
  jq -er --arg runner "$ci_runner_label" '
    .runner_contracts[] | select(.runner == $runner) | .commands[]
  ' "$ci_inventory"
)

printf 'Verified declared runner commands for %s.\n' "$ci_runner_label"
