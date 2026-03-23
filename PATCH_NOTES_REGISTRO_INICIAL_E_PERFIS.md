# Patch Notes — Registro inicial, licenciamento e perfis de empresa

## Ajustes aplicados

### 1. Registro inicial automático da aplicação
- criada uma tela/bloqueio de primeira inicialização (`StartupRegistrationGate`)
- nessa etapa o usuário informa apenas a empresa licenciada (nome e CPF/CNPJ)
- o nome da estação é capturado automaticamente pelo backend Tauri
- a chave da máquina é gerada automaticamente
- ao confirmar, o sistema salva as configurações e já tenta registrar/validar a estação na licença

### 2. Correção do fluxo de auto-registro
- o componente de licenciamento agora reconsulta a API após:
  - cadastro automático da empresa
  - cadastro automático da máquina
- isso permite concluir o registro na mesma validação, em vez de continuar bloqueado na primeira execução

### 3. Separação entre licença e perfis de empresa
- a tela de Configurações foi separada em dois blocos:
  - Licenciamento da aplicação
  - Perfis de empresa para importação/exportação
- a gravação de licença não sobrescreve mais os dados dos perfis
- a gravação dos perfis não depende mais do status/licença da aplicação

### 4. Correção da persistência de múltiplos perfis
- corrigido o backend que reduzia qualquer bundle para apenas 1 perfil
- agora `save_profile_bundle` e `load_profile_bundle` preservam todos os perfis
- o perfil selecionado é mantido corretamente no arquivo legado `conversion_profile.json`

### 5. Uso de perfis no módulo NFS-e
- o módulo NFS-e agora carrega o perfil selecionado a partir do bundle
- ao salvar o perfil ativo, o bundle inteiro é preservado
- não há mais colapso automático para perfil único ao salvar

### 6. Dashboard ajustado
- quantidade de perfis agora mostra o total real do bundle
- textos atualizados para deixar claro que perfis e licença são responsabilidades distintas

## Validação executada neste ambiente
- `npm run typecheck` ✅
- `npm run build:web` ✅

## Observação
- não foi possível executar `cargo check`, `cargo fmt`, `cargo clippy` ou `cargo test` neste ambiente porque o Rust/Cargo não estava disponível no container.
