//! Manager CLI plus embedded server: `install`, `token`, `up`, `down`,
//! `status` and `logs` over a local SQLite store.

pub mod cfg;
pub mod cli;
pub mod commands;
pub mod config;
pub mod paths;
pub mod steam;
