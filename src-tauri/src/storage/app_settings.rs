use anyhow::Result;
use tauri::AppHandle;

use crate::core::domain::document::{ConversionProfile, ProfileBundle};

pub fn save_conversion_profile(app: &AppHandle, profile: &ConversionProfile) -> Result<()> {
    let bundle = ProfileBundle {
        selected_profile_id: profile.profile_id.clone(),
        profiles: vec![profile.clone()],
    };
    crate::storage::profiles::save_profile_bundle(app, &bundle)
}

pub fn load_conversion_profile(app: &AppHandle) -> Result<Option<ConversionProfile>> {
    let bundle = crate::storage::profiles::load_profile_bundle(app)?;
    Ok(bundle.and_then(|item| {
        item.profiles
            .iter()
            .find(|profile| profile.profile_id == item.selected_profile_id)
            .cloned()
            .or_else(|| item.profiles.first().cloned())
    }))
}
