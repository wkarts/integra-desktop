use crate::core::domain::document::{ConversionProfile, NfseDocument};
use crate::core::exporters::{csv::export_documents_to_csv, txt_fixed::export_documents_to_txt};

#[tauri::command]
pub fn export_nfse_txt(documents: Vec<NfseDocument>, profile: ConversionProfile) -> Result<String, String> {
    export_documents_to_txt(&documents, &profile).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_nfse_csv(documents: Vec<NfseDocument>, profile: ConversionProfile) -> Result<String, String> {
    export_documents_to_csv(&documents, &profile).map_err(|e| e.to_string())
}
