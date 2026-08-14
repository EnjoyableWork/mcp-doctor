#!/usr/bin/env bash

set -euo pipefail

compat_script_directory="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
  pwd
)"
compat_repository_root="$(
  cd -- "${compat_script_directory}/.."
  pwd
)"
compat_matrix="${compat_repository_root}/tests/compatibility/matrix.json"

compat_require_command() {
  local command_name="$1"

  if ! command -v "${command_name}" >/dev/null 2>&1; then
    printf 'Compatibility check requires %s.\n' "${command_name}" >&2
    exit 1
  fi
}

for compat_command in cargo docker git jq; do
  compat_require_command "${compat_command}"
done

compat_sha256() {
  local file_path="$1"

  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${file_path}" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${file_path}" | awk '{print $1}'
  else
    printf 'Compatibility check requires sha256sum or shasum.\n' >&2
    exit 1
  fi
}

compat_runtime_value() {
  local runtime_id="$1"
  local field="$2"

  jq -er --arg runtime_id "${runtime_id}" --arg field "${field}" \
    '.runtimes[$runtime_id][$field]' "${compat_matrix}"
}

compat_case_value() {
  local case_id="$1"
  local field="$2"

  jq -er --arg case_id "${case_id}" --arg field "${field}" \
    '.cases[] | select(.id == $case_id) | .[$field]' "${compat_matrix}"
}

compat_case_lock_value() {
  local case_id="$1"
  local field="$2"

  jq -er --arg case_id "${case_id}" --arg field "${field}" \
    '.cases[] | select(.id == $case_id) | .dependency_lock[$field]' \
    "${compat_matrix}"
}

compat_active_case_value() {
  local case_id="$1"
  local field="$2"

  jq -er --arg case_id "${case_id}" --arg field "${field}" \
    '.active_legacy.cases[] | select(.id == $case_id) | .[$field]' \
    "${compat_matrix}"
}

compat_clone_case() {
  local case_id="$1"
  local destination="$2"
  local repository
  local release
  local expected_commit
  local actual_commit

  repository="$(compat_case_value "${case_id}" repository)"
  release="$(compat_case_value "${case_id}" release)"
  expected_commit="$(compat_case_value "${case_id}" commit)"

  if ! git clone --quiet --depth 1 --branch "${release}" \
    "${repository}" "${destination}" >>"${compat_prepare_log}" 2>&1; then
    printf 'Could not clone %s; preparation log follows:\n' "${case_id}" >&2
    tail -n 100 "${compat_prepare_log}" >&2
    exit 1
  fi
  actual_commit="$(git -C "${destination}" rev-parse HEAD)"
  if [[ "${actual_commit}" != "${expected_commit}" ]]; then
    printf '%s resolved to %s; expected %s.\n' \
      "${case_id}" "${actual_commit}" "${expected_commit}" >&2
    exit 1
  fi
}

compat_verify_lock() {
  local case_id="$1"
  local file_path="$2"
  local expected_hash
  local actual_hash

  expected_hash="$(compat_case_lock_value "${case_id}" sha256)"
  actual_hash="$(compat_sha256 "${file_path}")"
  if [[ "${actual_hash}" != "${expected_hash}" ]]; then
    printf '%s lock digest was %s; expected %s.\n' \
      "${case_id}" "${actual_hash}" "${expected_hash}" >&2
    exit 1
  fi
}

compat_assert_report() {
  local case_id="$1"
  local report_path="$2"

  if ! jq -e '
    .schema_version == "mcp-doctor.report/v1" and
    .schema_stability == "stable" and
    .protocol_revision == "2026-07-28" and
    .primary_diagnosis == null and
    .independent_findings == [] and
    .outcome == "passed" and
    .exit_code == 0 and
    .summary.required == 5 and
    .summary.required_skipped == 0 and
    .summary.failed == 0 and
    ([.checks[] | select(.requirement == "required") |
      (.state == "performed" and .outcome == "passed")] | all) and
    ([.checks[] | select(.id == "runtime.tools")] | length) == 1 and
    ([.checks[] | select(.id == "runtime.tools") |
      (.state == "skipped" and .skip_reason == "not_authorized")] | all)
  ' "${report_path}" >/dev/null; then
    printf '%s returned an unexpected diagnostic report:\n' "${case_id}" >&2
    jq '{schema_version, schema_stability, protocol_revision,
      primary_diagnosis, independent_findings, outcome, exit_code,
      summary, checks}' "${report_path}" >&2
    exit 1
  fi
}

compat_assert_active_report() {
  local case_id="$1"
  local command_name="$2"
  local report_path="$3"
  local expected_cases="$4"
  local expected_required

  if [[ "${command_name}" == "break" ]]; then
    expected_required="$((expected_cases + 8))"
  else
    expected_required="$((expected_cases + 7))"
  fi
  if ! jq -e \
    --arg command_name "${command_name}" \
    --argjson expected_cases "${expected_cases}" \
    --argjson expected_required "${expected_required}" '
    .schema_version == "mcp-doctor.report/v1" and
    .schema_stability == "stable" and
    .protocol_revision == "2025-11-25" and
    .negotiated_protocol_revision == "2025-11-25" and
    .primary_diagnosis == null and
    .independent_findings == [] and
    .outcome == "passed" and
    .exit_code == 0 and
    .summary.required == $expected_required and
    .summary.required_skipped == 0 and
    .summary.failed == 0 and
    ([.checks[] | select(.requirement == "required") |
      (.state == "performed" and .outcome == "passed")] | all) and
    ([.checks[] | select(.id | startswith("runtime.tools.case["))] | length) ==
      $expected_cases and
    ([.checks[] | select(.id | startswith("runtime.tools.case[")) |
      (.state == "performed" and .outcome == "passed")] | all) and
    (if $command_name == "break" then
      ([.checks[] | select(.id == "generation.cases") |
        (.state == "performed" and .outcome == "passed")] | all) and
      ([.checks[] | select(.id == "generation.cases")] | length) == 1
    else
      ([.checks[] | select(.id == "generation.cases")] | length) == 0
    end)
  ' "${report_path}" >/dev/null; then
    printf '%s %s returned an unexpected diagnostic report:\n' \
      "${case_id}" "${command_name}" >&2
    jq '{schema_version, schema_stability, protocol_revision,
      negotiated_protocol_revision, primary_diagnosis, independent_findings,
      outcome, exit_code, summary, checks}' "${report_path}" >&2
    exit 1
  fi
}

compat_run_case() {
  local case_id="$1"
  shift
  local report_path="${compat_reports}/${case_id}.json"
  local command_status
  local remaining_container

  printf 'Inspecting %s.\n' "${case_id}"
  set +e
  "${compat_doctor}" inspect --format json -- "$@" >"${report_path}"
  command_status=$?
  set -e

  if ((command_status != 0)); then
    printf '%s exited %s; redacted report follows:\n' \
      "${case_id}" "${command_status}" >&2
    jq . "${report_path}" >&2
    exit 1
  fi

  compat_assert_report "${case_id}" "${report_path}"
  remaining_container="$(
    docker -H "${compat_docker_host}" ps -q \
      --filter "label=org.enjoyablework.mcp-doctor.compatibility=${case_id}"
  )"
  if [[ -n "${remaining_container}" ]]; then
    printf '%s left a compatibility container running.\n' "${case_id}" >&2
    exit 1
  fi
  printf 'PASS %s\n' "${case_id}"
}

compat_run_active_case() {
  local case_id="$1"
  local container_label="$2"
  shift 2
  local tool
  local effects
  local scenario
  local scenario_path
  local expected_scenario_hash
  local actual_scenario_hash
  local break_cases
  local break_seed
  local command_name
  local expected_cases
  local report_path
  local command_status
  local remaining_container
  local -a doctor_arguments

  tool="$(compat_active_case_value "${case_id}" tool)"
  effects="$(compat_active_case_value "${case_id}" effects)"
  scenario="$(compat_active_case_value "${case_id}" scenario)"
  expected_scenario_hash="$(compat_active_case_value "${case_id}" scenario_sha256)"
  break_cases="$(compat_active_case_value "${case_id}" break_cases)"
  break_seed="$(compat_active_case_value "${case_id}" break_seed)"
  if [[ "${scenario}" != tests/compatibility/scenarios/*.json ]]; then
    printf '%s selected an unexpected scenario path.\n' "${case_id}" >&2
    exit 1
  fi
  scenario_path="${compat_repository_root}/${scenario}"
  if [[ ! -f "${scenario_path}" || -L "${scenario_path}" ]]; then
    printf '%s scenario is not a regular repository file.\n' "${case_id}" >&2
    exit 1
  fi
  actual_scenario_hash="$(compat_sha256 "${scenario_path}")"
  if [[ "${actual_scenario_hash}" != "${expected_scenario_hash}" ]]; then
    printf '%s scenario digest was %s; expected %s.\n' \
      "${case_id}" "${actual_scenario_hash}" "${expected_scenario_hash}" >&2
    exit 1
  fi

  for command_name in check break; do
    report_path="${compat_reports}/active-${case_id}-${command_name}.json"
    if [[ "${command_name}" == "check" ]]; then
      expected_cases=1
      doctor_arguments=(
        check
        --protocol-version 2025-11-25
        --scenario "${scenario_path}"
        --allow-tool "${tool}"
        --format json
      )
    else
      expected_cases="${break_cases}"
      doctor_arguments=(
        break
        --protocol-version 2025-11-25
        --tool "${tool}"
        --allow-tool "${tool}"
        --effects "${effects}"
        --cases "${break_cases}"
        --seed "${break_seed}"
        --format json
      )
    fi

    printf 'Running legacy %s against %s.\n' "${command_name}" "${case_id}"
    set +e
    "${compat_doctor}" "${doctor_arguments[@]}" -- "$@" >"${report_path}"
    command_status=$?
    set -e
    if ((command_status != 0)); then
      printf '%s %s exited %s; redacted report follows:\n' \
        "${case_id}" "${command_name}" "${command_status}" >&2
      jq . "${report_path}" >&2
      exit 1
    fi
    compat_assert_active_report \
      "${case_id}" "${command_name}" "${report_path}" "${expected_cases}"
    if grep -F -- "${tool}" "${report_path}" >/dev/null; then
      printf '%s %s report disclosed its selected tool.\n' \
        "${case_id}" "${command_name}" >&2
      exit 1
    fi
    remaining_container="$(
      docker -H "${compat_docker_host}" ps -q \
        --filter "label=org.enjoyablework.mcp-doctor.compatibility=${container_label}"
    )"
    if [[ -n "${remaining_container}" ]]; then
      printf '%s %s left a compatibility container running.\n' \
        "${case_id}" "${command_name}" >&2
      exit 1
    fi
    printf 'PASS legacy %s %s\n' "${command_name}" "${case_id}"
  done
}

compat_run_logged() {
  local phase="$1"
  shift

  : >"${compat_prepare_log}"
  if ! "$@" >>"${compat_prepare_log}" 2>&1; then
    printf '%s failed; preparation log follows:\n' "${phase}" >&2
    tail -n 100 "${compat_prepare_log}" >&2
    exit 1
  fi
}

cd -- "${compat_repository_root}"

jq -e '
  .schema_version == "mcp-doctor.compatibility/v1" and
  .protocol_revision == "2026-07-28" and
  .tool_execution == "forbidden" and
  .transport == "stdio" and
  .active_legacy.protocol_revision == "2025-11-25" and
  .active_legacy.transport == "stdio" and
  .active_legacy.commands == ["check", "break"] and
  (.active_legacy.cases | length) == 2 and
  all(.active_legacy.cases[];
    .expected_outcome == "passed" and
    .effects == "read_only" and
    (.break_cases | type) == "number" and
    .break_cases >= 1 and .break_cases <= 100 and
    (.break_seed | type) == "number")
' "${compat_matrix}" >/dev/null

docker info >/dev/null
compat_docker_host="$(
  docker context inspect --format '{{.Endpoints.docker.Host}}'
)"
compat_host_uid="$(id -u)"
compat_host_gid="$(id -g)"

compat_target_directory="$(
  cargo metadata --locked --no-deps --format-version 1 | jq -er .target_directory
)"
mkdir -p -- "${compat_target_directory}"
compat_work_root="$(
  mktemp -d "${compat_target_directory}/mcp-doctor-compat.XXXXXX"
)"
compat_work_prefix="${compat_target_directory}/mcp-doctor-compat."
compat_reports="${compat_work_root}/reports"
compat_prepare_log="${compat_work_root}/preparation.log"

compat_cleanup() {
  if [[ "${compat_work_root}" != "${compat_work_prefix}"* ]]; then
    printf 'Refusing to remove unexpected compatibility path: %s\n' \
      "${compat_work_root}" >&2
    return 1
  fi

  if [[ -d "${compat_work_root}" ]]; then
    chmod -R u+w -- "${compat_work_root}" 2>/dev/null || true
    rm -rf -- "${compat_work_root}"
  fi
}

trap compat_cleanup EXIT

mkdir -p -- \
  "${compat_reports}" \
  "${compat_work_root}/artifacts" \
  "${compat_work_root}/composer-cache" \
  "${compat_work_root}/composer-home" \
  "${compat_work_root}/corepack" \
  "${compat_work_root}/dart-cache" \
  "${compat_work_root}/dart-home" \
  "${compat_work_root}/go-build-cache" \
  "${compat_work_root}/go-home" \
  "${compat_work_root}/go-mod-cache" \
  "${compat_work_root}/node-home"

compat_rust_target="${compat_work_root}/rust-target"
compat_doctor="${compat_rust_target}/debug/mcp-doctor"
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="${compat_rust_target}" \
  cargo build --locked --bin mcp-doctor

compat_node_image="$(compat_runtime_value node image)"
compat_go_image="$(compat_runtime_value go image)"
compat_dart_image="$(compat_runtime_value dart image)"
compat_php_image="$(compat_runtime_value php image)"

for compat_image in \
  "${compat_node_image}" \
  "${compat_go_image}" \
  "${compat_dart_image}" \
  "${compat_php_image}"; do
  docker -H "${compat_docker_host}" pull "${compat_image}" >/dev/null
done

compat_typescript_case="official-typescript-todos"
compat_go_case="official-go-hello"
compat_dart_case="independent-dart-strict-current"
compat_php_case="independent-php-simple"
compat_typescript_root="${compat_work_root}/typescript-sdk"
compat_go_root="${compat_work_root}/go-sdk"
compat_dart_root="${compat_work_root}/mcp-dart"
compat_php_root="${compat_work_root}/mcp-sdk-php"

compat_clone_case "${compat_typescript_case}" "${compat_typescript_root}"
compat_clone_case "${compat_go_case}" "${compat_go_root}"
compat_clone_case "${compat_dart_case}" "${compat_dart_root}"
compat_clone_case "${compat_php_case}" "${compat_php_root}"

compat_verify_lock "${compat_typescript_case}" \
  "${compat_typescript_root}/pnpm-lock.yaml"
compat_verify_lock "${compat_go_case}" "${compat_go_root}/go.sum"

cp -- \
  "${compat_repository_root}/tests/compatibility/locks/mcp_dart-v2.4.0.pubspec.lock" \
  "${compat_dart_root}/pubspec.lock"
cp -- \
  "${compat_repository_root}/tests/compatibility/locks/mcp-sdk-php-v2.0.0.composer.lock" \
  "${compat_php_root}/composer.lock"
compat_verify_lock "${compat_dart_case}" "${compat_dart_root}/pubspec.lock"
compat_verify_lock "${compat_php_case}" "${compat_php_root}/composer.lock"

printf 'Preparing pinned TypeScript dependencies in an isolated container.\n'
compat_run_logged 'TypeScript dependency preparation' \
  docker -H "${compat_docker_host}" run --rm \
  --user "${compat_host_uid}:${compat_host_gid}" \
  -e CI=true \
  -e COREPACK_HOME=/work/corepack \
  -e HOME=/work/node-home \
  -v "${compat_work_root}:/work" \
  -w /work/typescript-sdk \
  "${compat_node_image}" \
  sh -c '
    corepack pnpm --version | grep -Fx 10.26.1 >/dev/null
    corepack pnpm install --frozen-lockfile \
      --filter "@mcp-examples/todos-server..."
    corepack pnpm -r --filter "@mcp-examples/todos-server..." \
      --if-present build
  '
compat_verify_lock "${compat_typescript_case}" \
  "${compat_typescript_root}/pnpm-lock.yaml"

printf 'Preparing pinned Go dependencies in an isolated container.\n'
compat_run_logged 'Go dependency download' \
  docker -H "${compat_docker_host}" run --rm \
  --user "${compat_host_uid}:${compat_host_gid}" \
  -e GOCACHE=/work/go-build-cache \
  -e GOMODCACHE=/work/go-mod-cache \
  -e HOME=/work/go-home \
  -v "${compat_work_root}:/work" \
  -w /work/go-sdk \
  "${compat_go_image}" \
  go mod download
compat_run_logged 'Go offline verification and build' \
  docker -H "${compat_docker_host}" run --rm --network none \
  --user "${compat_host_uid}:${compat_host_gid}" \
  -e CGO_ENABLED=0 \
  -e GOCACHE=/work/go-build-cache \
  -e GOMODCACHE=/work/go-mod-cache \
  -e HOME=/work/go-home \
  -v "${compat_work_root}:/work" \
  -w /work/go-sdk \
  "${compat_go_image}" \
  sh -c '
    go mod verify
    go build -buildvcs=false -mod=readonly -trimpath \
      -o /work/artifacts/official-go-hello ./examples/server/hello
  '
compat_verify_lock "${compat_go_case}" "${compat_go_root}/go.sum"

printf 'Preparing pinned Dart dependencies in an isolated container.\n'
compat_run_logged 'Dart dependency preparation' \
  docker -H "${compat_docker_host}" run --rm \
  --user "${compat_host_uid}:${compat_host_gid}" \
  -e HOME=/work/dart-home \
  -e PUB_CACHE=/work/dart-cache \
  -v "${compat_work_root}:/work" \
  -w /work/mcp-dart \
  "${compat_dart_image}" \
  dart pub get --enforce-lockfile
compat_run_logged 'Dart offline build' \
  docker -H "${compat_docker_host}" run --rm --network none \
  --user "${compat_host_uid}:${compat_host_gid}" \
  -e HOME=/work/dart-home \
  -e PUB_CACHE=/work/dart-cache \
  -v "${compat_work_root}:/work" \
  -w /work/mcp-dart \
  "${compat_dart_image}" \
  dart compile exe example/mcp_2026_07_28/server.dart \
  -o /work/artifacts/independent-dart-strict-current
compat_verify_lock "${compat_dart_case}" "${compat_dart_root}/pubspec.lock"

printf 'Preparing pinned PHP dependencies in an isolated container.\n'
compat_run_logged 'PHP dependency preparation' \
  docker -H "${compat_docker_host}" run --rm \
  --user "${compat_host_uid}:${compat_host_gid}" \
  -e COMPOSER_CACHE_DIR=/work/composer-cache \
  -e HOME=/work/composer-home \
  -v "${compat_work_root}:/work" \
  -w /work/mcp-sdk-php \
  "${compat_php_image}" \
  install --no-dev --prefer-dist --no-interaction --no-progress \
  --no-plugins --no-scripts
compat_verify_lock "${compat_php_case}" "${compat_php_root}/composer.lock"

compat_run_case "${compat_typescript_case}" \
  docker -H "${compat_docker_host}" run --rm -i --pull never \
  --network none --read-only --security-opt no-new-privileges \
  --cap-drop ALL --init \
  --label "org.enjoyablework.mcp-doctor.compatibility=${compat_typescript_case}" \
  --tmpfs /tmp:rw,noexec,nosuid,size=64m \
  --user "${compat_host_uid}:${compat_host_gid}" \
  -e HOME=/tmp \
  -v "${compat_typescript_root}:/src:ro" \
  -w /src \
  "${compat_node_image}" \
  node node_modules/tsx/dist/cli.mjs examples/todos-server/server.ts

compat_run_case "${compat_go_case}" \
  docker -H "${compat_docker_host}" run --rm -i --pull never \
  --network none --read-only --security-opt no-new-privileges \
  --cap-drop ALL --init \
  --label "org.enjoyablework.mcp-doctor.compatibility=${compat_go_case}" \
  --tmpfs /tmp:rw,noexec,nosuid,size=64m \
  --user "${compat_host_uid}:${compat_host_gid}" \
  -v "${compat_work_root}/artifacts:/server:ro" \
  "${compat_go_image}" \
  /server/official-go-hello

compat_run_case "${compat_dart_case}" \
  docker -H "${compat_docker_host}" run --rm -i --pull never \
  --network none --read-only --security-opt no-new-privileges \
  --cap-drop ALL --init \
  --label "org.enjoyablework.mcp-doctor.compatibility=${compat_dart_case}" \
  --tmpfs /tmp:rw,noexec,nosuid,size=64m \
  --user "${compat_host_uid}:${compat_host_gid}" \
  -v "${compat_work_root}/artifacts:/server:ro" \
  "${compat_dart_image}" \
  /server/independent-dart-strict-current

compat_run_case "${compat_php_case}" \
  docker -H "${compat_docker_host}" run --rm -i --pull never \
  --network none --read-only --security-opt no-new-privileges \
  --cap-drop ALL --init \
  --label "org.enjoyablework.mcp-doctor.compatibility=${compat_php_case}" \
  --tmpfs /tmp:rw,noexec,nosuid,size=64m \
  --user "${compat_host_uid}:${compat_host_gid}" \
  -e HOME=/tmp \
  --entrypoint php \
  -v "${compat_php_root}:/app:ro" \
  -w /app \
  "${compat_php_image}" \
  examples/simple_server_stdio.php

compat_active_go_label="active-${compat_go_case}"
compat_run_active_case "${compat_go_case}" "${compat_active_go_label}" \
  docker -H "${compat_docker_host}" run --rm -i --pull never \
  --network none --read-only --security-opt no-new-privileges \
  --cap-drop ALL --init \
  --label "org.enjoyablework.mcp-doctor.compatibility=${compat_active_go_label}" \
  --tmpfs /tmp:rw,noexec,nosuid,size=64m \
  --user "${compat_host_uid}:${compat_host_gid}" \
  -v "${compat_work_root}/artifacts:/server:ro" \
  "${compat_go_image}" \
  /server/official-go-hello

compat_active_php_label="active-${compat_php_case}"
compat_run_active_case "${compat_php_case}" "${compat_active_php_label}" \
  docker -H "${compat_docker_host}" run --rm -i --pull never \
  --network none --read-only --security-opt no-new-privileges \
  --cap-drop ALL --init \
  --label "org.enjoyablework.mcp-doctor.compatibility=${compat_active_php_label}" \
  --tmpfs /tmp:rw,noexec,nosuid,size=64m \
  --user "${compat_host_uid}:${compat_host_gid}" \
  -e HOME=/tmp \
  --entrypoint php \
  -v "${compat_php_root}:/app:ro" \
  -w /app \
  "${compat_php_image}" \
  examples/simple_server_stdio.php

printf 'All four pinned current-revision cases and four active legacy journeys passed.\n'
