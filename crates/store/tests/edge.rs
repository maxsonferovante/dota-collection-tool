//! Store edge paths: unreadable locations, bad payloads, kind filters,
//! and overflow spill/replay tolerance.

use dct_core::{Frame, Happening, HappeningKind};
use dct_store::{HappeningRecord, Store, replay, spill};
use serde_json::json;

const PROVIDER: &str = r#"{"name": "Dota 2", "appid": 570, "version": 47, "timestamp": 1}"#;

fn frame_body(kills: u16, game_time: u32) -> String {
    format!(
        r#"{{"provider": {PROVIDER},
        "map": {{"name": "dota", "matchid": "7", "game_time": {game_time},
        "clock_time": {game_time}, "daytime": true, "nightstalker_night": false,
        "game_state": "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS",
        "paused": false, "win_team": "none", "customgamename": ""}},
        "player": {{"steamid": "1", "name": "Rin", "activity": "playing",
        "kills": {kills}, "deaths": 0, "assists": 0, "team_name": "radiant"}},
        "auth": {{"token": "t"}}}}"#
    )
}

#[tokio::test]
async fn open_fails_when_parent_is_a_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, b"nope").expect("write");
    let err = match Store::open(&blocker.join("collect.db")).await {
        Ok(_) => panic!("must fail"),
        Err(err) => err,
    };
    assert!(!err.to_string().is_empty());
}

#[tokio::test]
async fn insert_rejects_non_utf8_payload() {
    let store = Store::open_in_memory().await.expect("open");
    let raw = frame_body(0, 600);
    let frame = Frame::from_slice(raw.as_bytes()).expect("parses");
    let err = store
        .insert_frame(&frame, &[0xff, 0xfe, 0x00], 1)
        .await
        .expect_err("must fail");
    assert!(!err.to_string().is_empty());
}

#[tokio::test]
async fn export_honors_match_and_kind_filters() {
    let store = Store::open_in_memory().await.expect("open");
    let event = Happening {
        kind: HappeningKind::Kill,
        actor: Some("Rin".to_owned()),
        detail: json!({"kills": 1}),
    };
    store
        .insert_happenings(&[HappeningRecord {
            match_id: "7",
            tick: 600,
            happening: &event,
            recorded_at: 1,
        }])
        .await
        .expect("insert");

    let kills = store
        .export_happenings(Some("7"), Some("kill"))
        .await
        .expect("export");
    assert_eq!(kills.len(), 1);
    let deaths = store
        .export_happenings(Some("7"), Some("death"))
        .await
        .expect("export");
    assert!(deaths.is_empty());
}

#[tokio::test]
async fn spill_creates_missing_parents() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("deep").join("nested.jsonl");
    spill(&path, frame_body(0, 600).as_bytes(), 1)
        .await
        .expect("spill");
    assert!(path.is_file());
}

#[tokio::test]
async fn replay_rejects_unreadable_location() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::open_in_memory().await.expect("open");
    let err = replay(&store, dir.path()).await.expect_err("must fail");
    assert!(!err.to_string().is_empty());
}

#[tokio::test]
async fn replay_counts_inserted_and_skipped_lines() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::open_in_memory().await.expect("open");
    let path = dir.path().join("overflow.jsonl");
    let line = |received_at: i64, payload: &str| {
        format!("{{\"received_at\": {received_at}, \"payload\": {payload}}}\n")
    };
    let compact = |kills: u16, game_time: u32| {
        let value: serde_json::Value =
            serde_json::from_str(&frame_body(kills, game_time)).expect("valid frame");
        value.to_string()
    };
    let mut content = String::new();
    content.push_str(&line(1, &compact(0, 600)));
    content.push_str(&line(2, &compact(1, 601)));
    content.push('\n');
    content.push_str(&line(3, r#"{"bogus": true}"#));
    content.push_str("{\"received_at\": 4}\n");
    content.push_str("{broken\n");
    std::fs::write(&path, content).expect("write");

    let stats = replay(&store, &path).await.expect("replay");
    assert_eq!(stats.inserted, 2);
    assert_eq!(stats.skipped, 3);
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "");

    let frames = store.export_frames(Some("7")).await.expect("export");
    assert_eq!(frames.len(), 2);
    let kills = store
        .export_happenings(Some("7"), Some("kill"))
        .await
        .expect("export");
    assert_eq!(kills.len(), 1);
}
