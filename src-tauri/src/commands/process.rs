use tauri::AppHandle;

use crate::core::{
    normalizers::nfse_normalizer::normalize_nfse_document,
    parsers::nfse::parse_nfse_xml,
    validation::warnings::collect_document_warnings,
};
use crate::core::domain::document::{NfseDocument, ProcessBatchInputItem, ProcessBatchResult};

#[tauri::command]
pub fn process_nfse_xml_batch(items: Vec<ProcessBatchInputItem>, app: AppHandle) -> Result<ProcessBatchResult, String> {
    let mut documents: Vec<NfseDocument> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for item in items {
        match parse_nfse_xml(&item.xml, &item.file_name) {
            Ok(document) => {
                let mut normalized = normalize_nfse_document(document);
                normalized.warnings = collect_document_warnings(&normalized);
                warnings.extend(normalized.warnings.clone().into_iter().map(|message| format!("{}: {}", normalized.file_name, message)));
                documents.push(normalized);
            }
            Err(error) => {
                errors.push(format!("{}: {}", item.file_name, error));
            }
        }
    }

    crate::storage::logs::append_log(&app, &format!("Processados {} XML(s), {} erro(s).", documents.len(), errors.len()))
        .map_err(|e| e.to_string())?;

    Ok(ProcessBatchResult { documents, warnings, errors })
}
