mod aggregate;
mod break_command;
mod capabilities;
mod check;
mod contract;
mod diff;
mod inspect;
mod report_artifacts;
mod transport;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::error::ErrorKind as ClapErrorKind;
use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};
use transport::http::RemoteOptions;

/// Diagnose protocol, schema, and runtime failures in MCP servers.
#[derive(Debug, Parser)]
#[command(name = "mcp-doctor", bin_name = "mcp-doctor", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Passively inspect a local STDIO server or one Streamable HTTP endpoint.
    Inspect(InspectArgs),
    /// Replay reviewed cases against one explicitly authorized local or remote tool.
    Check(CheckArgs),
    /// Generate deterministic boundary cases for one explicitly authorized tool.
    Break(BreakArgs),
    /// Compare two same-revision bounded snapshots without starting or contacting a target.
    Diff(DiffArgs),
    /// Aggregate bounded stable reports without starting or contacting a target.
    Aggregate(AggregateArgs),
    /// Show deterministic compiled capabilities without inspecting a host or target.
    Capabilities(CapabilitiesArgs),
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("transport")
        .required(true)
        .multiple(false)
        .args(["endpoint", "target"])
))]
struct InspectArgs {
    #[command(flatten)]
    report: ReportArgs,

    /// Exact MCP revision to inspect; legacy revisions are passive and opt-in only.
    #[arg(long, value_enum, default_value_t = InspectProtocolVersion::Current)]
    protocol_version: InspectProtocolVersion,

    /// Write a sensitive selected-revision contract snapshot to one new file.
    #[arg(long, value_name = "PATH")]
    snapshot: Option<PathBuf>,

    /// Acknowledge the same exact snapshot path and its sensitive advertised content.
    #[arg(long, value_name = "EXACT-PATH")]
    allow_sensitive_snapshot: Option<PathBuf>,

    /// One absolute Streamable HTTP endpoint.
    #[arg(value_name = "URL")]
    endpoint: Option<String>,

    #[command(flatten)]
    remote: RemoteArgs,

    /// Server executable followed by its literal arguments after `--`.
    #[arg(last = true, num_args = 1.., allow_hyphen_values = true)]
    target: Vec<OsString>,
}

#[derive(Debug, Args)]
struct DiffArgs {
    /// Diff format. JSON uses mcp-doctor.contract-diff/v1alpha1.
    #[arg(long, value_enum, default_value_t = DiffOutputFormat::Human)]
    format: DiffOutputFormat,

    /// Earlier bounded local contract snapshot.
    #[arg(value_name = "BEFORE")]
    before: PathBuf,

    /// Later bounded local contract snapshot.
    #[arg(value_name = "AFTER")]
    after: PathBuf,
}

#[derive(Debug, Args)]
struct AggregateArgs {
    /// Write stable mcp-doctor.aggregate/v1 JSON to this explicit new file.
    #[arg(long, value_name = "PATH")]
    output: PathBuf,

    /// Stdout projection; JSON is byte-identical to the required artifact.
    #[arg(long, value_enum, default_value_t = AggregateOutputFormat::Human)]
    format: AggregateOutputFormat,

    /// Ordered stable mcp-doctor.report/v1 JSON files.
    #[arg(value_name = "REPORT", required = true, num_args = 1..=32)]
    reports: Vec<PathBuf>,
}

#[derive(Debug, Args)]
struct CapabilitiesArgs {
    /// Output format. JSON uses stable mcp-doctor.capabilities/v1.
    #[arg(long, value_enum, default_value_t = CapabilitiesOutputFormat::Human)]
    format: CapabilitiesOutputFormat,

    /// Exact capability-manifest schema requested; unsupported versions never fall back.
    #[arg(
        long,
        value_name = "SCHEMA",
        default_value = capabilities::CAPABILITIES_SCHEMA_VERSION
    )]
    schema_version: String,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("transport")
        .required(true)
        .multiple(false)
        .args(["endpoint", "target"])
))]
struct CheckArgs {
    /// Exact MCP revision to use; legacy active behavior is opt-in only.
    #[arg(long, value_enum, default_value_t = ActiveProtocolVersion::Current)]
    protocol_version: ActiveProtocolVersion,

    /// Versioned JSON scenario containing one exact tool and ordered reviewed cases.
    #[arg(long, value_name = "PATH")]
    scenario: PathBuf,

    /// Exact tool name independently authorized for this run.
    #[arg(long, value_name = "EXACT-NAME")]
    allow_tool: String,

    /// Additionally authorize a scenario classified as side_effecting.
    #[arg(long)]
    allow_side_effects: bool,

    #[command(flatten)]
    report: ReportArgs,

    /// One absolute Streamable HTTP endpoint.
    #[arg(value_name = "URL")]
    endpoint: Option<String>,

    #[command(flatten)]
    remote: RemoteArgs,

    /// Server executable followed by its literal arguments after `--`.
    #[arg(last = true, num_args = 1.., allow_hyphen_values = true)]
    target: Vec<OsString>,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("transport")
        .required(true)
        .multiple(false)
        .args(["endpoint", "target"])
))]
struct BreakArgs {
    /// Exact MCP revision to use; legacy active behavior is opt-in only.
    #[arg(long, value_enum, default_value_t = ActiveProtocolVersion::Current)]
    protocol_version: ActiveProtocolVersion,

    /// Exact tool selected as the only generation and execution target.
    #[arg(long, value_name = "EXACT-NAME")]
    tool: String,

    /// Independently authorize the same exact tool for this run.
    #[arg(long, value_name = "EXACT-NAME")]
    allow_tool: String,

    /// Classify the selected tool's possible effects for this run.
    #[arg(long, value_enum)]
    effects: ToolEffects,

    /// Additionally authorize generated calls to a side_effecting tool.
    #[arg(long)]
    allow_side_effects: bool,

    /// Number of deterministic generated cases to run, from 1 through 100.
    #[arg(long, value_name = "COUNT")]
    cases: usize,

    /// Reproducible unsigned 64-bit generation seed.
    #[arg(long, value_name = "U64")]
    seed: u64,

    #[command(flatten)]
    report: ReportArgs,

    /// One absolute Streamable HTTP endpoint.
    #[arg(value_name = "URL")]
    endpoint: Option<String>,

    #[command(flatten)]
    remote: RemoteArgs,

    /// Server executable followed by its literal arguments after `--`.
    #[arg(last = true, num_args = 1.., allow_hyphen_values = true)]
    target: Vec<OsString>,
}

#[derive(Debug, Args)]
struct ReportArgs {
    /// Stdout report format. JSON uses stable mcp-doctor.report/v1; JUnit uses a conservative common subset.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,

    /// Write stable mcp-doctor.report/v1 JSON to one explicit new file.
    #[arg(long, value_name = "PATH")]
    json_report: Option<PathBuf>,

    /// Write the JUnit projection to one explicit new file.
    #[arg(long, value_name = "PATH")]
    junit_report: Option<PathBuf>,
}

#[derive(Debug, Args, Default)]
struct RemoteArgs {
    /// Authorize an eligible private destination for this exact endpoint.
    #[arg(long, value_name = "EXACT-URL", requires = "endpoint")]
    allow_private_network: Option<String>,

    /// Authorize cleartext HTTP for this exact all-loopback endpoint.
    #[arg(long, value_name = "EXACT-URL", requires = "endpoint")]
    allow_cleartext_http: Option<String>,

    /// Authorize environment-provided credentials for this exact HTTPS endpoint.
    #[arg(long, value_name = "EXACT-URL", requires = "endpoint")]
    allow_credentials_to: Option<String>,

    /// Read one bearer token from this invoking-process environment variable.
    #[arg(long, value_name = "NAME", requires = "endpoint")]
    bearer_token_env: Option<String>,

    /// Read a custom end-to-end field from FIELD=ENV_NAME; may be repeated.
    #[arg(long, value_name = "FIELD=NAME", requires = "endpoint")]
    header_env: Vec<String>,

    /// Add at most 32 PEM CA certificates from one bounded regular file.
    #[arg(long, value_name = "PATH", requires = "endpoint")]
    tls_ca_file: Option<PathBuf>,
}

impl RemoteArgs {
    fn into_options(self, endpoint: String) -> RemoteOptions {
        RemoteOptions {
            endpoint,
            allow_private_network: self.allow_private_network,
            allow_cleartext_http: self.allow_cleartext_http,
            allow_credentials_to: self.allow_credentials_to,
            bearer_token_env: self.bearer_token_env,
            header_env: self.header_env,
            tls_ca_file: self.tls_ca_file,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
    Junit,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
enum DiffOutputFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
enum AggregateOutputFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
enum CapabilitiesOutputFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
enum InspectProtocolVersion {
    #[value(name = "2026-07-28", alias = "current")]
    Current,
    #[value(name = "2025-11-25")]
    V2025_11_25,
    #[value(name = "2025-06-18")]
    V2025_06_18,
}

impl From<InspectProtocolVersion> for contract::ProtocolRevision {
    fn from(version: InspectProtocolVersion) -> Self {
        match version {
            InspectProtocolVersion::Current => Self::V2026_07_28,
            InspectProtocolVersion::V2025_11_25 => Self::V2025_11_25,
            InspectProtocolVersion::V2025_06_18 => Self::V2025_06_18,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
enum ActiveProtocolVersion {
    #[value(name = "2026-07-28", alias = "current")]
    Current,
    #[value(name = "2025-11-25")]
    V2025_11_25,
    #[value(name = "2025-06-18")]
    V2025_06_18,
}

impl From<ActiveProtocolVersion> for contract::ActiveProtocolRevision {
    fn from(version: ActiveProtocolVersion) -> Self {
        match version {
            ActiveProtocolVersion::Current => Self::V2026_07_28,
            ActiveProtocolVersion::V2025_11_25 => Self::V2025_11_25,
            ActiveProtocolVersion::V2025_06_18 => Self::V2025_06_18,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
enum ToolEffects {
    #[value(name = "read_only")]
    ReadOnly,
    #[value(name = "side_effecting")]
    SideEffecting,
}

impl From<OutputFormat> for contract::ReportFormat {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Human => Self::Human,
            OutputFormat::Json => Self::Json,
            OutputFormat::Junit => Self::Junit,
        }
    }
}

impl ReportArgs {
    fn prepare(
        &self,
        reserved_paths: &[&Path],
    ) -> Result<
        (
            contract::ReportRequest,
            report_artifacts::ReportArtifactDestinations,
        ),
        report_artifacts::ReportArtifactError,
    > {
        let destinations = report_artifacts::ReportArtifactDestinations::prepare(
            self.json_report.clone(),
            self.junit_report.clone(),
            reserved_paths,
        )?;
        let request = contract::ReportRequest::new(
            self.format.into(),
            destinations.requests_json(),
            destinations.requests_junit(),
        );
        Ok((request, destinations))
    }
}

impl From<DiffOutputFormat> for contract::DiffFormat {
    fn from(format: DiffOutputFormat) -> Self {
        match format {
            DiffOutputFormat::Human => Self::Human,
            DiffOutputFormat::Json => Self::Json,
        }
    }
}

impl From<AggregateOutputFormat> for aggregate::AggregateFormat {
    fn from(format: AggregateOutputFormat) -> Self {
        match format {
            AggregateOutputFormat::Human => Self::Human,
            AggregateOutputFormat::Json => Self::Json,
        }
    }
}

impl From<CapabilitiesOutputFormat> for capabilities::CapabilitiesFormat {
    fn from(format: CapabilitiesOutputFormat) -> Self {
        match format {
            CapabilitiesOutputFormat::Human => Self::Human,
            CapabilitiesOutputFormat::Json => Self::Json,
        }
    }
}

fn emit_diagnostic(
    diagnostic: contract::Diagnostic,
    request: contract::ReportRequest,
    destinations: report_artifacts::ReportArtifactDestinations,
) -> ExitCode {
    let rendered = diagnostic.render(request);
    let exit = rendered.exit;
    let output = rendered.output;
    if let Some(error) = rendered.error {
        print!("{output}");
        eprintln!("error: {error}");
        if let Err(cleanup) = destinations.cancel() {
            eprintln!("error: {cleanup}");
        }
        return exit;
    }

    let persistence = destinations.persist(rendered.artifacts);
    print!("{output}");
    match persistence {
        Ok(()) => exit,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(4)
        }
    }
}

fn emit_invocation_error(
    error: &dyn std::fmt::Display,
    destinations: report_artifacts::ReportArtifactDestinations,
) -> ExitCode {
    eprintln!("error: {error}");
    if let Err(cleanup) = destinations.cancel() {
        eprintln!("error: {cleanup}");
        ExitCode::from(4)
    } else {
        ExitCode::from(2)
    }
}

fn emit_inspect(
    output: inspect::InspectOutput,
    destination: Option<contract::SnapshotDestination>,
    request: contract::ReportRequest,
    report_destinations: report_artifacts::ReportArtifactDestinations,
) -> ExitCode {
    if let Some(snapshot) = output.snapshot {
        let Some(destination) = destination else {
            return emit_invocation_error(
                &"a contract snapshot was produced without an authorized destination",
                report_destinations,
            );
        };
        if let Err(error) = destination.persist(&snapshot) {
            return emit_invocation_error(&error, report_destinations);
        }
    }
    emit_diagnostic(output.diagnostic, request, report_destinations)
}

fn emit_contract_diff(diff: contract::RenderedContractDiff) -> ExitCode {
    print!("{}", diff.output);
    if let Some(error) = diff.error {
        eprintln!("error: {error}");
    }
    diff.exit
}

fn emit_aggregate(
    result: Result<aggregate::RenderedAggregate, aggregate::AggregateError>,
    destinations: report_artifacts::ReportArtifactDestinations,
    deadline: &aggregate::AggregateDeadline,
) -> ExitCode {
    let rendered = match result {
        Ok(rendered) => rendered,
        Err(error) => {
            eprintln!("error: {error}");
            return if let Err(cleanup) = destinations.cancel() {
                eprintln!("error: {cleanup}");
                ExitCode::from(4)
            } else {
                ExitCode::from(error.exit_code())
            };
        }
    };
    let exit = rendered.exit;
    let artifact = contract::RenderedReportArtifact {
        format: contract::ReportArtifactFormat::Json,
        output: rendered.artifact,
    };
    match destinations.persist_checked(vec![artifact], || deadline.within_limit()) {
        Ok(()) => {
            print!("{}", rendered.stdout);
            ExitCode::from(exit)
        }
        Err(error) => {
            eprintln!("error: {error}");
            if error == report_artifacts::ReportArtifactError::OperationLimit {
                ExitCode::from(2)
            } else {
                ExitCode::from(4)
            }
        }
    }
}

fn emit_capabilities(rendered: capabilities::RenderedCapabilities) -> ExitCode {
    print!("{}", rendered.stdout);
    if let Some(error) = rendered.error {
        eprintln!("error: {error}");
    }
    ExitCode::from(rendered.exit_code)
}

fn parse_cli() -> Result<Cli, ExitCode> {
    let first_argument = std::env::args_os().nth(1);
    let sanitized_command = match first_argument.as_deref() {
        Some(value) if value == OsStr::new("aggregate") => Some("aggregate"),
        Some(value) if value == OsStr::new("capabilities") => Some("capabilities"),
        _ => None,
    };
    let Some(sanitized_command) = sanitized_command else {
        return Ok(Cli::parse());
    };

    match Cli::try_parse() {
        Ok(cli) => Ok(cli),
        Err(error)
            if matches!(
                error.kind(),
                ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion
            ) =>
        {
            if error.print().is_ok() {
                Err(ExitCode::SUCCESS)
            } else {
                Err(ExitCode::from(4))
            }
        }
        Err(_) => {
            eprintln!(
                "error: invalid {sanitized_command} invocation; use `mcp-doctor {sanitized_command} --help`"
            );
            Err(ExitCode::from(2))
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = match parse_cli() {
        Ok(cli) => cli,
        Err(exit) => return exit,
    };
    match cli.command {
        None => contract::success_exit(),
        Some(Command::Inspect(arguments)) => {
            let revision = arguments.protocol_version.into();
            let destination = match contract::prepare_snapshot_destination(
                arguments.snapshot,
                arguments.allow_sensitive_snapshot,
            ) {
                Ok(destination) => destination,
                Err(error) => {
                    eprintln!("error: {error}");
                    return ExitCode::from(2);
                }
            };
            let reserved_paths = destination
                .as_ref()
                .map(contract::SnapshotDestination::path)
                .into_iter()
                .collect::<Vec<_>>();
            let (report_request, report_destinations) =
                match arguments.report.prepare(&reserved_paths) {
                    Ok(prepared) => prepared,
                    Err(error) => {
                        eprintln!("error: {error}");
                        return ExitCode::from(2);
                    }
                };
            drop(reserved_paths);
            let capture_snapshot = destination.is_some();
            if let Some(endpoint) = arguments.endpoint {
                match inspect::run_http(
                    arguments.remote.into_options(endpoint),
                    revision,
                    capture_snapshot,
                )
                .await
                {
                    Ok(output) => {
                        emit_inspect(output, destination, report_request, report_destinations)
                    }
                    Err(error) => emit_invocation_error(&error, report_destinations),
                }
            } else {
                match inspect::run_stdio(arguments.target, revision, capture_snapshot).await {
                    Ok(output) => {
                        emit_inspect(output, destination, report_request, report_destinations)
                    }
                    Err(error) => emit_invocation_error(&error, report_destinations),
                }
            }
        }
        Some(Command::Diff(arguments)) => emit_contract_diff(diff::run(
            &arguments.before,
            &arguments.after,
            arguments.format.into(),
        )),
        Some(Command::Aggregate(arguments)) => {
            let deadline = aggregate::AggregateDeadline::start();
            let destinations = match report_artifacts::ReportArtifactDestinations::prepare(
                Some(arguments.output),
                None,
                &[],
            ) {
                Ok(destinations) => destinations,
                Err(error) => {
                    eprintln!("error: {error}");
                    return ExitCode::from(2);
                }
            };
            emit_aggregate(
                aggregate::run(&arguments.reports, arguments.format.into(), &deadline),
                destinations,
                &deadline,
            )
        }
        Some(Command::Capabilities(arguments)) => emit_capabilities(capabilities::render(
            arguments.format.into(),
            &arguments.schema_version,
        )),
        Some(Command::Check(arguments)) => {
            let revision = arguments.protocol_version.into();
            let (report_request, report_destinations) = match arguments.report.prepare(&[]) {
                Ok(prepared) => prepared,
                Err(error) => {
                    eprintln!("error: {error}");
                    return ExitCode::from(2);
                }
            };
            if let Some(endpoint) = arguments.endpoint {
                let diagnostic = check::run_http(
                    arguments.remote.into_options(endpoint),
                    &arguments.scenario,
                    &arguments.allow_tool,
                    arguments.allow_side_effects,
                    revision,
                )
                .await;
                emit_diagnostic(diagnostic, report_request, report_destinations)
            } else {
                match check::run_stdio(
                    arguments.target,
                    &arguments.scenario,
                    &arguments.allow_tool,
                    arguments.allow_side_effects,
                    revision,
                )
                .await
                {
                    Ok(diagnostic) => {
                        emit_diagnostic(diagnostic, report_request, report_destinations)
                    }
                    Err(error) => emit_invocation_error(&error, report_destinations),
                }
            }
        }
        Some(Command::Break(arguments)) => {
            let BreakArgs {
                protocol_version,
                tool,
                allow_tool,
                effects,
                allow_side_effects,
                cases,
                seed,
                report,
                endpoint,
                remote,
                target,
            } = arguments;
            let (report_request, report_destinations) = match report.prepare(&[]) {
                Ok(prepared) => prepared,
                Err(error) => {
                    eprintln!("error: {error}");
                    return ExitCode::from(2);
                }
            };
            let options = break_command::BreakOptions {
                revision: protocol_version.into(),
                tool,
                allowed_tool: &allow_tool,
                side_effecting: effects == ToolEffects::SideEffecting,
                allow_side_effects,
                cases,
                seed,
            };
            if let Some(endpoint) = endpoint {
                let diagnostic =
                    break_command::run_http(remote.into_options(endpoint), options).await;
                emit_diagnostic(diagnostic, report_request, report_destinations)
            } else {
                match break_command::run_stdio(target, options).await {
                    Ok(diagnostic) => {
                        emit_diagnostic(diagnostic, report_request, report_destinations)
                    }
                    Err(error) => emit_invocation_error(&error, report_destinations),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, capabilities};
    use clap::CommandFactory;

    #[test]
    fn command_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn capability_command_inventory_matches_the_cli() {
        let cli = Cli::command();
        let mut cli_commands = cli
            .get_subcommands()
            .map(|command| command.get_name())
            .collect::<Vec<_>>();
        cli_commands.sort_unstable();
        assert_eq!(
            cli_commands,
            capabilities::command_names().collect::<Vec<_>>()
        );
    }
}
