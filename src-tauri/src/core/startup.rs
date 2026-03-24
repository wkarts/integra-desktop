use std::env;

use crate::core::domain::license::StartupLicenseContext;

#[derive(Debug, Clone, Default)]
pub struct ParsedStartupLicenseArgs {
    pub public: StartupLicenseContext,
    pub local_license_token: Option<String>,
    pub developer_secret: Option<String>,
}

pub fn parse_startup_license_args() -> ParsedStartupLicenseArgs {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let mut context = StartupLicenseContext {
        args: args.clone(),
        ..StartupLicenseContext::default()
    };
    let mut parsed = ParsedStartupLicenseArgs {
        public: context.clone(),
        local_license_token: None,
        developer_secret: None,
    };

    let mut index = 0usize;
    while index < args.len() {
        let current = &args[index];
        if !current.starts_with("--") {
            index += 1;
            continue;
        }

        let (key, inline_value) = split_arg(current);
        let mut consumed_next = false;
        let next_value = if let Some(value) = inline_value {
            Some(value.to_string())
        } else if index + 1 < args.len() && !args[index + 1].starts_with("--") {
            consumed_next = true;
            Some(args[index + 1].clone())
        } else {
            None
        };

        match key {
            "--auto-register" => {
                context.auto_register_enabled = true;
            }
            "--auto-register-company" => {
                context.auto_register_enabled = true;
                context.auto_register_company = true;
            }
            "--auto-register-device" => {
                context.auto_register_enabled = true;
                context.auto_register_device = true;
            }
            "--lic" | "--licenses" => {
                if let Some(value) = next_value.clone() {
                    context.requested_licenses = value.parse::<u32>().ok();
                }
            }
            "--company-document" | "--document" | "--cnpj" | "--cpf-cnpj" => {
                context.company_document = next_value.clone();
            }
            "--company-name" | "--empresa" => {
                context.company_name = next_value.clone();
            }
            "--company-email" | "--email" => {
                context.company_email = next_value.clone();
            }
            "--station-name" | "--station" => {
                context.station_name = next_value.clone();
            }
            "--device-name" => {
                context.device_name = next_value.clone();
            }
            "--device" | "--device-id" | "--device-identifier" => {
                context.device_identifier = next_value.clone();
            }
            "--validation-mode" => {
                context.validation_mode = next_value.clone();
            }
            "--ui-mode" => {
                context.interface_mode = next_value.clone();
            }
            "--silent" | "--headless" | "--no-ui" => {
                context.no_ui = true;
                context.interface_mode = Some("silent".to_string());
            }
            "--local-license" => {
                context.local_license_enabled = true;
            }
            "--local-license-generate" => {
                context.local_license_enabled = true;
                context.local_license_generate = true;
            }
            "--local-license-file" => {
                context.local_license_enabled = true;
                context.local_license_file_path = next_value.clone();
            }
            "--local-license-token" => {
                parsed.local_license_token = next_value.clone();
            }
            "--local-license-secret" => {
                parsed.developer_secret = next_value.clone();
            }
            "--local-license-account" => {
                context.local_license_account = next_value.clone();
            }
            "--local-license-issuer" => {
                context.local_license_issuer = next_value.clone();
            }
            _ => {}
        }

        if consumed_next {
            index += 1;
        }
        index += 1;
    }

    if context.auto_register_enabled && !context.auto_register_company && !context.auto_register_device {
        context.auto_register_company = true;
        context.auto_register_device = true;
    }

    if context.interface_mode.is_none() {
        context.interface_mode = Some(if context.no_ui {
            "silent".to_string()
        } else {
            "interactive".to_string()
        });
    }

    if context.validation_mode.is_none() {
        context.validation_mode = Some("standard".to_string());
    }

    context.local_license_token_present = parsed.local_license_token.as_ref().map(|v| !v.trim().is_empty()).unwrap_or(false);
    context.developer_secret_present = parsed.developer_secret.as_ref().map(|v| !v.trim().is_empty()).unwrap_or(false);

    parsed.public = context;
    parsed
}

fn split_arg(arg: &str) -> (&str, Option<&str>) {
    if let Some((key, value)) = arg.split_once('=') {
        (key, Some(value))
    } else {
        (arg, None)
    }
}
