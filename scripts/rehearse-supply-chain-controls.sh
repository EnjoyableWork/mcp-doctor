#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 0 ]]; then
  printf 'usage: %s\n' "$0" >&2
  exit 2
fi

rehearsal_script_directory="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"

"$rehearsal_script_directory/verify-source-artifacts.sh" --worktree >/dev/null

umask 077
rehearsal_temp_parent=${TMPDIR:-/tmp}
rehearsal_temp_root="$(mktemp -d "${rehearsal_temp_parent%/}/mcp-doctor-supply-rehearsal.XXXXXX")"
rehearsal_temp_prefix="${rehearsal_temp_parent%/}/mcp-doctor-supply-rehearsal."

rehearsal_cleanup() {
  if [[ "$rehearsal_temp_root" != "$rehearsal_temp_prefix"* ]]; then
    printf 'refusing to remove unexpected supply-chain rehearsal path\n' >&2
    return 1
  fi
  if [[ -d "$rehearsal_temp_root" ]]; then
    rm -rf -- "$rehearsal_temp_root"
  fi
}
trap rehearsal_cleanup EXIT

rehearsal_repo="$rehearsal_temp_root/repository"
mkdir -p -- "$rehearsal_repo/scripts"
install -m 0755 \
  "$rehearsal_script_directory/verify-source-artifacts.sh" \
  "$rehearsal_repo/scripts/verify-source-artifacts.sh"
printf '# synthetic reviewable source\n' >"$rehearsal_repo/README.md"
printf '#!/usr/bin/env bash\nprintf "synthetic\\n"\n' \
  >"$rehearsal_repo/scripts/source.sh"
chmod 0755 "$rehearsal_repo/scripts/source.sh"

git -C "$rehearsal_repo" init --quiet --initial-branch=main
git -C "$rehearsal_repo" config user.email synthetic@example.invalid
git -C "$rehearsal_repo" config user.name 'Synthetic Supply Chain Rehearsal'
git -C "$rehearsal_repo" add README.md scripts
git -C "$rehearsal_repo" commit --quiet -m 'test: add reviewable source'
"$rehearsal_repo/scripts/verify-source-artifacts.sh" HEAD >/dev/null

rehearsal_expect_rejection() {
  local description=$1
  if "$rehearsal_repo/scripts/verify-source-artifacts.sh" HEAD \
    >/dev/null 2>&1; then
    printf 'source-artifact verifier accepted %s\n' "$description" >&2
    exit 1
  fi
}

printf '\177ELF\002\001\001\000synthetic\n' >"$rehearsal_repo/generated-tool"
chmod 0755 "$rehearsal_repo/generated-tool"
git -C "$rehearsal_repo" add generated-tool
git -C "$rehearsal_repo" commit --quiet -m 'test: add generated executable'
rehearsal_expect_rejection 'a generated executable'
git -C "$rehearsal_repo" rm --quiet generated-tool
git -C "$rehearsal_repo" commit --quiet -m 'test: remove generated executable'

printf 'synthetic\000binary\n' >"$rehearsal_repo/unreviewable.dat"
git -C "$rehearsal_repo" add unreviewable.dat
git -C "$rehearsal_repo" commit --quiet -m 'test: add NUL-bearing artifact'
rehearsal_expect_rejection 'a NUL-bearing binary artifact'
git -C "$rehearsal_repo" rm --quiet unreviewable.dat
git -C "$rehearsal_repo" commit --quiet -m 'test: remove NUL-bearing artifact'

printf '\377\376synthetic binary without NUL\n' >"$rehearsal_repo/unreviewable.dat"
git -C "$rehearsal_repo" add unreviewable.dat
git -C "$rehearsal_repo" commit --quiet -m 'test: add non-UTF-8 artifact'
rehearsal_expect_rejection 'a non-UTF-8 binary artifact'
git -C "$rehearsal_repo" rm --quiet unreviewable.dat
git -C "$rehearsal_repo" commit --quiet -m 'test: remove non-UTF-8 artifact'

printf 'synthetic text with a generated executable extension\n' \
  >"$rehearsal_repo/generated.exe"
git -C "$rehearsal_repo" add generated.exe
git -C "$rehearsal_repo" commit --quiet -m 'test: add generated executable extension'
rehearsal_expect_rejection 'a generated executable extension'

rehearsal_historical_verifier="$rehearsal_script_directory/verify-historical-homebrew-formula.sh"
rehearsal_readonly_verifier="$rehearsal_script_directory/verify-read-only-repository-settings.sh"
if [[ ! -x "$rehearsal_historical_verifier" ]] ||
  [[ ! -x "$rehearsal_readonly_verifier" ]]; then
  printf 'focused supply-chain verifier is unavailable\n' >&2
  exit 1
fi

rehearsal_fake_bin="$rehearsal_temp_root/fake-bin"
rehearsal_fake_gh="$rehearsal_fake_bin/gh"
rehearsal_historical_formula="$rehearsal_temp_root/historical-mcp-doctor.rb"
rehearsal_current_formula="$rehearsal_temp_root/current-mcp-doctor.rb"
rehearsal_mutated_formula="$rehearsal_temp_root/mutated-mcp-doctor.rb"
rehearsal_gh_log="$rehearsal_temp_root/gh.log"
rehearsal_historical_tap_commit=1111111111111111111111111111111111111111
rehearsal_current_tap_commit=2222222222222222222222222222222222222222
rehearsal_package_sha=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
rehearsal_source_url=https://github.com/EnjoyableWork/mcp-doctor/releases/download/v0.3.0/mcp-doctor-0.3.0.crate
rehearsal_historical_endpoint="repos/EnjoyableWork/homebrew-tap/contents/Formula/mcp-doctor.rb?ref=$rehearsal_historical_tap_commit"

mkdir -p -- "$rehearsal_fake_bin"
# The quoted lines intentionally defer expansion to the generated fake process.
# shellcheck disable=SC2016
printf '%s\n' \
  '#!/usr/bin/env bash' \
  'set -euo pipefail' \
  'if [[ $# -lt 2 ]] || [[ "$1" != api ]]; then exit 2; fi' \
  'if [[ "$2" == graphql ]]; then' \
  '  printf "graphql\n" >>"$REHEARSAL_GH_LOG"' \
  '  if [[ "${REHEARSAL_GRAPHQL_EXIT:-0}" != 0 ]]; then exit "$REHEARSAL_GRAPHQL_EXIT"; fi' \
  '  printf "%s\n" "$REHEARSAL_GRAPHQL_RESPONSE"' \
  '  exit 0' \
  'fi' \
  'fake_endpoint=' \
  'for fake_argument in "$@"; do fake_endpoint=$fake_argument; done' \
  'printf "%s\\n" "$fake_endpoint" >>"$REHEARSAL_GH_LOG"' \
  'case "$fake_endpoint" in' \
  '  "$REHEARSAL_HISTORICAL_ENDPOINT") fake_source=$REHEARSAL_HISTORICAL_FORMULA ;;' \
  '  repos/EnjoyableWork/homebrew-tap/commits/main)' \
  '    printf "{\\\"sha\\\":\\\"%s\\\"}\\n" "$REHEARSAL_CURRENT_TAP_COMMIT"' \
  '    exit 0' \
  '    ;;' \
  '  "repos/EnjoyableWork/homebrew-tap/contents/Formula/mcp-doctor.rb?ref=$REHEARSAL_CURRENT_TAP_COMMIT")' \
  '    fake_source=$REHEARSAL_CURRENT_FORMULA' \
  '    ;;' \
  '  *) exit 1 ;;' \
  'esac' \
  'fake_content="$(base64 <"$fake_source" | tr -d "\\n")"' \
  'printf "{\\\"type\\\":\\\"file\\\",\\\"encoding\\\":\\\"base64\\\",\\\"content\\\":\\\"%s\\\"}\\n" "$fake_content"' \
  >"$rehearsal_fake_gh"
chmod 0755 "$rehearsal_fake_gh"

printf '%s\n' \
  'class McpDoctor < Formula' \
  '  desc "Synthetic historical formula"' \
  '  homepage "https://github.com/EnjoyableWork/mcp-doctor"' \
  "  url \"$rehearsal_source_url\"" \
  "  sha256 \"$rehearsal_package_sha\"" \
  '  license "MIT"' \
  'end' \
  >"$rehearsal_historical_formula"
printf '%s\n' \
  'class McpDoctor < Formula' \
  '  desc "Synthetic current formula"' \
  '  homepage "https://github.com/EnjoyableWork/mcp-doctor"' \
  '  url "https://github.com/EnjoyableWork/mcp-doctor/releases/download/v0.3.2/mcp-doctor-0.3.2.crate"' \
  '  sha256 "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"' \
  '  license "MIT"' \
  'end' \
  >"$rehearsal_current_formula"
printf '%s\n' \
  'class McpDoctor < Formula' \
  '  desc "Mutated historical formula"' \
  '  homepage "https://github.com/EnjoyableWork/mcp-doctor"' \
  "  url \"$rehearsal_source_url\"" \
  "  sha256 \"$rehearsal_package_sha\"" \
  '  license "MIT"' \
  'end' \
  >"$rehearsal_mutated_formula"

rehearsal_verify_readonly_settings() {
  local graphql_response=$1
  local graphql_exit=${2:-0}
  PATH="$rehearsal_fake_bin:$PATH" \
    REHEARSAL_GH_LOG="$rehearsal_gh_log" \
    REHEARSAL_GRAPHQL_RESPONSE="$graphql_response" \
    REHEARSAL_GRAPHQL_EXIT="$graphql_exit" \
    "$rehearsal_readonly_verifier" \
    EnjoyableWork/mcp-doctor \
    2026-03-10
}

: >"$rehearsal_gh_log"
rehearsal_verify_readonly_settings \
  '{"data":{"repository":{"nameWithOwner":"EnjoyableWork/mcp-doctor","autoMergeAllowed":false}}}'
if [[ "$(wc -l <"$rehearsal_gh_log" | tr -d '[:space:]')" != 1 ]] ||
  ! grep -Fx graphql "$rehearsal_gh_log" >/dev/null; then
  printf 'read-only repository verifier used an unexpected API boundary\n' >&2
  exit 1
fi

for rehearsal_unsafe_response in \
  '{"data":{"repository":{"nameWithOwner":"EnjoyableWork/mcp-doctor","autoMergeAllowed":true}}}' \
  '{"data":{"repository":{"nameWithOwner":"EnjoyableWork/mcp-doctor","autoMergeAllowed":null}}}' \
  '{"data":{"repository":{"nameWithOwner":"EnjoyableWork/another","autoMergeAllowed":false}}}' \
  '{"data":{"repository":null}}' \
  '{"errors":[{"type":"SYNTHETIC"}],"data":{"repository":null}}' \
  'not-json'; do
  : >"$rehearsal_gh_log"
  if rehearsal_verify_readonly_settings "$rehearsal_unsafe_response" \
    >/dev/null 2>&1; then
    printf 'read-only repository verifier accepted unsafe synthetic evidence\n' >&2
    exit 1
  fi
done

: >"$rehearsal_gh_log"
if rehearsal_verify_readonly_settings '{}' 1 >/dev/null 2>&1; then
  printf 'read-only repository verifier accepted a failed API request\n' >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  rehearsal_historical_formula_sha="$(
    sha256sum "$rehearsal_historical_formula" | awk '{ print $1 }'
  )"
elif command -v shasum >/dev/null 2>&1; then
  rehearsal_historical_formula_sha="$(
    shasum -a 256 "$rehearsal_historical_formula" | awk '{ print $1 }'
  )"
else
  printf 'a SHA-256 implementation is required for the supply-chain rehearsal\n' >&2
  exit 2
fi

rehearsal_verify_historical_formula() {
  local formula_source=$1
  PATH="$rehearsal_fake_bin:$PATH" \
    REHEARSAL_GH_LOG="$rehearsal_gh_log" \
    REHEARSAL_HISTORICAL_ENDPOINT="$rehearsal_historical_endpoint" \
    REHEARSAL_HISTORICAL_FORMULA="$formula_source" \
    REHEARSAL_CURRENT_FORMULA="$rehearsal_current_formula" \
    REHEARSAL_CURRENT_TAP_COMMIT="$rehearsal_current_tap_commit" \
    "$rehearsal_historical_verifier" \
    EnjoyableWork/homebrew-tap \
    "$rehearsal_historical_tap_commit" \
    Formula/mcp-doctor.rb \
    "$rehearsal_historical_formula" \
    "$rehearsal_historical_formula_sha" \
    "$rehearsal_source_url" \
    "$rehearsal_package_sha" \
    2026-03-10
}

: >"$rehearsal_gh_log"
rehearsal_verify_historical_formula "$rehearsal_historical_formula"
if [[ "$(wc -l <"$rehearsal_gh_log" | tr -d '[:space:]')" != 1 ]] ||
  ! grep -Fx "$rehearsal_historical_endpoint" "$rehearsal_gh_log" >/dev/null ||
  grep -F '/commits/main' "$rehearsal_gh_log" >/dev/null; then
  printf 'historical Homebrew verifier consulted rolling tap state\n' >&2
  exit 1
fi

: >"$rehearsal_gh_log"
if rehearsal_verify_historical_formula "$rehearsal_mutated_formula" \
  >/dev/null 2>&1; then
  printf 'historical Homebrew verifier accepted a mutated recorded formula\n' >&2
  exit 1
fi
if [[ "$(wc -l <"$rehearsal_gh_log" | tr -d '[:space:]')" != 1 ]] ||
  ! grep -Fx "$rehearsal_historical_endpoint" "$rehearsal_gh_log" >/dev/null; then
  printf 'historical Homebrew negative did not use the recorded commit\n' >&2
  exit 1
fi

printf 'Historical Homebrew evidence remains strict after rolling tap advancement.\n'
printf 'Read-only repository settings remain verifiable without contents write access.\n'
printf 'Supply-chain artifact negative exercises passed in a disposable repository.\n'
