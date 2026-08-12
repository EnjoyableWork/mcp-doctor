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

printf 'Supply-chain artifact negative exercises passed in a disposable repository.\n'
