//! Store behaviour: frames, happenings, filters and overflow replay.

use dct_core::{Frame, HappeningKind, derive_happenings};
use serde_json::json;

use dct_store::{HappeningFilter, HappeningRecord, Store, now_millis, overflow};

const PLAYING: &str = r#"{
    "provider": {"name": "Dota 2", "appid": 570, "version": 47, "timestamp": 1},
    "map": {"name": "dota", "matchid": "42", "game_time": 600, "clock_time": 600,
        "daytime": true, "nightstalker_night": false,
        "game_state": "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS",
        "paused": false, "win_team": "none", "customgamename": ""},
    "player": {"steamid": "1", "name": "Rin", "activity": "playing",
        "kills": 1, "deaths": 0, "assists": 0, "team_name": "radiant"},
    "auth": {"token": "t"}
}"#;

#[tokio::test]
async fn frames_and_happenings_roundtrip() {
    let store = Store::open_in_memory().await.expect("open");
    let first = Frame::from_slice(PLAYING.as_bytes()).expect("parses");
    let at = now_millis();
    store
        .insert_frame(&first, PLAYING.as_bytes(), at)
        .await
        .expect("insert");
    let exported = store.export_frames(Some("42")).await.expect("export");
    assert_eq!(exported.len(), 1);
    assert_eq!(exported[0].payload_kind, "playing");
    assert!(exported[0].normalized_payload.contains("provider"));

    let mut second_value = serde_json::from_str::<serde_json::Value>(PLAYING).expect("json");
    second_value["player"]["kills"] = json!(2);
    second_value["map"]["game_time"] = json!(601);
    let second: Frame = serde_json::from_value(second_value.clone()).expect("parses");
    let events = derive_happenings(Some(&first), &second);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, HappeningKind::Kill);
    store
        .insert_frame(&second, second_value.to_string().as_bytes(), at + 1)
        .await
        .expect("insert");
    let records: Vec<HappeningRecord> = events
        .iter()
        .map(|event| HappeningRecord {
            match_id: "42",
            tick: 601,
            happening: event,
            recorded_at: at + 1,
        })
        .collect();
    store.insert_happenings(&records).await.expect("insert");

    let all = store
        .query_happenings(&HappeningFilter {
            limit: 50,
            ..Default::default()
        })
        .await
        .expect("query");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].kind, "kill");
    assert_eq!(all[0].actor.as_deref(), Some("Rin"));

    let by_match = store
        .query_happenings(&HappeningFilter {
            match_id: Some("nope".to_owned()),
            limit: 50,
            ..Default::default()
        })
        .await
        .expect("query");
    assert!(by_match.is_empty());

    let by_kind = store
        .query_happenings(&HappeningFilter {
            kind: Some("death".to_owned()),
            limit: 50,
            ..Default::default()
        })
        .await
        .expect("query");
    assert!(by_kind.is_empty());
}

#[tokio::test]
async fn overflow_spill_and_replay() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("overflow.jsonl");
    overflow::spill(&path, PLAYING.as_bytes(), 7)
        .await
        .expect("spill");
    overflow::spill(&path, b"{broken", 8).await.expect("spill");

    let store = Store::open_in_memory().await.expect("open");
    let stats = overflow::replay(&store, &path).await.expect("replay");
    assert_eq!(stats.inserted, 1);
    assert_eq!(stats.skipped, 1);

    let missing = overflow::replay(&store, &dir.path().join("absent.jsonl"))
        .await
        .expect("replay");
    assert_eq!(missing, Default::default());
}

#[tokio::test]
async fn file_database_creates_parents_and_migrates() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("nested").join("collect.db");
    let store = Store::open(&path).await.expect("open");
    assert!(path.exists());
    let found = store
        .query_happenings(&HappeningFilter {
            limit: 10,
            ..Default::default()
        })
        .await
        .expect("query");
    assert!(found.is_empty());
}
