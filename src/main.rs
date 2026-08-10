mod contract;
mod inspect;
mod transport;

use std::ffi::OsString;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

/// Diagnose protocol, schema, and runtime failures in MCP servers.
#[derive(Debug, Parser)]
#[command(name = "mcp-doctor", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Passively inspect a local MCP server over STDIO.
    Inspect(InspectArgs),
}

#[derive(Debug, Args)]
struct InspectArgs {
    /// Server executable followed by its literal arguments.
    #[arg(last = true, required = true, num_args = 1.., allow_hyphen_values = true)]
    target: Vec<OsString>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    match Cli::parse().command {
        None => contract::success_exit(),
        Some(Command::Inspect(arguments)) => match inspect::run(arguments.target).await {
            Ok(diagnostic) => {
                print!("{}", diagnostic.output);
                diagnostic.exit
            }
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::from(2)
            }
        },
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
