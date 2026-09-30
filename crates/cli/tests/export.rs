//! `export` against a seeded store: layout, filters and token scrubbing.

use dct_cli::cli::ExportArgs;
use dct_cli::commands::export;
use dct_core::{Frame, Happening, HappeningKind};
use dct_store::{HappeningRecord, Store};
use serde_json::json;

fn payload(game_time: u32) -> String {
    format!(
        r#"{{"provider": {{"name": "Dota 2", "appid": 570, "version": 48, "timestamp": 1}},
        "map": {{"name": "dota", "matchid": "42", "game_time": {game_time}, "clock_time": {game_time},
        "daytime": true, "nightstalker_night": false,
        "game_state": "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS",
        "paused": false, "win_team": "none", "customgamename": ""}},
        "player": {{"steamid": "1", "name": "Rin", "activity": "playing",
        "kills": 1, "deaths": 0, "assists": 0, "team_name": "radiant"}},
        "auth": {{"token": "super-secret-token"}}}}"#
    )
}

async fn seed(db: &std::path::Path) -> Store {
    let store = Store::open(db).await.expect("open");
    for game_time in [600, 601, 602] {
        let raw = payload(game_time);
        let frame = Frame::from_slice(raw.as_bytes()).expect("parses");
        store
            .insert_frame(&frame, raw.as_bytes(), i64::from(game_time))
            .await
            .expect("insert");
    }
    let event = Happening {
        kind: HappeningKind::Kill,
        actor: Some("Rin".to_owned()),
        detail: json!({"kills": 1, "streak": 1}),
    };
    store
        .insert_happenings(&[HappeningRecord {
            match_id: "42",
            tick: 600,
            happening: &event,
            recorded_at: 602,
        }])
        .await
        .expect("insert");
    store
}

fn args(out: Option<std::path::PathBuf>, match_id: Option<&str>) -> ExportArgs {
    ExportArgs {
        out,
        match_id: match_id.map(str::to_owned),
        kind: None,
    }
}

#[tokio::test]
async fn run_reproduces_the_exports_layout() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("collect.db");
    seed(&db).await;
    let out = dir.path().join("exports-out");

    let report = export::run(&args(Some(out.clone()), None), Some(db))
        .await
        .expect("run");
    assert_eq!(report.frames, 3);
    assert_eq!(report.happenings, 1);

    let index: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out.join("frames_index.json")).expect("read index"),
    )
    .expect("json");
    let rows = index.as_array().expect("array");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0]["id"], 1);
    assert_eq!(rows[2]["id"], 3);
    assert_eq!(rows[0]["match_id"], "42");
    assert_eq!(rows[0]["payload_bytes"], payload(600).len() as u64);

    let happenings: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out.join("happenings.json")).expect("read happenings"),
    )
    .expect("json");
    let events = happenings.as_array().expect("array");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["kind"], "kill");
    assert_eq!(events[0]["actor"], "Rin");
    let detail = events[0]["detail"].as_str().expect("detail string");
    assert!(detail.contains("kills"));

    for name in [
        "payload_first_id1.json",
        "payload_middle_id2.json",
        "payload_last_id3.json",
    ] {
        let sample = std::fs::read_to_string(out.join("samples").join(name)).expect("sample");
        assert!(sample.contains("\"provider\""), "{name}");
        assert!(sample.contains("redacted"), "{name}");
        assert!(!sample.contains("super-secret-token"), "{name}");
    }
}

#[tokio::test]
async fn run_defaults_beside_the_database_and_honors_filters() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("state").join("collect.db");
    seed(&db).await;

    let report = export::run(&args(None, None), Some(db.clone()))
        .await
        .expect("run");
    assert_eq!(report.dir, dir.path().join("state").join("exports"));
    assert!(report.dir.join("frames_index.json").is_file());

    let filtered = export::run(
        &args(Some(dir.path().join("empty-out")), Some("nope")),
        Some(db),
    )
    .await
    .expect("run");
    assert_eq!(filtered.frames, 0);
    assert_eq!(filtered.happenings, 0);
    let index: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(filtered.dir.join("frames_index.json")).expect("read"),
    )
    .expect("json");
    assert_eq!(index, json!([]));
}
