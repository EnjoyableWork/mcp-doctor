use std::fmt;
#[cfg(not(unix))]
use std::future::pending;

pub(crate) const GRACE_MS: u64 = 2_000;
pub(crate) const REAP_MS: u64 = 2_000;
pub(crate) const CLEANUP_MS: u64 = GRACE_MS + REAP_MS;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum Interruptible<T> {
    Completed(T),
    Interrupted { cleanup_failed: bool },
}

impl<T> Interruptible<T> {
    pub(crate) const fn completed(value: T) -> Self {
        Self::Completed(value)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct RegistrationError;

impl fmt::Display for RegistrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Unix interruption handling could not be initialized")
    }
}

#[cfg(unix)]
pub(crate) struct Interruption {
    interrupt: tokio::signal::unix::Signal,
    terminate: tokio::signal::unix::Signal,
    observed: bool,
}

#[cfg(unix)]
impl Interruption {
    pub(crate) fn register() -> Result<Self, RegistrationError> {
        use tokio::signal::unix::{SignalKind, signal};

        if internal_test_registration_failure() {
            return Err(RegistrationError);
        }
        let interrupt = signal(SignalKind::interrupt()).map_err(|_| RegistrationError)?;
        let terminate = signal(SignalKind::terminate()).map_err(|_| RegistrationError)?;
        Ok(Self {
            interrupt,
            terminate,
            observed: false,
        })
    }

    pub(crate) async fn wait(&mut self) {
        if self.observed {
            return;
        }
        tokio::select! {
            biased;
            _ = self.interrupt.recv() => {}
            _ = self.terminate.recv() => {}
        }
        self.observed = true;
    }

    pub(crate) async fn checkpoint(&mut self) -> bool {
        if self.observed {
            return true;
        }
        tokio::select! {
            biased;
            _ = self.wait() => true,
            _ = std::future::ready(()) => false,
        }
    }
}

#[cfg(not(unix))]
pub(crate) struct Interruption;

#[cfg(not(unix))]
impl Interruption {
    pub(crate) const fn register() -> Result<Self, RegistrationError> {
        Ok(Self)
    }

    pub(crate) async fn wait(&mut self) {
        pending::<()>().await;
    }

    pub(crate) async fn checkpoint(&mut self) -> bool {
        false
    }
}

#[cfg(all(unix, feature = "internal-test-fixtures"))]
fn internal_test_registration_failure() -> bool {
    std::env::var_os("MCP_DOCTOR_TEST_MODE").as_deref() == Some(std::ffi::OsStr::new("1"))
        && std::env::var_os("MCP_DOCTOR_INTERNAL_TEST_SIGNAL_REGISTRATION_FAILURE").as_deref()
            == Some(std::ffi::OsStr::new("1"))
}

#[cfg(all(unix, not(feature = "internal-test-fixtures")))]
const fn internal_test_registration_failure() -> bool {
    false
}
