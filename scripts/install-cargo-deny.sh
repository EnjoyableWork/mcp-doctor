#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 1 ]]; then
  printf 'usage: %s <empty destination directory>\n' "$0" >&2
  exit 2
fi

deny_destination=$1
deny_version=0.20.2
deny_target=x86_64-unknown-linux-musl
deny_archive="cargo-deny-${deny_version}-${deny_target}.tar.gz"
deny_root="cargo-deny-${deny_version}-${deny_target}"
deny_sha256=9f12ed4c49936e09b48bf862b595cde2fe64fcbd9d74dfacac6131ca824c8d5f
deny_url="https://github.com/EmbarkStudios/cargo-deny/releases/download/${deny_version}/${deny_archive}"

case "$deny_destination" in
  /*) ;;
  *)
    printf 'cargo-deny destination must be an absolute path\n' >&2
    exit 2
    ;;
esac
if [[ "$deny_destination" == / ]] || [[ -L "$deny_destination" ]]; then
  printf 'cargo-deny destination is unsafe\n' >&2
  exit 2
fi
if [[ -e "$deny_destination" ]] && [[ ! -d "$deny_destination" ]]; then
  printf 'cargo-deny destination must be a directory\n' >&2
  exit 2
fi
if [[ -e "$deny_destination" ]] &&
  [[ -n "$(find "$deny_destination" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null)" ]]; then
  printf 'cargo-deny destination must be empty\n' >&2
  exit 2
fi

for deny_command in curl install tar uname; do
  if ! command -v "$deny_command" >/dev/null 2>&1; then
    printf 'required cargo-deny installer command is unavailable: %s\n' \
      "$deny_command" >&2
    exit 2
  fi
done
if [[ "$(uname -s)" != Linux ]] || [[ "$(uname -m)" != x86_64 ]]; then
  printf 'cargo-deny installer supports only its reviewed Linux x86_64 CI target\n' >&2
  exit 2
fi

deny_hash() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print $1 }'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{ print $1 }'
  else
    printf 'a SHA-256 implementation is required\n' >&2
    return 2
  fi
}

umask 077
deny_temp_parent=${TMPDIR:-/tmp}
deny_temp_root="$(mktemp -d "${deny_temp_parent%/}/mcp-doctor-cargo-deny.XXXXXX")"
deny_temp_prefix="${deny_temp_parent%/}/mcp-doctor-cargo-deny."

deny_cleanup() {
  if [[ "$deny_temp_root" != "$deny_temp_prefix"* ]]; then
    printf 'refusing to remove unexpected cargo-deny installer path\n' >&2
    return 1
  fi
  if [[ -d "$deny_temp_root" ]]; then
    rm -rf -- "$deny_temp_root"
  fi
}
trap deny_cleanup EXIT

deny_download="$deny_temp_root/$deny_archive"
curl --disable --fail --silent --show-error --location \
  --proto '=https' \
  --proto-redir '=https' \
  --proxy '' \
  --connect-timeout 10 \
  --max-time 60 \
  --max-redirs 3 \
  --max-filesize 6000000 \
  --header 'User-Agent: mcp-doctor-ci-tool-installer/0.1 (+https://github.com/EnjoyableWork/mcp-doctor)' \
  --output "$deny_download" \
  "$deny_url"

if [[ "$(deny_hash "$deny_download")" != "$deny_sha256" ]]; then
  printf 'cargo-deny archive digest does not match the reviewed value\n' >&2
  exit 1
fi

deny_expected_entries="$deny_root/
$deny_root/README.md
$deny_root/LICENSE-APACHE
$deny_root/cargo-deny
$deny_root/LICENSE-MIT"
deny_observed_entries="$(tar -tzf "$deny_download")"
if [[ "$deny_observed_entries" != "$deny_expected_entries" ]]; then
  printf 'cargo-deny archive layout is not the reviewed layout\n' >&2
  exit 1
fi

mkdir -p -- "$deny_destination"
tar -xzf "$deny_download" \
  -C "$deny_temp_root" \
  "$deny_root/cargo-deny"
install -m 0755 "$deny_temp_root/$deny_root/cargo-deny" \
  "$deny_destination/cargo-deny"

if [[ "$("$deny_destination/cargo-deny" --version)" != "cargo-deny $deny_version" ]]; then
  printf 'installed cargo-deny did not report the reviewed version\n' >&2
  exit 1
fi

printf 'Installed cargo-deny %s from the reviewed archive digest.\n' "$deny_version"
