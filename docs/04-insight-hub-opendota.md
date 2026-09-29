# 04 — lily4178993/dota2-insight-hub (React + OpenDota): Analytics Pós-Partida

**Repo:** https://github.com/lily4178993/dota2-insight-hub | **Demo:** https://dota2insighthub.netlify.app/ | **Licença:** MIT (2023 Nelly Telli) | **Estado:** protótipo educacional inativo (~2 anos), CRA 5 + React 18, 2 stars

## 1. Não é GSI

Zero `.cfg`, zero `POST localhost`, zero websocket. Todo dado vem de `axios.get()` contra `https://api.opendota.com/api` no `useEffect` da `Home.jsx`. Exibe agregados históricos (`turbo_picks/wins, pro_pick, duration, league_name`), nunca `health/mana/gold` live. Classificação: **Web Analytics (pull REST)**, não overlay push.

README cita "Backend Node.js + Dota 2 API" — impreciso: não há `server/` nem Express; Node é só build do CRA.

## 2. Stack e arquitetura

`react 18.2 + react-router 6.16 + @reduxjs/toolkit 1.9.7 + axios 1.5.1 + CSS puro + prop-types + jest`. Deploy estático Netlify.

Rotas: `/, /home` (4 cards), `/details/:lista (heroes|items|proMatches|proPlayers)`, `/details/:lista/:id`, `/aboutme`, `/references`. Fluxo: 4 `createAsyncThunk` → Redux arrays brutos → `Details` + `DetailsItem` (find em memória, sem refetch por ID).

Features: Hero Insights (~124 heróis, 6 blocos em `HeroInfo.jsx`), Item DB (`constants/items` objeto→array), Pro Matches (~100 últimas), Pro Players, filtro `Names|Counts` (só sort, sem busca textual), truque de imagem HD via `cdn.cloudflare.steamstatic.com/.../renders/<nome>.png`.

## 3. Endpoints (sem key, ~60 req/min anônimo)

| Slice | GET | Uso |
|---|---|---|
| `heroesSlice` | `/heroStats` | `localized_name, primary_attr, base_str/agi/int, *_gain, attack_range/rate, move_speed, day/night_vision, turbo_picks/wins, pro_pick` |
| `itemsSlice` | `/constants/items` | `Object.entries()` → array; `dname, cost, qual, mc, cd, lore, hint, notes` |
| `matchesSlice` | `/proMatches` | `match_id, duration, start_time, league_name, radiant/dire_name/score` |
| `playersSlice` | `/proPlayers` | `account_id, personaname, avatarfull/medium, steamid, loccountrycode, team_name/tag` |

Não usa STRATZ GraphQL nem `api.steampowered.com` (exigiriam token/key). `dotenv` instalado mas sem `process.env` real.

## 4. Bugs/limites observados

- Herói indexado por `data[id-1]` (quebra se `hero_id` não sequencial; o próprio repo usa `find` para match/player).
- `Counts` hardcoded defasados (`Heroes:122` vs 124+ da API); sem paginação/busca/cache/retry/skeleton; CSS global sem tokens.
