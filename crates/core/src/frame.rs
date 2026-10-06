//! One accepted game POST: the full frame plus its delta envelope.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use serde_json::Value;

use crate::abilities::GameAbilities;
use crate::buildings::Buildings;
use crate::couriers::Couriers;
use crate::draft::Draft;
use crate::error::CoreError;
use crate::events::GameEvent;
use crate::hero::Heroes;
use crate::items::GameItems;
use crate::league::League;
use crate::map::Map;
use crate::minimap::Minimap;
use crate::neutralitems::NeutralItems;
use crate::player::Players;
use crate::provider::{Auth, Provider};
use crate::roshan::Roshan;
use crate::support::{empty_as_none, empty_seq_as_none};
use crate::wearables::Wearables;

/// A complete snapshot of one game report.
///
/// Every block is optional: the game only sends what the config enables,
/// and empty objects count as absent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    pub provider: Provider,
    #[serde(default)]
    pub auth: Option<Auth>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub map: Option<Map>,
    #[serde(alias = "player", default, deserialize_with = "empty_as_none")]
    pub players: Option<Players>,
    #[serde(alias = "hero", default, deserialize_with = "empty_as_none")]
    pub heroes: Option<Heroes>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub abilities: Option<GameAbilities>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub items: Option<GameItems>,
    #[serde(default, deserialize_with = "empty_seq_as_none")]
    pub events: Option<Vec<GameEvent>>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub buildings: Option<Buildings>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub league: Option<League>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub draft: Option<Draft>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub wearables: Option<Wearables>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub minimap: Option<Minimap>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub roshan: Option<Roshan>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub couriers: Option<Couriers>,
    #[serde(default, deserialize_with = "empty_as_none")]
    pub neutralitems: Option<NeutralItems>,
    /// Previous values for whatever changed since the last report.
    #[serde(default)]
    pub previously: Option<Value>,
    /// Root fields that are new since the last report.
    #[serde(default)]
    pub added: Option<Value>,
    /// Root-level fields introduced by newer GSI versions.
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Frame {
    /// Parse one raw POST body. Unknown fields never fail the parse.
    pub fn from_slice(body: &[u8]) -> Result<Self, CoreError> {
        serde_json::from_slice(body).map_err(CoreError::InvalidJson)
    }

    /// Match id when the frame carries one, for indexing.
    pub fn match_id(&self) -> Option<&str> {
        self.map.as_ref().map(|map| map.match_id.as_str())
    }
}
