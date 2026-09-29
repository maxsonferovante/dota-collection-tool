//! Equipped cosmetic item ids, playing or spectating.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::keys::{PlayerSlot, Side, is_side_grouping};
use crate::support::from_object_map;

/// Wearable slots of one player, keyed by slot name.
pub type WearableSet = HashMap<String, u32>;

/// Wearable block in either camera mode.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Wearables {
    Playing(WearableSet),
    Spectating(HashMap<Side, HashMap<PlayerSlot, WearableSet>>),
}

impl<'de> Deserialize<'de> for Wearables {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = HashMap::<String, Value>::deserialize(deserializer)?;
        if is_side_grouping(raw.keys()) {
            let grouped: HashMap<Side, HashMap<PlayerSlot, WearableSet>> =
                from_object_map(raw).map_err(serde::de::Error::custom)?;
            Ok(Wearables::Spectating(grouped))
        } else {
            let single: WearableSet = from_object_map(raw).map_err(serde::de::Error::custom)?;
            Ok(Wearables::Playing(single))
        }
    }
}
