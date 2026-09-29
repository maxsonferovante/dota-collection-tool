//! `logs` against a seeded store: formats plus the run path.

use dct_cli::commands::logs;
use dct_core::{Happening, HappeningKind};
use dct_store::{HappeningFilter, HappeningRecord, Store, StoredHappening};
use serde_json::json;

fn row() -> StoredHappening {
    StoredHappening {
        id: 7,
        match_id: "9".to_owned(),
        tick: 601,
        kind: "kill".to_owned(),
        actor: Some("Rin".to_owned()),
        detail: json!({"kills": 2}),
    }
}

#[test]
fn text_format_holds_tick_kind_actor_detail() {
    assert_eq!(logs::format_text(&row()), "#601 kill Rin {\"kills\":2}");
}

#[test]
fn json_format_roundtrips() {
    let parsed: serde_json::Value = serde_json::from_str(&logs::format_json(&row())).expect("json");
    assert_eq!(parsed["kind"], "kill");
    assert_eq!(parsed["tick"], 601);
    assert_eq!(parsed["detail"]["kills"], 2);
}

#[tokio::test]
async fn run_lists_seeded_happenings() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("collect.db");
    let store = Store::open(&db).await.expect("open");
    let event = Happening {
        kind: HappeningKind::Death,
        actor: Some("Kai".to_owned()),
        detail: json!({"deaths": 1}),
    };
    store
        .insert_happenings(&[HappeningRecord {
            match_id: "9",
            tick: 602,
            happening: &event,
            recorded_at: 1,
        }])
        .await
        .expect("insert");

    logs::run(
        &dct_cli::cli::LogsArgs {
            match_id: Some("9".to_owned()),
            kind: None,
            limit: 10,
            follow: false,
            json: true,
        },
        Some(db),
    )
    .await
    .expect("run");

    let rows = store
        .query_happenings(&HappeningFilter {
            after_id: Some(0),
            limit: 10,
            ..Default::default()
        })
        .await
        .expect("query");
    assert_eq!(rows.len(), 1);
}
