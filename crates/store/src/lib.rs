//! Async SQLite persistence: raw frames plus derived happenings.
//!
//! The writer side is append-only; readers serve the `logs` command.

pub mod error;
pub mod overflow;
pub mod store;

pub use error::StoreError;
pub use overflow::{ReplayStats, replay, spill};
pub use store::{
    ExportedFrame, ExportedHappening, HappeningFilter, HappeningRecord, Store, StoredHappening,
    now_millis,
};
