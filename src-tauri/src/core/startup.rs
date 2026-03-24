use crate::core::domain::license::StartupLicenseContext;

#[derive(Debug, Clone, Default)]
pub struct ParsedStartupLicenseArgs {
    pub public: StartupLicenseContext,
    pub local_license_token: Option<String>,
    pub developer_secret: Option<String>,
}

pub fn parse_startup_license_args() -> ParsedStartupLicenseArgs {
    ParsedStartupLicenseArgs {
        public: StartupLicenseContext {
            args: Vec::new(),
            ..StartupLicenseContext::default()
        },
        local_license_token: None,
        developer_secret: None,
    }
}
