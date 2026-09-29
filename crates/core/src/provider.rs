//! Reporter identity plus the echoed auth token.

use serde::{Deserialize, Serialize};

/// Who sent the frame: always the game client itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provider {
    /// Product name, e.g. `Dota 2`.
    pub name: String,
    /// Application id (570 for Dota 2).
    #[serde(alias = "appid", default)]
    pub app_id: u32,
    /// Client build version.
    #[serde(default)]
    pub version: u32,
    /// Client timestamp of the report.
    #[serde(default)]
    pub timestamp: u32,
}

/// Authentication echoed back from the config file. Never validated here;
/// the server layer compares it against the active token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Auth {
    pub token: Option<String>,
}
