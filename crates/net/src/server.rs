//! Ingestion server: one POST endpoint plus health.
//!
//! The handler answers fast and never touches the database: accepted bodies
//! travel through a bounded queue to a single writer task. A full queue
//! spills to the overflow file with a counter, and the game still gets its
//! success response.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use dct_core::{Frame, derive_happenings};
use dct_store::{HappeningRecord, Store, now_millis};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

/// Content type of success responses, as the game client expects.
pub const SUCCESS_CONTENT_TYPE: &str = "text/html";

/// How many frames wait for the writer before spilling to disk.
pub const DEFAULT_QUEUE_CAPACITY: usize = 1024;

/// Server tunables.
#[derive(Debug, Clone)]
pub struct IngestConfig {
    /// Token compared against `auth.token` of every POST.
    pub token: String,
    /// Where full-queue frames spill as newline-delimited JSON.
    pub overflow_path: PathBuf,
    /// Bounded queue depth between HTTP and writer.
    pub queue_capacity: usize,
}

/// Counters for the ingestion pipeline.
#[derive(Debug, Default)]
pub struct Metrics {
    received: AtomicU64,
    stored: AtomicU64,
    dropped: AtomicU64,
    rejected: AtomicU64,
    invalid: AtomicU64,
}

/// Point-in-time copy of [`Metrics`].
#[derive(Debug, PartialEq, Serialize)]
pub struct MetricsSnapshot {
    pub received: u64,
    pub stored: u64,
    pub dropped: u64,
    pub rejected: u64,
    pub invalid: u64,
}

impl Metrics {
    fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            received: self.received.load(Ordering::Relaxed),
            stored: self.stored.load(Ordering::Relaxed),
            dropped: self.dropped.load(Ordering::Relaxed),
            rejected: self.rejected.load(Ordering::Relaxed),
            invalid: self.invalid.load(Ordering::Relaxed),
        }
    }
}

struct QueuedFrame {
    raw: Vec<u8>,
    received_at: i64,
}

struct AppState {
    store: Store,
    token: String,
    sender: mpsc::Sender<QueuedFrame>,
    metrics: Metrics,
    overflow_path: PathBuf,
}

#[derive(Deserialize)]
struct Envelope {
    #[serde(default)]
    auth: Option<TokenField>,
}

#[derive(Deserialize)]
struct TokenField {
    #[serde(default)]
    token: Option<String>,
}

/// Minimal token check without parsing the whole frame.
fn authorized(body: &[u8], expected: &str) -> Option<bool> {
    let envelope: Envelope = serde_json::from_slice(body).ok()?;
    Some(envelope.auth.and_then(|field| field.token).as_deref() == Some(expected))
}

async fn ingest(State(state): State<Arc<AppState>>, body: Bytes) -> Response {
    let received_at = now_millis();
    state.metrics.received.fetch_add(1, Ordering::Relaxed);
    match authorized(&body, &state.token) {
        Some(true) => {}
        Some(false) => {
            state.metrics.rejected.fetch_add(1, Ordering::Relaxed);
            return StatusCode::UNAUTHORIZED.into_response();
        }
        // Malformed bodies still get success so the game keeps flowing;
        // the writer counts them as invalid and stores nothing.
        None => {}
    }
    let frame = QueuedFrame {
        raw: body.to_vec(),
        received_at,
    };
    match state.sender.try_send(frame) {
        Ok(()) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, SUCCESS_CONTENT_TYPE)],
            "OK",
        )
            .into_response(),
        Err(_) => {
            state.metrics.dropped.fetch_add(1, Ordering::Relaxed);
            if let Err(err) = dct_store::spill(&state.overflow_path, &body, received_at).await {
                tracing::warn!(error = %err, "overflow spill failed");
            }
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, SUCCESS_CONTENT_TYPE)],
                "OK",
            )
                .into_response()
        }
    }
}

async fn health(State(state): State<Arc<AppState>>) -> Response {
    let snapshot = state.metrics.snapshot();
    let body = serde_json::json!({
        "status": "ok",
        "received": snapshot.received,
        "stored": snapshot.stored,
        "dropped": snapshot.dropped,
        "rejected": snapshot.rejected,
        "invalid": snapshot.invalid,
    })
    .to_string();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        body,
    )
        .into_response()
}

async fn store_one(state: &AppState, previous: &mut HashMap<String, Frame>, queued: QueuedFrame) {
    let parsed = match Frame::from_slice(&queued.raw) {
        Ok(frame) => frame,
        Err(_) => {
            state.metrics.invalid.fetch_add(1, Ordering::Relaxed);
            return;
        }
    };
    let key = parsed.match_id().unwrap_or("").to_owned();
    let tick = parsed
        .map
        .as_ref()
        .map(|map| i64::from(map.game_time))
        .unwrap_or(0);
    let happenings = derive_happenings(previous.get(&key), &parsed);
    let records: Vec<HappeningRecord> = happenings
        .iter()
        .map(|event| HappeningRecord {
            match_id: parsed.match_id().unwrap_or(""),
            tick,
            happening: event,
            recorded_at: queued.received_at,
        })
        .collect();
    let outcome = store_frame_and_events(state, &parsed, &queued, &records).await;
    if let Err(err) = outcome {
        tracing::warn!(error = %err, "frame store failed");
        return;
    }
    previous.insert(key, parsed);
    state.metrics.stored.fetch_add(1, Ordering::Relaxed);
}

async fn store_frame_and_events(
    state: &AppState,
    parsed: &Frame,
    queued: &QueuedFrame,
    records: &[HappeningRecord<'_>],
) -> Result<(), dct_store::StoreError> {
    state
        .store
        .insert_frame(parsed, &queued.raw, queued.received_at)
        .await?;
    state.store.insert_happenings(records).await?;
    Ok(())
}

async fn writer_task(state: Arc<AppState>, mut inbox: mpsc::Receiver<QueuedFrame>) {
    let mut previous: HashMap<String, Frame> = HashMap::new();
    while let Some(queued) = inbox.recv().await {
        store_one(&state, &mut previous, queued).await;
    }
}

fn router(state: Arc<AppState>) -> axum::Router {
    axum::Router::new()
        .route("/", post(ingest))
        .route("/health", get(health))
        .with_state(state)
}

/// Serve on a bound address until `shutdown` resolves.
pub async fn run(
    addr: SocketAddr,
    store: Store,
    config: IngestConfig,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> Result<(), crate::NetError> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| crate::NetError::Bind {
            addr: addr.to_string(),
            reason: err.to_string(),
        })?;
    run_on(listener, store, config, shutdown).await
}

/// Serve on an already-bound listener (the test seam for ephemeral ports).
pub async fn run_on(
    listener: TcpListener,
    store: Store,
    config: IngestConfig,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> Result<(), crate::NetError> {
    let (sender, inbox) = mpsc::channel(config.queue_capacity);
    let state = Arc::new(AppState {
        store,
        token: config.token,
        sender,
        metrics: Metrics::default(),
        overflow_path: config.overflow_path,
    });
    tokio::spawn(writer_task(state.clone(), inbox));
    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown)
        .await
        .map_err(|err| crate::NetError::Shutdown(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_check_behaviour() {
        assert_eq!(authorized(br#"{"auth":{"token":"a"}}"#, "a"), Some(true));
        assert_eq!(authorized(br#"{"auth":{"token":"b"}}"#, "a"), Some(false));
        assert_eq!(authorized(br#"{}"#, "a"), Some(false));
        assert_eq!(authorized(b"{broken", "a"), None);
    }

    async fn test_state() -> (Arc<AppState>, mpsc::Receiver<QueuedFrame>) {
        let store = Store::open_in_memory().await.expect("open");
        let (sender, inbox) = mpsc::channel(4);
        let state = Arc::new(AppState {
            store,
            token: "a".to_owned(),
            sender,
            metrics: Metrics::default(),
            overflow_path: PathBuf::from("overflow.jsonl"),
        });
        (state, inbox)
    }

    const FRAME: &str = r#"{
        "provider": {"name": "Dota 2", "appid": 570, "version": 47, "timestamp": 1},
        "map": {"name": "dota", "matchid": "3", "game_time": 5, "clock_time": 5,
            "daytime": true, "nightstalker_night": false,
            "game_state": "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS",
            "paused": false, "win_team": "none", "customgamename": ""},
        "auth": {"token": "a"}
    }"#;

    #[tokio::test]
    async fn writer_counts_stored_and_invalid() {
        let (state, _inbox) = test_state().await;
        let mut previous = HashMap::new();
        store_one(
            &state,
            &mut previous,
            QueuedFrame {
                raw: FRAME.as_bytes().to_vec(),
                received_at: 1,
            },
        )
        .await;
        store_one(
            &state,
            &mut previous,
            QueuedFrame {
                raw: b"{broken".to_vec(),
                received_at: 2,
            },
        )
        .await;
        let snapshot = state.metrics.snapshot();
        assert_eq!(
            snapshot,
            MetricsSnapshot {
                received: 0,
                stored: 1,
                dropped: 0,
                rejected: 0,
                invalid: 1,
            }
        );
    }
}
