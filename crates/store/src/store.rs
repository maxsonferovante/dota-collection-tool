//! Append-only store: raw frames plus derived happenings.
//!
//! [`Store::open`] targets a file database (creating parent directories and
//! running migrations); [`Store::open_in_memory`] is the test seam.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use dct_core::{Frame, Happening, classify};
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
    /// Only rows newer than this id (powers `--follow`).
    pub after_id: Option<i64>,
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

/// One raw frame for export: index columns plus the stored payload.
#[derive(Debug, PartialEq)]
pub struct ExportedFrame {
    pub id: i64,
    pub match_id: String,
    pub game_time: i64,
    pub clock_time: i64,
    pub received_at: i64,
    pub payload: String,
    pub payload_kind: String,
    pub normalized_payload: String,
}

/// One happening for export, with the detail kept as the stored string.
#[derive(Debug, PartialEq)]
pub struct ExportedHappening {
    pub id: i64,
    pub match_id: String,
    pub tick: i64,
    pub kind: String,
    pub actor: Option<String>,
    pub detail: String,
    pub recorded_at: i64,
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
        let normalized_payload =
            serde_json::to_string(frame).map_err(|err| StoreError::Write(err.to_string()))?;
        let id: i64 = sqlx::query(
            "INSERT INTO frames
             (match_id, game_time, clock_time, received_at, payload, payload_kind, normalized_payload)
             VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(match_id)
        .bind(game_time)
        .bind(clock_time)
        .bind(received_at)
        .bind(payload)
        .bind(classify(frame).as_str())
        .bind(normalized_payload)
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
        let mut sql =
            String::from("SELECT id, match_id, tick, kind, actor, detail FROM happenings");
        let mut conditions = Vec::new();
        if filter.match_id.is_some() {
            conditions.push("match_id = ?");
        }
        if filter.kind.is_some() {
            conditions.push("kind = ?");
        }
        if filter.after_id.is_some() {
            conditions.push("id > ?");
        }
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY id ASC LIMIT ?");
        // Audited: `sql` only concatenates static fragments above; every
        // dynamic value travels as a bound parameter below.
        let mut query = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()));
        if let Some(match_id) = &filter.match_id {
            query = query.bind(match_id);
        }
        if let Some(kind) = &filter.kind {
            query = query.bind(kind);
        }
        if let Some(after_id) = filter.after_id {
            query = query.bind(after_id);
        }
        query = query.bind(limit);
        let rows = query
            .fetch_all(&self.pool)
            .await
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

    /// Read every frame oldest-first for export, optionally scoped to a match.
    pub async fn export_frames(
        &self,
        match_id: Option<&str>,
    ) -> Result<Vec<ExportedFrame>, StoreError> {
        let mut sql = String::from(
            "SELECT id, match_id, game_time, clock_time, received_at, payload,
                    payload_kind, normalized_payload FROM frames",
        );
        if match_id.is_some() {
            sql.push_str(" WHERE match_id = ?");
        }
        sql.push_str(" ORDER BY id ASC");
        // Audited: `sql` only concatenates the static fragment above; the
        // match id travels as a bound parameter below.
        let mut query = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()));
        if let Some(match_id) = match_id {
            query = query.bind(match_id);
        }
        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|err| StoreError::Read(err.to_string()))?;
        rows.into_iter()
            .map(|row| {
                Ok(ExportedFrame {
                    id: row.get("id"),
                    match_id: row.get("match_id"),
                    game_time: row.get("game_time"),
                    clock_time: row.get("clock_time"),
                    received_at: row.get("received_at"),
                    payload: row.get("payload"),
                    payload_kind: row.get("payload_kind"),
                    normalized_payload: row.get("normalized_payload"),
                })
            })
            .collect()
    }

    /// Read every happening oldest-first for export, honoring the filters.
    pub async fn export_happenings(
        &self,
        match_id: Option<&str>,
        kind: Option<&str>,
    ) -> Result<Vec<ExportedHappening>, StoreError> {
        let mut sql = String::from(
            "SELECT id, match_id, tick, kind, actor, detail, recorded_at FROM happenings",
        );
        let mut conditions = Vec::new();
        if match_id.is_some() {
            conditions.push("match_id = ?");
        }
        if kind.is_some() {
            conditions.push("kind = ?");
        }
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY id ASC");
        // Audited: `sql` only concatenates static fragments above; every
        // dynamic value travels as a bound parameter below.
        let mut query = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()));
        if let Some(match_id) = match_id {
            query = query.bind(match_id);
        }
        if let Some(kind) = kind {
            query = query.bind(kind);
        }
        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|err| StoreError::Read(err.to_string()))?;
        rows.into_iter()
            .map(|row| {
                Ok(ExportedHappening {
                    id: row.get("id"),
                    match_id: row.get("match_id"),
                    tick: row.get("tick"),
                    kind: row.get("kind"),
                    actor: row.get("actor"),
                    detail: row.get("detail"),
                    recorded_at: row.get("recorded_at"),
                })
            })
            .collect()
    }
}
