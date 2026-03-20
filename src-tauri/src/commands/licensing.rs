use std::time::Duration;

use chrono::{NaiveDate, Utc};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tauri::AppHandle;

use crate::core::domain::license::{
    AppMeta, LicenseRuntimeStatus, LicenseSettings, LicenseSnapshot, LicensedCompany,
    LicensedDevice, LocalLicense,
};

const URL_WS_LICENCA: &str = "https://api.rest.wwsoftwares.com.br";
const URL_ENDPOINT: &str = "/api/v1/";
const DEFAULT_APP_INSTANCE: &str = "integra-desktop";

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
    let mut next = normalize_license_settings(settings);
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
) -> Result<LicenseRuntimeStatus, String> {
    let mut next_settings = normalize_license_settings(settings.clone());
    if next_settings.machine_key.trim().is_empty() {
        next_settings.machine_key = machine_fingerprint();
    }

    let station_name = resolve_station_name(&next_settings.station_name);

    let base_url = next_settings.service_url.trim_end_matches('/');
    let company_document = only_digits(&next_settings.company_document);

    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let company_url = format!("{}/cliente/{}", base_url, company_document);
    let response = match client
        .get(company_url)
        .query(&[
            ("chave", next_settings.machine_key.as_str()),
            ("nomemaq", station_name.as_str()),
            ("app", next_settings.app_instance.as_str()),
            ("versao", env!("CARGO_PKG_VERSION")),
        ])
        .send()
    {
        Ok(resp) => resp,
        Err(error) => {
            return Ok(from_cache_or_default(
                &app,
                &next_settings,
                false,
                &format!("Falha de rede no licenciamento: {error}"),
            ));
        }
    };

    if !response.status().is_success() {
        return Ok(from_cache_or_default(
            &app,
            &next_settings,
            false,
            &format!(
                "Licenciamento retornou status HTTP {}",
                response.status().as_u16()
            ),
        ));
    }

    let body: Value = response.json().map_err(|e| e.to_string())?;
    let status_code = find_i64(&body, &["STATUS", "status"]).unwrap_or(1) as i32;

    let mut company = map_licensed_company(&body, &company_document, &next_settings);
    let mut local_license = map_local_license(&body, &company);
    let mut devices = map_devices(&body, &company_document);

    let current_idx = devices
        .iter()
        .position(|item| item.chave == next_settings.machine_key);

    if current_idx.is_none() && next_settings.auto_register_machine {
        let _ = ensure_company_registered(base_url, &client, &company, &local_license);

        if register_machine(base_url, &client, &company, &next_settings, &station_name) {
            devices.push(LicensedDevice {
                cnpj: company_document.clone(),
                chave: next_settings.machine_key.clone(),
                nome: next_settings.company_name.clone(),
                nome_compu: station_name.clone(),
                versaoexe: env!("CARGO_PKG_VERSION").into(),
                sistema_operacional: std::env::consts::OS.into(),
                tipo: next_settings.app_instance.clone(),
                ..LicensedDevice::default()
            });
        }
    }

    if let Some(existing_idx) = devices
        .iter()
        .position(|item| item.chave == next_settings.machine_key)
    {
        let existing = &devices[existing_idx];
        if existing.idmaquina > 0 {
            let _ = update_machine(
                base_url,
                &client,
                existing.idmaquina,
                &company,
                &next_settings,
                &station_name,
            );
        }
    }

    let current_device = devices
        .iter()
        .find(|item| item.chave == next_settings.machine_key)
        .cloned();

    let seats_total = company.n_maquinas.max(0) as u32;
    let seats_used = devices.len() as u32;
    let machine_registered = current_device.is_some();
    let machine_blocked = current_device
        .as_ref()
        .map(|item| item.bloqueado)
        .unwrap_or(false);
    let company_blocked = company.bloqueado || company.bloqueio_admin || local_license.bloqueio;

    if company.razaosocial.is_empty() && !next_settings.company_name.is_empty() {
        company.razaosocial = next_settings.company_name.clone();
    }

    if local_license.cnpj.is_empty() {
        local_license.cnpj = company_document.clone();
    }

    let expiry = first_non_empty(vec![
        company.data_val_lic.clone(),
        local_license.competencia.clone(),
        find_string(&body, &["DATA_VAL_LIC", "data_val_lic", "validade"]).unwrap_or_default(),
    ]);

    let expired = expiry.as_deref().map(is_expired).unwrap_or(false);

    let seat_violation = seats_total > 0 && seats_used > seats_total && !machine_registered;
    let active = company.ativo || local_license.ativo;

    let mut allowed = status_code > 0
        && active
        && !company_blocked
        && !machine_blocked
        && !expired
        && !seat_violation;

    if seats_total > 0 && !machine_registered && seats_used >= seats_total {
        allowed = false;
    }

    let block_reason = if company_blocked {
        Some("company_blocked".into())
    } else if machine_blocked {
        Some("machine_blocked".into())
    } else if expired {
        Some("license_expired".into())
    } else if !machine_registered && seats_total > 0 && seats_used >= seats_total {
        Some("seats_exceeded".into())
    } else if !active {
        Some("company_inactive".into())
    } else if status_code <= 0 {
        Some("service_denied".into())
    } else {
        None
    };

    let message = match block_reason.as_deref() {
        Some("company_blocked") => "Empresa bloqueada no licenciamento.".into(),
        Some("machine_blocked") => "Estação bloqueada no licenciamento.".into(),
        Some("license_expired") => "Licença vencida para esta empresa.".into(),
        Some("seats_exceeded") => "Limite de máquinas excedido para esta licença.".into(),
        Some("company_inactive") => "Empresa inativa no serviço de licenciamento.".into(),
        Some("service_denied") => "Serviço de licenciamento recusou a validação.".into(),
        _ => "Licença válida para execução silenciosa.".into(),
    };

    let runtime = LicenseRuntimeStatus {
        online: true,
        allowed,
        blocked: !allowed,
        machine_registered,
        machine_blocked,
        seats_total,
        seats_used,
        expiry,
        message,
        block_reason,
        technical_message: find_string(&body, &["MESSAGE", "message", "mensagem"])
            .unwrap_or_else(|| "OK".into()),
        company_name: company.razaosocial.clone(),
        company_document: company_document.clone(),
        machine_key: next_settings.machine_key.clone(),
        status_code,
        local_license: Some(local_license.clone()),
        licensed_company: Some(company.clone()),
        licensed_device: current_device.clone(),
    };

    let snapshot = LicenseSnapshot {
        last_sync_at: Utc::now().to_rfc3339(),
        local_license: Some(local_license),
        licensed_company: Some(company),
        licensed_devices: devices,
        runtime_status: runtime.clone(),
    };

    crate::storage::license::save_license_snapshot(&app, &snapshot).map_err(|e| e.to_string())?;
    crate::storage::license::save_license_settings(&app, &next_settings)
        .map_err(|e| e.to_string())?;

    if !runtime.allowed {
        crate::storage::logs::append_log(
            &app,
            &format!(
                "Licenciamento bloqueado: {} | {:?}",
                runtime.company_document, runtime.block_reason
            ),
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(runtime)
}

fn from_cache_or_default(
    app: &AppHandle,
    settings: &LicenseSettings,
    online: bool,
    technical_message: &str,
) -> LicenseRuntimeStatus {
    let fallback = LicenseRuntimeStatus {
        online,
        allowed: true,
        blocked: false,
        machine_registered: false,
        machine_blocked: false,
        seats_total: 0,
        seats_used: 0,
        expiry: None,
        message: "Operando com cache local de licenciamento.".into(),
        block_reason: None,
        technical_message: technical_message.into(),
        company_name: settings.company_name.clone(),
        company_document: only_digits(&settings.company_document),
        machine_key: settings.machine_key.clone(),
        status_code: 0,
        local_license: None,
        licensed_company: None,
        licensed_device: None,
    };

    let cached = crate::storage::license::load_license_snapshot(app)
        .ok()
        .flatten()
        .map(|snapshot| {
            let mut runtime = snapshot.runtime_status;
            runtime.online = online;
            runtime.technical_message = technical_message.into();
            runtime
        });

    cached.unwrap_or(fallback)
}

fn resolve_station_name(input: &str) -> String {
    if input.trim().is_empty() {
        return hostname::get()
            .ok()
            .and_then(|v| v.into_string().ok())
            .unwrap_or_else(|| "ESTACAO".into());
    }
    input.to_string()
}

fn normalize_license_settings(settings: LicenseSettings) -> LicenseSettings {
    let mut next = settings;
    next.service_url = format!("{}{}", URL_WS_LICENCA.trim_end_matches('/'), URL_ENDPOINT)
        .trim_end_matches('/')
        .to_string();
    next.app_instance = DEFAULT_APP_INSTANCE.into();
    next.auto_register_machine = true;
    next
}

fn ensure_company_registered(
    base_url: &str,
    client: &Client,
    company: &LicensedCompany,
    local_license: &LocalLicense,
) -> bool {
    let payload = json!({
        "cnpj": company.cnpj,
        "razaosocial": company.razaosocial,
        "emp_nomefantasia": company.emp_nomefantasia,
        "emp_email": company.emp_email,
        "emp_endereco": company.emp_endereco,
        "emp_numero": company.emp_numero,
        "emp_bairro": company.emp_bairro,
        "emp_cidade": company.emp_cidade,
        "emp_uf": company.emp_uf,
        "emp_cep": company.emp_cep,
        "emp_complemento": company.emp_complemento,
        "emp_serie": company.emp_serie,
        "n_maquinas": company.n_maquinas,
        "ativo": company.ativo,
        "bloqueado": company.bloqueado,
        "serial": local_license.serial,
        "serial_key": local_license.serial_key,
    });

    client
        .post(format!("{}/clientes", base_url))
        .json(&payload)
        .send()
        .map(|res| res.status().is_success() || res.status() == StatusCode::CONFLICT)
        .unwrap_or(false)
}

fn register_machine(
    base_url: &str,
    client: &Client,
    company: &LicensedCompany,
    settings: &LicenseSettings,
    station_name: &str,
) -> bool {
    let payload = json!({
        "cnpj": company.cnpj,
        "chave": settings.machine_key,
        "nome": company.razaosocial,
        "nomemaq": station_name,
        "versaoexe": env!("CARGO_PKG_VERSION"),
        "sistema_operacional": std::env::consts::OS,
        "tipo": settings.app_instance,
        "email": settings.company_email,
    });

    client
        .post(format!("{}/maquinas", base_url))
        .json(&payload)
        .send()
        .map(|res| res.status().is_success() || res.status() == StatusCode::CONFLICT)
        .unwrap_or(false)
}

fn update_machine(
    base_url: &str,
    client: &Client,
    machine_id: i64,
    company: &LicensedCompany,
    settings: &LicenseSettings,
    station_name: &str,
) -> bool {
    let payload = json!({
        "idmaquina": machine_id,
        "cnpj": company.cnpj,
        "chave": settings.machine_key,
        "nome": company.razaosocial,
        "nomemaq": station_name,
        "versaoexe": env!("CARGO_PKG_VERSION"),
        "sistema_operacional": std::env::consts::OS,
        "tipo": settings.app_instance,
        "email": settings.company_email,
    });

    client
        .put(format!("{}/maquinas/{}", base_url, machine_id))
        .json(&payload)
        .send()
        .map(|res| res.status().is_success())
        .unwrap_or(false)
}

fn map_licensed_company(
    body: &Value,
    company_document: &str,
    settings: &LicenseSettings,
) -> LicensedCompany {
    LicensedCompany {
        idcliente: find_i64(body, &["IDCLIENTE", "idcliente"]).unwrap_or_default(),
        datacad: find_string(body, &["DATACAD", "datacad"]).unwrap_or_default(),
        cnpj: first_non_empty(vec![
            find_string(body, &["CNPJ", "cnpj"]).unwrap_or_default(),
            company_document.to_string(),
        ])
        .unwrap_or_default(),
        emp_inscestrg: find_string(body, &["EMP_INSCESTRG", "emp_inscestrg"]).unwrap_or_default(),
        emp_inscmunicipal: find_string(body, &["EMP_INSCMUNICIPAL", "emp_inscmunicipal"])
            .unwrap_or_default(),
        emp_nomefantasia: find_string(
            body,
            &[
                "EMP_NOMEFANTASIA",
                "emp_nomefantasia",
                "FANTASIA",
                "fantasia",
            ],
        )
        .unwrap_or_default(),
        razaosocial: find_string(body, &["RAZAOSOCIAL", "razaosocial", "RAZAO", "razao"])
            .unwrap_or_else(|| settings.company_name.clone()),
        emp_endereco: find_string(body, &["EMP_ENDERECO", "emp_endereco"]).unwrap_or_default(),
        emp_numero: find_string(body, &["EMP_NUMERO", "emp_numero"]).unwrap_or_default(),
        emp_bairro: find_string(body, &["EMP_BAIRRO", "emp_bairro"]).unwrap_or_default(),
        emp_cidade: find_string(body, &["EMP_CIDADE", "emp_cidade"]).unwrap_or_default(),
        emp_uf: find_string(body, &["EMP_UF", "emp_uf"]).unwrap_or_default(),
        emp_cep: find_string(body, &["EMP_CEP", "emp_cep"]).unwrap_or_default(),
        emp_complemento: find_string(body, &["EMP_COMPLEMENTO", "emp_complemento"])
            .unwrap_or_default(),
        telefone1: find_string(body, &["TELEFONE1", "telefone1"]).unwrap_or_default(),
        emp_email: find_string(body, &["EMP_EMAIL", "emp_email"])
            .unwrap_or_else(|| settings.company_email.clone()),
        emp_website: find_string(body, &["EMP_WEBSITE", "emp_website"]).unwrap_or_default(),
        emp_responsavel: find_string(body, &["EMP_RESPONSAVEL", "emp_responsavel"])
            .unwrap_or_default(),
        emp_cnae: find_string(body, &["EMP_CNAE", "emp_cnae"]).unwrap_or_default(),
        emp_serie: find_string(body, &["EMP_SERIE", "emp_serie"]).unwrap_or_default(),
        idrepresentante: find_i64(body, &["IDREPRESENTANTE", "idrepresentante"])
            .unwrap_or_default(),
        bloqueio_admin: find_bool(body, &["BLOQUEIO_ADMIN", "bloqueio_admin"]),
        bloqueado: find_bool(body, &["BLOQUEADO", "bloqueado"]),
        ativo: !find_string(body, &["ATIVO", "ativo"])
            .map(|v| matches!(v.as_str(), "N" | "0" | "false"))
            .unwrap_or(false),
        dia_venc_mensalidade: find_i64(body, &["dia_venc_mensalidade", "DIA_VENC_MENSALIDADE"])
            .unwrap_or_default(),
        forma_pagamento: find_string(body, &["forma_pagamento", "FORMA_PAGAMENTO"])
            .unwrap_or_default(),
        emp_obs: find_string(body, &["emp_obs", "EMP_OBS"]).unwrap_or_default(),
        n_maquinas: find_i64(body, &["n_maquinas", "N_MAQUINAS", "QTD_MAQ", "qtd_maq"])
            .unwrap_or_default(),
        data_val_lic: find_string(body, &["DATA_VAL_LIC", "data_val_lic"]).unwrap_or_default(),
        api_whatsapp: find_string(body, &["API_WHATSAPP", "api_whatsapp"]).unwrap_or_default(),
        token_whatsapp: find_string(body, &["TOKEN_WHATSAPP", "token_whatsapp"])
            .unwrap_or_default(),
        default_msg_whatsapp: find_string(body, &["DEFAULT_MSG_WHATSAPP", "default_msg_whatsapp"])
            .unwrap_or_default(),
    }
}

fn map_local_license(body: &Value, company: &LicensedCompany) -> LocalLicense {
    LocalLicense {
        id: find_i64(body, &["ID", "id", "IDLICENCA"]).unwrap_or_default(),
        empresa: find_string(body, &["Empresa", "EMPRESA", "empresa"])
            .unwrap_or_else(|| company.razaosocial.clone()),
        cnpj: find_string(body, &["CNPJ", "cnpj"]).unwrap_or_else(|| company.cnpj.clone()),
        fantasia: find_string(body, &["Fantasia", "fantasia", "EMP_NOMEFANTASIA"])
            .unwrap_or_else(|| company.emp_nomefantasia.clone()),
        serial: find_string(body, &["Serial", "serial"]).unwrap_or_default(),
        licencas: find_string(body, &["licencas", "LICENCAS"]).unwrap_or_default(),
        ativo: !find_string(body, &["Ativo", "ATIVO", "ativo"])
            .map(|v| matches!(v.as_str(), "N" | "0" | "false"))
            .unwrap_or(false),
        endereco: find_string(body, &["Endereco", "ENDERECO", "emp_endereco"]).unwrap_or_default(),
        bairro: find_string(body, &["Bairro", "BAIRRO", "emp_bairro"]).unwrap_or_default(),
        cidade: find_string(body, &["Cidade", "CIDADE", "emp_cidade"]).unwrap_or_default(),
        uf: find_string(body, &["UF", "uf", "emp_uf"]).unwrap_or_default(),
        cep: find_string(body, &["CEP", "cep", "emp_cep"]).unwrap_or_default(),
        numero: find_string(body, &["Numero", "NUMERO", "emp_numero"]).unwrap_or_default(),
        email: find_string(body, &["email", "EMAIL", "emp_email"]).unwrap_or_default(),
        complemento: find_string(body, &["Complemento", "COMPLEMENTO", "emp_complemento"])
            .unwrap_or_default(),
        dias: find_i64(body, &["dias", "DIAS"]).unwrap_or_default(),
        competencia: find_string(body, &["Competencia", "competencia", "DATA_VAL_LIC"])
            .unwrap_or_default(),
        bloqueio: find_bool(body, &["Bloqueio", "BLOQUEIO", "bloqueado", "BLOQUEADO"]),
        retaguarda: find_bool(body, &["Retaguarda", "RETAGUARDA"]),
        pdv: find_bool(body, &["PDV", "pdv"]),
        cte: find_bool(body, &["CTE", "cte"]),
        mdfe: find_bool(body, &["MDFE", "mdfe"]),
        nfe: find_bool(body, &["NFE", "nfe"]),
        frente: find_bool(body, &["Frente", "FRENTE"]),
        sat: find_bool(body, &["SAT", "sat"]),
        app: find_bool(body, &["APP", "app"]),
        boletos: find_bool(body, &["Boletos", "BOLETOS"]),
        mfe: find_bool(body, &["MFE", "mfe"]),
        commerce: find_bool(body, &["Commerce", "COMMERCE"]),
        serial_key: find_string(body, &["SerialKey", "SERIALKEY", "serial_key"])
            .unwrap_or_default(),
        terminal_ativo: find_bool(body, &["TerminalAtivo", "TERMINALATIVO"]),
        usadas: find_i64(body, &["Usadas", "USADAS", "usadas"]).unwrap_or_default(),
    }
}

fn map_devices(body: &Value, company_document: &str) -> Vec<LicensedDevice> {
    find_array(
        body,
        &["COMPUTADORES", "computadores", "maquinas", "MAQUINAS"],
    )
    .map(|items| {
        items
            .iter()
            .map(|item| LicensedDevice {
                idmaquina: find_i64(item, &["IDMAQUINA", "idmaquina"]).unwrap_or_default(),
                cnpj: find_string(item, &["CNPJ", "cnpj"])
                    .unwrap_or_else(|| company_document.into()),
                chave: find_string(item, &["CHAVE", "chave"]).unwrap_or_default(),
                nome: find_string(item, &["NOME", "nome"]).unwrap_or_default(),
                bloqueado: find_bool(item, &["BLOQUEADO", "bloqueado"]),
                modulos: find_string(item, &["modulos", "MODULOS"]).unwrap_or_default(),
                nome_compu: find_string(item, &["nome_compu", "NOME_COMPU", "nomemaq"])
                    .unwrap_or_default(),
                prog_acesso: find_string(item, &["Prog_acesso", "prog_acesso"]).unwrap_or_default(),
                cod_ace_remoto: find_string(item, &["Cod_ace_Remoto", "cod_ace_remoto"])
                    .unwrap_or_default(),
                versao_bd: find_string(item, &["versao_bd", "VERSAO_BD"]).unwrap_or_default(),
                versaoexe: find_string(item, &["versaoexe", "VERSAOEXE"]).unwrap_or_default(),
                sistema_operacional: find_string(
                    item,
                    &["sistema_operacional", "SISTEMA_OPERACIONAL"],
                )
                .unwrap_or_default(),
                memoria_ram: find_string(item, &["memoria_ram", "MEMORIA_RAM"]).unwrap_or_default(),
                tipo: find_string(item, &["tipo", "TIPO"]).unwrap_or_default(),
                observacao: find_string(item, &["observacao", "OBSERVACAO"]).unwrap_or_default(),
                tecnico_instalacao: find_string(
                    item,
                    &["tecnico_instalacao", "TECNICO_INSTALACAO"],
                )
                .unwrap_or_default(),
            })
            .collect::<Vec<LicensedDevice>>()
    })
    .unwrap_or_default()
}

fn is_expired(value: &str) -> bool {
    parse_date(value)
        .map(|date| date < Utc::now().date_naive())
        .unwrap_or(false)
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    ["%Y-%m-%d", "%d/%m/%Y", "%Y%m%d", "%d-%m-%Y"]
        .iter()
        .find_map(|fmt| NaiveDate::parse_from_str(trimmed, fmt).ok())
}

fn machine_fingerprint() -> String {
    let host = hostname::get()
        .ok()
        .and_then(|value| value.into_string().ok())
        .unwrap_or_else(|| "unknown-host".into());
    let user = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown-user".into());

    let base = format!(
        "{}|{}|{}|{}|{}",
        host,
        user,
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_NAME")
    );

    let mut hasher = Sha256::new();
    hasher.update(base.as_bytes());
    let result = hasher.finalize();

    result[..16]
        .iter()
        .map(|item| format!("{:02X}", item))
        .collect::<Vec<String>>()
        .join("")
}

fn only_digits(value: &str) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn first_non_empty(values: Vec<String>) -> Option<String> {
    values.into_iter().find(|value| !value.trim().is_empty())
}

fn find_string(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            keys.iter()
                .find_map(|key| map.get(*key))
                .and_then(|item| match item {
                    Value::String(text) => Some(text.clone()),
                    Value::Number(number) => Some(number.to_string()),
                    Value::Bool(flag) => Some(flag.to_string()),
                    _ => None,
                })
        }
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
        }
        _ => None,
    }
}

fn find_bool(value: &Value, keys: &[&str]) -> bool {
    find_string(value, keys)
        .map(|text| {
            matches!(
                text.to_lowercase().as_str(),
                "1" | "s" | "true" | "y" | "yes"
            )
        })
        .unwrap_or(false)
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
