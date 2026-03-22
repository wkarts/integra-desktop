use std::io::{Cursor, Read};

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use zip::ZipArchive;

use crate::core::domain::document::{
    ConversionProfile, NfseDocument, ProcessBatchInputItem, ProcessBatchResult,
};
use crate::core::{
    normalizers::nfse_normalizer::normalize_nfse_document,
    parsers::nfse::parse_nfse_xml_with_layout, validation::warnings::collect_document_warnings,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadInputItem {
    pub file_name: String,
    pub kind: String,
    pub content: String,
}

fn process_xml(
    file_name: &str,
    xml: &str,
    profile: Option<&ConversionProfile>,
    documents: &mut Vec<NfseDocument>,
    warnings: &mut Vec<String>,
    errors: &mut Vec<String>,
) {
    let configured_layout = profile.map(|item| item.nfse_layout.as_str());
    match parse_nfse_xml_with_layout(xml, file_name, configured_layout) {
        Ok(document) => {
            let mut normalized = normalize_nfse_document(document);
            normalized.warnings = collect_document_warnings(&normalized);
            if let Some(item) = profile {
                let profile_city = item.company_municipio_nome.trim();
                if !profile_city.is_empty()
                    && !profile_city.eq_ignore_ascii_case(&normalized.municipio_nome)
                {
                    normalized.warnings.push(format!(
                        "Município do XML ({}) difere do município configurado na empresa ({}).",
                        normalized.municipio_nome, profile_city
                    ));
                }
            }
            warnings.extend(
                normalized
                    .warnings
                    .clone()
                    .into_iter()
                    .map(|message| format!("{}: {}", normalized.file_name, message)),
            );
            documents.push(normalized);
        }
        Err(error) => {
            errors.push(format!("{}: {}", file_name, error));
        }
    }
}

#[tauri::command]
pub fn process_nfse_xml_batch(
    items: Vec<ProcessBatchInputItem>,
    profile: Option<ConversionProfile>,
    app: AppHandle,
) -> Result<ProcessBatchResult, String> {
    let mut documents: Vec<NfseDocument> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for item in items {
        process_xml(
            &item.file_name,
            &item.xml,
            profile.as_ref(),
            &mut documents,
            &mut warnings,
            &mut errors,
        );
    }

    crate::storage::logs::append_log(
        &app,
        &format!(
            "Processados {} XML(s), {} erro(s).",
            documents.len(),
            errors.len()
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(ProcessBatchResult {
        documents,
        warnings,
        errors,
    })
}

#[tauri::command]
pub fn process_nfse_upload_batch(
    items: Vec<UploadInputItem>,
    profile: Option<ConversionProfile>,
    app: AppHandle,
) -> Result<ProcessBatchResult, String> {
    let mut documents: Vec<NfseDocument> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for item in items {
        match item.kind.as_str() {
            "xml" => process_xml(
                &item.file_name,
                &item.content,
                profile.as_ref(),
                &mut documents,
                &mut warnings,
                &mut errors,
            ),
            "zip" => {
                let zip_bytes = STANDARD
                    .decode(item.content.as_bytes())
                    .map_err(|e| format!("{}: ZIP inválido ({})", item.file_name, e))?;
                let reader = Cursor::new(zip_bytes);
                let mut archive = ZipArchive::new(reader)
                    .map_err(|e| format!("{}: Falha ao abrir ZIP ({})", item.file_name, e))?;

                for index in 0..archive.len() {
                    let mut entry = archive
                        .by_index(index)
                        .map_err(|e| format!("{}: Falha no item do ZIP ({})", item.file_name, e))?;

                    let entry_name = entry.name().to_string();
                    if !entry_name.to_ascii_lowercase().ends_with(".xml") {
                        continue;
                    }

                    let mut xml = String::new();
                    if let Err(error) = entry.read_to_string(&mut xml) {
                        errors.push(format!(
                            "{}#{}: conteúdo XML inválido ({})",
                            item.file_name, entry_name, error
                        ));
                        continue;
                    }

                    let nested_file_name = format!("{}#{}", item.file_name, entry_name);
                    process_xml(
                        &nested_file_name,
                        &xml,
                        profile.as_ref(),
                        &mut documents,
                        &mut warnings,
                        &mut errors,
                    );
                }
            }
            _ => errors.push(format!(
                "{}: tipo de entrada não suportado ({})",
                item.file_name, item.kind
            )),
        }
    }

    crate::storage::logs::append_log(
        &app,
        &format!(
            "Lote combinado concluído: {} documento(s), {} erro(s), {} aviso(s).",
            documents.len(),
            errors.len(),
            warnings.len()
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(ProcessBatchResult {
        documents,
        warnings,
        errors,
    })
}
