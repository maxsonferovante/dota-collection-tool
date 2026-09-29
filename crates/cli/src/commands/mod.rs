//! Command dispatch. Handlers land per issue; unknown tasks stay stubs.

pub mod install;
pub mod logs;
pub mod token;

use anyhow::{Context, Result};

use crate::cli::{Cli, Command};
use crate::lifecycle;

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
        Command::Up(args) => {
            let exe = std::env::current_exe().context("cannot locate this binary for detach")?;
            lifecycle::up(&args, &exe, port, db).await?;
        }
        Command::Down => {
            lifecycle::down(db).await?;
        }
        Command::Status => {
            lifecycle::status(db, port).await?;
        }
        Command::Logs(args) => {
            logs::run(&args, db).await?;
        }
    }
    Ok(())
}
