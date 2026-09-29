//! `token`: show the active token, or generate/rotate it.
//!
//! The value is printed to stdout by these explicit commands only —
//! never into logs.

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::cli::TokenArgs;
use crate::config;
use crate::paths;

/// Run `token`: returns the relevant token.
pub async fn run(args: &TokenArgs, db_override: Option<PathBuf>) -> Result<String> {
    let paths = paths::resolve(db_override)?;
    if args.show {
        let token = config::active_token(&paths.config)
            .await?
            .context("no token stored yet (run `token` without flags to generate one)")?;
        println!("{token}");
        return Ok(token);
    }
    let token = config::fresh_token(&paths.config).await?;
    if args.rotate {
        println!("token rotated");
    } else {
        println!("{token}");
    }
    Ok(token)
}
