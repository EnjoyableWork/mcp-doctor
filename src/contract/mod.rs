#![allow(
    dead_code,
    reason = "MCPD-004 defines contracts before MCPD-005 adds their first transport consumer"
)]

mod limits;
mod model;
mod protocol;
mod redaction;
mod report;

pub(super) fn success_exit() -> std::process::ExitCode {
    report::ExitStatus::Success.into()
}
