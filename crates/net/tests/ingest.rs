//! Ingestion behaviour over real HTTP on an ephemeral port.

use std::time::Duration;

use dct_net::{IngestConfig, run_on};
use dct_store::{HappeningFilter, Store};

const BODY: &str = r#"{
    "provider": {"name": "Dota 2", "appid": 570, "version": 47, "timestamp": 1},
    "map": {"name": "dota", "matchid": "9", "game_time": 10, "clock_time": 10,
        "daytime": true, "nightstalker_night": false,
        "game_state": "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS",
        "paused": false, "win_team": "none", "customgamename": ""},
    "player": {"steamid": "1", "name": "Rin", "activity": "playing",
        "kills": 0, "deaths": 0, "assists": 0, "team_name": "radiant"},
    "auth": {"token": "secret"}
}"#;

struct Running {
    base: String,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    task: tokio::task::JoinHandle<()>,
}

async fn spawn_server() -> (Running, Store, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::open(&dir.path().join("collect.db"))
        .await
        .expect("open");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let base = format!("http://{}", listener.local_addr().expect("addr"));
    let (shutdown, waiter) = tokio::sync::oneshot::channel::<()>();
    let task_store = store.clone();
    let overflow = dir.path().join("overflow.jsonl");
    let task = tokio::spawn(async move {
        run_on(
            listener,
            task_store,
            IngestConfig {
                token: "secret".to_owned(),
                overflow_path: overflow,
                queue_capacity: 16,
            },
            async move {
                let _ = waiter.await;
            },
        )
        .await
        .expect("serve");
    });
    // Let the listener accept before the first request lands.
    tokio::time::sleep(Duration::from_millis(50)).await;
    (
        Running {
            base,
            shutdown: Some(shutdown),
            task,
        },
        store,
        dir,
    )
}

impl Running {
    async fn stop(mut self) {
        drop(self.shutdown.take());
        self.task.await.expect("join");
    }
}

async fn settle() {
    tokio::time::sleep(Duration::from_millis(200)).await;
}

#[tokio::test]
async fn valid_post_is_stored() {
    let (running, store, _dir) = spawn_server().await;
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/", running.base))
        .header("content-type", "application/json")
        .body(BODY)
        .send()
        .await
        .expect("post");
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers()["content-type"],
        dct_net::SUCCESS_CONTENT_TYPE
    );
    settle().await;
    let second = BODY.replace("\"kills\": 0", "\"kills\": 1").replace(
        "\"game_time\": 10, \"clock_time\": 10",
        "\"game_time\": 11, \"clock_time\": 11",
    );
    let response = client
        .post(format!("{}/", running.base))
        .header("content-type", "application/json")
        .body(second)
        .send()
        .await
        .expect("post");
    assert_eq!(response.status(), 200);
    settle().await;
    let found = store
        .query_happenings(&HappeningFilter {
            limit: 50,
            ..Default::default()
        })
        .await
        .expect("query");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].kind, "kill");
    assert_eq!(found[0].match_id, "9");
    running.stop().await;
}

#[tokio::test]
async fn wrong_token_is_rejected() {
    let (running, store, _dir) = spawn_server().await;
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/", running.base))
        .body(BODY.replace("secret", "wrong"))
        .send()
        .await
        .expect("post");
    assert_eq!(response.status(), 401);
    settle().await;
    let found = store
        .query_happenings(&HappeningFilter {
            limit: 50,
            ..Default::default()
        })
        .await
        .expect("query");
    assert!(found.is_empty());
    running.stop().await;
}

#[tokio::test]
async fn malformed_body_gets_success_but_stores_nothing() {
    let (running, store, _dir) = spawn_server().await;
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/", running.base))
        .body("{broken")
        .send()
        .await
        .expect("post");
    assert_eq!(response.status(), 200);
    settle().await;
    let found = store
        .query_happenings(&HappeningFilter {
            limit: 50,
            ..Default::default()
        })
        .await
        .expect("query");
    assert!(found.is_empty());
    running.stop().await;
}

#[tokio::test]
async fn health_reports_ok() {
    let (running, _store, _dir) = spawn_server().await;
    let body = reqwest::get(format!("{}/health", running.base))
        .await
        .expect("get")
        .text()
        .await
        .expect("body");
    assert!(body.contains("ok"));
    running.stop().await;
}
