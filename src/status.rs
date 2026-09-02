use std::fmt::{self, Write as _};
use std::io::Write;

use serde::Serialize;

pub(crate) const STATUS_SCHEMA_VERSION: &str = "mcp-doctor.status/v1";
pub(crate) const MAXIMUM_EVENT_BYTES: usize = 512;
pub(crate) const MAXIMUM_EVENTS: usize = 128;
pub(crate) const MAXIMUM_OUTPUT_BYTES: usize = MAXIMUM_EVENT_BYTES * MAXIMUM_EVENTS;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum StatusFormat {
    Plain,
    Jsonl,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StatusCommand {
    Inspect,
    Check,
    Break,
    Reject,
}

impl StatusCommand {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Inspect => "inspect",
            Self::Check => "check",
            Self::Break => "break",
            Self::Reject => "reject",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StatusTransport {
    Stdio,
    StreamableHttp,
}

impl StatusTransport {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Stdio => "stdio",
            Self::StreamableHttp => "streamable_http",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StatusPhase {
    InputPreparation,
    TargetPreparation,
    TargetStartup,
    Discovery,
    Cleanup,
    ReportPublication,
}

impl StatusPhase {
    const fn as_str(self) -> &'static str {
        match self {
            Self::InputPreparation => "input_preparation",
            Self::TargetPreparation => "target_preparation",
            Self::TargetStartup => "target_startup",
            Self::Discovery => "discovery",
            Self::Cleanup => "cleanup",
            Self::ReportPublication => "report_publication",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StatusCeilingKind {
    Startup,
    Discovery,
    CleanupGrace,
}

impl StatusCeilingKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Startup => "startup",
            Self::Discovery => "discovery",
            Self::CleanupGrace => "cleanup_grace",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StatusErrorKind {
    InvalidInvocationOrInput,
    InternalOrOutputFailure,
}

impl StatusErrorKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInvocationOrInput => "invalid_invocation_or_input",
            Self::InternalOrOutputFailure => "internal_or_output_failure",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct StatusCeiling {
    pub(crate) kind: StatusCeilingKind,
    pub(crate) milliseconds: u64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct StatusContext {
    command: StatusCommand,
    transport: StatusTransport,
    limit_profile: &'static str,
}

impl StatusContext {
    pub(crate) const fn new(
        command: StatusCommand,
        transport: StatusTransport,
        limit_profile: &'static str,
    ) -> Self {
        Self {
            command,
            transport,
            limit_profile,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
enum StatusEvent {
    InvocationAccepted,
    PhaseStarted {
        phase: StatusPhase,
        #[serde(skip_serializing_if = "Option::is_none")]
        ceiling_kind: Option<StatusCeilingKind>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ceiling_ms: Option<u64>,
    },
    CaseStarted {
        ordinal: u64,
        total: u64,
        request_ceiling_ms: u64,
        response_ceiling_ms: u64,
    },
    Error {
        error_kind: StatusErrorKind,
    },
    Completed {
        exit_code: u8,
        exit_meaning: &'static str,
    },
}

#[derive(Debug, Serialize)]
struct StatusRecord {
    schema_version: &'static str,
    command: StatusCommand,
    transport: StatusTransport,
    limit_profile: &'static str,
    #[serde(flatten)]
    event: StatusEvent,
}

pub(crate) trait StatusObserver {
    fn phase_started(&mut self, phase: StatusPhase, ceiling: Option<StatusCeiling>);

    fn case_started(
        &mut self,
        ordinal: u64,
        total: u64,
        request_ceiling_ms: u64,
        response_ceiling_ms: u64,
    );
}

pub(crate) struct StatusReporter<W: Write> {
    writer: W,
    format: Option<StatusFormat>,
    context: StatusContext,
    events: usize,
    output_bytes: usize,
    failed: bool,
}

impl<W: Write> StatusReporter<W> {
    pub(crate) const fn new(
        writer: W,
        format: Option<StatusFormat>,
        context: StatusContext,
    ) -> Self {
        Self {
            writer,
            format,
            context,
            events: 0,
            output_bytes: 0,
            failed: false,
        }
    }

    pub(crate) fn invocation_accepted(&mut self) {
        self.emit(StatusEvent::InvocationAccepted);
    }

    pub(crate) fn error(&mut self, kind: StatusErrorKind, error: &dyn fmt::Display) {
        match self.format {
            Some(StatusFormat::Jsonl) => self.emit(StatusEvent::Error { error_kind: kind }),
            Some(StatusFormat::Plain) => {
                self.emit(StatusEvent::Error { error_kind: kind });
                self.write_plain_error(error);
            }
            None => {
                let _ = writeln!(self.writer, "error: {error}");
            }
        }
    }

    pub(crate) fn complete(&mut self, exit_code: u8) -> u8 {
        self.emit(StatusEvent::Completed {
            exit_code,
            exit_meaning: exit_meaning(exit_code),
        });
        if self.failed { 4 } else { exit_code }
    }

    #[cfg(test)]
    const fn failed(&self) -> bool {
        self.failed
    }

    #[cfg(test)]
    fn into_inner(self) -> W {
        self.writer
    }

    fn emit(&mut self, event: StatusEvent) {
        let Some(format) = self.format else {
            return;
        };
        if self.failed {
            return;
        }

        let record = StatusRecord {
            schema_version: STATUS_SCHEMA_VERSION,
            command: self.context.command,
            transport: self.context.transport,
            limit_profile: self.context.limit_profile,
            event,
        };
        let mut output = match format {
            StatusFormat::Plain => render_plain(record),
            StatusFormat::Jsonl => match serde_json::to_string(&record) {
                Ok(output) => output,
                Err(_) => {
                    self.failed = true;
                    return;
                }
            },
        };
        output.push('\n');
        self.write_event(output.as_bytes());
    }

    fn write_event(&mut self, bytes: &[u8]) {
        let next_events = self.events.saturating_add(1);
        let next_bytes = self.output_bytes.saturating_add(bytes.len());
        if bytes.len() > MAXIMUM_EVENT_BYTES
            || next_events > MAXIMUM_EVENTS
            || next_bytes > MAXIMUM_OUTPUT_BYTES
        {
            self.failed = true;
            return;
        }

        if self.writer.write_all(bytes).is_err() || self.writer.flush().is_err() {
            self.failed = true;
            return;
        }
        self.events = next_events;
        self.output_bytes = next_bytes;
    }

    fn write_plain_error(&mut self, error: &dyn fmt::Display) {
        if self.failed {
            return;
        }
        let mut output = String::new();
        if writeln!(output, "error: {error}").is_err()
            || output.len() > MAXIMUM_EVENT_BYTES
            || self.output_bytes.saturating_add(output.len()) > MAXIMUM_OUTPUT_BYTES
            || self.writer.write_all(output.as_bytes()).is_err()
            || self.writer.flush().is_err()
        {
            self.failed = true;
            return;
        }
        self.output_bytes = self.output_bytes.saturating_add(output.len());
    }
}

impl<W: Write> StatusObserver for StatusReporter<W> {
    fn phase_started(&mut self, phase: StatusPhase, ceiling: Option<StatusCeiling>) {
        self.emit(StatusEvent::PhaseStarted {
            phase,
            ceiling_kind: ceiling.map(|value| value.kind),
            ceiling_ms: ceiling.map(|value| value.milliseconds),
        });
    }

    fn case_started(
        &mut self,
        ordinal: u64,
        total: u64,
        request_ceiling_ms: u64,
        response_ceiling_ms: u64,
    ) {
        self.emit(StatusEvent::CaseStarted {
            ordinal,
            total,
            request_ceiling_ms,
            response_ceiling_ms,
        });
    }
}

fn render_plain(record: StatusRecord) -> String {
    let mut output = format!(
        "mcp-doctor status · {} · command={} · transport={} · limit_profile={}",
        event_name(record.event),
        record.command.as_str(),
        record.transport.as_str(),
        record.limit_profile,
    );
    match record.event {
        StatusEvent::InvocationAccepted => {}
        StatusEvent::PhaseStarted {
            phase,
            ceiling_kind,
            ceiling_ms,
        } => {
            let _ = write!(output, " · phase={}", phase.as_str());
            if let (Some(kind), Some(milliseconds)) = (ceiling_kind, ceiling_ms) {
                let _ = write!(
                    output,
                    " · ceiling_kind={} · ceiling_ms={milliseconds}",
                    kind.as_str()
                );
            }
        }
        StatusEvent::CaseStarted {
            ordinal,
            total,
            request_ceiling_ms,
            response_ceiling_ms,
        } => {
            let _ = write!(
                output,
                " · ordinal={ordinal} · total={total} · request_ceiling_ms={request_ceiling_ms} · response_ceiling_ms={response_ceiling_ms}"
            );
        }
        StatusEvent::Error { error_kind } => {
            let _ = write!(output, " · error_kind={}", error_kind.as_str());
        }
        StatusEvent::Completed {
            exit_code,
            exit_meaning,
        } => {
            let _ = write!(
                output,
                " · exit_code={exit_code} · exit_meaning={exit_meaning}"
            );
        }
    }
    output
}

const fn event_name(event: StatusEvent) -> &'static str {
    match event {
        StatusEvent::InvocationAccepted => "invocation_accepted",
        StatusEvent::PhaseStarted { .. } => "phase_started",
        StatusEvent::CaseStarted { .. } => "case_started",
        StatusEvent::Error { .. } => "error",
        StatusEvent::Completed { .. } => "completed",
    }
}

pub(crate) const fn exit_meaning(exit_code: u8) -> &'static str {
    match exit_code {
        0 => "success",
        1 => "unsuccessful_result",
        2 => "invalid_invocation_or_input",
        3 => "incomplete_evidence",
        _ => "internal_or_output_failure",
    }
}

#[cfg(test)]
pub(crate) struct NoStatus;

#[cfg(test)]
impl StatusObserver for NoStatus {
    fn phase_started(&mut self, _phase: StatusPhase, _ceiling: Option<StatusCeiling>) {}

    fn case_started(
        &mut self,
        _ordinal: u64,
        _total: u64,
        _request_ceiling_ms: u64,
        _response_ceiling_ms: u64,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    use serde_json::Value;

    use super::{
        MAXIMUM_EVENT_BYTES, MAXIMUM_EVENTS, MAXIMUM_OUTPUT_BYTES, StatusCeiling,
        StatusCeilingKind, StatusCommand, StatusContext, StatusErrorKind, StatusFormat,
        StatusObserver, StatusPhase, StatusReporter, StatusTransport,
    };

    fn context() -> StatusContext {
        StatusContext::new(StatusCommand::Check, StatusTransport::Stdio, "default")
    }

    #[derive(Default)]
    struct CountingWriter {
        bytes: Vec<u8>,
        writes: usize,
        flushes: usize,
    }

    impl Write for CountingWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.writes += 1;
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }

    #[test]
    fn jsonl_records_are_independently_valid_bounded_and_value_free() {
        let mut reporter = StatusReporter::new(Vec::new(), Some(StatusFormat::Jsonl), context());
        reporter.invocation_accepted();
        reporter.phase_started(
            StatusPhase::Discovery,
            Some(StatusCeiling {
                kind: StatusCeilingKind::Discovery,
                milliseconds: 10_000,
            }),
        );
        reporter.case_started(1, 2, 30_000, 30_000);
        reporter.error(
            StatusErrorKind::InvalidInvocationOrInput,
            &"synthetic-private-error-never-render",
        );
        assert_eq!(reporter.complete(2), 2);
        let output = String::from_utf8(reporter.into_inner()).unwrap();
        let records = output
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(records.len(), 5);
        assert_eq!(records[0]["event"], "invocation_accepted");
        assert_eq!(records[1]["phase"], "discovery");
        assert_eq!(records[2]["ordinal"], 1);
        assert_eq!(records[3]["error_kind"], "invalid_invocation_or_input");
        assert_eq!(records[4]["exit_meaning"], "invalid_invocation_or_input");
        assert!(!output.contains("synthetic-private-error-never-render"));
        assert!(output.lines().all(|line| line.len() < MAXIMUM_EVENT_BYTES));
        assert!(output.len() <= MAXIMUM_OUTPUT_BYTES);
    }

    #[test]
    fn plain_records_are_noninteractive_and_keep_safe_error_context() {
        let mut reporter = StatusReporter::new(Vec::new(), Some(StatusFormat::Plain), context());
        reporter.invocation_accepted();
        reporter.error(
            StatusErrorKind::InvalidInvocationOrInput,
            &"synthetic safe invocation error",
        );
        assert_eq!(reporter.complete(2), 2);
        let output = String::from_utf8(reporter.into_inner()).unwrap();

        assert!(output.contains("mcp-doctor status · invocation_accepted"));
        assert!(output.contains("error_kind=invalid_invocation_or_input"));
        assert!(output.contains("error: synthetic safe invocation error"));
        assert!(!output.contains('\r'));
        assert!(!output.contains("\u{1b}["));
    }

    #[test]
    fn disabled_status_writes_only_existing_error_text() {
        let mut reporter = StatusReporter::new(Vec::new(), None, context());
        reporter.invocation_accepted();
        reporter.phase_started(StatusPhase::TargetPreparation, None);
        reporter.error(
            StatusErrorKind::InvalidInvocationOrInput,
            &"synthetic safe invocation error",
        );
        assert_eq!(reporter.complete(2), 2);
        assert_eq!(
            reporter.into_inner(),
            b"error: synthetic safe invocation error\n"
        );
    }

    #[test]
    fn output_limits_fail_closed_without_additional_writes() {
        let mut reporter = StatusReporter::new(Vec::new(), Some(StatusFormat::Jsonl), context());
        for _ in 0..=MAXIMUM_EVENTS {
            reporter.invocation_accepted();
        }
        assert!(reporter.failed());
        assert_eq!(reporter.complete(0), 4);
        assert!(reporter.into_inner().len() <= MAXIMUM_OUTPUT_BYTES);
    }

    #[test]
    fn every_complete_record_is_flushed_before_emit_returns() {
        let mut reporter = StatusReporter::new(
            CountingWriter::default(),
            Some(StatusFormat::Jsonl),
            context(),
        );
        reporter.invocation_accepted();
        reporter.phase_started(StatusPhase::TargetPreparation, None);
        assert_eq!(reporter.complete(0), 0);

        let writer = reporter.into_inner();
        assert_eq!(writer.writes, 3);
        assert_eq!(writer.flushes, 3);
        assert_eq!(
            writer.bytes.iter().filter(|byte| **byte == b'\n').count(),
            3
        );
    }
}
