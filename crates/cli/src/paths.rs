//! Local file locations: database, config, overflow spill and PID file.
//!
//! Everything lives side by side so a custom `--db` keeps its companions
//! next to it; otherwise the directory holding this executable is used, so a
//! portable install (including the Windows `.exe`) keeps its state next to
//! the binary. When the executable directory cannot be determined, the
//! platform data directory is used as a fallback.

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

/// Directory holding the running executable (portable default).
fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()?
        .parent()
        .map(Path::to_path_buf)
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

/// Resolve paths: explicit `--db` wins, otherwise the executable directory
/// (falling back to the platform data directory when undetectable).
pub fn resolve(db_override: Option<PathBuf>) -> Result<Paths> {
    match db_override {
        Some(db) => Ok(beside(&db)),
        None => {
            let dir = exe_dir().map(Ok).unwrap_or_else(data_dir)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_paths_live_beside_the_executable() {
        let exe_dir = std::env::current_exe()
            .expect("current exe")
            .parent()
            .expect("exe parent")
            .to_path_buf();
        let paths = resolve(None).expect("resolve");
        assert_eq!(paths.db, exe_dir.join("collect.db"));
        assert_eq!(paths.config, exe_dir.join("config.toml"));
        assert_eq!(paths.overflow, exe_dir.join("overflow.jsonl"));
        assert_eq!(paths.pid, exe_dir.join("server.pid"));
        assert_eq!(paths.log, exe_dir.join("server.log"));
    }

    #[test]
    fn explicit_db_keeps_companions_beside_it() {
        let paths = beside(Path::new("/tmp/x/collect.db"));
        assert_eq!(paths.db, PathBuf::from("/tmp/x/collect.db"));
        assert_eq!(paths.config, PathBuf::from("/tmp/x/config.toml"));
    }
}
