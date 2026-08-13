use std::ffi::{OsStr, OsString};
use std::fmt;
use std::future::pending;
use std::io;
#[cfg(windows)]
use std::path::Path;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

#[cfg(windows)]
use process_wrap::tokio::JobObject;
#[cfg(unix)]
use process_wrap::tokio::ProcessGroup;
use process_wrap::tokio::{ChildWrapper, CommandWrap, KillOnDrop};
use serde_json::Value;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout, Command};
use tokio::time::Instant;

use super::{Conversation, ProbeRequest, ProbeResponse};

const READ_BUFFER_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct StdioLimits {
    pub(crate) startup_ms: u64,
    pub(crate) discovery_ms: u64,
    pub(crate) request_ms: u64,
    pub(crate) response_ms: u64,
    pub(crate) shutdown_grace_ms: u64,
    pub(crate) total_ms: u64,
    pub(crate) message_bytes: u64,
    pub(crate) stdout_bytes: u64,
    pub(crate) stderr_bytes: u64,
    pub(crate) aggregate_output_bytes: u64,
    pub(crate) message_count: u64,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct StdioTarget {
    executable: OsString,
    arguments: Vec<OsString>,
    environment: Vec<(OsString, OsString)>,
}

impl StdioTarget {
    pub(crate) fn new(executable: OsString, arguments: Vec<OsString>) -> Result<Self, TargetError> {
        Self::with_environment(executable, arguments, Vec::new())
    }

    pub(crate) fn with_environment(
        executable: OsString,
        arguments: Vec<OsString>,
        environment: Vec<(OsString, OsString)>,
    ) -> Result<Self, TargetError> {
        if executable.is_empty() {
            return Err(TargetError::EmptyExecutable);
        }

        #[cfg(windows)]
        if is_windows_batch_file(&executable) {
            return Err(TargetError::WindowsBatchFile);
        }

        Ok(Self {
            executable,
            arguments,
            environment,
        })
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command.args(&self.arguments);
        command
    }

    #[cfg(test)]
    fn executable(&self) -> &OsStr {
        &self.executable
    }

    #[cfg(test)]
    fn arguments(&self) -> &[OsString] {
        &self.arguments
    }
}

impl fmt::Debug for StdioTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StdioTarget")
            .field("executable", &"[REDACTED]")
            .field("argument_count", &self.arguments.len())
            .field("environment_count", &self.environment.len())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TargetError {
    EmptyExecutable,
    #[cfg(windows)]
    WindowsBatchFile,
}

impl fmt::Display for TargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyExecutable => formatter.write_str("the server executable cannot be empty"),
            #[cfg(windows)]
            Self::WindowsBatchFile => formatter.write_str(
                "Windows batch targets are not supported; use a native executable directly",
            ),
        }
    }
}

#[cfg(windows)]
fn is_windows_batch_file(executable: &OsStr) -> bool {
    Path::new(executable)
        .file_name()
        .map(OsStr::to_string_lossy)
        .is_some_and(|file_name| {
            let normalized = file_name
                .split(':')
                .next()
                .unwrap_or_default()
                .trim_end_matches([' ', '.']);
            normalized.rsplit_once('.').is_some_and(|(_, extension)| {
                extension.eq_ignore_ascii_case("bat") || extension.eq_ignore_ascii_case("cmd")
            })
        })
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum StdioLimit {
    StartupTime,
    DiscoveryTime,
    RequestTime,
    ResponseTime,
    TotalTime,
    MessageBytes,
    StdoutBytes,
    StderrBytes,
    AggregateOutputBytes,
    MessageCount,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum StdioStream {
    Process,
    Stdin,
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum StdioFailure {
    ProcessStart,
    Io {
        stream: StdioStream,
    },
    InvalidMessage {
        byte_count: usize,
        index: usize,
    },
    EarlyExit,
    Limit {
        kind: StdioLimit,
        observed: u64,
        maximum: u64,
    },
}

impl StdioFailure {
    fn limit(kind: StdioLimit, observed: u64, maximum: u64) -> Self {
        debug_assert!(observed > maximum);
        Self::Limit {
            kind,
            observed,
            maximum,
        }
    }

    fn timeout(deadline: StageDeadline) -> Self {
        Self::limit(
            deadline.kind,
            deadline.maximum.saturating_add(1),
            deadline.maximum,
        )
    }
}

#[derive(Debug)]
pub(crate) struct StdioRun {
    responses: Vec<ProbeResponse>,
    failure: Option<StdioFailure>,
    cleanup_failed: bool,
}

impl StdioRun {
    pub(crate) fn response(&self) -> Option<&ProbeResponse> {
        self.responses.first()
    }

    pub(crate) fn responses(&self) -> &[ProbeResponse] {
        &self.responses
    }

    pub(crate) const fn failure(&self) -> Option<StdioFailure> {
        self.failure
    }

    pub(crate) const fn cleanup_failed(&self) -> bool {
        self.cleanup_failed
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StdioTransport {
    limits: StdioLimits,
}

impl StdioTransport {
    pub(crate) const fn new(limits: StdioLimits) -> Self {
        Self { limits }
    }

    pub(crate) async fn probe<C>(self, target: &StdioTarget, conversation: &mut C) -> StdioRun
    where
        C: Conversation,
    {
        let run_started = Instant::now();
        let total_deadline =
            StageDeadline::after(run_started, self.limits.total_ms, StdioLimit::TotalTime);
        let startup_deadline = StageDeadline::earliest([
            total_deadline,
            StageDeadline::after(run_started, self.limits.startup_ms, StdioLimit::StartupTime),
        ]);

        let spawn_result = ManagedProcess::spawn(target, self.limits);
        let mut process = match spawn_result {
            Ok(process) => process,
            Err(failure) => {
                return StdioRun {
                    responses: Vec::new(),
                    failure: Some(failure),
                    cleanup_failed: false,
                };
            }
        };

        let operation = async {
            if Instant::now() > startup_deadline.at {
                return Err(StdioFailure::timeout(startup_deadline));
            }

            let mut responses = Vec::new();
            loop {
                if Instant::now() > total_deadline.at {
                    return Err(StdioFailure::timeout(total_deadline));
                }
                let request = conversation.next_request(responses.last());
                if Instant::now() > total_deadline.at {
                    return Err(StdioFailure::timeout(total_deadline));
                }
                let Some(request) = request else {
                    break;
                };
                let discovery = responses.is_empty();
                if !request.expects_response() {
                    process
                        .send_notification(&request, total_deadline, discovery)
                        .await?;
                    continue;
                }
                let response = process
                    .exchange(&request, total_deadline, discovery)
                    .await?;
                responses.push(response);
            }
            Ok(responses)
        }
        .await;

        let shutdown = process.shutdown(total_deadline).await;
        let (responses, mut failure) = match operation {
            Ok(responses) => (responses, None),
            Err(failure) => (Vec::new(), Some(failure)),
        };
        if failure.is_none() {
            failure = shutdown.background_failure;
        }

        StdioRun {
            responses,
            failure,
            cleanup_failed: shutdown.cleanup_failed,
        }
    }
}

struct ManagedProcess {
    child: Box<dyn ChildWrapper>,
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
    decoder: FrameDecoder,
    protocol: ProtocolTracker,
    output: OutputBudget,
    limits: StdioLimits,
    cleaned: bool,
}

impl ManagedProcess {
    fn spawn(target: &StdioTarget, limits: StdioLimits) -> Result<Self, StdioFailure> {
        let mut command = target.command();
        command
            .env_clear()
            .envs(constrained_environment(std::env::vars_os()))
            .envs(target.environment.iter().map(|(name, value)| (name, value)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut command = CommandWrap::from(command);
        command.wrap(KillOnDrop);
        #[cfg(unix)]
        command.wrap(ProcessGroup::leader());
        #[cfg(windows)]
        command.wrap(JobObject);

        let mut child = command.spawn().map_err(|_| StdioFailure::ProcessStart)?;
        let stdin = child.stdin().take();
        let stdout = child.stdout().take();
        let stderr = child.stderr().take();

        Ok(Self {
            child,
            stdin,
            stdout,
            stderr,
            decoder: FrameDecoder::new(limits.message_bytes, limits.message_count),
            protocol: ProtocolTracker::default(),
            output: OutputBudget::new(limits),
            limits,
            cleaned: false,
        })
    }

    async fn exchange(
        &mut self,
        request: &ProbeRequest,
        total_deadline: StageDeadline,
        discovery: bool,
    ) -> Result<ProbeResponse, StdioFailure> {
        if self.stdin.is_none() {
            return Err(StdioFailure::Io {
                stream: StdioStream::Stdin,
            });
        }
        if self.stdout.is_none() {
            return Err(StdioFailure::Io {
                stream: StdioStream::Stdout,
            });
        }
        if self.stderr.is_none() {
            return Err(StdioFailure::Io {
                stream: StdioStream::Stderr,
            });
        }

        let stage_deadline = if discovery {
            StageDeadline::earliest([
                total_deadline,
                StageDeadline::after(
                    Instant::now(),
                    self.limits.discovery_ms,
                    StdioLimit::DiscoveryTime,
                ),
            ])
        } else {
            total_deadline
        };
        let request_size = u64::try_from(request.as_bytes().len()).unwrap_or(u64::MAX);
        if request_size > self.limits.message_bytes {
            return Err(StdioFailure::limit(
                StdioLimit::MessageBytes,
                request_size,
                self.limits.message_bytes,
            ));
        }

        let request_deadline = StageDeadline::earliest([
            stage_deadline,
            StageDeadline::after(
                Instant::now(),
                self.limits.request_ms,
                StdioLimit::RequestTime,
            ),
        ]);
        self.protocol.begin(request.id());
        let stdin = self.stdin.as_mut().expect("the stdin pipe was checked");
        match tokio::time::timeout_at(request_deadline.at, async {
            stdin.write_all(request.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await
        })
        .await
        {
            Ok(Ok(())) => {}
            Ok(Err(_)) => {
                return Err(StdioFailure::Io {
                    stream: StdioStream::Stdin,
                });
            }
            Err(_) => return Err(StdioFailure::timeout(request_deadline)),
        }

        let response_deadline = StageDeadline::earliest([
            stage_deadline,
            StageDeadline::after(
                Instant::now(),
                self.limits.response_ms,
                StdioLimit::ResponseTime,
            ),
        ]);
        let mut stdout_buffer = [0_u8; READ_BUFFER_BYTES];
        let mut stderr_buffer = [0_u8; READ_BUFFER_BYTES];

        loop {
            match next_activity(
                &mut self.child,
                false,
                &mut self.stdout,
                &mut self.stderr,
                &mut stdout_buffer,
                &mut stderr_buffer,
                response_deadline.at,
            )
            .await
            {
                Activity::Stdout(Ok(0)) => {
                    self.stdout.take();
                    self.decoder.finish()?;
                }
                Activity::Stdout(Ok(read)) => {
                    self.observe_stdout(&stdout_buffer[..read])?;
                    let frames = self.decoder.push(&stdout_buffer[..read])?;
                    let mut response = None;
                    for frame in frames {
                        if let Some(observed) = self.protocol.observe(frame)? {
                            response = Some(observed);
                        }
                    }
                    if let Some(response) = response {
                        return Ok(response);
                    }
                }
                Activity::Stdout(Err(_)) => {
                    self.stdout.take();
                    return Err(StdioFailure::Io {
                        stream: StdioStream::Stdout,
                    });
                }
                Activity::Stderr(Ok(0)) => {
                    self.stderr.take();
                }
                Activity::Stderr(Ok(read)) => {
                    self.observe_stderr(read)?;
                }
                Activity::Stderr(Err(_)) => {
                    self.stderr.take();
                    return Err(StdioFailure::Io {
                        stream: StdioStream::Stderr,
                    });
                }
                Activity::Child(Ok(_)) => return Err(StdioFailure::EarlyExit),
                Activity::Child(Err(_)) => {
                    return Err(StdioFailure::Io {
                        stream: StdioStream::Process,
                    });
                }
                Activity::Deadline => return Err(StdioFailure::timeout(response_deadline)),
            }
        }
    }

    async fn send_notification(
        &mut self,
        request: &ProbeRequest,
        total_deadline: StageDeadline,
        discovery: bool,
    ) -> Result<(), StdioFailure> {
        debug_assert!(!request.expects_response());
        if self.stdin.is_none() {
            return Err(StdioFailure::Io {
                stream: StdioStream::Stdin,
            });
        }
        let stage_deadline = if discovery {
            StageDeadline::earliest([
                total_deadline,
                StageDeadline::after(
                    Instant::now(),
                    self.limits.discovery_ms,
                    StdioLimit::DiscoveryTime,
                ),
            ])
        } else {
            total_deadline
        };
        let request_size = u64::try_from(request.as_bytes().len()).unwrap_or(u64::MAX);
        if request_size > self.limits.message_bytes {
            return Err(StdioFailure::limit(
                StdioLimit::MessageBytes,
                request_size,
                self.limits.message_bytes,
            ));
        }
        let request_deadline = StageDeadline::earliest([
            stage_deadline,
            StageDeadline::after(
                Instant::now(),
                self.limits.request_ms,
                StdioLimit::RequestTime,
            ),
        ]);
        let stdin = self.stdin.as_mut().expect("the stdin pipe was checked");
        match tokio::time::timeout_at(request_deadline.at, async {
            stdin.write_all(request.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await
        })
        .await
        {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) => Err(StdioFailure::Io {
                stream: StdioStream::Stdin,
            }),
            Err(_) => Err(StdioFailure::timeout(request_deadline)),
        }
    }

    fn observe_stdout(&mut self, bytes: &[u8]) -> Result<(), StdioFailure> {
        if let Some(failure) = self.output.observe_stdout(bytes.len()) {
            self.stdout.take();
            if matches!(
                failure,
                StdioFailure::Limit {
                    kind: StdioLimit::AggregateOutputBytes,
                    ..
                }
            ) {
                self.stderr.take();
            }
            return Err(failure);
        }
        Ok(())
    }

    fn observe_stderr(&mut self, bytes: usize) -> Result<(), StdioFailure> {
        if let Some(failure) = self.output.observe_stderr(bytes) {
            self.stderr.take();
            if matches!(
                failure,
                StdioFailure::Limit {
                    kind: StdioLimit::AggregateOutputBytes,
                    ..
                }
            ) {
                self.stdout.take();
            }
            return Err(failure);
        }
        Ok(())
    }

    async fn shutdown(&mut self, total_deadline: StageDeadline) -> ShutdownResult {
        self.stdin.take();
        let grace_deadline = StageDeadline::earliest([
            total_deadline,
            StageDeadline::after(
                Instant::now(),
                self.limits.shutdown_grace_ms,
                StdioLimit::TotalTime,
            ),
        ]);
        let mut stdout_buffer = [0_u8; READ_BUFFER_BYTES];
        let mut stderr_buffer = [0_u8; READ_BUFFER_BYTES];
        let mut child_done = false;
        let mut background_failure = None;
        let mut cleanup_failed = false;

        while !child_done && Instant::now() < grace_deadline.at {
            let activity = next_activity(
                &mut self.child,
                child_done,
                &mut self.stdout,
                &mut self.stderr,
                &mut stdout_buffer,
                &mut stderr_buffer,
                grace_deadline.at,
            )
            .await;
            match self.handle_shutdown_activity(activity, &stdout_buffer, &mut background_failure) {
                ShutdownActivity::Continue => {}
                ShutdownActivity::ChildDone => child_done = true,
                ShutdownActivity::Deadline => break,
                ShutdownActivity::CleanupFailed => cleanup_failed = true,
            }
        }

        match self.child.start_kill() {
            Ok(()) => {}
            Err(_) if child_done => {}
            Err(_) => cleanup_failed = true,
        }

        while !(child_done && self.stdout.is_none() && self.stderr.is_none()) {
            if Instant::now() >= total_deadline.at {
                cleanup_failed = true;
                break;
            }

            let activity = next_activity(
                &mut self.child,
                child_done,
                &mut self.stdout,
                &mut self.stderr,
                &mut stdout_buffer,
                &mut stderr_buffer,
                total_deadline.at,
            )
            .await;
            match self.handle_shutdown_activity(activity, &stdout_buffer, &mut background_failure) {
                ShutdownActivity::Continue => {}
                ShutdownActivity::ChildDone => child_done = true,
                ShutdownActivity::Deadline => {
                    cleanup_failed = true;
                    break;
                }
                ShutdownActivity::CleanupFailed => cleanup_failed = true,
            }
        }

        self.cleaned = child_done;
        ShutdownResult {
            background_failure,
            cleanup_failed: cleanup_failed || !child_done,
        }
    }

    fn handle_shutdown_activity(
        &mut self,
        activity: Activity,
        stdout_buffer: &[u8; READ_BUFFER_BYTES],
        background_failure: &mut Option<StdioFailure>,
    ) -> ShutdownActivity {
        let mut remember = |failure| {
            if background_failure.is_none() {
                *background_failure = Some(failure);
            }
        };

        match activity {
            Activity::Stdout(Ok(0)) => {
                self.stdout.take();
                if let Err(failure) = self.decoder.finish() {
                    remember(failure);
                }
                ShutdownActivity::Continue
            }
            Activity::Stdout(Ok(read)) => {
                if let Err(failure) = self.observe_stdout(&stdout_buffer[..read]) {
                    remember(failure);
                    return ShutdownActivity::Continue;
                }
                match self.decoder.push(&stdout_buffer[..read]) {
                    Ok(frames) => {
                        for frame in frames {
                            if let Err(failure) = self.protocol.observe(frame) {
                                remember(failure);
                                break;
                            }
                        }
                    }
                    Err(failure) => remember(failure),
                }
                ShutdownActivity::Continue
            }
            Activity::Stdout(Err(_)) => {
                self.stdout.take();
                remember(StdioFailure::Io {
                    stream: StdioStream::Stdout,
                });
                ShutdownActivity::Continue
            }
            Activity::Stderr(Ok(0)) => {
                self.stderr.take();
                ShutdownActivity::Continue
            }
            Activity::Stderr(Ok(read)) => {
                if let Err(failure) = self.observe_stderr(read) {
                    remember(failure);
                }
                ShutdownActivity::Continue
            }
            Activity::Stderr(Err(_)) => {
                self.stderr.take();
                remember(StdioFailure::Io {
                    stream: StdioStream::Stderr,
                });
                ShutdownActivity::Continue
            }
            Activity::Child(Ok(_)) => ShutdownActivity::ChildDone,
            Activity::Child(Err(_)) => ShutdownActivity::CleanupFailed,
            Activity::Deadline => ShutdownActivity::Deadline,
        }
    }
}

impl Drop for ManagedProcess {
    fn drop(&mut self) {
        if !self.cleaned {
            let _ = self.child.start_kill();
        }
    }
}

#[derive(Debug)]
struct ShutdownResult {
    background_failure: Option<StdioFailure>,
    cleanup_failed: bool,
}

enum ShutdownActivity {
    Continue,
    ChildDone,
    Deadline,
    CleanupFailed,
}

enum Activity {
    Stdout(Result<usize, io::ErrorKind>),
    Stderr(Result<usize, io::ErrorKind>),
    Child(Result<ExitStatus, io::ErrorKind>),
    Deadline,
}

async fn next_activity(
    child: &mut Box<dyn ChildWrapper>,
    child_done: bool,
    stdout: &mut Option<ChildStdout>,
    stderr: &mut Option<ChildStderr>,
    stdout_buffer: &mut [u8; READ_BUFFER_BYTES],
    stderr_buffer: &mut [u8; READ_BUFFER_BYTES],
    deadline: Instant,
) -> Activity {
    if Instant::now() >= deadline {
        return Activity::Deadline;
    }

    tokio::select! {
        biased;
        result = read_or_pending(stdout, stdout_buffer) => Activity::Stdout(result),
        result = read_or_pending(stderr, stderr_buffer) => Activity::Stderr(result),
        result = wait_or_pending(child, child_done) => Activity::Child(result),
        _ = tokio::time::sleep_until(deadline) => Activity::Deadline,
    }
}

async fn read_or_pending<R>(
    stream: &mut Option<R>,
    buffer: &mut [u8],
) -> Result<usize, io::ErrorKind>
where
    R: AsyncRead + Unpin,
{
    match stream {
        Some(stream) => stream.read(buffer).await.map_err(|error| error.kind()),
        None => pending().await,
    }
}

async fn wait_or_pending(
    child: &mut Box<dyn ChildWrapper>,
    child_done: bool,
) -> Result<ExitStatus, io::ErrorKind> {
    if child_done {
        pending().await
    } else {
        child.wait().await.map_err(|error| error.kind())
    }
}

#[derive(Debug, Clone, Copy)]
struct StageDeadline {
    at: Instant,
    kind: StdioLimit,
    maximum: u64,
}

impl StageDeadline {
    fn after(start: Instant, milliseconds: u64, kind: StdioLimit) -> Self {
        Self {
            at: start + Duration::from_millis(milliseconds),
            kind,
            maximum: milliseconds,
        }
    }

    fn earliest<const N: usize>(deadlines: [Self; N]) -> Self {
        deadlines
            .into_iter()
            .min_by_key(|deadline| deadline.at)
            .expect("a stage must have at least one deadline")
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Frame {
    bytes: Vec<u8>,
    index: usize,
}

struct FrameDecoder {
    current: Vec<u8>,
    message_bytes: u64,
    message_count: u64,
    observed_messages: u64,
    next_index: usize,
}

impl FrameDecoder {
    fn new(message_bytes: u64, message_count: u64) -> Self {
        Self {
            current: Vec::new(),
            message_bytes,
            message_count,
            observed_messages: 0,
            next_index: 0,
        }
    }

    fn push(&mut self, bytes: &[u8]) -> Result<Vec<Frame>, StdioFailure> {
        let mut frames = Vec::new();
        for byte in bytes {
            if *byte == b'\n' {
                self.observed_messages = self.observed_messages.saturating_add(1);
                if self.observed_messages > self.message_count {
                    return Err(StdioFailure::limit(
                        StdioLimit::MessageCount,
                        self.observed_messages,
                        self.message_count,
                    ));
                }
                frames.push(Frame {
                    bytes: std::mem::take(&mut self.current),
                    index: self.next_index,
                });
                self.next_index = self.next_index.saturating_add(1);
                continue;
            }

            let observed = u64::try_from(self.current.len())
                .unwrap_or(u64::MAX)
                .saturating_add(1);
            if observed > self.message_bytes {
                return Err(StdioFailure::limit(
                    StdioLimit::MessageBytes,
                    observed,
                    self.message_bytes,
                ));
            }
            self.current.push(*byte);
        }
        Ok(frames)
    }

    fn finish(&mut self) -> Result<(), StdioFailure> {
        if self.current.is_empty() {
            Ok(())
        } else {
            let failure = StdioFailure::InvalidMessage {
                byte_count: self.current.len(),
                index: self.next_index,
            };
            self.current.clear();
            Err(failure)
        }
    }
}

#[derive(Default)]
struct ProtocolTracker {
    active_request: Option<i64>,
    completed_requests: Vec<i64>,
}

impl ProtocolTracker {
    fn begin(&mut self, request_id: i64) {
        assert!(
            self.active_request.is_none(),
            "a new request cannot begin while another response is pending"
        );
        assert!(
            !self.completed_requests.contains(&request_id),
            "a locally generated request id cannot be reused"
        );
        self.active_request = Some(request_id);
    }

    fn observe(&mut self, frame: Frame) -> Result<Option<ProbeResponse>, StdioFailure> {
        let invalid = || StdioFailure::InvalidMessage {
            byte_count: frame.bytes.len(),
            index: frame.index,
        };
        let value: Value = serde_json::from_slice(&frame.bytes).map_err(|_| invalid())?;
        let object = value.as_object().ok_or_else(invalid)?;
        if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            return Err(invalid());
        }

        if let Some(method) = object.get("method") {
            if !method.is_string()
                || object.contains_key("id")
                || object.contains_key("result")
                || object.contains_key("error")
            {
                return Err(invalid());
            }
            return Ok(None);
        }

        let response_id = object.get("id").and_then(Value::as_i64);
        let correct_id = response_id == self.active_request;
        let result_shape = object.contains_key("result") ^ object.contains_key("error");
        if !correct_id || !result_shape {
            return Err(invalid());
        }

        let request_id = response_id.expect("a matching active response id was checked");
        self.active_request = None;
        self.completed_requests.push(request_id);
        Ok(Some(ProbeResponse::new(request_id, frame.bytes)))
    }
}

struct OutputBudget {
    stdout: u64,
    stderr: u64,
    aggregate: u64,
    limits: StdioLimits,
}

impl OutputBudget {
    const fn new(limits: StdioLimits) -> Self {
        Self {
            stdout: 0,
            stderr: 0,
            aggregate: 0,
            limits,
        }
    }

    fn observe_stdout(&mut self, bytes: usize) -> Option<StdioFailure> {
        let bytes = u64::try_from(bytes).unwrap_or(u64::MAX);
        let previous_stdout = self.stdout;
        let previous_aggregate = self.aggregate;
        self.stdout = self.stdout.saturating_add(bytes);
        self.aggregate = self.aggregate.saturating_add(bytes);

        first_output_failure(
            previous_stdout,
            self.stdout,
            self.limits.stdout_bytes,
            StdioLimit::StdoutBytes,
            previous_aggregate,
            self.aggregate,
            self.limits.aggregate_output_bytes,
        )
    }

    fn observe_stderr(&mut self, bytes: usize) -> Option<StdioFailure> {
        let bytes = u64::try_from(bytes).unwrap_or(u64::MAX);
        let previous_stderr = self.stderr;
        let previous_aggregate = self.aggregate;
        self.stderr = self.stderr.saturating_add(bytes);
        self.aggregate = self.aggregate.saturating_add(bytes);

        first_output_failure(
            previous_stderr,
            self.stderr,
            self.limits.stderr_bytes,
            StdioLimit::StderrBytes,
            previous_aggregate,
            self.aggregate,
            self.limits.aggregate_output_bytes,
        )
    }
}

fn first_output_failure(
    previous_stream: u64,
    observed_stream: u64,
    maximum_stream: u64,
    stream_kind: StdioLimit,
    previous_aggregate: u64,
    observed_aggregate: u64,
    maximum_aggregate: u64,
) -> Option<StdioFailure> {
    let stream_crossing = (observed_stream > maximum_stream).then(|| {
        maximum_stream
            .saturating_sub(previous_stream)
            .saturating_add(1)
    });
    let aggregate_crossing = (observed_aggregate > maximum_aggregate).then(|| {
        maximum_aggregate
            .saturating_sub(previous_aggregate)
            .saturating_add(1)
    });

    match (stream_crossing, aggregate_crossing) {
        (None, None) => None,
        (Some(_), None) => Some(StdioFailure::limit(
            stream_kind,
            observed_stream,
            maximum_stream,
        )),
        (None, Some(_)) => Some(StdioFailure::limit(
            StdioLimit::AggregateOutputBytes,
            observed_aggregate,
            maximum_aggregate,
        )),
        (Some(stream_at), Some(aggregate_at)) if stream_at <= aggregate_at => Some(
            StdioFailure::limit(stream_kind, observed_stream, maximum_stream),
        ),
        (Some(_), Some(_)) => Some(StdioFailure::limit(
            StdioLimit::AggregateOutputBytes,
            observed_aggregate,
            maximum_aggregate,
        )),
    }
}

fn constrained_environment<I>(environment: I) -> Vec<(OsString, OsString)>
where
    I: IntoIterator<Item = (OsString, OsString)>,
{
    environment
        .into_iter()
        .filter(|(name, _)| allowed_environment_name(name))
        .collect()
}

fn allowed_environment_name(name: &OsStr) -> bool {
    #[cfg(windows)]
    {
        let name = name.to_string_lossy();
        ["PATH", "PATHEXT", "SystemRoot", "WINDIR"]
            .iter()
            .any(|allowed| name.eq_ignore_ascii_case(allowed))
    }

    #[cfg(not(windows))]
    {
        name == OsStr::new("PATH")
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};

    use super::{
        FrameDecoder, OutputBudget, ProtocolTracker, StdioFailure, StdioLimit, StdioLimits,
        StdioTarget, constrained_environment,
    };
    use crate::transport::ProbeRequest;

    fn small_limits() -> StdioLimits {
        StdioLimits {
            startup_ms: 100,
            discovery_ms: 100,
            request_ms: 100,
            response_ms: 100,
            shutdown_grace_ms: 50,
            total_ms: 500,
            message_bytes: 64,
            stdout_bytes: 128,
            stderr_bytes: 32,
            aggregate_output_bytes: 128,
            message_count: 2,
        }
    }

    #[test]
    fn target_debug_output_never_reveals_the_command_or_arguments() {
        let executable = OsString::from("synthetic-secret-executable-7f2c");
        let argument = OsString::from("synthetic-secret-argument-7f2c");
        let target = StdioTarget::new(executable.clone(), vec![argument.clone()])
            .expect("a synthetic native target should be accepted");

        let rendered = format!("{target:?}");
        assert!(!rendered.contains(executable.to_string_lossy().as_ref()));
        assert!(!rendered.contains(argument.to_string_lossy().as_ref()));
        assert!(rendered.contains("[REDACTED]"));
        assert_eq!(target.executable(), executable.as_os_str());
        assert_eq!(target.arguments(), [argument]);
    }

    #[test]
    fn child_environment_keeps_only_platform_launch_requirements() {
        let source = vec![
            (OsString::from("PATH"), OsString::from("synthetic-path")),
            (
                OsString::from("MCP_DOCTOR_SECRET"),
                OsString::from("synthetic-secret-never-inherit-7f2c"),
            ),
            (OsString::from("HOME"), OsString::from("synthetic-home")),
        ];
        let filtered = constrained_environment(source);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].0, OsStr::new("PATH"));
        assert_eq!(filtered[0].1, OsStr::new("synthetic-path"));
    }

    #[test]
    fn locally_generated_request_is_unframed_and_redacted() {
        let request = ProbeRequest::new(
            7,
            br#"{"jsonrpc":"2.0","id":7,"method":"server/discover","params":{}}"#.to_vec(),
        );

        assert_eq!(
            request
                .as_bytes()
                .iter()
                .filter(|byte| **byte == b'\n')
                .count(),
            0
        );
        let rendered = format!("{request:?}");
        assert!(rendered.contains("[REDACTED]"));
        assert!(!rendered.contains("jsonrpc"));
    }

    #[test]
    fn framing_stops_at_message_and_count_limits_without_unbounded_reads() {
        let mut decoder = FrameDecoder::new(4, 2);
        let frames = decoder
            .push(b"{}\n[]\n")
            .expect("two bounded frames should be accepted");
        assert_eq!(frames.len(), 2);

        assert_eq!(
            decoder.push(b"0\n"),
            Err(StdioFailure::Limit {
                kind: StdioLimit::MessageCount,
                observed: 3,
                maximum: 2,
            })
        );

        let mut oversized = FrameDecoder::new(4, 2);
        assert_eq!(
            oversized.push(b"12345"),
            Err(StdioFailure::Limit {
                kind: StdioLimit::MessageBytes,
                observed: 5,
                maximum: 4,
            })
        );
    }

    #[test]
    fn output_budget_reports_the_first_stream_or_aggregate_boundary() {
        let limits = small_limits();
        let mut output = OutputBudget::new(limits);
        assert_eq!(output.observe_stdout(100), None);
        assert_eq!(
            output.observe_stderr(29),
            Some(StdioFailure::Limit {
                kind: StdioLimit::AggregateOutputBytes,
                observed: 129,
                maximum: 128,
            })
        );

        let mut stderr = OutputBudget::new(limits);
        assert_eq!(
            stderr.observe_stderr(33),
            Some(StdioFailure::Limit {
                kind: StdioLimit::StderrBytes,
                observed: 33,
                maximum: 32,
            })
        );
    }

    #[test]
    fn response_parser_rejects_server_requests_and_duplicate_responses() {
        let mut decoder = FrameDecoder::new(256, 8);
        let mut tracker = ProtocolTracker::default();
        tracker.begin(1);
        let mut frames = decoder
            .push(b"{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n")
            .expect("a bounded response frame should decode");
        let response = tracker
            .observe(frames.remove(0))
            .expect("the first matching response should be accepted")
            .expect("the matching response should complete the probe");
        assert!(response.byte_count() > 0);

        let mut duplicate = decoder
            .push(b"{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n")
            .expect("a bounded duplicate frame should decode");
        assert!(matches!(
            tracker.observe(duplicate.remove(0)),
            Err(StdioFailure::InvalidMessage { index: 1, .. })
        ));

        let mut request_decoder = FrameDecoder::new(256, 8);
        let mut request_tracker = ProtocolTracker::default();
        request_tracker.begin(1);
        let mut request = request_decoder
            .push(b"{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"roots/list\"}\n")
            .expect("a bounded server request frame should decode");
        assert!(matches!(
            request_tracker.observe(request.remove(0)),
            Err(StdioFailure::InvalidMessage { .. })
        ));
    }

    #[cfg(windows)]
    #[test]
    fn windows_batch_targets_are_rejected_before_rust_can_invoke_a_shell() {
        use super::TargetError;

        for target in [
            "server.cmd",
            "server.BAT",
            "server.cmd ",
            "server.BAT...",
            "server.cmd:stream",
            ".cmd",
        ] {
            assert_eq!(
                StdioTarget::new(OsString::from(target), Vec::new()),
                Err(TargetError::WindowsBatchFile),
                "expected the synthetic Windows batch target {target:?} to be rejected"
            );
        }
    }
}
