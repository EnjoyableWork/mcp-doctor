#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <mcp-doctor executable> <expected version> <fixture executable>" >&2
  exit 2
fi

smoke_executable=$1
smoke_version=$2
smoke_fixture=$3

for smoke_required_executable in "${smoke_executable}" "${smoke_fixture}"; do
  if [[ ! -x "${smoke_required_executable}" || -L "${smoke_required_executable}" ]]; then
    echo "installed smoke executable or fixture is missing, symbolic, or not executable" >&2
    exit 1
  fi
done
if ! command -v jq >/dev/null 2>&1; then
  echo "installed smoke requires jq" >&2
  exit 1
fi

smoke_executable="$(cd -- "$(dirname -- "${smoke_executable}")" && pwd)/$(basename -- "${smoke_executable}")"
smoke_fixture="$(cd -- "$(dirname -- "${smoke_fixture}")" && pwd)/$(basename -- "${smoke_fixture}")"
smoke_temp_parent=${TMPDIR:-/tmp}
smoke_root_prefix="${smoke_temp_parent%/}/mcp-doctor-release-smoke."
smoke_root=$(mktemp -d "${smoke_root_prefix}XXXXXX")
smoke_home="${smoke_root}/home"
smoke_capabilities="${smoke_root}/capabilities.json"
smoke_report="${smoke_root}/report.json"
smoke_json_artifact="${smoke_root}/report-artifact.json"
smoke_junit_artifact="${smoke_root}/report-artifact.xml"
smoke_legacy_scenario="${smoke_root}/legacy-scenario.json"
smoke_workflow_scenario="${smoke_root}/workflow-scenario.json"
smoke_workflow_report="${smoke_root}/workflow-report.json"
smoke_legacy_check="${smoke_root}/legacy-check.json"
smoke_legacy_break="${smoke_root}/legacy-break.json"
smoke_reject="${smoke_root}/reject.json"
smoke_reject_marker="${smoke_root}/reject-count.txt"
smoke_snapshot="${smoke_root}/contract.json"
smoke_legacy_11_snapshot="${smoke_root}/contract-2025-11-25.json"
smoke_legacy_06_snapshot="${smoke_root}/contract-2025-06-18.json"
smoke_legacy_report="${smoke_root}/legacy-report.json"
smoke_diff="${smoke_root}/diff.json"
smoke_aggregate="${smoke_root}/aggregate.json"
smoke_aggregate_stdout="${smoke_root}/aggregate-stdout.json"
smoke_stderr="${smoke_root}/stderr.txt"
smoke_path=${PATH:?PATH must locate the fixture platform loader}

cleanup_smoke_root() {
  if [[ "${smoke_root}" != "${smoke_root_prefix}"* ]]; then
    echo "refusing to remove an unexpected installed-smoke path" >&2
    return 1
  fi
  if [[ -d "${smoke_root}" ]]; then
    rm -rf -- "${smoke_root}"
  fi
}
trap cleanup_smoke_root EXIT

mkdir -p -- "${smoke_home}"

run_mcp_doctor() {
  env -i \
    HOME="${smoke_home}" \
    LANG=C \
    LC_ALL=C \
    NO_COLOR=1 \
    PATH="${smoke_path}" \
    TEMP="${smoke_root}" \
    TMP="${smoke_root}" \
    TMPDIR="${smoke_root}" \
    TZ=UTC \
    USERPROFILE="${smoke_home}" \
    "${smoke_executable}" "$@"
}

version_output=$(run_mcp_doctor --version)
if [[ "${version_output}" != "mcp-doctor ${smoke_version}" ]]; then
  echo "installed executable reported an unexpected version" >&2
  exit 1
fi

smoke_agent_guide="https://github.com/EnjoyableWork/mcp-doctor/blob/v${smoke_version}/docs/agents.md"
help_output=$(run_mcp_doctor --help)
if ! grep -F -- "Coding agents: ${smoke_agent_guide}" <<<"${help_output}" >/dev/null; then
  echo "installed executable omitted its exact-version coding-agent guide" >&2
  exit 1
fi

if ! run_mcp_doctor capabilities --format json \
  >"${smoke_capabilities}" 2>"${smoke_stderr}"; then
  echo "installed compiled-capability smoke failed" >&2
  exit 1
fi
if [[ -s "${smoke_stderr}" ]]; then
  echo "installed compiled-capability smoke wrote unexpected stderr" >&2
  exit 1
fi
jq -e --arg version "${smoke_version}" '
  .schema_version == "mcp-doctor.capabilities/v1" and
  .schema_stability == "stable" and
  .product == {name: "mcp-doctor", version: $version} and
  ([.commands[].name] == [
    "aggregate", "break", "capabilities", "check", "diff", "inspect", "reject"
  ]) and
  ([.protocol_support[] |
    select(.command == "inspect" and .transport == "stdio") |
    .revisions] == [["2026-07-28", "2025-11-25", "2025-06-18"]]) and
  ([.protocol_support[] |
    select(.command == "check" and .transport == "streamable_http") |
    .revisions] == [["2026-07-28", "2025-11-25", "2025-06-18"]]) and
  ([.protocol_support[] |
    select(.command == "reject" and .transport == "stdio") |
    .revisions] == [["2026-07-28"]]) and
  ([.protocol_support[] |
    select(.command == "reject" and .transport == "streamable_http") |
    .revisions] == [["2026-07-28"]]) and
  .schema_versions.diagnostic_report == ["mcp-doctor.report/v1"] and
  .schema_versions.scenario == [
    "mcp-doctor.scenario/v1alpha1",
    "mcp-doctor.scenario/v2alpha1"
  ] and
  .schema_versions.generator == ["mcp-doctor.generator/v1"] and
  .schema_versions.contract_snapshot == ["mcp-doctor.contract-snapshot/v1alpha1"] and
  .schema_versions.contract_diff == ["mcp-doctor.contract-diff/v1alpha1"] and
  .exit_semantics.version == "mcp-doctor.exit/v1" and
  ([.exit_semantics.codes[].code] == [0, 1, 2, 3, 4]) and
  .platform == {
    family: "unix",
    process_tree_control: "process_group",
    file_identity: "device_inode"
  } and
  ([.limit_profiles[] | select(.hard == true)] | length) == 4 and
  ([.limit_profiles[] |
    select(.id == "mcp-doctor.limits/diagnostic/v1") |
    .selections] == [["default", "slow-start"]]) and
  ([.limit_profiles[] |
    select(.id == "mcp-doctor.limits/diagnostic/v1") |
    .selectable_for] == [["break", "check", "inspect"]]) and
  .limits.output_bytes == 65536
' "${smoke_capabilities}" >/dev/null

if ! run_mcp_doctor inspect --limit-profile slow-start --format json \
  --json-report "${smoke_json_artifact}" \
  --junit-report "${smoke_junit_artifact}" \
  --snapshot "${smoke_snapshot}" \
  --allow-sensitive-snapshot "${smoke_snapshot}" \
  -- "${smoke_fixture}" catalog-valid \
  >"${smoke_report}" 2>"${smoke_stderr}"; then
  echo "installed passive diagnostic smoke failed" >&2
  exit 1
fi
if [[ -s "${smoke_stderr}" ]]; then
  echo "installed passive diagnostic smoke wrote unexpected stderr" >&2
  exit 1
fi

jq -e '
  .schema_version == "mcp-doctor.report/v1" and
  .schema_stability == "stable" and
  .protocol_revision == "2026-07-28" and
  .primary_diagnosis == null and
  .independent_findings == [] and
  .outcome == "passed" and
  .exit_code == 0 and
  .limits.profile == "slow-start" and
  .limits.startup_ms == 30000 and
  .limits.discovery_ms == 30000 and
  .limits.request_ms == 60000 and
  .limits.response_ms == 60000 and
  .limits.shutdown_grace_ms == 2000 and
  .limits.total_ms == 240000 and
  .limits.redirects == 0 and
  .limits.retries == 0 and
  .limits.concurrency == 1 and
  .summary.required == 5 and
  .summary.required_skipped == 0 and
  .summary.failed == 0 and
  ([.checks[] | select(.requirement == "required") |
    (.state == "performed" and .outcome == "passed")] | all) and
  ([.checks[] | select(.id == "runtime.tools")] | length) == 1 and
  ([.checks[] | select(.id == "runtime.tools") |
    (.state == "skipped" and .skip_reason == "not_authorized" and
     (has("blocked_by") | not))] | all)
' "${smoke_report}" >/dev/null

if ! cmp -s -- "${smoke_report}" "${smoke_json_artifact}"; then
  echo "installed diagnostic JSON artifact diverged from the stdout projection" >&2
  exit 1
fi
if ! grep -F -- "limits.profile=slow-start" "${smoke_junit_artifact}" >/dev/null; then
  echo "installed diagnostic JUnit artifact omitted the selected limit profile" >&2
  exit 1
fi

if ! run_mcp_doctor reject \
  --tool synthetic.reviewed \
  --allow-tool synthetic.reviewed \
  --effects read_only \
  --seed 7529 \
  --format json \
  -- "${smoke_fixture}" reject-success "${smoke_reject_marker}" \
  >"${smoke_reject}" 2>"${smoke_stderr}"; then
  echo "installed current-revision reject smoke failed" >&2
  exit 1
fi
if [[ -s "${smoke_stderr}" ]]; then
  echo "installed current-revision reject smoke wrote unexpected stderr" >&2
  exit 1
fi
if [[ "$(cat -- "${smoke_reject_marker}")" != "7" ]]; then
  echo "installed current-revision reject smoke sent an unexpected call count" >&2
  exit 1
fi
jq -e '
  .schema_version == "mcp-doctor.report/v1" and
  .protocol_revision == "2026-07-28" and
  .primary_diagnosis == null and
  .independent_findings == [] and
  .outcome == "passed" and
  .exit_code == 0 and
  .summary.failed == 0 and
  ([.checks[] | select(.requirement == "required") |
    (.state == "performed" and .outcome == "passed")] | all) and
  ([.checks[] | select(.id == "generation.cases") |
    (.state == "performed" and .outcome == "passed")] | all) and
  ([.checks[] | select(.id | startswith("runtime.tools.case["))] | length) == 7 and
  ([.checks[] | select(.id | startswith("runtime.tools.case[")) |
    select(.state == "performed" and .outcome == "passed")] | length) == 7 and
  ([.checks[] | select(.id | startswith("runtime.tools.case[")) |
    select(.state == "skipped" and .skip_reason == "not_applicable")] | length) == 0
' "${smoke_reject}" >/dev/null
for smoke_private_reject_value in \
  synthetic.reviewed \
  synthetic-secret-payload-7f2c \
  synthetic_private_mode_never_report_7f2c \
  mcp-doctor-invalid-enum \
  sequence \
  secret; do
  if grep -F -- "${smoke_private_reject_value}" "${smoke_reject}" >/dev/null; then
    echo "installed current-revision reject report disclosed a private value" >&2
    exit 1
  fi
done

jq -n '
  {
    schema_version: "mcp-doctor.scenario/v2alpha1",
    steps: [{
      id: "installed-private-lookup",
      tool: "synthetic.workflow.lookup",
      safety: {effects: "read_only"},
      arguments: {query: "synthetic-secret-payload-7f2c"},
      captures: {resource_id: "/resource/id"},
      expect: {result: "success"}
    }, {
      id: "installed-private-read",
      tool: "synthetic.workflow.read",
      safety: {effects: "read_only"},
      arguments: {id: null},
      argument_refs: {"/id": "resource_id"},
      expect: {result: "success"}
    }]
  }
' >"${smoke_workflow_scenario}"

if ! run_mcp_doctor check \
  --scenario "${smoke_workflow_scenario}" \
  --allow-tool synthetic.workflow.lookup \
  --allow-tool synthetic.workflow.read \
  --format json \
  -- "${smoke_fixture}" workflow-read-only \
  >"${smoke_workflow_report}" 2>"${smoke_stderr}"; then
  echo "installed current-revision workflow smoke failed" >&2
  exit 1
fi
if [[ -s "${smoke_stderr}" ]]; then
  echo "installed current-revision workflow smoke wrote unexpected stderr" >&2
  exit 1
fi
jq -e '
  .schema_version == "mcp-doctor.report/v1" and
  .protocol_revision == "2026-07-28" and
  .primary_diagnosis == null and
  .independent_findings == [] and
  .outcome == "passed" and
  .exit_code == 0 and
  ([.checks[] | select(.id | startswith("runtime.workflow.step["))] | length) == 2 and
  ([.checks[] | select(.id | startswith("runtime.workflow.step[")) |
    select(.state == "performed" and .outcome == "passed")] | length) == 2
' "${smoke_workflow_report}" >/dev/null
for smoke_private_workflow_value in \
  synthetic.workflow.lookup \
  synthetic.workflow.read \
  installed-private-lookup \
  installed-private-read \
  synthetic-secret-payload-7f2c \
  resource_id; do
  if grep -F -- "${smoke_private_workflow_value}" "${smoke_workflow_report}" >/dev/null; then
    echo "installed workflow report disclosed a private value" >&2
    exit 1
  fi
done

jq -n '
  {
    schema_version: "mcp-doctor.scenario/v1alpha1",
    tool: "synthetic.reviewed",
    safety: {effects: "read_only"},
    cases: [{
      id: "installed-legacy-case",
      arguments: {sequence: 0},
      expect: {result: "success"}
    }]
  }
' >"${smoke_legacy_scenario}"

for smoke_active_revision in 2025-11-25 2025-06-18; do
  if ! run_mcp_doctor check \
    --protocol-version "${smoke_active_revision}" \
    --scenario "${smoke_legacy_scenario}" \
    --allow-tool synthetic.reviewed \
    --format json \
    -- "${smoke_fixture}" legacy-active-success \
    >"${smoke_legacy_check}" 2>"${smoke_stderr}"; then
    echo "installed ${smoke_active_revision} check smoke failed" >&2
    exit 1
  fi
  if [[ -s "${smoke_stderr}" ]]; then
    echo "installed ${smoke_active_revision} check smoke wrote unexpected stderr" >&2
    exit 1
  fi

  if ! run_mcp_doctor break \
    --protocol-version "${smoke_active_revision}" \
    --tool synthetic.generated \
    --allow-tool synthetic.generated \
    --effects read_only \
    --cases 2 \
    --seed 6027 \
    --format json \
    -- "${smoke_fixture}" legacy-break-success 2 \
    >"${smoke_legacy_break}" 2>"${smoke_stderr}"; then
    echo "installed ${smoke_active_revision} break smoke failed" >&2
    exit 1
  fi
  if [[ -s "${smoke_stderr}" ]]; then
    echo "installed ${smoke_active_revision} break smoke wrote unexpected stderr" >&2
    exit 1
  fi

  jq -e --arg revision "${smoke_active_revision}" '
    .schema_version == "mcp-doctor.report/v1" and
    .protocol_revision == $revision and
    .negotiated_protocol_revision == $revision and
    .primary_diagnosis == null and
    .independent_findings == [] and
    .outcome == "passed" and
    .exit_code == 0 and
    .summary.required == 8 and
    .summary.required_skipped == 0 and
    .summary.failed == 0 and
    ([.checks[] | select(.requirement == "required") |
      (.state == "performed" and .outcome == "passed")] | all) and
    ([.checks[] | select(.id | startswith("runtime.tools.case["))] | length) == 1
  ' "${smoke_legacy_check}" >/dev/null

  jq -e --arg revision "${smoke_active_revision}" '
    .schema_version == "mcp-doctor.report/v1" and
    .protocol_revision == $revision and
    .negotiated_protocol_revision == $revision and
    .primary_diagnosis == null and
    .independent_findings == [] and
    .outcome == "passed" and
    .exit_code == 0 and
    .summary.required == 10 and
    .summary.required_skipped == 0 and
    .summary.failed == 0 and
    ([.checks[] | select(.requirement == "required") |
      (.state == "performed" and .outcome == "passed")] | all) and
    ([.checks[] | select(.id == "generation.cases") |
      (.state == "performed" and .outcome == "passed")] | all) and
    ([.checks[] | select(.id | startswith("runtime.tools.case["))] | length) == 2
  ' "${smoke_legacy_break}" >/dev/null
done

for smoke_private_active_value in \
  synthetic.reviewed \
  synthetic.generated \
  installed-legacy-case \
  synthetic-secret-payload-7f2c \
  synthetic_private_query_never_report_7f2c \
  synthetic_private_limit_never_report_7f2c \
  synthetic_private_flags_never_report_7f2c; do
  if grep -F -- "${smoke_private_active_value}" \
    "${smoke_legacy_check}" "${smoke_legacy_break}" >/dev/null; then
    echo "installed legacy active report disclosed a private value" >&2
    exit 1
  fi
done

if ! run_mcp_doctor aggregate --format json \
  --output "${smoke_aggregate}" "${smoke_json_artifact}" \
  >"${smoke_aggregate_stdout}" 2>"${smoke_stderr}"; then
  echo "installed offline diagnostic aggregate smoke failed" >&2
  exit 1
fi
if [[ -s "${smoke_stderr}" ]]; then
  echo "installed offline diagnostic aggregate smoke wrote unexpected stderr" >&2
  exit 1
fi
if ! cmp -s -- "${smoke_aggregate}" "${smoke_aggregate_stdout}"; then
  echo "installed diagnostic aggregate artifact diverged from JSON stdout" >&2
  exit 1
fi
jq -e '
  .schema_version == "mcp-doctor.aggregate/v1" and
  .schema_stability == "stable" and
  .outcome == "passed" and
  .exit_code == 0 and
  .summary == {members: 1, passed: 1, failed: 0, incomplete: 0} and
  (.members | length) == 1 and
  .members[0].ordinal == 0 and
  .members[0].report.schema_version == "mcp-doctor.report/v1" and
  .members[0].report.outcome == "passed"
' "${smoke_aggregate}" >/dev/null
if [[ ! -f "${smoke_junit_artifact}" || -L "${smoke_junit_artifact}" ]]; then
  echo "installed diagnostic did not create a regular JUnit artifact" >&2
  exit 1
fi
for smoke_junit_evidence in \
  '<testsuites name="mcp-doctor"' \
  'name="runtime.tools"' \
  'report_outcome=passed' \
  'exit_code=0'; do
  if ! grep -F -- "${smoke_junit_evidence}" "${smoke_junit_artifact}" >/dev/null; then
    echo "installed diagnostic JUnit artifact omitted required evidence" >&2
    exit 1
  fi
done

jq -e '
  .schema_version == "mcp-doctor.contract-snapshot/v1alpha1" and
  .protocol_revision == "2026-07-28" and
  .capabilities.tools.advertised == true and
  (.catalogs.tools.contracts | length) == 2 and
  (.catalogs.tools.correlation | length) == 2 and
  (.catalogs.prompts.contracts | length) == 1 and
  (.catalogs.resources.contracts | length) == 1 and
  (.catalogs.resource_templates.contracts | length) == 1
' "${smoke_snapshot}" >/dev/null

if ! run_mcp_doctor diff --format json "${smoke_snapshot}" "${smoke_snapshot}" \
  >"${smoke_diff}" 2>"${smoke_stderr}"; then
  echo "installed offline contract diff smoke failed" >&2
  exit 1
fi
if [[ -s "${smoke_stderr}" ]]; then
  echo "installed offline contract diff smoke wrote unexpected stderr" >&2
  exit 1
fi
jq -e '
  .schema_version == "mcp-doctor.contract-diff/v1alpha1" and
  .protocol_revision == "2026-07-28" and
  .outcome == "unchanged" and
  .exit_code == 0 and
  .summary.total == 0 and
  (.findings | length) == 0 and
  ([.checks[] | .state == "performed"] | all)
' "${smoke_diff}" >/dev/null

for smoke_legacy_case in \
  "2025-11-25|draft_2020_12|${smoke_legacy_11_snapshot}" \
  "2025-06-18|ambiguous|${smoke_legacy_06_snapshot}"; do
  IFS='|' read -r smoke_legacy_revision smoke_legacy_dialect smoke_legacy_snapshot \
    <<<"${smoke_legacy_case}"
  if ! run_mcp_doctor inspect --format json \
    --protocol-version "${smoke_legacy_revision}" \
    --snapshot "${smoke_legacy_snapshot}" \
    --allow-sensitive-snapshot "${smoke_legacy_snapshot}" \
    -- "${smoke_fixture}" legacy-success \
    >"${smoke_legacy_report}" 2>"${smoke_stderr}"; then
    echo "installed legacy snapshot smoke failed" >&2
    exit 1
  fi
  if [[ -s "${smoke_stderr}" ]]; then
    echo "installed legacy snapshot smoke wrote unexpected stderr" >&2
    exit 1
  fi
  jq -e --arg revision "${smoke_legacy_revision}" '
    .schema_version == "mcp-doctor.report/v1" and
    .protocol_revision == $revision and
    .negotiated_protocol_revision == $revision and
    .outcome == "passed" and
    .exit_code == 0
  ' "${smoke_legacy_report}" >/dev/null
  jq -e \
    --arg revision "${smoke_legacy_revision}" \
    --arg dialect "${smoke_legacy_dialect}" '
    .schema_version == "mcp-doctor.contract-snapshot/v1alpha1" and
    .protocol_revision == $revision and
    .negotiated_protocol_revision == $revision and
    .capabilities.tools.advertised == true and
    (.catalogs.tools.contracts | length) == 1 and
    .catalogs.tools.contracts[0].input_schema_dialect == $dialect and
    .catalogs.tools.contracts[0].output_schema_dialect == $dialect and
    (if $revision == "2025-11-25" then
      .capabilities.logging.advertised == true and
      .capabilities.tasks == {
        advertised: true,
        list: true,
        cancel: true,
        requests_tools_call: true
      }
    else
      ((.capabilities | has("logging")) | not) and
      ((.capabilities | has("tasks")) | not)
    end)
  ' "${smoke_legacy_snapshot}" >/dev/null

  if ! run_mcp_doctor diff --format json \
    "${smoke_legacy_snapshot}" "${smoke_legacy_snapshot}" \
    >"${smoke_diff}" 2>"${smoke_stderr}"; then
    echo "installed legacy offline contract diff smoke failed" >&2
    exit 1
  fi
  if [[ -s "${smoke_stderr}" ]]; then
    echo "installed legacy offline contract diff wrote unexpected stderr" >&2
    exit 1
  fi
  jq -e --arg revision "${smoke_legacy_revision}" '
    .schema_version == "mcp-doctor.contract-diff/v1alpha1" and
    .protocol_revision == $revision and
    .outcome == "unchanged" and
    .exit_code == 0 and
    .summary.total == 0 and
    (.findings | length) == 0 and
    ([.checks[] | .state == "performed"] | all)
  ' "${smoke_diff}" >/dev/null
done
