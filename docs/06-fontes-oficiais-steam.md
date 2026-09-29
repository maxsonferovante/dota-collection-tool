# 06 — Fontes Oficiais Steam/Valve sobre GSI

## 1. Achado central: não há página oficial dedicada ao GSI do Dota 2

- A Valve Developer Community mantém a especificação canônica para CS:GO/CS2 (`Counter-Strike: Global Offensive Game State Integration`), sem equivalente para Dota 2. A página estava inacessível por proteção anti-bot (Anubis) no momento da pesquisa, mas os trechos indexados + o README da crate confirmam o reuso por analogia.
- O próprio README da `dota-gsi` (lib.rs/crates/dota-gsi) aponta o `.cfg` para `developer.valvesoftware.com/wiki/Counter-Strike:_Global_Offensive_Game_State_Integration` e a launch option para `help.steampowered.com/en/faqs/view/7d01-d2dd-d75e-2955` (FAQ Steam de opções de inicialização). Ou seja: a lib trata a doc de CS como referência para Dota.
- Síntese comunitária (auo.nu, 2023-04-13): "Although GSI is officially launched for CS, there is no official information or release for its use in Dota 2... extracted based on CS documentation and via debugging."

## 2. Fontes oficiais localizadas

| Fonte | Tipo | O que declara |
|---|---|---|
| `dota2.com/newsentry/4491783379124370818` — Update 11/03/2022 | Patch note oficial | GSI só funciona com `-gamestateintegration` na linha de comando. Motivo: impacto no desempenho quadro a quadro. |
| `help.steampowered.com/en/faqs/view/7d01-d2dd-d75e-2955` | FAQ Steam oficial | Como definir launch options (referenciado pela crate como passo 2). |
| `github.com/ValveSoftware/Dota-2/issues/2333` | Tracker oficial Valve (Linux/Mac Reborn) | Caminho `.../dota 2 beta/game/dota/cfg/gamestate_integration/` + flag `-gamestateintegration`; relato de GSI sem efeito no Linux (Windows OK); exemplo `.cfg` com 6 blocos; menção a stutter com a flag ativa. |
| `support.overwolf.com/.../how-to-enable-game-state-integration-for-dota-2` (atualizado 26/11/2025) | Suporte terceiro, procedimento Steam | Biblioteca → Dota 2 → Properties → General → Launch Options → `-gamestateintegration` → restart Steam + app. Confirma o procedimento, não é doc Valve. |

## 3. Semântica de endpoint herdada da doc CS (aplicada por analogia ao Dota)

Trechos indexados da página oficial CS, confirmados pelos `.cfg` dos dois repos Dota:

- `uri`: destino do `POST JSON` (local ou remoto, `https` com validação de certificado).
- `timeout`: espera por `HTTP 2XX`; sem 2XX o próximo POST não é enviado; timeout → re-heartbeat com estado completo sem delta. Default CS: `1.1s`.
- `buffer`: agrega eventos por X s para enviar delta maior. Default CS: `0.1s`.
- `throttle`: intervalo mínimo após 2XX antes do próximo envio. Default CS: `1.0s`.
- `heartbeat`: envio periódico mesmo sem mudança; permite detectar offline/desconexão.
- `auth`: seção opcional, ecoada como strings JSON; recomendação de `https` quando presente.
- Envelope de diff: bloco global `previously` (estado anterior do que mudou) + `added` (campos novos ausentes no estado anterior). Estado completo + delta a partir do último 2XX.
- Múltiplos `.cfg` (`gamestate_integration_<nome>.cfg`) coexistem; o client replica para todos.

Limite da analogia: nomes de blocos CS (`map/round/player_id/allplayers_* /bomb/phase_countdowns...`) não são os blocos Dota (`map/player/hero/abilities/items/buildings/league/draft/wearables/minimap/roshan/couriers/neutralitems`). A doc CS não lista nenhum bloco Dota.

## 4. O que a documentação oficial não cobre (lacunas)

- Lista completa dos 16 blocos Dota e seus campos/tipos.
- Regra Jogando (só local) vs Espectando (10 jogadores + extras).
- Tipos `Playing (objeto único)` vs `Spectating (mapa time→player)` e `{}` como vazio.
- Campos spectator extras (`net_worth`, `hero_damage`, wards, `roshan`, `couriers`, etc.).
- Comportamentos observados: busca de `.cfg` a cada início de partida sem hot-reload de `uri` já configurada; `throttle/buffer` ignorados no Linux em relato; necessidade de restart após editar `.cfg`.

Esses pontos vêm de debugging e das libs comunitárias (C#/Rust/Kotlin/JS), não de spec Valve.
