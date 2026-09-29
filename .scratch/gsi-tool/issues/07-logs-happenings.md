# 07 — Logs de happenings com filtros e follow

**What to build:** como usuário, consulto o que aconteceu na partida filtrando por partida, tipo e limite, acompanho ao vivo e exporto em JSON — com execuções silenciosas por padrão e segredos nunca impressos.

**Blocked by:** 03 — Persistência de frames e happenings em SQLite.

**Status:** ready-for-agent

- [ ] Filtros por partida, tipo e limite retornam happenings corretos em texto e em JSON
- [ ] Modo follow acompanha novos happenings ao vivo
- [ ] Logs operacionais vão ao stderr com verbosidade ajustável; token nunca aparece; payload bruto só com verbose
