//! Inventory, stash, teleport and neutral slots, playing or spectating.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::keys::{ItemSlot, PlayerSlot, Side, is_side_grouping};
use crate::support::from_object_map;

/// One item entry. `name` is `"empty"` for vacant positions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub purchaser: Option<u32>,
    #[serde(default)]
    pub can_cast: Option<bool>,
    #[serde(default)]
    pub cooldown: Option<u32>,
    #[serde(default)]
    pub passive: Option<bool>,
    #[serde(default)]
    pub charges: Option<u32>,
    #[serde(default, flatten)]
    pub extra: HashMap<String, Value>,
}

impl Item {
    /// True when the position holds no item.
    pub fn is_empty(&self) -> bool {
        self.name == "empty"
    }
}

/// All item positions of one hero.
pub type ItemSet = HashMap<ItemSlot, Item>;

/// Item block in either camera mode.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum GameItems {
    Playing(ItemSet),
    Spectating(HashMap<Side, HashMap<PlayerSlot, ItemSet>>),
}

impl<'de> Deserialize<'de> for GameItems {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = HashMap::<String, Value>::deserialize(deserializer)?;
        if is_side_grouping(raw.keys()) {
            let grouped: HashMap<Side, HashMap<PlayerSlot, ItemSet>> =
                from_object_map(raw).map_err(serde::de::Error::custom)?;
            Ok(GameItems::Spectating(grouped))
        } else {
            let single: ItemSet = from_object_map(raw).map_err(serde::de::Error::custom)?;
            Ok(GameItems::Playing(single))
        }
    }
}
