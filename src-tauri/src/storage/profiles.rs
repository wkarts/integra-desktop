use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use tauri::{AppHandle, Manager};

use crate::core::domain::document::{ConversionProfile, ProfileBundle};

pub fn save_profile_bundle(app: &AppHandle, bundle: &ProfileBundle) -> Result<()> {
    let file = profile_bundle_file(app)?;
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&file, serde_json::to_string_pretty(bundle)?)?;
    if let Some(selected) = bundle
        .profiles
        .iter()
        .find(|item| item.profile_id == bundle.selected_profile_id)
    {
        let single_file = single_profile_file(app)?;
        fs::write(single_file, serde_json::to_string_pretty(selected)?)?;
    }
    Ok(())
}

pub fn load_profile_bundle(app: &AppHandle) -> Result<Option<ProfileBundle>> {
    let file = profile_bundle_file(app)?;
    if file.exists() {
        let content = fs::read_to_string(file)?;
        return Ok(Some(serde_json::from_str(&content)?));
    }

    let single_file = single_profile_file(app)?;
    if single_file.exists() {
        let content = fs::read_to_string(single_file)?;
        let profile: ConversionProfile = serde_json::from_str(&content)?;
        return Ok(Some(ProfileBundle {
            selected_profile_id: profile.profile_id.clone(),
            profiles: vec![profile],
        }));
    }

    Ok(None)
}

fn profile_bundle_file(app: &AppHandle) -> Result<PathBuf> {
    let dir = app.path().app_data_dir()?;
    Ok(dir.join("config").join("profiles.json"))
}

fn single_profile_file(app: &AppHandle) -> Result<PathBuf> {
    let dir = app.path().app_data_dir()?;
    Ok(dir.join("config").join("conversion_profile.json"))
}
