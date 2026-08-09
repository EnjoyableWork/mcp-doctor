## Outcome

Describe the problem and the focused observable result of this change.

Related issue or `PROJECT.md` ticket:

## Execution and data-safety impact

Describe any effect on process execution, tool calls, network targets, limits,
cleanup, schemas, credentials, redaction, or user/production data. Write `None`
when there is no effect.

## Protocol and compatibility

List affected MCP revisions, transports, platforms, report formats, or package
surfaces. State any relevant boundary that remains unverified.

## Validation

List the exact commands, tests, fixtures, and manual checks run.

## Checklist

- [ ] This pull request addresses one accepted outcome and preserves unrelated work.
- [ ] I linked the governing issue or ticket, or explained why none is needed.
- [ ] I added or updated the narrowest useful tests for changed behavior.
- [ ] I ran the applicable repository checks and listed them above.
- [ ] Passive behavior remains passive, or active execution is explicit and bounded.
- [ ] Every started process, request, parser, and generator path remains bounded and cleaned up.
- [ ] Examples and fixtures are synthetic and no secret, private endpoint, user data, raw tool result, or unreviewed log is included.
- [ ] Suspected vulnerabilities use private reporting rather than this pull request.
- [ ] I updated public and delivery documentation for changed behavior or claims.
- [ ] I have the right to submit this contribution under the MIT License and agree to the Code of Conduct.
