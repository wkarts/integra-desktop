# Componente genérico de licenciamento para Rust + Tauri

Este pacote já sai pré-configurado para a sua API WWSoftware em `https://api.rest.wwsoftwares.com.br/api/v1`.

Este componente porta a lógica principal do Delphi para Rust/Tauri com:

- validação online;
- cache offline em JSON;
- geração automática de `device_key`;
- fluxo com login e sem login;
- comando Tauri pronto para `invoke`.

## 1. Dependência

Adicione o diretório como crate local ou publique internamente.

```toml
[dependencies]
generic-license-tauri = { path = "./generic-license-tauri", features = ["tauri-commands"] }
```

## 2. Configuração

Monte a configuração em `main.rs`:

```rust
use generic_license_tauri::{GenericLicenseService, models::LicenseConfig};

let config = LicenseConfig {
    base_url: std::env::var("LICENSE_API_BASE_URL").unwrap_or_else(|_| "https://api.rest.wwsoftwares.com.br/api/v1".to_string()),
    api_token: std::env::var("LICENSE_API_TOKEN").ok(),
    status_endpoint: std::env::var("LICENSE_API_COMPANY_STATUS_ENDPOINT").unwrap_or_else(|_| "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/cliente/{document}".to_string()),
    register_company_endpoint: std::env::var("LICENSE_API_REGISTER_COMPANY_ENDPOINT").unwrap_or_else(|_| "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/clientes".to_string()),
    register_device_endpoint: std::env::var("LICENSE_API_REGISTER_DEVICE_ENDPOINT").unwrap_or_else(|_| "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/maquinas".to_string()),
    update_device_endpoint: std::env::var("LICENSE_API_UPDATE_DEVICE_ENDPOINT").unwrap_or_else(|_| "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/maquinas/IDMAQUINA/{id}".to_string()),
    cache_namespace: "erp-desktop".to_string(),
    ..Default::default()
};

let service = GenericLicenseService::new(config);
```

## 3. Registro no Tauri

Arquivo de exemplo: `examples/with-login/src-tauri-main.rs`

```rust
.manage(std::sync::Arc::new(service))
.invoke_handler(tauri::generate_handler![license_check])
```

## 4. Uso com login

No frontend, antes da autenticação do usuário:

```ts
const decision = await invoke("license_check", {
  input: {
    company_document: cnpj,
    company_name: razaoSocial,
    app_id: "erp-desktop",
    app_name: "ERP Desktop",
    app_version: "1.0.0",
    login_context: true
  }
});

if (!decision.allowed) {
  alert(decision.message);
  return;
}
```

Exemplo completo em `examples/with-login/frontend-login.ts`.

## 5. Uso sem login

Valide a licença no startup da aplicação, antes de abrir as telas principais.

Exemplo em `examples/without-login/frontend-startup.ts`.

## 6. Formato sugerido da API

### GET `/81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/cliente/{document}`

```json
{
  "status": 1,
  "message": "Cliente localizado",
  "license": {
    "document": "12345678000199",
    "company_name": "Empresa XPTO",
    "blocked": false,
    "active": true,
    "expires_at": "2026-12-31T23:59:59-03:00",
    "max_devices": 3,
    "devices": [
      {
        "id": "10",
        "device_key": "sha256",
        "device_name": "HOST - windows - x86_64",
        "blocked": false
      }
    ]
  }
}
```

## 7. Mapeamento da lógica Delphi para o Rust/Tauri

- `CheckLic(...)` -> `GenericLicenseService::check(...)`
- `LocalCacheHandler` -> `OfflineCache`
- `TReadClientesAutorizados` -> `LicenseApiClient`
- serial/chave da máquina -> `generate_device_key(...)`
- chamada na tela de login -> `license_check` antes do login
- aplicação sem login -> `license_check` no startup

## 8. Observações

1. O pacote já sai apontando para o seu domínio e para os mesmos endpoints UUID usados no legado Delphi.
2. A leitura da licença aceita tanto o formato genérico quanto o retorno legado em maiúsculas da sua API atual.
3. O componente continua reutilizável para qualquer aplicativo Tauri, inclusive white-label.
4. Caso queira, você pode substituir a lógica de `device_key` por um serial próprio vindo do seu backend/local.

## 9. Estrutura

- `src/service.rs`: regra principal
- `src/client.rs`: cliente HTTP
- `src/cache.rs`: cache offline
- `src/device.rs`: fingerprint do dispositivo
- `src/commands.rs`: comando Tauri
