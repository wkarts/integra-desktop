use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicenseSettings {
    pub service_url: String,
    pub company_name: String,
    pub company_document: String,
    pub company_email: String,
    pub station_name: String,
    pub machine_key: String,
    pub auto_register_machine: bool,
    pub app_instance: String,
    #[serde(default)]
    pub auto_register_requested_licenses: Option<u32>,
    #[serde(default)]
    pub auto_register_validation_mode: String,
    #[serde(default)]
    pub auto_register_interface_mode: String,
    #[serde(default)]
    pub auto_register_device_identifier: String,
    #[serde(default)]
    pub licensing_disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalLicense {
    pub id: i64,
    pub empresa: String,
    pub cnpj: String,
    pub fantasia: String,
    pub serial: String,
    pub licencas: String,
    pub ativo: bool,
    pub endereco: String,
    pub bairro: String,
    pub cidade: String,
    pub uf: String,
    pub cep: String,
    pub numero: String,
    pub email: String,
    pub complemento: String,
    pub dias: i64,
    pub competencia: String,
    pub bloqueio: bool,
    pub retaguarda: bool,
    pub pdv: bool,
    pub cte: bool,
    pub mdfe: bool,
    pub nfe: bool,
    pub frente: bool,
    pub sat: bool,
    pub app: bool,
    pub boletos: bool,
    pub mfe: bool,
    pub commerce: bool,
    pub serial_key: String,
    pub terminal_ativo: bool,
    pub usadas: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicensedCompany {
    pub idcliente: i64,
    pub datacad: String,
    pub cnpj: String,
    pub emp_inscestrg: String,
    pub emp_inscmunicipal: String,
    pub emp_nomefantasia: String,
    pub razaosocial: String,
    pub emp_endereco: String,
    pub emp_numero: String,
    pub emp_bairro: String,
    pub emp_cidade: String,
    pub emp_uf: String,
    pub emp_cep: String,
    pub emp_complemento: String,
    pub telefone1: String,
    pub emp_email: String,
    pub emp_website: String,
    pub emp_responsavel: String,
    pub emp_cnae: String,
    pub emp_serie: String,
    pub idrepresentante: i64,
    pub bloqueio_admin: bool,
    pub bloqueado: bool,
    pub ativo: bool,
    pub dia_venc_mensalidade: i64,
    pub forma_pagamento: String,
    pub emp_obs: String,
    pub n_maquinas: i64,
    pub data_val_lic: String,
    pub api_whatsapp: String,
    pub token_whatsapp: String,
    pub default_msg_whatsapp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicensedDevice {
    pub idmaquina: i64,
    pub cnpj: String,
    pub chave: String,
    pub nome: String,
    pub bloqueado: bool,
    pub modulos: String,
    pub nome_compu: String,
    pub prog_acesso: String,
    pub cod_ace_remoto: String,
    pub versao_bd: String,
    pub versaoexe: String,
    pub sistema_operacional: String,
    pub memoria_ram: String,
    pub tipo: String,
    pub observacao: String,
    pub tecnico_instalacao: String,
    pub serial_number: String,
    pub hostname: String,
    pub station_name: String,
    pub machine_guid: String,
    pub bios_serial: String,
    pub motherboard_serial: String,
    pub full_device_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegistrationDeviceInfo {
    pub station_name: String,
    pub device_display_name: String,
    pub hostname: String,
    pub computer_name: String,
    pub serial_number: String,
    pub machine_guid: String,
    pub bios_serial: String,
    pub motherboard_serial: String,
    pub logged_user: String,
    pub os_name: String,
    pub os_version: String,
    pub os_arch: String,
    pub domain_name: String,
    pub install_mode: String,
    pub mac_addresses: Vec<String>,
    pub device_key: String,
    pub registration_file_found: bool,
    pub registration_file_path: Option<String>,
    pub registration_file_verified: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicenseRuntimeStatus {
    pub online: bool,
    pub allowed: bool,
    pub blocked: bool,
    pub machine_registered: bool,
    pub machine_blocked: bool,
    pub seats_total: u32,
    pub seats_used: u32,
    pub expiry: Option<String>,
    pub message: String,
    pub block_reason: Option<String>,
    pub technical_message: String,
    pub company_name: String,
    pub company_document: String,
    pub machine_key: String,
    pub status_code: i32,
    pub local_license: Option<LocalLicense>,
    pub licensed_company: Option<LicensedCompany>,
    pub licensed_device: Option<LicensedDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicenseSnapshot {
    pub last_sync_at: String,
    pub local_license: Option<LocalLicense>,
    pub licensed_company: Option<LicensedCompany>,
    pub licensed_devices: Vec<LicensedDevice>,
    pub runtime_status: LicenseRuntimeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMeta {
    pub product_name: String,
    pub version: String,
    pub build_hash: String,
    pub app_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StartupLicenseContext {
    pub auto_register_enabled: bool,
    pub auto_register_company: bool,
    pub auto_register_device: bool,
    pub requested_licenses: Option<u32>,
    pub company_name: Option<String>,
    pub company_document: Option<String>,
    pub company_email: Option<String>,
    pub station_name: Option<String>,
    pub device_name: Option<String>,
    pub device_identifier: Option<String>,
    pub validation_mode: Option<String>,
    pub interface_mode: Option<String>,
    pub local_license_enabled: bool,
    pub local_license_generate: bool,
    pub local_license_file_path: Option<String>,
    pub local_license_token_present: bool,
    pub developer_secret_present: bool,
    pub licensing_disabled: bool,
    pub local_license_account: Option<String>,
    pub local_license_issuer: Option<String>,
    pub no_ui: bool,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerateLocalLicenseRequest {
    pub company_name: String,
    pub company_document: String,
    pub company_email: String,
    pub station_name: String,
    pub machine_key: String,
    pub serial_number: String,
    pub requested_licenses: Option<u32>,
    pub expires_at: Option<String>,
    pub app_instance: String,
    pub developer_token: Option<String>,
    pub developer_secret: Option<String>,
    pub output_path: Option<String>,
    pub account_name: Option<String>,
    pub issuer_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalLicenseDocument {
    pub version: u32,
    pub issuer: String,
    pub app_instance: String,
    pub company_name: String,
    pub company_document: String,
    pub company_email: String,
    pub station_name: String,
    pub machine_key: String,
    pub serial_number: String,
    pub requested_licenses: Option<u32>,
    pub issued_at: String,
    pub expires_at: Option<String>,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneratedLocalLicense {
    pub file_content: String,
    pub file_path: Option<String>,
    pub signature: String,
    pub otpauth_uri: Option<String>,
    pub payload: LocalLicenseDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidateLocalLicenseRequest {
    pub file_path: Option<String>,
    pub content_b64: Option<String>,
    pub company_document: Option<String>,
    pub machine_key: Option<String>,
    pub developer_token: Option<String>,
    pub developer_secret: Option<String>,
    #[serde(default)]
    pub enforce_machine_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalLicenseValidationResult {
    pub valid: bool,
    pub reason_code: String,
    pub message: String,
    pub file_path: Option<String>,
    pub otpauth_uri: Option<String>,
    pub payload: Option<LocalLicenseDocument>,
}
