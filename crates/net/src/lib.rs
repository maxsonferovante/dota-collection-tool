//! HTTP ingestion: accepts game POSTs and forwards frames to the store.
//!
//! The handler never blocks on the database; backpressure is bounded with
//! a counted overflow path. Success responses use the content type the
//! game client expects.

pub mod error;
pub mod server;

pub use error::NetError;
pub use server::{DEFAULT_QUEUE_CAPACITY, IngestConfig, SUCCESS_CONTENT_TYPE, run, run_on};
