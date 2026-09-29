//! Small deserialization helpers shared by every block.

use std::collections::HashMap;
use std::fmt::Debug;

use serde::{Deserialize, Deserializer, de::Error as _};
use serde_json::{Map, Value};

/// Deserialize an optional block, treating a missing or empty object as
/// absent. The game sends `{}` for blocks with nothing to report.
pub fn empty_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: serde::de::DeserializeOwned + Debug,
{
    let raw = Option::<Map<String, Value>>::deserialize(deserializer)?;
    match raw {
        None => Ok(None),
        Some(entries) if entries.is_empty() => Ok(None),
        Some(entries) => serde_json::from_value(Value::Object(entries))
            .map(Some)
            .map_err(D::Error::custom),
    }
}

/// Deserialize an optional sequence, treating a missing or empty array as
/// absent.
pub fn empty_seq_as_none<'de, D, T>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    let raw = Option::<Vec<T>>::deserialize(deserializer)?;
    match raw {
        None => Ok(None),
        Some(entries) if entries.is_empty() => Ok(None),
        Some(entries) => Ok(Some(entries)),
    }
}
/// Rebuild a typed value from a string-keyed object map.
pub fn from_object_map<T>(entries: HashMap<String, Value>) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let object: Map<String, Value> = entries.into_iter().collect();
    serde_json::from_value(Value::Object(object)).map_err(|err| err.to_string())
}
