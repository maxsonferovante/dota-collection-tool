//! Binary entry point: parse, set up logging, dispatch.

use anyhow::Result;
use clap::Parser;

use dct_cli::{cli::Cli, commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    commands::run(cli).await
}
