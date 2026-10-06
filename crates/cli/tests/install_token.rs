//! `install` plus `token` against temporary directories.

use std::path::PathBuf;

use dct_cli::cfg;
use dct_cli::cli::{InstallArgs, TokenArgs};
use dct_cli::commands::{install, token};
use dct_cli::steam;

#[test]
fn config_names_are_restricted() {
    assert!(cfg::valid_name("dct"));
    assert!(cfg::valid_name("my-collector_2"));
    assert!(!cfg::valid_name(""));
    assert!(!cfg::valid_name("has space"));
    assert!(!cfg::valid_name("semi;colon"));
    assert!(!cfg::valid_name("dot.cfg"));
}

#[test]
fn render_covers_all_blocks_and_escapes() {
    let raw = cfg::render("dct", "http://127.0.0.1:53000/", "tok\"en\\");
    for block in [
        "auth",
        "provider",
        "map",
        "player",
        "hero",
        "abilities",
        "items",
        "events",
        "buildings",
        "league",
        "draft",
        "wearables",
        "minimap",
        "roshan",
        "couriers",
        "neutralitems",
    ] {
        assert!(raw.contains(&format!("\"{block}\"")), "missing {block}");
    }
    assert!(raw.contains("http://127.0.0.1:53000/"));
    assert!(raw.contains("tok\\\"en\\\\"));
}

#[test]
fn profiles_have_conservative_settings() {
    assert_eq!(cfg::Profile::Economical.settings().unwrap().buffer, 0.20);
    assert_eq!(cfg::Profile::Economical.settings().unwrap().throttle, 0.20);
    assert_eq!(cfg::Profile::Balanced.settings().unwrap().buffer, 0.10);
    assert_eq!(cfg::Profile::Balanced.settings().unwrap().throttle, 0.10);
    assert_eq!(cfg::Profile::LowLatency.settings().unwrap().buffer, 0.02);
    assert_eq!(cfg::Profile::LowLatency.settings().unwrap().throttle, 0.05);
    assert!(cfg::Profile::Custom.settings().is_none());
}

#[tokio::test]
async fn profile_defaults_and_round_trips_without_losing_token() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = dir.path().join("config.toml");

    assert_eq!(
        dct_cli::config::active_profile(&config).await.unwrap(),
        cfg::Profile::Balanced
    );
    let token = dct_cli::config::fresh_token(&config).await.unwrap();
    dct_cli::config::set_profile(&config, cfg::Profile::LowLatency)
        .await
        .unwrap();

    assert_eq!(
        dct_cli::config::active_profile(&config).await.unwrap(),
        cfg::Profile::LowLatency
    );
    assert_eq!(
        dct_cli::config::active_token(&config).await.unwrap(),
        Some(token)
    );
}

#[test]
fn library_paths_parse_old_and_new_manifests() {
    let old = "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"D:\\Steam\"\n\t}\n}\n";
    assert_eq!(steam::library_paths(old), vec![PathBuf::from("D:\\Steam")]);
    let new = "\"libraryfolders\"\n{\n\t\"contentstatsid\" \"-1\"\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"/media/games/SteamLibrary\"\n\t}\n}\n";
    assert_eq!(
        steam::library_paths(new),
        vec![PathBuf::from("/media/games/SteamLibrary")]
    );
    assert!(steam::library_paths("nothing quoted here").is_empty());
}

#[tokio::test]
async fn install_and_token_flow() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("dota 2 beta");
    let db = dir.path().join("state").join("collect.db");
    let base = InstallArgs {
        name: "test".to_owned(),
        dota_dir: Some(root.clone()),
        token: None,
        force: false,
    };

    let written = install::run(&base, 53_000, Some(db.clone()))
        .await
        .expect("install");
    assert!(written.is_file());
    let content = std::fs::read_to_string(&written).expect("read");
    assert!(content.contains("http://127.0.0.1:53000/"));

    let shown = token::run(
        &TokenArgs {
            show: true,
            rotate: false,
        },
        Some(db.clone()),
    )
    .await
    .expect("show");
    assert!(shown.len() == 64);
    assert!(content.contains(&shown));

    assert!(install::run(&base, 53_000, Some(db.clone())).await.is_err());

    let rotated = token::run(
        &TokenArgs {
            show: false,
            rotate: true,
        },
        Some(db.clone()),
    )
    .await
    .expect("rotate");
    assert_ne!(rotated, shown);

    let forced = InstallArgs {
        force: true,
        ..base
    };
    let rewritten = install::run(&forced, 53_000, Some(db))
        .await
        .expect("force install");
    assert_eq!(rewritten, written);
    assert!(written.with_extension("cfg.bak").is_file());
    let updated = std::fs::read_to_string(&written).expect("read");
    assert!(updated.contains(&rotated));
    assert!(!updated.contains(&shown));
}
