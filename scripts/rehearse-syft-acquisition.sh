#!/usr/bin/env bash

set -Eeuo pipefail

if [[ $# -ne 0 ]]; then
  printf 'usage: %s\n' "$0" >&2
  exit 2
fi

syft_rehearsal_script_directory="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
syft_rehearsal_case=setup

syft_rehearsal_error() {
  local syft_rehearsal_status=$?
  local syft_rehearsal_line=$1

  trap - ERR
  printf 'Syft rehearsal case failed: case=%s line=%s status=%s\n' \
    "$syft_rehearsal_case" \
    "$syft_rehearsal_line" \
    "$syft_rehearsal_status" >&2
  exit "$syft_rehearsal_status"
}
trap 'syft_rehearsal_error "$LINENO"' ERR

syft_rehearsal_temp_parent=${TMPDIR:-/tmp}
syft_rehearsal_root="$(
  mktemp -d "${syft_rehearsal_temp_parent%/}/mcp-doctor-syft-rehearsal.XXXXXX"
)"
syft_rehearsal_prefix="${syft_rehearsal_temp_parent%/}/mcp-doctor-syft-rehearsal."

syft_rehearsal_cleanup() {
  if [[ "$syft_rehearsal_root" != "$syft_rehearsal_prefix"* ]]; then
    printf 'refusing to remove unexpected Syft rehearsal path\n' >&2
    return 1
  fi
  if [[ -d "$syft_rehearsal_root" ]]; then
    rm -rf -- "$syft_rehearsal_root"
  fi
}
trap syft_rehearsal_cleanup EXIT

syft_rehearsal_repository="$syft_rehearsal_root/repository"
syft_rehearsal_assets="$syft_rehearsal_root/assets"
syft_rehearsal_fake_bin="$syft_rehearsal_root/bin"
mkdir -p -- \
  "$syft_rehearsal_repository/.github" \
  "$syft_rehearsal_repository/scripts" \
  "$syft_rehearsal_assets" \
  "$syft_rehearsal_fake_bin"
install -m 0755 \
  "$syft_rehearsal_script_directory/install-syft.sh" \
  "$syft_rehearsal_repository/scripts/install-syft.sh"
install -m 0755 \
  "$syft_rehearsal_script_directory/generate-release-sbom.sh" \
  "$syft_rehearsal_repository/scripts/generate-release-sbom.sh"

syft_rehearsal_hash() {
  local syft_rehearsal_hash_output

  if command -v sha256sum >/dev/null 2>&1; then
    syft_rehearsal_hash_output="$(sha256sum "$1")"
  else
    syft_rehearsal_hash_output="$(shasum -a 256 "$1")"
  fi
  printf '%s\n' "${syft_rehearsal_hash_output%% *}"
}

syft_rehearsal_make_archive() {
  local syft_rehearsal_archive=$1
  local syft_rehearsal_version=$2
  local syft_rehearsal_platform=$3
  local syft_rehearsal_layout=${4:-valid}
  local syft_rehearsal_scan=${5:-success}
  local syft_rehearsal_payload

  syft_rehearsal_payload="$(mktemp -d "$syft_rehearsal_root/payload.XXXXXX")"
  printf 'synthetic changelog\n' >"$syft_rehearsal_payload/CHANGELOG.md"
  printf 'synthetic license\n' >"$syft_rehearsal_payload/LICENSE"
  printf 'synthetic readme\n' >"$syft_rehearsal_payload/README.md"
  printf '%s\n' \
    '#!/bin/bash' \
    'set -euo pipefail' \
    "case \"\${1:-}\" in" \
    '  version)' \
    "    printf 'Application: syft\\nVersion:    ${syft_rehearsal_version}\\nPlatform:   ${syft_rehearsal_platform}\\n'" \
    '    ;;' \
    '  scan)' \
    "    case '$syft_rehearsal_scan' in" \
    '      success)' \
    "        printf '%s\\n' '{\"spdxVersion\":\"SPDX-2.3\",\"documentNamespace\":\"https://example.invalid/synthetic\",\"packages\":[{\"name\":\"synthetic\"}]}'" \
    '        ;;' \
    '      oversized)' \
    '        /bin/dd if=/dev/zero bs=1000001 count=10 2>/dev/null' \
    '        ;;' \
    "      *) printf '%s\\n' 'SYNTHETIC_PRIVATE_SYFT_ERROR' >&2; exit 23 ;;" \
    '    esac' \
    '    ;;' \
    '  *) exit 2 ;;' \
    'esac' >"$syft_rehearsal_payload/syft"
  chmod 0755 "$syft_rehearsal_payload/syft"

  if [[ "$syft_rehearsal_layout" == valid ]]; then
    COPYFILE_DISABLE=1 tar -czf "$syft_rehearsal_archive" \
      -C "$syft_rehearsal_payload" \
      CHANGELOG.md LICENSE README.md syft
  else
    printf 'unexpected\n' >"$syft_rehearsal_payload/UNEXPECTED"
    COPYFILE_DISABLE=1 tar -czf "$syft_rehearsal_archive" \
      -C "$syft_rehearsal_payload" \
      CHANGELOG.md LICENSE README.md syft UNEXPECTED
  fi
}

syft_rehearsal_x64="$syft_rehearsal_assets/syft_1.51.0_linux_amd64.tar.gz"
syft_rehearsal_arm64="$syft_rehearsal_assets/syft_1.51.0_linux_arm64.tar.gz"
syft_rehearsal_wrong_digest="$syft_rehearsal_assets/wrong-digest.tar.gz"
syft_rehearsal_wrong_layout="$syft_rehearsal_assets/wrong-layout.tar.gz"
syft_rehearsal_wrong_version="$syft_rehearsal_assets/wrong-version.tar.gz"
syft_rehearsal_wrong_platform="$syft_rehearsal_assets/wrong-platform.tar.gz"
syft_rehearsal_failed_scan="$syft_rehearsal_assets/failed-scan.tar.gz"
syft_rehearsal_oversized_scan="$syft_rehearsal_assets/oversized-scan.tar.gz"
syft_rehearsal_make_archive "$syft_rehearsal_x64" 1.51.0 linux/amd64
syft_rehearsal_make_archive "$syft_rehearsal_arm64" 1.51.0 linux/arm64
cp "$syft_rehearsal_x64" "$syft_rehearsal_wrong_digest"
printf 'X' | dd \
  of="$syft_rehearsal_wrong_digest" bs=1 seek=0 conv=notrunc 2>/dev/null
syft_rehearsal_make_archive \
  "$syft_rehearsal_wrong_layout" 1.51.0 linux/amd64 invalid
syft_rehearsal_make_archive \
  "$syft_rehearsal_wrong_version" 1.49.0 linux/amd64
syft_rehearsal_make_archive \
  "$syft_rehearsal_wrong_platform" 1.51.0 linux/arm64
syft_rehearsal_make_archive \
  "$syft_rehearsal_failed_scan" 1.51.0 linux/amd64 valid failure
syft_rehearsal_make_archive \
  "$syft_rehearsal_oversized_scan" 1.51.0 linux/amd64 valid oversized

syft_rehearsal_write_controls() {
  local syft_rehearsal_selected_x64=$1
  local syft_rehearsal_selected_arm64=$2
  local syft_rehearsal_x64_bytes syft_rehearsal_arm64_bytes
  local syft_rehearsal_x64_sha syft_rehearsal_arm64_sha

  syft_rehearsal_x64_bytes="$(wc -c <"$syft_rehearsal_selected_x64" | tr -d '[:space:]')"
  syft_rehearsal_arm64_bytes="$(wc -c <"$syft_rehearsal_selected_arm64" | tr -d '[:space:]')"
  syft_rehearsal_x64_sha="$(syft_rehearsal_hash "$syft_rehearsal_selected_x64")"
  syft_rehearsal_arm64_sha="$(syft_rehearsal_hash "$syft_rehearsal_selected_arm64")"

  jq -n \
    --arg x64_sha "$syft_rehearsal_x64_sha" \
    --argjson x64_bytes "$syft_rehearsal_x64_bytes" \
    --arg arm64_sha "$syft_rehearsal_arm64_sha" \
    --argjson arm64_bytes "$syft_rehearsal_arm64_bytes" '
      {
        standalone_tools: [
          {
            name: "syft",
            version: "1.51.0",
            repository: "anchore/syft",
            tag: "v1.51.0",
            release_immutable: true,
            latest_release_required: true,
            assets: [
              {
                target: "x86_64-unknown-linux-gnu",
                archive: "syft_1.51.0_linux_amd64.tar.gz",
                bytes: $x64_bytes,
                sha256: $x64_sha
              },
              {
                target: "aarch64-unknown-linux-gnu",
                archive: "syft_1.51.0_linux_arm64.tar.gz",
                bytes: $arm64_bytes,
                sha256: $arm64_sha
              }
            ]
          }
        ]
      }
    ' >"$syft_rehearsal_repository/.github/supply-chain-controls.json"
}

cat >"$syft_rehearsal_fake_bin/uname" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
  -s) printf '%s\n' "${FAKE_UNAME_SYSTEM:?}" ;;
  -m) printf '%s\n' "${FAKE_UNAME_ARCH:?}" ;;
  *) exit 2 ;;
esac
EOF

cat >"$syft_rehearsal_fake_bin/sleep" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
sleep_count=0
if [[ -f "${FAKE_SLEEP_STATE:?}" ]]; then
  sleep_count="$(<"$FAKE_SLEEP_STATE")"
fi
printf '%d\n' "$((sleep_count + 1))" >"$FAKE_SLEEP_STATE"
EOF

cat >"$syft_rehearsal_fake_bin/timeout" <<'EOF'
#!/bin/bash
set -euo pipefail
test "$1" = --signal=TERM
test "$2" = --kill-after=5s
test "$3" = 120s
shift 3
exec "$@"
EOF

cat >"$syft_rehearsal_fake_bin/curl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
fake_output=
fake_url=
while [[ $# -gt 0 ]]; do
  case "$1" in
    --output)
      fake_output=$2
      shift 2
      ;;
    https://*)
      fake_url=$1
      shift
      ;;
    *) shift ;;
  esac
done
if [[ -z "$fake_output" ]] || [[ -z "$fake_url" ]]; then
  exit 2
fi

fake_attempt=0
if [[ -f "${FAKE_CURL_STATE:?}" ]]; then
  fake_attempt="$(<"$FAKE_CURL_STATE")"
fi
fake_attempt=$((fake_attempt + 1))
printf '%d\n' "$fake_attempt" >"$FAKE_CURL_STATE"

if [[ "$fake_url" != "${FAKE_EXPECTED_URL:?}" ]]; then
  printf '000'
  exit 2
fi

case "${FAKE_CURL_SCENARIO:?}" in
  success)
    install -m 0600 "${FAKE_CURL_ARCHIVE:?}" "$fake_output"
    printf '200'
    ;;
  transient-http)
    if [[ "$fake_attempt" -eq 1 ]]; then
      printf 'synthetic transient response\n' >"$fake_output"
      printf '%s' "${FAKE_HTTP_STATUS:?}"
    else
      install -m 0600 "${FAKE_CURL_ARCHIVE:?}" "$fake_output"
      printf '200'
    fi
    ;;
  transient-curl)
    if [[ "$fake_attempt" -eq 1 ]]; then
      printf 'synthetic partial response\n' >"$fake_output"
      printf '000'
      exit "${FAKE_CURL_EXIT:?}"
    else
      install -m 0600 "${FAKE_CURL_ARCHIVE:?}" "$fake_output"
      printf '200'
    fi
    ;;
  partial-success)
    if [[ "$fake_attempt" -eq 1 ]]; then
      printf 'synthetic partial response\n' >"$fake_output"
      printf '200'
      exit "${FAKE_CURL_EXIT:?}"
    else
      install -m 0600 "${FAKE_CURL_ARCHIVE:?}" "$fake_output"
      printf '200'
    fi
    ;;
  exhausted)
    printf 'synthetic transient response\n' >"$fake_output"
    printf '503'
    ;;
  permanent-http)
    printf 'synthetic permanent response\n' >"$fake_output"
    printf '%s' "${FAKE_HTTP_STATUS:?}"
    ;;
  permanent-curl)
    printf '000'
    exit "${FAKE_CURL_EXIT:?}"
    ;;
  permanent-http-transient-curl)
    printf 'synthetic permanent response\n' >"$fake_output"
    printf '%s' "${FAKE_HTTP_STATUS:?}"
    exit "${FAKE_CURL_EXIT:?}"
    ;;
  transient-http-permanent-curl)
    printf 'synthetic transient response\n' >"$fake_output"
    printf '%s' "${FAKE_HTTP_STATUS:?}"
    exit "${FAKE_CURL_EXIT:?}"
    ;;
  *) exit 2 ;;
esac
EOF
chmod 0755 \
  "$syft_rehearsal_fake_bin/uname" \
  "$syft_rehearsal_fake_bin/sleep" \
  "$syft_rehearsal_fake_bin/timeout" \
  "$syft_rehearsal_fake_bin/curl"

syft_rehearsal_read_count() {
  if [[ -f "$1" ]]; then
    tr -d '[:space:]' <"$1"
  else
    printf '0'
  fi
}

syft_rehearsal_run_installer() {
  local syft_rehearsal_name=$1
  local syft_rehearsal_arch=$2
  local syft_rehearsal_archive=$3
  local syft_rehearsal_scenario=$4
  local syft_rehearsal_expected_url=$5
  local syft_rehearsal_destination="$syft_rehearsal_root/destination-$syft_rehearsal_name"
  local syft_rehearsal_curl_state="$syft_rehearsal_root/curl-$syft_rehearsal_name"
  local syft_rehearsal_sleep_state="$syft_rehearsal_root/sleep-$syft_rehearsal_name"

  syft_rehearsal_case="$syft_rehearsal_name"

  env \
    FAKE_CURL_ARCHIVE="$syft_rehearsal_archive" \
    FAKE_CURL_SCENARIO="$syft_rehearsal_scenario" \
    FAKE_CURL_STATE="$syft_rehearsal_curl_state" \
    FAKE_EXPECTED_URL="$syft_rehearsal_expected_url" \
    FAKE_HTTP_STATUS="${FAKE_HTTP_STATUS:-503}" \
    FAKE_CURL_EXIT="${FAKE_CURL_EXIT:-56}" \
    FAKE_SLEEP_STATE="$syft_rehearsal_sleep_state" \
    FAKE_UNAME_ARCH="$syft_rehearsal_arch" \
    FAKE_UNAME_SYSTEM="${FAKE_UNAME_SYSTEM:-Linux}" \
    PATH="$syft_rehearsal_fake_bin:$PATH" \
    "$syft_rehearsal_repository/scripts/install-syft.sh" \
      "$syft_rehearsal_destination" >/dev/null 2>&1
}

syft_rehearsal_expect_failure() {
  if syft_rehearsal_run_installer "$@"; then
    printf 'Syft installer unexpectedly accepted %s\n' "$1" >&2
    exit 1
  fi
}

syft_rehearsal_x64_url='https://github.com/anchore/syft/releases/download/v1.51.0/syft_1.51.0_linux_amd64.tar.gz'
syft_rehearsal_arm64_url='https://github.com/anchore/syft/releases/download/v1.51.0/syft_1.51.0_linux_arm64.tar.gz'
syft_rehearsal_write_controls "$syft_rehearsal_x64" "$syft_rehearsal_arm64"

syft_rehearsal_run_installer \
  success-x64 x86_64 "$syft_rehearsal_x64" success "$syft_rehearsal_x64_url"
syft_rehearsal_run_installer \
  success-arm64 aarch64 "$syft_rehearsal_arm64" success "$syft_rehearsal_arm64_url"

for syft_rehearsal_status in 408 429 500 502 503 504; do
  FAKE_HTTP_STATUS=$syft_rehearsal_status \
    syft_rehearsal_run_installer \
      "http-$syft_rehearsal_status" x86_64 "$syft_rehearsal_x64" \
      transient-http "$syft_rehearsal_x64_url"
  test "$(syft_rehearsal_read_count "$syft_rehearsal_root/curl-http-$syft_rehearsal_status")" = 2
  test "$(syft_rehearsal_read_count "$syft_rehearsal_root/sleep-http-$syft_rehearsal_status")" = 1
done

for syft_rehearsal_exit in 6 7 18 28 52 55 56 92; do
  FAKE_CURL_EXIT=$syft_rehearsal_exit \
    syft_rehearsal_run_installer \
      "curl-$syft_rehearsal_exit" x86_64 "$syft_rehearsal_x64" \
      transient-curl "$syft_rehearsal_x64_url"
  test "$(syft_rehearsal_read_count "$syft_rehearsal_root/curl-curl-$syft_rehearsal_exit")" = 2
done

FAKE_CURL_EXIT=18 \
  syft_rehearsal_run_installer \
    partial-success x86_64 "$syft_rehearsal_x64" \
    partial-success "$syft_rehearsal_x64_url"
test "$(syft_rehearsal_read_count "$syft_rehearsal_root/curl-partial-success")" = 2

syft_rehearsal_expect_failure \
  exhausted x86_64 "$syft_rehearsal_x64" exhausted "$syft_rehearsal_x64_url"
test "$(syft_rehearsal_read_count "$syft_rehearsal_root/curl-exhausted")" = 3
test "$(syft_rehearsal_read_count "$syft_rehearsal_root/sleep-exhausted")" = 2

for syft_rehearsal_status in 400 401 403 404 501; do
  FAKE_HTTP_STATUS=$syft_rehearsal_status \
    syft_rehearsal_expect_failure \
      "permanent-http-$syft_rehearsal_status" x86_64 \
      "$syft_rehearsal_x64" permanent-http "$syft_rehearsal_x64_url"
  test "$(syft_rehearsal_read_count "$syft_rehearsal_root/curl-permanent-http-$syft_rehearsal_status")" = 1
done

for syft_rehearsal_exit in 35 60 63 77; do
  FAKE_CURL_EXIT=$syft_rehearsal_exit \
    syft_rehearsal_expect_failure \
      "permanent-curl-$syft_rehearsal_exit" x86_64 \
      "$syft_rehearsal_x64" permanent-curl "$syft_rehearsal_x64_url"
  test "$(syft_rehearsal_read_count "$syft_rehearsal_root/curl-permanent-curl-$syft_rehearsal_exit")" = 1
done

FAKE_HTTP_STATUS=404 FAKE_CURL_EXIT=56 \
  syft_rehearsal_expect_failure \
    permanent-http-with-transient-curl x86_64 "$syft_rehearsal_x64" \
    permanent-http-transient-curl "$syft_rehearsal_x64_url"
test "$(syft_rehearsal_read_count \
  "$syft_rehearsal_root/curl-permanent-http-with-transient-curl")" = 1

FAKE_HTTP_STATUS=503 FAKE_CURL_EXIT=60 \
  syft_rehearsal_expect_failure \
    transient-http-with-trust-failure x86_64 "$syft_rehearsal_x64" \
    transient-http-permanent-curl "$syft_rehearsal_x64_url"
test "$(syft_rehearsal_read_count \
  "$syft_rehearsal_root/curl-transient-http-with-trust-failure")" = 1

syft_rehearsal_expect_failure \
  wrong-digest x86_64 "$syft_rehearsal_wrong_digest" \
  success "$syft_rehearsal_x64_url"
test "$(syft_rehearsal_read_count "$syft_rehearsal_root/curl-wrong-digest")" = 1

syft_rehearsal_write_controls \
  "$syft_rehearsal_wrong_layout" "$syft_rehearsal_arm64"
syft_rehearsal_expect_failure \
  wrong-layout x86_64 "$syft_rehearsal_wrong_layout" \
  success "$syft_rehearsal_x64_url"
test "$(syft_rehearsal_read_count "$syft_rehearsal_root/curl-wrong-layout")" = 1

syft_rehearsal_write_controls \
  "$syft_rehearsal_wrong_version" "$syft_rehearsal_arm64"
syft_rehearsal_expect_failure \
  wrong-version x86_64 "$syft_rehearsal_wrong_version" \
  success "$syft_rehearsal_x64_url"
test "$(syft_rehearsal_read_count "$syft_rehearsal_root/curl-wrong-version")" = 1

syft_rehearsal_write_controls \
  "$syft_rehearsal_wrong_platform" "$syft_rehearsal_arm64"
syft_rehearsal_expect_failure \
  wrong-platform x86_64 "$syft_rehearsal_wrong_platform" \
  success "$syft_rehearsal_x64_url"
test "$(syft_rehearsal_read_count "$syft_rehearsal_root/curl-wrong-platform")" = 1

syft_rehearsal_write_controls "$syft_rehearsal_x64" "$syft_rehearsal_arm64"
FAKE_UNAME_SYSTEM=Darwin syft_rehearsal_expect_failure \
  unsupported-system x86_64 "$syft_rehearsal_x64" success "$syft_rehearsal_x64_url"
test "$(syft_rehearsal_read_count "$syft_rehearsal_root/curl-unsupported-system")" = 0
syft_rehearsal_expect_failure \
  unsupported-architecture riscv64 "$syft_rehearsal_x64" success "$syft_rehearsal_x64_url"
test "$(syft_rehearsal_read_count "$syft_rehearsal_root/curl-unsupported-architecture")" = 0

syft_rehearsal_release_archive="$syft_rehearsal_root/release.tar.gz"
syft_rehearsal_sbom="$syft_rehearsal_root/release.spdx.json"
printf 'synthetic release bytes\n' >"$syft_rehearsal_root/release"
COPYFILE_DISABLE=1 tar -czf "$syft_rehearsal_release_archive" \
  -C "$syft_rehearsal_root" release
syft_rehearsal_case=generation-success
env \
  FAKE_CURL_ARCHIVE="$syft_rehearsal_x64" \
  FAKE_CURL_SCENARIO=success \
  FAKE_CURL_STATE="$syft_rehearsal_root/curl-generation" \
  FAKE_EXPECTED_URL="$syft_rehearsal_x64_url" \
  FAKE_HTTP_STATUS=503 \
  FAKE_CURL_EXIT=56 \
  FAKE_SLEEP_STATE="$syft_rehearsal_root/sleep-generation" \
  FAKE_UNAME_ARCH=x86_64 \
  FAKE_UNAME_SYSTEM=Linux \
  PATH="$syft_rehearsal_fake_bin:$PATH" \
  "$syft_rehearsal_repository/scripts/generate-release-sbom.sh" \
    "$syft_rehearsal_release_archive" "$syft_rehearsal_sbom" >/dev/null
jq -e '
  .spdxVersion == "SPDX-2.3" and
  .documentNamespace == "https://example.invalid/synthetic" and
  (.packages | length) == 1
' "$syft_rehearsal_sbom" >/dev/null

syft_rehearsal_write_controls \
  "$syft_rehearsal_failed_scan" "$syft_rehearsal_arm64"
syft_rehearsal_case=generation-failure
syft_rehearsal_failed_sbom="$syft_rehearsal_root/failed.spdx.json"
syft_rehearsal_failed_stderr="$syft_rehearsal_root/failed.stderr"
if env \
  FAKE_CURL_ARCHIVE="$syft_rehearsal_failed_scan" \
  FAKE_CURL_SCENARIO=success \
  FAKE_CURL_STATE="$syft_rehearsal_root/curl-generation-failure" \
  FAKE_EXPECTED_URL="$syft_rehearsal_x64_url" \
  FAKE_HTTP_STATUS=503 \
  FAKE_CURL_EXIT=56 \
  FAKE_SLEEP_STATE="$syft_rehearsal_root/sleep-generation-failure" \
  FAKE_UNAME_ARCH=x86_64 \
  FAKE_UNAME_SYSTEM=Linux \
  PATH="$syft_rehearsal_fake_bin:$PATH" \
  "$syft_rehearsal_repository/scripts/generate-release-sbom.sh" \
    "$syft_rehearsal_release_archive" "$syft_rehearsal_failed_sbom" \
    >/dev/null 2>"$syft_rehearsal_failed_stderr"; then
  printf 'SBOM generation failure was unexpectedly accepted\n' >&2
  exit 1
fi
test "$(syft_rehearsal_read_count "$syft_rehearsal_root/curl-generation-failure")" = 1
test ! -e "$syft_rehearsal_failed_sbom"
grep -Fx 'Syft failed to generate the release SBOM' \
  "$syft_rehearsal_failed_stderr" >/dev/null
if grep -F 'SYNTHETIC_PRIVATE_SYFT_ERROR' \
  "$syft_rehearsal_failed_stderr" >/dev/null; then
  printf 'SBOM generation exposed private Syft stderr\n' >&2
  exit 1
fi

syft_rehearsal_write_controls \
  "$syft_rehearsal_oversized_scan" "$syft_rehearsal_arm64"
syft_rehearsal_case=generation-oversized
syft_rehearsal_oversized_sbom="$syft_rehearsal_root/oversized.spdx.json"
if env \
  FAKE_CURL_ARCHIVE="$syft_rehearsal_oversized_scan" \
  FAKE_CURL_SCENARIO=success \
  FAKE_CURL_STATE="$syft_rehearsal_root/curl-generation-oversized" \
  FAKE_EXPECTED_URL="$syft_rehearsal_x64_url" \
  FAKE_HTTP_STATUS=503 \
  FAKE_CURL_EXIT=56 \
  FAKE_SLEEP_STATE="$syft_rehearsal_root/sleep-generation-oversized" \
  FAKE_UNAME_ARCH=x86_64 \
  FAKE_UNAME_SYSTEM=Linux \
  PATH="$syft_rehearsal_fake_bin:$PATH" \
  "$syft_rehearsal_repository/scripts/generate-release-sbom.sh" \
    "$syft_rehearsal_release_archive" "$syft_rehearsal_oversized_sbom" \
    >/dev/null 2>&1; then
  printf 'Oversized SBOM generation was unexpectedly accepted\n' >&2
  exit 1
fi
test "$(syft_rehearsal_read_count "$syft_rehearsal_root/curl-generation-oversized")" = 1
test ! -e "$syft_rehearsal_oversized_sbom"

syft_rehearsal_empty_input="$syft_rehearsal_root/empty-release.tar.gz"
syft_rehearsal_empty_output="$syft_rehearsal_root/empty.spdx.json"
syft_rehearsal_case=generation-empty-input
: >"$syft_rehearsal_empty_input"
if PATH="$syft_rehearsal_fake_bin:$PATH" \
  "$syft_rehearsal_repository/scripts/generate-release-sbom.sh" \
    "$syft_rehearsal_empty_input" "$syft_rehearsal_empty_output" \
    >/dev/null 2>&1; then
  printf 'Empty SBOM input was unexpectedly accepted\n' >&2
  exit 1
fi
test ! -e "$syft_rehearsal_empty_output"

printf 'Syft acquisition and SBOM generation rehearsals passed offline.\n'
