use std::fmt::{self, Write as _};

use serde::Serialize;

use crate::aggregate::AGGREGATE_SCHEMA_VERSION;
use crate::contract::{
    BADGE_REPORT_VERSION, DIFF_SCHEMA_VERSION, ExitStatus, GENERATOR_VERSION, KnownRevision,
    MARKDOWN_REPORT_VERSION, ProtocolRevision, REPORT_SCHEMA_VERSION, SCENARIO_SCHEMA_VERSION,
    SNAPSHOT_SCHEMA_VERSION, WORKFLOW_SCHEMA_VERSION, diagnostic_stdio_limit_profile,
};
use crate::status::{
    MAXIMUM_EVENT_BYTES as MAXIMUM_STATUS_EVENT_BYTES, MAXIMUM_EVENTS as MAXIMUM_STATUS_EVENTS,
    MAXIMUM_OUTPUT_BYTES as MAXIMUM_STATUS_OUTPUT_BYTES, STATUS_SCHEMA_VERSION,
};

pub(crate) const CAPABILITIES_SCHEMA_VERSION: &str = "mcp-doctor.capabilities/v1";
pub(crate) const EXIT_SEMANTICS_VERSION: &str = "mcp-doctor.exit/v1";
pub(crate) const CAPABILITIES_LIMIT_PROFILE: &str = "mcp-doctor.limits/capabilities/v1";
pub(crate) const MAXIMUM_OUTPUT_BYTES: usize = 64 * 1024;

const AGGREGATE_LIMIT_PROFILE: &str = "mcp-doctor.limits/aggregate/v1";
const CONTRACT_DIFF_LIMIT_PROFILE: &str = "mcp-doctor.limits/contract-diff/v1alpha1";
const DIAGNOSTIC_LIMIT_PROFILE: &str = "mcp-doctor.limits/diagnostic/v1";
const DIAGNOSTIC_LIMIT_SELECTIONS: &[&str] = &["default", "slow-start"];
const DIAGNOSTIC_LIMIT_SELECTABLE_FOR: &[&str] = &["break", "check", "inspect"];

const HUMAN_REPORTER: &str = "human";
const JSON_REPORTER: &str = "json";
const JUNIT_REPORTER: &str = "junit";
const MARKDOWN_REPORTER: &str = "markdown";
const BADGE_REPORTER: &str = "badge";

const NO_SCHEMAS: &[&str] = &[];
const NO_REPORTERS: &[&str] = &[];
const AGGREGATE_SCHEMAS: &[&str] = &[AGGREGATE_SCHEMA_VERSION];
const CAPABILITIES_SCHEMAS: &[&str] = &[CAPABILITIES_SCHEMA_VERSION];
const CONTRACT_DIFF_SCHEMAS: &[&str] = &[DIFF_SCHEMA_VERSION];
const CONTRACT_SNAPSHOT_SCHEMAS: &[&str] = &[SNAPSHOT_SCHEMA_VERSION];
const DIAGNOSTIC_REPORT_SCHEMAS: &[&str] = &[REPORT_SCHEMA_VERSION];
const MARKDOWN_REPORT_SCHEMAS: &[&str] = &[MARKDOWN_REPORT_VERSION];
const BADGE_REPORT_SCHEMAS: &[&str] = &[BADGE_REPORT_VERSION];
const GENERATOR_SCHEMAS: &[&str] = &[GENERATOR_VERSION];
const SCENARIO_SCHEMAS: &[&str] = &[SCENARIO_SCHEMA_VERSION, WORKFLOW_SCHEMA_VERSION];
const STATUS_SCHEMAS: &[&str] = &[STATUS_SCHEMA_VERSION];
const STATUS_COMMANDS: &[&str] = &["break", "check", "inspect", "reject"];

const HUMAN_JSON_REPORTERS: &[&str] = &[HUMAN_REPORTER, JSON_REPORTER];
const DIAGNOSTIC_REPORTERS: &[&str] = &[HUMAN_REPORTER, JSON_REPORTER, JUNIT_REPORTER];
const DIAGNOSTIC_ARTIFACT_REPORTERS: &[&str] = &[
    JSON_REPORTER,
    JUNIT_REPORTER,
    MARKDOWN_REPORTER,
    BADGE_REPORTER,
];
const STATUS_REPRESENTATIONS: &[StatusRepresentation<'static>] = &[
    StatusRepresentation {
        name: "plain",
        machine_readable: false,
    },
    StatusRepresentation {
        name: "jsonl",
        machine_readable: true,
    },
];

const CURRENT_REVISION: &str = ProtocolRevision::V2026_07_28.as_str();
const V2025_11_25: &str = ProtocolRevision::V2025_11_25.as_str();
const V2025_06_18: &str = ProtocolRevision::V2025_06_18.as_str();
const V2025_03_26: &str = KnownRevision::V2025_03_26.as_str();
const V2024_11_05: &str = KnownRevision::V2024_11_05.as_str();
const ACTIVE_REVISIONS: &[&str] = &[CURRENT_REVISION, V2025_11_25, V2025_06_18];
const INSPECT_REVISIONS: &[&str] = &[CURRENT_REVISION, V2025_11_25, V2025_06_18];
const MODERN_INSPECT_REVISIONS: &[&str] = &[CURRENT_REVISION];
const PROTOCOL_SELECTION_MODES: &[&str] = &["auto", "exact"];

const COMMANDS: &[CommandCapability<'static>] = &[
    CommandCapability {
        name: "aggregate",
        activity: "offline",
        reporters: HUMAN_JSON_REPORTERS,
        artifact_reporters: NO_REPORTERS,
        input_schema_versions: DIAGNOSTIC_REPORT_SCHEMAS,
        output_schema_versions: AGGREGATE_SCHEMAS,
        generator_versions: NO_SCHEMAS,
        limit_profile: AGGREGATE_LIMIT_PROFILE,
    },
    CommandCapability {
        name: "break",
        activity: "active",
        reporters: DIAGNOSTIC_REPORTERS,
        artifact_reporters: DIAGNOSTIC_ARTIFACT_REPORTERS,
        input_schema_versions: NO_SCHEMAS,
        output_schema_versions: DIAGNOSTIC_REPORT_SCHEMAS,
        generator_versions: GENERATOR_SCHEMAS,
        limit_profile: DIAGNOSTIC_LIMIT_PROFILE,
    },
    CommandCapability {
        name: "capabilities",
        activity: "compiled_only",
        reporters: HUMAN_JSON_REPORTERS,
        artifact_reporters: NO_REPORTERS,
        input_schema_versions: NO_SCHEMAS,
        output_schema_versions: CAPABILITIES_SCHEMAS,
        generator_versions: NO_SCHEMAS,
        limit_profile: CAPABILITIES_LIMIT_PROFILE,
    },
    CommandCapability {
        name: "check",
        activity: "active",
        reporters: DIAGNOSTIC_REPORTERS,
        artifact_reporters: DIAGNOSTIC_ARTIFACT_REPORTERS,
        input_schema_versions: SCENARIO_SCHEMAS,
        output_schema_versions: DIAGNOSTIC_REPORT_SCHEMAS,
        generator_versions: NO_SCHEMAS,
        limit_profile: DIAGNOSTIC_LIMIT_PROFILE,
    },
    CommandCapability {
        name: "diff",
        activity: "offline",
        reporters: HUMAN_JSON_REPORTERS,
        artifact_reporters: NO_REPORTERS,
        input_schema_versions: CONTRACT_SNAPSHOT_SCHEMAS,
        output_schema_versions: CONTRACT_DIFF_SCHEMAS,
        generator_versions: NO_SCHEMAS,
        limit_profile: CONTRACT_DIFF_LIMIT_PROFILE,
    },
    CommandCapability {
        name: "inspect",
        activity: "passive",
        reporters: DIAGNOSTIC_REPORTERS,
        artifact_reporters: DIAGNOSTIC_ARTIFACT_REPORTERS,
        input_schema_versions: NO_SCHEMAS,
        output_schema_versions: &[REPORT_SCHEMA_VERSION, SNAPSHOT_SCHEMA_VERSION],
        generator_versions: NO_SCHEMAS,
        limit_profile: DIAGNOSTIC_LIMIT_PROFILE,
    },
    CommandCapability {
        name: "reject",
        activity: "active",
        reporters: DIAGNOSTIC_REPORTERS,
        artifact_reporters: DIAGNOSTIC_ARTIFACT_REPORTERS,
        input_schema_versions: NO_SCHEMAS,
        output_schema_versions: DIAGNOSTIC_REPORT_SCHEMAS,
        generator_versions: GENERATOR_SCHEMAS,
        limit_profile: DIAGNOSTIC_LIMIT_PROFILE,
    },
];

const PROTOCOL_REVISIONS: &[ProtocolRevisionCapability<'static>] = &[
    ProtocolRevisionCapability {
        revision: CURRENT_REVISION,
        recognition: "supported",
    },
    ProtocolRevisionCapability {
        revision: V2025_11_25,
        recognition: "supported",
    },
    ProtocolRevisionCapability {
        revision: V2025_06_18,
        recognition: "supported",
    },
    ProtocolRevisionCapability {
        revision: V2025_03_26,
        recognition: "recognized_unsupported",
    },
    ProtocolRevisionCapability {
        revision: V2024_11_05,
        recognition: "recognized_unsupported",
    },
];

const PROTOCOL_SUPPORT: &[ProtocolSupport<'static>] = &[
    ProtocolSupport {
        command: "break",
        transport: "stdio",
        revisions: ACTIVE_REVISIONS,
    },
    ProtocolSupport {
        command: "break",
        transport: "streamable_http",
        revisions: ACTIVE_REVISIONS,
    },
    ProtocolSupport {
        command: "check",
        transport: "stdio",
        revisions: ACTIVE_REVISIONS,
    },
    ProtocolSupport {
        command: "check",
        transport: "streamable_http",
        revisions: ACTIVE_REVISIONS,
    },
    ProtocolSupport {
        command: "inspect",
        transport: "stdio",
        revisions: INSPECT_REVISIONS,
    },
    ProtocolSupport {
        command: "inspect",
        transport: "streamable_http",
        revisions: INSPECT_REVISIONS,
    },
    ProtocolSupport {
        command: "reject",
        transport: "stdio",
        revisions: &[CURRENT_REVISION],
    },
    ProtocolSupport {
        command: "reject",
        transport: "streamable_http",
        revisions: &[CURRENT_REVISION],
    },
];

const AUTO_SELECTION_TRANSPORTS: &[AutoSelectionTransportCapability<'static>] = &[
    AutoSelectionTransportCapability {
        transport: "stdio",
        legacy_path: "stdio_legacy_initialization",
        max_prepared_targets: 0,
        max_process_launches: 2,
        max_lifecycle_requests: 2,
        max_lifecycle_notifications: 1,
        max_fallbacks: 1,
        shared_total_and_aggregate_budgets: true,
    },
    AutoSelectionTransportCapability {
        transport: "streamable_http",
        legacy_path: "http_legacy_initialization",
        max_prepared_targets: 1,
        max_process_launches: 0,
        max_lifecycle_requests: 2,
        max_lifecycle_notifications: 1,
        max_fallbacks: 1,
        shared_total_and_aggregate_budgets: true,
    },
];

const REPORTERS: &[ReporterCapability<'static>] = &[
    ReporterCapability {
        name: HUMAN_REPORTER,
        machine_readable: false,
    },
    ReporterCapability {
        name: JSON_REPORTER,
        machine_readable: true,
    },
    ReporterCapability {
        name: JUNIT_REPORTER,
        machine_readable: true,
    },
    ReporterCapability {
        name: MARKDOWN_REPORTER,
        machine_readable: false,
    },
    ReporterCapability {
        name: BADGE_REPORTER,
        machine_readable: true,
    },
];

const EXIT_CODES: &[ExitCodeCapability<'static>] = &[
    ExitCodeCapability {
        code: ExitStatus::Success.code(),
        meaning: "success",
    },
    ExitCodeCapability {
        code: ExitStatus::DiagnosticFailure.code(),
        meaning: "unsuccessful_result",
    },
    ExitCodeCapability {
        code: ExitStatus::InvocationError.code(),
        meaning: "invalid_invocation_or_input",
    },
    ExitCodeCapability {
        code: ExitStatus::Incomplete.code(),
        meaning: "incomplete_evidence",
    },
    ExitCodeCapability {
        code: ExitStatus::InternalError.code(),
        meaning: "internal_or_output_failure",
    },
];

const LIMIT_PROFILES: &[LimitProfileCapability<'static>] = &[
    LimitProfileCapability {
        id: AGGREGATE_LIMIT_PROFILE,
        default_for: &["aggregate"],
        hard: true,
        selections: &[],
        selectable_for: &[],
    },
    LimitProfileCapability {
        id: CAPABILITIES_LIMIT_PROFILE,
        default_for: &["capabilities"],
        hard: true,
        selections: &[],
        selectable_for: &[],
    },
    LimitProfileCapability {
        id: CONTRACT_DIFF_LIMIT_PROFILE,
        default_for: &["diff"],
        hard: true,
        selections: &[],
        selectable_for: &[],
    },
    LimitProfileCapability {
        id: DIAGNOSTIC_LIMIT_PROFILE,
        default_for: &["break", "check", "inspect", "reject"],
        hard: true,
        selections: DIAGNOSTIC_LIMIT_SELECTIONS,
        selectable_for: DIAGNOSTIC_LIMIT_SELECTABLE_FOR,
    },
];

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum CapabilitiesFormat {
    Human,
    Json,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct RenderedCapabilities {
    pub(crate) stdout: String,
    pub(crate) error: Option<CapabilitiesError>,
    pub(crate) exit_code: u8,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum CapabilitiesError {
    UnsupportedSchema,
    Render,
}

impl fmt::Display for CapabilitiesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchema => write!(
                formatter,
                "unsupported capabilities schema; supported schema: {CAPABILITIES_SCHEMA_VERSION}"
            ),
            Self::Render => formatter.write_str(
                "the capabilities response could not be rendered within its fixed output limit",
            ),
        }
    }
}

impl std::error::Error for CapabilitiesError {}

#[derive(Debug, Serialize)]
struct CapabilitiesManifest<'a> {
    schema_version: &'a str,
    schema_stability: &'a str,
    product: ProductCapability<'a>,
    commands: &'a [CommandCapability<'a>],
    protocol_revisions: &'a [ProtocolRevisionCapability<'a>],
    protocol_support: &'a [ProtocolSupport<'a>],
    protocol_selection: ProtocolSelectionCapability<'a>,
    schema_versions: SchemaVersions<'a>,
    status: StatusCapability<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    interruption: Option<InterruptionCapability<'a>>,
    reporters: &'a [ReporterCapability<'a>],
    exit_semantics: ExitSemantics<'a>,
    platform: PlatformCapabilities<'a>,
    limit_profiles: &'a [LimitProfileCapability<'a>],
    diagnostic_time_ceiling_profiles: [DiagnosticTimeCeilingProfile<'a>; 2],
    limits: CapabilitiesLimits,
}

#[derive(Debug, Serialize)]
struct ProductCapability<'a> {
    name: &'a str,
    version: &'a str,
}

#[derive(Debug, Serialize)]
struct CommandCapability<'a> {
    name: &'a str,
    activity: &'a str,
    reporters: &'a [&'a str],
    artifact_reporters: &'a [&'a str],
    input_schema_versions: &'a [&'a str],
    output_schema_versions: &'a [&'a str],
    generator_versions: &'a [&'a str],
    limit_profile: &'a str,
}

#[derive(Debug, Serialize)]
struct ProtocolRevisionCapability<'a> {
    revision: &'a str,
    recognition: &'a str,
}

#[derive(Debug, Serialize)]
struct ProtocolSupport<'a> {
    command: &'a str,
    transport: &'a str,
    revisions: &'a [&'a str],
}

#[derive(Debug, Serialize)]
struct ProtocolSelectionCapability<'a> {
    command: &'a str,
    default_mode: &'a str,
    modes: &'a [&'a str],
    compiled_modern_revisions: &'a [&'a str],
    exact_revisions: &'a [&'a str],
    exact_max_lifecycle_requests: u8,
    exact_max_fallbacks: u8,
    auto_transports: &'a [AutoSelectionTransportCapability<'a>],
}

#[derive(Debug, Serialize)]
struct AutoSelectionTransportCapability<'a> {
    transport: &'a str,
    legacy_path: &'a str,
    max_prepared_targets: u8,
    max_process_launches: u8,
    max_lifecycle_requests: u8,
    max_lifecycle_notifications: u8,
    max_fallbacks: u8,
    shared_total_and_aggregate_budgets: bool,
}

#[derive(Debug, Serialize)]
struct SchemaVersions<'a> {
    aggregate: &'a [&'a str],
    capabilities: &'a [&'a str],
    contract_diff: &'a [&'a str],
    contract_snapshot: &'a [&'a str],
    diagnostic_report: &'a [&'a str],
    markdown_report: &'a [&'a str],
    badge_report: &'a [&'a str],
    generator: &'a [&'a str],
    scenario: &'a [&'a str],
    status: &'a [&'a str],
}

#[derive(Debug, Serialize)]
struct StatusCapability<'a> {
    schema_version: &'a str,
    commands: &'a [&'a str],
    representations: &'a [StatusRepresentation<'a>],
    stream: &'a str,
    #[serde(rename = "default")]
    default_mode: &'a str,
    jsonl_stderr_exclusive: bool,
    limits: StatusLimits,
}

#[derive(Debug, Serialize)]
struct StatusRepresentation<'a> {
    name: &'a str,
    machine_readable: bool,
}

#[derive(Debug, Serialize)]
struct StatusLimits {
    event_bytes: usize,
    events: usize,
    output_bytes: usize,
    write_retries: u8,
}

#[derive(Debug, Serialize)]
struct InterruptionCapability<'a> {
    platform_family: &'a str,
    transport: &'a str,
    commands: &'a [&'a str],
    signals: &'a [&'a str],
    graceful_cleanup_ms: u64,
    forced_reap_ms: u64,
    cleanup_ceiling_ms: u64,
    incomplete_exit_code: u8,
    status_completion_reason: &'a str,
    publishes_report: bool,
    repeated_signal_forces_exit: bool,
}

#[derive(Debug, Serialize)]
struct ReporterCapability<'a> {
    name: &'a str,
    machine_readable: bool,
}

#[derive(Debug, Serialize)]
struct ExitSemantics<'a> {
    version: &'a str,
    codes: &'a [ExitCodeCapability<'a>],
}

#[derive(Debug, Serialize)]
struct ExitCodeCapability<'a> {
    code: u8,
    meaning: &'a str,
}

#[derive(Debug, Serialize)]
struct PlatformCapabilities<'a> {
    family: &'a str,
    process_tree_control: &'a str,
    file_identity: &'a str,
}

#[derive(Debug, Serialize)]
struct LimitProfileCapability<'a> {
    id: &'a str,
    default_for: &'a [&'a str],
    hard: bool,
    selections: &'a [&'a str],
    selectable_for: &'a [&'a str],
}

#[derive(Debug, Serialize)]
struct DiagnosticTimeCeilingProfile<'a> {
    profile: &'a str,
    startup: TimeCeiling<'a>,
    discovery: TimeCeiling<'a>,
    request: TimeCeiling<'a>,
    response: TimeCeiling<'a>,
    cleanup_grace: TimeCeiling<'a>,
    total: TimeCeiling<'a>,
    whole_process_exit_guarantee: bool,
}

#[derive(Debug, Serialize)]
struct TimeCeiling<'a> {
    milliseconds: u64,
    scope: &'a str,
}

#[derive(Debug, Serialize)]
struct CapabilitiesLimits {
    output_bytes: usize,
    runtime_shutdown_timeout_ms: u64,
}

#[derive(Debug, Serialize)]
struct CapabilitiesErrorDocument<'a> {
    schema_version: &'a str,
    schema_stability: &'a str,
    error: UnsupportedSchemaError<'a>,
}

#[derive(Debug, Serialize)]
struct UnsupportedSchemaError<'a> {
    code: &'a str,
    supported_schema_versions: &'a [&'a str],
}

pub(crate) fn render(format: CapabilitiesFormat, requested_schema: &str) -> RenderedCapabilities {
    if requested_schema != CAPABILITIES_SCHEMA_VERSION {
        return render_unsupported_schema(format);
    }

    let manifest = manifest();
    let output = match format {
        CapabilitiesFormat::Human => render_human(&manifest),
        CapabilitiesFormat::Json => render_json(&manifest),
    };
    match output {
        Ok(stdout) => RenderedCapabilities {
            stdout,
            error: None,
            exit_code: 0,
        },
        Err(error) => RenderedCapabilities {
            stdout: String::new(),
            error: Some(error),
            exit_code: 4,
        },
    }
}

#[cfg(test)]
pub(crate) fn command_names() -> impl Iterator<Item = &'static str> {
    COMMANDS.iter().map(|command| command.name)
}

fn manifest() -> CapabilitiesManifest<'static> {
    CapabilitiesManifest {
        schema_version: CAPABILITIES_SCHEMA_VERSION,
        schema_stability: "stable",
        product: ProductCapability {
            name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
        },
        commands: COMMANDS,
        protocol_revisions: PROTOCOL_REVISIONS,
        protocol_support: PROTOCOL_SUPPORT,
        protocol_selection: ProtocolSelectionCapability {
            command: "inspect",
            default_mode: "auto",
            modes: PROTOCOL_SELECTION_MODES,
            compiled_modern_revisions: MODERN_INSPECT_REVISIONS,
            exact_revisions: INSPECT_REVISIONS,
            exact_max_lifecycle_requests: 1,
            exact_max_fallbacks: 0,
            auto_transports: AUTO_SELECTION_TRANSPORTS,
        },
        schema_versions: SchemaVersions {
            aggregate: AGGREGATE_SCHEMAS,
            capabilities: CAPABILITIES_SCHEMAS,
            contract_diff: CONTRACT_DIFF_SCHEMAS,
            contract_snapshot: CONTRACT_SNAPSHOT_SCHEMAS,
            diagnostic_report: DIAGNOSTIC_REPORT_SCHEMAS,
            markdown_report: MARKDOWN_REPORT_SCHEMAS,
            badge_report: BADGE_REPORT_SCHEMAS,
            generator: GENERATOR_SCHEMAS,
            scenario: SCENARIO_SCHEMAS,
            status: STATUS_SCHEMAS,
        },
        status: StatusCapability {
            schema_version: STATUS_SCHEMA_VERSION,
            commands: STATUS_COMMANDS,
            representations: STATUS_REPRESENTATIONS,
            stream: "stderr",
            default_mode: "off",
            jsonl_stderr_exclusive: true,
            limits: StatusLimits {
                event_bytes: MAXIMUM_STATUS_EVENT_BYTES,
                events: MAXIMUM_STATUS_EVENTS,
                output_bytes: MAXIMUM_STATUS_OUTPUT_BYTES,
                write_retries: 0,
            },
        },
        interruption: interruption_capability(),
        reporters: REPORTERS,
        exit_semantics: ExitSemantics {
            version: EXIT_SEMANTICS_VERSION,
            codes: EXIT_CODES,
        },
        platform: platform_capabilities(),
        limit_profiles: LIMIT_PROFILES,
        diagnostic_time_ceiling_profiles: diagnostic_time_ceiling_profiles(),
        limits: CapabilitiesLimits {
            output_bytes: MAXIMUM_OUTPUT_BYTES,
            runtime_shutdown_timeout_ms: crate::RUNTIME_SHUTDOWN_TIMEOUT_MS,
        },
    }
}

#[cfg(unix)]
fn interruption_capability() -> Option<InterruptionCapability<'static>> {
    use crate::interruption::{CLEANUP_MS, GRACE_MS, REAP_MS};

    Some(InterruptionCapability {
        platform_family: "unix",
        transport: "stdio",
        commands: STATUS_COMMANDS,
        signals: &["SIGINT", "SIGTERM"],
        graceful_cleanup_ms: GRACE_MS,
        forced_reap_ms: REAP_MS,
        cleanup_ceiling_ms: CLEANUP_MS,
        incomplete_exit_code: 3,
        status_completion_reason: "interrupted",
        publishes_report: false,
        repeated_signal_forces_exit: false,
    })
}

#[cfg(not(unix))]
const fn interruption_capability() -> Option<InterruptionCapability<'static>> {
    None
}

fn diagnostic_time_ceiling_profiles() -> [DiagnosticTimeCeilingProfile<'static>; 2] {
    use crate::contract::DiagnosticLimitProfile;

    [
        diagnostic_time_ceiling_profile(DiagnosticLimitProfile::Default),
        diagnostic_time_ceiling_profile(DiagnosticLimitProfile::SlowStart),
    ]
}

fn diagnostic_time_ceiling_profile(
    selected: crate::contract::DiagnosticLimitProfile,
) -> DiagnosticTimeCeilingProfile<'static> {
    let limits = diagnostic_stdio_limit_profile(selected);
    DiagnosticTimeCeilingProfile {
        profile: selected.as_str(),
        startup: TimeCeiling {
            milliseconds: limits.startup_ms,
            scope: "target_preparation_or_process_start",
        },
        discovery: TimeCeiling {
            milliseconds: limits.discovery_ms,
            scope: "one_discovery_phase",
        },
        request: TimeCeiling {
            milliseconds: limits.request_ms,
            scope: "one_request_write_or_http_exchange",
        },
        response: TimeCeiling {
            milliseconds: limits.response_ms,
            scope: "one_response_wait",
        },
        cleanup_grace: TimeCeiling {
            milliseconds: limits.shutdown_grace_ms,
            scope: "graceful_cleanup_before_forced_termination",
        },
        total: TimeCeiling {
            milliseconds: limits.total_ms,
            scope: "stdio_startup_or_http_preparation_through_cleanup",
        },
        whole_process_exit_guarantee: false,
    }
}

fn render_unsupported_schema(format: CapabilitiesFormat) -> RenderedCapabilities {
    if format == CapabilitiesFormat::Human {
        return RenderedCapabilities {
            stdout: String::new(),
            error: Some(CapabilitiesError::UnsupportedSchema),
            exit_code: 2,
        };
    }

    let document = CapabilitiesErrorDocument {
        schema_version: CAPABILITIES_SCHEMA_VERSION,
        schema_stability: "stable",
        error: UnsupportedSchemaError {
            code: "unsupported_schema_version",
            supported_schema_versions: CAPABILITIES_SCHEMAS,
        },
    };
    match render_json(&document) {
        Ok(stdout) => RenderedCapabilities {
            stdout,
            error: None,
            exit_code: 2,
        },
        Err(error) => RenderedCapabilities {
            stdout: String::new(),
            error: Some(error),
            exit_code: 4,
        },
    }
}

fn render_json<T: Serialize>(value: &T) -> Result<String, CapabilitiesError> {
    let mut output = serde_json::to_string_pretty(value).map_err(|_| CapabilitiesError::Render)?;
    output.push('\n');
    ensure_output_bound(output)
}

fn render_human(manifest: &CapabilitiesManifest<'_>) -> Result<String, CapabilitiesError> {
    let mut output = String::new();
    writeln!(
        output,
        "mcp-doctor capabilities · {}",
        manifest.schema_version
    )
    .map_err(|_| CapabilitiesError::Render)?;
    writeln!(
        output,
        "Product: {} {}",
        manifest.product.name, manifest.product.version
    )
    .map_err(|_| CapabilitiesError::Render)?;
    writeln!(
        output,
        "Platform: {} · process tree {} · file identity {}",
        manifest.platform.family,
        manifest.platform.process_tree_control,
        manifest.platform.file_identity
    )
    .map_err(|_| CapabilitiesError::Render)?;
    writeln!(output, "Commands:").map_err(|_| CapabilitiesError::Render)?;
    for command in manifest.commands {
        writeln!(
            output,
            "  {} · {} · reporters {} · artifacts {} · limits {}",
            command.name,
            command.activity,
            command.reporters.join(","),
            command.artifact_reporters.join(","),
            command.limit_profile
        )
        .map_err(|_| CapabilitiesError::Render)?;
    }
    for profile in manifest
        .limit_profiles
        .iter()
        .filter(|profile| !profile.selections.is_empty())
    {
        writeln!(
            output,
            "Limit selections: {} · {} · commands {}",
            profile.id,
            profile.selections.join(","),
            profile.selectable_for.join(",")
        )
        .map_err(|_| CapabilitiesError::Render)?;
    }
    writeln!(
        output,
        "Status: {} · default {} · stream {} · commands {} · representations {}",
        manifest.status.schema_version,
        manifest.status.default_mode,
        manifest.status.stream,
        manifest.status.commands.join(","),
        manifest
            .status
            .representations
            .iter()
            .map(|representation| representation.name)
            .collect::<Vec<_>>()
            .join(",")
    )
    .map_err(|_| CapabilitiesError::Render)?;
    if let Some(interruption) = &manifest.interruption {
        writeln!(
            output,
            "Interruption: {} · {} · signals {} · commands {} · graceful_cleanup_ms={} · forced_reap_ms={} · cleanup_ceiling_ms={} · exit_code={} · completion_reason={} · publishes_report={} · repeated_signal_forces_exit={}",
            interruption.platform_family,
            interruption.transport,
            interruption.signals.join(","),
            interruption.commands.join(","),
            interruption.graceful_cleanup_ms,
            interruption.forced_reap_ms,
            interruption.cleanup_ceiling_ms,
            interruption.incomplete_exit_code,
            interruption.status_completion_reason,
            interruption.publishes_report,
            interruption.repeated_signal_forces_exit,
        )
        .map_err(|_| CapabilitiesError::Render)?;
    }
    if let Some(profile) = manifest.diagnostic_time_ceiling_profiles.first() {
        writeln!(
            output,
            "Time ceiling scopes: startup={} · discovery={} · request={} · response={} · cleanup_grace={} · total={}",
            profile.startup.scope,
            profile.discovery.scope,
            profile.request.scope,
            profile.response.scope,
            profile.cleanup_grace.scope,
            profile.total.scope,
        )
        .map_err(|_| CapabilitiesError::Render)?;
    }
    for profile in &manifest.diagnostic_time_ceiling_profiles {
        writeln!(
            output,
            "Time ceilings: {} · startup_ms={} · discovery_ms={} · request_ms={} · response_ms={} · cleanup_grace_ms={} · total_ms={} · whole_process_exit_guarantee={}",
            profile.profile,
            profile.startup.milliseconds,
            profile.discovery.milliseconds,
            profile.request.milliseconds,
            profile.response.milliseconds,
            profile.cleanup_grace.milliseconds,
            profile.total.milliseconds,
            profile.whole_process_exit_guarantee,
        )
        .map_err(|_| CapabilitiesError::Render)?;
    }
    writeln!(
        output,
        "Runtime shutdown: timeout_ms={} · scope=after_command_completion",
        manifest.limits.runtime_shutdown_timeout_ms,
    )
    .map_err(|_| CapabilitiesError::Render)?;
    writeln!(output, "Protocol support:").map_err(|_| CapabilitiesError::Render)?;
    for support in manifest.protocol_support {
        writeln!(
            output,
            "  {} · {} · {}",
            support.command,
            support.transport,
            support.revisions.join(",")
        )
        .map_err(|_| CapabilitiesError::Render)?;
    }
    writeln!(
        output,
        "Passive selection: {} · default {} · modes {} · modern {} · exact {}",
        manifest.protocol_selection.command,
        manifest.protocol_selection.default_mode,
        manifest.protocol_selection.modes.join(","),
        manifest
            .protocol_selection
            .compiled_modern_revisions
            .join(","),
        manifest.protocol_selection.exact_revisions.join(",")
    )
    .map_err(|_| CapabilitiesError::Render)?;
    for transport in manifest.protocol_selection.auto_transports {
        writeln!(
            output,
            "  auto · {} · path {} · prepared_targets {} · process_launches {} · lifecycle_requests {} · lifecycle_notifications {} · fallbacks {} · shared_budgets {}",
            transport.transport,
            transport.legacy_path,
            transport.max_prepared_targets,
            transport.max_process_launches,
            transport.max_lifecycle_requests,
            transport.max_lifecycle_notifications,
            transport.max_fallbacks,
            transport.shared_total_and_aggregate_budgets
        )
        .map_err(|_| CapabilitiesError::Render)?;
    }
    writeln!(
        output,
        "Exit semantics: {}",
        manifest.exit_semantics.version
    )
    .map_err(|_| CapabilitiesError::Render)?;
    ensure_output_bound(output)
}

fn ensure_output_bound(output: String) -> Result<String, CapabilitiesError> {
    if output.len() > MAXIMUM_OUTPUT_BYTES {
        Err(CapabilitiesError::Render)
    } else {
        Ok(output)
    }
}

#[cfg(unix)]
const fn platform_capabilities() -> PlatformCapabilities<'static> {
    PlatformCapabilities {
        family: "unix",
        process_tree_control: "process_group",
        file_identity: "device_inode",
    }
}

#[cfg(windows)]
const fn platform_capabilities() -> PlatformCapabilities<'static> {
    PlatformCapabilities {
        family: "windows",
        process_tree_control: "job_object",
        file_identity: "volume_file_id",
    }
}

#[cfg(not(any(unix, windows)))]
const fn platform_capabilities() -> PlatformCapabilities<'static> {
    PlatformCapabilities {
        family: "other",
        process_tree_control: "kill_on_drop",
        file_identity: "canonical_path",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{
        CAPABILITIES_SCHEMA_VERSION, CapabilitiesError, CapabilitiesFormat, MAXIMUM_OUTPUT_BYTES,
        render,
    };

    #[test]
    fn json_manifest_is_deterministic_bounded_and_compiled_only() {
        let first = render(CapabilitiesFormat::Json, CAPABILITIES_SCHEMA_VERSION);
        let second = render(CapabilitiesFormat::Json, CAPABILITIES_SCHEMA_VERSION);

        assert_eq!(first, second);
        assert_eq!(first.exit_code, 0);
        assert!(first.error.is_none());
        assert!(first.stdout.len() <= MAXIMUM_OUTPUT_BYTES);
        let document: Value = serde_json::from_str(&first.stdout).unwrap();
        assert_eq!(document["schema_version"], CAPABILITIES_SCHEMA_VERSION);
        assert_eq!(document["schema_stability"], "stable");
        assert_eq!(document["product"]["name"], "mcp-doctor");
        assert_eq!(document["product"]["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(document["limits"]["output_bytes"], MAXIMUM_OUTPUT_BYTES);
        assert_eq!(
            document["limits"]["runtime_shutdown_timeout_ms"],
            crate::RUNTIME_SHUTDOWN_TIMEOUT_MS
        );
        assert_eq!(document["protocol_selection"]["default_mode"], "auto");
        assert_eq!(
            document["protocol_selection"]["compiled_modern_revisions"],
            serde_json::json!(["2026-07-28"])
        );
        assert!(first.stdout.contains("mcp-doctor.limits/capabilities/v1"));
    }

    #[test]
    fn unknown_schema_is_exact_value_free_and_machine_readable() {
        const PRIVATE_REQUEST: &str = "synthetic-private-schema-request";

        let human = render(CapabilitiesFormat::Human, PRIVATE_REQUEST);
        assert_eq!(human.exit_code, 2);
        assert!(human.stdout.is_empty());
        assert_eq!(human.error, Some(CapabilitiesError::UnsupportedSchema));
        assert!(!human.error.unwrap().to_string().contains(PRIVATE_REQUEST));

        let json = render(CapabilitiesFormat::Json, PRIVATE_REQUEST);
        assert_eq!(json.exit_code, 2);
        assert!(json.error.is_none());
        assert!(!json.stdout.contains(PRIVATE_REQUEST));
        let document: Value = serde_json::from_str(&json.stdout).unwrap();
        assert_eq!(document["schema_version"], CAPABILITIES_SCHEMA_VERSION);
        assert_eq!(document["error"]["code"], "unsupported_schema_version");
        assert_eq!(
            document["error"]["supported_schema_versions"],
            serde_json::json!([CAPABILITIES_SCHEMA_VERSION])
        );
    }

    #[test]
    fn human_manifest_is_deterministic_and_bounded() {
        let rendered = render(CapabilitiesFormat::Human, CAPABILITIES_SCHEMA_VERSION);
        assert_eq!(rendered.exit_code, 0);
        assert!(rendered.error.is_none());
        assert!(rendered.stdout.len() <= MAXIMUM_OUTPUT_BYTES);
        assert!(
            rendered
                .stdout
                .starts_with("mcp-doctor capabilities · mcp-doctor.capabilities/v1\n")
        );
        assert!(rendered.stdout.contains("inspect · passive"));
        assert!(
            rendered
                .stdout
                .contains("check · stdio · 2026-07-28,2025-11-25,2025-06-18")
        );
        assert!(rendered.stdout.contains("mcp-doctor.exit/v1"));
        assert!(
            rendered
                .stdout
                .contains("Passive selection: inspect · default auto")
        );
    }
}
