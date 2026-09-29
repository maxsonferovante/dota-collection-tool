# dota-collection-tool

Local game-state collector: an HTTP sink for the game's JSON POSTs, a manager
CLI (`install`, `token`, `up`, `down`, `status`, `logs`) and an async SQLite
store of raw frames plus derived happenings.

The game client only emits when started with its documented integration
launch flag (official March 2022 update, per-frame performance reasons).

## Layout

- `crates/core` — domain types plus tolerant parsing (no async runtime)
- `crates/net` — HTTP ingestion server
- `crates/store` — async SQLite persistence
- `crates/cli` — manager CLI plus embedded server (`dct` binary)

## Develop

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -- install --help
```
