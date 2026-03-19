use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use tauri::{AppHandle, Manager};

use crate::core::domain::license::LicenseSettings;

pub fn save_license_settings(app: &AppHandle, settings: &LicenseSettings) -> Result<()> {
    let file = settings_file(app)?;
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(file, serde_json::to_string_pretty(settings)?)?;
    Ok(())
}

pub fn load_license_settings(app: &AppHandle) -> Result<Option<LicenseSettings>> {
    let file = settings_file(app)?;
    if !file.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(file)?;
    Ok(Some(serde_json::from_str(&content)?))
}

fn settings_file(app: &AppHandle) -> Result<PathBuf> {
    let dir = app.path().app_data_dir()?;
    Ok(dir.join("config").join("license_settings.json"))
}
