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

Status: pending

The protected follow-up pull request will record the exact emergency pull
request and squash commit, the requirements incomplete at merge time, removal
time, empty-bypass readback, canonical public-projection verification, and
post-merge gate evidence. Security-sensitive detail and actor identities are
not public evidence.
