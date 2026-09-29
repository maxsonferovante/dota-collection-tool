//! Full product flow through the real binary: install, token, up, burst
//! with heartbeat, logs, follow, down.

use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_dct"))
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind")
        .local_addr()
        .expect("addr")
        .port()
}

async fn run_dct(db: &Path, port: u16, args: &[&str]) -> std::process::Output {
    tokio::process::Command::new(binary())
        .arg("--port")
        .arg(port.to_string())
        .arg("--db")
        .arg(db)
        .args(args)
        .output()
        .await
        .expect("spawn")
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

fn envelope(token: &str, inner: &str) -> String {
    if inner.is_empty() {
        format!(
            r#"{{"provider": {{"name": "Dota 2", "appid": 570, "version": 47, "timestamp": 1}},
            "auth": {{"token": "{token}"}}}}"#
        )
    } else {
        format!(
            r#"{{"provider": {{"name": "Dota 2", "appid": 570, "version": 47, "timestamp": 1}},
            {inner},
            "auth": {{"token": "{token}"}}}}"#
        )
    }
}

fn playing_inner(kills: u16, game_time: u32) -> String {
    format!(
        r#""map": {{"name": "dota", "matchid": "5", "game_time": {game_time}, "clock_time": {game_time},
        "daytime": true, "nightstalker_night": false,
        "game_state": "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS",
        "paused": false, "win_team": "none", "customgamename": ""}},
        "player": {{"steamid": "1", "name": "Rin", "activity": "playing",
        "kills": {kills}, "deaths": 0, "assists": 0, "team_name": "radiant"}}"#
    )
}

fn frame(token: &str, kills: u16, game_time: u32) -> String {
    envelope(token, &playing_inner(kills, game_time))
}

fn heartbeat(token: &str) -> String {
    envelope(token, "")
}

#[test]
fn discovery_prefers_libraries_holding_the_manifest() {
    let dir = tempfile::tempdir().expect("tempdir");
    let plain = dir.path().join("steam-a");
    let library = dir.path().join("steam-b");
    for root in [&plain, &library] {
        std::fs::create_dir_all(root.join("steamapps")).expect("mkdir");
    }
    // No manifest anywhere: nothing found.
    assert!(dct_cli::steam::find_dota_root_in(std::slice::from_ref(&plain)).is_none());
    // Manifest plus game tree: found, even when listed second.
    std::fs::write(library.join("steamapps").join("appmanifest_570.acf"), "").expect("write");
    let game = library
        .join("steamapps")
        .join("common")
        .join("dota 2 beta")
        .join("game")
        .join("dota");
    std::fs::create_dir_all(&game).expect("mkdir");
    let found = dct_cli::steam::find_dota_root_in(&[plain, library]).expect("found");
    assert!(found.ends_with("dota 2 beta"));
    assert_eq!(
        dct_cli::steam::integration_dir(&found)
            .file_name()
            .expect("name"),
        "gamestate_integration"
    );
}

#[test]
fn discovery_follows_libraryfolders_vdf() {
    let dir = tempfile::tempdir().expect("tempdir");
    let primary = dir.path().join("steam");
    let extra = dir.path().join("extra-library");
    std::fs::create_dir_all(primary.join("steamapps")).expect("mkdir");
    let game = extra
        .join("steamapps")
        .join("common")
        .join("dota 2 beta")
        .join("game")
        .join("dota");
    std::fs::create_dir_all(&game).expect("mkdir");
    std::fs::write(extra.join("steamapps").join("appmanifest_570.acf"), "").expect("write");
    let manifest = format!(
        "\"libraryfolders\"\n{{\n\t\"0\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n}}\n",
        extra.display()
    );
    std::fs::write(
        primary.join("steamapps").join("libraryfolders.vdf"),
        manifest,
    )
    .expect("write");
    let found = dct_cli::steam::find_dota_root_in(std::slice::from_ref(&primary)).expect("found");
    assert!(found.ends_with("dota 2 beta"));
}

#[tokio::test]
async fn full_flow_install_up_burst_logs_down() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("dota 2 beta");
    let db = dir.path().join("state").join("collect.db");
    let port = free_port();

    let install = run_dct(
        &db,
        port,
        &[
            "install",
            "--dota-dir",
            root.to_str().expect("utf8"),
            "--name",
            "e2e",
        ],
    )
    .await;
    assert!(install.status.success());
    let install_out = String::from_utf8_lossy(&install.stdout).into_owned();
    assert!(install_out.contains("launch flag"), "{install_out}");
    assert!(install_out.contains("restart"), "{install_out}");
    let cfg = root
        .join("game")
        .join("dota")
        .join("cfg")
        .join("gamestate_integration")
        .join("gamestate_integration_e2e.cfg");
    assert!(cfg.is_file());
    let cfg_text = std::fs::read_to_string(&cfg).expect("read cfg");
    for knob in [
        "\"uri\"",
        "\"timeout\"",
        "\"buffer\"",
        "\"throttle\"",
        "\"heartbeat\"",
    ] {
        assert!(cfg_text.contains(knob), "missing {knob}");
    }

    let shown = run_dct(&db, port, &["token", "--show"]).await;
    assert!(shown.status.success());
    let old_token = String::from_utf8_lossy(&shown.stdout).trim().to_owned();
    assert_eq!(old_token.len(), 64);
    assert!(cfg_text.contains(&old_token));

    let rotated = run_dct(&db, port, &["token", "--rotate"]).await;
    assert!(rotated.status.success());
    let reshown = run_dct(&db, port, &["token", "--show"]).await;
    let token = String::from_utf8_lossy(&reshown.stdout).trim().to_owned();
    assert_eq!(token.len(), 64);
    assert_ne!(token, old_token);

    // Force reinstall picks up the rotated token and backs the old file up.
    let reinstall = run_dct(
        &db,
        port,
        &[
            "install",
            "--dota-dir",
            root.to_str().expect("utf8"),
            "--name",
            "e2e",
            "--force",
        ],
    )
    .await;
    assert!(reinstall.status.success());
    assert!(cfg.with_extension("cfg.bak").is_file());
    let refreshed = std::fs::read_to_string(&cfg).expect("read cfg");
    assert!(refreshed.contains(&token));
    assert!(!refreshed.contains(&old_token));

    let up = run_dct(&db, port, &["up", "--detach"]).await;
    assert!(up.status.success());

    // The rotated-out token is rejected end to end.
    let stale = post_frame(port, &frame(&old_token, 9, 1)).await;
    assert!(stale.starts_with("HTTP/1.1 401"), "{stale}");

    let status = run_dct(&db, port, &["status"]).await;
    let status_out = String::from_utf8_lossy(&status.stdout).into_owned();
    assert!(status_out.contains("running"), "{status_out}");

    for tick in 1..=10u32 {
        let kills = if tick >= 5 { 1 } else { 0 };
        let reply = post_frame(port, &frame(&token, kills, tick)).await;
        assert!(reply.starts_with("HTTP/1.1 200"), "{reply}");
    }
    let heartbeat_reply = post_frame(port, &heartbeat(&token)).await;
    assert!(heartbeat_reply.starts_with("HTTP/1.1 200"));
    tokio::time::sleep(Duration::from_millis(500)).await;

    let logs = run_dct(&db, port, &["logs", "--match", "5"]).await;
    assert!(logs.status.success());
    let text = String::from_utf8_lossy(&logs.stdout).into_owned();
    assert!(text.contains("kill"), "{text}");

    // Heartbeat posts store no happenings: still exactly one line.
    let json_logs = run_dct(&db, port, &["logs", "--match", "5", "--json"]).await;
    assert!(json_logs.status.success());
    let json_text = String::from_utf8_lossy(&json_logs.stdout).into_owned();
    let lines: Vec<&str> = json_text.lines().collect();
    assert_eq!(lines.len(), 1);
    let parsed: serde_json::Value = serde_json::from_str(lines[0]).expect("json line");
    assert_eq!(parsed["kind"], "kill");

    let limited = run_dct(
        &db,
        port,
        &["logs", "--match", "5", "--kind", "kill", "--limit", "5"],
    )
    .await;
    assert!(limited.status.success());
    let limited_text = String::from_utf8_lossy(&limited.stdout).into_owned();
    assert!(limited_text.contains("kill"), "{limited_text}");

    let mut follow = tokio::process::Command::new(binary())
        .arg("--port")
        .arg(port.to_string())
        .arg("--db")
        .arg(&db)
        .args(["logs", "--match", "5", "--follow"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn follow");
    tokio::time::sleep(Duration::from_millis(700)).await;
    let follow_reply = post_frame(port, &frame(&token, 2, 11)).await;
    assert!(follow_reply.starts_with("HTTP/1.1 200"));
    let stdout = follow.stdout.take().expect("piped");
    let mut lines = BufReader::new(stdout).lines();
    let seen = tokio::time::timeout(Duration::from_secs(5), lines.next_line())
        .await
        .expect("timeout")
        .expect("read")
        .expect("line");
    assert!(seen.contains("kill"), "{seen}");
    follow.kill().await.expect("kill");

    let down = run_dct(&db, port, &["down"]).await;
    assert!(down.status.success());
    assert!(!dir.path().join("state").join("server.pid").exists());
    let status = run_dct(&db, port, &["status"]).await;
    let status_out = String::from_utf8_lossy(&status.stdout);
    assert!(status_out.contains("stopped"), "{status_out}");
}
