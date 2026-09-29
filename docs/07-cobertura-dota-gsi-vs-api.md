# 07 — Cobertura da crate dota-gsi vs o que a API GSI fornece

**Alvo:** `https://lib.rs/crates/dota-gsi` (v0.5.0, 04/05/2026, MIT) = `github.com/tomasfarias/dota-gsi`. **Veredito:** cobertura parcial — não tem tudo que a API fornece. Transporte OK, modelagem incompleta.

## 1. Base da comparação

- API considerada: 16 blocos documentados no repo C# + `.cfg` + envelope `previously/added` + semântica `uri/timeout/buffer/throttle/heartbeat/auth` (ver `01` e `06`).
- Implementação lida: `src/components/mod.rs` (`GameState`, `Provider`, `Map`, `Auth`), `players.rs`, `event.rs`, README/lib.rs (`.cfg` de 9 blocos).

## 2. Blocos: 10 de 16 tipados

| Bloco API | Crate | Estado |
|---|---|---|
| `provider` | `Provider {name, app_id (alias appid), version, timestamp}` | Coberto |
| `map` | `Map` parcial (ver §3) | Parcial |
| `player` | `GamePlayers: Playing \| Spectating` | Coberto (base) |
| `hero` | `GameHeroes: Playing \| Spectating` | Coberto (base) |
| `abilities` | `GameAbilities` | Coberto |
| `items` | `GameItems` (slot/stash/teleport/neutral, `"empty"`) | Coberto (base) |
| `buildings` | `HashMap<Team, Buildings>` | Coberto (base) |
| `wearables` | `GameWearables` | Coberto |
| `draft` | `Option<HashMap<Team, HashMap<PlayerID, Value>>>` | Não tipado (`serde_json::Value`) |
| `auth` | `Auth {token: Option<String>}` | Só desserializa, não valida |
| `events` | ausente | **Não coberto** |
| `league` | ausente | **Não coberto** |
| `minimap` | ausente | **Não coberto** |
| `roshan` | ausente | **Não coberto** |
| `couriers` | ausente | **Não coberto** |
| `neutralitems` | ausente | **Não coberto** |

O `.cfg` de exemplo da crate habilita só 9 blocos (`buildings/provider/map/player/hero/abilities/items/draft/wearables`), consistente com o struct.

## 3. `Map`: campos de placar/Roshan ausentes

- Crate tem: `name, match_id (alias matchid), game_time, clock_time, daytime, nightstalker_night, game_state, paused, win_team, custom_game_name (alias customgamename), ward_purchase_cooldown?`.
- API (via C#) tem ainda: `RadiantScore/DireScore`, `Radiant/DireWardPurchaseCooldown`, `RoshanState/ RoshanStateEndTime`, `IsDaytime/IsNightstalkerNight` já mapeados.
- `DotaGameRulesState` mapeia 11 valores + `Undefined(String)` — cobertura de enum OK.

## 4. `PlayerInformation`: campos spectator descartados

- Crate armazena: `steamid/name/activity/kills/deaths/assists/last_hits/denies/kill_streak/kill_list/commands_issued/team_name/gold*/net_worth?/gpm/xpm`.
- JSON real de spectator (teste em `players.rs`) contém ainda: `camps_stacked, consumable_gold_spent, gold_lost_to_death, gold_spent_on_buybacks, hero_damage, item_gold_spent, support_gold_spent, wards_*, runes_activated...` — o struct não declara esses campos, então o `serde` os descarta na desserialização (perda silenciosa, sem erro).
- `PlayerID` parseia `"playerN"`; `GamePlayers` distingue `Playing` vs `Spectating`; modo misto resulta em `panic!` (teste `test_game_players_mixed_mode_panics`).

## 5. Diff/eventos: 3 famílias vs ~90 do C# e 6 tipos do bloco `events`

- Crate (`event.rs`, feature `diff`): `Ability {LevelledUp, WentOnCooldown, WentOffCooldown, Activated, Deactivated}`, `Map {StartedDay, StartedNight}`, `Player {SecuredKill, Died, Assisted}`. `GameState::diff` só compara `map+abilities+players` (não `hero/items/buildings/wearables/draft`).
- Não cobre o bloco `events` da API (`Courier_killed, Roshan_killed, Aegis_*, Tip, Bounty_rune`) nem torres, itens, Roshan, draft, wards.
- Comportamento: só emite a partir do 2º tick; ignora jogador novo/saído (interseção); `Previously` da API não é modelado como campo (diff é interno entre snapshots guardados no handler).

## 6. Transporte e envelope

- Coberto: `TcpListener + httparse + broadcast` para N handlers, resposta `200 OK text/html`, `Content-Length`, canal de shutdown, exemplos `echoslam/killfeed/recall`, `empty_map_as_none` (`{}` → `None`).
- Não coberto: validação de `auth.token`, roteamento por path/método, backpressure persistente, bloco `added` como campo tipado.

## 7. Conclusão factual

A crate entrega transporte + parsing tipado do núcleo (`provider/map/player/hero/abilities/items/buildings/wearables` + `draft` bruto + `auth` ecoado) + diff mínimo de kills/mortes/assists/dia-noite/abilities. Não cobre 6 blocos (`events/league/minimap/roshan/couriers/neutralitems`), não tipa `draft`, descarta campos spectator extras de `player`, não modela placar/Roshan em `map` nem o envelope `previously/added` como tipos. Para "tudo que a API fornece", faltam esses blocos/campos.
