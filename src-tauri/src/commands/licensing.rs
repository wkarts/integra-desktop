use chrono::{Local, Utc};
use generic_license_tauri::{
    collect_device_metadata, default_device_name, generate_device_key,
    models::{
        ApplicationLicenseRecord, CompanyRecord, DeviceRecord, LicenseCheckInput, LicenseConfig,
        LicenseDecision, LicenseRecord,
    },
    registration::discover_registration_file,
    GenericLicenseService,
};
use tauri::AppHandle;

use crate::core::domain::license::{
    AppMeta, GenerateLocalLicenseRequest, GeneratedLocalLicense, LicenseRuntimeStatus,
    LicenseSettings, LicenseSnapshot, LicensedCompany, LicensedDevice, LocalLicense,
    LocalLicenseValidationResult, RegistrationDeviceInfo, StartupLicenseContext,
    ValidateLocalLicenseRequest,
};
use crate::core::local_license::{
    generate_local_license as generate_local_license_artifact,
    validate_local_license as validate_local_license_artifact,
};
use std::collections::HashMap;

const DEFAULT_LICENSE_BASE_URL: &str = "https://api.rest.wwsoftwares.com.br/api/v1";
const DEFAULT_APP_INSTANCE: &str = "integra-desktop";
const DEFAULT_APP_ID: &str = "br.com.wwsoftwares.integra.desktop";
const DEFAULT_PRODUCT_NAME: &str = "Integra Desktop";

#[tauri::command]
pub fn get_machine_fingerprint() -> Result<String, String> {
    Ok(generate_machine_fingerprint(DEFAULT_APP_INSTANCE))
}

#[tauri::command]
pub fn get_default_station_name() -> Result<String, String> {
    Ok(default_device_name())
}

#[tauri::command]
pub fn get_registration_device_info(
    settings: Option<LicenseSettings>,
) -> Result<RegistrationDeviceInfo, String> {
    let startup = parse_startup_licensing_context(std::env::args().collect());
    let base_settings = settings.unwrap_or_default();
    let normalized_settings =
        normalize_license_settings(apply_startup_overrides(base_settings, &startup));
    let config = build_license_config(&normalized_settings, None);
    let device = collect_device_metadata();
    let registration = discover_registration_file(&config).map_err(|e| e.to_string())?;
    let input = build_license_input(&normalized_settings, &device, None);
    let device_key = generate_device_key(&input);

    let serial_number = optional_string(&device.serial_number)
        .or_else(|| optional_string(&device.bios_serial))
        .or_else(|| optional_string(&device.motherboard_serial))
        .or_else(|| optional_string(&device.machine_guid))
        .unwrap_or_default();

    Ok(RegistrationDeviceInfo {
        station_name: device.station_name.clone(),
        device_display_name: full_device_name(
            Some(device.station_name.as_str()),
            Some(device.computer_name.as_str()),
            Some(device.hostname.as_str()),
        ),
        hostname: device.hostname,
        computer_name: device.computer_name,
        serial_number,
        machine_guid: device.machine_guid,
        bios_serial: device.bios_serial,
        motherboard_serial: device.motherboard_serial,
        logged_user: device.logged_user,
        os_name: device.os_name,
        os_version: device.os_version,
        os_arch: device.os_arch,
        domain_name: device.domain_name,
        install_mode: device.install_mode,
        mac_addresses: device.mac_addresses,
        device_key,
        registration_file_found: registration.file_path.is_some(),
        registration_file_path: registration.file_path,
        registration_file_verified: registration.verified,
    })
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
pub fn get_startup_licensing_context() -> Result<StartupLicenseContext, String> {
    Ok(parse_startup_licensing_context(std::env::args().collect()))
}

#[tauri::command]
pub fn generate_local_license(
    request: GenerateLocalLicenseRequest,
) -> Result<GeneratedLocalLicense, String> {
    generate_local_license_artifact(request)
}

#[tauri::command]
pub fn validate_local_license(
    request: ValidateLocalLicenseRequest,
) -> Result<LocalLicenseValidationResult, String> {
    validate_local_license_artifact(request)
}

#[tauri::command]
pub fn load_license_settings(app: AppHandle) -> Result<Option<LicenseSettings>, String> {
    let startup = parse_startup_licensing_context(std::env::args().collect());
    let saved = crate::storage::license::load_license_settings(&app).map_err(|e| e.to_string())?;

    if let Some(settings) = saved {
        return Ok(Some(normalize_license_settings(apply_startup_overrides(
            settings, &startup,
        ))));
    }

    if startup_has_runtime_overrides(&startup) {
        return Ok(Some(normalize_license_settings(apply_startup_overrides(
            LicenseSettings::default(),
            &startup,
        ))));
    }

    Ok(None)
}

#[tauri::command]
pub fn save_license_settings(
    settings: LicenseSettings,
    app: AppHandle,
) -> Result<LicenseSettings, String> {
    let startup = parse_startup_licensing_context(std::env::args().collect());
    let next = normalize_license_settings(apply_startup_overrides(settings, &startup));
    crate::storage::license::save_license_settings(&app, &next).map_err(|e| e.to_string())?;
    Ok(next)
}

#[tauri::command]
pub async fn check_license_status(
    settings: LicenseSettings,
    app: AppHandle,
) -> Result<LicenseRuntimeStatus, String> {
    let raw_args: Vec<String> = std::env::args().collect();
    let startup = parse_startup_licensing_context(raw_args.clone());
    let parsed_args = parse_startup_args(&raw_args);
    let next_settings = normalize_license_settings(apply_startup_overrides(settings, &startup));

    if next_settings.licensing_disabled {
        return Ok(build_licensing_disabled_runtime(&next_settings));
    }

    if startup.local_license_enabled {
        let validation = validate_local_license_artifact(ValidateLocalLicenseRequest {
            file_path: startup.local_license_file_path.clone().or_else(|| {
                crate::core::local_license::default_local_license_path(&next_settings.app_instance)
            }),
            content_b64: None,
            company_document: optional_string(&next_settings.company_document),
            machine_key: optional_string(&next_settings.machine_key),
            developer_token: parse_string_arg(&parsed_args, &["local-license-token"]),
            developer_secret: parse_string_arg(&parsed_args, &["local-license-secret"]),
            enforce_machine_match: false,
        });

        if let Ok(local_validation) = validation {
            let device = collect_device_metadata();
            let runtime = if local_validation.valid {
                map_local_license_validation_to_runtime(&local_validation, &next_settings, &device)
            } else {
                map_local_license_validation_failure_to_runtime(&local_validation, &next_settings)
            };

            let persisted_settings =
                hydrate_settings_from_local_validation(next_settings.clone(), &local_validation);
            persist_license_runtime_snapshot(&app, &runtime, &persisted_settings)?;
            return Ok(runtime);
        }
    }

    let company_document = only_digits(&next_settings.company_document);
    let station_name = resolve_station_name(&next_settings.station_name);

    let device = collect_device_metadata();
    let input = build_license_input(&next_settings, &device, None);

    let service = GenericLicenseService::new(build_license_config(&next_settings, None));
    let result = service.check(input).await;
    let (runtime, snapshot_devices, persisted_settings) = match result {
        Ok(decision) => {
            let runtime = map_decision_to_runtime(
                &decision,
                &next_settings,
                &company_document,
                &station_name,
            );
            let snapshot_devices =
                build_snapshot_devices(&decision, &company_document, &station_name, &next_settings);
            let persisted_settings =
                hydrate_settings_from_decision(next_settings.clone(), &decision, &station_name);
            (runtime, snapshot_devices, persisted_settings)
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
    crate::storage::license::save_license_settings(&app, &persisted_settings)
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

fn persist_license_runtime_snapshot(
    app: &AppHandle,
    runtime: &LicenseRuntimeStatus,
    settings: &LicenseSettings,
) -> Result<(), String> {
    let snapshot = LicenseSnapshot {
        last_sync_at: Utc::now().to_rfc3339(),
        local_license: runtime.local_license.clone(),
        licensed_company: runtime.licensed_company.clone(),
        licensed_devices: runtime
            .licensed_device
            .clone()
            .map(|item| vec![item])
            .unwrap_or_default(),
        runtime_status: runtime.clone(),
    };

    crate::storage::license::save_license_snapshot(app, &snapshot).map_err(|e| e.to_string())?;
    crate::storage::license::save_license_settings(app, settings).map_err(|e| e.to_string())?;
    Ok(())
}

fn build_licensing_disabled_runtime(settings: &LicenseSettings) -> LicenseRuntimeStatus {
    LicenseRuntimeStatus {
        online: false,
        allowed: true,
        blocked: false,
        machine_registered: true,
        machine_blocked: false,
        seats_total: 0,
        seats_used: 0,
        expiry: None,
        message: "licenciamento desabilitado na configuração da aplicação".to_string(),
        block_reason: None,
        technical_message: "source=settings | mode=licensing-disabled".to_string(),
        company_name: settings.company_name.clone(),
        company_document: settings.company_document.clone(),
        machine_key: settings.machine_key.clone(),
        status_code: 1,
        local_license: None,
        licensed_company: None,
        licensed_device: None,
    }
}

fn build_license_config(
    settings: &LicenseSettings,
    _startup: Option<&StartupLicenseContext>,
) -> LicenseConfig {
    let auto_register_company = settings.auto_register_machine;
    let auto_register_device = settings.auto_register_machine;

    LicenseConfig {
        base_url: resolve_base_url(&settings.service_url),
        api_token: std::env::var("LICENSE_API_TOKEN").ok(),
        resolve_activation_endpoint: std::env::var("LICENSE_API_RESOLVE_ENDPOINT")
            .unwrap_or_else(|_| "/licensing/activation/resolve".to_string()),
        status_endpoint: std::env::var("LICENSE_API_COMPANY_STATUS_ENDPOINT").unwrap_or_else(
            |_| {
                "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/cliente/{document}".to_string()
            },
        ),
        register_company_endpoint: std::env::var("LICENSE_API_REGISTER_COMPANY_ENDPOINT")
            .unwrap_or_else(|_| {
                "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/clientes"
                    .to_string()
            }),
        register_device_endpoint: std::env::var("LICENSE_API_REGISTER_DEVICE_ENDPOINT")
            .unwrap_or_else(|_| {
                "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/maquinas"
                    .to_string()
            }),
        update_device_endpoint: std::env::var("LICENSE_API_UPDATE_DEVICE_ENDPOINT")
            .unwrap_or_else(|_| {
                "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/maquinas/IDMAQUINA/{id}".to_string()
            }),
        offline_max_age_days: 15,
        warn_before_expiration_in_days: 5,
        auto_register_company_on_missing: auto_register_company,
        auto_register_device_on_missing: auto_register_device,
        auto_update_device_name: true,
        block_on_company_blocked: true,
        block_on_device_blocked: true,
        block_on_device_missing: true,
        block_on_expired: true,
        cache_namespace: settings.app_instance.clone(),
        prefer_resolve_activation: true,
        allow_legacy_fallback: true,
        enable_registration_file_lookup: true,
        registration_file_names: registration_file_names(),
        registration_file_extra_paths: registration_file_extra_paths(),
        registration_public_key_base64: std::env::var("LICENSE_REGISTRATION_PUBLIC_KEY_BASE64")
            .ok(),
    }
}

fn registration_file_names() -> Vec<String> {
    if let Ok(value) = std::env::var("LICENSE_REGISTRATION_FILE_NAMES") {
        let parsed = value
            .split(';')
            .flat_map(|chunk| chunk.split(','))
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .map(|item| item.to_string())
            .collect::<Vec<String>>();
        if !parsed.is_empty() {
            return parsed;
        }
    }

    vec![
        "wwreg.json".to_string(),
        "wwreg.lic".to_string(),
        "integra-desktop.lic".to_string(),
        "license.wwreg".to_string(),
    ]
}

fn registration_file_extra_paths() -> Vec<String> {
    if let Ok(value) = std::env::var("LICENSE_REGISTRATION_FILE_EXTRA_PATHS") {
        return value
            .split(';')
            .flat_map(|chunk| chunk.split(','))
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .map(|item| item.to_string())
            .collect::<Vec<String>>();
    }

    Vec::new()
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
        format!("{normalized}/api/v1")
    }
}

fn resolve_station_name(input: &str) -> String {
    if input.trim().is_empty() {
        return default_device_name();
    }
    input.trim().to_string()
}

fn parse_startup_licensing_context(args: Vec<String>) -> StartupLicenseContext {
    let mut context = StartupLicenseContext {
        args: args.iter().skip(1).cloned().collect(),
        ..StartupLicenseContext::default()
    };
    let parsed = parse_startup_args(&args);

    context.auto_register_enabled = has_any_flag(
        &parsed,
        &[
            "auto-register",
            "auto-register-company",
            "auto-register-device",
        ],
    );
    context.auto_register_company = has_any_flag(&parsed, &["auto-register-company"]);
    context.auto_register_device = has_any_flag(&parsed, &["auto-register-device"]);
    context.requested_licenses = parse_u32_arg(&parsed, &["licenses", "lic"]);
    context.company_document = parse_string_arg(&parsed, &["company-document", "document", "cnpj"]);
    context.company_name = parse_string_arg(&parsed, &["company-name", "empresa"]);
    context.company_email = parse_string_arg(&parsed, &["company-email", "email"]);
    context.station_name = parse_string_arg(&parsed, &["station-name", "station"]);
    context.device_name = parse_string_arg(&parsed, &["device-name"]);
    context.device_identifier =
        parse_string_arg(&parsed, &["device", "device-id", "device-identifier"]);
    context.validation_mode = parse_string_arg(&parsed, &["validation-mode"]);
    context.interface_mode = parse_string_arg(&parsed, &["ui-mode"]);
    context.local_license_enabled = has_any_flag(&parsed, &["local-license"]);
    context.local_license_generate = has_any_flag(&parsed, &["local-license-generate"]);
    context.local_license_file_path = parse_string_arg(&parsed, &["local-license-file"]);
    context.local_license_token_present = has_non_empty_arg(&parsed, &["local-license-token"]);
    context.developer_secret_present = has_non_empty_arg(&parsed, &["local-license-secret"]);
    context.local_license_account = parse_string_arg(&parsed, &["local-license-account"]);
    context.local_license_issuer = parse_string_arg(&parsed, &["local-license-issuer"]);
    context.no_ui = has_any_flag(&parsed, &["silent", "headless", "no-ui"]);
    context.licensing_disabled = has_any_flag(
        &parsed,
        &["disable-licensing", "licensing-disabled", "no-license"],
    );

    if context.no_ui && context.interface_mode.is_none() {
        context.interface_mode = Some("silent".to_string());
    }

    context
}

fn startup_has_runtime_overrides(startup: &StartupLicenseContext) -> bool {
    startup.licensing_disabled
        || startup.auto_register_enabled
        || startup.auto_register_company
        || startup.auto_register_device
        || startup.requested_licenses.is_some()
        || startup.company_name.is_some()
        || startup.company_document.is_some()
        || startup.company_email.is_some()
        || startup.station_name.is_some()
        || startup.device_name.is_some()
        || startup.device_identifier.is_some()
        || startup.validation_mode.is_some()
        || startup.interface_mode.is_some()
        || startup.local_license_enabled
        || startup.local_license_generate
        || startup.local_license_file_path.is_some()
        || startup.local_license_token_present
        || startup.developer_secret_present
        || startup.local_license_account.is_some()
        || startup.local_license_issuer.is_some()
        || startup.no_ui
}

fn apply_startup_overrides(
    mut settings: LicenseSettings,
    startup: &StartupLicenseContext,
) -> LicenseSettings {
    if let Some(company_name) = &startup.company_name {
        settings.company_name = company_name.clone();
    }
    if let Some(company_document) = &startup.company_document {
        settings.company_document = company_document.clone();
    }
    if let Some(company_email) = &startup.company_email {
        settings.company_email = company_email.clone();
    }
    if let Some(station_name) = startup
        .station_name
        .as_ref()
        .or(startup.device_name.as_ref())
    {
        settings.station_name = station_name.clone();
    }
    if let Some(requested_licenses) = startup.requested_licenses {
        settings.auto_register_requested_licenses = Some(requested_licenses);
    }
    if let Some(validation_mode) = &startup.validation_mode {
        settings.auto_register_validation_mode = validation_mode.clone();
    }
    if let Some(interface_mode) = &startup.interface_mode {
        settings.auto_register_interface_mode = interface_mode.clone();
    }
    if let Some(device_identifier) = &startup.device_identifier {
        settings.auto_register_device_identifier = device_identifier.clone();
    }
    if startup.auto_register_enabled
        || startup.auto_register_company
        || startup.auto_register_device
    {
        settings.auto_register_machine = true;
    }
    if startup.licensing_disabled {
        settings.licensing_disabled = true;
    }

    settings
}

fn parse_startup_args(args: &[String]) -> HashMap<String, Option<String>> {
    let mut parsed: HashMap<String, Option<String>> = HashMap::new();
    let mut index = 1usize;

    while index < args.len() {
        let current = args[index].trim();
        let normalized_flag = if current.starts_with("--") {
            Some(&current[2..])
        } else if current.starts_with('-') || current.starts_with('/') {
            Some(&current[1..])
        } else {
            None
        };

        let Some(raw) = normalized_flag else {
            index += 1;
            continue;
        };
        if raw.is_empty() {
            index += 1;
            continue;
        }

        if let Some((key, value)) = raw.split_once('=') {
            parsed.insert(key.trim().to_ascii_lowercase(), optional_string(value));
            index += 1;
            continue;
        }

        let key = raw.trim().to_ascii_lowercase();
        let mut value: Option<String> = None;
        if index + 1 < args.len() {
            let next = args[index + 1].trim();
            if !next.starts_with("--") {
                value = optional_string(next);
                index += 1;
            }
        }

        parsed.insert(key, value);
        index += 1;
    }

    parsed
}

fn has_any_flag(parsed: &HashMap<String, Option<String>>, keys: &[&str]) -> bool {
    keys.iter().any(|key| parsed.contains_key(*key))
}

fn parse_string_arg(parsed: &HashMap<String, Option<String>>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(value) = parsed.get(*key).and_then(|v| v.clone()) {
            return optional_string(&value);
        }
    }
    None
}

fn parse_u32_arg(parsed: &HashMap<String, Option<String>>, keys: &[&str]) -> Option<u32> {
    parse_string_arg(parsed, keys).and_then(|value| value.parse::<u32>().ok())
}

fn has_non_empty_arg(parsed: &HashMap<String, Option<String>>, keys: &[&str]) -> bool {
    parse_string_arg(parsed, keys).is_some()
}

fn normalize_license_settings(settings: LicenseSettings) -> LicenseSettings {
    let mut next = settings;

    next.service_url = resolve_base_url(&next.service_url);
    if next.app_instance.trim().is_empty() {
        next.app_instance = DEFAULT_APP_INSTANCE.to_string();
    }
    if next.machine_key.trim().is_empty() {
        next.machine_key = generate_machine_fingerprint(&next.app_instance);
    }

    next.company_document = only_digits(&next.company_document);
    next.company_name = next.company_name.trim().to_string();
    next.company_email = next.company_email.trim().to_string();
    next.station_name = next.station_name.trim().to_string();
    next.auto_register_validation_mode = if next.auto_register_validation_mode.trim().is_empty() {
        "standard".to_string()
    } else {
        next.auto_register_validation_mode.trim().to_string()
    };
    next.auto_register_interface_mode = if next.auto_register_interface_mode.trim().is_empty() {
        "interactive".to_string()
    } else {
        next.auto_register_interface_mode.trim().to_string()
    };
    next.auto_register_device_identifier = next.auto_register_device_identifier.trim().to_string();

    next
}

fn build_license_input(
    settings: &LicenseSettings,
    device: &generic_license_tauri::device::DeviceCollectedInfo,
    _startup: Option<&StartupLicenseContext>,
) -> LicenseCheckInput {
    let requested_licenses = settings.auto_register_requested_licenses;
    let validation_mode = optional_string(&settings.auto_register_validation_mode);
    let interface_mode = optional_string(&settings.auto_register_interface_mode);
    let device_identifier = optional_string(&settings.auto_register_device_identifier);

    let mut input = LicenseCheckInput {
        company_document: only_digits(&settings.company_document),
        company_name: optional_string(&settings.company_name),
        company_email: optional_string(&settings.company_email),
        company_legal_name: optional_string(&settings.company_name),
        app_id: DEFAULT_APP_ID.to_string(),
        app_name: DEFAULT_PRODUCT_NAME.to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        app_slug: Some(settings.app_instance.clone()),
        device_key: optional_string(&settings.machine_key),
        device_name: Some(full_device_name(
            Some(device.station_name.as_str()),
            Some(device.computer_name.as_str()),
            Some(device.hostname.as_str()),
        )),
        station_name: Some(resolve_station_name(&settings.station_name)),
        hostname: optional_string(&device.hostname),
        computer_name: optional_string(&device.computer_name),
        serial_number: optional_string(&device.serial_number),
        machine_guid: optional_string(&device.machine_guid),
        bios_serial: optional_string(&device.bios_serial),
        motherboard_serial: optional_string(&device.motherboard_serial),
        logged_user: optional_string(&device.logged_user),
        os_name: optional_string(&device.os_name),
        os_version: optional_string(&device.os_version),
        os_arch: optional_string(&device.os_arch),
        domain_name: optional_string(&device.domain_name),
        mac_addresses: device.mac_addresses.clone(),
        install_mode: optional_string(&device.install_mode),
        registration_file_content_b64: None,
        registration_file_path: None,
        registration_file_verified: None,
        allow_company_auto_create: Some(settings.auto_register_machine),
        allow_device_auto_create: Some(settings.auto_register_machine),
        allow_device_auto_update: Some(true),
        requested_licenses,
        device_identifier: device_identifier.clone(),
        validation_mode: validation_mode.clone(),
        interface_mode: interface_mode.clone(),
        local_license_mode: None,
        metadata: std::collections::BTreeMap::from([
            (
                "app_product_name".to_string(),
                DEFAULT_PRODUCT_NAME.to_string(),
            ),
            ("app_instance".to_string(), settings.app_instance.clone()),
            (
                "device_display_name".to_string(),
                full_device_name(
                    Some(device.station_name.as_str()),
                    Some(device.computer_name.as_str()),
                    Some(device.hostname.as_str()),
                ),
            ),
            (
                "validation_mode".to_string(),
                validation_mode
                    .clone()
                    .unwrap_or_else(|| "standard".to_string()),
            ),
            (
                "interface_mode".to_string(),
                interface_mode
                    .clone()
                    .unwrap_or_else(|| "interactive".to_string()),
            ),
            (
                "requested_licenses".to_string(),
                requested_licenses
                    .map(|item| item.to_string())
                    .unwrap_or_default(),
            ),
            (
                "device_identifier".to_string(),
                device_identifier.clone().unwrap_or_default(),
            ),
            ("startup_mode".to_string(), "disabled".to_string()),
        ]),
        login_context: false,
    };

    if input.device_key.as_deref().unwrap_or("").is_empty() {
        input.device_key = Some(generate_device_key(&input));
    }

    input
}

#[allow(dead_code)]
fn map_local_license_validation_to_runtime(
    validation: &LocalLicenseValidationResult,
    settings: &LicenseSettings,
    device: &generic_license_tauri::device::DeviceCollectedInfo,
) -> LicenseRuntimeStatus {
    let payload = validation.payload.as_ref();
    let company_name = payload
        .map(|item| item.company_name.clone())
        .filter(|item| !item.trim().is_empty())
        .unwrap_or_else(|| settings.company_name.clone());
    let company_document = payload
        .map(|item| item.company_document.clone())
        .filter(|item| !item.trim().is_empty())
        .unwrap_or_else(|| settings.company_document.clone());
    let machine_key = payload
        .map(|item| item.machine_key.clone())
        .filter(|item| !item.trim().is_empty())
        .unwrap_or_else(|| settings.machine_key.clone());
    let seats_total = payload
        .and_then(|item| item.requested_licenses)
        .unwrap_or(1);

    let local_license = LocalLicense {
        empresa: company_name.clone(),
        cnpj: company_document.clone(),
        serial: payload
            .map(|item| item.serial_number.clone())
            .unwrap_or_default(),
        licencas: seats_total.to_string(),
        ativo: true,
        app: true,
        terminal_ativo: true,
        serial_key: machine_key.clone(),
        usadas: 1,
        ..LocalLicense::default()
    };

    let licensed_company = LicensedCompany {
        cnpj: company_document.clone(),
        emp_nomefantasia: company_name.clone(),
        razaosocial: company_name.clone(),
        emp_email: payload
            .map(|item| item.company_email.clone())
            .unwrap_or_default(),
        emp_serie: settings.app_instance.clone(),
        ativo: true,
        bloqueado: false,
        n_maquinas: seats_total as i64,
        data_val_lic: payload
            .and_then(|item| item.expires_at.clone())
            .unwrap_or_default(),
        emp_obs: "licença local validada".to_string(),
        ..LicensedCompany::default()
    };

    let licensed_device = LicensedDevice {
        cnpj: company_document.clone(),
        chave: machine_key.clone(),
        nome: company_name.clone(),
        bloqueado: false,
        modulos: settings.app_instance.clone(),
        nome_compu: device.computer_name.clone(),
        prog_acesso: device.logged_user.clone(),
        cod_ace_remoto: device.domain_name.clone(),
        versaoexe: env!("CARGO_PKG_VERSION").to_string(),
        sistema_operacional: full_os_name(
            Some(device.os_name.as_str()),
            Some(device.os_version.as_str()),
            Some(device.os_arch.as_str()),
        ),
        tipo: device.install_mode.clone(),
        observacao: "licença local".to_string(),
        tecnico_instalacao: device.logged_user.clone(),
        serial_number: payload
            .map(|item| item.serial_number.clone())
            .unwrap_or_else(|| device.serial_number.clone()),
        hostname: device.hostname.clone(),
        station_name: payload
            .map(|item| item.station_name.clone())
            .unwrap_or_else(|| device.station_name.clone()),
        machine_guid: device.machine_guid.clone(),
        bios_serial: device.bios_serial.clone(),
        motherboard_serial: device.motherboard_serial.clone(),
        full_device_name: full_device_name(
            Some(device.station_name.as_str()),
            Some(device.computer_name.as_str()),
            Some(device.hostname.as_str()),
        ),
        ..LicensedDevice::default()
    };

    LicenseRuntimeStatus {
        online: false,
        allowed: true,
        blocked: false,
        machine_registered: true,
        machine_blocked: false,
        seats_total,
        seats_used: 1,
        expiry: payload.and_then(|item| item.expires_at.clone()),
        message: validation.message.clone(),
        block_reason: None,
        technical_message: format!(
            "source=local-license | reason={} | file={} | otpauth={}",
            validation.reason_code,
            validation.file_path.clone().unwrap_or_default(),
            validation.otpauth_uri.clone().unwrap_or_default()
        ),
        company_name,
        company_document,
        machine_key,
        status_code: 1,
        local_license: Some(local_license),
        licensed_company: Some(licensed_company),
        licensed_device: Some(licensed_device),
    }
}

#[allow(dead_code)]
fn map_local_license_validation_failure_to_runtime(
    validation: &LocalLicenseValidationResult,
    settings: &LicenseSettings,
) -> LicenseRuntimeStatus {
    LicenseRuntimeStatus {
        online: false,
        allowed: false,
        blocked: true,
        machine_registered: false,
        machine_blocked: false,
        seats_total: 0,
        seats_used: 0,
        expiry: None,
        message: validation.message.clone(),
        block_reason: Some(validation.reason_code.to_lowercase()),
        technical_message: format!(
            "source=local-license | reason={} | file={}",
            validation.reason_code,
            validation.file_path.clone().unwrap_or_default()
        ),
        company_name: settings.company_name.clone(),
        company_document: settings.company_document.clone(),
        machine_key: settings.machine_key.clone(),
        status_code: 0,
        local_license: None,
        licensed_company: None,
        licensed_device: None,
    }
}

fn build_snapshot_devices(
    decision: &LicenseDecision,
    company_document: &str,
    station_name: &str,
    settings: &LicenseSettings,
) -> Vec<LicensedDevice> {
    if let Some(license) = &decision.license {
        if !license.devices.is_empty() {
            return license
                .devices
                .iter()
                .map(|item| map_licensed_device(item, company_document, station_name, settings))
                .collect::<Vec<LicensedDevice>>();
        }
    }

    decision
        .device
        .as_ref()
        .map(|item| {
            vec![map_licensed_device(
                item,
                company_document,
                station_name,
                settings,
            )]
        })
        .unwrap_or_default()
}

fn hydrate_settings_from_decision(
    mut settings: LicenseSettings,
    decision: &LicenseDecision,
    station_name: &str,
) -> LicenseSettings {
    if settings.company_document.trim().is_empty() {
        if let Some(company) = &decision.company {
            if let Some(document) = &company.document {
                settings.company_document = only_digits(document);
            }
        } else if let Some(license) = &decision.license {
            if let Some(document) = &license.document {
                settings.company_document = only_digits(document);
            }
        }
    }

    if settings.company_name.trim().is_empty() {
        if let Some(company) = &decision.company {
            if let Some(name) = company.legal_name.clone().or_else(|| {
                decision
                    .license
                    .as_ref()
                    .and_then(|item| item.company_name.clone())
            }) {
                settings.company_name = name;
            }
        } else if let Some(license) = &decision.license {
            if let Some(name) = &license.company_name {
                settings.company_name = name.clone();
            }
        }
    }

    if settings.company_email.trim().is_empty() {
        if let Some(company) = &decision.company {
            if let Some(email) = &company.email {
                settings.company_email = email.clone();
            }
        } else if let Some(license) = &decision.license {
            if let Some(email) = &license.company_email {
                settings.company_email = email.clone();
            }
        }
    }

    if settings.station_name.trim().is_empty() {
        settings.station_name = station_name.to_string();
    }

    settings.company_document = only_digits(&settings.company_document);
    settings
}

fn hydrate_settings_from_local_validation(
    mut settings: LicenseSettings,
    validation: &LocalLicenseValidationResult,
) -> LicenseSettings {
    if let Some(payload) = validation.payload.as_ref() {
        if settings.company_name.trim().is_empty() {
            settings.company_name = payload.company_name.clone();
        }
        if settings.company_document.trim().is_empty() {
            settings.company_document = only_digits(&payload.company_document);
        }
        if settings.company_email.trim().is_empty() {
            settings.company_email = payload.company_email.clone();
        }
        if settings.station_name.trim().is_empty() {
            settings.station_name = payload.station_name.clone();
        }
        if settings.machine_key.trim().is_empty() {
            settings.machine_key = payload.machine_key.clone();
        }
    }

    settings
}

fn map_decision_to_runtime(
    decision: &LicenseDecision,
    settings: &LicenseSettings,
    company_document: &str,
    station_name: &str,
) -> LicenseRuntimeStatus {
    let local_license = decision
        .license
        .as_ref()
        .map(|item| map_local_license(item, settings, company_document));
    let licensed_company = map_company_to_runtime(decision, settings, company_document);
    let licensed_device = decision
        .device
        .as_ref()
        .map(|item| map_licensed_device(item, company_document, station_name, settings));

    let seats_total = decision
        .application_license
        .as_ref()
        .and_then(|item| item.max_devices)
        .or_else(|| decision.license.as_ref().and_then(|item| item.max_devices))
        .unwrap_or(0);
    let seats_used = decision
        .application_license
        .as_ref()
        .and_then(|item| item.devices_in_use)
        .or_else(|| {
            decision
                .license
                .as_ref()
                .and_then(|item| item.devices_in_use)
        })
        .unwrap_or_else(|| {
            decision
                .license
                .as_ref()
                .map(|item| item.devices.len() as u32)
                .unwrap_or(0)
        });
    let machine_registered = decision
        .device
        .as_ref()
        .map(|item| item.bound || item.reused_existing || item.id.is_some())
        .unwrap_or(false);
    let machine_blocked = decision
        .device
        .as_ref()
        .map(|item| item.blocked)
        .unwrap_or(false);
    let company_blocked = licensed_company
        .as_ref()
        .map(|item| item.bloqueado || item.bloqueio_admin)
        .unwrap_or(false);
    let company_inactive = licensed_company
        .as_ref()
        .map(|item| !item.ativo)
        .unwrap_or(false);
    let expiry = decision
        .application_license
        .as_ref()
        .and_then(|item| item.expires_at.as_ref())
        .or_else(|| {
            decision
                .license
                .as_ref()
                .and_then(|item| item.expires_at.as_ref())
        })
        .map(format_expiry);
    let expired = decision
        .application_license
        .as_ref()
        .and_then(|item| item.expires_at.as_ref())
        .or_else(|| {
            decision
                .license
                .as_ref()
                .and_then(|item| item.expires_at.as_ref())
        })
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
        company_document: licensed_company
            .as_ref()
            .map(|item| item.cnpj.clone())
            .unwrap_or_else(|| company_document.to_string()),
        machine_key: settings.machine_key.clone(),
        status_code: if decision.allowed { 1 } else { 0 },
        local_license,
        licensed_company,
        licensed_device,
    }
}

fn map_company_to_runtime(
    decision: &LicenseDecision,
    settings: &LicenseSettings,
    company_document: &str,
) -> Option<LicensedCompany> {
    if let Some(company) = &decision.company {
        return Some(map_company_record(
            company,
            decision.application_license.as_ref(),
            decision.license.as_ref(),
            settings,
            company_document,
        ));
    }

    decision
        .license
        .as_ref()
        .map(|item| map_licensed_company(item, settings, company_document))
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

    if let Some(reason_code) = &decision.reason_code {
        return Some(reason_code.to_lowercase());
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
    let mut parts = Vec::new();

    if let Some(step) = &decision.step {
        parts.push(format!("step={step}"));
    }
    if let Some(reason_code) = &decision.reason_code {
        parts.push(format!("reason={reason_code}"));
    }
    if let Some(warning) = &decision.warning {
        parts.push(warning.clone());
    }
    if decision.used_offline_cache {
        parts.push("cache offline do componente".to_string());
    }
    if !decision.source.trim().is_empty() {
        parts.push(format!("source={}", decision.source));
    }
    if !decision.diagnostics.is_empty() {
        let diagnostics = decision
            .diagnostics
            .iter()
            .map(|item| format!("{}:{}:{}", item.step, item.code, item.message))
            .collect::<Vec<String>>()
            .join(" | ");
        parts.push(diagnostics);
    }

    if parts.is_empty() {
        decision.source.clone()
    } else {
        parts.join(" | ")
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
        email: license
            .company_email
            .clone()
            .unwrap_or_else(|| settings.company_email.clone()),
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

fn map_company_record(
    company: &CompanyRecord,
    app_license: Option<&ApplicationLicenseRecord>,
    license: Option<&LicenseRecord>,
    settings: &LicenseSettings,
    company_document: &str,
) -> LicensedCompany {
    LicensedCompany {
        idcliente: company
            .id
            .as_deref()
            .and_then(|item| item.parse::<i64>().ok())
            .unwrap_or_default(),
        cnpj: company
            .document
            .clone()
            .or_else(|| license.and_then(|item| item.document.clone()))
            .unwrap_or_else(|| company_document.to_string()),
        emp_nomefantasia: company
            .legal_name
            .clone()
            .or_else(|| license.and_then(|item| item.company_name.clone()))
            .unwrap_or_else(|| settings.company_name.clone()),
        razaosocial: company
            .legal_name
            .clone()
            .or_else(|| license.and_then(|item| item.company_name.clone()))
            .unwrap_or_else(|| settings.company_name.clone()),
        emp_email: company
            .email
            .clone()
            .or_else(|| license.and_then(|item| item.company_email.clone()))
            .unwrap_or_else(|| settings.company_email.clone()),
        emp_serie: app_license
            .and_then(|item| item.application_slug.clone())
            .unwrap_or_else(|| settings.app_instance.clone()),
        bloqueio_admin: false,
        bloqueado: license.map(|item| item.blocked).unwrap_or(false),
        ativo: license.map(|item| item.active).unwrap_or(true),
        n_maquinas: app_license
            .and_then(|item| item.max_devices)
            .or_else(|| license.and_then(|item| item.max_devices))
            .unwrap_or(0) as i64,
        data_val_lic: app_license
            .and_then(|item| item.expires_at.as_ref())
            .or_else(|| license.and_then(|item| item.expires_at.as_ref()))
            .map(format_expiry)
            .unwrap_or_default(),
        emp_obs: DEFAULT_PRODUCT_NAME.to_string(),
        ..LicensedCompany::default()
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
        emp_email: license
            .company_email
            .clone()
            .unwrap_or_else(|| settings.company_email.clone()),
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
    let full_name = full_device_name(
        device.station_name.as_deref().or(Some(station_name)),
        device.computer_name.as_deref(),
        device.hostname.as_deref(),
    );

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
            .computer_name
            .clone()
            .or_else(|| device.hostname.clone())
            .or_else(|| device.device_name.clone())
            .unwrap_or_else(|| station_name.to_string()),
        prog_acesso: device.logged_user.clone().unwrap_or_default(),
        cod_ace_remoto: device.domain_name.clone().unwrap_or_default(),
        versao_bd: String::new(),
        versaoexe: env!("CARGO_PKG_VERSION").to_string(),
        sistema_operacional: full_os_name(
            device.os_name.as_deref(),
            device.os_version.as_deref(),
            device.os_arch.as_deref(),
        ),
        memoria_ram: String::new(),
        tipo: device
            .install_mode
            .clone()
            .unwrap_or_else(|| settings.app_instance.clone()),
        observacao: DEFAULT_PRODUCT_NAME.to_string(),
        tecnico_instalacao: device.logged_user.clone().unwrap_or_default(),
        serial_number: device.serial_number.clone().unwrap_or_default(),
        hostname: device.hostname.clone().unwrap_or_default(),
        station_name: device
            .station_name
            .clone()
            .or_else(|| device.device_name.clone())
            .unwrap_or_else(|| station_name.to_string()),
        machine_guid: device.machine_guid.clone().unwrap_or_default(),
        bios_serial: device.bios_serial.clone().unwrap_or_default(),
        motherboard_serial: device.motherboard_serial.clone().unwrap_or_default(),
        full_device_name: full_name,
    }
}

fn full_os_name(os_name: Option<&str>, os_version: Option<&str>, os_arch: Option<&str>) -> String {
    let mut parts = Vec::new();

    if let Some(value) = os_name {
        if !value.trim().is_empty() {
            parts.push(value.trim().to_string());
        }
    }
    if let Some(value) = os_version {
        if !value.trim().is_empty() {
            parts.push(value.trim().to_string());
        }
    }
    if let Some(value) = os_arch {
        if !value.trim().is_empty() {
            parts.push(value.trim().to_string());
        }
    }

    parts.join(" | ")
}

fn full_device_name(
    station_name: Option<&str>,
    computer_name: Option<&str>,
    hostname: Option<&str>,
) -> String {
    let mut parts = Vec::new();

    if let Some(value) = station_name {
        let trimmed = value.trim();
        if !trimmed.is_empty() && !parts.iter().any(|item| item == trimmed) {
            parts.push(trimmed.to_string());
        }
    }
    if let Some(value) = computer_name {
        let trimmed = value.trim();
        if !trimmed.is_empty() && !parts.iter().any(|item| item == trimmed) {
            parts.push(trimmed.to_string());
        }
    }
    if let Some(value) = hostname {
        let trimmed = value.trim();
        if !trimmed.is_empty() && !parts.iter().any(|item| item == trimmed) {
            parts.push(trimmed.to_string());
        }
    }

    parts.join(" | ")
}

fn format_expiry(value: &chrono::DateTime<chrono::FixedOffset>) -> String {
    value
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn generate_machine_fingerprint(app_instance: &str) -> String {
    let settings = LicenseSettings {
        app_instance: app_instance.to_string(),
        ..LicenseSettings::default()
    };
    let device = collect_device_metadata();
    let input = build_license_input(&settings, &device, None);
    generate_device_key(&input)
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
