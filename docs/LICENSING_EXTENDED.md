# Licenciamento estendido: fluxo padrão, auto-registro opcional e licença local opcional

## Objetivo

Este documento descreve a ampliação do licenciamento do Integra Desktop sem quebrar o fluxo já existente.

A implementação separa três cenários:

1. **Fluxo padrão atual**
   - valida empresa/dispositivo já cadastrados;
   - mantém compatibilidade total;
   - continua funcionando quando nenhum parâmetro novo é informado.

2. **Fluxo opcional de auto-registro**
   - habilitado explicitamente por parâmetros na execução do binário;
   - permite localizar/cadastrar empresa e dispositivo automaticamente;
   - aceita quantidade de licenças solicitadas e metadados adicionais.

3. **Fluxo opcional de licença local**
   - habilitado explicitamente por parâmetros;
   - valida um arquivo de licença local com assinatura baseada em secret do desenvolvedor;
   - pode gerar URI `otpauth://` compatível com Google Authenticator/QR renderer externo.

---

## Requisito de compatibilidade

Se nenhum parâmetro novo for informado:

- o fluxo atual continua sendo executado normalmente;
- o comportamento padrão não é substituído;
- os recursos novos apenas complementam o sistema.

---

## Parâmetros de execução suportados

### Auto-registro

| Parâmetro | Finalidade | Formato | Padrão |
|---|---|---|---|
| `--auto-register` | habilita o modo opcional de auto-registro | flag | desativado |
| `--auto-register-company` | força auto-criação/reaproveitamento da empresa | flag | desativado |
| `--auto-register-device` | força auto-criação/atualização do dispositivo | flag | desativado |
| `--lic` / `--licenses` | quantidade de licenças solicitadas/liberadas no cadastro | inteiro | não informado |
| `--company-document` / `--document` / `--cnpj` | documento da empresa | texto | vazio |
| `--company-name` / `--empresa` | razão social/nome da empresa | texto | vazio |
| `--company-email` / `--email` | e-mail da empresa | texto | vazio |
| `--station-name` / `--station` | nome lógico da estação | texto | captura automática |
| `--device-name` | nome amigável do dispositivo | texto | captura automática |
| `--device` / `--device-id` / `--device-identifier` | identificador adicional do dispositivo | texto | vazio |
| `--validation-mode` | define o modo de validação | `standard`, `online-only`, `local-only`, `prefer-local` | `standard` |
| `--ui-mode` | define o comportamento da interface | `interactive`, `assisted`, `silent` | `interactive` |
| `--silent` / `--headless` / `--no-ui` | força interface silenciosa | flag | desativado |

### Licença local

| Parâmetro | Finalidade | Formato | Padrão |
|---|---|---|---|
| `--local-license` | habilita a validação de licença local | flag | desativado |
| `--local-license-generate` | indica intenção de gerar licença local | flag | desativado |
| `--local-license-file` | caminho do arquivo local | texto | `<exe>/<app>.local.lic.json` |
| `--local-license-token` | token opcional do desenvolvedor | texto | vazio |
| `--local-license-secret` | secret opcional do desenvolvedor | texto | vazio |
| `--local-license-account` | conta usada na URI `otpauth://` | texto | documento da empresa |
| `--local-license-issuer` | emissor da URI `otpauth://` | texto | `WWSoftwares Local License` |

---

## Exemplos reais de uso

### Fluxo padrão atual

```bash
integra.exe
```

### Auto-registro com quantidade de licenças

```bash
integra.exe --auto-register --lic 5 --company-document 12345678000199 --company-name "Empresa Exemplo LTDA" --company-email suporte@empresa.com.br
```

### Auto-registro com identificador interno do dispositivo

```bash
integra.exe --auto-register --device-id ETQ-CPD-01 --station-name CPD-01 --validation-mode standard
```

### Auto-registro com interface silenciosa

```bash
integra.exe --auto-register --licenses 10 --headless --company-document 12345678000199
```

### Validação de licença local

```bash
integra.exe --local-license --local-license-file "C:\ProgramData\WWSoftwares\IntegraDesktop\integra-desktop.local.lic.json" --local-license-secret "SEU_SECRET_INTERNO"
```

### Modo local-only

```bash
integra.exe --local-license --validation-mode local-only --local-license-secret "SEU_SECRET_INTERNO"
```

---

## Combinações válidas

- `--auto-register` pode ser combinado com `--company-*`, `--device-*`, `--lic`, `--validation-mode` e `--ui-mode`.
- `--local-license` pode ser combinado com `--validation-mode prefer-local` ou `local-only`.
- `--headless`/`--silent` pode ser combinado com auto-registro para cenários de implantação automatizada.

### Recomendações

- Em produção, prefira `standard` ou `prefer-local`.
- Use `local-only` apenas quando a operação realmente depender de licença local.
- Não persista `--local-license-secret` em atalhos públicos ou scripts expostos.

---

## Fluxo operacional implementado

### 1. Fluxo padrão

1. a aplicação sobe normalmente;
2. coleta metadados do dispositivo;
3. valida empresa/dispositivo já cadastrados;
4. segue o comportamento atual.

### 2. Fluxo de auto-registro

1. a aplicação detecta `--auto-register`;
2. faz overlay dos dados recebidos por parâmetro sobre o `LicenseSettings` local;
3. completa os dados faltantes com coleta automática do dispositivo;
4. envia ao componente genérico de licenciamento:
   - empresa;
   - dispositivo;
   - quantidade de licenças solicitadas;
   - modo de validação;
   - comportamento de interface;
   - identificador adicional do dispositivo;
5. se a empresa/dispositivo não existirem, o componente pode cadastrar automaticamente;
6. após o cadastro, a validação continua normalmente.

### 3. Fluxo de licença local

1. a aplicação detecta `--local-license`;
2. tenta localizar o arquivo local informado ou o caminho padrão;
3. valida a assinatura usando o secret do desenvolvedor;
4. opcionalmente confere documento da empresa e chave da máquina;
5. se estiver válido, libera a execução usando runtime local;
6. se estiver inválido e `validation-mode=local-only`, bloqueia;
7. caso contrário, segue para o webservice normal.

---

## Estrutura da licença local

O arquivo local gerado/validado segue JSON assinado:

```json
{
  "version": 1,
  "issuer": "WWSoftwares Local License",
  "app_instance": "integra-desktop",
  "company_name": "Empresa Exemplo LTDA",
  "company_document": "12345678000199",
  "company_email": "suporte@empresa.com.br",
  "station_name": "CPD-01",
  "machine_key": "...",
  "serial_number": "ABC123456",
  "requested_licenses": 5,
  "issued_at": "2026-03-23T10:00:00-03:00",
  "expires_at": "2026-12-31T23:59:59-03:00",
  "signature": "BASE64_SHA256_SIGNATURE"
}
```

---

## Segurança

### Auto-registro

- o fluxo continua opcional;
- se não habilitado, nada muda;
- o backend continua sendo o responsável final pela política de licença;
- parâmetros de linha de comando devem ser tratados como operação assistida ou de implantação.

### Licença local

- assinatura baseada em secret do desenvolvedor;
- suporte opcional a token adicional via `LICENSE_LOCAL_DEV_TOKEN`;
- recomendação de uso com secret em variável de ambiente:
  - `LICENSE_LOCAL_DEV_SECRET`
  - `LICENSE_LOCAL_DEV_TOKEN`
- geração de URI `otpauth://` para integração com QR renderer/Google Authenticator.

---

## Variáveis de ambiente relacionadas

| Variável | Uso |
|---|---|
| `LICENSE_API_TOKEN` | bearer token da API de licenciamento |
| `LICENSE_API_RESOLVE_ENDPOINT` | endpoint orquestrado de ativação |
| `LICENSE_API_COMPANY_STATUS_ENDPOINT` | endpoint legado de status da empresa |
| `LICENSE_API_REGISTER_COMPANY_ENDPOINT` | endpoint legado de cadastro da empresa |
| `LICENSE_API_REGISTER_DEVICE_ENDPOINT` | endpoint legado de cadastro do dispositivo |
| `LICENSE_API_UPDATE_DEVICE_ENDPOINT` | endpoint legado de atualização do dispositivo |
| `LICENSE_REGISTRATION_PUBLIC_KEY_BASE64` | chave pública do arquivo/certificado local de registro |
| `LICENSE_LOCAL_DEV_SECRET` | secret do desenvolvedor para licença local |
| `LICENSE_LOCAL_DEV_TOKEN` | token opcional adicional para licença local |

---

## Comandos Tauri adicionados

- `get_startup_licensing_context`
- `generate_local_license`
- `validate_local_license`

---

## Observações finais

- o objetivo desta ampliação é oferecer mais flexibilidade operacional ao desenvolvedor, implantação e suporte técnico;
- o fluxo atual permanece compatível;
- os novos recursos só entram em ação quando explicitamente habilitados.
