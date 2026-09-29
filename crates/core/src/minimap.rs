//! Minimap marks (wards, pings, unit blips). Spectator-only.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One mark on the minimap. Shapes vary, so known coordinates are typed
/// and everything else is preserved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinimapMark {
    #[serde(default)]
    pub xpos: Option<i32>,
    #[serde(default)]
    pub ypos: Option<i32>,
    #[serde(default, flatten)]
    pub rest: HashMap<String, Value>,
}

/// Minimap block keyed by mark id.
pub type Minimap = HashMap<String, MinimapMark>;
