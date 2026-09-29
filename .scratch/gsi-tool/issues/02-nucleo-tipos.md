# 02 — Núcleo de tipos tolerantes com os 16 blocos

**What to build:** como usuário, envio qualquer POST real do jogo (jogando, espectando, menu, heartbeat) e a ferramenta entende todos os 16 blocos sem quebrar — blocos vazios viram ausentes, slots vazios viram vazios, campos futuros desconhecidos são ignorados.

**Blocked by:** 01 — Scaffold do workspace e portões de qualidade.

**Status:** ready-for-agent

- [ ] Todos os 16 blocos da API parseiam a partir de fixtures realistas anonimizadas (idle, hero select, strategy, em-progresso, spectator cheio, só-heartbeat)
- [ ] Formatos Playing (objeto único) e Spectating (mapa time→jogador) ambos aceitos; envelope previous/added tipado
- [ ] `{}` vira ausente, `"empty"` vira vazio, campos desconhecidos não falham, nenhuma entrada válida causa panic
- [ ] Tipos, nomes e docs 100% próprios, sem referência a implementações comunitárias
