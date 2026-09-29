# 00 — Visão Geral Executiva: GSI Dota 2 + 3 Repositórios

**Data:** 2026-09-28 | **Projeto:** dota-collection-tool | **Fontes:** antonpup/Dota2GSI, tomasfarias/dota-gsi, lily4178993/dota2-insight-hub + protocolo GSI nativo Valve

## TL;DR

1. **GSI é push HTTP do próprio jogo.** Dota 2 envia `POST JSON` para `http://127.0.0.1:<porta>/` a cada mudança de estado. Sem leitura de memória, sem ban. Configuração = 1 arquivo `.cfg`.
2. **antonpup/Dota2GSI (C#)** — implementação em C# / .NET 8: 16 blocos tipados + ~90 eventos por diff. Limitado a Windows/`HttpListener`.
3. **tomasfarias/dota-gsi (Rust, não Python)** — implementação em Rust: `TCP + httparse + tokio`, sem framework web, `broadcast` para N handlers, `diff` mínimo (kill/death/day-night/ability). Ativo em 2026, MIT.
4. **lily4178993/dota2-insight-hub (React + OpenDota)** não usa GSI. É analytics pós-partida via `GET api.opendota.com`. Modelo complementar (hero/item/pro-match/pro-player), não tempo real.

## Mapa da documentação

| Arquivo | Conteúdo |
|---|---|
| `01-gsi-nativo-protocolo.md` | Protocolo Valve: `.cfg`, `uri/timeout/buffer/throttle/heartbeat/auth/data`, ciclo POST/200, paths, limites jogador vs spectator |
| `02-antonpup-dota2gsi-csharp.md` | Lib C#: arquitetura, GameState, eventos, exemplo de uso documentado no repo, limitações |
| `03-tomasfarias-dota-gsi-rust.md` | Crate Rust: transporte, `Playing vs Spectating`, `empty_map_as_none`, diff, exemplos |
| `04-insight-hub-opendota.md` | App React: por que não é GSI, endpoints OpenDota observados, limites |
| `05-comparativo.md` | Comparativo factual push vs pull entre as 3 implementações |
| `06-fontes-oficiais-steam.md` | O que a documentação oficial Steam/Valve declara (e lacunas) |
| `07-cobertura-dota-gsi-vs-api.md` | Cobertura da crate dota-gsi vs blocos da API (10/16 tipados) |

## Correção importante

O enunciado dizia que `tomasfarias/dota-gsi` era Python. **É Rust** (`crate dota-gsi v0.5.0`, edition 2024).
