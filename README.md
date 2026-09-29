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

## Use

```sh
# 1. Install the game config (auto-detects the Dota directory)
dct install --name dct
# Custom locations / overwrite with backup:
dct install --dota-dir "/path/to/dota 2 beta" --name dct --force

# 2. The game only emits with its integration launch flag: add it in
# Steam (Dota 2 → Properties → Launch Options), then restart the game
# after every (re)install — the client does not hot-reload endpoints.

# 3. Tokens
dct token            # generate and print a fresh token
dct token --show     # print the active token
dct token --rotate   # replace the active token

# 4. Serve
dct up               # foreground (Ctrl-C stops gracefully)
dct up --detach      # background, prints the PID
dct status           # running or stopped, plus health counters
dct down             # stop the detached server

# 5. Logs
dct logs --match 123 --kind kill --limit 20
dct logs --match 123 --json
dct logs --follow

# Point at another database (companions live beside it):
dct --db /tmp/collect.db --port 53000 up
```

## Develop

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -- install --help
```
