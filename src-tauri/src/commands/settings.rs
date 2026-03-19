use tauri::AppHandle;

use crate::core::domain::document::ConversionProfile;

#[tauri::command]
pub fn save_conversion_profile(profile: ConversionProfile, app: AppHandle) -> Result<(), String> {
    crate::storage::app_settings::save_conversion_profile(&app, &profile).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_conversion_profile(app: AppHandle) -> Result<Option<ConversionProfile>, String> {
    crate::storage::app_settings::load_conversion_profile(&app).map_err(|e| e.to_string())
}
