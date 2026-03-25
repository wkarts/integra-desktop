# Patch Notes — Exportação de Faturas Legado

## Ajustes implementados

- Separação explícita entre o fluxo de exportação **XML/SPED** e o fluxo de exportação **LEGADO TXT/CSV/pipe**.
- Novo command Rust: `export_nfe_faturas_legacy_txt`.
- O botão padrão de exportação TXT/CSV agora exporta **apenas registros não legados**.
- A exportação SPED agora considera **apenas registros não legados**.
- Novo botão na interface: **Exportar Faturas Legado**.
- Detecção de contexto legado baseada em `row.legado`, ou seja, na origem real dos dados carregados.
- A exportação legado ignora a exigência de `SPED MATCH`, evitando que parcelas importadas pelo quadro legado sejam descartadas indevidamente.
- Quando houver contexto legado, a interface exibe contadores separados para:
  - XML/SPED elegíveis
  - Legado elegíveis

## Arquivos alterados

- `src/modules/nfe-faturas/pages/NfeFaturasPage.tsx`
- `src/modules/nfe-faturas/services/tauriService.ts`
- `src-tauri/src/commands/nfe_faturas.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/lib.rs`

## Observação

O patch foi preparado com foco em não quebrar o fluxo atual já existente.
