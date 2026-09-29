//! Typed errors for the persistence crate.

/// Errors produced by the frame store.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The database could not be opened or migrated.
    #[error("cannot open database: {0}")]
    Open(String),
    /// A write failed after the overflow path was exhausted.
    #[error("write failed: {0}")]
    Write(String),
    /// A read failed.
    #[error("read failed: {0}")]
    Read(String),
}
