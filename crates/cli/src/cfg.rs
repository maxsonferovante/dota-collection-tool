//! Game config writer: renders and installs the integration file.
//!
//! The writer owns its KeyValues output: quoted keys, one entry per line,
//! `//` comments. Names are restricted to keep file names safe.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// User-facing GSI collection profiles.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    Economical,
    #[default]
    Balanced,
    LowLatency,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProfileSettings {
    pub buffer: f32,
    pub throttle: f32,
    pub heartbeat: f32,
}

impl Profile {
    pub const SELECTABLE: [Self; 3] = [Self::Economical, Self::Balanced, Self::LowLatency];

    pub fn settings(self) -> Option<ProfileSettings> {
        match self {
            Self::Economical => Some(ProfileSettings {
                buffer: 0.20,
                throttle: 0.20,
                heartbeat: 30.0,
            }),
            Self::Balanced => Some(ProfileSettings {
                buffer: 0.10,
                throttle: 0.10,
                heartbeat: 30.0,
            }),
            Self::LowLatency => Some(ProfileSettings {
                buffer: 0.02,
                throttle: 0.05,
                heartbeat: 15.0,
            }),
            Self::Custom => None,
        }
    }
}

/// Config names may only carry these chars (the file name embeds the name).
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|char| char.is_ascii_alphanumeric() || char == '-' || char == '_')
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Render the full config: endpoint, send-rate knobs, all data blocks on,
/// plus the auth token.
pub fn render(name: &str, uri: &str, token: &str) -> String {
    let mut out = format!("\"{} Integration Configuration\"\n{{\n", escape(name));
    out.push_str(&format!("    \"uri\"       \"{}\"\n", escape(uri)));
    out.push_str("    \"timeout\"   \"5.0\"\n");
    out.push_str("    \"buffer\"    \"0.1\"\n");
    out.push_str("    \"throttle\"  \"0.1\"\n");
    out.push_str("    \"heartbeat\" \"30.0\"\n");
    out.push_str("    \"data\"\n    {\n");
    for block in [
        "auth",
        "provider",
        "map",
        "player",
        "hero",
        "abilities",
        "items",
        "events",
        "buildings",
        "league",
        "draft",
        "wearables",
        "minimap",
        "roshan",
        "couriers",
        "neutralitems",
    ] {
        out.push_str(&format!("        \"{block}\"      \"1\"\n"));
    }
    out.push_str("    }\n");
    out.push_str("    \"auth\"\n    {\n");
    out.push_str(&format!("        \"token\"     \"{}\"\n", escape(token)));
    out.push_str("    }\n}\n");
    out
}

/// Write the file into `dir`, refusing to overwrite unless `force` backs
/// the existing file up as `.bak` first. Returns the written path.
pub async fn install(dir: &Path, name: &str, content: &str, force: bool) -> Result<PathBuf> {
    if !valid_name(name) {
        bail!("invalid config name {name:?}: use letters, digits, '-' or '_'");
    }
    tokio::fs::create_dir_all(dir)
        .await
        .with_context(|| format!("cannot create {}", dir.display()))?;
    let path = dir.join(format!("gamestate_integration_{name}.cfg"));
    if tokio::fs::try_exists(&path)
        .await
        .context("cannot probe existing config")?
    {
        if !force {
            bail!(
                "config already exists at {} (rerun with --force to back it up)",
                path.display()
            );
        }
        let backup = path.with_extension("cfg.bak");
        tokio::fs::rename(&path, &backup)
            .await
            .with_context(|| format!("cannot back up to {}", backup.display()))?;
    }
    tokio::fs::write(&path, content)
        .await
        .with_context(|| format!("cannot write {}", path.display()))?;
    Ok(path)
}
