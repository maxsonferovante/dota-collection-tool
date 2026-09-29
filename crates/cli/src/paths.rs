//! Local file locations: database, config, overflow spill and PID file.
//!
//! Everything lives side by side so a custom `--db` keeps its companions
//! next to it; otherwise the platform data directory is used.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Resolved local paths for one invocation.
#[derive(Debug, Clone)]
pub struct Paths {
    /// SQLite database file.
    pub db: PathBuf,
    /// Local TOML config (currently: the auth token).
    pub config: PathBuf,
    /// Full-queue spill file (newline-delimited JSON).
    pub overflow: PathBuf,
    /// Detached server PID file.
    pub pid: PathBuf,
    /// Detached server log file.
    pub log: PathBuf,
}

fn data_dir() -> Result<PathBuf> {
    dirs::data_dir()
        .map(|dir| dir.join("dct"))
        .context("cannot locate the platform data directory (set --db explicitly)")
}

/// Resolve companions next to an explicit database path.
pub fn beside(db: &Path) -> Paths {
    let parent = db.parent().filter(|dir| !dir.as_os_str().is_empty());
    let stem = |name: &str| match parent {
        Some(dir) => dir.join(name),
        None => PathBuf::from(name),
    };
    Paths {
        db: db.to_owned(),
        config: stem("config.toml"),
        overflow: stem("overflow.jsonl"),
        pid: stem("server.pid"),
        log: stem("server.log"),
    }
}

/// Resolve paths: explicit `--db` wins, otherwise the data directory.
pub fn resolve(db_override: Option<PathBuf>) -> Result<Paths> {
    match db_override {
        Some(db) => Ok(beside(&db)),
        None => {
            let dir = data_dir()?;
            Ok(Paths {
                db: dir.join("collect.db"),
                config: dir.join("config.toml"),
                overflow: dir.join("overflow.jsonl"),
                pid: dir.join("server.pid"),
                log: dir.join("server.log"),
            })
        }
    }
}
