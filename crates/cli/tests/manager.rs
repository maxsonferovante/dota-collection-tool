//! Manager odds and ends: fake Steam roots, nested state paths,
//! invalid inputs, and the dispatch table.

use std::path::{Path, PathBuf};

use dct_cli::cli::{Cli, Command, ExportArgs, InstallArgs, TokenArgs};
use dct_cli::commands::{self, install, token};
use dct_cli::{cfg, config, paths, steam};

fn fake_steam_root(dir: &Path, manifest: bool, second_candidate: bool) -> PathBuf {
    let root = dir.join("Steam");
    let first = root.join("steamapps").join("common").join("dota 2 beta");
    let target = if second_candidate {
        root.join("common").join("dota 2 beta")
    } else {
        first
    };
    std::fs::create_dir_all(target.join("game").join("dota")).expect("dota dir");
    if manifest {
        std::fs::create_dir_all(root.join("steamapps")).expect("steamapps");
        std::fs::write(
            root.join("steamapps").join("appmanifest_570.acf"),
            "\"AppState\" {}",
        )
        .expect("manifest");
    }
    root
}

#[test]
fn discovery_finds_both_candidate_layouts() {
    let dir = tempfile::tempdir().expect("tempdir");
    let first = fake_steam_root(&dir.path().join("one"), true, false);
    let found = steam::find_dota_root_in(std::slice::from_ref(&first)).expect("found");
    assert!(found.ends_with(Path::new("steamapps").join("common").join("dota 2 beta")));

    let second = fake_steam_root(&dir.path().join("two"), true, true);
    let found = steam::find_dota_root_in(std::slice::from_ref(&second)).expect("found");
    assert!(found.ends_with(Path::new("common").join("dota 2 beta")));
}

#[test]
fn discovery_misses_without_manifest_or_game_dir() {
    let dir = tempfile::tempdir().expect("tempdir");
    let bare = fake_steam_root(&dir.path().join("bare"), false, false);
    assert!(steam::find_dota_root_in(std::slice::from_ref(&bare)).is_none());
    let missing = dir.path().join("does-not-exist");
    assert!(steam::find_dota_root_in(std::slice::from_ref(&missing)).is_none());
    // Smoke: runs against the real machine, found or not.
    let _ = steam::find_dota_root();
}

#[test]
fn discovery_reads_extra_libraries_from_vdf() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = fake_steam_root(dir.path(), true, false);
    let extra = dir.path().join("ExtraLib");
    std::fs::create_dir_all(
        extra
            .join("steamapps")
            .join("common")
            .join("dota 2 beta")
            .join("game")
            .join("dota"),
    )
    .expect("extra dota");
    std::fs::write(
        extra.join("steamapps").join("appmanifest_570.acf"),
        "\"AppState\" {}",
    )
    .expect("extra manifest");
    std::fs::write(
        root.join("steamapps").join("libraryfolders.vdf"),
        format!(
            "\"libraryfolders\"\n{{\n\t\"path\"\t\t\"{}\"\n}}\n",
            extra.display()
        ),
    )
    .expect("vdf");
    // The primary library still wins when it holds the game.
    let found = steam::find_dota_root_in(std::slice::from_ref(&root)).expect("found");
    assert!(found.starts_with(&root));

    let lonely = dir.path().join("LonelyRoot");
    std::fs::create_dir_all(lonely.join("steamapps")).expect("steamapps");
    std::fs::write(
        lonely.join("steamapps").join("libraryfolders.vdf"),
        format!(
            "\"libraryfolders\"\n{{\n\t\"path\"\t\t\"{}\"\n}}\n",
            extra.display()
        ),
    )
    .expect("vdf");
    let found = steam::find_dota_root_in(std::slice::from_ref(&lonely)).expect("found");
    assert!(found.starts_with(&extra));

    let game_root = steam::find_dota_root_in(std::slice::from_ref(&root)).expect("found");
    assert_eq!(
        steam::integration_dir(&game_root),
        game_root
            .join("game")
            .join("dota")
            .join("cfg")
            .join("gamestate_integration")
    );
}

#[test]
fn beside_supports_bare_filenames() {
    let resolved = paths::beside(Path::new("collect.db"));
    assert_eq!(resolved.db, PathBuf::from("collect.db"));
    assert_eq!(resolved.config, PathBuf::from("config.toml"));
}

#[tokio::test]
async fn token_bare_call_generates_and_nested_config_writes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("nested").join("state").join("collect.db");
    let generated = token::run(
        &TokenArgs {
            show: false,
            rotate: false,
        },
        Some(db),
    )
    .await
    .expect("generate");
    assert_eq!(generated.len(), 64);
}

#[tokio::test]
async fn active_token_rejects_invalid_toml() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = dir.path().join("config.toml");
    std::fs::write(&config, "not = [valid").expect("write");
    let err = config::active_token(&config).await.expect_err("must fail");
    assert!(err.to_string().contains("TOML"), "{err}");
}

#[tokio::test]
async fn config_install_rejects_bad_names() {
    let dir = tempfile::tempdir().expect("tempdir");
    let err = cfg::install(dir.path(), "has space", "content", false)
        .await
        .expect_err("must fail");
    assert!(err.to_string().contains("invalid config name"), "{err}");
}

#[tokio::test]
async fn install_accepts_explicit_token_and_fails_without_detection() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("dota 2 beta");
    let db = dir.path().join("collect.db");
    let written = install::run(
        &InstallArgs {
            name: "explicit".to_owned(),
            dota_dir: Some(root),
            token: Some("fixed-token".to_owned()),
            force: false,
        },
        53_001,
        Some(db),
    )
    .await
    .expect("install");
    let content = std::fs::read_to_string(&written).expect("read");
    assert!(content.contains("fixed-token"));
    assert!(content.contains("http://127.0.0.1:53001/"));

    // Hide any real Steam install so auto-detection deterministically fails.
    let previous_home = std::env::var_os("HOME");
    // SAFETY: no other test in this binary reads HOME; restored below.
    unsafe {
        std::env::set_var("HOME", dir.path().join("no-home"));
    }
    let err = install::run(
        &InstallArgs {
            name: "auto".to_owned(),
            dota_dir: None,
            token: Some("fixed-token".to_owned()),
            force: false,
        },
        53_001,
        Some(dir.path().join("other.db")),
    )
    .await
    .expect_err("auto-detect must fail without Steam");
    match previous_home {
        // SAFETY: same scope as above; restored before returning.
        Some(home) => unsafe {
            std::env::set_var("HOME", home);
        },
        None => unsafe {
            std::env::remove_var("HOME");
        },
    }
    assert!(err.to_string().contains("--dota-dir"), "{err}");
}

#[tokio::test]
async fn dispatch_runs_export_on_empty_store() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("collect.db");
    commands::run(Cli {
        port: 53_000,
        db: Some(db),
        verbose: false,
        command: Command::Export(ExportArgs {
            out: Some(dir.path().join("out")),
            match_id: None,
            kind: None,
        }),
    })
    .await
    .expect("dispatch");
    assert!(dir.path().join("out").join("frames_index.json").is_file());
}

#[tokio::test]
async fn export_scrubs_only_valid_json_payloads() {
    use dct_core::Frame;
    use dct_store::Store;

    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("collect.db");
    let store = Store::open(&db).await.expect("open");
    // Valid frame metadata, but a stored body that is not JSON at all.
    let raw = r#"{"provider": {"name": "Dota 2", "appid": 570, "version": 47, "timestamp": 1},
        "map": {"name": "dota", "matchid": "11", "game_time": 1, "clock_time": 1,
        "daytime": true, "nightstalker_night": false,
        "game_state": "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS",
        "paused": false, "win_team": "none", "customgamename": ""},
        "auth": {"token": "t"}}"#;
    let frame = Frame::from_slice(raw.as_bytes()).expect("parses");
    store
        .insert_frame(&frame, b"not json at all", 1)
        .await
        .expect("insert");

    let report = dct_cli::commands::export::run(
        &ExportArgs {
            out: Some(dir.path().join("out")),
            match_id: None,
            kind: None,
        },
        Some(db),
    )
    .await
    .expect("export");
    assert_eq!(report.frames, 1);
    let sample = std::fs::read_to_string(
        dir.path()
            .join("out")
            .join("samples")
            .join("payload_first_id1.json"),
    )
    .expect("sample");
    assert_eq!(sample, "not json at all");
}
