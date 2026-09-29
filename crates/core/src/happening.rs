//! Happenings derived by comparing consecutive frames.
//!
//! Only increases produce events, so replays of the same snapshot stay
//! silent. Mode switches (playing to spectating) yield nothing rather
//! than noise.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::abilities::GameAbilities;
use crate::frame::Frame;
use crate::keys::{AbilitySlot, PlayerSlot};
use crate::player::{PlayerInfo, Players};

/// The kind of a derived happening.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HappeningKind {
    Kill,
    Death,
    Assist,
    DayStarted,
    NightStarted,
    AbilityLeveled,
    AbilityCooldown,
    AbilityReady,
}

impl HappeningKind {
    /// Stable string used for storage and filtering.
    pub fn as_str(&self) -> &'static str {
        match self {
            HappeningKind::Kill => "kill",
            HappeningKind::Death => "death",
            HappeningKind::Assist => "assist",
            HappeningKind::DayStarted => "day_started",
            HappeningKind::NightStarted => "night_started",
            HappeningKind::AbilityLeveled => "ability_leveled",
            HappeningKind::AbilityCooldown => "ability_cooldown",
            HappeningKind::AbilityReady => "ability_ready",
        }
    }
}

/// One derived event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Happening {
    pub kind: HappeningKind,
    pub actor: Option<String>,
    pub detail: Value,
}

fn kill_events(previous: &PlayerInfo, current: &PlayerInfo) -> Vec<Happening> {
    let mut out = Vec::new();
    if previous.kills < current.kills {
        out.push(Happening {
            kind: HappeningKind::Kill,
            actor: Some(current.name.clone()),
            detail: json!({"kills": current.kills, "streak": current.kill_streak}),
        });
    }
    if previous.deaths < current.deaths {
        out.push(Happening {
            kind: HappeningKind::Death,
            actor: Some(current.name.clone()),
            detail: json!({"deaths": current.deaths}),
        });
    }
    if previous.assists < current.assists {
        out.push(Happening {
            kind: HappeningKind::Assist,
            actor: Some(current.name.clone()),
            detail: json!({"assists": current.assists}),
        });
    }
    out
}

fn player_pairs<'a>(
    previous: &'a Players,
    current: &'a Players,
) -> Vec<(&'a PlayerInfo, &'a PlayerInfo)> {
    match (previous, current) {
        (Players::Playing(before), Players::Playing(after)) => {
            vec![(before.as_ref(), after.as_ref())]
        }
        (Players::Spectating(before), Players::Spectating(after)) => {
            let mut pairs = Vec::new();
            for (side, before_players) in before {
                let Some(after_players) = after.get(side) else {
                    continue;
                };
                for (slot, before_info) in before_players {
                    if let Some(after_info) = after_players.get(slot) {
                        pairs.push((before_info, after_info));
                    }
                }
            }
            pairs
        }
        _ => Vec::new(),
    }
}

fn ability_events(
    slot: AbilitySlot,
    player: Option<PlayerSlot>,
    was: &crate::abilities::Ability,
    is: &crate::abilities::Ability,
    actor: Option<String>,
) -> Vec<Happening> {
    let mut out = Vec::new();
    let slot_id = slot.0;
    let player_id = player.map(|slot| slot.0);
    if was.level < is.level {
        out.push(Happening {
            kind: HappeningKind::AbilityLeveled,
            actor: actor.clone(),
            detail: json!({
                "ability": is.name, "slot": slot_id, "player": player_id,
                "level": is.level, "ultimate": is.ultimate,
            }),
        });
    }
    if was.cooldown == 0 && is.cooldown > 0 {
        out.push(Happening {
            kind: HappeningKind::AbilityCooldown,
            actor: actor.clone(),
            detail: json!({
                "ability": is.name, "slot": slot_id, "player": player_id,
                "remaining": is.cooldown, "ultimate": is.ultimate,
            }),
        });
    }
    if was.cooldown > 0 && is.cooldown == 0 {
        out.push(Happening {
            kind: HappeningKind::AbilityReady,
            actor,
            detail: json!({
                "ability": is.name, "slot": slot_id, "player": player_id,
                "ultimate": is.ultimate,
            }),
        });
    }
    out
}

/// Compare two consecutive frames and emit happenings.
///
/// The first frame of a match (`None` previous) never emits: there is
/// nothing to compare against yet.
pub fn derive_happenings(previous: Option<&Frame>, current: &Frame) -> Vec<Happening> {
    let Some(before) = previous else {
        return Vec::new();
    };
    let mut out = Vec::new();

    if let (Some(before_map), Some(after_map)) = (before.map.as_ref(), current.map.as_ref()) {
        match (before_map.daytime, after_map.daytime) {
            (true, false) => out.push(Happening {
                kind: HappeningKind::NightStarted,
                actor: None,
                detail: json!({"nightstalker": after_map.nightstalker_night}),
            }),
            (false, true) => out.push(Happening {
                kind: HappeningKind::DayStarted,
                actor: None,
                detail: json!({}),
            }),
            _ => {}
        }
    }

    if let (Some(before_players), Some(after_players)) =
        (before.players.as_ref(), current.players.as_ref())
    {
        for (was, is) in player_pairs(before_players, after_players) {
            out.extend(kill_events(was, is));
        }
    }

    if let (Some(before_abilities), Some(after_abilities)) =
        (before.abilities.as_ref(), current.abilities.as_ref())
    {
        out.extend(ability_pairs(before_abilities, after_abilities));
    }

    out
}

fn ability_pairs(before: &GameAbilities, after: &GameAbilities) -> Vec<Happening> {
    match (before, after) {
        (GameAbilities::Playing(was), GameAbilities::Playing(is)) => {
            ability_set_events(None, was, is)
        }
        (GameAbilities::Spectating(was), GameAbilities::Spectating(is)) => {
            let mut out = Vec::new();
            for (side, was_players) in was {
                let Some(is_players) = is.get(side) else {
                    continue;
                };
                for (slot, was_set) in was_players {
                    if let Some(is_set) = is_players.get(slot) {
                        out.extend(ability_set_events(Some(*slot), was_set, is_set));
                    }
                }
            }
            out
        }
        _ => Vec::new(),
    }
}

fn ability_set_events(
    player: Option<PlayerSlot>,
    was: &std::collections::HashMap<AbilitySlot, crate::abilities::Ability>,
    is: &std::collections::HashMap<AbilitySlot, crate::abilities::Ability>,
) -> Vec<Happening> {
    let mut out = Vec::new();
    for (slot, was_ability) in was {
        if let Some(is_ability) = is.get(slot) {
            out.extend(ability_events(*slot, player, was_ability, is_ability, None));
        }
    }
    out
}
