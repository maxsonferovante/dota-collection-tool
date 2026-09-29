//! Shared key types for dynamic protocol maps.
//!
//! The protocol addresses repeated entries by string keys (`playerN`,
//! `abilityN`, `slotN` …) and groups spectator data under side keys
//! (`radiant`, `dire`, `team2`, `team3`). These types parse those keys.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

/// Which side of the match a value belongs to.
///
/// Spectator payloads group players under `team2` (Radiant) and `team3`
/// (Dire); regular payloads use the side names directly.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum Side {
    Radiant,
    Dire,
    #[default]
    None,
    Other(String),
}

impl<'de> Deserialize<'de> for Side {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(match raw.as_str() {
            "radiant" | "team2" => Side::Radiant,
            "dire" | "team3" => Side::Dire,
            "none" => Side::None,
            _ => Side::Other(raw),
        })
    }
}

impl Serialize for Side {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Side::Radiant => serializer.serialize_str("radiant"),
            Side::Dire => serializer.serialize_str("dire"),
            Side::None => serializer.serialize_str("none"),
            Side::Other(raw) => serializer.serialize_str(raw),
        }
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Radiant => write!(f, "radiant"),
            Side::Dire => write!(f, "dire"),
            Side::None => write!(f, "none"),
            Side::Other(raw) => write!(f, "{raw}"),
        }
    }
}

/// True when every key in a map is a side key, i.e. the map holds a
/// spectator grouping rather than a single entry.
pub fn is_side_grouping(keys: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
    keys.into_iter().all(|key| {
        matches!(
            key.as_ref(),
            "radiant" | "dire" | "team2" | "team3" | "none"
        )
    })
}

/// A `playerN` slot inside spectator groupings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerSlot(pub u8);

impl<'de> Deserialize<'de> for PlayerSlot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.strip_prefix("player")
            .and_then(|rest| rest.parse::<u8>().ok())
            .map(PlayerSlot)
            .ok_or_else(|| D::Error::custom(format!("bad player slot: {raw}")))
    }
}

impl Serialize for PlayerSlot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("player{}", self.0))
    }
}

/// An `abilityN` slot inside ability sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AbilitySlot(pub u8);

impl<'de> Deserialize<'de> for AbilitySlot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.strip_prefix("ability")
            .and_then(|rest| rest.parse::<u8>().ok())
            .map(AbilitySlot)
            .ok_or_else(|| D::Error::custom(format!("bad ability slot: {raw}")))
    }
}

impl Serialize for AbilitySlot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("ability{}", self.0))
    }
}

/// An inventory position: `slotN`, `stashN`, `teleportN` or `neutralN`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemSlot {
    Slot(u8),
    Stash(u8),
    Teleport,
    Neutral,
    Other(String),
}

impl<'de> Deserialize<'de> for ItemSlot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        if let Some(rest) = raw.strip_prefix("slot") {
            if let Ok(index) = rest.parse::<u8>() {
                return Ok(ItemSlot::Slot(index));
            }
        }
        if let Some(rest) = raw.strip_prefix("stash") {
            if let Ok(index) = rest.parse::<u8>() {
                return Ok(ItemSlot::Stash(index));
            }
        }
        if raw.starts_with("teleport") {
            return Ok(ItemSlot::Teleport);
        }
        if raw.starts_with("neutral") {
            return Ok(ItemSlot::Neutral);
        }
        Ok(ItemSlot::Other(raw))
    }
}

impl Serialize for ItemSlot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ItemSlot::Slot(index) => serializer.serialize_str(&format!("slot{index}")),
            ItemSlot::Stash(index) => serializer.serialize_str(&format!("stash{index}")),
            ItemSlot::Teleport => serializer.serialize_str("teleport0"),
            ItemSlot::Neutral => serializer.serialize_str("neutral0"),
            ItemSlot::Other(raw) => serializer.serialize_str(raw),
        }
    }
}
