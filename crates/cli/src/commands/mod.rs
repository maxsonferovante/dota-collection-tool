//! Command dispatch. Handlers land per issue; unknown tasks stay stubs.

pub mod install;
pub mod token;

use anyhow::{Result, bail};

use crate::cli::{Cli, Command};

/// Run the parsed command.
pub async fn run(cli: Cli) -> Result<()> {
    let db = cli.db.clone();
    let port = cli.port;
    match cli.command {
        Command::Install(args) => {
            install::run(&args, port, db).await?;
        }
        Command::Token(args) => {
            token::run(&args, db).await?;
        }
        Command::Up(_) => bail!("up: not yet implemented (issue #6)"),
        Command::Down => bail!("down: not yet implemented (issue #6)"),
        Command::Status => bail!("status: not yet implemented (issue #6)"),
        Command::Logs(_) => bail!("logs: not yet implemented (issue #7)"),
    }
    Ok(())
}
