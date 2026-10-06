# 09 — Processamento GSI e persistência

O `dota-collection-tool` recebe os `POST` JSON enviados pelo Game State
Integration do Dota 2, interpreta os blocos do protocolo e persiste tanto o
payload original quanto a representação normalizada.

O modelo cobre `ability`, `annotation`, `building`, `draft`, `event`,
`gamestate`, `hero`, `item`, `json`, `map`, `player` e `provider`.

## Pipeline

```text
POST do Dota 2 → json::parse_payload → RawGameState
                 → PayloadProcessor::normalize → Frame/GameState
                 → SQLite: bruto + normalizado + índices
```

O parser aceita blocos ausentes ou vazios, preserva campos novos e classifica
o modo como `playing`, `spectating`, `post_game` ou `unknown`.

## Persistência

| Coluna | Uso |
|---|---|
| `payload` | JSON bruto exatamente como recebido |
| `normalized_payload` | Modelo Rust serializado após o parsing |
| `payload_kind` | Modo detectado do payload |
| `match_id`, `game_time`, `clock_time` | Índices para consulta |
| `received_at` | Horário de recebimento |

Campos desconhecidos permanecem no payload bruto e nos mapas `extra` dos
modelos, permitindo reprocessar dados antigos quando o modelo evoluir.

## Draft e eventos

O wire format usa `pick0_id`, `pick0_class`, `ban0_id` e `ban0_class`; o módulo
`draft` transforma esses campos em `picks` e `bans` tipados. Eventos preservam
`game_time`, `event_type` e detalhes adicionais. Repetições entre frames podem
ser deduplicadas por `(game_time, event_type)`, enquanto happenings derivados
são armazenados separadamente.

## Verificação

```sh
cargo test --workspace
cargo llvm-cov --workspace --exclude dct-gui --fail-under-lines 90
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

O schema evolui por migrações SQL incrementais em
`crates/store/migrations`; não edite migrações já aplicadas.
