//! `export`: dump the frames index, happenings and sample payloads.
//!
//! Reproduces the `exports/` layout: `frames_index.json` (one row per frame
//! with `payload_bytes`), `happenings.json` (full rows, detail kept as the
//! stored string) and `samples/` with the first, middle and last payloads.
//! Sample payloads carry the auth token redacted: the game echoes it on
//! every POST, and exports are meant to be shared.

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::cli::ExportArgs;
use crate::paths;
use dct_store::Store;

/// Where the export landed plus row counts.
#[derive(Debug)]
pub struct ExportReport {
    pub dir: PathBuf,
    pub frames: usize,
    pub happenings: usize,
}

/// Scrub the echoed auth token from a stored payload for export.
fn scrubbed(payload: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(payload) {
        Ok(mut value) => {
            if let Some(token) = value.get_mut("auth").and_then(|auth| auth.get_mut("token")) {
                *token = serde_json::Value::String(String::from("redacted"));
            }
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| payload.to_owned())
        }
        Err(_) => payload.to_owned(),
    }
}

async fn write_json(path: &PathBuf, value: &serde_json::Value) -> Result<()> {
    let raw = serde_json::to_string_pretty(value).context("cannot encode export")?;
    tokio::fs::write(path, raw)
        .await
        .with_context(|| format!("cannot write {}", path.display()))?;
    Ok(())
}

/// Run `export`: dump everything (optionally scoped to a match) into `out`.
pub async fn run(args: &ExportArgs, db_override: Option<PathBuf>) -> Result<ExportReport> {
    let paths = paths::resolve(db_override)?;
    let store = Store::open(&paths.db)
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;
    let match_id = args.match_id.as_deref();
    let frames = store
        .export_frames(match_id)
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;
    let happenings = store
        .export_happenings(match_id, args.kind.as_deref())
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;

    let dir = match &args.out {
        Some(out) => out.clone(),
        None => match paths.db.parent().filter(|dir| !dir.as_os_str().is_empty()) {
            Some(parent) => parent.join("exports"),
            None => PathBuf::from("exports"),
        },
    };
    tokio::fs::create_dir_all(dir.join("samples"))
        .await
        .with_context(|| format!("cannot create {}", dir.display()))?;

    let index: Vec<serde_json::Value> = frames
        .iter()
        .map(|frame| {
            serde_json::json!({
                "id": frame.id,
                "match_id": frame.match_id,
                "game_time": frame.game_time,
                "clock_time": frame.clock_time,
                "received_at": frame.received_at,
                "payload_bytes": frame.payload.len(),
            })
        })
        .collect();
    write_json(
        &dir.join("frames_index.json"),
        &serde_json::Value::Array(index),
    )
    .await?;

    let rows: Vec<serde_json::Value> = happenings
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.id,
                "match_id": row.match_id,
                "tick": row.tick,
                "kind": row.kind,
                "actor": row.actor,
                "detail": row.detail,
                "recorded_at": row.recorded_at,
            })
        })
        .collect();
    write_json(
        &dir.join("happenings.json"),
        &serde_json::Value::Array(rows),
    )
    .await?;

    if let (Some(first), Some(last)) = (frames.first(), frames.last()) {
        let middle = &frames[frames.len() / 2];
        for (label, frame) in [("first", first), ("middle", middle), ("last", last)] {
            let name = format!("payload_{label}_id{}.json", frame.id);
            let path = dir.join("samples").join(name);
            tokio::fs::write(&path, scrubbed(&frame.payload))
                .await
                .with_context(|| format!("cannot write {}", path.display()))?;
        }
    }

    let report = ExportReport {
        dir,
        frames: frames.len(),
        happenings: happenings.len(),
    };
    println!(
        "exported {} frames and {} happenings to {}",
        report.frames,
        report.happenings,
        report.dir.display()
    );
    Ok(report)
}
