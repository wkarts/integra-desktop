# PATCH NOTES - LICENCIAMENTO STARTUP / COMPATIBILIDADE

## Ajustes aplicados

- removido o acionamento de registro e auto-registro por parâmetros de linha de comando no startup
- mantidos os recursos novos de auto-registro e licença local como funcionalidades adicionais do módulo
- adicionado parâmetro de startup para desabilitar totalmente o licenciamento:
  - `--disable-licensing`
  - `--licensing-disabled`
  - `--no-license`
- corrigido o modal inicial para não disparar múltiplas inicializações por efeito duplicado
- removido o auto-enable incorreto de `auto_register_machine` no bootstrap do frontend
- mantido o fluxo padrão atual do licenciamento
- mantido o recurso adicional de auto-registro pelo fluxo interno da aplicação
- mantido o recurso adicional de licença local por comandos próprios

## Resultado esperado

- a aplicação deixa de travar/recarregar repetidamente por causa dos argumentos de startup ligados ao registro
- o modal inicial deixa de ser disparado em cascata
- o desenvolvedor pode desabilitar totalmente o licenciamento por parâmetro quando necessário
- os recursos novos continuam disponíveis sem quebrar o fluxo padrão existente
