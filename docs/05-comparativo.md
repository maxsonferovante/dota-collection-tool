# 05 — Comparativo Factual entre as 3 Implementações

## 1. Push (GSI) vs Pull (OpenDota)

| Eixo | antonpup (C# GSI) | tomasfarias (Rust GSI) | insight-hub (OpenDota) |
|---|---|---|---|
| Direção | Push: Dota → localhost | Push: Dota → localhost | Pull: browser → nuvem |
| Latência observada | <1s (HP/mana/gold/eventos) | <1s | minutos–dias (replay parse) |
| Requisito | Dota + `.cfg` | Dota + `.cfg` + `-gamestateintegration` | só internet |
| Cobertura documentada | partida atual (16 blocos) | partida atual (9 blocos tipados) | meta/histórico (hero/item/pro) |
| Auth | nenhuma (rede local, token não validado) | nenhuma (token não validado) | nenhuma (60 req/min anônimo) |
| Offline | funciona local | funciona local | quebra |
| Escopo | partida atual | partida atual | histórico/agregado |

GSI responde "o que acontece **nesta** partida"; OpenDota responde "o que a história diz".

## 2. Diferenças observadas

| Aspecto | antonpup | tomasfarias | insight-hub |
|---|---|---|---|
| Linguagem/stack | C# / .NET 8 / Newtonsoft.Json | Rust / tokio / httparse / serde | React 18 / Redux Toolkit / axios |
| Blocos GSI tipados | 16 | 9 (+ `draft` como `Value`) | 0 (não usa GSI) |
| Eventos | ~90 por diff | diff mínimo (map+abilities+players) | nenhum live |
| Transporte | `HttpListener` + thread | `TcpListener` manual + async | `axios.get` OpenDota |
| Manutenção | release 02/2024 | commits 05/2026 | inativo desde 2023 |
| Licença | ambígua (só texto Json.NET) | MIT | MIT |
| Plataforma | Windows-dependente | cross-plataforma | web estático |
