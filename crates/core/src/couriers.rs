//! Courier snapshots keyed by courier id. Spectator-only.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One courier. Vitals are typed where stable; upgrades, cargo and any
/// future fields are preserved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Courier {
    #[serde(default)]
    pub health: Option<i64>,
    #[serde(default)]
    pub max_health: Option<i64>,
    #[serde(default)]
    pub alive: Option<bool>,
    #[serde(default, flatten)]
    pub rest: HashMap<String, Value>,
}

/// Courier block keyed by courier id.
pub type Couriers = HashMap<String, Courier>;
