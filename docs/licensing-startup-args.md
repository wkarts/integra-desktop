# Catálogo de parâmetros de inicialização (licenciamento)

> Escopo: `Integra Desktop` (startup args via Tauri/Rust, lidos por `get_startup_licensing_context`).

## Regras de parsing

- Formatos aceitos:
  - `--chave valor`
  - `--chave=valor`
  - flags booleanas somente com presença: `--chave`
- Nomes são tratados sem diferenciação entre maiúsculas/minúsculas.
- Se um parâmetro for informado várias vezes, vale o último valor lido.
- `--silent`, `--headless` e `--no-ui` também forçam `ui-mode=silent` quando `--ui-mode` não for informado.
- Os parâmetros de startup são aplicados em tempo de execução mesmo quando já existe `license_settings.json` salvo (override em memória + persistência ao salvar configurações).

## Catálogo

| Grupo | Parâmetros aceitos | Campo interno | Tipo | Efeito |
|---|---|---|---|---|
| Auto registro global | `--auto-register` | `auto_register_enabled` | flag | Ativa modo de auto-registro no bootstrap. |
| Auto registro empresa | `--auto-register-company` | `auto_register_company` | flag | Sinaliza criação automática da empresa quando aplicável. |
| Auto registro dispositivo | `--auto-register-device` | `auto_register_device` | flag | Sinaliza criação automática do dispositivo quando aplicável. |
| Quantidade licenças | `--lic`, `--licenses` | `requested_licenses` | inteiro (`u32`) | Define quantidade solicitada para registro automático. |
| Documento empresa | `--company-document`, `--document`, `--cnpj` | `company_document` | string | Define documento/CNPJ no contexto inicial. |
| Nome empresa | `--company-name`, `--empresa` | `company_name` | string | Define nome da empresa no contexto inicial. |
| E-mail empresa | `--company-email`, `--email` | `company_email` | string | Define e-mail da empresa no contexto inicial. |
| Nome estação | `--station-name`, `--station` | `station_name` | string | Define estação para registro/licenciamento. |
| Nome dispositivo | `--device-name` | `device_name` | string | Define nome amigável do dispositivo para startup. |
| Identificador dispositivo | `--device`, `--device-id`, `--device-identifier` | `device_identifier` | string | Define identificador estável do dispositivo. |
| Modo validação | `--validation-mode` | `validation_mode` | string | Define estratégia de validação (`standard`, etc.). |
| Modo UI | `--ui-mode` | `interface_mode` | string | Define modo de interface no fluxo de ativação. |
| Sem UI | `--silent`, `--headless`, `--no-ui` | `no_ui` | flag | Evita abertura do gate visual de ativação no bootstrap. |
| Licença local ativa | `--local-license` | `local_license_enabled` | flag | Informa modo de licença local ativo no startup. |
| Gerar licença local | `--local-license-generate` | `local_license_generate` | flag | Indica operação de geração local de licença. |
| Arquivo licença local | `--local-license-file` | `local_license_file_path` | string | Define caminho do artefato local de licença. |
| Token licença local | `--local-license-token` | `local_license_token_present` | string/flag | Marca presença de token para fluxo local. |
| Segredo licença local | `--local-license-secret` | `developer_secret_present` | string/flag | Marca presença de segredo para assinatura/validação. |
| Conta licença local | `--local-license-account` | `local_license_account` | string | Define conta associada à licença local. |
| Emissor licença local | `--local-license-issuer` | `local_license_issuer` | string | Define emissor da licença local. |
| Desabilitar licenciamento | `--disable-licensing`, `--licensing-disabled`, `--no-license` | `licensing_disabled` | flag | Desativa completamente a validação de licenciamento no runtime. |

## Exemplo completo

```bash
integra-desktop \
  --auto-register \
  --auto-register-company \
  --auto-register-device \
  --licenses=5 \
  --cnpj 12345678000199 \
  --company-name "Empresa Exemplo" \
  --company-email suporte@empresa.com.br \
  --station-name "CPD-01" \
  --device-identifier "FILIAL-01-PC-01" \
  --validation-mode standard \
  --ui-mode interactive
```

## Exemplo modo sem UI

```bash
integra-desktop --no-ui --auto-register --cnpj 12345678000199
```

## Exemplo para desabilitar licenciamento

```bash
integra-desktop --no-license
```
