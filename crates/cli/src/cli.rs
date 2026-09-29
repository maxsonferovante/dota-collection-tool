//! Command-line surface: global options plus one subcommand per task.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Local game-state collector: HTTP sink plus manager CLI over SQLite.
#[derive(Debug, Parser)]
#[command(name = "dct", version, about = "Local game-state collector")]
pub struct Cli {
    /// TCP port the server listens on.
    #[arg(long, default_value_t = 53_000)]
    pub port: u16,

    /// SQLite database path. Defaults next to the local config file.
    #[arg(long)]
    pub db: Option<PathBuf>,

    /// Verbose operational logging on stderr.
    #[arg(long, short)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

/// The six manager tasks.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Write the game config file into the Dota cfg directory.
    Install(InstallArgs),
    /// Generate, show or rotate the auth token.
    Token(TokenArgs),
    /// Start the server (foreground by default).
    Up(UpArgs),
    /// Stop a detached server.
    Down,
    /// Show whether the server is running and healthy.
    Status,
    /// Query recorded happenings.
    Logs(LogsArgs),
}

/// Options for `install`.
#[derive(Debug, Clone, Parser)]
pub struct InstallArgs {
    /// Config name: the file becomes `gamestate_integration_<NAME>.cfg`.
    #[arg(long, default_value = "dct")]
    pub name: String,

    /// Override the detected Dota directory.
    #[arg(long)]
    pub dota_dir: Option<PathBuf>,

    /// Use this token instead of the stored one.
    #[arg(long)]
    pub token: Option<String>,

    /// Overwrite an existing config file (backs it up as `.bak`).
    #[arg(long)]
    pub force: bool,
}

/// Options for `token`.
#[derive(Debug, Clone, Parser)]
pub struct TokenArgs {
    /// Print the active token instead of generating a new one.
    #[arg(long)]
    pub show: bool,

    /// Replace the active token with a fresh one.
    #[arg(long)]
    pub rotate: bool,
}

/// Options for `up`.
#[derive(Debug, Parser)]
pub struct UpArgs {
    /// Detach into the background and print the PID.
    #[arg(long)]
    pub detach: bool,
}

/// Options for `logs`.
#[derive(Debug, Parser)]
pub struct LogsArgs {
    /// Only happenings from this match.
    #[arg(long = "match")]
    pub match_id: Option<String>,

    /// Only happenings of this kind.
    #[arg(long)]
    pub kind: Option<String>,

    /// Maximum happenings to print.
    #[arg(long, default_value_t = 50)]
    pub limit: usize,

    /// Keep following new happenings.
    #[arg(long)]
    pub follow: bool,

    /// Print newline-delimited JSON instead of text.
    #[arg(long)]
    pub json: bool,
}
