//! Command dispatch. Each handler lands in its own issue.

use anyhow::{Result, bail};

use crate::cli::{Cli, Command};

/// Run the parsed command. Handlers are implemented per issue.
pub async fn run(cli: Cli) -> Result<()> {
    let command = cli.command;
    match command {
        Command::Install(_) => bail!("install: not yet implemented (issue #5)"),
        Command::Token(_) => bail!("token: not yet implemented (issue #5)"),
        Command::Up(_) => bail!("up: not yet implemented (issue #6)"),
        Command::Down => bail!("down: not yet implemented (issue #6)"),
        Command::Status => bail!("status: not yet implemented (issue #6)"),
        Command::Logs(_) => bail!("logs: not yet implemented (issue #7)"),
    }
}
