use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use tauri::{AppHandle, Manager};

use crate::core::domain::document::{ConversionProfile, ProfileBundle};

pub fn save_profile_bundle(app: &AppHandle, bundle: &ProfileBundle) -> Result<()> {
    let canonical_bundle = canonicalize_bundle(bundle);
    let file = profile_bundle_file(app)?;
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&file, serde_json::to_string_pretty(&canonical_bundle)?)?;
    let selected = canonical_bundle
        .profiles
        .iter()
        .find(|item| item.profile_id == canonical_bundle.selected_profile_id)
        .cloned()
        .or_else(|| canonical_bundle.profiles.first().cloned())
        .unwrap_or_default();
    let single_file = single_profile_file(app)?;
    fs::write(single_file, serde_json::to_string_pretty(&selected)?)?;
    Ok(())
}

pub fn load_profile_bundle(app: &AppHandle) -> Result<Option<ProfileBundle>> {
    let file = profile_bundle_file(app)?;
    if file.exists() {
        let content = fs::read_to_string(file)?;
        let bundle: ProfileBundle = serde_json::from_str(&content)?;
        return Ok(Some(canonicalize_bundle(&bundle)));
    }

    let single_file = single_profile_file(app)?;
    if single_file.exists() {
        let content = fs::read_to_string(single_file)?;
        let profile: ConversionProfile = serde_json::from_str(&content)?;
        return Ok(Some(canonicalize_bundle(&ProfileBundle {
            selected_profile_id: profile.profile_id.clone(),
            profiles: vec![profile],
        })));
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

fn canonicalize_bundle(bundle: &ProfileBundle) -> ProfileBundle {
    let mut profiles = bundle
        .profiles
        .iter()
        .filter(|item| !item.profile_id.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();

    if profiles.is_empty() {
        profiles.push(ConversionProfile::default());
    }

    let selected_profile_id = if profiles
        .iter()
        .any(|item| item.profile_id == bundle.selected_profile_id)
    {
        bundle.selected_profile_id.clone()
    } else {
        profiles
            .first()
            .map(|item| item.profile_id.clone())
            .unwrap_or_else(|| ConversionProfile::default().profile_id)
    };

    ProfileBundle {
        selected_profile_id,
        profiles,
    }
}
