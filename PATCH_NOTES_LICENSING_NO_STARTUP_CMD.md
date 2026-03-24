Remoção definitiva do uso de parâmetros de linha de comando no fluxo de licenciamento.

- o licenciamento não faz mais parsing de argumentos de inicialização
- o comando get_startup_licensing_context retorna contexto vazio por compatibilidade
- o fluxo volta a iniciar apenas por configuração persistida e interface normal da aplicação
- o modo licensing_disabled permanece disponível apenas pela configuração interna
