# Perfis de coleta GSI

O coletor oferece três perfis para equilibrar atualização dos frames e pressão
local. O perfil escolhido altera `buffer`, `throttle` e `heartbeat`; todos os
blocos de dados GSI continuam habilitados.

| Perfil | `buffer` | `throttle` | `heartbeat` | Uso recomendado |
| --- | ---: | ---: | ---: | --- |
| Econômica | `0.20` | `0.20` | `30.0` | Partidas comuns e menor pressão local. |
| Balanceada | `0.10` | `0.10` | `30.0` | Padrão para a maioria dos usuários. |
| Baixa latência | `0.02` | `0.05` | `15.0` | Análise ao vivo que precisa de dados mais recentes. |

Os valores são preferências solicitadas ao cliente do Dota, não garantias de
tempo real ou de ausência de impacto no jogo. O comportamento pode variar por
sistema operacional e versão do Dota. `buffer` e `throttle` menores podem
aumentar POSTs, parsing, gravações e pressão na fila.

## Validação local

Antes de mudar os valores, compare os perfis com payloads representativos e
observe a latência de ingestão, a taxa de frames, o crescimento da fila,
timeouts, erros e frames descartados. Faça também uma partida curta verificando
FPS e stutter. Os diagnósticos do coletor são locais; nenhuma métrica é enviada
para um serviço externo.

O GSI continua exigindo a opção de inicialização `-gamestateintegration` e um
reinício completo do Dota após instalar ou trocar o arquivo.
