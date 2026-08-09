mod contract;

use std::process::ExitCode;

use clap::Parser;

/// Diagnose protocol, schema, and runtime failures in MCP servers.
#[derive(Debug, Parser)]
#[command(name = "mcp-doctor", version)]
struct Cli;

fn main() -> ExitCode {
    Cli::parse();
    contract::success_exit()
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
