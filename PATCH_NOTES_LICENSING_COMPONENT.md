# Patch Notes - Integração do novo componente de licenciamento

## O que foi feito

- substituição do motor interno de licenciamento do Integra Desktop pelo componente `generic-license-tauri` anexado
- inclusão do componente como crate local em `src-tauri/generic-license-tauri`
- reimplementação de `src-tauri/src/commands/licensing.rs` para usar o novo serviço
- preservação dos comandos Tauri já usados pelo frontend: `load_license_settings`, `save_license_settings`, `check_license_status`, `get_machine_fingerprint`, `get_app_meta`
- preservação do formato de retorno `LicenseRuntimeStatus` para evitar quebra no React
- manutenção do snapshot local existente em `storage/license.rs`

## Ajustes de compatibilidade aplicados no componente

Foram feitos pequenos ajustes de compatibilidade no crate incorporado para aceitar melhor o payload legado já usado pelo projeto:

- leitura de `maquinas` e `MAQUINAS` como lista de dispositivos
- leitura de `n_maquinas` e `N_MAQUINAS` como quantidade máxima de dispositivos
- leitura de `IDMAQUINA` e `idmaquina` como identificador do dispositivo

## Observação

O frontend atual continua funcional porque os comandos e o contrato principal de retorno foram mantidos.
