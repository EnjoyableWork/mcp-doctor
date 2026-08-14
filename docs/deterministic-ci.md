# Deterministic CI audit

This record implements `MCPD-030` and `DEC-054` for GitHub issue #41. The
audit was performed on 2026-08-14 from `main` commit `3472952` and covers every
tracked Rust test, shell or PowerShell script, and GitHub Actions workflow.
`PROJECT.md` remains the ticket, decision, risk, and completion authority;
`AGENTS.md` contains the concise operating policy.

## Audit method

The inventory began with `git ls-files`, then searched the complete tracked
scope for clocks, elapsed-time measurements, sleeps, polling, timeouts,
retries, process and thread concurrency, workflow concurrency, platform command
use, and runner labels. Files with a hit are classified below. All other
tracked test, script, and workflow files were inspected and had no operational
timing, retry, or concurrency exception. The regression in
`tests/deterministic_ci_policy.rs` repeats the mechanically enforceable part of
the audit on every normal test run.

The classification vocabulary is:

- **Product bound:** a monotonic deadline that is part of `mcp-doctor`'s
  documented finite-work contract.
- **Product clock input:** verification time consumed by the product contract
  itself, with a pass/fail range that remains stable across ordinary clock
  transitions and no clock-derived expected output.
- **Outer watchdog:** a deadline that only stops a broken fixture, process,
  socket, channel, job, or workflow from hanging. It is not success evidence.
- **Deterministic fixture:** observable state or an explicit
  event/acknowledgement transition that proves the intended interleaving.
- **Accepted acquisition exception:** only `DEC-043`'s classified transient
  acquisition of one exact immutable size- and digest-pinned Syft asset.
- **Defect:** timing, retry, or incidental-runner behavior that supplies
  readiness or correctness evidence. A blocking defect is corrected here; a
  release-only defect is owned by the explicit `MCPD-031` prepublication gate.

## Test and product-code inventory

| Scope | Classification and disposition |
| --- | --- |
| `src/aggregate.rs` | The ten-second monotonic aggregate deadline is a product bound. Unit tests inject an `AggregateClock` and force before/after-limit state without sleeping. |
| `src/transport/stdio.rs` | Startup, discovery, request, response, shutdown-grace, and total monotonic deadlines are product bounds. Tokio deadline selection and `sleep_until` wait on those bounds; focused unit tests use deliberately small limit values. No test infers success from elapsed wall time. |
| `src/transport/http.rs` | Resolver, connect, request, response, shutdown-grace, and total deadlines are product bounds. The HTTP client explicitly disables application retry. Focused unit tests force expired and remaining-deadline states directly. |
| `tests/stdio.rs`, `tests/active.rs`, and `tests/break.rs` | Previous built-binary timeout assertions and resistant-tree cleanup proof used elapsed thresholds and fixed sleeps. The timeout journeys now assert the typed limit and causal report only; the compiled limit profile and narrow transport tests own the numeric contract. Cleanup now requires an explicit descendant-ready acknowledgement before the server response and acquisition of an exclusive readiness-marker lock; after return, the test must acquire that same lock immediately, directly proving the descendant released it on termination. This is the deterministic correction for the retained 2026-08-14 identical-source cleanup variance. |
| `tests/fixtures/stdio_server.rs` | `thread::park` and `Child::wait` are blocking event waits, not timers. A resistant descendant creates and exclusively locks its readiness marker, then acknowledges readiness over a pipe; the parent responds only after the acknowledgement. The test's successful post-return acquisition of that lock proves termination without a delayed survival marker or an inferred timing threshold. |
| `tests/http.rs` | Disposable listeners bind before client launch. Expected accepts use a channel receive deadline as an outer watchdog, with a synthetic connection only to release a failed blocking accept. SSE hold-open, initialization timeout, and teardown stall cases observe the peer close itself. The no-extra-connection check ends only on an explicit client-complete connection acknowledgement. Socket and channel deadlines are outer watchdogs and never success thresholds. The sole test wall-clock read selects a synthetic leaf-certificate year because the platform TLS contract validates the certificate against current verification time and limits its lifetime to 397 days. Its January-through-following-February window overlaps every ordinary year transition, changes no expected report value, and is classified as a product clock input rather than readiness evidence. The policy regression permits exactly this named use and rejects every additional test clock. |
| Remaining tracked Rust tests and test support | No `SystemTime::now`, `UNIX_EPOCH`, `Instant::now`, `.elapsed()`, or `thread::sleep` remains outside the exact TLS product-clock input. File, process, schema, reporter, and policy cases use direct state, return status, exact bytes, counters, or typed results. |

Workflow `timeout-minutes`, socket timeouts, and channel receive timeouts are
outer watchdogs. A watchdog firing fails the case; it cannot make a case pass.

## Script inventory

| Files | Classification and disposition |
| --- | --- |
| `scripts/verify-organization-controls.sh` | UTC date conversion checks policy recency and a finite live API budget. Fixture mode requires its caller-supplied fixed verification date and rejects a live clock override. Curl retry is explicitly zero. These are verification product bounds and product clock inputs, not synchronization. |
| `scripts/verify-security-controls.sh`, `scripts/verify-main-protection-admin.sh`, `scripts/verify-main-protection-public.sh`, `scripts/verify-supply-chain-controls.sh`, and `scripts/verify-community-license.sh` | Current UTC dates label live, perishable verification evidence; they do not coordinate fixtures or determine a rehearsed expected value. Curl connect and total deadlines are product bounds on finite live verification, and the scripts either use no application retry or explicitly set it to zero. |
| `scripts/install-cargo-deny.sh` | Curl connect and total deadlines are outer acquisition watchdogs. The exact immutable asset is attempted once and receives the same integrity, layout, version, and platform checks as every accepted repository-managed executable. |
| `scripts/package-release.sh` | `SOURCE_DATE_EPOCH` must equal the release commit timestamp and controls reproducible archive metadata. It is deterministic input, not a readiness clock. |
| `scripts/install-syft.sh` and `scripts/rehearse-syft-acquisition.sh` | The sole `DEC-043` acquisition exception permits at most three attempts for the enumerated transient curl/HTTP failures against one immutable asset, with a one-second delay, deletion of partial bytes, and identical size/digest/layout/version/platform verification. Curl's own retry is zero. The rehearsal replaces both curl and sleep with deterministic fakes and proves attempt counts; no correctness step is retried. |
| `scripts/generate-release-sbom.sh` | The standalone `timeout` process is an outer watchdog around one exact repository-acquired Syft invocation. A timeout fails generation and is never retried. |
| `scripts/verify-ci-tools.sh` and `scripts/verify-ci-tools.ps1` | The POSIX and Windows bootstrap uses only its selected shell's built-ins until it verifies the inventory parser, requires one exact runner contract, rejects unsafe command names, and checks every declared command before evidence work. It has no clock, sleep, retry, polling, concurrency, download, or fallback. |
| `scripts/verify-release-repository-controls.sh` | One broad `curl --retry 5` remains release-only legacy behavior. It is a defect owned by `MCPD-031` and cannot support another publication until corrected and rehearsed. |
| Remaining tracked scripts | No operational clock, fixed sleep, timeout, polling loop, positive retry, or concurrent process occurs. Script-local `command -v` checks, exact hash-tool alternatives, and `.github/ci-tools.json` declare the non-standard execution surface. Strings used to generate a Ruby formula or run Node/PHP inside digest-pinned compatibility containers do not grant ambient host-tool authority. |

## Workflow inventory

All six tracked workflows declare run-level concurrency behavior, and every one
of their 25 jobs has one `timeout-minutes` outer watchdog. CI and preflight
matrices use `fail-fast: false` to retain every native result, not to retry it.
PR/main concurrency cancellation discards obsolete work; it never reruns a
failed exact head. Release concurrency protects the demonstrated exclusive
publication resource and does not serialize the test suite.

| Files | Classification and disposition |
| --- | --- |
| `.github/workflows/ci.yml`, `.github/workflows/release-preflight.yml`, and `.github/workflows/compatibility.yml` | Required and supplemental evidence has no automatic test, job, or workflow retry. Exact runner labels, the exact Rust toolchain, full-SHA Actions, repository-acquired tools, and the declared runner commands in `.github/ci-tools.json` define the tool surface. Every source-checkout job verifies its exact runner contract immediately after checkout through `scripts/verify-ci-tools.sh` or `scripts/verify-ci-tools.ps1`; aggregate jobs use shell built-ins only. Missing or undeclared commands fail before evidence work. |
| `.github/workflows/release-authorization-negative.yml` | `continue-on-error` captures the expected rejected OIDC attempt so the following assertion can require that exact failure. It is a deterministic negative transition, not quarantine or retry. |
| `.github/workflows/release.yml` | Five broad curl retries and two fixed-five-second GitHub/crates.io publication-state loops remain in the release-only producer. Its source-checkout jobs also predate the executable runner-contract verifier. These defects are owned by `MCPD-031`; no successor release may cite this path until retry, state observation, and runner verification are corrected and pass a nonpublishing rehearsal. Existing immutable releases are unchanged. |
| `.github/workflows/release-channels.yml` | Three broad curl retries remain in the manually dispatched read-only installed-channel verifier, whose immutable `v0.1.0` through `v0.3.0` checkouts necessarily predate the new verifier scripts. Both boundaries are included in the same `MCPD-031` prepublication gate for a successor release without changing historical tags or artifacts. |

## CI tool inventory

`.github/ci-tools.json` declares the exact runner labels, Rust `1.97.1`,
non-standard runner commands, action-provided Homebrew command, digest-pinned
container commands, and repository-acquired `cargo-deny 0.20.2` and Syft
`1.51.0`. The acquired-tool identities must equal
`.github/supply-chain-controls.json`. The policy regression rejects `rg` as an
incidental runner dependency, requires every observed non-standard command
class to remain declared, and locks the current hosted-workflow verifier
coverage. The POSIX and PowerShell verifiers reject an unknown runner contract
and check every declared runner command before evidence work. Action-provided,
container-provided, generated, and repository-acquired tools are instead
verified at their controlled provision point. POSIX shell built-ins remain
bounded by the exact runner label; nothing is silently replaced or downloaded.

Adding a command, runner, Action-provided tool, container runtime, standalone
executable, sleep, positive retry, or timing primitive reopens `RISK-27` and
requires this inventory, its owning decision, and the policy regression to be
updated together.

## Preserved failure evidence and completion gate

- PR #40's first hosted CI attempt
  [31625931074](https://github.com/EnjoyableWork/mcp-doctor/actions/runs/31625931074)
  remains the evidence that optional `rg` and an independently read fixture
  clock were real runner-dependent defects; commit `c09b4b1` corrected that
  focused boundary without retry or timeout inflation.
- During the issue #75 work on 2026-08-14,
  `cleanup_terminates_a_resistant_process_tree_before_returning` failed once at
  about 12 seconds against an eight-second elapsed assertion, then passed from
  identical source in isolation. The green rerun is not acceptance evidence.
  `MCPD-030` replaces that timing proof with the forced descendant-ready and
  exclusive-lock-release transition described above.

`MCPD-030` is complete only after the normal local gates and all applicable
exact-head hosted checks pass on their first run, the protected change merges,
issue #41 closes, and exact-`main` gates pass. No retry, workflow rerun, product
timeout change, safety weakening, release mutation, live-setting change, or M4
claim may satisfy that gate.
