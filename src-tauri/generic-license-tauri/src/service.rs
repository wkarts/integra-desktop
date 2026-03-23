use chrono::Utc;

use crate::cache::OfflineCache;
use crate::client::LicenseApiClient;
use crate::device::{default_device_name, generate_device_key};
use crate::error::LicenseError;
use crate::models::{
    DeviceRecord, LicenseApiResponse, LicenseCheckInput, LicenseConfig, LicenseDecision,
    LicenseRecord,
};

pub struct GenericLicenseService {
    config: LicenseConfig,
    api: LicenseApiClient,
    cache: OfflineCache,
}

impl GenericLicenseService {
    pub fn new(config: LicenseConfig) -> Self {
        let api = LicenseApiClient::new(config.clone());
        let cache = OfflineCache::new(config.cache_namespace.clone());

        Self { config, api, cache }
    }

    pub async fn check(
        &self,
        mut input: LicenseCheckInput,
    ) -> Result<LicenseDecision, LicenseError> {
        input.company_document = only_digits(&input.company_document);

        if input.company_document.is_empty() {
            return Err(LicenseError::Invalid(
                "documento da empresa não informado".to_string(),
            ));
        }

        if input.device_key.as_deref().unwrap_or("").is_empty() {
            input.device_key = Some(generate_device_key(&input.app_id));
        }

        if input.device_name.as_deref().unwrap_or("").is_empty() {
            input.device_name = Some(default_device_name());
        }

        match self.api.company_status(&input.company_document).await {
            Ok(payload) => {
                let _ = self.cache.put(&input.company_document, &payload).await;
                self.evaluate(payload, &input, false).await
            }
            Err(err) => {
                let cached = self.cache.get(&input.company_document).await?;
                let cached = match cached {
                    Some(value) => value,
                    None => {
                        return Err(LicenseError::Invalid(format!(
                            "falha online ({}) e não existe cache offline para esta licença",
                            err
                        )))
                    }
                };

                let age_days = Utc::now()
                    .signed_duration_since(cached.cached_at.with_timezone(&Utc))
                    .num_days();

                if age_days > self.config.offline_max_age_days {
                    return Err(LicenseError::Invalid(
                        "falha online e o cache offline expirou".to_string(),
                    ));
                }

                self.evaluate(cached.payload, &input, true).await
            }
        }
    }

    async fn evaluate(
        &self,
        payload: LicenseApiResponse,
        input: &LicenseCheckInput,
        used_offline_cache: bool,
    ) -> Result<LicenseDecision, LicenseError> {
        if payload.status == 0 {
            if self.config.auto_register_company_on_missing {
                let _ = self.api.register_company(input).await;
            }

            return Ok(LicenseDecision {
                allowed: false,
                message: payload.message.unwrap_or_else(|| {
                    "empresa não cadastrada no serviço de licenciamento".to_string()
                }),
                used_offline_cache,
                warning: None,
                license: Some(payload.license),
                device: None,
                source: source_label(used_offline_cache),
            });
        }

        let license = payload.license.clone();

        if license.blocked && self.config.block_on_company_blocked {
            return Ok(deny(
                "empresa bloqueada no serviço de licenciamento",
                used_offline_cache,
                license,
                None,
            ));
        }

        if let Some(expires_at) = license.expires_at.clone() {
            if expires_at.with_timezone(&Utc) < Utc::now() && self.config.block_on_expired {
                return Ok(deny(
                    "licença expirada",
                    used_offline_cache,
                    payload.license,
                    None,
                ));
            }
        }

        let device_key = input.device_key.clone().unwrap_or_default();
        let device_name = input.device_name.clone().unwrap_or_default();
        let device = find_device(&license, &device_key);

        if device.is_none() {
            if self.config.auto_register_device_on_missing {
                let _ = self.api.register_device(input).await;
            }

            if self.config.block_on_device_missing {
                return Ok(deny(
                    "dispositivo não autorizado para esta licença",
                    used_offline_cache,
                    payload.license,
                    None,
                ));
            }
        }

        if let Some(current_device) = &device {
            if current_device.blocked && self.config.block_on_device_blocked {
                return Ok(deny(
                    "dispositivo bloqueado para esta licença",
                    used_offline_cache,
                    payload.license,
                    Some(current_device.clone()),
                ));
            }

            if self.config.auto_update_device_name {
                if let (Some(id), Some(name)) = (&current_device.id, &current_device.device_name) {
                    if name != &device_name {
                        let _ = self
                            .api
                            .update_device_name(id, &device_name, &input.app_version)
                            .await;
                    }
                }
            }
        }

        let warning = license.expires_at.clone().and_then(|expires_at| {
            let days_left = expires_at
                .with_timezone(&Utc)
                .signed_duration_since(Utc::now())
                .num_days();

            if days_left >= 0 && days_left <= self.config.warn_before_expiration_in_days {
                Some(format!("A licença vence em {} dia(s).", days_left))
            } else {
                None
            }
        });

        Ok(LicenseDecision {
            allowed: true,
            message: if used_offline_cache {
                "licença validada com cache offline".to_string()
            } else {
                "licença validada com sucesso".to_string()
            },
            used_offline_cache,
            warning,
            license: Some(payload.license),
            device,
            source: source_label(used_offline_cache),
        })
    }
}

fn deny(
    message: &str,
    used_offline_cache: bool,
    license: LicenseRecord,
    device: Option<DeviceRecord>,
) -> LicenseDecision {
    LicenseDecision {
        allowed: false,
        message: message.to_string(),
        used_offline_cache,
        warning: None,
        license: Some(license),
        device,
        source: source_label(used_offline_cache),
    }
}

fn source_label(used_offline_cache: bool) -> String {
    if used_offline_cache {
        "offline".to_string()
    } else {
        "online".to_string()
    }
}

fn find_device(license: &LicenseRecord, device_key: &str) -> Option<DeviceRecord> {
    license
        .devices
        .iter()
        .find(|device| device.device_key.as_deref().unwrap_or("") == device_key)
        .cloned()
}

fn only_digits(value: &str) -> String {
    value.chars().filter(|c| c.is_ascii_digit()).collect()
}
