//! Neutral item tiers and team findings. Spectator-only.
//!
//! Tier layouts change between patches, so the block is captured
//! structurally and typed gradually.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Neutral-item block preserved as reported.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeutralItems {
    #[serde(default, flatten)]
    pub tiers: HashMap<String, Value>,
}
