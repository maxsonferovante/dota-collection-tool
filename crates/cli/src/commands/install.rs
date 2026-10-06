//! `install`: detect the Dota directory, resolve the token, write the file.

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::cfg;
use crate::cli::InstallArgs;
use crate::config;
use crate::paths;
use crate::steam;

/// Run `install`: returns the written config path.
pub async fn run(args: &InstallArgs, port: u16, db_override: Option<PathBuf>) -> Result<PathBuf> {
    let paths = paths::resolve(db_override)?;
    let token = match &args.token {
        Some(token) => token.clone(),
        None => config::ensure_token(&paths.config).await?,
    };
    let root = match &args.dota_dir {
        Some(dir) => dir.clone(),
        None => steam::find_dota_root()
            .context("Dota install not found automatically (set --dota-dir to the game root)")?,
    };
    let dir = steam::integration_dir(&root);
    let uri = format!("http://127.0.0.1:{port}/");
    let profile = config::active_profile(&paths.config).await?;
    let content = cfg::render_profile(&args.name, &uri, &token, profile);
    let written = cfg::install(&dir, &args.name, &content, args.force).await?;
    println!("wrote {}", written.display());
    println!(
        "start the game with the integration launch flag, then restart it after (re)installing."
    );
    Ok(written)
}
