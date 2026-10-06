# CONTEXT.md

## Glossary

- **frame**: one accepted game POST — indexed columns (match id, game time,
  clock time, received-at) plus the raw JSON payload.
- **happening**: one derived event stored separately (match id, tick, kind,
  actor, detail JSON) — e.g. kill, death, assist, day/night, ability change.
- **payload**: the original JSON body received from Dota 2.
- **normalized_payload**: the JSON serialization of the parsed Rust `Frame`.
- **payload_kind**: `playing`, `spectating`, `post_game` or `unknown`.
- **Playing / Spectating**: single-object payload (playing) vs team→player
  map payload (observing).
- **throttle / buffer / heartbeat / timeout**: game-side send-rate knobs from
  the config file; heartbeat posts mark a quiet-but-alive game.
- **install / token / up / down / status / logs**: the six CLI tasks.

## Rules

- Clean-room: no type, module, comment or doc references any community
  implementation by name; no copied code.
- Parsing is total: unknown fields ignored, empty objects mean absent,
  no valid input panics.
- Every I/O operation is async; the HTTP layer never blocks on the store.
- Secrets (auth token) never appear in logs.
- Every accepted frame persists both the raw and normalized payload; schema
  changes are append-only migrations under `crates/store/migrations`.
