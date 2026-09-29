//! Player economy and scoreboard state, playing or spectating.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::keys::{PlayerSlot, Side, is_side_grouping};
use crate::support::from_object_map;

/// What the reported player is doing.
#[derive(Debug, Clone, PartialEq)]
pub enum Activity {
    Menu,
    Playing,
    Other(String),
}

impl<'de> Deserialize<'de> for Activity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(match raw.as_str() {
            "menu" => Activity::Menu,
            "playing" => Activity::Playing,
            _ => Activity::Other(raw),
        })
    }
}

impl Serialize for Activity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Activity::Menu => serializer.serialize_str("menu"),
            Activity::Playing => serializer.serialize_str("playing"),
            Activity::Other(raw) => serializer.serialize_str(raw),
        }
    }
}

/// Scoreboard plus economy for one player.
///
/// Core combat identity fields are required; everything else defaults so
/// older or newer payloads still parse. Spectator-only extras land in
/// `extra` instead of being dropped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub steamid: String,
    #[serde(default)]
    pub name: String,
    pub activity: Activity,
    #[serde(default)]
    pub kills: u16,
    #[serde(default)]
    pub deaths: u16,
    #[serde(default)]
    pub assists: u16,
    #[serde(default)]
    pub last_hits: u16,
    #[serde(default)]
    pub denies: u16,
    #[serde(default)]
    pub kill_streak: u16,
    #[serde(default)]
    pub kill_list: HashMap<String, u32>,
    #[serde(default)]
    pub commands_issued: u32,
    #[serde(alias = "team_name", default)]
    pub side_name: Side,
    #[serde(default)]
    pub gold: u32,
    #[serde(default)]
    pub gold_reliable: u32,
    #[serde(default)]
    pub gold_unreliable: u32,
    #[serde(default)]
    pub gold_from_hero_kills: u32,
    #[serde(default)]
    pub gold_from_creep_kills: u32,
    #[serde(default)]
    pub gold_from_income: u32,
    #[serde(default)]
    pub gold_from_shared: u32,
    #[serde(default)]
    pub net_worth: Option<u32>,
    #[serde(default)]
    pub gpm: u32,
    #[serde(default)]
    pub xpm: u32,
    /// Spectator-only extras (damage, wards, runes …), preserved as-is.
    #[serde(default, flatten)]
    pub extra: HashMap<String, Value>,
}

/// Player block in either camera mode.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Players {
    Playing(Box<PlayerInfo>),
    Spectating(HashMap<Side, HashMap<PlayerSlot, PlayerInfo>>),
}

impl<'de> Deserialize<'de> for Players {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = HashMap::<String, Value>::deserialize(deserializer)?;
        if is_side_grouping(raw.keys()) {
            let grouped: HashMap<Side, HashMap<PlayerSlot, PlayerInfo>> =
                from_object_map(raw).map_err(serde::de::Error::custom)?;
            Ok(Players::Spectating(grouped))
        } else {
            let single: PlayerInfo = from_object_map(raw).map_err(serde::de::Error::custom)?;
            Ok(Players::Playing(Box::new(single)))
        }
    }
}
