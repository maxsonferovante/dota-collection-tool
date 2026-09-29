//! Hero vitals and state, playing or spectating.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::keys::{PlayerSlot, Side, is_side_grouping};
use crate::support::from_object_map;

fn missing_hero() -> i16 {
    -1
}

/// Vitals for one hero. `-1` id means no hero picked yet. Talent picks and
/// any future fields land in `extra` instead of being dropped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeroInfo {
    #[serde(default)]
    pub xpos: Option<i32>,
    #[serde(default)]
    pub ypos: Option<i32>,
    #[serde(default = "missing_hero")]
    pub id: i16,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub level: Option<u8>,
    #[serde(default)]
    pub xp: Option<u32>,
    #[serde(default)]
    pub alive: Option<bool>,
    #[serde(default)]
    pub respawn_seconds: Option<u32>,
    #[serde(default)]
    pub buyback_cost: Option<u32>,
    #[serde(default)]
    pub buyback_cooldown: Option<u32>,
    #[serde(default)]
    pub health: Option<u32>,
    #[serde(default)]
    pub max_health: Option<u32>,
    #[serde(default)]
    pub health_percent: Option<u8>,
    #[serde(default)]
    pub mana: Option<u32>,
    #[serde(default)]
    pub max_mana: Option<u32>,
    #[serde(default)]
    pub mana_percent: Option<u8>,
    #[serde(default)]
    pub silenced: bool,
    #[serde(default)]
    pub stunned: bool,
    #[serde(default)]
    pub disarmed: bool,
    #[serde(default)]
    pub magicimmune: bool,
    #[serde(default)]
    pub hexed: bool,
    #[serde(default)]
    pub muted: bool,
    #[serde(rename = "break", default)]
    pub has_break: Option<bool>,
    #[serde(default)]
    pub smoked: Option<bool>,
    #[serde(default)]
    pub selected_unit: Option<String>,
    #[serde(default, flatten)]
    pub extra: HashMap<String, Value>,
}

/// Hero block in either camera mode.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Heroes {
    Playing(HeroInfo),
    Spectating(HashMap<Side, HashMap<PlayerSlot, HeroInfo>>),
}

impl<'de> Deserialize<'de> for Heroes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = HashMap::<String, Value>::deserialize(deserializer)?;
        if is_side_grouping(raw.keys()) {
            let grouped: HashMap<Side, HashMap<PlayerSlot, HeroInfo>> =
                from_object_map(raw).map_err(serde::de::Error::custom)?;
            Ok(Heroes::Spectating(grouped))
        } else {
            let single: HeroInfo = from_object_map(raw).map_err(serde::de::Error::custom)?;
            Ok(Heroes::Playing(single))
        }
    }
}
