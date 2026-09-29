# 08 — Endurecimento ponta a ponta e portões finais

**What to build:** como usuário, rodo o fluxo inteiro (install, token, up, partida simulada com heartbeat e throttle, logs, down) no Linux, Windows e macOS com todos os portões verdes — e como contribuidor sei que patches futuros do jogo não quebram a coleta.

**Blocked by:** 01 — Scaffold; 02 — Núcleo de tipos; 03 — Persistência; 04 — Servidor HTTP; 05 — Install/token; 06 — Ciclo de vida; 07 — Logs.

**Status:** ready-for-agent

- [ ] Fluxo completo ponta a ponta passa contra instalação simulada e banco temporário, incluindo heartbeat (jogo quieto vs conexão morta) e semântica de throttle/buffer
- [ ] Descoberta de caminho validada nos três SOs; restart do jogo após install documentado na saída
- [ ] Formatação, linter com warnings negados e suíte completa verdes; nenhum panic em entradas válidas; clean-room verificado (zero menção a implementações comunitárias)
