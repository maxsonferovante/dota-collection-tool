//! Async SQLite persistence: raw frames plus derived happenings.
//!
//! The writer side is append-only; readers serve the `logs` command.

pub mod error;

pub use error::StoreError;
