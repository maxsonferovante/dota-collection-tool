//! Tournament metadata. Only present in league lobbies.
//!
//! Fields vary by event, so the well-known core is typed and the rest is
//! preserved as-is.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// League and series metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct League {
    #[serde(alias = "league_id", default)]
    pub id: Option<Value>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default, flatten)]
    pub rest: HashMap<String, Value>,
}
