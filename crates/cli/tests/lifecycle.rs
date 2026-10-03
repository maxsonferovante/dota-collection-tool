//! Lifecycle against the real binary: detach, collect, stop.

use std::path::PathBuf;
use std::time::Duration;

use dct_cli::cli::UpArgs;
use dct_cli::lifecycle;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind")
        .local_addr()
        .expect("addr")
        .port()
}

fn body(token: &str, kills: u16, game_time: u32) -> String {
    format!(
        r#"{{"provider": {{"name": "Dota 2", "appid": 570, "version": 47, "timestamp": 1}},
        "map": {{"name": "dota", "matchid": "5", "game_time": {game_time}, "clock_time": {game_time},
        "daytime": true, "nightstalker_night": false,
        "game_state": "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS",
        "paused": false, "win_team": "none", "customgamename": ""}},
        "player": {{"steamid": "1", "name": "Rin", "activity": "playing",
        "kills": {kills}, "deaths": 0, "assists": 0, "team_name": "radiant"}},
        "auth": {{"token": "{token}"}}}}"#
    )
}

async fn post_frame(port: u16, payload: &str) -> String {
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("connect");
    let request = format!(
        "POST / HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
        payload.len()
    );
    stream.write_all(request.as_bytes()).await.expect("write");
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.expect("read");
    String::from_utf8_lossy(&raw).into_owned()
}

#[tokio::test]
async fn status_reports_stopped_when_absent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("collect.db");
    let report = lifecycle::status(Some(db), free_port())
        .await
        .expect("status");
    assert!(!report.running);
}

#[tokio::test]
async fn detach_collect_stop_cycle() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("state").join("collect.db");
    let port = free_port();
    let exe = PathBuf::from(env!("CARGO_BIN_EXE_dct"));
    let config = db.parent().expect("parent").join("config.toml");
    let token = dct_cli::config::fresh_token(&config).await.expect("token");

    let pid = lifecycle::up(&UpArgs { detach: true }, &exe, port, Some(db.clone()))
        .await
        .expect("detach");
    assert!(pid.is_some());

    let mut running = false;
    for _ in 0..100 {
        let report = lifecycle::status(Some(db.clone()), port)
            .await
            .expect("status");
        if report.health.is_some() {
            running = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(running, "detached server never became healthy");

    let first = post_frame(port, &body(&token, 0, 600)).await;
    assert!(first.starts_with("HTTP/1.1 200"), "{first}");
    let second = post_frame(port, &body(&token, 1, 601)).await;
    assert!(second.starts_with("HTTP/1.1 200"), "{second}");
    tokio::time::sleep(Duration::from_millis(500)).await;

    let store = dct_store::Store::open(&db).await.expect("open");
    let found = store
        .query_happenings(&dct_store::HappeningFilter {
            kind: Some("kill".to_owned()),
            limit: 10,
            ..Default::default()
        })
        .await
        .expect("query");
    assert_eq!(found.len(), 1);

    assert!(lifecycle::down(Some(db.clone())).await.expect("down"));
    assert!(!dir.path().join("state").join("server.pid").exists());
    let report = lifecycle::status(Some(db), port).await.expect("status");
    assert!(!report.running);
}

#[tokio::test]
async fn down_without_pid_file_reports_stopped() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("collect.db");
    assert!(!lifecycle::down(Some(db)).await.expect("down"));
}

#[tokio::test]
async fn down_clears_stale_pid() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("collect.db");
    std::fs::write(dir.path().join("server.pid"), "42424242").expect("pid");
    assert!(!lifecycle::down(Some(db)).await.expect("down"));
    assert!(!dir.path().join("server.pid").exists());
}

#[tokio::test]
async fn down_rejects_unreadable_pid_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("collect.db");
    std::fs::create_dir(dir.path().join("server.pid")).expect("pid dir");
    let err = lifecycle::down(Some(db)).await.expect_err("must fail");
    assert!(err.to_string().contains("cannot read pid file"), "{err}");
}

#[tokio::test]
async fn up_refuses_when_already_running() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("collect.db");
    std::fs::write(
        dir.path().join("server.pid"),
        std::process::id().to_string(),
    )
    .expect("pid");
    let err = lifecycle::up(
        &UpArgs { detach: true },
        std::path::Path::new("unused"),
        free_port(),
        Some(db),
    )
    .await
    .expect_err("must fail");
    assert!(err.to_string().contains("already running"), "{err}");
}

#[tokio::test]
async fn fetch_health_rejects_unhealthy_endpoint() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                use tokio::io::AsyncWriteExt;
                let _ = socket
                    .write_all(
                        b"HTTP/1.1 500 boom\r\nContent-Length: 4\r\nConnection: close\r\n\r\nnope",
                    )
                    .await;
                // Keep the socket open after FIN: dropping it makes macOS
                // RST the close and the client reports a read error instead.
                let _ = socket.shutdown().await;
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            });
        }
    });
    let err = lifecycle::fetch_health(port).await.expect_err("must fail");
    assert!(err.to_string().contains("unhealthy"), "{err}");
}

#[tokio::test]
async fn up_fails_when_the_port_never_serves_health() {
    // A listener that accepts and immediately drops: connects succeed,
    // health never answers, so the detached child can never be adopted.
    let holder = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = holder.local_addr().expect("addr").port();
    tokio::spawn(async move {
        loop {
            let _ = holder.accept().await;
        }
    });
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("collect.db");
    let exe = PathBuf::from(env!("CARGO_BIN_EXE_dct"));
    let err = lifecycle::up(&UpArgs { detach: true }, &exe, port, Some(db))
        .await
        .expect_err("must fail");
    assert!(err.to_string().contains("never became healthy"), "{err}");
}

#[tokio::test]
async fn down_clears_unparseable_pid() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("collect.db");
    std::fs::write(dir.path().join("server.pid"), "not-a-pid").expect("pid");
    assert!(!lifecycle::down(Some(db)).await.expect("down"));
    assert!(!dir.path().join("server.pid").exists());
}
