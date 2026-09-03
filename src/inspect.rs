use std::error::Error;
use std::ffi::OsString;
use std::fmt;

use crate::contract::{
    AutoDiscoveryOutcome, Diagnostic, DiagnosticLimitProfile, KnownRevision,
    PassiveCatalogConversation, PassiveProtocolSelection, ProtocolRevision,
    ProtocolSelectionEvidence, ProtocolSelectionMode, ProtocolSelectionPath,
    SnapshotDestinationError, capture_contract_snapshot, diagnostic_http_limit_profile,
    diagnostic_stdio_limit_profile, http_diagnostic, http_diagnostic_with_cleanup,
    render_catalog_diagnostic, render_http_catalog_diagnostic, render_http_diagnostic_for_revision,
    render_http_diagnostic_for_revision_with_negotiated, render_stdio_diagnostic_for_revision,
    stdio_diagnostic,
};
use crate::interruption::{Interruptible, Interruption};
use crate::status::{StatusCeiling, StatusCeilingKind, StatusObserver, StatusPhase};
use crate::transport::http::{
    HttpBudget, HttpFailure, HttpLimits, HttpRun, HttpTarget, HttpTransport, RemoteOptions,
    ResponseFailure, SystemResolver,
};
use crate::transport::stdio::{
    StdioBudget, StdioFailure, StdioLimit, StdioLimits, StdioRun, StdioTarget, StdioTransport,
    TargetError,
};

pub(crate) struct InspectOutput {
    pub(crate) diagnostic: Diagnostic,
    pub(crate) snapshot: Option<Vec<u8>>,
}

#[derive(Debug)]
pub(crate) enum InspectError {
    Target(TargetError),
    Snapshot(SnapshotDestinationError),
}

impl fmt::Display for InspectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Target(error) => error.fmt(formatter),
            Self::Snapshot(error) => error.fmt(formatter),
        }
    }
}

impl Error for InspectError {}

impl From<TargetError> for InspectError {
    fn from(error: TargetError) -> Self {
        Self::Target(error)
    }
}

impl From<SnapshotDestinationError> for InspectError {
    fn from(error: SnapshotDestinationError) -> Self {
        Self::Snapshot(error)
    }
}

pub(crate) async fn run_stdio(
    target: Vec<OsString>,
    selection: PassiveProtocolSelection,
    capture_snapshot: bool,
    limit_profile: DiagnosticLimitProfile,
    interruption: &mut Interruption,
    status: &mut dyn StatusObserver,
) -> Result<Interruptible<InspectOutput>, InspectError> {
    status.phase_started(StatusPhase::TargetPreparation, None);
    let (executable, arguments) = target
        .split_first()
        .expect("clap requires an inspect target");
    let target = StdioTarget::new(executable.clone(), arguments.to_vec())?;
    let profile = diagnostic_stdio_limit_profile(limit_profile);
    let limits = StdioLimits {
        startup_ms: profile.startup_ms,
        discovery_ms: profile.discovery_ms,
        request_ms: profile.request_ms,
        response_ms: profile.response_ms,
        shutdown_grace_ms: profile.shutdown_grace_ms,
        total_ms: profile.total_ms,
        message_bytes: profile.message_bytes,
        stdout_bytes: profile.stdout_bytes,
        stderr_bytes: profile.stderr_bytes,
        aggregate_output_bytes: profile.aggregate_output_bytes,
        message_count: profile.message_count,
    };
    match selection {
        PassiveProtocolSelection::Exact(revision) => {
            run_stdio_exact(
                &target,
                limits,
                revision,
                capture_snapshot,
                interruption,
                status,
            )
            .await
        }
        PassiveProtocolSelection::Auto => {
            run_stdio_auto(&target, limits, capture_snapshot, interruption, status).await
        }
    }
}

async fn run_stdio_exact(
    target: &StdioTarget,
    limits: StdioLimits,
    revision: ProtocolRevision,
    capture_snapshot: bool,
    interruption: &mut Interruption,
    status: &mut dyn StatusObserver,
) -> Result<Interruptible<InspectOutput>, InspectError> {
    let transport = StdioTransport::new(limits);
    let mut conversation = PassiveCatalogConversation::for_revision(revision);
    let result = transport
        .probe_with_status(target, &mut conversation, interruption, status)
        .await;

    if result.interrupted() {
        return Ok(Interruptible::Interrupted {
            cleanup_failed: result.cleanup_failed(),
        });
    }

    let selection = ProtocolSelectionEvidence::new(
        ProtocolSelectionMode::Exact,
        ProtocolSelectionPath::ExactPin,
        Some(revision),
        u64::from(result.process_started()),
        result.lifecycle_request_count(),
        result.lifecycle_notification_count(),
        0,
    );
    finish_stdio(
        &result,
        &conversation,
        revision,
        capture_snapshot,
        selection,
    )
    .map(Interruptible::completed)
}

async fn run_stdio_auto(
    target: &StdioTarget,
    limits: StdioLimits,
    capture_snapshot: bool,
    interruption: &mut Interruption,
    status: &mut dyn StatusObserver,
) -> Result<Interruptible<InspectOutput>, InspectError> {
    let mut budget = StdioBudget::new(limits);
    let mut modern = PassiveCatalogConversation::for_auto_modern();
    let modern_run = StdioTransport::new(limits)
        .probe_with_budget_and_status(target, &mut modern, &mut budget, interruption, status)
        .await;

    if modern_run.interrupted() {
        return Ok(Interruptible::Interrupted {
            cleanup_failed: modern_run.cleanup_failed(),
        });
    }

    let modern_launches = u64::from(modern_run.process_started());
    let modern_requests = modern_run.lifecycle_request_count();
    if modern_run.cleanup_failed()
        || internal_test_cleanup_failure()
        || modern_run
            .failure()
            .is_some_and(|failure| !stdio_legacy_signal(failure))
        || (modern_run.failure().is_none()
            && modern.auto_discovery_outcome() != AutoDiscoveryOutcome::LegacySignal)
    {
        let selection = ProtocolSelectionEvidence::new(
            ProtocolSelectionMode::Auto,
            ProtocolSelectionPath::ModernDiscovery,
            modern.auto_selected_revision(),
            modern_launches,
            modern_requests,
            0,
            0,
        );
        return finish_stdio(
            &modern_run,
            &modern,
            ProtocolRevision::CURRENT,
            capture_snapshot,
            selection,
        )
        .map(Interruptible::completed);
    }

    #[cfg(feature = "internal-test-fixtures")]
    if internal_test_exhaust_auto_total_budget() {
        budget.exhaust_total_for_test();
    }
    let mut legacy = PassiveCatalogConversation::for_auto_legacy();
    let legacy_run = StdioTransport::new(limits)
        .probe_with_budget_and_status(target, &mut legacy, &mut budget, interruption, status)
        .await;
    if legacy_run.interrupted() {
        return Ok(Interruptible::Interrupted {
            cleanup_failed: legacy_run.cleanup_failed(),
        });
    }
    let selected = legacy
        .negotiated_revision()
        .and_then(KnownRevision::supported)
        .filter(|revision| revision.uses_initialize());
    let selection = ProtocolSelectionEvidence::new(
        ProtocolSelectionMode::Auto,
        ProtocolSelectionPath::StdioLegacyInitialization,
        selected,
        modern_launches.saturating_add(u64::from(legacy_run.process_started())),
        modern_requests.saturating_add(legacy_run.lifecycle_request_count()),
        legacy_run.lifecycle_notification_count(),
        1,
    );
    finish_stdio(
        &legacy_run,
        &legacy,
        legacy.revision(),
        capture_snapshot,
        selection,
    )
    .map(Interruptible::completed)
}

const fn stdio_legacy_signal(failure: StdioFailure) -> bool {
    matches!(
        failure,
        StdioFailure::EarlyExit
            | StdioFailure::Limit {
                kind: StdioLimit::DiscoveryTime,
                ..
            }
    )
}

fn finish_stdio(
    result: &StdioRun,
    conversation: &PassiveCatalogConversation,
    revision: ProtocolRevision,
    capture_snapshot: bool,
    selection: ProtocolSelectionEvidence,
) -> Result<InspectOutput, InspectError> {
    debug_assert!(result.failure().is_some() || result.response().is_some());
    let cleanup_failed = result.cleanup_failed() || internal_test_cleanup_failure();
    let diagnostic = stdio_diagnostic(result.failure(), cleanup_failed);
    if result.failure().is_some() {
        Ok(InspectOutput {
            diagnostic: render_stdio_diagnostic_for_revision(diagnostic, revision)
                .with_protocol_selection(selection),
            snapshot: None,
        })
    } else {
        let diagnostic = render_catalog_diagnostic(diagnostic, conversation, result.responses());
        let snapshot = capture_if_complete(
            capture_snapshot,
            !cleanup_failed,
            revision,
            conversation.negotiated_revision(),
            result.responses(),
        )?;
        Ok(InspectOutput {
            diagnostic: diagnostic.with_protocol_selection(selection),
            snapshot,
        })
    }
}

pub(crate) async fn run_http(
    options: RemoteOptions,
    selection: PassiveProtocolSelection,
    capture_snapshot: bool,
    limit_profile: DiagnosticLimitProfile,
    status: &mut dyn StatusObserver,
) -> Result<InspectOutput, SnapshotDestinationError> {
    let profile = diagnostic_http_limit_profile(limit_profile);
    status.phase_started(
        StatusPhase::TargetPreparation,
        Some(StatusCeiling {
            kind: StatusCeilingKind::Startup,
            milliseconds: profile.startup_ms,
        }),
    );
    let limits = HttpLimits {
        startup_ms: profile.startup_ms,
        discovery_ms: profile.discovery_ms,
        request_ms: profile.request_ms,
        response_ms: profile.response_ms,
        shutdown_grace_ms: profile.shutdown_grace_ms,
        total_ms: profile.total_ms,
        endpoint_bytes: profile.endpoint_bytes,
        resolution_addresses: profile.resolution_addresses,
        trust_bytes: profile.trust_bytes,
        trust_certificates: profile.trust_certificates,
        request_fields: profile.request_fields,
        request_field_name_bytes: profile.request_field_name_bytes,
        request_field_value_bytes: profile.request_field_value_bytes,
        request_fields_bytes: profile.request_fields_bytes,
        response_fields: profile.response_fields,
        response_field_name_bytes: profile.response_field_name_bytes,
        response_field_value_bytes: profile.response_field_value_bytes,
        response_fields_bytes: profile.response_fields_bytes,
        message_bytes: profile.message_bytes,
        aggregate_output_bytes: profile.aggregate_output_bytes,
        message_count: profile.message_count,
        protocol_revisions: profile.protocol_revisions,
    };
    let target = match HttpTarget::prepare(options, limits, &SystemResolver).await {
        Ok(target) => target,
        Err(failure) => {
            let (mode, path, revision, selected) = match selection {
                PassiveProtocolSelection::Auto => (
                    ProtocolSelectionMode::Auto,
                    ProtocolSelectionPath::ModernDiscovery,
                    ProtocolRevision::CURRENT,
                    None,
                ),
                PassiveProtocolSelection::Exact(revision) => (
                    ProtocolSelectionMode::Exact,
                    ProtocolSelectionPath::ExactPin,
                    revision,
                    Some(revision),
                ),
            };
            return Ok(InspectOutput {
                diagnostic: render_http_diagnostic_for_revision(
                    http_diagnostic(Some(failure), None),
                    revision,
                )
                .with_protocol_selection(ProtocolSelectionEvidence::new(
                    mode, path, selected, 0, 0, 0, 0,
                )),
                snapshot: None,
            });
        }
    };

    match selection {
        PassiveProtocolSelection::Exact(revision) => {
            run_http_exact(target, revision, capture_snapshot, status).await
        }
        PassiveProtocolSelection::Auto => run_http_auto(target, capture_snapshot, status).await,
    }
}

async fn run_http_exact(
    target: HttpTarget,
    revision: ProtocolRevision,
    capture_snapshot: bool,
    status: &mut dyn StatusObserver,
) -> Result<InspectOutput, SnapshotDestinationError> {
    let transport = match HttpTransport::new_for_protocol(
        target,
        revision.as_str(),
        revision.uses_initialize(),
    ) {
        Ok(transport) => transport,
        Err(failure) => {
            return Ok(InspectOutput {
                diagnostic: render_http_diagnostic_for_revision(
                    http_diagnostic(Some(failure), Some(true)),
                    revision,
                )
                .with_protocol_selection(ProtocolSelectionEvidence::new(
                    ProtocolSelectionMode::Exact,
                    ProtocolSelectionPath::ExactPin,
                    Some(revision),
                    0,
                    0,
                    0,
                    0,
                )),
                snapshot: None,
            });
        }
    };
    let mut conversation = PassiveCatalogConversation::new_http_for_revision(revision);
    let result = transport.probe_with_status(&mut conversation, status).await;
    let selection = ProtocolSelectionEvidence::new(
        ProtocolSelectionMode::Exact,
        ProtocolSelectionPath::ExactPin,
        Some(revision),
        0,
        result.lifecycle_request_count(),
        result.lifecycle_notification_count(),
        0,
    );
    finish_http(
        &result,
        &conversation,
        revision,
        capture_snapshot,
        selection,
    )
}

async fn run_http_auto(
    target: HttpTarget,
    capture_snapshot: bool,
    status: &mut dyn StatusObserver,
) -> Result<InspectOutput, SnapshotDestinationError> {
    let legacy_target = target.same_authority();
    let transport = match HttpTransport::new_for_auto_probe(target) {
        Ok(transport) => transport,
        Err(failure) => {
            return Ok(InspectOutput {
                diagnostic: render_http_diagnostic_for_revision(
                    http_diagnostic(Some(failure), Some(true)),
                    ProtocolRevision::CURRENT,
                )
                .with_protocol_selection(ProtocolSelectionEvidence::new(
                    ProtocolSelectionMode::Auto,
                    ProtocolSelectionPath::ModernDiscovery,
                    None,
                    0,
                    0,
                    0,
                    0,
                )),
                snapshot: None,
            });
        }
    };
    let mut budget = HttpBudget::default();
    let mut modern = PassiveCatalogConversation::new_http_for_auto_modern();
    let modern_run = transport
        .probe_with_budget_and_status(&mut modern, &mut budget, status)
        .await;
    let legacy_signal = matches!(
        modern_run.failure(),
        Some(HttpFailure::Response(ResponseFailure::LegacyEra))
    ) && !modern_run.session_cleanup_failed();
    if !legacy_signal {
        let selection = ProtocolSelectionEvidence::new(
            ProtocolSelectionMode::Auto,
            ProtocolSelectionPath::ModernDiscovery,
            modern.auto_selected_revision(),
            0,
            modern_run.lifecycle_request_count(),
            modern_run.lifecycle_notification_count(),
            0,
        );
        return finish_http(
            &modern_run,
            &modern,
            ProtocolRevision::CURRENT,
            capture_snapshot,
            selection,
        );
    }

    let transport = match HttpTransport::new_for_protocol(
        legacy_target,
        ProtocolRevision::V2025_11_25.as_str(),
        true,
    ) {
        Ok(transport) => transport,
        Err(failure) => {
            return Ok(InspectOutput {
                diagnostic: render_http_diagnostic_for_revision(
                    http_diagnostic(Some(failure), Some(true)),
                    ProtocolRevision::V2025_11_25,
                )
                .with_protocol_selection(ProtocolSelectionEvidence::new(
                    ProtocolSelectionMode::Auto,
                    ProtocolSelectionPath::HttpLegacyInitialization,
                    None,
                    0,
                    modern_run.lifecycle_request_count(),
                    modern_run.lifecycle_notification_count(),
                    1,
                )),
                snapshot: None,
            });
        }
    };
    #[cfg(feature = "internal-test-fixtures")]
    if internal_test_exhaust_auto_total_budget() {
        budget.exhaust_total_for_test();
    }
    let mut legacy = PassiveCatalogConversation::new_http_for_auto_legacy();
    let legacy_run = transport
        .probe_with_budget_and_status(&mut legacy, &mut budget, status)
        .await;
    let selected = legacy
        .negotiated_revision()
        .and_then(KnownRevision::supported)
        .filter(|revision| revision.uses_initialize());
    let selection = ProtocolSelectionEvidence::new(
        ProtocolSelectionMode::Auto,
        ProtocolSelectionPath::HttpLegacyInitialization,
        selected,
        0,
        modern_run
            .lifecycle_request_count()
            .saturating_add(legacy_run.lifecycle_request_count()),
        modern_run
            .lifecycle_notification_count()
            .saturating_add(legacy_run.lifecycle_notification_count()),
        1,
    );
    finish_http(
        &legacy_run,
        &legacy,
        legacy.revision(),
        capture_snapshot,
        selection,
    )
}

fn finish_http(
    result: &HttpRun,
    conversation: &PassiveCatalogConversation,
    revision: ProtocolRevision,
    capture_snapshot: bool,
    selection: ProtocolSelectionEvidence,
) -> Result<InspectOutput, SnapshotDestinationError> {
    let cleanup_failed = result.session_cleanup_failed();
    let diagnostic = http_diagnostic_with_cleanup(
        result.failure(),
        Some(result.tls_applicable()),
        cleanup_failed,
    );
    if result.failure().is_some() {
        Ok(InspectOutput {
            diagnostic: render_http_diagnostic_for_revision_with_negotiated(
                diagnostic,
                revision,
                conversation.negotiated_revision(),
            )
            .with_protocol_selection(selection),
            snapshot: None,
        })
    } else {
        let diagnostic =
            render_http_catalog_diagnostic(diagnostic, conversation, result.responses());
        let snapshot = capture_if_complete(
            capture_snapshot,
            !cleanup_failed,
            revision,
            conversation.negotiated_revision(),
            result.responses(),
        )?;
        Ok(InspectOutput {
            diagnostic: diagnostic.with_protocol_selection(selection),
            snapshot,
        })
    }
}

fn capture_if_complete(
    requested: bool,
    cleanup_succeeded: bool,
    revision: ProtocolRevision,
    negotiated_revision: Option<KnownRevision>,
    responses: &[crate::transport::ProbeResponse],
) -> Result<Option<Vec<u8>>, SnapshotDestinationError> {
    if requested && cleanup_succeeded {
        capture_contract_snapshot(revision, negotiated_revision, responses).map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(feature = "internal-test-fixtures")]
fn internal_test_cleanup_failure() -> bool {
    std::env::var_os("MCP_DOCTOR_TEST_MODE").as_deref() == Some(std::ffi::OsStr::new("1"))
        && std::env::var_os("MCP_DOCTOR_INTERNAL_TEST_CLEANUP_FAILURE").as_deref()
            == Some(std::ffi::OsStr::new("1"))
}

#[cfg(feature = "internal-test-fixtures")]
fn internal_test_exhaust_auto_total_budget() -> bool {
    std::env::var_os("MCP_DOCTOR_TEST_MODE").as_deref() == Some(std::ffi::OsStr::new("1"))
        && std::env::var_os("MCP_DOCTOR_INTERNAL_TEST_EXHAUST_AUTO_TOTAL_BUDGET").as_deref()
            == Some(std::ffi::OsStr::new("1"))
}

#[cfg(not(feature = "internal-test-fixtures"))]
const fn internal_test_cleanup_failure() -> bool {
    false
}
