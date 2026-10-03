//! Protocol surface: phases, key types, blocks in both camera modes,
//! and happening derivation across frame pairs.

use dct_core::Frame;
use dct_core::abilities::GameAbilities;
use dct_core::hero::{HeroInfo, Heroes};
use dct_core::items::{GameItems, Item};
use dct_core::keys::{AbilitySlot, ItemSlot, PlayerSlot, Side, is_side_grouping};
use dct_core::map::{GamePhase, Map, RoshanPit};
use dct_core::player::{Activity, Players};
use dct_core::wearables::Wearables;
use dct_core::{HappeningKind, derive_happenings};

const PROVIDER: &str = r#"{"name": "Dota 2", "appid": 570, "version": 47, "timestamp": 1}"#;

fn frame(body: &str) -> Frame {
    Frame::from_slice(body.as_bytes()).expect("test frame must parse")
}

// --- GamePhase -----------------------------------------------------------

#[test]
fn phase_roundtrip_for_every_variant() {
    let cases = [
        ("DOTA_GAMERULES_STATE_DISCONNECT", GamePhase::Disconnected),
        (
            "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS",
            GamePhase::InProgress,
        ),
        (
            "DOTA_GAMERULES_STATE_HERO_SELECTION",
            GamePhase::HeroSelection,
        ),
        ("DOTA_GAMERULES_STATE_INIT", GamePhase::Starting),
        ("DOTA_GAMERULES_STATE_LAST", GamePhase::Ending),
        ("DOTA_GAMERULES_STATE_POST_GAME", GamePhase::PostGame),
        ("DOTA_GAMERULES_STATE_PRE_GAME", GamePhase::PreGame),
        (
            "DOTA_GAMERULES_STATE_STRATEGY_TIME",
            GamePhase::StrategyTime,
        ),
        (
            "DOTA_GAMERULES_STATE_WAIT_FOR_MAP_TO_LOAD",
            GamePhase::WaitingForMap,
        ),
        (
            "DOTA_GAMERULES_STATE_WAIT_FOR_PLAYERS_TO_LOAD",
            GamePhase::WaitingForPlayers,
        ),
        (
            "DOTA_GAMERULES_STATE_CUSTOM_GAME_SETUP",
            GamePhase::CustomGameSetup,
        ),
    ];
    for (raw, expected) in cases {
        let parsed = GamePhase::from(raw.to_owned());
        assert_eq!(parsed, expected);
        let back: String = parsed.into();
        assert_eq!(back, raw);
    }
}

#[test]
fn phase_unknown_roundtrips_and_defaults() {
    let parsed = GamePhase::from(String::from("SOMETHING_NEW"));
    assert_eq!(parsed, GamePhase::Undefined(String::from("SOMETHING_NEW")));
    let back: String = parsed.into();
    assert_eq!(back, "SOMETHING_NEW");
    assert_eq!(GamePhase::default(), GamePhase::Undefined(String::new()));
}

#[test]
fn roshan_pit_roundtrip_for_every_variant() {
    for raw in ["ALIVE", "RESPAWN_BASE", "RESPAWN_VARIABLE"] {
        let parsed = RoshanPit::from(raw.to_owned());
        let back: String = parsed.into();
        assert_eq!(back, raw);
    }
    let unknown = RoshanPit::from(String::from("GONE"));
    assert_eq!(unknown, RoshanPit::Undefined(String::from("GONE")));
    let back: String = unknown.into();
    assert_eq!(back, "GONE");
}

#[test]
fn map_parses_full_snapshot_with_aliases() {
    let map: Map = serde_json::from_value(serde_json::json!({
        "name": "dota", "matchid": "99", "game_time": 600, "clock_time": 600,
        "daytime": true, "nightstalker_night": false,
        "game_state": "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS",
        "paused": false, "win_team": "radiant", "customgamename": "ability draft",
        "ward_purchase_cooldown": 10,
        "radiant_ward_purchase_cooldown": 5, "dire_ward_purchase_cooldown": 7,
        "radiant_score": 5, "dire_score": 3,
        "roshan_state": "ALIVE", "roshan_state_end_time": 120
    }))
    .expect("full map must parse");
    assert_eq!(map.match_id, "99");
    assert_eq!(map.phase, GamePhase::InProgress);
    assert_eq!(map.winning_side, Side::Radiant);
    assert_eq!(map.custom_game_name, "ability draft");
    assert_eq!(map.ward_purchase_cooldown, Some(10));
    assert_eq!(map.radiant_score, Some(5));
    assert_eq!(map.dire_score, Some(3));
    assert_eq!(map.roshan_pit, Some(RoshanPit::Alive));
    assert_eq!(map.roshan_pit_end, Some(120));
}

// --- Side and key types --------------------------------------------------

#[test]
fn side_parses_every_spelling() {
    let parse = |raw: &str| serde_json::from_value::<Side>(serde_json::json!(raw)).unwrap();
    assert_eq!(parse("radiant"), Side::Radiant);
    assert_eq!(parse("team2"), Side::Radiant);
    assert_eq!(parse("dire"), Side::Dire);
    assert_eq!(parse("team3"), Side::Dire);
    assert_eq!(parse("none"), Side::None);
    assert_eq!(
        parse("team1"),
        Side::Other(String::from("team1")),
        "unknown sides are preserved"
    );
}

#[test]
fn side_serializes_and_displays() {
    let cases = [
        (Side::Radiant, "radiant"),
        (Side::Dire, "dire"),
        (Side::None, "none"),
        (Side::Other(String::from("team1")), "team1"),
    ];
    for (side, raw) in cases {
        let json = serde_json::to_value(&side).unwrap();
        assert_eq!(json, serde_json::json!(raw));
        assert_eq!(side.to_string(), raw);
    }
}

#[test]
fn side_grouping_detection() {
    assert!(is_side_grouping([
        "radiant", "team2", "team3", "dire", "none"
    ]));
    assert!(is_side_grouping(Vec::<String>::new()), "empty is vacuous");
    assert!(!is_side_grouping(["radiant", "steamid"]));
}

#[test]
fn player_slot_parses_and_rejects() {
    let slot: PlayerSlot = serde_json::from_value(serde_json::json!("player3")).unwrap();
    assert_eq!(slot, PlayerSlot(3));
    assert!(serde_json::from_value::<PlayerSlot>(serde_json::json!("player")).is_err());
    assert!(serde_json::from_value::<PlayerSlot>(serde_json::json!("hero0")).is_err());
    assert!(serde_json::from_value::<PlayerSlot>(serde_json::json!("player300")).is_err());
    let back = serde_json::to_value(slot).unwrap();
    assert_eq!(back, serde_json::json!("player3"));
}

#[test]
fn ability_slot_parses_and_rejects() {
    let slot: AbilitySlot = serde_json::from_value(serde_json::json!("ability2")).unwrap();
    assert_eq!(slot, AbilitySlot(2));
    assert!(serde_json::from_value::<AbilitySlot>(serde_json::json!("ability")).is_err());
    assert!(serde_json::from_value::<AbilitySlot>(serde_json::json!("slot0")).is_err());
    let back = serde_json::to_value(slot).unwrap();
    assert_eq!(back, serde_json::json!("ability2"));
}

#[test]
fn item_slot_parses_every_shape() {
    let parse = |raw: &str| serde_json::from_value::<ItemSlot>(serde_json::json!(raw)).unwrap();
    assert_eq!(parse("slot0"), ItemSlot::Slot(0));
    assert_eq!(parse("stash3"), ItemSlot::Stash(3));
    assert_eq!(parse("teleport0"), ItemSlot::Teleport);
    assert_eq!(parse("neutral0"), ItemSlot::Neutral);
    assert_eq!(parse("slotX"), ItemSlot::Other(String::from("slotX")));
    assert_eq!(parse("stashX"), ItemSlot::Other(String::from("stashX")));
    assert_eq!(parse("weird"), ItemSlot::Other(String::from("weird")));

    let roundtrip = |slot: ItemSlot, raw: &str| {
        let json = serde_json::to_value(&slot).unwrap();
        assert_eq!(json, serde_json::json!(raw));
    };
    roundtrip(ItemSlot::Slot(1), "slot1");
    roundtrip(ItemSlot::Stash(2), "stash2");
    roundtrip(ItemSlot::Teleport, "teleport0");
    roundtrip(ItemSlot::Neutral, "neutral0");
    roundtrip(ItemSlot::Other(String::from("weird")), "weird");
}

// --- Player / hero / item blocks -----------------------------------------

#[test]
fn activity_roundtrip_for_every_variant() {
    for raw in ["menu", "playing"] {
        let parsed: Activity = serde_json::from_value(serde_json::json!(raw)).unwrap();
        let back = serde_json::to_value(&parsed).unwrap();
        assert_eq!(back, serde_json::json!(raw));
    }
    let other: Activity = serde_json::from_value(serde_json::json!("drafting")).unwrap();
    assert_eq!(other, Activity::Other(String::from("drafting")));
    let back = serde_json::to_value(&other).unwrap();
    assert_eq!(back, serde_json::json!("drafting"));
}

#[test]
fn players_parse_in_both_camera_modes() {
    let single: Players = serde_json::from_value(serde_json::json!({
        "steamid": "1", "activity": "playing", "kills": 1
    }))
    .unwrap();
    assert!(matches!(single, Players::Playing(_)));
    let grouped: Players = serde_json::from_value(serde_json::json!({
        "team2": {"player0": {"steamid": "1", "activity": "playing"}},
        "team3": {"player0": {"steamid": "2", "activity": "menu"}}
    }))
    .unwrap();
    let Players::Spectating(groups) = &grouped else {
        panic!("must be spectating");
    };
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[&Side::Radiant][&PlayerSlot(0)].name, String::new());
    let back = serde_json::to_value(&grouped).unwrap();
    assert!(back.get("Spectating").is_some());
    assert!(back["Spectating"].get("radiant").is_some());
}

#[test]
fn hero_without_pick_reports_missing_id() {
    let hero: HeroInfo = serde_json::from_value(serde_json::json!({})).unwrap();
    assert_eq!(hero.id, -1);
    assert_eq!(hero.name, None);
}

#[test]
fn heroes_parse_in_both_camera_modes() {
    let single: Heroes =
        serde_json::from_value(serde_json::json!({"id": 42, "name": "npc_dota_hero_axe"})).unwrap();
    assert!(matches!(single, Heroes::Playing(_)));
    let grouped: Heroes = serde_json::from_value(serde_json::json!({
        "team2": {"player0": {"id": 42}},
        "team3": {"player0": {}}
    }))
    .unwrap();
    let Heroes::Spectating(groups) = &grouped else {
        panic!("must be spectating");
    };
    assert_eq!(groups[&Side::Dire][&PlayerSlot(0)].id, -1);
}

#[test]
fn item_emptiness_and_spectating() {
    let full: Item = serde_json::from_value(serde_json::json!({"name": "item_blink"})).unwrap();
    assert!(!full.is_empty());
    let vacant: Item = serde_json::from_value(serde_json::json!({"name": "empty"})).unwrap();
    assert!(vacant.is_empty());
    let grouped: GameItems = serde_json::from_value(serde_json::json!({
        "team2": {"player0": {"slot0": {"name": "item_blink"}}}
    }))
    .unwrap();
    assert!(matches!(grouped, GameItems::Spectating(_)));
}

#[test]
fn abilities_parse_in_both_camera_modes() {
    let single: GameAbilities = serde_json::from_value(serde_json::json!({
        "ability0": {"name": "blast", "level": 2}
    }))
    .unwrap();
    assert!(matches!(single, GameAbilities::Playing(_)));
    let grouped: GameAbilities = serde_json::from_value(serde_json::json!({
        "team2": {"player0": {"ability0": {"name": "blast", "level": 1}}}
    }))
    .unwrap();
    assert!(matches!(grouped, GameAbilities::Spectating(_)));
}

#[test]
fn wearables_parse_in_both_camera_modes() {
    let single: Wearables =
        serde_json::from_value(serde_json::json!({"wearable0": 13773})).unwrap();
    assert!(matches!(single, Wearables::Playing(_)));
    let grouped: Wearables = serde_json::from_value(serde_json::json!({
        "team2": {"player0": {"wearable0": 13773}}
    }))
    .unwrap();
    assert!(matches!(grouped, Wearables::Spectating(_)));
}

#[test]
fn missing_blocks_count_as_absent() {
    let idle = frame(&format!(r#"{{"provider": {PROVIDER}}}"#));
    assert_eq!(idle.map, None);
    assert_eq!(idle.players, None);
    assert_eq!(idle.events, None);
    assert_eq!(idle.match_id(), None);
}

#[test]
fn explicit_null_blocks_count_as_absent() {
    let idle = frame(&format!(
        r#"{{"provider": {PROVIDER}, "map": null, "player": null,
            "events": null, "abilities": null}}"#
    ));
    assert_eq!(idle.map, None);
    assert_eq!(idle.players, None);
    assert_eq!(idle.events, None);
    assert_eq!(idle.abilities, None);
}

// --- Happenings ------------------------------------------------------------

#[test]
fn happening_kind_labels_are_stable() {
    let cases = [
        (HappeningKind::Kill, "kill"),
        (HappeningKind::Death, "death"),
        (HappeningKind::Assist, "assist"),
        (HappeningKind::DayStarted, "day_started"),
        (HappeningKind::NightStarted, "night_started"),
        (HappeningKind::AbilityLeveled, "ability_leveled"),
        (HappeningKind::AbilityCooldown, "ability_cooldown"),
        (HappeningKind::AbilityReady, "ability_ready"),
    ];
    for (kind, label) in cases {
        assert_eq!(kind.as_str(), label);
    }
}

#[test]
fn first_frame_never_emits() {
    let current = frame(&format!(
        r#"{{"provider": {PROVIDER},
            "map": {{"daytime": true}},
            "player": {{"steamid": "1", "activity": "playing", "kills": 9}},
            "abilities": {{"ability0": {{"name": "blast", "level": 4}}}}}}"#
    ));
    assert!(derive_happenings(None, &current).is_empty());
}

fn playing_pair(before_player: &str, after_player: &str) -> (Frame, Frame) {
    let before = frame(&format!(
        r#"{{"provider": {PROVIDER},
            "map": {{"daytime": true}},
            "player": {before_player},
            "abilities": {{"ability0": {{"name": "blast", "level": 2, "cooldown": 5}},
                           "ability1": {{"name": "zap", "level": 1, "cooldown": 0}}}}}}"#
    ));
    let after = frame(&format!(
        r#"{{"provider": {PROVIDER},
            "map": {{"daytime": false, "nightstalker_night": true}},
            "player": {after_player},
            "abilities": {{"ability0": {{"name": "blast", "level": 3, "cooldown": 0}},
                           "ability1": {{"name": "zap", "level": 1, "cooldown": 10}}}}}}"#
    ));
    (before, after)
}

#[test]
fn playing_score_and_ability_events() {
    let (before, after) = playing_pair(
        r#"{"steamid": "1", "name": "Rin", "activity": "playing",
            "kills": 2, "deaths": 0, "assists": 4}"#,
        r#"{"steamid": "1", "name": "Rin", "activity": "playing",
            "kills": 3, "deaths": 1, "assists": 5}"#,
    );
    let kinds: Vec<&str> = derive_happenings(Some(&before), &after)
        .iter()
        .map(|event| event.kind.as_str())
        .collect();
    for expected in [
        "night_started",
        "kill",
        "death",
        "assist",
        "ability_leveled",
        "ability_ready",
        "ability_cooldown",
    ] {
        assert!(kinds.contains(&expected), "missing {expected} in {kinds:?}");
    }
    let kill = derive_happenings(Some(&before), &after)
        .into_iter()
        .find(|event| event.kind == HappeningKind::Kill)
        .unwrap();
    assert_eq!(kill.actor, Some(String::from("Rin")));
}

#[test]
fn day_start_emits_when_night_ends() {
    let before = frame(&format!(
        r#"{{"provider": {PROVIDER}, "map": {{"daytime": false}}}}"#
    ));
    let after = frame(&format!(
        r#"{{"provider": {PROVIDER}, "map": {{"daytime": true}}}}"#
    ));
    let events = derive_happenings(Some(&before), &after);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, HappeningKind::DayStarted);
}

#[test]
fn spectating_pairs_skip_missing_sides_and_slots() {
    let before = frame(&format!(
        r#"{{"provider": {PROVIDER},
            "player": {{
                "team2": {{
                    "player0": {{"steamid": "1", "name": "Rin",
                        "activity": "playing", "kills": 1}},
                    "player1": {{"steamid": "2", "activity": "playing"}}}},
                "team3": {{"player0": {{"steamid": "9", "activity": "playing"}}}}}}}}"#
    ));
    let after = frame(&format!(
        r#"{{"provider": {PROVIDER},
            "player": {{
                "team2": {{
                    "player0": {{"steamid": "1", "name": "Rin",
                        "activity": "playing", "kills": 2}}}}}}}}"#
    ));
    let events = derive_happenings(Some(&before), &after);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, HappeningKind::Kill);
}

#[test]
fn camera_mode_switch_yields_nothing() {
    let playing = frame(&format!(
        r#"{{"provider": {PROVIDER},
            "player": {{"steamid": "1", "activity": "playing", "kills": 1}},
            "abilities": {{"ability0": {{"name": "blast", "level": 1}}}}}}"#
    ));
    let spectating = frame(&format!(
        r#"{{"provider": {PROVIDER},
            "player": {{"team2": {{"player0": {{"steamid": "1",
                "activity": "playing", "kills": 5}}}}}},
            "abilities": {{"team2": {{"player0": {{"ability0": {{"name": "blast",
                "level": 4}}}}}}}}}}"#
    ));
    assert!(derive_happenings(Some(&playing), &spectating).is_empty());
}

#[test]
fn spectating_ability_events_carry_player_slot() {
    let pair = |level: u8| {
        format!(
            r#"{{"provider": {PROVIDER},
                "abilities": {{"team2": {{"player0": {{"ability0": {{"name": "blast",
                    "level": {level}, "cooldown": 0}}}}}},
                    "team3": {{"player0": {{"ability0": {{"name": "blast",
                    "level": 1, "cooldown": 0}}}}}}}}}}"#
        )
    };
    let before = frame(&pair(1));
    let after = frame(&pair(2));
    let events = derive_happenings(Some(&before), &after);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, HappeningKind::AbilityLeveled);
    assert_eq!(events[0].detail["player"], serde_json::json!(0));
}

#[test]
fn spectating_ability_events_skip_vanished_sides() {
    let before = frame(&format!(
        r#"{{"provider": {PROVIDER},
            "abilities": {{
                "team2": {{"player0": {{"ability0": {{"name": "blast",
                    "level": 1, "cooldown": 0}}}}}},
                "team3": {{"player0": {{"ability0": {{"name": "blast",
                    "level": 1, "cooldown": 0}}}}}}}}}}"#
    ));
    let after = frame(&format!(
        r#"{{"provider": {PROVIDER},
            "abilities": {{
                "team2": {{"player0": {{"ability0": {{"name": "blast",
                    "level": 2, "cooldown": 0}}}}}}}}}}"#
    ));
    let events = derive_happenings(Some(&before), &after);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, HappeningKind::AbilityLeveled);
}
