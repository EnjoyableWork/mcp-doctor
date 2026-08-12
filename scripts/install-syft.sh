#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 1 ]]; then
  printf 'usage: %s <empty destination directory>\n' "$0" >&2
  exit 2
fi

syft_destination=$1
syft_script_directory="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
syft_repository_root="$(dirname -- "$syft_script_directory")"
syft_canonical="$syft_repository_root/.github/supply-chain-controls.json"
syft_max_attempts=3
syft_attempt_max_seconds=20
syft_retry_delay_seconds=1

case "$syft_destination" in
  /*) ;;
  *)
    printf 'Syft destination must be an absolute path\n' >&2
    exit 2
    ;;
esac
if [[ "$syft_destination" == / ]] || [[ -L "$syft_destination" ]]; then
  printf 'Syft destination is unsafe\n' >&2
  exit 2
fi
if [[ -e "$syft_destination" ]] && [[ ! -d "$syft_destination" ]]; then
  printf 'Syft destination must be a directory\n' >&2
  exit 2
fi
if [[ -e "$syft_destination" ]] &&
  [[ -n "$(find "$syft_destination" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null)" ]]; then
  printf 'Syft destination must be empty\n' >&2
  exit 2
fi
if [[ ! -f "$syft_canonical" ]] || [[ -L "$syft_canonical" ]]; then
  printf 'canonical Syft control inventory is unavailable\n' >&2
  exit 2
fi

for syft_command in curl env find grep install jq mkdir mktemp sleep tar tr uname wc; do
  if ! command -v "$syft_command" >/dev/null 2>&1; then
    printf 'required Syft installer command is unavailable: %s\n' \
      "$syft_command" >&2
    exit 2
  fi
done
if ! command -v sha256sum >/dev/null 2>&1 &&
  ! command -v shasum >/dev/null 2>&1; then
  printf 'a SHA-256 implementation is required\n' >&2
  exit 2
fi

if ! jq -e '
  [.standalone_tools[] | select(.name == "syft")] as $tools |
  ($tools | length) == 1 and
  ($tools[0] as $tool |
    $tool.version == "1.51.0" and
    $tool.repository == "anchore/syft" and
    $tool.tag == "v1.51.0" and
    $tool.release_immutable == true and
    $tool.latest_release_required == true and
    ($tool.assets | length) == 2 and
    ($tool.assets | map(.target) | sort) == [
      "aarch64-unknown-linux-gnu",
      "x86_64-unknown-linux-gnu"
    ] and
    all($tool.assets[];
      ((.archive == "syft_\($tool.version)_linux_amd64.tar.gz" and
          .target == "x86_64-unknown-linux-gnu") or
       (.archive == "syft_\($tool.version)_linux_arm64.tar.gz" and
          .target == "aarch64-unknown-linux-gnu")) and
      (.bytes | type == "number" and . > 0 and . <= 35000000) and
      (.sha256 | type == "string" and test("^[0-9a-f]{64}$"))
    )
  )
' "$syft_canonical" >/dev/null 2>&1; then
  printf 'canonical Syft control inventory is invalid\n' >&2
  exit 2
fi

if [[ "$(uname -s)" != Linux ]]; then
  printf 'Syft installer supports only reviewed GNU/Linux release hosts\n' >&2
  exit 2
fi
case "$(uname -m)" in
  x86_64)
    syft_target=x86_64-unknown-linux-gnu
    syft_platform=linux/amd64
    ;;
  aarch64 | arm64)
    syft_target=aarch64-unknown-linux-gnu
    syft_platform=linux/arm64
    ;;
  *)
    printf 'Syft installer does not support this GNU/Linux architecture\n' >&2
    exit 2
    ;;
esac

IFS=$'\t' read -r \
  syft_version syft_repository syft_tag syft_archive syft_bytes syft_sha256 < <(
    jq -er --arg target "$syft_target" '
      .standalone_tools[] | select(.name == "syft") as $tool |
      $tool.assets[] | select(.target == $target) |
      [
        $tool.version,
        $tool.repository,
        $tool.tag,
        .archive,
        (.bytes | tostring),
        .sha256
      ] | @tsv
    ' "$syft_canonical"
  )
syft_url="https://github.com/${syft_repository}/releases/download/${syft_tag}/${syft_archive}"

syft_hash() {
  local syft_hash_output

  if command -v sha256sum >/dev/null 2>&1; then
    syft_hash_output="$(sha256sum "$1")" || return 1
  else
    syft_hash_output="$(shasum -a 256 "$1")" || return 1
  fi
  printf '%s\n' "${syft_hash_output%% *}"
}

syft_retryable_download_failure() {
  local syft_curl_status=$1
  local syft_http_status=$2

  if [[ "$syft_curl_status" -eq 0 ]]; then
    case "$syft_http_status" in
      408 | 429 | 500 | 502 | 503 | 504) return 0 ;;
      *) return 1 ;;
    esac
  fi

  case "$syft_curl_status" in
    6 | 7 | 18 | 28 | 52 | 55 | 56 | 92) ;;
    *) return 1 ;;
  esac

  # A response code is stronger evidence than a simultaneous transport
  # symptom. Retry a transient curl failure only before a response, during a
  # successful response body, or for an explicitly transient response.
  case "$syft_http_status" in
    000 | 200 | 408 | 429 | 500 | 502 | 503 | 504) return 0 ;;
    *) return 1 ;;
  esac
}

umask 077
syft_temp_parent=${TMPDIR:-/tmp}
case "$syft_temp_parent" in
  /*) ;;
  *)
    printf 'Syft temporary parent must be absolute\n' >&2
    exit 2
    ;;
esac
syft_temp_root="$(mktemp -d "${syft_temp_parent%/}/mcp-doctor-syft.XXXXXX")"
syft_temp_prefix="${syft_temp_parent%/}/mcp-doctor-syft."

syft_cleanup() {
  if [[ "$syft_temp_root" != "$syft_temp_prefix"* ]]; then
    printf 'refusing to remove unexpected Syft installer path\n' >&2
    return 1
  fi
  if [[ -d "$syft_temp_root" ]]; then
    rm -rf -- "$syft_temp_root"
  fi
}
trap syft_cleanup EXIT

syft_download="$syft_temp_root/$syft_archive"
syft_curl="$(command -v curl)"
case "$syft_curl" in
  /*) ;;
  *)
    printf 'required Syft installer command is not an absolute executable: curl\n' >&2
    exit 2
    ;;
esac
syft_attempt=1
while :; do
  if [[ -e "$syft_download" ]] || [[ -L "$syft_download" ]]; then
    unlink -- "$syft_download"
  fi

  set +e
  syft_http_status="$(
    env \
      -u ALL_PROXY -u all_proxy \
      -u AWS_CA_BUNDLE \
      -u CURL_CA_BUNDLE -u CURL_HOME \
      -u HTTP_PROXY -u http_proxy \
      -u HTTPS_PROXY -u https_proxy \
      -u NETRC \
      -u NO_PROXY -u no_proxy \
      -u REQUESTS_CA_BUNDLE \
      -u SSL_CERT_DIR -u SSL_CERT_FILE -u SSLKEYLOGFILE \
      LANG=C \
      LC_ALL=C \
      "$syft_curl" --disable --silent --show-error --location \
        --proto '=https' \
        --proto-redir '=https' \
        --proxy '' \
        --retry 0 \
        --connect-timeout 10 \
        --max-time "$syft_attempt_max_seconds" \
        --max-redirs 3 \
        --max-filesize "$syft_bytes" \
        --header 'User-Agent: mcp-doctor-ci-tool-installer/0.2 (+https://github.com/EnjoyableWork/mcp-doctor)' \
        --output "$syft_download" \
        --write-out '%{http_code}' \
        "$syft_url"
  )"
  syft_curl_status=$?
  set -e

  if [[ "$syft_curl_status" -eq 0 ]] && [[ "$syft_http_status" == 200 ]]; then
    break
  fi
  if [[ -e "$syft_download" ]] || [[ -L "$syft_download" ]]; then
    unlink -- "$syft_download"
  fi
  if [[ "$syft_attempt" -ge "$syft_max_attempts" ]] ||
    ! syft_retryable_download_failure \
      "$syft_curl_status" "$syft_http_status"; then
    printf 'Syft download failed (attempt=%d curl=%s http=%s)\n' \
      "$syft_attempt" "$syft_curl_status" "$syft_http_status" >&2
    exit 1
  fi

  printf 'Transient Syft download failure on attempt %d of %d; retrying the same immutable asset.\n' \
    "$syft_attempt" "$syft_max_attempts" >&2
  sleep "$syft_retry_delay_seconds"
  syft_attempt=$((syft_attempt + 1))
done

if [[ ! -f "$syft_download" ]] || [[ -L "$syft_download" ]]; then
  printf 'Syft download did not produce a regular archive\n' >&2
  exit 1
fi
syft_observed_bytes="$(wc -c <"$syft_download" | tr -d '[:space:]')"
if [[ "$syft_observed_bytes" != "$syft_bytes" ]]; then
  printf 'Syft archive size does not match the reviewed value\n' >&2
  exit 1
fi
if [[ "$(syft_hash "$syft_download")" != "$syft_sha256" ]]; then
  printf 'Syft archive digest does not match the reviewed value\n' >&2
  exit 1
fi

syft_expected_entries='CHANGELOG.md
LICENSE
README.md
syft'
syft_observed_entries="$(
  env -u GZIP -u TAR_OPTIONS COPYFILE_DISABLE=1 LC_ALL=C \
    tar -tzf "$syft_download"
)"
if [[ "$syft_observed_entries" != "$syft_expected_entries" ]]; then
  printf 'Syft archive layout is not the reviewed layout\n' >&2
  exit 1
fi

env -u GZIP -u TAR_OPTIONS COPYFILE_DISABLE=1 LC_ALL=C \
  tar -xzf "$syft_download" -C "$syft_temp_root" syft
if ! syft_version_output="$(
  env -i \
    GOMAXPROCS=1 \
    HOME="$syft_temp_root" \
    LANG=C \
    LC_ALL=C \
    NO_COLOR=1 \
    SYFT_CHECK_FOR_APP_UPDATE=false \
    "$syft_temp_root/syft" version 2>/dev/null
)" ||
  ! grep -Eq "^Version:[[:space:]]+${syft_version//./\\.}[[:space:]]*$" \
    <<<"$syft_version_output" ||
  ! grep -Eq "^Platform:[[:space:]]+${syft_platform}[[:space:]]*$" \
    <<<"$syft_version_output"; then
  printf 'installed Syft did not report the reviewed version and platform\n' >&2
  exit 1
fi

mkdir -p -- "$syft_destination"
install -m 0755 "$syft_temp_root/syft" "$syft_destination/syft"

printf 'Installed Syft %s for %s from the reviewed immutable asset digest.\n' \
  "$syft_version" "$syft_target"
