use serde::{Deserialize, Serialize};

use crate::core::domain::document::{ConversionProfile, NfseDocument};
use crate::core::exporters::{
    csv::export_documents_to_csv,
    standard_xml::{export_document_to_standard_xml, StandardizeXmlOptions},
    txt_fixed::export_documents_to_txt,
};
use crate::core::normalizers::nfse_normalizer::normalize_nfse_document;
use crate::core::parsers::nfse::parse_nfse_xml_with_layout;
use crate::core::validation::warnings::collect_document_warnings;

#[tauri::command]
pub fn export_nfse_txt(
    documents: Vec<NfseDocument>,
    profile: ConversionProfile,
) -> Result<String, String> {
    export_documents_to_txt(&documents, &profile).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_nfse_csv(
    documents: Vec<NfseDocument>,
    profile: ConversionProfile,
) -> Result<String, String> {
    export_documents_to_csv(&documents, &profile).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardizedXmlResult {
    pub detected_layout: String,
    pub provider: String,
    pub standardized_xml: String,
    pub warnings: Vec<String>,
    pub document: NfseDocument,
}

#[tauri::command]
pub fn convert_nfse_xml_to_standard(
    xml: String,
    file_name: String,
    profile: Option<ConversionProfile>,
    options: StandardizeXmlOptions,
) -> Result<StandardizedXmlResult, String> {
    let configured_layout = profile.as_ref().map(|item| item.nfse_layout.as_str());
    let parsed = parse_nfse_xml_with_layout(&xml, &file_name, configured_layout)
        .map_err(|e| e.to_string())?;
    let mut document = normalize_nfse_document(parsed);
    document.warnings = collect_document_warnings(&document);
    let standardized_xml = export_document_to_standard_xml(&document, &options);

    Ok(StandardizedXmlResult {
        detected_layout: document.layout.clone(),
        provider: document.provider.clone(),
        standardized_xml,
        warnings: document.warnings.clone(),
        document,
    })
}
