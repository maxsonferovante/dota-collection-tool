//! Tower, barracks and ancient health per side.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::keys::Side;

/// Health snapshot of one structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Structure {
    #[serde(default)]
    pub health: u32,
    #[serde(default)]
    pub max_health: u32,
}

/// Structures grouped by side, keyed by internal structure name.
pub type Buildings = HashMap<Side, HashMap<String, Structure>>;
