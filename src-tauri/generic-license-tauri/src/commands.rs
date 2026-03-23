#[cfg(feature = "tauri-commands")]
use std::sync::Arc;

#[cfg(feature = "tauri-commands")]
use tauri::State;

#[cfg(feature = "tauri-commands")]
use crate::{
    error::LicenseError,
    models::{LicenseCheckInput, LicenseDecision},
    service::GenericLicenseService,
};

#[cfg(feature = "tauri-commands")]
pub type SharedLicenseService = Arc<GenericLicenseService>;

#[cfg(feature = "tauri-commands")]
#[tauri::command]
pub async fn license_check(
    service: State<'_, SharedLicenseService>,
    input: LicenseCheckInput,
) -> Result<LicenseDecision, String> {
    service.check(input).await.map_err(map_error)
}

#[cfg(feature = "tauri-commands")]
fn map_error(err: LicenseError) -> String {
    err.to_string()
}
