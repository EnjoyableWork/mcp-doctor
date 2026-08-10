#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <cargo package> <stable version> <output directory>" >&2
  exit 2
fi

release_cargo_package=$1
release_version=$2
release_output_directory=$3

if [[ ! "${release_version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "release version must be a stable semantic version" >&2
  exit 1
fi
if [[ ! -f "${release_cargo_package}" || -L "${release_cargo_package}" ]]; then
  echo "Cargo package must be a regular, non-symbolic-link file" >&2
  exit 1
fi
if [[ -e "${release_output_directory}" ]]; then
  echo "release-channel output directory already exists" >&2
  exit 1
fi

release_package_name="mcp-doctor-${release_version}.crate"
if [[ "$(basename -- "${release_cargo_package}")" != "${release_package_name}" ]]; then
  echo "Cargo package filename does not match the release version" >&2
  exit 1
fi

release_cargo_package="$({ cd -- "$(dirname -- "${release_cargo_package}")" && pwd; })/${release_package_name}"
release_output_parent=$(dirname -- "${release_output_directory}")
release_output_name=$(basename -- "${release_output_directory}")
mkdir -p -- "${release_output_parent}"
release_output_parent=$(cd -- "${release_output_parent}" && pwd)
release_output_directory="${release_output_parent}/${release_output_name}"
release_stage_prefix="${release_output_parent}/.mcp-doctor-release-channels."
release_stage=$(mktemp -d "${release_stage_prefix}XXXXXX")

cleanup_release_stage() {
  if [[ -z "${release_stage:-}" ]]; then
    return
  fi
  if [[ "${release_stage}" != "${release_stage_prefix}"* ]]; then
    echo "refusing to remove an unexpected release-channel staging path" >&2
    return 1
  fi
  if [[ -d "${release_stage}" ]]; then
    rm -rf -- "${release_stage}"
  fi
}
trap cleanup_release_stage EXIT

release_package_root="mcp-doctor-${release_version}"
release_entry_count=0
while IFS= read -r release_entry; do
  release_entry_count=$((release_entry_count + 1))
  if [[ "${release_entry}" != "${release_package_root}/"* ]]; then
    echo "Cargo package contains a path outside its versioned root" >&2
    exit 1
  fi
  release_relative_entry=${release_entry#"${release_package_root}/"}
  if [[ "/${release_relative_entry}/" == *"/../"* ]]; then
    echo "Cargo package contains a parent-directory traversal" >&2
    exit 1
  fi
done < <(tar -tzf "${release_cargo_package}")
if [[ "${release_entry_count}" -eq 0 ]]; then
  echo "Cargo package is empty" >&2
  exit 1
fi
while IFS= read -r release_listing; do
  case "${release_listing:0:1}" in
    - | d) ;;
    *)
      echo "Cargo package contains a non-regular archive entry" >&2
      exit 1
      ;;
  esac
done < <(LC_ALL=C tar -tvzf "${release_cargo_package}")

release_inspection_root="${release_stage}/inspection"
mkdir -p -- "${release_inspection_root}"
tar -xzf "${release_cargo_package}" -C "${release_inspection_root}"
release_manifest="${release_inspection_root}/${release_package_root}/Cargo.toml"
release_lockfile="${release_inspection_root}/${release_package_root}/Cargo.lock"
if [[ ! -f "${release_manifest}" || ! -f "${release_lockfile}" ]]; then
  echo "Cargo package does not contain its manifest and lockfile" >&2
  exit 1
fi

release_metadata="${release_stage}/metadata.json"
cargo metadata \
  --locked \
  --offline \
  --no-deps \
  --format-version 1 \
  --manifest-path "${release_manifest}" >"${release_metadata}"
jq -e \
  --arg version "${release_version}" \
  '.packages | length == 1 and
   .[0].name == "mcp-doctor" and
   .[0].version == $version and
   .[0].license == "MIT" and
   .[0].repository == "https://github.com/EnjoyableWork/mcp-doctor" and
   any(.[0].targets[]; .name == "mcp-doctor" and any(.kind[]; . == "bin"))' \
  "${release_metadata}" >/dev/null

release_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print tolower($1) }'
  else
    shasum -a 256 "$1" | awk '{ print tolower($1) }'
  fi
}

release_package_hash=$(release_sha256 "${release_cargo_package}")
if [[ ! "${release_package_hash}" =~ ^[[:xdigit:]]{64}$ ]]; then
  echo "Cargo package SHA-256 could not be determined" >&2
  exit 1
fi

release_cargo_directory="${release_stage}/cargo"
release_formula_directory="${release_stage}/homebrew/Formula"
mkdir -p -- "${release_cargo_directory}" "${release_formula_directory}"
install -m 0644 \
  "${release_cargo_package}" \
  "${release_cargo_directory}/${release_package_name}"

cat >"${release_formula_directory}/mcp-doctor.rb" <<EOF
# typed: false
# frozen_string_literal: true

class McpDoctor < Formula
  desc "Diagnose protocol, schema, and runtime failures in MCP servers"
  homepage "https://github.com/EnjoyableWork/mcp-doctor"
  url "https://github.com/EnjoyableWork/mcp-doctor/releases/download/v${release_version}/${release_package_name}"
  sha256 "${release_package_hash}"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    server = testpath/"passive-server.rb"
    server.write <<~RUBY
      #!/usr/bin/env ruby
      require "json"

      requests = []
      while (line = STDIN.gets)
        request = JSON.parse(line)
        abort "active tool call attempted" if request["method"] == "tools/call"
        requests << request["method"]
        result = case request["method"]
        when "server/discover"
          {
            "resultType" => "complete",
            "supportedVersions" => ["2026-07-28"],
            "capabilities" => { "tools" => {} },
            "ttlMs" => 0,
            "cacheScope" => "private",
          }
        when "tools/list"
          {
            "resultType" => "complete",
            "tools" => [],
            "ttlMs" => 0,
            "cacheScope" => "private",
          }
        else
          abort "unexpected method"
        end
        STDOUT.puts(JSON.generate({ "jsonrpc" => "2.0", "id" => request["id"], "result" => result }))
        STDOUT.flush
      end
      abort "unexpected passive request sequence" unless requests == ["server/discover", "tools/list"]
    RUBY
    chmod 0755, server

    output = shell_output("#{bin}/mcp-doctor inspect --format json -- #{server}")
    assert_match '"outcome": "passed"', output
    assert_match '"skip_reason": "not_authorized"', output
  end
end
EOF

rm -rf -- "${release_inspection_root}"
rm -f -- "${release_metadata}"
mv -- "${release_stage}" "${release_output_directory}"
release_stage=

printf '%s\n' "${release_output_directory}"
