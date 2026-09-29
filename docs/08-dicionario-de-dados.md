# 08 — Dicionário de Dados do Payload GSI

Inferido dos payloads reais coletados pela ferramenta (`exports/samples/`, sessão demo com Sniper, modo Playing) mais os blocos de spectator documentados em `01` e `07`. Tipos usam a notação Rust da ferramenta (`u32`, `i32`, `bool`, `String`, `Option<…>`); entre parênteses o tipo JSON observado.

## Convenções que valem para tudo

| Convenção | Significado |
|---|---|
| `Playing` / `Spectating` | Jogando: bloco é um objeto único. Espectando: mapa `lado → playerN → objeto` (`team2` = Radiant, `team3` = Dire). |
| `{}` | Bloco vazio = ausente (a ferramenta lê como `None`). |
| `"empty"` | Slot de item vazio, não um item. |
| Chaves dinâmicas | `abilityN`, `slotN`/`stashN`/`teleportN`/`neutralN`, `preserved_neutralN`, `playerN`, `victimid_N`, `wearableN`, `oN` (minimap), nomes internos de estruturas. |
| `previously` / `added` | Envelope de delta: valores antigos do que mudou / campos novos desde o último POST. |
| Campos extras | O jogo adiciona campos sem aviso (`facet`, `attributes_level`, `accountid`, `player_slot`, `team_slot`, `gold_from_summon_kills`, `item_level`, `max_cooldown`…): a ferramenta preserva sem quebrar. |

## Raiz do frame

| Campo | Tipo | Descrição |
|---|---|---|
| `provider` | objeto (obrigatório) | Identidade do emissor (sempre o próprio jogo). |
| `auth` | objeto? | Token ecoado do `.cfg`; a ferramenta valida contra o token ativo. |
| `map`, `player`/`players`, `hero`/`heroes`, `abilities`, `items`, `events`, `buildings`, `league`, `draft`, `wearables`, `minimap`, `roshan`, `couriers`, `neutralitems` | objeto?/array? | Os 16 blocos; cada um opcional conforme o `.cfg` e o momento da partida. |
| `previously` | objeto? | Valores anteriores dos campos que mudaram neste POST. |
| `added` | objeto?/`null` | Campos que não existiam no POST anterior. |

## provider

| Campo | Tipo | Descrição |
|---|---|---|
| `name` | `String` | Nome do produto (`Dota 2`). |
| `appid` | `u32` | Id do app na Steam (570). |
| `version` | `u32` | Build do client (ex. 48). |
| `timestamp` | `u32` | Relógio do client no envio (epoch s). |

## auth

| Campo | Tipo | Descrição |
|---|---|---|
| `token` | `String?` | Token configurado no `.cfg`, ecoado a cada POST. |

## map

| Campo | Tipo | Descrição |
|---|---|---|
| `name` | `String` | Mapa (`start` em demo; nome do mapa real em partida). |
| `matchid` | `String` | Id da partida como string (`0` em demo/lobby local). |
| `game_time` | `u32` | Segundos desde o zero do jogo. |
| `clock_time` | `i32` | Relógio de partida exibido (pode ser negativo no pré-jogo). |
| `daytime` | `bool` | Se é dia no jogo. |
| `nightstalker_night` | `bool` | Noite forçada pelo Night Stalker. |
| `game_state` | `String` | Fase: `HERO_SELECTION`, `STRATEGY_TIME`, `PRE_GAME`, `GAME_IN_PROGRESS`, `POST_GAME`, etc. |
| `paused` | `bool` | Partida pausada. |
| `win_team` | `String` | `none` durante a partida; lado vencedor no fim. |
| `customgamename` | `String` | Nome do jogo customizado (vazio em partida normal). |
| `radiant_score` / `dire_score` | `u32?` | Placares de kills por lado. |
| `ward_purchase_cooldown` (+ por lado) | `u16?` | Cooldown de compra de sentinela. |
| `roshan_state` / `roshan_state_end_time` | `String?`/`u32?` | Estado do pit (`ALIVE`, `RESPAWN_*`) e fim previsto. |

## player (Playing: objeto único)

| Campo | Tipo | Descrição |
|---|---|---|
| `steamid` / `accountid` | `String` | Id Steam de 64 bits e id de conta. |
| `name` | `String` | Nome do jogador. |
| `activity` | `String` | `playing` ou `menu`. |
| `kills` / `deaths` / `assists` | `u16` | Placar pessoal. |
| `last_hits` / `denies` | `u16` | Finalizações e denies. |
| `kill_streak` | `u16` | Sequência atual de kills. |
| `kill_list` | mapa `victimid_N → u32` | Kills por vítima. |
| `commands_issued` | `u32` | Ordens emitidas. |
| `team_name` | `String` | Lado (`radiant`/`dire`). |
| `player_slot` / `team_slot` | `u32` | Índices de slot do jogador. |
| `gold`, `gold_reliable`, `gold_unreliable` | `u32` | Ouro atual e parcelas. |
| `gold_from_hero_kills`, `gold_from_creep_kills`, `gold_from_summon_kills`, `gold_from_income`, `gold_from_shared` | `u32` | Ouro por origem. |
| `gpm` / `xpm` | `u32` | Ouro/XP por minuto. |
| `net_worth` | `u32?` | Patrimônio (sempre em spectator; no modo Playing pode vir). |
| extras de spectator | variado | `hero_damage`, `wards_placed/purchased/destroyed`, `camps_stacked`, `runes_activated`, `support_gold_spent`, `item_gold_spent`… (só espectando). |

## hero (Playing: objeto único)

| Campo | Tipo | Descrição |
|---|---|---|
| `id` | `i16` | Id interno do herói (`-1` = sem pick). |
| `name` | `String?` | Nome interno (`npc_dota_hero_*`). |
| `xpos` / `ypos` | `i32?` | Posição no mapa (ausente em alguns frames). |
| `level` / `xp` | `u8?`/`u32?` | Nível e experiência. |
| `alive` / `respawn_seconds` | `bool?`/`u32?` | Vivo? Tempo para renascer. |
| `buyback_cost` / `buyback_cooldown` | `u32?` | Custo/cooldown do buyback. |
| `health`, `max_health`, `health_percent` | `u32?`/`u8?` | Vida. |
| `mana`, `max_mana`, `mana_percent` | `u32?`/`u8?` | Mana. |
| `silenced`, `stunned`, `disarmed`, `magicimmune`, `hexed`, `muted` | `bool` | Condições negativas. |
| `break` | `bool?` | Break aplicado (chave `break`). |
| `smoked`, `has_debuff` | `bool?` | Smoke / possui debuff. |
| `aghanims_scepter`, `aghanims_shard` | `bool` | Cetro e fragmento. |
| `facet` | `i32?` | Faceta escolhida. |
| `attributes_level` | `u32?` | Nível de atributos. |
| `talent_1`…`talent_8` | `bool` | Talentos escolhidos. |
| `selected_unit` | `String?` | Unidade selecionada (spectator). |

## abilities (`abilityN → objeto`)

| Campo | Tipo | Descrição |
|---|---|---|
| `name` | `String` | Nome interno da habilidade. |
| `level` | `u8` | Nível atual. |
| `can_cast` | `bool` | Pode ser conjurada agora. |
| `passive` | `bool` | É passiva. |
| `ability_active` | `bool` | Habilidade ativada (toggles). |
| `cooldown` / `max_cooldown` | `u16`/`u32` | Cooldown restante e total. |
| `charges`, `max_charges`, `charge_cooldown` | `u32` | Cargas (ex. Shrapnel: 3/3). |
| `ultimate` | `bool` | É ultimate. |

## items (`slotN`/`stashN`/`teleportN`/`neutralN`/`preserved_neutralN → objeto`)

| Campo | Tipo | Descrição |
|---|---|---|
| `name` | `String` | Nome interno (`item_*`) ou `"empty"`. |
| `purchaser` | `u32?` | Slot do comprador. |
| `can_cast` / `passive` | `bool?` | Usável / passivo. |
| `cooldown` / `max_cooldown` | `u32?` | Cooldown restante e total. |
| `charges` / `item_charges` | `u32?` | Cargas (ex. TP: 1). |
| `item_level` | `u32?` | Nível do item. |
| Chaves de posição | — | `slot0–8` inventário, `stash0–5` baú, `teleport0` TP, `neutral0–1` neutros, `preserved_neutral6–10` neutros guardados. |

## buildings (lado → nome interno → objeto)

| Campo | Tipo | Descrição |
|---|---|---|
| chave | `String` | Nome interno (`dota_badguys_tower1_mid`, `bad_rax_melee_bot`, `dota_badguys_fort`…). |
| `health` / `max_health` | `u32` | Vida atual e máxima da estrutura. |

## events (array)

| Campo | Tipo | Descrição |
|---|---|---|
| `event_type` | `String` | Tipo do envelope (observado: `generic_event`). |
| `game_time` | `u32` | Momento do evento. |
| `data` | `String` (JSON embutido!) | Detalhe serializado como **string JSON** — requer segundo parse. Ex.: `{"type":"CHAT_MESSAGE_HERO_KILL","value":189,"playerid1":4,…}`. Tipos observados: kills, streaks, compras, runas, dicas. |

> Atenção: `events.data` é string, não objeto — erro comum de parsing.

## league

| Campo | Tipo | Descrição |
|---|---|---|
| `league_id` | `u32`/`String` | Id da liga (`0` fora de torneio). |
| `match_id` | `String` | Id da partida (espelho). |
| demais | variado | Metadados de torneio (nome, tier, região, times, séries) quando em lobby de liga. |

## draft (lado → `playerN` → valor)

| Campo | Tipo | Descrição |
|---|---|---|
| valor | variado | Picks/bans do draft (só torneio); preservado sem tipagem fixa. |

## wearables (`wearableN → u32`)

| Campo | Tipo | Descrição |
|---|---|---|
| valor | `u32` | Id do item cosmético equipado no slot. |

## minimap (`oN → objeto`, spectator)

| Campo | Tipo | Descrição |
|---|---|---|
| chave | `String` | Id da marca (`o0`, `o1`, … — não é sequencial estável). |
| `unitname` | `String` | Nome interno da unidade (`npc_dota_creep_*`, rax, fillers…). |
| `image` | `String` | Ícone (`minimap_creep`, `minimap_racks90`…). |
| `team` | `u32` | Time dono (2 = Radiant, 3 = Dire, 4 = neutros). |
| `xpos` / `ypos` / `yaw` | `i32` | Posição e orientação. |
| `visionrange` | `u32` | Alcance de visão (750 creeps, 1400 kobolds…). |

## roshan / couriers / neutralitems (spectator)

| Bloco | Tipo | Descrição |
|---|---|---|
| `roshan` | objeto? | Snapshot do pit (`health`, `max_health`, drops); `{}` fora de spectator. |
| `couriers` | mapa id → objeto? | Um objeto por courier (`health`, `alive`, upgrades, carga); `{}` fora de spectator. |
| `neutralitems` | objeto? | Tiers e achados neutros por time; `{}` fora de spectator. |

## previously / added (envelope de delta)

| Campo | Tipo | Descrição |
|---|---|---|
| `previously` | objeto? | Subconjunto com valores **antigos**: ex. `{"player":{"xpm":613},"hero":{"xpos":…,"health":1106},"minimap":{"o78":{"xpos":…}}}`. |
| `added` | objeto?/`null` | Campos que surgiram neste POST (`null` quando nada é novo). |

## Divergências reais vs modelo antigo (registrado aqui para evolução)

- Minimap usa chaves `oN`, não ids numéricos.
- `events[].data` é string JSON aninhada.
- Slots `preserved_neutralN` e `neutral1` existem além de `neutral0`.
- `map.matchid` e `league.match_id` são strings; `league.league_id` veio int.
- `map.name` em demo é `start`, não nome de mapa real.
