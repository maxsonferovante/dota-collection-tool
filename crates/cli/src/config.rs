//! Auth token lifecycle: generate, persist locally, rotate.
//!
//! Tokens are 32 CSPRNG bytes rendered as hex. They live in a TOML file
//! next to the database and are injected into the game config at install
//! time. The value is printed only by explicit token commands.

use std::path::Path;

use anyhow::{Context, Result};
use rand::TryRng;
use serde::{Deserialize, Serialize};

/// On-disk local config.
#[derive(Debug, Default, Serialize, Deserialize)]
struct LocalConfig {
    #[serde(default)]
    token: Option<String>,
}

/// 32 random bytes as 64 lowercase hex chars.
pub fn generate_token() -> Result<String> {
    let mut bytes = [0u8; 32];
    rand::rngs::SysRng
        .try_fill_bytes(&mut bytes)
        .map_err(|err| anyhow::anyhow!("os entropy unavailable: {err}"))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

async fn read_config(path: &Path) -> Result<LocalConfig> {
    match tokio::fs::read_to_string(path).await {
        Ok(raw) => toml::from_str(&raw).context("config file is not valid TOML"),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(LocalConfig::default()),
        Err(err) => Err(err).context("cannot read config file"),
    }
}

async fn write_config(path: &Path, config: &LocalConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("cannot create config directory")?;
        }
    }
    let raw = toml::to_string_pretty(config).context("cannot encode config")?;
    tokio::fs::write(path, raw)
        .await
        .context("cannot write config file")?;
    Ok(())
}

/// The active token, if one was generated before.
pub async fn active_token(config_path: &Path) -> Result<Option<String>> {
    Ok(read_config(config_path).await?.token)
}

/// Generate, persist and return a fresh token.
pub async fn fresh_token(config_path: &Path) -> Result<String> {
    let token = generate_token()?;
    write_config(
        config_path,
        &LocalConfig {
            token: Some(token.clone()),
        },
    )
    .await?;
    Ok(token)
}

/// Return the stored token, generating one on first use.
pub async fn ensure_token(config_path: &Path) -> Result<String> {
    match active_token(config_path).await? {
        Some(token) => Ok(token),
        None => fresh_token(config_path).await,
    }
}
