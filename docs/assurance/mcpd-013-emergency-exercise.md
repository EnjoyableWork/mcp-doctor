# MCPD-013 emergency-bypass exercise

This is a bounded control exercise for `DEC-035`, not a production incident or
an assurance claim.

## Start record

| Field | Recorded value |
| --- | --- |
| Incident ID | `MCPD-013-EXERCISE-20260811-01` |
| Reason | Prove that the documented emergency path can merge exactly one dedicated pull request without weakening direct-update, deletion, or non-fast-forward protection, and can then return to an empty standing-bypass state. |
| Protected base | `main` at `09765f6fe13eb050de32033fc6d51b3e8b5da37f` |
| Exact change commit | Recorded in the dedicated pull request because a commit cannot contain its own identifier. |
| Canonical pre-change SHA-256 | `2e3377a5101c513c02bb177cbc95acc3707f77bab4c3ab8ed3e8576a3f828794` for `.github/rulesets/main.json` |
| Start time | `2026-08-11T21:39:06Z` |
| Temporary actor | Built-in repository-administrator role; the non-disclosing preflight confirmed the accepted single-administrator boundary. |
| Bypass mode | `pull_request` only |
| Pull-request budget | One dedicated pull request and one squash merge |
| Rules eligible for bypass | Pull-request merge requirements, including the strict `Required CI` and `Required release preflight` gates; the exercise must record which requirements were incomplete at merge time. |
| Prohibited paths | Disabling the ruleset, `always` bypass, direct update, deletion, and non-fast-forward update remain prohibited. |
| Rollback owner | Repository administrator |

Before the temporary actor is added, the dedicated pull request must identify
its exact head commit and show an incomplete required gate. The actor must be
removed immediately after that one merge. If the merge cannot complete, actor
removal takes precedence over investigation.

## Closure record

Status: closed at `2026-08-11T21:55:10Z`

| Field | Recorded value |
| --- | --- |
| Dedicated pull request | [PR 18](https://github.com/EnjoyableWork/mcp-doctor/pull/18) |
| Exact change commit | `05090b3b62ae145f06dbdd69f3346e4cd2fa607a` |
| Pre-merge state | `BLOCKED`; neither `Required CI` nor `Required release preflight` had reported |
| Requirement exercised | Strict required-status-check completion. The branch was current, had no unresolved review thread, and used a pull request and squash merge, so no other merge requirement needed bypass. |
| Merge-window activation | `2026-08-11T21:42:39.603Z`; one built-in repository-administrator role, `pull_request` mode |
| Merge | `2026-08-11T21:42:41Z`; squash commit `8487b47dbddb2dd1c50020b5b157d9807bc4fcd7` |
| Bypass removal | `2026-08-11T21:42:44.005Z`; active ruleset with zero bypass actors |
| Public projection after removal | `date=2026-08-11 canonical_sha256=2e3377a5101c513c02bb177cbc95acc3707f77bab4c3ab8ed3e8576a3f828794 result=PASS` |
| Administrative readback after removal | The same bounded date, canonical hash, and `PASS`; no settings, actor inventory, or identity were emitted |
| Post-removal CI | [Required CI](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31539153287/job/93938063807) passed on the exact squash commit |
| Post-removal release preflight | [Required release preflight](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31539153316/job/93940246247) passed on the exact squash commit |
| Head branch | Deleted after merge |

The first activation at `2026-08-11T21:41:04Z` did not merge or update a
protected ref. A local evidence-formatting command failed, and its rollback
trap restored the empty bypass list at `2026-08-11T21:41:38.192Z` while PR 18
remained open and `main` remained
`09765f6fe13eb050de32033fc6d51b3e8b5da37f`. The merge window above began only
after that empty state was read back again. Recording this aborted window is
part of the exercise evidence; it consumed no pull-request merge budget.

The actor existed only during the recorded windows and never had `always`
mode. The ruleset was never disabled, and the exercise did not directly update,
delete, or non-fast-forward `main`. Public evidence proves the configured and
effective rule projection. The authenticated result is intentionally
non-disclosing and self-attested because GitHub hides bypass actors and merge
settings from credential-free repository readback. No actor identity,
credential, security finding, or private setting value is part of this record.
