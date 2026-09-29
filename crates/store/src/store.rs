//! Append-only store: raw frames plus derived happenings.
//!
//! [`Store::open`] targets a file database (creating parent directories and
//! running migrations); [`Store::open_in_memory`] is the test seam.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use dct_core::{Frame, Happening};
use serde_json::Value;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::str::FromStr;

use crate::StoreError;

/// Current wall-clock time in milliseconds since the Unix epoch.
pub fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|span| span.as_millis() as i64)
        .unwrap_or(0)
}

/// One happening ready to persist.
pub struct HappeningRecord<'a> {
    pub match_id: &'a str,
    pub tick: i64,
    pub happening: &'a Happening,
    pub recorded_at: i64,
}

/// Filter for [`Store::query_happenings`].
#[derive(Debug, Default)]
pub struct HappeningFilter {
    pub match_id: Option<String>,
    pub kind: Option<String>,
    pub limit: usize,
}

/// One stored happening, most recent last.
#[derive(Debug, PartialEq)]
pub struct StoredHappening {
    pub id: i64,
    pub match_id: String,
    pub tick: i64,
    pub kind: String,
    pub actor: Option<String>,
    pub detail: Value,
}

/// Async SQLite store.
#[derive(Clone)]
pub struct Store {
    pool: SqlitePool,
}

impl Store {
    async fn connect(options: SqliteConnectOptions) -> Result<Self, StoreError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await
            .map_err(|err| StoreError::Open(err.to_string()))?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|err| StoreError::Open(err.to_string()))?;
        Ok(Self { pool })
    }

    /// Open (creating) a file database.
    pub async fn open(path: &Path) -> Result<Self, StoreError> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|err| StoreError::Open(err.to_string()))?;
            }
        }
        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
            .map_err(|err| StoreError::Open(err.to_string()))?
            .create_if_missing(true);
        Self::connect(options).await
    }

    /// Open an isolated in-memory database (tests).
    pub async fn open_in_memory() -> Result<Self, StoreError> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .map_err(|err| StoreError::Open(err.to_string()))?
            .shared_cache(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(|err| StoreError::Open(err.to_string()))?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|err| StoreError::Open(err.to_string()))?;
        Ok(Self { pool })
    }

    /// Append one accepted frame plus its raw body. Returns the row id.
    pub async fn insert_frame(
        &self,
        frame: &Frame,
        raw: &[u8],
        received_at: i64,
    ) -> Result<i64, StoreError> {
        let (match_id, game_time, clock_time) = match frame.map.as_ref() {
            Some(map) => (
                map.match_id.clone(),
                i64::from(map.game_time),
                i64::from(map.clock_time),
            ),
            None => (String::new(), 0, 0),
        };
        let payload = std::str::from_utf8(raw).map_err(|err| StoreError::Write(err.to_string()))?;
        let id: i64 = sqlx::query(
            "INSERT INTO frames (match_id, game_time, clock_time, received_at, payload)
             VALUES (?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(match_id)
        .bind(game_time)
        .bind(clock_time)
        .bind(received_at)
        .bind(payload)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| StoreError::Write(err.to_string()))?
        .get("id");
        Ok(id)
    }

    /// Append derived happenings in one statement batch.
    pub async fn insert_happenings(
        &self,
        records: &[HappeningRecord<'_>],
    ) -> Result<(), StoreError> {
        for record in records {
            sqlx::query(
                "INSERT INTO happenings (match_id, tick, kind, actor, detail, recorded_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(record.match_id)
            .bind(record.tick)
            .bind(record.happening.kind.as_str())
            .bind(record.happening.actor.clone())
            .bind(record.happening.detail.to_string())
            .bind(record.recorded_at)
            .execute(&self.pool)
            .await
            .map_err(|err| StoreError::Write(err.to_string()))?;
        }
        Ok(())
    }

    /// Read happenings oldest-first, honoring the filter.
    pub async fn query_happenings(
        &self,
        filter: &HappeningFilter,
    ) -> Result<Vec<StoredHappening>, StoreError> {
        let limit = filter.limit.clamp(1, 10_000) as i64;
        let rows = match (&filter.match_id, &filter.kind) {
            (Some(match_id), Some(kind)) => {
                sqlx::query(
                    "SELECT id, match_id, tick, kind, actor, detail FROM happenings
                     WHERE match_id = ? AND kind = ? ORDER BY id ASC LIMIT ?",
                )
                .bind(match_id)
                .bind(kind)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (Some(match_id), None) => {
                sqlx::query(
                    "SELECT id, match_id, tick, kind, actor, detail FROM happenings
                     WHERE match_id = ? ORDER BY id ASC LIMIT ?",
                )
                .bind(match_id)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(kind)) => {
                sqlx::query(
                    "SELECT id, match_id, tick, kind, actor, detail FROM happenings
                     WHERE kind = ? ORDER BY id ASC LIMIT ?",
                )
                .bind(kind)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, None) => {
                sqlx::query(
                    "SELECT id, match_id, tick, kind, actor, detail FROM happenings
                     ORDER BY id ASC LIMIT ?",
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|err| StoreError::Read(err.to_string()))?;

        rows.into_iter()
            .map(|row| {
                let detail_raw: String = row.get("detail");
                let detail: Value = serde_json::from_str(&detail_raw).unwrap_or(Value::Null);
                Ok(StoredHappening {
                    id: row.get("id"),
                    match_id: row.get("match_id"),
                    tick: row.get("tick"),
                    kind: row.get("kind"),
                    actor: row.get("actor"),
                    detail,
                })
            })
            .collect()
    }
}
