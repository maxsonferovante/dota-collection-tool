# 01 — GSI Nativo Valve: Protocolo, .cfg e Payload

## 1. O que é

Game State Integration (GSI) é o push oficial do Dota 2: o client envia `HTTP POST application/json` para um servidor local a cada tick relevante. Sem SDK, sem memória, sem anti-cheat.

- **Método:** `POST / HTTP/1.1`
- **Headers reais:** `User-Agent: Valve/Steam HTTP Client 1.0 (570)`, `Content-Type: application/json`, `Host`, `Content-Length: ~50-60KB`, `Accept`, `accept-encoding`, `accept-charset`
- **Resposta obrigatória:** `HTTP/1.1 200 OK` + `content-type: text/html`. Sem 200 o Dota retenta.
- **Frequência:** controlada por `throttle` + `buffer` (ex. `0.1` ≈ até 10 Hz). `heartbeat` mantém POST periódico sem mudança.

## 2. Instalação (.cfg)

**Caminho:**
```
<Steam>/steamapps/common/dota 2 beta/game/dota/cfg/gamestate_integration/gamestate_integration_<NOME>.cfg
```
- Linux: `~/.steam/steam/steamapps/common/dota 2 beta/game/dota/cfg/gamestate_integration/`
- Windows: `D:\Steam\steamapps\common\dota 2 beta\game\dota\cfg\gamestate_integration\`
- Nome do arquivo livre, deve terminar em `.cfg`. Formato Valve KeyValues (não JSON).
- Requisito obrigatório desde 11/03/2022: inicializar o jogo com a opção de linha de comando `-gamestateintegration`, caso contrário a GSI não envia nada. Fonte: [Atualização do Dota 2 — 11 de março de 2022](https://www.dota2.com/newsentry/4491783379124370818). Motivo declarado pela Valve: possível impacto no desempenho quadro a quadro ao usar a integração.

**Exemplo consolidado a partir dos .cfg documentados nos repos (C# usa `heartbeat 10.0`, Rust usa `30.0`):**
```
"Meu Coletor Configuration"
{
    "uri"          "http://127.0.0.1:53000/"
    "timeout"      "5.0"
    "buffer"       "0.1"
    "throttle"     "0.1"
    "heartbeat"    "30.0"
    "data"
    {
        "auth"         "1"
        "provider"     "1"
        "map"          "1"
        "player"       "1"
        "hero"         "1"
        "abilities"    "1"
        "items"        "1"
        "events"       "1"
        "buildings"    "1"
        "league"       "1"
        "draft"        "1"
        "wearables"    "1"
        "minimap"      "1"
        "roshan"       "1"
        "couriers"     "1"
        "neutralitems" "1"
    }
    "auth"
    {
        "token" "troque-por-token-longo"
    }
}
```

**Semântica dos campos:**

| Campo | Efeito |
|---|---|
| `uri` | Destino do POST. Deve ser idêntico ao bind do servidor. Porta livre (convenção: `3000` no C#, `53000` no Rust). |
| `timeout` | Segundos que o jogo aguarda o ACK antes de retentar. |
| `buffer` | Agrega mudanças por X s antes de enviar (lote). |
| `throttle` | Intervalo mínimo entre POSTs (controla taxa máx). |
| `heartbeat` | POST keep-alive mesmo sem mudança (C#: `10.0`, Rust: `30.0`). |
| `data { bloco "1" }` | Habilita bloco. Omitir ou `"0"` suprime o nó (reduz ~55KB/payload). |
| `auth.token` | Ecoado em `auth.token` no JSON. Comportamento observado: nenhuma das duas libs GSI valida o token. |

## 3. Blocos de dados (contrato)

`auth, provider, map, player, hero, abilities, items, events, buildings, league, draft, wearables, minimap, roshan, couriers, neutralitems`. Ver `previously` (snapshot anterior imediato) para diff.

Chaves dinâmicas observadas nos parsers (expressões usadas nos repos):
- `abilityN`, `slotN / stashN / teleportN / neutralN`, `victimid_N`, `towerN_top|mid|bot`, `rax_melee|range_top|mid|bot`, `playerN`.

## 4. Jogando vs Espectando (limite Valve)

- **Jogando:** só jogador local (`LocalPlayer`). Sem `net_worth`, sem dados dos outros 9.
- **Espectando (observer/caster):** todos os 10 jogadores + extras: `net_worth, hero_damage/healing, tower_damage, roshan {health, drops}, couriers, neutralitems, ward cooldown por time, selected_unit`.

## 5. Comportamentos de parsing observados

1. Bloco vazio chega como `{}` — o repo Rust trata como `None` (padrão `empty_map_as_none`).
2. IDs são internos: `npc_dota_hero_axe`, `item_blink` — os repos referenciam `heroes.json/items.json` e wiki fandom para resolução.
3. `"empty"` em slots de item = vazio, não item.
4. `auth.token` chega no JSON mas nenhuma lib faz checagem.
5. Listas antigas de GameState (ex. consultas Gemini) omitem `couriers/neutralitems/minimap/roshan/draft` — os repos analisados cobrem até 16 blocos (lista acima).
