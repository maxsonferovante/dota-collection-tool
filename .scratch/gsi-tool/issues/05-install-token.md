# 05 — Install da config e gestão do token

**What to build:** como jogador, rodo um comando e o arquivo de configuração aparece na pasta certa do Dota com o token ativo dentro — com detecção automática do diretório, override explícito, backup antes de sobrescrever e rotação de token que invalida o antigo.

**Blocked by:** 01 — Scaffold do workspace e portões de qualidade.

**Status:** ready-for-agent

- [ ] Detecção automática por SO encontra o diretório do Dota; override explícito vence; saída informa a flag de inicialização exigida pelo jogo
- [ ] Arquivo segue a convenção de prefixo/sufixo do jogo; sem sobrescrita sem force; sobrescrita gera backup; jogo exige restart após (re)instalar
- [ ] Token gera 32 bytes CSPRNG em hex, exibe o ativo, injeta no install e invalida o antigo ao rotacionar
- [ ] Escritor KeyValues próprio; nada referencia implementações comunitárias
