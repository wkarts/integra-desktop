use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDateTime, TimeZone};
use reqwest::{Client, RequestBuilder};
use serde_json::{json, Value};

use crate::error::LicenseError;
use crate::models::{
    DeviceRecord, LicenseApiResponse, LicenseCheckInput, LicenseConfig, LicenseRecord,
};

pub struct LicenseApiClient {
    config: LicenseConfig,
    http: Client,
}

impl LicenseApiClient {
    pub fn new(config: LicenseConfig) -> Self {
        Self {
            config,
            http: Client::new(),
        }
    }

    pub async fn company_status(&self, document: &str) -> Result<LicenseApiResponse, LicenseError> {
        self.guard()?;

        let endpoint = self.config.status_endpoint.replace("{document}", document);

        let response = self
            .with_auth(self.http.get(self.url(&endpoint)))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| LicenseError::Http(e.to_string()))?;

        if !response.status().is_success() {
            return Err(LicenseError::Http(format!("HTTP {}", response.status())));
        }

        let value = response
            .json::<Value>()
            .await
            .map_err(|e| LicenseError::Serde(e.to_string()))?;

        normalize_response(value)
    }

    pub async fn register_company(&self, input: &LicenseCheckInput) -> Result<(), LicenseError> {
        self.guard()?;

        let payload = json!({
            "DATACAD": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            "CNPJ": input.company_document,
            "RAZAOSOCIAL": input.company_name.clone().unwrap_or_default(),
            "EMP_NOMEFANTASIA": input.company_name.clone().unwrap_or_default(),
            "ATIVO": "S",
            "BLOQUEADO": "N",
            "DATA_VAL_LIC": Local::now().checked_add_days(chrono::Days::new(30)).unwrap_or_else(Local::now).format("%Y-%m-%d %H:%M:%S").to_string(),
            "dia_venc_mensalidade": Local::now().day(),
            "n_maquinas": 1,
            "EMP_SERIE": input.app_id,
            "emp_obs": input.app_name
        });

        let response = self
            .with_auth(
                self.http
                    .post(self.url(&self.config.register_company_endpoint)),
            )
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| LicenseError::Http(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(LicenseError::Http(format!("HTTP {}", response.status())))
        }
    }

    pub async fn register_device(&self, input: &LicenseCheckInput) -> Result<(), LicenseError> {
        self.guard()?;

        let payload = json!({
            "CNPJ": input.company_document,
            "CHAVE": input.device_key.clone().unwrap_or_default(),
            "NOME": input.device_name.clone().unwrap_or_default(),
            "nome_compu": input.device_name.clone().unwrap_or_default(),
            "BLOQUEADO": "N",
            "versaoexe": input.app_version,
            "observacao": input.app_name,
        });

        let response = self
            .with_auth(
                self.http
                    .post(self.url(&self.config.register_device_endpoint)),
            )
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| LicenseError::Http(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(LicenseError::Http(format!("HTTP {}", response.status())))
        }
    }

    pub async fn update_device_name(
        &self,
        device_id: &str,
        device_name: &str,
        app_version: &str,
    ) -> Result<(), LicenseError> {
        self.guard()?;

        let endpoint = self
            .config
            .update_device_endpoint
            .replace("{id}", device_id);

        let payload = json!({
            "NOME": device_name,
            "nome_compu": device_name,
            "versaoexe": app_version
        });

        let response = self
            .with_auth(self.http.put(self.url(&endpoint)))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| LicenseError::Http(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(LicenseError::Http(format!("HTTP {}", response.status())))
        }
    }

    fn with_auth(&self, builder: RequestBuilder) -> RequestBuilder {
        if let Some(token) = &self.config.api_token {
            if token.trim().is_empty() {
                builder
            } else {
                builder.bearer_auth(token)
            }
        } else {
            builder
        }
    }

    fn url(&self, endpoint: &str) -> String {
        format!(
            "{}/{}",
            self.config.base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        )
    }

    fn guard(&self) -> Result<(), LicenseError> {
        if self.config.base_url.trim().is_empty() {
            return Err(LicenseError::Config(
                "base_url da licença não informada".to_string(),
            ));
        }

        Ok(())
    }
}

fn normalize_response(value: Value) -> Result<LicenseApiResponse, LicenseError> {
    let obj = value
        .as_object()
        .ok_or_else(|| LicenseError::Serde("resposta inválida da API de licença".to_string()))?;

    if obj.contains_key("license") || obj.contains_key("status") {
        let status = obj.get("status").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let message = obj
            .get("message")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let license = obj
            .get("license")
            .map(normalize_license)
            .unwrap_or_else(|| normalize_license(&value));

        return Ok(LicenseApiResponse {
            status,
            message,
            license,
        });
    }

    let status = obj.get("STATUS").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let message = obj
        .get("MESSAGE")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    Ok(LicenseApiResponse {
        status,
        message,
        license: normalize_license(&value),
    })
}

fn normalize_license(value: &Value) -> LicenseRecord {
    let obj = match value.as_object() {
        Some(obj) => obj,
        None => return LicenseRecord::default(),
    };

    let devices = obj
        .get("devices")
        .or_else(|| obj.get("computers"))
        .or_else(|| obj.get("COMPUTADORES"))
        .or_else(|| obj.get("maquinas"))
        .or_else(|| obj.get("MAQUINAS"))
        .and_then(|v| v.as_array())
        .map(|items| items.iter().map(normalize_device).collect())
        .unwrap_or_default();

    LicenseRecord {
        document: obj
            .get("document")
            .or_else(|| obj.get("CNPJ"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        company_name: obj
            .get("company_name")
            .or_else(|| obj.get("RAZAO"))
            .or_else(|| obj.get("RAZAOSOCIAL"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        blocked: obj
            .get("blocked")
            .map(parse_bool)
            .or_else(|| obj.get("BLOQUEADO").map(parse_bool))
            .unwrap_or(false),
        active: obj
            .get("active")
            .map(parse_bool)
            .or_else(|| obj.get("ATIVO").map(parse_bool))
            .unwrap_or(true),
        expires_at: obj
            .get("expires_at")
            .or_else(|| obj.get("DATA_VAL_LIC"))
            .and_then(parse_datetime),
        max_devices: obj
            .get("max_devices")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .or_else(|| {
                obj.get("QTD_MAQ")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
            })
            .or_else(|| {
                obj.get("n_maquinas")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
            })
            .or_else(|| {
                obj.get("N_MAQUINAS")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
            }),
        devices,
    }
}

fn normalize_device(value: &Value) -> DeviceRecord {
    let obj = match value.as_object() {
        Some(obj) => obj,
        None => return DeviceRecord::default(),
    };

    DeviceRecord {
        id: obj
            .get("id")
            .or_else(|| obj.get("IDMAQ"))
            .or_else(|| obj.get("IDMAQUINA"))
            .or_else(|| obj.get("idmaquina"))
            .map(stringify_value),
        device_key: obj
            .get("device_key")
            .or_else(|| obj.get("key"))
            .or_else(|| obj.get("chave"))
            .or_else(|| obj.get("CHAVE"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        device_name: obj
            .get("device_name")
            .or_else(|| obj.get("NOME"))
            .or_else(|| obj.get("NOMEMAQ"))
            .or_else(|| obj.get("nome_compu"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        blocked: obj
            .get("blocked")
            .map(parse_bool)
            .or_else(|| obj.get("BLOQUEADO").map(parse_bool))
            .unwrap_or(false),
    }
}

fn parse_bool(value: &Value) -> bool {
    match value {
        Value::Bool(v) => *v,
        Value::Number(v) => v.as_i64().unwrap_or(0) != 0,
        Value::String(v) => matches!(
            v.trim().to_uppercase().as_str(),
            "1" | "TRUE" | "T" | "Y" | "YES" | "S" | "SIM"
        ),
        _ => false,
    }
}

fn parse_datetime(value: &Value) -> Option<DateTime<FixedOffset>> {
    let raw = value.as_str()?;

    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Some(dt);
    }

    if let Ok(dt) = DateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S%:z") {
        return Some(dt);
    }

    for fmt in ["%Y-%m-%d %H:%M:%S", "%d/%m/%Y %H:%M:%S"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(raw, fmt) {
            if let Some(local_dt) = Local.from_local_datetime(&naive).single() {
                return Some(local_dt.fixed_offset());
            }
        }
    }

    None
}

fn stringify_value(value: &Value) -> String {
    match value {
        Value::String(v) => v.clone(),
        Value::Number(v) => v.to_string(),
        Value::Bool(v) => {
            if *v {
                "true".to_string()
            } else {
                "false".to_string()
            }
        },
        _ => String::new(),
    }
}
