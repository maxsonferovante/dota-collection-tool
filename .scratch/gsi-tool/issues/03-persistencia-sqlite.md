# 03 — Persistência de frames e happenings em SQLite

**What to build:** como usuário, cada frame aceito fica gravado com partida, tempos e timestamp de recebimento mais o payload bruto, e os happenings derivados (kills, deaths, assists, dia/noite, habilidades) ficam consultáveis — tudo via operações assíncronas, com restart recuperando o arquivo de overflow.

**Blocked by:** 01 — Scaffold do workspace e portões de qualidade.

**Status:** ready-for-agent

- [ ] Cada frame persiste com id da partida, tempo de jogo/relógio, recebido-em e payload bruto; índices cobrem as consultas dos logs
- [ ] Happenings derivados gravados em separado (partida, tick, tipo, ator, detalhe)
- [ ] Rajadas concorrentes drenam sem perda além do caminho contado de overflow; restart reprocessa o overflow
- [ ] Toda operação de banco é assíncrona; nenhuma falha de I/O causa panic
