use chrono::{Local, Utc};
use generic_license_tauri::{
    default_device_name, generate_device_key,
    models::{DeviceRecord, LicenseCheckInput, LicenseConfig, LicenseDecision, LicenseRecord},
    GenericLicenseService,
};
use tauri::AppHandle;

use crate::core::domain::license::{
    AppMeta, LicenseRuntimeStatus, LicenseSettings, LicenseSnapshot, LicensedCompany,
    LicensedDevice, LocalLicense,
};

const DEFAULT_LICENSE_BASE_URL: &str = "https://api.rest.wwsoftwares.com.br/api/v1";
const DEFAULT_APP_INSTANCE: &str = "integra-desktop";
const DEFAULT_APP_ID: &str = "br.com.wwsoftwares.integra.desktop";
const DEFAULT_PRODUCT_NAME: &str = "Integra Desktop";

#[tauri::command]
pub fn get_machine_fingerprint() -> Result<String, String> {
    Ok(machine_fingerprint(DEFAULT_APP_INSTANCE))
}

#[tauri::command]
pub fn get_app_meta() -> Result<AppMeta, String> {
    Ok(AppMeta {
        product_name: DEFAULT_PRODUCT_NAME.into(),
        version: env!("CARGO_PKG_VERSION").into(),
        build_hash: option_env!("GIT_HASH").unwrap_or("dev-local").into(),
        app_id: DEFAULT_APP_ID.into(),
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
    let next = normalize_license_settings(settings);
    crate::storage::license::save_license_settings(&app, &next).map_err(|e| e.to_string())?;
    Ok(next)
}

#[tauri::command]
pub async fn check_license_status(
    settings: LicenseSettings,
    app: AppHandle,
) -> Result<LicenseRuntimeStatus, String> {
    let next_settings = normalize_license_settings(settings);
    let company_document = only_digits(&next_settings.company_document);
    let station_name = resolve_station_name(&next_settings.station_name);
    let app_name = DEFAULT_PRODUCT_NAME.to_string();

    let service = GenericLicenseService::new(build_license_config(&next_settings));
    let input = LicenseCheckInput {
        company_document: company_document.clone(),
        company_name: optional_string(&next_settings.company_name),
        app_id: next_settings.app_instance.clone(),
        app_name,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        device_key: Some(next_settings.machine_key.clone()),
        device_name: Some(station_name.clone()),
        login_context: false,
    };

    let result = service.check(input).await;
    let (runtime, snapshot_devices) = match result {
        Ok(decision) => {
            let runtime = map_decision_to_runtime(
                &decision,
                &next_settings,
                &company_document,
                &station_name,
            );
            let snapshot_devices = decision
                .license
                .as_ref()
                .map(|license| {
                    license
                        .devices
                        .iter()
                        .map(|item| {
                            map_licensed_device(
                                item,
                                &company_document,
                                &station_name,
                                &next_settings,
                            )
                        })
                        .collect::<Vec<LicensedDevice>>()
                })
                .unwrap_or_default();
            (runtime, snapshot_devices)
        }
        Err(error) => {
            return Ok(from_cache_or_default(
                &app,
                &next_settings,
                false,
                &format!("Falha no componente de licenciamento: {error}"),
            ));
        }
    };

    let snapshot = LicenseSnapshot {
        last_sync_at: Utc::now().to_rfc3339(),
        local_license: runtime.local_license.clone(),
        licensed_company: runtime.licensed_company.clone(),
        licensed_devices: snapshot_devices,
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

fn build_license_config(settings: &LicenseSettings) -> LicenseConfig {
    LicenseConfig {
        base_url: resolve_base_url(&settings.service_url),
        api_token: std::env::var("LICENSE_API_TOKEN").ok(),
        status_endpoint: std::env::var("LICENSE_API_COMPANY_STATUS_ENDPOINT")
            .unwrap_or_else(|_| "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/cliente/{document}".to_string()),
        register_company_endpoint: std::env::var("LICENSE_API_REGISTER_COMPANY_ENDPOINT")
            .unwrap_or_else(|_| "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/clientes".to_string()),
        register_device_endpoint: std::env::var("LICENSE_API_REGISTER_DEVICE_ENDPOINT")
            .unwrap_or_else(|_| "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/maquinas".to_string()),
        update_device_endpoint: std::env::var("LICENSE_API_UPDATE_DEVICE_ENDPOINT")
            .unwrap_or_else(|_| "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/maquinas/IDMAQUINA/{id}".to_string()),
        offline_max_age_days: 15,
        warn_before_expiration_in_days: 5,
        auto_register_company_on_missing: settings.auto_register_machine,
        auto_register_device_on_missing: settings.auto_register_machine,
        auto_update_device_name: true,
        block_on_company_blocked: true,
        block_on_device_blocked: true,
        block_on_device_missing: true,
        block_on_expired: true,
        cache_namespace: settings.app_instance.clone(),
    }
}

fn resolve_base_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return DEFAULT_LICENSE_BASE_URL.to_string();
    }

    let normalized = trimmed.trim_end_matches('/');
    if normalized.ends_with("/api/v1") {
        normalized.to_string()
    } else {
        format!("{}/api/v1", normalized)
    }
}

fn resolve_station_name(input: &str) -> String {
    if input.trim().is_empty() {
        return default_device_name();
    }
    input.trim().to_string()
}

fn normalize_license_settings(settings: LicenseSettings) -> LicenseSettings {
    let mut next = settings;

    next.service_url = resolve_base_url(&next.service_url);
    if next.app_instance.trim().is_empty() {
        next.app_instance = DEFAULT_APP_INSTANCE.to_string();
    }
    if next.machine_key.trim().is_empty() {
        next.machine_key = machine_fingerprint(&next.app_instance);
    }

    next.company_document = only_digits(&next.company_document);
    next.company_name = next.company_name.trim().to_string();
    next.company_email = next.company_email.trim().to_string();
    next.station_name = next.station_name.trim().to_string();

    next
}

fn map_decision_to_runtime(
    decision: &LicenseDecision,
    settings: &LicenseSettings,
    company_document: &str,
    station_name: &str,
) -> LicenseRuntimeStatus {
    let license = decision.license.as_ref();
    let local_license = license.map(|item| map_local_license(item, settings, company_document));
    let licensed_company =
        license.map(|item| map_licensed_company(item, settings, company_document));
    let licensed_device = decision
        .device
        .as_ref()
        .map(|item| map_licensed_device(item, company_document, station_name, settings));

    let seats_total = license.and_then(|item| item.max_devices).unwrap_or(0);
    let seats_used = license.map(|item| item.devices.len() as u32).unwrap_or(0);
    let machine_registered = licensed_device.is_some();
    let machine_blocked = licensed_device
        .as_ref()
        .map(|item| item.bloqueado)
        .unwrap_or(false);
    let company_blocked = licensed_company
        .as_ref()
        .map(|item| item.bloqueado || item.bloqueio_admin)
        .unwrap_or(false);
    let company_inactive = licensed_company
        .as_ref()
        .map(|item| !item.ativo)
        .unwrap_or(false);
    let expiry = license
        .and_then(|item| item.expires_at.as_ref())
        .map(format_expiry);
    let expired = license
        .and_then(|item| item.expires_at.as_ref())
        .map(|item| item.with_timezone(&Utc) < Utc::now())
        .unwrap_or(false);
    let seats_exceeded = !machine_registered && seats_total > 0 && seats_used >= seats_total;

    let block_reason = infer_block_reason(
        decision,
        company_blocked,
        machine_blocked,
        company_inactive,
        expired,
        seats_exceeded,
        machine_registered,
    );

    LicenseRuntimeStatus {
        online: !decision.used_offline_cache,
        allowed: decision.allowed,
        blocked: !decision.allowed,
        machine_registered,
        machine_blocked,
        seats_total,
        seats_used,
        expiry,
        message: decision.message.clone(),
        block_reason,
        technical_message: build_technical_message(decision),
        company_name: licensed_company
            .as_ref()
            .map(|item| item.razaosocial.clone())
            .unwrap_or_else(|| settings.company_name.clone()),
        company_document: company_document.to_string(),
        machine_key: settings.machine_key.clone(),
        status_code: if decision.allowed { 1 } else { 0 },
        local_license,
        licensed_company,
        licensed_device,
    }
}

fn infer_block_reason(
    decision: &LicenseDecision,
    company_blocked: bool,
    machine_blocked: bool,
    company_inactive: bool,
    expired: bool,
    seats_exceeded: bool,
    machine_registered: bool,
) -> Option<String> {
    if decision.allowed {
        return None;
    }

    if company_blocked {
        Some("company_blocked".into())
    } else if machine_blocked {
        Some("machine_blocked".into())
    } else if expired {
        Some("license_expired".into())
    } else if seats_exceeded {
        Some("seats_exceeded".into())
    } else if company_inactive {
        Some("company_inactive".into())
    } else if !machine_registered {
        Some("device_not_registered".into())
    } else {
        Some("service_denied".into())
    }
}

fn build_technical_message(decision: &LicenseDecision) -> String {
    match (&decision.warning, decision.used_offline_cache) {
        (Some(warning), true) => format!("{} | cache offline do componente", warning),
        (Some(warning), false) => warning.clone(),
        (None, true) => format!("{} | cache offline do componente", decision.source),
        (None, false) => decision.source.clone(),
    }
}

fn map_local_license(
    license: &LicenseRecord,
    settings: &LicenseSettings,
    company_document: &str,
) -> LocalLicense {
    LocalLicense {
        empresa: license
            .company_name
            .clone()
            .unwrap_or_else(|| settings.company_name.clone()),
        cnpj: license
            .document
            .clone()
            .unwrap_or_else(|| company_document.to_string()),
        fantasia: license
            .company_name
            .clone()
            .unwrap_or_else(|| settings.company_name.clone()),
        licencas: settings.app_instance.clone(),
        ativo: license.active,
        email: settings.company_email.clone(),
        competencia: license
            .expires_at
            .as_ref()
            .map(format_expiry)
            .unwrap_or_default(),
        bloqueio: license.blocked,
        app: true,
        terminal_ativo: !license.blocked,
        usadas: license.devices.len() as i64,
        ..LocalLicense::default()
    }
}

fn map_licensed_company(
    license: &LicenseRecord,
    settings: &LicenseSettings,
    company_document: &str,
) -> LicensedCompany {
    LicensedCompany {
        cnpj: license
            .document
            .clone()
            .unwrap_or_else(|| company_document.to_string()),
        emp_nomefantasia: license
            .company_name
            .clone()
            .unwrap_or_else(|| settings.company_name.clone()),
        razaosocial: license
            .company_name
            .clone()
            .unwrap_or_else(|| settings.company_name.clone()),
        emp_email: settings.company_email.clone(),
        emp_serie: settings.app_instance.clone(),
        bloqueio_admin: false,
        bloqueado: license.blocked,
        ativo: license.active,
        n_maquinas: license.max_devices.unwrap_or(0) as i64,
        data_val_lic: license
            .expires_at
            .as_ref()
            .map(format_expiry)
            .unwrap_or_default(),
        emp_obs: DEFAULT_PRODUCT_NAME.to_string(),
        ..LicensedCompany::default()
    }
}

fn map_licensed_device(
    device: &DeviceRecord,
    company_document: &str,
    station_name: &str,
    settings: &LicenseSettings,
) -> LicensedDevice {
    LicensedDevice {
        idmaquina: device
            .id
            .as_deref()
            .and_then(|item| item.parse::<i64>().ok())
            .unwrap_or_default(),
        cnpj: company_document.to_string(),
        chave: device.device_key.clone().unwrap_or_default(),
        nome: settings.company_name.clone(),
        bloqueado: device.blocked,
        modulos: settings.app_instance.clone(),
        nome_compu: device
            .device_name
            .clone()
            .unwrap_or_else(|| station_name.to_string()),
        versaoexe: env!("CARGO_PKG_VERSION").to_string(),
        sistema_operacional: std::env::consts::OS.to_string(),
        tipo: settings.app_instance.clone(),
        observacao: DEFAULT_PRODUCT_NAME.to_string(),
        ..LicensedDevice::default()
    }
}

fn format_expiry(value: &chrono::DateTime<chrono::FixedOffset>) -> String {
    value
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn machine_fingerprint(app_instance: &str) -> String {
    generate_device_key(app_instance)
}

fn optional_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn only_digits(value: &str) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
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
