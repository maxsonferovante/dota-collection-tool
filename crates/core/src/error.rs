//! Typed errors for the domain crate.

/// Errors produced while parsing or validating collected frames.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// The payload is not valid JSON.
    #[error("invalid JSON payload: {0}")]
    InvalidJson(#[from] serde_json::Error),
    /// The payload is valid JSON but has no usable content.
    #[error("empty frame: no usable content")]
    Empty,
}
