use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use tauri::{AppHandle, Manager};

use crate::commands::nfe_faturas::NfeFaturasSettings;

pub fn save_nfe_faturas_settings(app: &AppHandle, settings: &NfeFaturasSettings) -> Result<()> {
    let file = settings_file(app)?;
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(file, serde_json::to_string_pretty(settings)?)?;
    Ok(())
}

pub fn load_nfe_faturas_settings(app: &AppHandle) -> Result<Option<NfeFaturasSettings>> {
    let file = settings_file(app)?;
    if !file.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(file)?;
    let settings = serde_json::from_str::<NfeFaturasSettings>(&content)?;
    Ok(Some(settings))
}

fn settings_file(app: &AppHandle) -> Result<PathBuf> {
    let dir = app.path().app_data_dir()?;
    Ok(dir.join("config").join("nfe_faturas_settings.json"))
}
