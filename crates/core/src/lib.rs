//! Domain types for collected game frames.
//!
//! Tolerant parsing of the full protocol surface plus derived happenings.
//! Parsing is total: unknown fields are ignored, empty objects mean absent,
//! and no valid input panics.

pub mod error;

pub use error::CoreError;
