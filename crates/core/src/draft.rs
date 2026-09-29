//! Tournament draft picks and bans per side.
//!
//! Draft shapes change between events, so picks are preserved untyped.

use std::collections::HashMap;

use serde_json::Value;

use crate::keys::{PlayerSlot, Side};

/// Draft block grouped by side.
pub type Draft = HashMap<Side, HashMap<PlayerSlot, Value>>;
