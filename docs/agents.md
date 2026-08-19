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

## Release identity

This guide and the canonical skill belong to `mcp-doctor` `0.3.3`:

| Item | Exact identity |
| --- | --- |
| CLI | `mcp-doctor 0.3.3` |
| Release | [`v0.3.3`](https://github.com/EnjoyableWork/mcp-doctor/releases/tag/v0.3.3) |
| Companion archive | `mcp-doctor-agent-skill-v0.3.3.tar.gz` |
| Archive manifest | `SHA256SUMS` from the same immutable release |
| Canonical `SKILL.md` | SHA-256 `4ef5796bded1d2b7261e1b7d330c347aa9dfde9f7826cb8ab879290d9a40b1cf` |

Do not combine the skill from one release with a different CLI version. The
skill checks the installed version and compiled capabilities before a new
diagnosis, then stops on a mismatch.

## Install explicitly

Install the CLI through one of the [documented channels](../README.md#install)
first. CLI installation never changes an agent host. Download the companion
archive and `SHA256SUMS` separately from the exact release page; do not pipe a
remote installer into a shell.

Verify the archive before extracting it.

On macOS or GNU/Linux:

```bash
archive=mcp-doctor-agent-skill-v0.3.3.tar.gz
expected_archive=$(sed -n \
  's/^\([[:xdigit:]]\{64\}\)  mcp-doctor-agent-skill-v0\.3\.3\.tar\.gz$/\1/p' \
  SHA256SUMS)
test "${#expected_archive}" = 64
actual_archive=$(shasum -a 256 "$archive" | awk '{print $1}')
test "$actual_archive" = "$expected_archive"
mkdir mcp-doctor-skill-stage
tar -xzf "$archive" -C mcp-doctor-skill-stage
test "$(shasum -a 256 mcp-doctor-skill-stage/mcp-doctor/SKILL.md \
  | awk '{print $1}')" = \
  4ef5796bded1d2b7261e1b7d330c347aa9dfde9f7826cb8ab879290d9a40b1cf
```

On Windows PowerShell:

```powershell
$archive = 'mcp-doctor-agent-skill-v0.3.3.tar.gz'
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
if ($actualSkill -ne '4ef5796bded1d2b7261e1b7d330c347aa9dfde9f7826cb8ab879290d9a40b1cf') {
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
  4ef5796bded1d2b7261e1b7d330c347aa9dfde9f7826cb8ab879290d9a40b1cf
rm "$MCP_DOCTOR_SKILLS_ROOT/mcp-doctor/SKILL.md"
rmdir "$MCP_DOCTOR_SKILLS_ROOT/mcp-doctor"
```

On Windows PowerShell, compute the same SHA-256 first, then remove only the file
and now-empty directory:

```powershell
$installedSkill = Join-Path $destination 'SKILL.md'
$installedDigest = (Get-FileHash -Algorithm SHA256 -LiteralPath $installedSkill).Hash.ToLowerInvariant()
if ($installedDigest -ne '4ef5796bded1d2b7261e1b7d330c347aa9dfde9f7826cb8ab879290d9a40b1cf') {
    throw 'Installed skill was modified; refusing documented removal.'
}
Remove-Item -LiteralPath $installedSkill
Remove-Item -LiteralPath $destination
```

If the digest differs, stop and review the local changes instead of deleting
them. Updating uses the same verified removal followed by a fresh explicit
install; the CLI and package channels never update a skill automatically.

## Dated host evidence

The dated host matrix records exact-version clean-workspace observations only
after one first-attempt explicit invocation uses a synthetic PATH-preferred
`mcp-doctor` recorder. A missing, failed, or variant observation withholds that
host claim; it is not rerun into acceptance.

The `0.3.3` candidate has not yet completed a new host observation. The last
verified route is Codex CLI `0.147.0` paired with the exact `mcp-doctor` `0.3.2`
CLI and companion skill. That remains historical evidence for `0.3.2`, not a
support claim for `0.3.3`, a later Codex version, or independent adoption.

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
[Read the bounded recorder and withheld-claim record](assurance/mcpd-035-agent-skill.md).

## Primary host documentation

The routes above were revalidated on 2026-08-16 against the
[Agent Skills specification](https://agentskills.io/specification),
[Codex skills documentation](https://developers.openai.com/codex/skills/),
[Claude Code skills documentation](https://code.claude.com/docs/en/slash-commands),
[Cursor skills documentation](https://cursor.com/docs/skills),
[VS Code Agent Skills documentation](https://code.visualstudio.com/docs/agent-customization/agent-skills),
[Kiro Agent Skills documentation](https://kiro.dev/docs/skills/), and the
[Kiro Crew implementation record](https://github.com/kirodotdev/KiroCrew).
