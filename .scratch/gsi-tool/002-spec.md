---
title: "SPEC — Ferramenta própria de coleta GSI (servidor HTTP + CLI gestor + SQLite async)"
labels: [ready-for-agent]
feature: gsi-tool
kind: spec
---

## Declaração do Problema

Como jogador/observador de Dota 2 que precisa registrar o estado da partida em tempo real, quero uma ferramenta própria, instalada localmente, que receba os POSTs JSON enviados pelo client do jogo, persista cada frame em banco local e exponha um CLI que instala a configuração no Dota, gera o token de autenticação, liga/desliga o servidor e consulta os eventos — sem depender de serviços externos, sem leitura de memória do jogo e sem citar ou copiar implementações comunitárias existentes.

## Solução

Um workspace Rust com servidor HTTP local que recebe os POSTs do jogo respondendo 200 rapidamente, núcleo de tipos tolerantes cobrindo os 16 blocos da API, persistência assíncrona em SQLite (frames brutos mais happenings derivados) e CLI gestor com install, token, up, down, status e logs. Toda operação de I/O é assíncrona; código e documentação da ferramenta não mencionam nenhuma implementação de referência.

## Histórias de Usuário

1. Como jogador, quero instalar o arquivo de configuração via CLI, para não editar arquivos manualmente.
2. Como jogador, quero que o CLI detecte automaticamente meu diretório de instalação do Dota, para não procurar a pasta de cfg.
3. Como jogador, quero sobrescrever o diretório detectado explicitamente, para suportar bibliotecas Steam customizadas.
4. Como jogador, quero que a instalação nunca sobrescreva config existente sem flag explícita, para não perder configuração de outra ferramenta.
5. Como jogador, quero um backup de qualquer config substituída, para poder reverter.
6. Como jogador, quero que o install informe a flag de inicialização exigida pelo jogo desde a atualização oficial de março de 2022, para não coletar nada sem saber por quê.
7. Como jogador, quero gerar um token de autenticação novo via CLI, para que só meu client alimente meu servidor.
8. Como jogador, quero rotacionar o token, para que um token vazado pare de funcionar.
9. Como jogador, quero exibir o token atual, para confirmar qual valor está ativo sem adivinhar.
10. Como jogador, quero que o install injete o token ativo no arquivo de config, para jogo e servidor concordarem.
11. Como jogador, quero subir o servidor em foreground, para acompanhar durante testes.
12. Como jogador, quero parar o servidor em foreground com Ctrl-C de forma graciosa, para que frames em voo sejam gravados.
13. Como jogador, quero subir o servidor em background (detached), para continuar coletando enquanto jogo.
14. Como jogador, quero desligar o servidor detached via CLI, para não caçar PIDs manualmente.
15. Como jogador, quero consultar o status do servidor, para saber se a coleta está rodando e saudável.
16. Como observador, quero coletar os dados dos 10 jogadores enquanto especto, para cobrir a partida inteira.
17. Como usuário, quero que cada POST recebido seja respondido imediatamente com sucesso, para que o jogo nunca trave esperando minha ferramenta.
18. Como usuário, quero que requisições com token errado ou ausente sejam rejeitadas, para que tráfego estranho nunca polua meu banco.
19. Como usuário, quero que cada frame aceito seja persistido com id da partida, tempo de jogo, tempo de relógio e timestamp de recebimento, para reproduzir a partida tick a tick.
20. Como usuário, quero o payload bruto preservado ao lado das colunas indexadas, para que campos futuros sobrevivam a patches.
21. Como usuário, quero os happenings derivados (kills, deaths, assists, transições dia/noite, mudanças de nível/cooldown de habilidades) guardados em separado, para consultar eventos sem varrer JSON bruto.
22. Como usuário, quero consultar happenings filtrando por partida, tipo e limite, para achar rápido o que aconteceu.
23. Como usuário, quero acompanhar o log ao vivo, para ver eventos durante a partida.
24. Como usuário, quero saída JSON nos logs, para encadear eventos com outras ferramentas.
25. Como usuário, quero logs operacionais no stderr com verbosidade ajustável, para execuções silenciosas e debug possível.
26. Como usuário, quero que o token nunca apareça nos logs, para que segredos não vazem.
27. Como usuário, quero que a ferramenta continue funcionando quando frames chegam mais rápido do que o banco drena, para que teamfight quente não estoure memória nem bloqueie o jogo.
28. Como usuário, quero que frames excedentes sejam contados e despejados num arquivo local de overflow, para que nada se perca em silêncio.
29. Como usuário, quero que blocos vazios sejam tratados como ausentes e slots vazios de item como vazios, para que menu/idle não quebrem o parsing.
30. Como usuário, quero que campos futuros desconhecidos sejam ignorados, para que um patch não quebre a coleta.
31. Como usuário, quero payloads Playing (objeto único) e Spectating (mapa time→jogador) ambos aceitos, para que jogar e observar funcionem.
32. Como usuário, quero posts de heartbeat registrados, para distinguir jogo quieto de conexão morta.
33. Como usuário, quero descoberta de caminho Steam no Linux, Windows e macOS, para que a ferramenta funcione na minha máquina.
34. Como contribuidor, quero lints estritos e formatação aplicados, para manter o código limpo.
35. Como contribuidor, quero testes unitários e de integração com fixtures realistas anonimizadas, para cobrir parsing, auth, semântica de buffer e fluxos do CLI.
36. Como operador, quero que um restart reprocesse o arquivo de overflow, para que frames excedentes entrem no banco depois.

## Decisões de Implementação

- Workspace com quatro crates: core (tipos de domínio mais parsing tolerante, sem runtime async), net (servidor HTTP), store (persistência SQLite assíncrona) e CLI (superfície de comandos mais ciclo de vida do servidor). Binário único; nenhuma biblioteca publicada.
- Contrato HTTP: um endpoint POST que aceita o JSON do jogo e responde sucesso no content type esperado pelo client, mais um endpoint de saúde para o comando status; método/caminho divergentes são rejeitados sem efeito no banco.
- Modelo de domínio: 16 blocos da API como estruturas opcionais tipadas mais envelope tipado para estados previous/added; apelidos seguem os nomes do protocolo; chaves dinâmicas seguem os padrões do protocolo (habilidades, slots de item, vítimas, torres, jogadores); parsing total sem panics.
- Persistência: coleção de frames chaveada por (id da partida, tempo de jogo/relógio, recebido-em) com payload bruto, mais coleção de happenings (partida, tick, tipo, ator, detalhe); modo write-ahead; pool limitado; migrações embutidas.
- Concorrência: fila única limitada entre HTTP e worker de persistência; cheia descarta o mais antigo com contador e despeja em log local de overflow; HTTP nunca bloqueia no banco; runtime async multithread em todas as operações de I/O.
- CLI: install (nome, override de diretório, force/backup), token (gerar/exibir/rotacionar), up (foreground padrão, detached com arquivo PID), down, status (PID mais saúde), logs (partida/tipo/limite/follow/json).
- Arquivo de config: escritor KeyValues próprio; nome segue convenção de prefixo/sufixo do jogo; detecção por SO com override explícito vencendo; sem sobrescrita sem force (gera backup); jogo exige restart após (re)instalar pois não recarrega endpoint registrado a quente.
- Autenticação: 32 bytes CSPRNG em hex em arquivo de config local ao lado do banco, injetado no install, validado por requisição; divergência rejeitada sem logar o segredo.
- Regra de originalidade (dura): nenhum tipo, módulo, comentário ou doc referencia implementação comunitária por nome; nenhum código copiado; conhecimento do protocolo vem das notas de pesquisa próprias.
- Erros: tipados por crate de biblioteca, contextuais livres só na borda do CLI; nenhum unwrap/expect em I/O ou parsing.

## Decisões de Teste

- Um bom teste aqui exercita comportamento externo (status/corpo HTTP, exit code/stdout do CLI, linhas no banco), nunca internos; usa fixtures realistas anonimizadas (idle, hero select, strategy, em-progresso, spectator cheio, só-heartbeat); afirma frames persistidos e happenings derivados.
- Costura principal (ideal é uma só; codebase greenfield, então a costura é proposta — sinalize se preferir outra): borda do CLI de ponta a ponta contra diretórios temporários e servidor em loopback (install escreve arquivo e backup; token rotate invalida o antigo; up/down/status; POSTs reais geram frames e happenings; logs filtram). Essa única costura atravessa net, core e store pelo caminho real.
- Costura secundária pura (sem I/O, para diagnóstico fino): parsing do core — bytes entram, frame tipado sai (vazios, desconhecidos, Playing vs Spectating, envelope).
- Materiais prévios: nenhum no repo além das notas de pesquisa em docs/ (nomes de blocos, chaves dinâmicas, extras de spectator); fixtures devem espelhá-las.
- Portões: formatação, linter com warnings negados, suíte completa a cada mudança.

## Fora de Escopo

- UI de analytics pós-partida, sync remoto/nuvem ou enriquecimento via APIs públicas de histórico.
- Leitura de memória do jogo, parsing de replays ou polling de APIs oficiais.
- Instalação como serviço do SO além do detached com PID; hospedagem multiusuário; TLS remoto; componente servidor central.
- Publicação de crate reutilizável.

## Notas Adicionais

- Derivado do PRD `.scratch/gsi-tool/001-prd.md` (label ready-for-agent) mais o grill de 12 decisões travadas; sem entrevista adicional conforme a skill.
- Tracker local em `.scratch/` (sem git remoto); labels default com `ready-for-agent` aplicada neste spec.
- Docs próprios da ferramenta devem manter clean-room e não referenciar implementações comunitárias.

> [!IMPORTANT]
> **Endurecimento pós code-review (PR #16, issue #8).** O review de dois eixos não apontou violações duras, mas o eixo Spec encontrou 5 gaps — todos corrigidos no próprio PR e cobertos pelo teste e2e:
>
> 1. **Rotate invalida o antigo** — o e2e só exercia `token --show`; agora rotaciona, reafirma o novo via `--show` e prova rejeição 401 com o token antigo.
> 2. **Install com backup** — o e2e só cobria install fresco; agora reinstala com `--force`, afirma o `.bak` e que o arquivo carrega o token novo.
> 3. **Knobs e heartbeat** — o e2e só checava `200`; agora afirma `uri/timeout/buffer/throttle/heartbeat` no `.cfg` e que o post de heartbeat não gera happening espúrio (segue exatamente 1 linha).
> 4. **Descoberta de caminho** — o seam `find_dota_root_in` era testado só com diretório genérico; novo teste segue `libraryfolders.vdf` até a biblioteca com o manifesto.
> 5. **Logs filtram e status saudável** — agora afirma `--kind/--limit` e `status` running no meio do fluxo.
