#!/usr/bin/env bash

set -euo pipefail

if [[ $# -gt 1 ]]; then
  printf 'usage: %s [--worktree|tree-ish]\n' "$0" >&2
  exit 2
fi

artifact_script_directory="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
artifact_repository_root="$(dirname -- "$artifact_script_directory")"
artifact_source=${1:---worktree}

for artifact_command in cmp curl git iconv od sed tr; do
  if ! command -v "$artifact_command" >/dev/null 2>&1; then
    printf 'required source-artifact command is unavailable: %s\n' \
      "$artifact_command" >&2
    exit 2
  fi
done

cd -- "$artifact_repository_root"
if [[ "$artifact_source" != --worktree ]]; then
  if ! git rev-parse --verify "${artifact_source}^{tree}" >/dev/null 2>&1; then
    printf 'source-artifact tree-ish does not resolve to a Git tree\n' >&2
    exit 2
  fi
fi

artifact_exercise_fork_write_negative() {
  if [[ "${GITHUB_ACTIONS-}" != true ]] ||
    [[ "${GITHUB_EVENT_NAME-}" != pull_request ]] ||
    [[ "${GITHUB_REPOSITORY-}" != EnjoyableWork/mcp-doctor ]]; then
    return 0
  fi

  if [[ -n "${GITHUB_TOKEN-}" ]] || [[ -n "${GH_TOKEN-}" ]] ||
    [[ -n "${ACTIONS_ID_TOKEN_REQUEST_TOKEN-}" ]] ||
    [[ -n "${ACTIONS_ID_TOKEN_REQUEST_URL-}" ]]; then
    printf 'fork exercise found repository or OIDC authority in untrusted code\n' >&2
    return 1
  fi
  if git config --local --get-regexp '^http\..*\.extraheader$' \
    >/dev/null 2>&1; then
    printf 'fork exercise found persisted checkout authorization\n' >&2
    return 1
  fi

  local exercise_status
  exercise_status="$(
    env \
      -u ALL_PROXY -u all_proxy \
      -u HTTP_PROXY -u http_proxy \
      -u HTTPS_PROXY -u https_proxy \
      -u NO_PROXY -u no_proxy \
      curl --disable --silent --show-error \
        --proto '=https' \
        --proto-redir '=https' \
        --proxy '' \
        --connect-timeout 10 \
        --max-time 20 \
        --max-redirs 0 \
        --output /dev/null \
        --write-out '%{http_code}' \
        --request POST \
        --header 'Accept: application/vnd.github+json' \
        --header 'Content-Type: application/json' \
        --header 'User-Agent: mcp-doctor-fork-write-negative/0.1' \
        --data '{"ref":"refs/heads/mcpd-016-fork-write-negative-must-not-exist","sha":"0000000000000000000000000000000000000000"}' \
        https://api.github.com/repos/EnjoyableWork/mcp-doctor/git/refs
  )"
  if [[ "$exercise_status" != 401 ]] && [[ "$exercise_status" != 404 ]]; then
    printf 'fork exercise did not receive the expected unauthenticated write rejection\n' >&2
    return 1
  fi

  printf 'MCPD-016 fork write-negative passed: no repository/OIDC token, no persisted checkout authorization, and repository write rejected.\n'
}

artifact_exercise_fork_write_negative

umask 077
artifact_temp_parent=${TMPDIR:-/tmp}
artifact_temp_root="$(mktemp -d "${artifact_temp_parent%/}/mcp-doctor-source-artifacts.XXXXXX")"
artifact_temp_prefix="${artifact_temp_parent%/}/mcp-doctor-source-artifacts."

artifact_cleanup() {
  if [[ "$artifact_temp_root" != "$artifact_temp_prefix"* ]]; then
    printf 'refusing to remove unexpected source-artifact path\n' >&2
    return 1
  fi
  if [[ -d "$artifact_temp_root" ]]; then
    rm -rf -- "$artifact_temp_root"
  fi
}
trap artifact_cleanup EXIT

artifact_count=0

artifact_reject() {
  local path=$1
  local reason=$2
  printf 'source artifact rejected: %s: %s\n' "$path" "$reason" >&2
  return 1
}

artifact_check_blob() {
  local path=$1
  local mode=$2
  local blob=$3
  local lower_path magic

  if [[ "$path" == *$'\n'* ]] || [[ "$path" == *$'\r'* ]] ||
    [[ "$path" == *$'\t'* ]]; then
    artifact_reject "$path" 'control characters in a tracked path are not reviewable'
    return 1
  fi

  lower_path="$(printf '%s' "$path" | tr '[:upper:]' '[:lower:]')"
  case "$lower_path" in
    *.7z | *.a | *.app | *.bin | *.bz2 | *.class | *.crate | *.dmg | *.dll | \
      *.dylib | *.egg | *.elf | *.exe | *.gem | *.gz | *.iso | *.jar | \
      *.lib | *.msi | *.o | *.obj | *.pdb | *.pkg | *.pyc | *.rlib | *.so | \
      *.tar | *.tgz | *.wasm | *.whl | *.xz | *.zip)
      artifact_reject "$path" 'generated executable, library, package, or archive extension'
      return 1
      ;;
  esac

  if ! cmp -s "$blob" <(LC_ALL=C tr -d '\000' <"$blob"); then
    artifact_reject "$path" 'NUL-bearing binary content is not reviewable source'
    return 1
  fi

  if ! iconv -f UTF-8 -t UTF-8 "$blob" >/dev/null 2>&1; then
    artifact_reject "$path" 'non-UTF-8 content is not reviewable source text'
    return 1
  fi

  if ! cmp -s "$blob" <(
    LC_ALL=C tr -d '\001-\010\013\014\016-\037\177' <"$blob"
  ); then
    artifact_reject "$path" 'disallowed ASCII control bytes are not reviewable source text'
    return 1
  fi

  magic="$(od -An -tx1 -N8 "$blob" | tr -d '[:space:]')"
  case "$magic" in
    7f454c46* | 4d5a* | feedface* | feedfacf* | cefaedfe* | cffaedfe* | \
      cafebabe* | bebafeca* | 0061736d* | 213c617263683e0a* | 1f8b* | \
      425a68* | fd377a585a00* | 504b0304* | 504b0506* | 504b0708* | \
      25504446* | 89504e470d0a1a0a* | ffd8ff* | 474946383761* | \
      474946383961*)
      artifact_reject "$path" 'executable, archive, document, or binary media signature'
      return 1
      ;;
  esac

  if [[ "$(sed -n '1p' "$blob")" == \
    'version https://git-lfs.github.com/spec/v1' ]]; then
    artifact_reject "$path" 'Git LFS pointer hides content outside reviewable history'
    return 1
  fi

  if [[ "$mode" == 100755 ]]; then
    if [[ "$path" != scripts/*.sh ]] ||
      [[ "$(od -An -tx1 -N2 "$blob" | tr -d '[:space:]')" != 2321 ]]; then
      artifact_reject "$path" 'executable mode is reserved for reviewable scripts/*.sh source'
      return 1
    fi
  elif [[ "$mode" != 100644 ]]; then
    artifact_reject "$path" "unsupported Git mode $mode"
    return 1
  fi

  artifact_count=$((artifact_count + 1))
}

if [[ "$artifact_source" == --worktree ]]; then
  while IFS= read -r -d '' artifact_path; do
    if [[ -L "$artifact_path" ]] || [[ ! -f "$artifact_path" ]]; then
      artifact_reject "$artifact_path" 'only regular source files may be tracked'
      exit 1
    fi
    artifact_mode=100644
    if [[ -x "$artifact_path" ]]; then
      artifact_mode=100755
    fi
    artifact_check_blob "$artifact_path" "$artifact_mode" "$artifact_path"
  done < <(git ls-files --cached --others --exclude-standard -z)
else
  while IFS= read -r -d '' artifact_entry; do
    artifact_metadata=${artifact_entry%%$'\t'*}
    artifact_path=${artifact_entry#*$'\t'}
    read -r artifact_mode artifact_type artifact_oid <<<"$artifact_metadata"
    if [[ "$artifact_type" != blob ]]; then
      artifact_reject "$artifact_path" 'only regular source blobs may be tracked'
      exit 1
    fi
    artifact_blob="$artifact_temp_root/blob"
    git cat-file blob "$artifact_oid" >"$artifact_blob"
    artifact_check_blob "$artifact_path" "$artifact_mode" "$artifact_blob"
  done < <(git ls-tree -r -z --full-tree "$artifact_source")
fi

if ((artifact_count == 0)); then
  printf 'source-artifact verification found no files\n' >&2
  exit 1
fi

printf 'Verified %d reviewable source files with no generated executable or binary artifact.\n' \
  "$artifact_count"
