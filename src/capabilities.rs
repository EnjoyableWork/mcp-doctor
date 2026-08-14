use std::fmt::{self, Write as _};

use serde::Serialize;

use crate::aggregate::AGGREGATE_SCHEMA_VERSION;
use crate::contract::{
    DIFF_SCHEMA_VERSION, ExitStatus, GENERATOR_VERSION, KnownRevision, ProtocolRevision,
    REPORT_SCHEMA_VERSION, SCENARIO_SCHEMA_VERSION, SNAPSHOT_SCHEMA_VERSION,
};

pub(crate) const CAPABILITIES_SCHEMA_VERSION: &str = "mcp-doctor.capabilities/v1";
pub(crate) const EXIT_SEMANTICS_VERSION: &str = "mcp-doctor.exit/v1";
pub(crate) const CAPABILITIES_LIMIT_PROFILE: &str = "mcp-doctor.limits/capabilities/v1";
pub(crate) const MAXIMUM_OUTPUT_BYTES: usize = 64 * 1024;

const AGGREGATE_LIMIT_PROFILE: &str = "mcp-doctor.limits/aggregate/v1";
const CONTRACT_DIFF_LIMIT_PROFILE: &str = "mcp-doctor.limits/contract-diff/v1alpha1";
const DIAGNOSTIC_LIMIT_PROFILE: &str = "mcp-doctor.limits/diagnostic/v1";

const HUMAN_REPORTER: &str = "human";
const JSON_REPORTER: &str = "json";
const JUNIT_REPORTER: &str = "junit";

const NO_SCHEMAS: &[&str] = &[];
const AGGREGATE_SCHEMAS: &[&str] = &[AGGREGATE_SCHEMA_VERSION];
const CAPABILITIES_SCHEMAS: &[&str] = &[CAPABILITIES_SCHEMA_VERSION];
const CONTRACT_DIFF_SCHEMAS: &[&str] = &[DIFF_SCHEMA_VERSION];
const CONTRACT_SNAPSHOT_SCHEMAS: &[&str] = &[SNAPSHOT_SCHEMA_VERSION];
const DIAGNOSTIC_REPORT_SCHEMAS: &[&str] = &[REPORT_SCHEMA_VERSION];
const GENERATOR_SCHEMAS: &[&str] = &[GENERATOR_VERSION];
const SCENARIO_SCHEMAS: &[&str] = &[SCENARIO_SCHEMA_VERSION];

const HUMAN_JSON_REPORTERS: &[&str] = &[HUMAN_REPORTER, JSON_REPORTER];
const DIAGNOSTIC_REPORTERS: &[&str] = &[HUMAN_REPORTER, JSON_REPORTER, JUNIT_REPORTER];

const CURRENT_REVISION: &str = ProtocolRevision::V2026_07_28.as_str();
const V2025_11_25: &str = ProtocolRevision::V2025_11_25.as_str();
const V2025_06_18: &str = ProtocolRevision::V2025_06_18.as_str();
const V2025_03_26: &str = KnownRevision::V2025_03_26.as_str();
const V2024_11_05: &str = KnownRevision::V2024_11_05.as_str();
const ACTIVE_REVISIONS: &[&str] = &[CURRENT_REVISION, V2025_11_25];
const INSPECT_REVISIONS: &[&str] = &[CURRENT_REVISION, V2025_11_25, V2025_06_18];

const COMMANDS: &[CommandCapability<'static>] = &[
    CommandCapability {
        name: "aggregate",
        activity: "offline",
        reporters: HUMAN_JSON_REPORTERS,
        input_schema_versions: DIAGNOSTIC_REPORT_SCHEMAS,
        output_schema_versions: AGGREGATE_SCHEMAS,
        generator_versions: NO_SCHEMAS,
        limit_profile: AGGREGATE_LIMIT_PROFILE,
    },
    CommandCapability {
        name: "break",
        activity: "active",
        reporters: DIAGNOSTIC_REPORTERS,
        input_schema_versions: NO_SCHEMAS,
        output_schema_versions: DIAGNOSTIC_REPORT_SCHEMAS,
        generator_versions: GENERATOR_SCHEMAS,
        limit_profile: DIAGNOSTIC_LIMIT_PROFILE,
    },
    CommandCapability {
        name: "capabilities",
        activity: "compiled_only",
        reporters: HUMAN_JSON_REPORTERS,
        input_schema_versions: NO_SCHEMAS,
        output_schema_versions: CAPABILITIES_SCHEMAS,
        generator_versions: NO_SCHEMAS,
        limit_profile: CAPABILITIES_LIMIT_PROFILE,
    },
    CommandCapability {
        name: "check",
        activity: "active",
        reporters: DIAGNOSTIC_REPORTERS,
        input_schema_versions: SCENARIO_SCHEMAS,
        output_schema_versions: DIAGNOSTIC_REPORT_SCHEMAS,
        generator_versions: NO_SCHEMAS,
        limit_profile: DIAGNOSTIC_LIMIT_PROFILE,
    },
    CommandCapability {
        name: "diff",
        activity: "offline",
        reporters: HUMAN_JSON_REPORTERS,
        input_schema_versions: CONTRACT_SNAPSHOT_SCHEMAS,
        output_schema_versions: CONTRACT_DIFF_SCHEMAS,
        generator_versions: NO_SCHEMAS,
        limit_profile: CONTRACT_DIFF_LIMIT_PROFILE,
    },
    CommandCapability {
        name: "inspect",
        activity: "passive",
        reporters: DIAGNOSTIC_REPORTERS,
        input_schema_versions: NO_SCHEMAS,
        output_schema_versions: &[REPORT_SCHEMA_VERSION, SNAPSHOT_SCHEMA_VERSION],
        generator_versions: NO_SCHEMAS,
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
    },
    LimitProfileCapability {
        id: CAPABILITIES_LIMIT_PROFILE,
        default_for: &["capabilities"],
        hard: true,
    },
    LimitProfileCapability {
        id: CONTRACT_DIFF_LIMIT_PROFILE,
        default_for: &["diff"],
        hard: true,
    },
    LimitProfileCapability {
        id: DIAGNOSTIC_LIMIT_PROFILE,
        default_for: &["break", "check", "inspect"],
        hard: true,
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
    schema_versions: SchemaVersions<'a>,
    reporters: &'a [ReporterCapability<'a>],
    exit_semantics: ExitSemantics<'a>,
    platform: PlatformCapabilities<'a>,
    limit_profiles: &'a [LimitProfileCapability<'a>],
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
struct SchemaVersions<'a> {
    aggregate: &'a [&'a str],
    capabilities: &'a [&'a str],
    contract_diff: &'a [&'a str],
    contract_snapshot: &'a [&'a str],
    diagnostic_report: &'a [&'a str],
    generator: &'a [&'a str],
    scenario: &'a [&'a str],
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
}

#[derive(Debug, Serialize)]
struct CapabilitiesLimits {
    output_bytes: usize,
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
        schema_versions: SchemaVersions {
            aggregate: AGGREGATE_SCHEMAS,
            capabilities: CAPABILITIES_SCHEMAS,
            contract_diff: CONTRACT_DIFF_SCHEMAS,
            contract_snapshot: CONTRACT_SNAPSHOT_SCHEMAS,
            diagnostic_report: DIAGNOSTIC_REPORT_SCHEMAS,
            generator: GENERATOR_SCHEMAS,
            scenario: SCENARIO_SCHEMAS,
        },
        reporters: REPORTERS,
        exit_semantics: ExitSemantics {
            version: EXIT_SEMANTICS_VERSION,
            codes: EXIT_CODES,
        },
        platform: platform_capabilities(),
        limit_profiles: LIMIT_PROFILES,
        limits: CapabilitiesLimits {
            output_bytes: MAXIMUM_OUTPUT_BYTES,
        },
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
            "  {} · {} · reporters {} · limits {}",
            command.name,
            command.activity,
            command.reporters.join(","),
            command.limit_profile
        )
        .map_err(|_| CapabilitiesError::Render)?;
    }
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
                .contains("check · stdio · 2026-07-28,2025-11-25")
        );
        assert!(rendered.stdout.contains("mcp-doctor.exit/v1"));
    }
}
