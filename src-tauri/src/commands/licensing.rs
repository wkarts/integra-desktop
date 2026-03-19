use std::time::Duration;

use reqwest::blocking::Client;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tauri::AppHandle;

use crate::core::domain::license::{AppMeta, LicenseCheckResult, LicenseSettings};

#[tauri::command]
pub fn get_machine_fingerprint() -> Result<String, String> {
    Ok(machine_fingerprint())
}

#[tauri::command]
pub fn get_app_meta() -> Result<AppMeta, String> {
    Ok(AppMeta {
        product_name: "Integra Desktop".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        build_hash: option_env!("GIT_HASH").unwrap_or("dev-local").into(),
        app_id: "br.com.wwsoftwares.integra.desktop".into(),
    })
}

#[tauri::command]
pub fn load_license_settings(app: AppHandle) -> Result<Option<LicenseSettings>, String> {
    crate::storage::license::load_license_settings(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_license_settings(
    settings: LicenseSettings,
    app: AppHandle,
) -> Result<LicenseSettings, String> {
    let mut next = settings;
    if next.machine_key.trim().is_empty() {
        next.machine_key = machine_fingerprint();
    }
    crate::storage::license::save_license_settings(&app, &next).map_err(|e| e.to_string())?;
    Ok(next)
}

#[tauri::command]
pub fn check_license_status(
    settings: LicenseSettings,
    app: AppHandle,
) -> Result<LicenseCheckResult, String> {
    let mut next_settings = settings.clone();
    if next_settings.machine_key.trim().is_empty() {
        next_settings.machine_key = machine_fingerprint();
    }

    if next_settings.service_url.trim().is_empty() {
        return Ok(LicenseCheckResult {
            allowed: true,
            online: false,
            active: false,
            blocked: false,
            device_registered: false,
            device_blocked: false,
            seats_total: 0,
            seats_used: 0,
            company_name: next_settings.company_name.clone(),
            company_document: next_settings.company_document.clone(),
            expires_at: None,
            message: "Webservice de licenciamento ainda não configurado. Operação em modo assistido.".into(),
            machine_key: next_settings.machine_key.clone(),
            status_code: 0,
        });
    }

    let base_url = next_settings.service_url.trim_end_matches('/');
    let company_document = only_digits(&next_settings.company_document);
    let station_name = if next_settings.station_name.trim().is_empty() {
        hostname::get().ok().and_then(|v| v.into_string().ok()).unwrap_or_else(|| "ESTACAO".into())
    } else {
        next_settings.station_name.clone()
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let request_url = format!("{}/cliente/{}", base_url, company_document);
    let response = client
        .get(request_url)
        .query(&[
            ("chave", next_settings.machine_key.as_str()),
            ("nomemaq", station_name.as_str()),
            ("app", next_settings.app_instance.as_str()),
            ("versao", env!("CARGO_PKG_VERSION")),
        ])
        .send()
        .map_err(|e| e.to_string())?;

    let body: Value = response.json().map_err(|e| e.to_string())?;
    let status_code = find_i64(&body, &["STATUS", "status"]).unwrap_or(1) as i32;
    let active = find_string(&body, &["ATIVO", "ativo"]).map(|v| !matches!(v.as_str(), "N" | "0" | "false")).unwrap_or(true);
    let blocked = find_string(&body, &["BLOQUEADO", "bloqueado"]).map(|v| matches!(v.as_str(), "S" | "1" | "true")).unwrap_or(false);
    let company_name = find_string(&body, &["RAZAO", "razao", "empresa", "EMPRESA"]).unwrap_or_else(|| next_settings.company_name.clone());
    let expires_at = find_string(&body, &["DATA_VAL_LIC", "data_val_lic", "expires_at", "validade"]);
    let seats_total = find_i64(&body, &["QTD_MAQ", "qtd_maq", "n_maquinas", "N_MAQUINAS"]).unwrap_or(0).max(0) as u32;
    let devices = find_array(&body, &["COMPUTADORES", "computadores", "maquinas", "MAQUINAS"]).cloned().unwrap_or_default();
    let seats_used = devices.len() as u32;
    let mut device_registered = false;
    let mut device_blocked = false;

    for device in &devices {
        let key = find_string(device, &["CHAVE", "chave"]).unwrap_or_default();
        if key == next_settings.machine_key {
            device_registered = true;
            device_blocked = find_string(device, &["BLOQUEADO", "bloqueado"]).map(|v| matches!(v.as_str(), "S" | "1" | "true")).unwrap_or(false);
            break;
        }
    }

    if !device_registered && next_settings.auto_register_machine {
        let payload = json!({
            "cnpj": company_document,
            "chave": next_settings.machine_key,
            "nome": next_settings.company_name,
            "nomemaq": station_name,
            "versaoexe": env!("CARGO_PKG_VERSION"),
            "sistema_operacional": std::env::consts::OS,
            "tipo": next_settings.app_instance,
            "email": next_settings.company_email,
        });
        let _ = client.post(format!("{}/maquinas", base_url)).json(&payload).send();
    }

    let has_free_slot = seats_total == 0 || seats_used < seats_total || device_registered;
    let allowed = status_code > 0 && active && !blocked && !device_blocked && has_free_slot;
    let mut message = find_string(&body, &["MESSAGE", "message", "mensagem"]).unwrap_or_default();
    if message.trim().is_empty() {
        message = if allowed {
            if device_registered {
                "Licença validada e estação autorizada.".into()
            } else if has_free_slot {
                "Licença validada. Estação ainda não retornou como cadastrada no serviço.".into()
            } else {
                "Quantidade de estações liberadas já foi atingida.".into()
            }
        } else if blocked || device_blocked {
            "Licença ou estação bloqueada pelo serviço.".into()
        } else {
            "Licença não autorizada para esta estação.".into()
        };
    }

    let result = LicenseCheckResult {
        allowed,
        online: true,
        active,
        blocked,
        device_registered,
        device_blocked,
        seats_total,
        seats_used,
        company_name,
        company_document: company_document.clone(),
        expires_at,
        message,
        machine_key: next_settings.machine_key.clone(),
        status_code,
    };

    crate::storage::license::save_license_settings(&app, &next_settings).map_err(|e| e.to_string())?;
    crate::storage::logs::append_log(&app, &format!("Licenciamento consultado: {} | {}", result.company_document, result.message)).map_err(|e| e.to_string())?;
    Ok(result)
}

fn machine_fingerprint() -> String {
    let host = hostname::get().ok().and_then(|value| value.into_string().ok()).unwrap_or_else(|| "unknown-host".into());
    let user = std::env::var("USERNAME").or_else(|_| std::env::var("USER")).unwrap_or_else(|_| "unknown-user".into());
    let base = format!("{}|{}|{}|{}|{}", host, user, std::env::consts::OS, std::env::consts::ARCH, env!("CARGO_PKG_NAME"));
    let mut hasher = Sha256::new();
    hasher.update(base.as_bytes());
    let result = hasher.finalize();
    result[..16].iter().map(|item| format!("{:02X}", item)).collect::<Vec<String>>().join("")
}

fn only_digits(value: &str) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn find_string(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => keys.iter().find_map(|key| map.get(*key)).and_then(|item| match item {
            Value::String(text) => Some(text.clone()),
            Value::Number(number) => Some(number.to_string()),
            Value::Bool(flag) => Some(flag.to_string()),
            _ => None,
        }),
        _ => None,
    }
}

fn find_i64(value: &Value, keys: &[&str]) -> Option<i64> {
    match value {
        Value::Object(map) => {
            keys.iter()
                .find_map(|key| map.get(*key))
                .and_then(|item| match item {
                    Value::Number(number) => number.as_i64(),
                    Value::String(text) => text.parse::<i64>().ok(),
                    _ => None,
                })
        },
        _ => None,
    }
}

fn find_array<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Vec<Value>> {
    match value {
        Value::Object(map) => keys
            .iter()
            .find_map(|key| map.get(*key))
            .and_then(|item| item.as_array()),
        _ => None,
    }
}
