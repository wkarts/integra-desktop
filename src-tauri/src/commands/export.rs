use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use zip::{write::FileOptions, ZipArchive, ZipWriter};

use crate::core::domain::document::{ConversionProfile, NfseDocument};
use crate::core::exporters::{
    csv::export_documents_to_csv,
    standard_xml::{
        export_document_to_standard_xml, standardize_document_for_xml, StandardizeXmlOptions,
    },
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardizedXmlBatchEntry {
    pub source_name: String,
    pub output_file_name: String,
    pub detected_layout: String,
    pub provider: String,
    pub standardized_xml: String,
    pub warnings: Vec<String>,
    pub document: NfseDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardizedXmlBatchResult {
    pub entries: Vec<StandardizedXmlBatchEntry>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub zip_file_name: String,
    pub zip_base64: String,
}

#[tauri::command]
pub fn convert_nfse_xml_to_standard(
    xml: String,
    file_name: String,
    profile: Option<ConversionProfile>,
    options: StandardizeXmlOptions,
) -> Result<StandardizedXmlResult, String> {
    build_standardized_result(&xml, &file_name, profile.as_ref(), &options)
}

#[tauri::command]
pub fn convert_nfse_mixed_batch_to_standard(
    items: Vec<crate::commands::process::UploadInputItem>,
    paths: Vec<String>,
    profile: Option<ConversionProfile>,
    options: StandardizeXmlOptions,
    app: AppHandle,
) -> Result<StandardizedXmlBatchResult, String> {
    let mut entries = Vec::new();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for item in items {
        match item.kind.as_str() {
            "xml" => process_single_xml(
                &item.file_name,
                &item.content,
                profile.as_ref(),
                &options,
                &mut entries,
                &mut warnings,
                &mut errors,
            ),
            "zip" => process_zip_bytes(
                &item.file_name,
                &STANDARD
                    .decode(item.content.as_bytes())
                    .map_err(|e| format!("{}: ZIP inválido ({})", item.file_name, e))?,
                profile.as_ref(),
                &options,
                &mut entries,
                &mut warnings,
                &mut errors,
            ),
            other => errors.push(format!(
                "{}: tipo de entrada não suportado ({})",
                item.file_name, other
            )),
        }
    }

    for raw_path in paths {
        let path = PathBuf::from(&raw_path);
        if !path.exists() {
            errors.push(format!("{}: caminho não encontrado", raw_path));
            continue;
        }
        process_path(
            &path,
            profile.as_ref(),
            &options,
            &mut entries,
            &mut warnings,
            &mut errors,
        );
    }

    finalize_batch_result(entries, warnings, errors, &app)
}

#[tauri::command]
pub fn convert_nfse_upload_batch_to_standard(
    items: Vec<crate::commands::process::UploadInputItem>,
    profile: Option<ConversionProfile>,
    options: StandardizeXmlOptions,
    app: AppHandle,
) -> Result<StandardizedXmlBatchResult, String> {
    let mut entries = Vec::new();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for item in items {
        match item.kind.as_str() {
            "xml" => process_single_xml(
                &item.file_name,
                &item.content,
                profile.as_ref(),
                &options,
                &mut entries,
                &mut warnings,
                &mut errors,
            ),
            "zip" => process_zip_bytes(
                &item.file_name,
                &STANDARD
                    .decode(item.content.as_bytes())
                    .map_err(|e| format!("{}: ZIP inválido ({})", item.file_name, e))?,
                profile.as_ref(),
                &options,
                &mut entries,
                &mut warnings,
                &mut errors,
            ),
            other => errors.push(format!(
                "{}: tipo de entrada não suportado ({})",
                item.file_name, other
            )),
        }
    }

    finalize_batch_result(entries, warnings, errors, &app)
}

#[tauri::command]
pub fn convert_nfse_path_batch_to_standard(
    paths: Vec<String>,
    profile: Option<ConversionProfile>,
    options: StandardizeXmlOptions,
    app: AppHandle,
) -> Result<StandardizedXmlBatchResult, String> {
    let mut entries = Vec::new();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for raw_path in paths {
        let path = PathBuf::from(&raw_path);
        if !path.exists() {
            errors.push(format!("{}: caminho não encontrado", raw_path));
            continue;
        }
        process_path(
            &path,
            profile.as_ref(),
            &options,
            &mut entries,
            &mut warnings,
            &mut errors,
        );
    }

    finalize_batch_result(entries, warnings, errors, &app)
}

#[tauri::command]
pub fn dialog_pick_nfse_converter_files(app: AppHandle) -> Result<Vec<String>, String> {
    let paths = app
        .dialog()
        .file()
        .add_filter("XML e ZIP", &["xml", "zip"])
        .blocking_pick_files()
        .unwrap_or_default()
        .into_iter()
        .filter_map(file_path_to_string)
        .collect::<Vec<_>>();
    Ok(paths)
}

#[tauri::command]
pub fn dialog_pick_nfse_converter_directory(app: AppHandle) -> Result<Option<String>, String> {
    Ok(app
        .dialog()
        .file()
        .blocking_pick_folder()
        .and_then(file_path_to_string))
}

fn finalize_batch_result(
    entries: Vec<StandardizedXmlBatchEntry>,
    warnings: Vec<String>,
    errors: Vec<String>,
    app: &AppHandle,
) -> Result<StandardizedXmlBatchResult, String> {
    let zip_bytes = build_zip_bytes(&entries).map_err(|e| e.to_string())?;
    crate::storage::logs::append_log(
        app,
        &format!(
            "Conversão XML em lote concluída: {} arquivo(s), {} erro(s), {} aviso(s).",
            entries.len(),
            errors.len(),
            warnings.len()
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(StandardizedXmlBatchResult {
        entries,
        warnings,
        errors,
        zip_file_name: "nfse-portabilidade-lote.zip".to_string(),
        zip_base64: STANDARD.encode(zip_bytes),
    })
}

fn process_path(
    path: &Path,
    profile: Option<&ConversionProfile>,
    options: &StandardizeXmlOptions,
    entries: &mut Vec<StandardizedXmlBatchEntry>,
    warnings: &mut Vec<String>,
    errors: &mut Vec<String>,
) {
    if path.is_dir() {
        let read_dir = match fs::read_dir(path) {
            Ok(read_dir) => read_dir,
            Err(error) => {
                errors.push(format!(
                    "{}: falha ao ler diretório ({})",
                    path.display(),
                    error
                ));
                return;
            }
        };

        for child in read_dir.flatten() {
            process_path(&child.path(), profile, options, entries, warnings, errors);
        }
        return;
    }

    let lower = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match lower.as_str() {
        "xml" => match fs::read_to_string(path) {
            Ok(xml) => process_single_xml(
                &path.display().to_string(),
                &xml,
                profile,
                options,
                entries,
                warnings,
                errors,
            ),
            Err(error) => errors.push(format!("{}: falha ao ler XML ({})", path.display(), error)),
        },
        "zip" => match fs::read(path) {
            Ok(bytes) => process_zip_bytes(
                &path.display().to_string(),
                &bytes,
                profile,
                options,
                entries,
                warnings,
                errors,
            ),
            Err(error) => errors.push(format!("{}: falha ao ler ZIP ({})", path.display(), error)),
        },
        _ => {}
    }
}

fn process_zip_bytes(
    file_name: &str,
    bytes: &[u8],
    profile: Option<&ConversionProfile>,
    options: &StandardizeXmlOptions,
    entries: &mut Vec<StandardizedXmlBatchEntry>,
    warnings: &mut Vec<String>,
    errors: &mut Vec<String>,
) {
    let reader = Cursor::new(bytes.to_vec());
    let mut archive = match ZipArchive::new(reader) {
        Ok(archive) => archive,
        Err(error) => {
            errors.push(format!("{}: falha ao abrir ZIP ({})", file_name, error));
            return;
        }
    };

    for index in 0..archive.len() {
        let mut entry = match archive.by_index(index) {
            Ok(entry) => entry,
            Err(error) => {
                errors.push(format!("{}: falha no item do ZIP ({})", file_name, error));
                continue;
            }
        };

        let entry_name = entry.name().to_string();
        if !entry_name.to_ascii_lowercase().ends_with(".xml") {
            continue;
        }

        let mut xml = String::new();
        if let Err(error) = entry.read_to_string(&mut xml) {
            errors.push(format!(
                "{}#{}: conteúdo XML inválido ({})",
                file_name, entry_name, error
            ));
            continue;
        }

        let nested_file_name = format!("{}#{}", file_name, entry_name);
        process_single_xml(
            &nested_file_name,
            &xml,
            profile,
            options,
            entries,
            warnings,
            errors,
        );
    }
}

fn process_single_xml(
    file_name: &str,
    xml: &str,
    profile: Option<&ConversionProfile>,
    options: &StandardizeXmlOptions,
    entries: &mut Vec<StandardizedXmlBatchEntry>,
    warnings: &mut Vec<String>,
    errors: &mut Vec<String>,
) {
    match build_standardized_result(xml, file_name, profile, options) {
        Ok(result) => {
            warnings.extend(
                result
                    .warnings
                    .iter()
                    .map(|warning| format!("{}: {}", file_name, warning)),
            );
            entries.push(StandardizedXmlBatchEntry {
                source_name: file_name.to_string(),
                output_file_name: build_output_file_name(
                    file_name,
                    &options.target,
                    &result.detected_layout,
                ),
                detected_layout: result.detected_layout,
                provider: result.provider,
                standardized_xml: result.standardized_xml,
                warnings: result.warnings,
                document: result.document,
            });
        }
        Err(error) => errors.push(format!("{}: {}", file_name, error)),
    }
}

fn build_standardized_result(
    xml: &str,
    file_name: &str,
    profile: Option<&ConversionProfile>,
    options: &StandardizeXmlOptions,
) -> Result<StandardizedXmlResult, String> {
    let configured_layout = profile.as_ref().map(|item| item.nfse_layout.as_str());
    let parsed =
        parse_nfse_xml_with_layout(xml, file_name, configured_layout).map_err(|e| e.to_string())?;
    let mut document = normalize_nfse_document(parsed);
    document.warnings = collect_document_warnings(&document);
    let target = resolve_target_layout(options, &document.layout);
    let mut options = options.clone();
    options.target = target;
    let standardized_document = standardize_document_for_xml(&document, profile, &options);
    let standardized_xml = export_document_to_standard_xml(&standardized_document, &options);

    Ok(StandardizedXmlResult {
        detected_layout: document.layout.clone(),
        provider: document.provider.clone(),
        standardized_xml,
        warnings: document.warnings.clone(),
        document: standardized_document,
    })
}

fn resolve_target_layout(options: &StandardizeXmlOptions, detected_layout: &str) -> String {
    if options.target != "same_layout" {
        return options.target.clone();
    }

    match detected_layout {
        layout if layout.contains("v1") => "abrasf_v1".to_string(),
        "ubaira_custom" => "salvador_like".to_string(),
        _ => "abrasf_v2".to_string(),
    }
}

fn build_output_file_name(source_name: &str, target: &str, detected_layout: &str) -> String {
    let base = Path::new(source_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("nfse")
        .replace('#', "_")
        .replace(['/', '\\'], "_");
    let effective_target = if target == "same_layout" {
        format!("same-{}", detected_layout)
    } else {
        target.to_string()
    };
    format!("{}_{}.xml", base, effective_target)
}

fn build_zip_bytes(entries: &[StandardizedXmlBatchEntry]) -> anyhow::Result<Vec<u8>> {
    let cursor = Cursor::new(Vec::<u8>::new());
    let mut writer = ZipWriter::new(cursor);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in entries {
        writer.start_file(&entry.output_file_name, options)?;
        writer.write_all(entry.standardized_xml.as_bytes())?;
    }

    Ok(writer.finish()?.into_inner())
}

fn file_path_to_string(path: tauri_plugin_dialog::FilePath) -> Option<String> {
    match path {
        tauri_plugin_dialog::FilePath::Path(path_buf) => {
            Some(path_buf.to_string_lossy().to_string())
        }
        #[cfg(target_os = "android")]
        tauri_plugin_dialog::FilePath::Uri(uri) => Some(uri.to_string()),
        #[cfg(not(target_os = "android"))]
        _ => None,
    }
}
