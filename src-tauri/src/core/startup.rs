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

    for current in &args {
        let (key, inline_value) = split_arg(current);
        match key {
            "--disable-licensing" | "--licensing-disabled" | "--no-license" => {
                context.licensing_disabled = parse_bool_flag(inline_value).unwrap_or(true);
            }
            _ => {}
        }
    }

    ParsedStartupLicenseArgs {
        public: context,
        local_license_token: None,
        developer_secret: None,
    }
}

fn split_arg(arg: &str) -> (&str, Option<&str>) {
    if let Some((key, value)) = arg.split_once('=') {
        (key, Some(value))
    } else {
        (arg, None)
    }
}

fn parse_bool_flag(value: Option<&str>) -> Option<bool> {
    value.map(|item| {
        matches!(
            item.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on" | "sim"
        )
    })
}
