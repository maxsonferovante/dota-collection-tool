//! Frame parsing across camera modes plus happening derivation.

use dct_core::{Frame, HappeningKind, derive_happenings};

const IDLE: &str = r#"{
    "provider": {"name": "Dota 2", "appid": 570, "version": 47, "timestamp": 1658690112},
    "player": {},
    "draft": {},
    "auth": {"token": "1234"}
}"#;

const HEARTBEAT: &str = r#"{
    "provider": {"name": "Dota 2", "appid": 570, "version": 47, "timestamp": 1658690113},
    "auth": {"token": "1234"}
}"#;

const PLAYING: &str = r#"{
    "provider": {"name": "Dota 2", "appid": 570, "version": 47, "timestamp": 1659035016},
    "map": {
        "name": "dota", "matchid": "777", "game_time": 600, "clock_time": 600,
        "daytime": true, "nightstalker_night": false,
        "game_state": "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS",
        "paused": false, "win_team": "none", "customgamename": "",
        "radiant_score": 5, "dire_score": 3
    },
    "player": {
        "steamid": "1", "name": "Rin", "activity": "playing",
        "kills": 2, "deaths": 1, "assists": 4, "team_name": "radiant",
        "gold": 900, "gpm": 400, "xpm": 420, "net_worth": 5000
    },
    "hero": {
        "id": 42, "name": "npc_dota_hero_skeleton_king", "level": 8,
        "health": 1200, "max_health": 1200, "mana": 300, "max_mana": 300,
        "talent_1": true, "future_flag_xyz": 1
    },
    "abilities": {
        "ability0": {"name": "blast", "level": 2, "cooldown": 0, "ultimate": false}
    },
    "items": {
        "slot0": {"name": "item_blink"},
        "slot1": {"name": "empty"},
        "teleport0": {"name": "item_tpscroll", "charges": 1},
        "neutral0": {"name": "empty"}
    },
    "wearables": {"wearable0": 13773},
    "auth": {"token": "1234"}
}"#;

const SPECTATING: &str = r#"{
    "provider": {"name": "Dota 2", "appid": 570, "version": 47, "timestamp": 1659035017},
    "map": {
        "name": "dota", "matchid": "777", "game_time": 601, "clock_time": 601,
        "daytime": false, "nightstalker_night": false,
        "game_state": "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS",
        "paused": false, "win_team": "none", "customgamename": ""
    },
    "player": {
        "team2": {
            "player0": {"steamid": "1", "name": "Rin", "activity": "playing",
                "kills": 3, "deaths": 1, "assists": 4, "team_name": "radiant",
                "net_worth": 5200, "hero_damage": 2725, "wards_placed": 3}
        },
        "team3": {
            "player5": {"steamid": "2", "name": "Kai", "activity": "playing",
                "kills": 1, "deaths": 2, "assists": 1, "team_name": "dire"}
        }
    },
    "hero": {
        "team2": {"player0": {"id": 42, "name": "npc_dota_hero_skeleton_king"}},
        "team3": {"player5": {"id": 7, "name": "npc_dota_hero_earthshaker"}}
    },
    "buildings": {
        "radiant": {"dota_goodguys_tower1_mid": {"health": 1800, "max_health": 1800}}
    },
    "events": [{"game_time": 601, "event_type": "bounty_rune_pickup", "team": 2}],
    "league": {"league_id": 123, "name": "Pro Cup"},
    "draft": {"radiant": {"player0": 42}},
    "minimap": {"mark1": {"xpos": 100, "ypos": -200}},
    "roshan": {"health": 5500, "max_health": 5500},
    "couriers": {"courier0": {"health": 50, "alive": true}},
    "neutralitems": {"tier1": {"max_count": 4}},
    "previously": {"map": {"daytime": true}},
    "added": {"events": true},
    "auth": {"token": "1234"}
}"#;

#[test]
fn idle_frame_parses_with_empty_blocks_absent() {
    let frame = Frame::from_slice(IDLE.as_bytes()).expect("idle parses");
    assert_eq!(frame.provider.app_id, 570);
    assert!(frame.players.is_none());
    assert!(frame.map.is_none());
    assert_eq!(frame.auth.unwrap().token.as_deref(), Some("1234"));
}

#[test]
fn heartbeat_without_blocks_parses() {
    let frame = Frame::from_slice(HEARTBEAT.as_bytes()).expect("heartbeat parses");
    assert!(frame.map.is_none());
    assert!(frame.players.is_none());
}

#[test]
fn playing_frame_parses_all_core_blocks() {
    let frame = Frame::from_slice(PLAYING.as_bytes()).expect("playing parses");
    let map = frame.map.as_ref().expect("map present");
    assert_eq!(map.match_id, "777");
    assert_eq!(map.radiant_score, Some(5));
    assert_eq!(map.dire_score, Some(3));
    assert!(matches!(
        frame.players,
        Some(dct_core::player::Players::Playing(_))
    ));
    let items = frame.items.as_ref().expect("items present");
    match items {
        dct_core::items::GameItems::Playing(set) => {
            assert_eq!(set.len(), 4);
        }
        _ => panic!("expected playing items"),
    }
    assert_eq!(frame.match_id(), Some("777"));
}

#[test]
fn unknown_fields_never_fail() {
    let frame = Frame::from_slice(PLAYING.as_bytes()).expect("unknown fields parse");
    let heroes = frame.heroes.expect("heroes present");
    match heroes {
        dct_core::hero::Heroes::Playing(hero) => {
            assert!(hero.extra.contains_key("future_flag_xyz"));
            assert_eq!(hero.extra.get("talent_1"), Some(&serde_json::json!(true)));
        }
        _ => panic!("expected playing hero"),
    }
}

#[test]
fn spectator_frame_parses_groupings_and_extra_blocks() {
    let frame = Frame::from_slice(SPECTATING.as_bytes()).expect("spectating parses");
    match frame.players.expect("players present") {
        dct_core::player::Players::Spectating(grouped) => {
            assert_eq!(grouped.len(), 2);
            let radiant = &grouped[&dct_core::Side::Radiant];
            assert_eq!(radiant.len(), 1);
        }
        _ => panic!("expected spectating players"),
    }
    assert!(frame.events.expect("events").len() == 1);
    assert!(frame.league.is_some());
    assert!(frame.minimap.is_some());
    assert!(frame.roshan.is_some());
    assert!(frame.couriers.is_some());
    assert!(frame.neutralitems.is_some());
    assert!(frame.previously.is_some());
    assert!(frame.added.is_some());
}

#[test]
fn spectator_extras_are_preserved_not_dropped() {
    let frame = Frame::from_slice(SPECTATING.as_bytes()).expect("spectating parses");
    match frame.players.expect("players") {
        dct_core::player::Players::Spectating(grouped) => {
            let info = &grouped[&dct_core::Side::Radiant][&dct_core::PlayerSlot(0)];
            assert_eq!(
                info.extra.get("hero_damage"),
                Some(&serde_json::json!(2725))
            );
        }
        _ => panic!("expected spectating"),
    }
}

#[test]
fn invalid_json_fails() {
    assert!(Frame::from_slice(b"{nope").is_err());
}

#[test]
fn first_frame_emits_nothing() {
    let current = Frame::from_slice(PLAYING.as_bytes()).expect("parses");
    assert!(derive_happenings(None, &current).is_empty());
}

#[test]
fn kill_death_and_nightfall_derive() {
    let before = Frame::from_slice(PLAYING.as_bytes()).expect("parses");
    let mut after_value = serde_json::from_str::<serde_json::Value>(PLAYING).expect("json");
    after_value["player"]["kills"] = serde_json::json!(3);
    after_value["player"]["deaths"] = serde_json::json!(2);
    after_value["map"]["daytime"] = serde_json::json!(false);
    after_value["abilities"]["ability0"]["level"] = serde_json::json!(3);
    after_value["abilities"]["ability0"]["cooldown"] = serde_json::json!(12);
    let after: Frame = serde_json::from_value(after_value).expect("parses");

    let kinds: Vec<_> = derive_happenings(Some(&before), &after)
        .into_iter()
        .map(|event| event.kind)
        .collect();
    assert!(kinds.contains(&HappeningKind::Kill));
    assert!(kinds.contains(&HappeningKind::Death));
    assert!(kinds.contains(&HappeningKind::NightStarted));
    assert!(kinds.contains(&HappeningKind::AbilityLeveled));
    assert!(kinds.contains(&HappeningKind::AbilityCooldown));
}

#[test]
fn identical_frames_stay_silent() {
    let frame = Frame::from_slice(PLAYING.as_bytes()).expect("parses");
    let again = frame.clone();
    assert!(derive_happenings(Some(&frame), &again).is_empty());
}

#[test]
fn camera_mode_switch_stays_silent() {
    let playing = Frame::from_slice(PLAYING.as_bytes()).expect("parses");
    // Align the clock so only the camera mode differs between fixtures.
    let mut spectating_value = serde_json::from_str::<serde_json::Value>(SPECTATING).expect("json");
    spectating_value["map"]["daytime"] = serde_json::json!(true);
    let spectating: Frame = serde_json::from_value(spectating_value).expect("parses");
    assert!(derive_happenings(Some(&playing), &spectating).is_empty());
}
