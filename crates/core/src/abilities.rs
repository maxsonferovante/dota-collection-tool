//! Ability levels and cooldowns, playing or spectating.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::keys::{AbilitySlot, PlayerSlot, Side, is_side_grouping};
use crate::support::from_object_map;

/// One ability entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ability {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub level: u8,
    #[serde(default)]
    pub can_cast: bool,
    #[serde(default)]
    pub passive: bool,
    #[serde(default)]
    pub ability_active: bool,
    #[serde(default)]
    pub cooldown: u16,
    #[serde(default)]
    pub ultimate: bool,
    #[serde(default, flatten)]
    pub extra: HashMap<String, Value>,
}

/// All abilities of one hero, keyed by `abilityN`.
pub type AbilitySet = HashMap<AbilitySlot, Ability>;

/// Ability block in either camera mode.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum GameAbilities {
    Playing(AbilitySet),
    Spectating(HashMap<Side, HashMap<PlayerSlot, AbilitySet>>),
}

impl<'de> Deserialize<'de> for GameAbilities {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = HashMap::<String, Value>::deserialize(deserializer)?;
        if is_side_grouping(raw.keys()) {
            let grouped: HashMap<Side, HashMap<PlayerSlot, AbilitySet>> =
                from_object_map(raw).map_err(serde::de::Error::custom)?;
            Ok(GameAbilities::Spectating(grouped))
        } else {
            let single: AbilitySet = from_object_map(raw).map_err(serde::de::Error::custom)?;
            Ok(GameAbilities::Playing(single))
        }
    }
}
