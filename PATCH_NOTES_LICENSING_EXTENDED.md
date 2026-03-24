# Patch notes - licenciamento estendido

## Entregue

- auto-registro opcional por parâmetros de linha de comando
- expansão de parâmetros suportados
- overlay dos parâmetros no fluxo atual sem quebrar compatibilidade
- geração opcional de licença local
- validação opcional de licença local com secret/token do desenvolvedor
- geração de URI `otpauth://` compatível com QR renderer/Google Authenticator
- novos comandos Tauri para contexto de inicialização e licença local
- documentação operacional completa

## Arquivos principais alterados

- `src-tauri/src/commands/licensing.rs`
- `src-tauri/src/core/domain/license.rs`
- `src-tauri/src/core/startup.rs`
- `src-tauri/src/core/local_license.rs`
- `src-tauri/src/core/mod.rs`
- `src-tauri/generic-license-tauri/src/models.rs`
- `src-tauri/generic-license-tauri/src/client.rs`
- `src-tauri/generic-license-tauri/src/service.rs`
- `src/shared/types/index.ts`
- `src/modules/nfse-servicos/services/tauriService.ts`
- `src/modules/licensing/components/StartupRegistrationGate.tsx`
- `docs/LICENSING_EXTENDED.md`
- `examples/licensing/local-license.example.json`
