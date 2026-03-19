use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use tauri::{AppHandle, Manager};

use crate::core::domain::document::ConversionProfile;

pub fn save_conversion_profile(app: &AppHandle, profile: &ConversionProfile) -> Result<()> {
    let file = profile_file(app)?;
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(file, serde_json::to_string_pretty(profile)?)?;
    Ok(())
}

pub fn load_conversion_profile(app: &AppHandle) -> Result<Option<ConversionProfile>> {
    let file = profile_file(app)?;
    if !file.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(file)?;
    Ok(Some(serde_json::from_str(&content)?))
}

fn profile_file(app: &AppHandle) -> Result<PathBuf> {
    let dir = app.path().app_data_dir()?;
    Ok(dir.join("config").join("conversion_profile.json"))
}
