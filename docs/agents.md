# Use mcp-doctor with coding agents

The optional `mcp-doctor` Agent Skill teaches a coding agent to run one safe,
passive preflight and interpret the stable redacted report. The interaction is:

```text
coding agent -> terminal -> mcp-doctor CLI -> exact MCP server target
```

The skill does not turn `mcp-doctor` into an MCP server, grant terminal
permissions, install the CLI, select a target, or authorize tool calls. It
refuses `check`, `break`, and `reject`; the CLI remains the enforcement boundary
even when a host does not follow an instruction.

## Choose a skill installation route

The CLI and Agent Skill are separate installations. Install the CLI through a
[documented channel](../README.md#install), then choose one skill route:

| Route | Use it when | Authority |
| --- | --- | --- |
| [Smithery](https://smithery.ai/skills/enjoyable/mcp-doctor) | You want discovery and the current install command for a selected agent host | Third-party installation convenience backed by the canonical GitHub directory |
| [GitHub source](https://github.com/EnjoyableWork/mcp-doctor/tree/main/.agents/skills/mcp-doctor) | You want to inspect the current bundle or keep it under version control | Canonical source for the current skill |
| [GitHub Release](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.4.0) | You want an immutable skill and CLI pair with published checksums | Canonical release authority |

On Smithery, select the intended agent and use the command the listing displays
for that host. The listing currently resolves to the canonical GitHub skill
directory, but it is not an immutable release record. For a reproducible install,
use the exact release archive and verification flow below. Do not install both
the Smithery and manual copies into the same discovery root.

## Published release identity

The immutable `mcp-doctor` `0.4.0` release published the original portable
single-file skill:

| Item | Exact identity |
| --- | --- |
| CLI | `mcp-doctor 0.4.0` |
| GitHub Release | `v0.4.0` |
| Companion archive | `mcp-doctor-agent-skill-v0.4.0.tar.gz` |
| Archive manifest | `SHA256SUMS` from the same immutable release |
| Published `SKILL.md` | SHA-256 `aacc85b225bcb93cd0f5cc926388ad1f5763a0b5cb771255148453c2257cd991` |

Do not combine the skill from one release with a different CLI version. The
skill checks the installed version and compiled capabilities before a new
diagnosis, then stops on a mismatch.

The source tree now carries a ChatGPT presentation bundle paired with the same
`mcp-doctor 0.4.0` CLI contract. It does not alter the immutable release above:

| Source item | Exact identity |
| --- | --- |
| Source `SKILL.md` | SHA-256 `05d0f9ab54b41d9f74605aa3c84c6d9837484539895033e3bcf058afcc596cb6` |
| Source `agents/openai.yaml` | SHA-256 `a56095c3f3eb2ed6bdbceb9b4d6c40289b5bb45733c4c950c32a0c02bbd680d6` |
| Source `assets/icon.svg` | SHA-256 `8140b500f4bc70688a473bc9ec63cdb0b1a3e229596215340588053a3ee1d71b` |

The behavioral instructions remain self-contained in `SKILL.md`. The other two
files add OpenAI-specific presentation and invocation metadata; a host that
supports only `SKILL.md` does not need them.

On a first run without the CLI, the skill presents the exact `0.4.0` Cargo,
Homebrew, and verified-release routes, then stops. Those commands are a
human-facing prerequisite handoff; the skill never installs software itself.

## Build the ChatGPT upload ZIP

The canonical source layout is:

```text
mcp-doctor/
|-- SKILL.md
|-- agents/
|   `-- openai.yaml
`-- assets/
    `-- icon.svg
```

`agents/openai.yaml` presents the display name **MCP Doctor**, the concise
human-facing line “Diagnose MCP servers before users do—from local to
production,” the approved logo, and a passive starter prompt. ChatGPT and Codex
use the fuller frontmatter description for implicit matching, so it also names
the exact trigger and safety boundaries. Implicit invocation remains enabled,
but the skill still requires one exact selected target before it runs anything.
The SVG leaves a transparent safe area of about 12% on each edge and the
metadata does not request a brand-color tile. A host may still draw its own
surface behind transparent icons.

Build a deterministic ZIP whose upload root contains `SKILL.md`, `agents/`, and
`assets/` directly:

```bash
SOURCE_DATE_EPOCH=$(git log -1 --format=%ct) \
  scripts/package-chatgpt-skill.sh 0.4.0 target/skill-dist
scripts/verify-chatgpt-skill.sh 0.4.0 \
  target/skill-dist/mcp-doctor-chatgpt-skill-v0.4.0.zip
```

Upload `mcp-doctor-chatgpt-skill-v0.4.0.zip` through the ChatGPT skill import
flow available to the account. OpenAI documents standalone skills for the
ChatGPT desktop app, Codex CLI, and the IDE extension. For public installation
by other people across ChatGPT surfaces, OpenAI currently directs vendors to
package reusable skills as plugins; this ZIP is a standalone skill bundle, not
a plugin or a public-listing claim.

## Install explicitly

Install the CLI through one of the [documented channels](../README.md#install)
first. CLI installation never changes an agent host. Download the companion
archive and `SHA256SUMS` separately from the exact release page; do not pipe a
remote installer into a shell.

Verify the archive before extracting it.

On macOS or GNU/Linux:

```bash
archive=mcp-doctor-agent-skill-v0.4.0.tar.gz
expected_archive=$(sed -n \
  's/^\([[:xdigit:]]\{64\}\)  mcp-doctor-agent-skill-v0\.4\.0\.tar\.gz$/\1/p' \
  SHA256SUMS)
test "${#expected_archive}" = 64
actual_archive=$(shasum -a 256 "$archive" | awk '{print $1}')
test "$actual_archive" = "$expected_archive"
mkdir mcp-doctor-skill-stage
tar -xzf "$archive" -C mcp-doctor-skill-stage
test "$(shasum -a 256 mcp-doctor-skill-stage/mcp-doctor/SKILL.md \
  | awk '{print $1}')" = \
  aacc85b225bcb93cd0f5cc926388ad1f5763a0b5cb771255148453c2257cd991
```

On Windows PowerShell:

```powershell
$archive = 'mcp-doctor-agent-skill-v0.4.0.tar.gz'
$checksumLines = @(Get-Content -LiteralPath SHA256SUMS | Where-Object {
    $_.EndsWith("  $archive", [StringComparison]::Ordinal)
})
if ($checksumLines.Count -ne 1 -or $checksumLines[0].Length -ne (66 + $archive.Length)) {
    throw 'Agent Skill archive checksum entry is missing or ambiguous.'
}
$expectedArchive = $checksumLines[0].Substring(0, 64).ToLowerInvariant()
if ($expectedArchive -notmatch '^[0-9a-f]{64}$') { throw 'Invalid archive checksum entry.' }
$actualArchive = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash.ToLowerInvariant()
if ($actualArchive -ne $expectedArchive) { throw 'Agent Skill archive digest mismatch.' }
New-Item -ItemType Directory -Path mcp-doctor-skill-stage | Out-Null
tar -xzf $archive -C mcp-doctor-skill-stage
if ($LASTEXITCODE -ne 0) { throw 'Could not extract the Agent Skill archive.' }
$skillFile = 'mcp-doctor-skill-stage\mcp-doctor\SKILL.md'
$actualSkill = (Get-FileHash -Algorithm SHA256 -LiteralPath $skillFile).Hash.ToLowerInvariant()
if ($actualSkill -ne 'aacc85b225bcb93cd0f5cc926388ad1f5763a0b5cb771255148453c2257cd991') {
    throw 'Canonical SKILL.md digest mismatch.'
}
```

Choose one primary-documented skill root:

| Host | Personal skill root | Workspace skill root |
| --- | --- | --- |
| Codex | `~/.agents/skills` | `.agents/skills` |
| Claude Code | `~/.claude/skills` | `.claude/skills` |
| Cursor | `~/.agents/skills` | `.agents/skills` |
| VS Code / GitHub Copilot | `~/.agents/skills` | `.agents/skills` |
| Kiro IDE | `~/.kiro/skills` | `.kiro/skills` |
| Kiro CLI | `~/.kiro/skills` | `.kiro/skills` |
| Kiro Crew | Not claimed | `.kiro/skills` in one foreground interactive workspace only |

Install without overwriting an existing skill. On macOS or GNU/Linux, set the
release-specific destination you selected above:

```bash
MCP_DOCTOR_SKILLS_ROOT="$HOME/.agents/skills"
mkdir -p "$MCP_DOCTOR_SKILLS_ROOT"
mkdir "$MCP_DOCTOR_SKILLS_ROOT/mcp-doctor"
install -m 0644 mcp-doctor-skill-stage/mcp-doctor/SKILL.md \
  "$MCP_DOCTOR_SKILLS_ROOT/mcp-doctor/SKILL.md"
```

`mkdir` must fail if `mcp-doctor` already exists. Inspect that existing skill;
never replace it silently. Use `~/.claude/skills`, `~/.kiro/skills`, or a
workspace root from the table when that is the host and scope you selected.

On Windows PowerShell:

```powershell
$skillsRoot = Join-Path $env:USERPROFILE '.agents\skills'
$destination = Join-Path $skillsRoot 'mcp-doctor'
New-Item -ItemType Directory -Path $skillsRoot -Force | Out-Null
if (Test-Path -LiteralPath $destination) { throw 'mcp-doctor skill already exists.' }
New-Item -ItemType Directory -Path $destination -Force:$false | Out-Null
Copy-Item -LiteralPath $skillFile -Destination (Join-Path $destination 'SKILL.md')
```

Change only `$skillsRoot` when selecting a different documented host root.

## Verify discovery and invoke explicitly

Start a new foreground session in the intended workspace. Host discovery and
model selection are separate: seeing a skill proves only that the host found
its metadata. Explicit invocation is the supported route; implicit selection is
best-effort host behavior and is never guaranteed.

| Host | Verify discovery | Explicit invocation |
| --- | --- | --- |
| Codex | Run `/skills` and find `mcp-doctor` | Mention `$mcp-doctor` in the prompt |
| Claude Code | Type `/` and find `mcp-doctor` | Run `/mcp-doctor` with one exact target |
| Cursor | Open **Customize → Skills** | Select `/mcp-doctor` in Agent chat |
| VS Code / GitHub Copilot | Run `/skills` or open **Configure Skills** | Select `/mcp-doctor` in agent chat |
| Kiro IDE | Open **Agent Steering & Skills** | Select `/mcp-doctor` in chat |
| Kiro CLI | Run `/context show` | Run `/mcp-doctor` with one exact target |
| Kiro Crew | Confirm the workspace skill in one foreground conversation | Run `/mcp-doctor` in that conversation only |

The Kiro Crew route does not cover schedules, proactive loops, webhooks,
heartbeats, Apps, synthesized skills, persistent lessons, background tasks,
unattended work, messaging surfaces, or cross-surface behavior.

## Remove safely

Remove only the exact unmodified file installed above. On macOS or GNU/Linux:

```bash
MCP_DOCTOR_SKILLS_ROOT="$HOME/.agents/skills"
test "$(shasum -a 256 "$MCP_DOCTOR_SKILLS_ROOT/mcp-doctor/SKILL.md" \
  | awk '{print $1}')" = \
  aacc85b225bcb93cd0f5cc926388ad1f5763a0b5cb771255148453c2257cd991
rm "$MCP_DOCTOR_SKILLS_ROOT/mcp-doctor/SKILL.md"
rmdir "$MCP_DOCTOR_SKILLS_ROOT/mcp-doctor"
```

On Windows PowerShell, compute the same SHA-256 first, then remove only the file
and now-empty directory:

```powershell
$installedSkill = Join-Path $destination 'SKILL.md'
$installedDigest = (Get-FileHash -Algorithm SHA256 -LiteralPath $installedSkill).Hash.ToLowerInvariant()
if ($installedDigest -ne 'aacc85b225bcb93cd0f5cc926388ad1f5763a0b5cb771255148453c2257cd991') {
    throw 'Installed skill was modified; refusing documented removal.'
}
Remove-Item -LiteralPath $installedSkill
Remove-Item -LiteralPath $destination
```

If the digest differs, stop and review the local changes instead of deleting
them. Updating uses the same verified removal followed by a fresh explicit
install; the CLI and package channels never update a skill automatically.

## Evaluation contract

The source skill has two evaluated packaging profiles: the portable
`mcp-doctor/SKILL.md` instruction file and the ChatGPT bundle containing that
file plus `agents/openai.yaml` and `assets/icon.svg`. The prompts, assertions,
synthetic recorder, and result-recording rules stay outside both artifacts, so
a vendor does not need repository test files at runtime.

The public [Agent Skill evaluation contract](evaluations/agent-skill-v1.md)
separates artifact-profile conformance, registry ingestion, host discovery,
selection, safe execution, and result quality. A listing or successful upload
proves only the boundary it observed; it is not evidence of host execution or
independent adoption. Vendor claims remain withheld until the exact skill and
host identity pass their applicable cases on a first attempt.

OpenAI documents `SKILL.md` as required, with `assets/` and
`agents/openai.yaml` optional. The optional files do not weaken the portable
instruction-only profile or create a runtime dependency.

## Dated host evidence

The dated host matrix records exact-version clean-workspace observations only
after one first-attempt explicit invocation uses a synthetic PATH-preferred
`mcp-doctor` recorder. A missing, failed, or variant observation withholds that
host claim; it is not rerun into acceptance.

Neither the published `0.4.0` release skill nor the newer ChatGPT source bundle
has completed a new host observation. The last verified route is Codex CLI
`0.147.0` paired with the exact `mcp-doctor` `0.3.2` CLI and companion skill.
That remains historical evidence for `0.3.2`, not a support claim for `0.3.3`,
`0.4.0`, a later Codex version, ChatGPT, or independent adoption.

| Host | Exact version | Discovery | Explicit invocation | Implicit observation |
| --- | --- | --- | --- | --- |
| Codex CLI | `0.147.0` | 2026-08-16 clean headless discovery passed; exact `0.3.2` route supported | Seven explicit synthetic cases passed; exact `0.3.2` route supported | Passive case selected the skill and passed; best-effort observation only |
| Claude Code | `2.1.220` | Clean bare `/mcp-doctor` returned `Unknown command` | Variant; not claimed | Not evaluated after explicit variant |
| Cursor Agent | `2026.05.20-2b5dd59` | First headless invocation required authentication despite successful status | Not reached; not claimed | Not evaluated |
| VS Code / GitHub Copilot | Not installed on the review host | Not claimed | Not claimed | Not claimed |
| Kiro IDE | `1.0.288` | Foreground accessibility state timed out | Not reached; not claimed | Not evaluated |
| Kiro CLI | Not found on PATH | Not claimed | Not claimed | Not claimed |
| Kiro Crew | `0.1.3` | Foreground accessibility state timed out | Not reached; not claimed | Not evaluated |

These are scoped integration observations, not deterministic model-correctness,
adoption, universal compatibility, or independent-use evidence.
[Read the bounded recorder and withheld-claim record](assurance/v0.3.2-agent-skill.md).

## Primary host documentation

The routes above were revalidated on 2026-08-24 against the
[Agent Skills specification](https://agentskills.io/specification),
[OpenAI skills documentation](https://learn.chatgpt.com/docs/build-skills),
[Claude Code skills documentation](https://code.claude.com/docs/en/slash-commands),
[Cursor skills documentation](https://cursor.com/docs/skills),
[VS Code Agent Skills documentation](https://code.visualstudio.com/docs/agent-customization/agent-skills),
[Kiro Agent Skills documentation](https://kiro.dev/docs/skills/), and the
[Kiro Crew implementation record](https://github.com/kirodotdev/KiroCrew).
