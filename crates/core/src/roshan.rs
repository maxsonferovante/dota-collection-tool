//! Roshan pit snapshot. Spectator-only.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Roshan health plus drops; drop shapes vary so details are preserved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Roshan {
    #[serde(default)]
    pub health: Option<i64>,
    #[serde(default)]
    pub max_health: Option<i64>,
    #[serde(default, flatten)]
    pub rest: HashMap<String, Value>,
}
