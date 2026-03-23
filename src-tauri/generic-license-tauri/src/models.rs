use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseApiResponse {
    pub status: i32,
    pub message: Option<String>,
    #[serde(default)]
    pub license: LicenseRecord,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LicenseRecord {
    pub document: Option<String>,
    pub company_name: Option<String>,
    #[serde(default)]
    pub blocked: bool,
    #[serde(default)]
    pub active: bool,
    pub expires_at: Option<DateTime<FixedOffset>>,
    pub max_devices: Option<u32>,
    #[serde(default)]
    pub devices: Vec<DeviceRecord>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceRecord {
    pub id: Option<String>,
    pub device_key: Option<String>,
    pub device_name: Option<String>,
    #[serde(default)]
    pub blocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedLicenseFile {
    pub cached_at: DateTime<FixedOffset>,
    pub payload: LicenseApiResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseCheckInput {
    pub company_document: String,
    pub company_name: Option<String>,
    pub app_id: String,
    pub app_name: String,
    pub app_version: String,
    pub device_key: Option<String>,
    pub device_name: Option<String>,
    #[serde(default)]
    pub login_context: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseDecision {
    pub allowed: bool,
    pub message: String,
    pub used_offline_cache: bool,
    pub warning: Option<String>,
    pub license: Option<LicenseRecord>,
    pub device: Option<DeviceRecord>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseConfig {
    pub base_url: String,
    pub api_token: Option<String>,
    pub status_endpoint: String,
    pub register_company_endpoint: String,
    pub register_device_endpoint: String,
    pub update_device_endpoint: String,
    pub offline_max_age_days: i64,
    pub warn_before_expiration_in_days: i64,
    pub auto_register_company_on_missing: bool,
    pub auto_register_device_on_missing: bool,
    pub auto_update_device_name: bool,
    pub block_on_company_blocked: bool,
    pub block_on_device_blocked: bool,
    pub block_on_device_missing: bool,
    pub block_on_expired: bool,
    pub cache_namespace: String,
}

impl Default for LicenseConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.rest.wwsoftwares.com.br/api/v1".to_string(),
            api_token: None,
            status_endpoint: "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/cliente/{document}".to_string(),
            register_company_endpoint: "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/clientes".to_string(),
            register_device_endpoint: "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/maquinas".to_string(),
            update_device_endpoint: "81b3767f-7bc5-4275-9453-a6a921010a17/86d7b2bee439957e040b72be6fea5fc2/maquinas/IDMAQUINA/{id}".to_string(),
            offline_max_age_days: 15,
            warn_before_expiration_in_days: 5,
            auto_register_company_on_missing: false,
            auto_register_device_on_missing: false,
            auto_update_device_name: true,
            block_on_company_blocked: true,
            block_on_device_blocked: true,
            block_on_device_missing: true,
            block_on_expired: true,
            cache_namespace: "default".to_string(),
        }
    }
}
