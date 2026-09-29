//! Typed errors for the HTTP crate.

/// Errors produced while serving ingestion requests.
#[derive(Debug, thiserror::Error)]
pub enum NetError {
    /// The socket could not be bound.
    #[error("cannot bind {addr}: {reason}")]
    Bind { addr: String, reason: String },
    /// The request could not be authenticated.
    #[error("unauthorized")]
    Unauthorized,
    /// The payload was rejected.
    #[error("bad payload: {0}")]
    BadPayload(String),
    /// Graceful shutdown failed.
    #[error("shutdown failed: {0}")]
    Shutdown(String),
}
