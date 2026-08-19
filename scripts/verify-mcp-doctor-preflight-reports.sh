#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 2 || ! -f "$1" || ! -f "$2" ]]; then
  echo "the two expected mcp-doctor reports were not published" >&2
  exit 1
fi

for report in "$1" "$2"; do
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
