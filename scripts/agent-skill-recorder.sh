#!/usr/bin/env bash

set -euo pipefail

if [[ -z "${MCP_DOCTOR_AGENT_RECORDER_LOG:-}" ]]; then
  echo "MCP_DOCTOR_AGENT_RECORDER_LOG must name one disposable log" >&2
  exit 70
fi
if [[ -L "${MCP_DOCTOR_AGENT_RECORDER_LOG}" ]]; then
  echo "Agent Skill recorder log must not be symbolic" >&2
  exit 70
fi

agent_recorder_report=report.json
agent_recorder_report_sha256=d38f22800942d648f92aa6a694a5dfb0061a0789b04a4f1fbc478cd709c56fd1

agent_recorder_record() {
  printf '%s\n' "$1" >>"${MCP_DOCTOR_AGENT_RECORDER_LOG}"
}

agent_recorder_hash() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print tolower($1) }'
  else
    shasum -a 256 "$1" | awk '{ print tolower($1) }'
  fi
}

agent_recorder_emit_report() {
  if [[ ! -f "${agent_recorder_report}" || -L "${agent_recorder_report}" ]]; then
    agent_recorder_record 'rejected synthetic report fixture'
    exit 70
  fi
  agent_recorder_observed_report_sha256=$(agent_recorder_hash "${agent_recorder_report}")
  if [[ "${agent_recorder_observed_report_sha256}" != "${agent_recorder_report_sha256}" ]]; then
    agent_recorder_record 'rejected synthetic report fixture'
    exit 70
  fi
  cat -- "${agent_recorder_report}"
}

case "${1:-}" in
  --version)
    if [[ $# -ne 1 ]]; then
      agent_recorder_record 'rejected version invocation'
      exit 70
    fi
    agent_recorder_record 'mcp-doctor --version'
    printf 'mcp-doctor 0.4.0\n'
    ;;
  capabilities)
    if [[ $# -ne 3 || "${2:-}" != --format || "${3:-}" != json ]]; then
      agent_recorder_record 'rejected capabilities invocation'
      exit 70
    fi
    agent_recorder_record 'mcp-doctor capabilities --format json'
    printf '%s\n' '{"schema_version":"mcp-doctor.capabilities/v1","schema_stability":"stable","product":{"name":"mcp-doctor","version":"0.4.0"},"commands":[{"name":"inspect","activity":"passive","reporters":["human","json","junit"],"input_schema_versions":[],"output_schema_versions":["mcp-doctor.report/v1"],"generator_versions":[],"limit_profile":"mcp-doctor.limits/diagnostic/v1"}],"protocol_revisions":[{"revision":"2026-07-28","recognition":"supported"}],"protocol_support":[{"command":"inspect","transport":"stdio","revisions":["2026-07-28"]}],"schema_versions":{"aggregate":["mcp-doctor.aggregate/v1"],"capabilities":["mcp-doctor.capabilities/v1"],"contract_diff":["mcp-doctor.contract-diff/v1alpha1"],"contract_snapshot":["mcp-doctor.contract-snapshot/v1alpha1"],"diagnostic_report":["mcp-doctor.report/v1"],"generator":["mcp-doctor.generator/v1"],"scenario":["mcp-doctor.scenario/v1alpha1"]},"reporters":[{"name":"human","machine_readable":false},{"name":"json","machine_readable":true},{"name":"junit","machine_readable":true}],"exit_semantics":{"version":"mcp-doctor.exit/v1","codes":[{"code":0,"meaning":"passed"},{"code":1,"meaning":"failed"},{"code":2,"meaning":"invalid_invocation"},{"code":3,"meaning":"incomplete"},{"code":4,"meaning":"internal_failure"}]},"platform":{"family":"unix","process_tree_control":"process_group","file_identity":"device_inode"},"limit_profiles":[{"id":"mcp-doctor.limits/diagnostic/v1","default_for":["inspect"],"hard":true,"selections":["default"],"selectable_for":["inspect"]}],"limits":{"output_bytes":65536}}'
    ;;
  inspect)
    if [[ $# -eq 6 && "${2:-}" == --format && "${3:-}" == json &&
      "${4:-}" == -- && "${5:-}" == ./synthetic-mcp-server &&
      "${6:-}" == --stdio ]]; then
      agent_recorder_record 'mcp-doctor inspect --format json -- ./synthetic-mcp-server --stdio'
    elif [[ $# -eq 4 && "${2:-}" == --format && "${3:-}" == json &&
      "${4:-}" == https://mcp.invalid/example ]]; then
      agent_recorder_record 'mcp-doctor inspect --format json https://mcp.invalid/example'
    else
      agent_recorder_record 'rejected inspect invocation'
      exit 70
    fi
    agent_recorder_emit_report
    exit 1
    ;;
  check | break | reject)
    agent_recorder_record 'rejected active mcp-doctor command'
    exit 70
    ;;
  *)
    agent_recorder_record 'rejected unknown mcp-doctor invocation'
    exit 70
    ;;
esac
