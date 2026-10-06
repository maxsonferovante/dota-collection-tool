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

The GSI processing and storage contract is documented in
[`docs/09-gsi-processamento-e-persistencia.md`](docs/09-gsi-processamento-e-persistencia.md).
Each accepted frame stores the original JSON, the normalized Rust model and
the detected client mode (`playing`, `spectating`, `post_game` or `unknown`).

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

### Collection profiles

The desktop app offers three GSI collection profiles. **Balanced** is the
default and is recommended for most users:

| Profile | buffer | throttle | heartbeat | Recommendation |
| --- | ---: | ---: | ---: | --- |
| Economical | `0.20` | `0.20` | `30.0` | Lowest local update pressure; use for normal play. |
| Balanced | `0.10` | `0.10` | `30.0` | General-purpose compromise between freshness and impact. |
| Low latency | `0.02` | `0.05` | `15.0` | Faster live analysis, with potentially higher local overhead. |

Low latency is not a real-time guarantee. Actual behavior depends on the Dota
version and operating system, and the profile can affect local processing,
storage and the game's frame-time. Change the profile in **Connect Dota 2**,
install the config, then restart Dota. The app's status panel reports whether
it is waiting for the first payload, collecting, or receiving no recent data,
along with approximate rate and local drop/error counters.

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

## Desktop app (no command line)

`dct-gui` is the same collector with a window instead of a terminal:

1. **Collector service** — shows Running/Stopped, Start/Stop/Refresh,
   auto-refresh every 3 s, and the port (default `53000`).
2. **Dota install folder** — Browse... (native folder picker with no
   quoting issues), Auto-detect, then Install game config. Pick the
   `dota 2 beta` folder; spaces in the path are handled.
3. **Export JSON** — output folder picker (or Default), optional
   match/kind filters, Export, plus Open folder.

```sh
cargo run -p dct-gui   # local run
```

State lives next to the app, shared with the `dct` CLI, so both can be
used interchangeably. After installing, still add
`-gamestateintegration` to the Steam launch options and restart the game.

### First run on macOS

Binaries downloaded from GitHub Releases arrive without the executable
bit and under macOS quarantine (Gatekeeper), so double-clicking fails.
Fix both once per download (Apple Silicon uses the `arm64` files;
`x86_64` is for Intel Macs, `.exe` for Windows):

```sh
cd ~/Downloads
chmod +x dct-gui-macos-arm64 dct-macos-arm64
xattr -d com.apple.quarantine dct-gui-macos-arm64 dct-macos-arm64
```

Then run `./dct-gui-macos-arm64` (or double-click it in Finder). If macOS
still refuses to open it ("cannot verify the developer"), right-click the
file → Open → Open to approve it once.

## Release builds (Windows, Linux, macOS)

Cross-compiling uses `zig` as the linker, so no MinGW/MSVC toolchain is
needed. One-time setup:

```sh
rustup target add x86_64-pc-windows-gnu x86_64-unknown-linux-gnu
cargo install cargo-zigbuild   # plus: brew install zig
```

```sh
# Windows x86-64 (portable .exe: state lives next to it)
cargo zigbuild --release --target x86_64-pc-windows-gnu -p dct-cli
# -> target/x86_64-pc-windows-gnu/release/dct.exe

# Linux x86-64
cargo zigbuild --release --target x86_64-unknown-linux-gnu -p dct-cli
# -> target/x86_64-unknown-linux-gnu/release/dct

# macOS native (arm64 on Apple Silicon)
cargo build --release -p dct-cli
# -> target/release/dct
```

(For Intel Macs add `--target x86_64-apple-darwin`, no `zigbuild` needed.)

## Published releases (manual)

Tagged releases are built by the `Release binaries` workflow
(`.github/workflows/release.yml`), which runs **only when triggered by
hand** (Actions tab → Run workflow). It takes the latest `main`, then:
1. runs the full test suite with a **90% line-coverage gate**
   (`cargo llvm-cov --workspace --exclude dct-gui --fail-under-lines 90`;
   the GUI shell is excluded — it cannot run headless on Linux — but its
   logic lives in the tested `dct-cli` library),
2. builds `dct-gui` + `dct` for Windows x86-64 and macOS (arm64 + Intel),
3. creates the tag (`version` input, e.g. `v0.2.0`, or an auto date-based tag)
   and publishes the GitHub Release with the binaries attached.

## Develop

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --exclude dct-gui --fail-under-lines 90
cargo run -- install --help
```

## Acknowledgements

This project was informed by and helped by the protocol and model analysis
from [MrBean355/dota2-gsi](https://github.com/MrBean355/dota2-gsi/tree/main),
especially when mapping the Dota 2 Game State Integration payloads and their
playing/spectating data structures. The implementation in this repository is
written independently in Rust and adds SQLite persistence for the received
payloads and normalized game state.

## License

MIT plus the Commons Clause v1.0 (see [`LICENSE`](LICENSE)): you may view,
modify and redistribute freely, but selling this tool (or a paid service
based substantially on it) requires the author's written consent.
