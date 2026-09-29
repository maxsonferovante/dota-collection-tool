//! Discrete gameplay events: couriers, Roshan, Aegis, tips, runes.
//!
//! The game only reports a small fixed set of event types; per-type
//! details ride along in `detail` instead of being dropped.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One gameplay event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameEvent {
    #[serde(default)]
    pub game_time: u32,
    #[serde(alias = "event_type", default)]
    pub kind: String,
    #[serde(default, flatten)]
    pub detail: HashMap<String, Value>,
}
