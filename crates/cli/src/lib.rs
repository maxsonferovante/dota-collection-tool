//! Manager CLI plus embedded server: `install`, `token`, `up`, `down`,
//! `status`, `logs` and `export` over a local SQLite store.

pub mod cfg;
pub mod cli;
pub mod commands;
pub mod config;
pub mod lifecycle;
pub mod paths;
pub mod steam;
