use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

use crate::core::domain::license::{
    GenerateLocalLicenseRequest, GeneratedLocalLicense, LocalLicenseDocument,
    LocalLicenseValidationResult, ValidateLocalLicenseRequest,
};
use crate::core::startup::ParsedStartupLicenseArgs;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalLicenseUnsignedDocument {
    version: u32,
    issuer: String,
    app_instance: String,
    company_name: String,
    company_document: String,
    company_email: String,
    station_name: String,
    machine_key: String,
    serial_number: String,
    requested_licenses: Option<u32>,
    issued_at: String,
    expires_at: Option<String>,
}

pub fn generate_local_license(
    request: GenerateLocalLicenseRequest,
) -> Result<GeneratedLocalLicense, String> {
    validate_developer_token(request.developer_token.as_deref())?;
    let secret = resolve_secret(request.developer_secret.as_deref())?;
    let issuer = request
        .issuer_name
        .clone()
        .unwrap_or_else(|| "WWSoftwares Local License".to_string());

    let unsigned = LocalLicenseUnsignedDocument {
        version: 1,
        issuer: issuer.clone(),
        app_instance: non_empty(request.app_instance, "integra-desktop"),
        company_name: request.company_name.trim().to_string(),
        company_document: only_digits(&request.company_document),
        company_email: request.company_email.trim().to_string(),
        station_name: request.station_name.trim().to_string(),
        machine_key: request.machine_key.trim().to_string(),
        serial_number: request.serial_number.trim().to_string(),
        requested_licenses: request.requested_licenses,
        issued_at: Local::now().to_rfc3339(),
        expires_at: request.expires_at.clone().filter(|v| !v.trim().is_empty()),
    };

    let signature = sign_document(&unsigned, &secret)?;
    let payload = LocalLicenseDocument {
        version: unsigned.version,
        issuer: unsigned.issuer.clone(),
        app_instance: unsigned.app_instance.clone(),
        company_name: unsigned.company_name.clone(),
        company_document: unsigned.company_document.clone(),
        company_email: unsigned.company_email.clone(),
        station_name: unsigned.station_name.clone(),
        machine_key: unsigned.machine_key.clone(),
        serial_number: unsigned.serial_number.clone(),
        requested_licenses: unsigned.requested_licenses,
        issued_at: unsigned.issued_at.clone(),
        expires_at: unsigned.expires_at.clone(),
        signature: signature.clone(),
    };

    let file_content = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
    let file_path = request.output_path.clone().filter(|v| !v.trim().is_empty());
    if let Some(path) = &file_path {
        if let Some(parent) = PathBuf::from(path).parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(path, &file_content).map_err(|e| e.to_string())?;
    }

    let account = request
        .account_name
        .clone()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| unsigned.company_document.clone());
    let otpauth_uri = Some(build_otpauth_uri(&issuer, &account, &secret));

    Ok(GeneratedLocalLicense {
        file_content,
        file_path,
        signature,
        otpauth_uri,
        payload,
    })
}

pub fn validate_local_license(
    request: ValidateLocalLicenseRequest,
) -> Result<LocalLicenseValidationResult, String> {
    validate_developer_token(request.developer_token.as_deref())?;
    let secret = resolve_secret(request.developer_secret.as_deref())?;
    let (content, file_path) = read_license_content(&request)?;
    let payload: LocalLicenseDocument =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let unsigned = LocalLicenseUnsignedDocument {
        version: payload.version,
        issuer: payload.issuer.clone(),
        app_instance: payload.app_instance.clone(),
        company_name: payload.company_name.clone(),
        company_document: payload.company_document.clone(),
        company_email: payload.company_email.clone(),
        station_name: payload.station_name.clone(),
        machine_key: payload.machine_key.clone(),
        serial_number: payload.serial_number.clone(),
        requested_licenses: payload.requested_licenses,
        issued_at: payload.issued_at.clone(),
        expires_at: payload.expires_at.clone(),
    };

    let expected = sign_document(&unsigned, &secret)?;
    if expected != payload.signature {
        return Ok(LocalLicenseValidationResult {
            valid: false,
            reason_code: "LOCAL_LICENSE_SIGNATURE_INVALID".to_string(),
            message: "a assinatura da licença local é inválida".to_string(),
            file_path,
            otpauth_uri: Some(build_otpauth_uri(
                &payload.issuer,
                &payload.company_document,
                &secret,
            )),
            payload: Some(payload),
        });
    }

    if let Some(expires_at) = payload.expires_at.as_ref() {
        let parsed = DateTime::parse_from_rfc3339(expires_at).map_err(|e| e.to_string())?;
        if parsed.with_timezone(&Utc) < Utc::now() {
            return Ok(LocalLicenseValidationResult {
                valid: false,
                reason_code: "LOCAL_LICENSE_EXPIRED".to_string(),
                message: "a licença local expirou".to_string(),
                file_path,
                otpauth_uri: Some(build_otpauth_uri(
                    &payload.issuer,
                    &payload.company_document,
                    &secret,
                )),
                payload: Some(payload),
            });
        }
    }

    if let Some(document) = request
        .company_document
        .as_ref()
        .filter(|v| !v.trim().is_empty())
    {
        if only_digits(document) != only_digits(&payload.company_document) {
            return Ok(LocalLicenseValidationResult {
                valid: false,
                reason_code: "LOCAL_LICENSE_COMPANY_MISMATCH".to_string(),
                message: "o documento da licença local não corresponde à empresa informada"
                    .to_string(),
                file_path,
                otpauth_uri: Some(build_otpauth_uri(
                    &payload.issuer,
                    &payload.company_document,
                    &secret,
                )),
                payload: Some(payload),
            });
        }
    }

    if request.enforce_machine_match {
        if let Some(machine_key) = request
            .machine_key
            .as_ref()
            .filter(|v| !v.trim().is_empty())
        {
            if machine_key.trim() != payload.machine_key.trim() {
                return Ok(LocalLicenseValidationResult {
                    valid: false,
                    reason_code: "LOCAL_LICENSE_MACHINE_MISMATCH".to_string(),
                    message: "a licença local não corresponde à máquina atual".to_string(),
                    file_path,
                    otpauth_uri: Some(build_otpauth_uri(
                        &payload.issuer,
                        &payload.company_document,
                        &secret,
                    )),
                    payload: Some(payload),
                });
            }
        }
    }

    Ok(LocalLicenseValidationResult {
        valid: true,
        reason_code: "LOCAL_LICENSE_VALID".to_string(),
        message: "licença local validada com sucesso".to_string(),
        file_path,
        otpauth_uri: Some(build_otpauth_uri(
            &payload.issuer,
            &payload.company_document,
            &secret,
        )),
        payload: Some(payload),
    })
}

#[allow(dead_code)]
pub fn validate_local_license_from_startup(
    startup: &ParsedStartupLicenseArgs,
    app_instance: &str,
    company_document: Option<&str>,
    machine_key: Option<&str>,
) -> Result<Option<LocalLicenseValidationResult>, String> {
    if !startup.public.local_license_enabled {
        return Ok(None);
    }

    let request = ValidateLocalLicenseRequest {
        file_path: startup
            .public
            .local_license_file_path
            .clone()
            .or_else(|| default_local_license_path(app_instance)),
        content_b64: None,
        company_document: company_document.map(|v| v.to_string()),
        machine_key: machine_key.map(|v| v.to_string()),
        developer_token: startup.local_license_token.clone(),
        developer_secret: startup.developer_secret.clone(),
        enforce_machine_match: false,
    };

    let result = validate_local_license(request)?;
    Ok(Some(result))
}

#[allow(dead_code)]
pub fn default_local_license_path(app_instance: &str) -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    Some(
        dir.join(format!("{}.local.lic.json", app_instance))
            .to_string_lossy()
            .to_string(),
    )
}

fn sign_document(document: &LocalLicenseUnsignedDocument, secret: &str) -> Result<String, String> {
    let raw = serde_json::to_vec(document).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(b"|");
    hasher.update(raw);
    Ok(BASE64.encode(hasher.finalize()))
}

fn read_license_content(
    request: &ValidateLocalLicenseRequest,
) -> Result<(String, Option<String>), String> {
    if let Some(content_b64) = request
        .content_b64
        .as_ref()
        .filter(|v| !v.trim().is_empty())
    {
        let bytes = BASE64.decode(content_b64).map_err(|e| e.to_string())?;
        let content = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        return Ok((content, request.file_path.clone()));
    }

    let path = request
        .file_path
        .as_ref()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| "arquivo de licença local não informado".to_string())?;
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok((content, Some(path.clone())))
}

fn resolve_secret(input: Option<&str>) -> Result<String, String> {
    if let Some(value) = input.filter(|v| !v.trim().is_empty()) {
        return Ok(value.trim().to_string());
    }

    std::env::var("LICENSE_LOCAL_DEV_SECRET").map_err(|_| {
        "secret do desenvolvedor não informado (use --local-license-secret ou LICENSE_LOCAL_DEV_SECRET)"
            .to_string()
    })
}

fn validate_developer_token(token: Option<&str>) -> Result<(), String> {
    if let Ok(expected) = std::env::var("LICENSE_LOCAL_DEV_TOKEN") {
        if expected.trim().is_empty() {
            return Ok(());
        }

        let provided = token.unwrap_or("").trim();
        if provided != expected.trim() {
            return Err(
                "token do desenvolvedor inválido para operação de licença local".to_string(),
            );
        }
    }

    Ok(())
}

fn build_otpauth_uri(issuer: &str, account: &str, secret: &str) -> String {
    let issuer_clean = issuer.trim().replace(' ', "%20");
    let account_clean = account.trim().replace(' ', "%20");
    let base32_secret = base32_encode(secret.as_bytes());
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}",
        issuer_clean, account_clean, base32_secret, issuer_clean
    )
}

fn base32_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut output = String::new();
    let mut buffer = 0u16;
    let mut bits_left = 0u8;

    for &byte in bytes {
        buffer = (buffer << 8) | byte as u16;
        bits_left += 8;
        while bits_left >= 5 {
            let index = ((buffer >> (bits_left - 5)) & 0b1_1111) as usize;
            output.push(ALPHABET[index] as char);
            bits_left -= 5;
        }
    }

    if bits_left > 0 {
        let index = ((buffer << (5 - bits_left)) & 0b1_1111) as usize;
        output.push(ALPHABET[index] as char);
    }

    output
}

fn only_digits(value: &str) -> String {
    value.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn non_empty(value: String, default: &str) -> String {
    if value.trim().is_empty() {
        default.to_string()
    } else {
        value.trim().to_string()
    }
}
