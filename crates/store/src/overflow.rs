//! Overflow spillover: when the bounded queue is full, frames land here as
//! newline-delimited JSON and are replayed into the store on restart.
//!
//! Game bodies are compact single-line JSON, but replay stays tolerant:
//! malformed lines are counted and skipped, never fatal.

use std::path::Path;

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::{Store, StoreError, store::now_millis};

/// One spilled line.
#[derive(Debug)]
pub struct Spilled {
    pub received_at: i64,
    pub payload: Value,
}

/// Append one raw body to the overflow file.
pub async fn spill(path: &Path, raw: &[u8], received_at: i64) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| StoreError::Write(err.to_string()))?;
        }
    }
    let payload: Value = serde_json::from_slice(raw).unwrap_or(Value::Null);
    let line = json!({"received_at": received_at, "payload": payload}).to_string();
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .map_err(|err| StoreError::Write(err.to_string()))?;
    file.write_all(line.as_bytes())
        .await
        .map_err(|err| StoreError::Write(err.to_string()))?;
    file.write_all(b"\n")
        .await
        .map_err(|err| StoreError::Write(err.to_string()))?;
    Ok(())
}

/// Replay outcome: inserted frames, skipped lines.
#[derive(Debug, Default, PartialEq)]
pub struct ReplayStats {
    pub inserted: usize,
    pub skipped: usize,
}

/// Reinsert spilled frames, then truncate the file. Returns per-line stats.
pub async fn replay(store: &Store, path: &Path) -> Result<ReplayStats, StoreError> {
    let file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(ReplayStats::default()),
        Err(err) => return Err(StoreError::Read(err.to_string())),
    };
    let mut lines = BufReader::new(file).lines();
    let mut stats = ReplayStats::default();
    let mut previous: Option<dct_core::Frame> = None;
    while let Some(line) = lines
        .next_line()
        .await
        .map_err(|err| StoreError::Read(err.to_string()))?
    {
        if line.trim().is_empty() {
            continue;
        }
        match parse_spilled(&line) {
            Some(spilled) => {
                let raw = spilled.payload.to_string();
                match dct_core::Frame::from_slice(raw.as_bytes()) {
                    Ok(frame) => {
                        store
                            .insert_frame(&frame, raw.as_bytes(), spilled.received_at)
                            .await?;
                        let happenings = dct_core::derive_happenings(previous.as_ref(), &frame);
                        let tick = frame
                            .map
                            .as_ref()
                            .map(|map| i64::from(map.game_time))
                            .unwrap_or(0);
                        let records: Vec<crate::HappeningRecord> = happenings
                            .iter()
                            .map(|event| crate::HappeningRecord {
                                match_id: frame.match_id().unwrap_or(""),
                                tick,
                                happening: event,
                                recorded_at: spilled.received_at,
                            })
                            .collect();
                        store.insert_happenings(&records).await?;
                        previous = Some(frame);
                        stats.inserted += 1;
                    }
                    Err(_) => stats.skipped += 1,
                }
            }
            None => stats.skipped += 1,
        }
    }
    tokio::fs::write(path, b"")
        .await
        .map_err(|err| StoreError::Write(err.to_string()))?;
    Ok(stats)
}

fn parse_spilled(line: &str) -> Option<Spilled> {
    let value: Value = serde_json::from_str(line).ok()?;
    let received_at = value
        .get("received_at")?
        .as_i64()
        .unwrap_or_else(now_millis);
    let payload = value.get("payload")?.clone();
    Some(Spilled {
        received_at,
        payload,
    })
}
