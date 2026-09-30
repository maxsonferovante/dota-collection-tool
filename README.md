# dota-collection-tool

Local game-state collector: an HTTP sink for the game's JSON POSTs, a manager
CLI (`install`, `token`, `up`, `down`, `status`, `logs`, `export`) and an
async SQLite store of raw frames plus derived happenings.

State (`collect.db`, `config.toml`, ...) lives next to the executable by
default, so a portable install keeps everything in one folder; override
with `--db` (companions follow the database).

## Layout

- `crates/core` — domain types plus tolerant parsing (no async runtime)
- `crates/net` — HTTP ingestion server
- `crates/store` — async SQLite persistence
- `crates/cli` — manager CLI plus embedded server (`dct` binary)

## Setup with the game

Three things must line up: the config file, the launch flag, and the server.

### 1. Install the game config

```sh
dct install --name dct
```

This auto-detects the Dota directory and writes
`.../game/dota/cfg/gamestate_integration/gamestate_integration_dct.cfg`
(pointing at `http://127.0.0.1:53000/` with all data blocks on).
Custom locations and overwrite with backup:

```sh
dct install --dota-dir "/path/to/dota 2 beta" --name dct --force
```

### 2. Enable the `-gamestateintegration` launch flag (required)

Since the official March 2022 update, the client only emits game state
when started with its integration launch flag (per-frame performance
reasons). Without it, nothing is sent — no error, just silence.

1. Open **Steam** → **Library** → right-click **Dota 2** → **Properties**.
2. In the **General** tab, find **Launch Options**.
3. Add exactly:
   `-gamestateintegration`
4. Close the window and **restart the game completely**.

Restart the game again after every `(re)install` — the client does not
hot-reload an already-registered endpoint.

### 3. Serve and collect

```sh
dct up --detach   # background, prints the PID (or: dct up for foreground)
dct status        # must say "running"
```

Play or spectate a match (spectating records all 10 players; playing
records only your own), then:

```sh
dct logs --match <match-id> --limit 20
dct logs --follow
dct down
```

### Troubleshooting: no data arriving

1. `dct status` — is the server actually running?
2. Launch flag present **and** game restarted after adding it?
3. Config file present at the path printed by `install`?
4. Token match: `dct token --show` equals the `"token"` in the `.cfg`
   (after `token --rotate`, rerun `install --force` and restart the game).
5. Port clash: another service on `53000`? Use `--port` consistently in
   `install` and `up`.

## CLI reference

Field meaning, types and notes: [`docs/08-dicionario-de-dados.md`](docs/08-dicionario-de-dados.md).

```sh
# Tokens
dct token            # generate and print a fresh token
dct token --show     # print the active token
dct token --rotate   # replace the active token

# Serve
dct up               # foreground (Ctrl-C stops gracefully)
dct up --detach      # background, prints the PID
dct status           # running or stopped, plus health counters
dct down             # stop the detached server

# Logs
dct logs --match 123 --kind kill --limit 20
dct logs --match 123 --json
dct logs --follow

# Export (frames index, happenings, first/middle/last payloads)
dct export                        # into ./exports next to the database
dct export --out ./my-exports --match 123 --kind kill

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
