//! Spectator draft model, including the pick0/ban0 wire-format transformation.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DraftHero {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamDraft {
    #[serde(default, rename = "home_team")]
    pub is_home_team: bool,
    #[serde(default)]
    pub picks: Vec<DraftHero>,
    #[serde(default)]
    pub bans: Vec<DraftHero>,
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Draft {
    pub active_team: i32,
    pub is_pick: bool,
    pub active_team_time_remaining: i32,
    pub radiant_bonus_time: i32,
    pub dire_bonus_time: i32,
    pub radiant: TeamDraft,
    pub dire: TeamDraft,
    pub extra: BTreeMap<String, Value>,
}

impl<'de> Deserialize<'de> for Draft {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut root = BTreeMap::<String, Value>::deserialize(deserializer)?;
        let active_team = take_i32(&mut root, "activeteam");
        let is_pick = take_bool(&mut root, "pick");
        let active_team_time_remaining = take_i32(&mut root, "activeteam_time_remaining");
        let radiant_bonus_time = take_i32(&mut root, "radiant_bonus_time");
        let dire_bonus_time = take_i32(&mut root, "dire_bonus_time");
        let radiant = take_team(&mut root, "team2");
        let dire = take_team(&mut root, "team3");
        Ok(Self {
            active_team,
            is_pick,
            active_team_time_remaining,
            radiant_bonus_time,
            dire_bonus_time,
            radiant,
            dire,
            extra: root,
        })
    }
}

fn take_i32(root: &mut BTreeMap<String, Value>, key: &str) -> i32 {
    root.remove(key)
        .and_then(|v| v.as_i64())
        .unwrap_or_default() as i32
}
fn take_bool(root: &mut BTreeMap<String, Value>, key: &str) -> bool {
    root.remove(key)
        .and_then(|v| v.as_bool())
        .unwrap_or_default()
}
fn take_team(root: &mut BTreeMap<String, Value>, key: &str) -> TeamDraft {
    let Some(Value::Object(team)) = root.remove(key) else {
        return TeamDraft {
            is_home_team: false,
            picks: Vec::new(),
            bans: Vec::new(),
            extra: BTreeMap::new(),
        };
    };
    let mut team = team.into_iter().collect::<BTreeMap<_, _>>();
    let is_home_team = team
        .remove("home_team")
        .and_then(|v| v.as_bool())
        .unwrap_or_default();
    let mut picks = Vec::new();
    let mut bans = Vec::new();
    let mut extra = BTreeMap::new();
    let fields = team.keys().cloned().collect::<Vec<_>>();
    for field in fields {
        if field.ends_with("_class") {
            let id_key = field.replace("_class", "_id");
            if team.contains_key(&id_key) {
                continue;
            }
        }
        let parsed = ["pick", "ban"].iter().find_map(|kind| {
            field
                .strip_prefix(kind)
                .and_then(|rest| rest.strip_suffix("_id"))
                .filter(|index| !index.is_empty() && index.chars().all(|c| c.is_ascii_digit()))
                .map(|index| (*kind, index.to_owned()))
        });
        if let Some((kind, index)) = parsed {
            let id = team
                .remove(&field)
                .and_then(|v| v.as_i64())
                .unwrap_or_default() as i32;
            let class_key = format!("{kind}{index}_class");
            let name = team
                .remove(&class_key)
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default();
            let hero = DraftHero { id, name };
            match kind {
                "pick" => picks.push(hero),
                "ban" => bans.push(hero),
                _ => {}
            }
            continue;
        }
        if let Some(value) = team.remove(&field) {
            extra.insert(field, value);
        }
    }
    TeamDraft {
        is_home_team,
        picks,
        bans,
        extra,
    }
}
