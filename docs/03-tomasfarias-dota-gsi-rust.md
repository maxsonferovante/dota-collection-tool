# 03 — tomasfarias/dota-gsi (Rust): Transporte e Diff

**Repo:** https://github.com/tomasfarias/dota-gsi | **Crate:** `dota-gsi v0.5.0`, edition 2024 | **Licença:** MIT (2022 Tomás Farías) | **Estado:** ativo (commits 05/2026), 83 commits, 22 stars, 0 issues | **Atenção:** é Rust, não Python

## 1. Propósito

Transporte + parsing + fan-out + diff. Sem framework web, sem persistência, sem validação de token.

## 2. Arquitetura (sem axum/actix)

`tokio + bytes + httparse + serde/serde_json + broadcast`. Estrutura: `lib.rs (Server/ServerBuilder/Listener/Handler/MutHandler/process)`, `components/ (GameState/Provider/Map/players/heroes/abilities/items/buildings/wearables/team)`, `event.rs (GameEvent)`, `diff.rs (trait Diffable)`, `handlers/ (echo, DiffHandler)`, `examples/ (echoslam, killfeed, recall)`.

- `Handler` (stateless) / `MutHandler` (com estado) + `broadcast::channel(16)` eventos e `channel(1)` shutdown.
- `Listener`: `TcpListener::bind(uri)`, `spawn(process(socket))` por conexão. Sem handlers → `NoHandlersAvailable` (desliga).
- `process()`: lê até `httparse` completar (7 headers esperados), extrai `Content-Length`, lê corpo, responde `200 OK text/html`, faz `broadcast(Bytes)`.

## 3. Uso documentado no repo

```rust
let server = ServerBuilder::new("127.0.0.1:53000").register(echo).start()?;
server.run_forever().await;
```

Porta livre por convenção (`53000` no README; testes usam `10080/30080/40080`).

## 4. Modelo de dados

`GameState { provider, buildings?, map?, players? (alias player), heroes? (alias hero), abilities?, items?, draft? (Value não-tipado), wearables?, auth? }`.

- Padrão `empty_map_as_none`: `{}` → `None`.
- `GamePlayers/GameHeroes/GameAbilities/GameItems/GameWearables` = `untagged Playing(Single) | Spectating(HashMap<Team, HashMap<PlayerID, T>>)` — câmera do jogador vs observer.
- `PlayerInformation`: `steamid/name/activity/kills/deaths/assists/last_hits/denies/kill_streak/kill_list/commands_issued/team/gold*/net_worth?/gpm/xpm`.
- `Hero`: `xpos/ypos/id (-1 = sem pick)/name/level/xp/alive/respawn/buyback/health*/mana*/silenced/stunned/.../talent_1..8`.
- `Map.game_state`: `Disconnected/InProgress/HeroSelection/Starting/Ending/PostGame/PreGame/StrategyTime/WaitingForMap/WaitingForPlayers/CustomGameSetup/Undefined`.
- `diff` implementado: só `map+abilities+players` → `StartedDay/StartedNight, SecuredKill/Died/Assisted, LevelledUp/WentOnCooldown/OffCooldown/Activated/Deactivated`. Comportamento observado: `Playing vs Spectating` resulta em `panic!`.

## 5. .cfg documentado no repo

`heartbeat 30.0` e 9 blocos (`buildings/provider/map/player/hero/abilities/items/draft/wearables`) + `auth.token`. Difere do repo C# que documenta 16 blocos.

## 6. Testes e deps observados

`cargo test` cobre `Content-Length`, `process()` com POST cru 173B, broadcast, shutdown, deserialização idle/init/strategy/in-progress e diff. Deps: `tokio, bytes, httparse, serde/serde_json, broadcast`.
