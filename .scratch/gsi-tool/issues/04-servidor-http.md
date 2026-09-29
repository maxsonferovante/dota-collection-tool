# 04 — Servidor HTTP que recebe os POSTs do jogo

**What to build:** como usuário, aponto o jogo para o servidor local e cada POST vira linhas no banco em segundos — com token errado rejeitado sem poluir nada e com resposta imediata para o jogo nunca travar, mesmo sob rajada de teamfight.

**Blocked by:** 02 — Núcleo de tipos tolerantes com os 16 blocos; 03 — Persistência de frames e happenings em SQLite.

**Status:** ready-for-agent

- [ ] POST com token válido responde sucesso no content type esperado e persiste frame mais happenings
- [ ] Token ausente ou errado é rejeitado sem gravar nada e sem logar o segredo
- [ ] Endpoint de saúde responde para o comando status
- [ ] Fila cheia descarta o mais antigo com contador, despeja em overflow local e mantém resposta rápida; método/caminho divergentes não tocam o banco
