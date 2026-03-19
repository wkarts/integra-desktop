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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicenseCheckResult {
    pub allowed: bool,
    pub online: bool,
    pub active: bool,
    pub blocked: bool,
    pub device_registered: bool,
    pub device_blocked: bool,
    pub seats_total: u32,
    pub seats_used: u32,
    pub company_name: String,
    pub company_document: String,
    pub expires_at: Option<String>,
    pub message: String,
    pub machine_key: String,
    pub status_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMeta {
    pub product_name: String,
    pub version: String,
    pub build_hash: String,
    pub app_id: String,
}
