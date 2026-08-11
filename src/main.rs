mod break_command;
mod check;
mod contract;
mod inspect;
mod transport;

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

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
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("transport")
        .required(true)
        .multiple(false)
        .args(["endpoint", "target"])
))]
struct InspectArgs {
    /// Report format. JSON uses stable mcp-doctor.report/v1; JUnit uses a conservative common subset.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,

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
struct CheckArgs {
    /// Versioned JSON scenario containing one exact tool and ordered reviewed cases.
    #[arg(long, value_name = "PATH")]
    scenario: PathBuf,

    /// Exact tool name independently authorized for this run.
    #[arg(long, value_name = "EXACT-NAME")]
    allow_tool: String,

    /// Additionally authorize a scenario classified as side_effecting.
    #[arg(long)]
    allow_side_effects: bool,

    /// Report format. JSON uses stable mcp-doctor.report/v1; JUnit uses a conservative common subset.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,

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

    /// Report format. JSON uses stable mcp-doctor.report/v1; JUnit uses a conservative common subset.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,

    /// One absolute Streamable HTTP endpoint.
    #[arg(value_name = "URL")]
    endpoint: Option<String>,

    #[command(flatten)]
    remote: RemoteArgs,

    /// Server executable followed by its literal arguments after `--`.
    #[arg(last = true, num_args = 1.., allow_hyphen_values = true)]
    target: Vec<OsString>,
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

fn emit_diagnostic(diagnostic: contract::RenderedDiagnostic) -> ExitCode {
    print!("{}", diagnostic.output);
    if let Some(error) = diagnostic.error {
        eprintln!("error: {error}");
    }
    diagnostic.exit
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    match Cli::parse().command {
        None => contract::success_exit(),
        Some(Command::Inspect(arguments)) => {
            if let Some(endpoint) = arguments.endpoint {
                let diagnostic = inspect::run_http(
                    arguments.remote.into_options(endpoint),
                    arguments.format.into(),
                )
                .await;
                emit_diagnostic(diagnostic)
            } else {
                match inspect::run_stdio(arguments.target, arguments.format.into()).await {
                    Ok(diagnostic) => emit_diagnostic(diagnostic),
                    Err(error) => {
                        eprintln!("error: {error}");
                        ExitCode::from(2)
                    }
                }
            }
        }
        Some(Command::Check(arguments)) => {
            if let Some(endpoint) = arguments.endpoint {
                let diagnostic = check::run_http(
                    arguments.remote.into_options(endpoint),
                    &arguments.scenario,
                    &arguments.allow_tool,
                    arguments.allow_side_effects,
                    arguments.format.into(),
                )
                .await;
                emit_diagnostic(diagnostic)
            } else {
                match check::run_stdio(
                    arguments.target,
                    &arguments.scenario,
                    &arguments.allow_tool,
                    arguments.allow_side_effects,
                    arguments.format.into(),
                )
                .await
                {
                    Ok(diagnostic) => emit_diagnostic(diagnostic),
                    Err(error) => {
                        eprintln!("error: {error}");
                        ExitCode::from(2)
                    }
                }
            }
        }
        Some(Command::Break(arguments)) => {
            let BreakArgs {
                tool,
                allow_tool,
                effects,
                allow_side_effects,
                cases,
                seed,
                format,
                endpoint,
                remote,
                target,
            } = arguments;
            let options = break_command::BreakOptions {
                tool,
                allowed_tool: &allow_tool,
                side_effecting: effects == ToolEffects::SideEffecting,
                allow_side_effects,
                cases,
                seed,
                format: format.into(),
            };
            if let Some(endpoint) = endpoint {
                let diagnostic =
                    break_command::run_http(remote.into_options(endpoint), options).await;
                emit_diagnostic(diagnostic)
            } else {
                match break_command::run_stdio(target, options).await {
                    Ok(diagnostic) => emit_diagnostic(diagnostic),
                    Err(error) => {
                        eprintln!("error: {error}");
                        ExitCode::from(2)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::CommandFactory;

    #[test]
    fn command_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
