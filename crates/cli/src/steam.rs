//! Steam library discovery without registry or SDK calls.
//!
//! Probes well-known Steam roots per platform, then scans
//! `libraryfolders.vdf` for libraries holding the Dota app manifest
//! (570). An explicit `--dota-dir` always wins over detection.

use std::path::{Path, PathBuf};

const DOTA_APP_MANIFEST: &str = "appmanifest_570.acf";

fn steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(home) = dirs::home_dir() {
        roots.push(home.join(".steam").join("steam"));
        roots.push(home.join(".local").join("share").join("Steam"));
        roots.push(
            home.join("Library")
                .join("Application Support")
                .join("Steam"),
        );
    }
    #[cfg(windows)]
    {
        roots.push(PathBuf::from("C:\\Program Files (x86)\\Steam"));
        roots.push(PathBuf::from("D:\\Steam"));
    }
    roots
}

fn manifest_present(library: &Path) -> bool {
    library.join("steamapps").join(DOTA_APP_MANIFEST).is_file()
}

/// Extract quoted `path` values from a `libraryfolders.vdf` file.
///
/// The manifest format changed across Steam versions, but library paths
/// always appear as the value of a `"path"` key, so a line scan is enough.
pub fn library_paths(manifest: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for line in manifest.lines() {
        let quoted: Vec<&str> = line.split('"').skip(1).step_by(2).collect();
        let mut pairs = quoted.into_iter();
        while let Some(key) = pairs.next() {
            if key == "path" {
                if let Some(value) = pairs.next() {
                    if !value.is_empty() {
                        paths.push(PathBuf::from(value));
                    }
                }
            }
        }
    }
    paths
}

fn libraries_of(root: &Path) -> Vec<PathBuf> {
    let mut libraries = vec![root.to_owned()];
    let manifest = root.join("steamapps").join("libraryfolders.vdf");
    if let Ok(raw) = std::fs::read_to_string(manifest) {
        libraries.extend(library_paths(&raw));
    }
    libraries
}

/// Locate the Dota install root (the `dota 2 beta` directory).
pub fn find_dota_root() -> Option<PathBuf> {
    find_dota_root_in(&steam_roots())
}

/// Same as [`find_dota_root`], over explicit roots (the test seam that lets
/// fake Steam libraries validate discovery on any platform).
pub fn find_dota_root_in(roots: &[PathBuf]) -> Option<PathBuf> {
    for root in roots {
        for library in libraries_of(root) {
            if !manifest_present(&library) {
                continue;
            }
            for candidate in [
                library.join("steamapps").join("common").join("dota 2 beta"),
                library.join("common").join("dota 2 beta"),
            ] {
                if candidate.join("game").join("dota").is_dir() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

/// The `gamestate_integration` directory below a Dota root.
pub fn integration_dir(dota_root: &Path) -> PathBuf {
    dota_root
        .join("game")
        .join("dota")
        .join("cfg")
        .join("gamestate_integration")
}
