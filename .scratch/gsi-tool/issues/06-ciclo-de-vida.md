# 06 — Ciclo de vida up/down/status

**What to build:** como jogador, subo o servidor para testar em foreground, subo detached para jogar, desligo sem caçar PID e consulto se a coleta está saudável — com Ctrl-C gravando os frames em voo.

**Blocked by:** 04 — Servidor HTTP que recebe os POSTs do jogo; 05 — Install da config e gestão do token.

**Status:** ready-for-agent

- [ ] `up` em foreground bloqueia e Ctrl-C encerra graciosamente gravando frames em voo
- [ ] `up` detached solta com arquivo PID; `down` encerra pelo PID; `status` mostra PID mais saúde via endpoint
- [ ] Ciclo completo funciona contra diretório temporário e banco temporário sem tocar a instalação real
