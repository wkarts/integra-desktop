use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use chrono::Local;
use roxmltree::{Document, Node};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::{DialogExt, FilePath, MessageDialogKind};
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NfeFaturasSettings {
    pub cod_prosoft: String,
    pub chk_append_d: bool,
    pub chk_forcar_duas_linhas: bool,
    pub chk_usar_sped: bool,
    pub chk_ie_sped_padrao: bool,
    pub chk_somente_com_sped: bool,
    pub chk_incluir_sem_dup: bool,
    pub chk_venc30: bool,
    pub chk_incluir_faturas_sped: bool,
    pub chk_recriar_c140_c141: bool,
    pub sel_modo_parcelas: String,
    pub num_qtd_parcelas_geral: u32,
    pub txt_regras_fornecedor_parcelas: String,
    pub num_venc_intervalo_dias: i32,
    pub txt_venc_dias_por_parcela: String,
    pub sel_multi_sped_modo: String,
    pub sel_consolidacao_interna_nfe: String,
    pub chk_consolidar_cnpj: bool,
    pub txt_cnpjs_consolidar: String,
    pub origem: String,
    pub tipo: String,
    pub serie_digits: String,
    pub ctx_nota: String,
    pub chk_exportar_filtrados: bool,
}

impl Default for NfeFaturasSettings {
    fn default() -> Self {
        Self {
            cod_prosoft: String::new(),
            chk_append_d: false,
            chk_forcar_duas_linhas: false,
            chk_usar_sped: true,
            chk_ie_sped_padrao: false,
            chk_somente_com_sped: true,
            chk_incluir_sem_dup: false,
            chk_venc30: true,
            chk_incluir_faturas_sped: false,
            chk_recriar_c140_c141: false,
            sel_modo_parcelas: "respeitar_xml".into(),
            num_qtd_parcelas_geral: 1,
            txt_regras_fornecedor_parcelas: String::new(),
            num_venc_intervalo_dias: 30,
            txt_venc_dias_por_parcela: String::new(),
            sel_multi_sped_modo: "atualizar_individual".into(),
            sel_consolidacao_interna_nfe: "nao".into(),
            chk_consolidar_cnpj: false,
            txt_cnpjs_consolidar: String::new(),
            origem: "0".into(),
            tipo: "1".into(),
            serie_digits: "0".into(),
            ctx_nota: "entrada".into(),
            chk_exportar_filtrados: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NfeFaturasRow {
    pub chave: String,
    pub desdob: u8,
    pub cnpj_filial: String,
    pub cnpj_cpf: String,
    pub uf: String,
    pub ie: String,
    pub nf_serie: String,
    pub nf_numero: String,
    pub data_emissao: String,
    pub data_entrada: String,
    pub num_fatura: String,
    pub data_vencimento: String,
    pub valor_bruto_fat: String,
    pub source: String,
    pub sped_matched: bool,
    pub legado: bool,
    pub consolidated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NfeMetaDup {
    pub n_dup: String,
    pub d_venc: String,
    pub v_dup: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NfeMeta {
    pub chave: String,
    pub n_fat: String,
    pub v_orig: String,
    pub v_liq: String,
    pub v_nf: String,
    pub dup_list: Vec<NfeMetaDup>,
    pub nf_numero: String,
    pub serie: String,
    pub cnpj_terceiro: String,
    pub dt_emi: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NamedText {
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NfeFaturasCounts {
    pub xml: usize,
    pub sped: usize,
    pub zip: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NfeFaturasLogItem {
    pub ts: String,
    pub level: String,
    pub msg: String,
    pub meta: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NfeFaturasProcessResult {
    pub rows: Vec<NfeFaturasRow>,
    pub counts: NfeFaturasCounts,
    pub logs: Vec<NfeFaturasLogItem>,
    pub sped_files: Vec<NamedText>,
    pub nfe_metas: Vec<NfeMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NfeFaturasLegacyPreview {
    pub notas: usize,
    pub parcelas: usize,
    pub invalid_count: usize,
    pub invalid_lines: Vec<String>,
    pub warning_count: usize,
    pub warnings: Vec<String>,
    pub divergences: Vec<String>,
    pub divergence_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NfeFaturasLegacyResult {
    pub rows: Vec<NfeFaturasRow>,
    pub preview: NfeFaturasLegacyPreview,
    pub logs: Vec<NfeFaturasLogItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NfeFaturasExportResult {
    pub output_paths: Vec<String>,
    pub lines: usize,
    pub records: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
struct SpedC100Item {
    dt_escr: String,
    desdob: u8,
}

#[derive(Debug, Clone)]
struct VirtualEntry {
    name: String,
    kind: String,
    text: String,
}

#[derive(Debug, Clone)]
struct LegacyDup {
    n_dup: String,
    d_venc: String,
    v_dup: f64,
}

#[derive(Debug, Clone)]
struct LegacyGroup {
    cnpj: String,
    uf: String,
    ie: String,
    serie: String,
    nf_numero: String,
    dt_escrit: String,
    total_nf: f64,
    duplicatas: Vec<LegacyDup>,
}

#[derive(Debug, Clone)]
struct LegacyStats {
    rows: Vec<NfeFaturasRow>,
    notes: usize,
    parcelas: usize,
    invalid_count: usize,
    invalid_lines: Vec<String>,
    warning_count: usize,
    warnings: Vec<String>,
    divergences: Vec<String>,
    divergence_count: usize,
}

#[derive(Debug, Clone)]
struct ExportLineItem {
    row: NfeFaturasRow,
    item3: String,
}

fn file_path_to_string(path: FilePath) -> Option<String> {
    path.into_path()
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn dialog_pick_nfe_faturas_files(app: AppHandle) -> Result<Vec<String>, String> {
    let paths = app
        .dialog()
        .file()
        .add_filter("Entradas fiscais", &["xml", "zip", "txt", "sped", "csv"])
        .blocking_pick_files()
        .unwrap_or_default()
        .into_iter()
        .filter_map(file_path_to_string)
        .collect::<Vec<_>>();
    Ok(paths)
}

#[tauri::command]
pub fn dialog_pick_nfe_faturas_directory(app: AppHandle) -> Result<Option<String>, String> {
    Ok(app
        .dialog()
        .file()
        .blocking_pick_folder()
        .and_then(file_path_to_string))
}

#[tauri::command]
pub fn dialog_pick_nfe_faturas_legacy_file(app: AppHandle) -> Result<Option<String>, String> {
    Ok(app
        .dialog()
        .file()
        .add_filter("TXT/CSV legado", &["txt", "csv"])
        .blocking_pick_file()
        .and_then(file_path_to_string))
}

#[tauri::command]
pub fn dialog_pick_nfe_faturas_output_dir(app: AppHandle) -> Result<Option<String>, String> {
    Ok(app
        .dialog()
        .file()
        .blocking_pick_folder()
        .and_then(file_path_to_string))
}

#[tauri::command]
pub fn dialog_save_nfe_faturas_file(
    default_name: String,
    title: String,
    extensions: Vec<String>,
    app: AppHandle,
) -> Result<Option<String>, String> {
    let ext_refs = extensions.iter().map(|v| v.as_str()).collect::<Vec<_>>();
    Ok(app
        .dialog()
        .file()
        .set_title(title)
        .set_file_name(default_name)
        .add_filter("Arquivo", &ext_refs)
        .blocking_save_file()
        .and_then(file_path_to_string))
}

#[tauri::command]
pub fn dialog_message_info(title: String, message: String, app: AppHandle) -> Result<(), String> {
    app.dialog()
        .message(message)
        .title(title)
        .kind(MessageDialogKind::Info)
        .blocking_show();
    Ok(())
}

#[tauri::command]
pub fn dialog_message_warning(
    title: String,
    message: String,
    app: AppHandle,
) -> Result<(), String> {
    app.dialog()
        .message(message)
        .title(title)
        .kind(MessageDialogKind::Warning)
        .blocking_show();
    Ok(())
}

#[tauri::command]
pub fn dialog_message_error(title: String, message: String, app: AppHandle) -> Result<(), String> {
    app.dialog()
        .message(message)
        .title(title)
        .kind(MessageDialogKind::Error)
        .blocking_show();
    Ok(())
}

#[tauri::command]
pub fn dialog_confirm(title: String, message: String, app: AppHandle) -> Result<bool, String> {
    Ok(app
        .dialog()
        .message(message)
        .title(title)
        .kind(MessageDialogKind::Warning)
        .blocking_show())
}

#[tauri::command]
pub fn clipboard_write_text(text: String, app: AppHandle) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_nfe_faturas_settings(app: AppHandle) -> Result<NfeFaturasSettings, String> {
    Ok(
        crate::storage::nfe_faturas_settings::load_nfe_faturas_settings(&app)
            .map_err(|e| e.to_string())?
            .unwrap_or_default(),
    )
}

#[tauri::command]
pub fn save_nfe_faturas_settings(
    settings: NfeFaturasSettings,
    app: AppHandle,
) -> Result<(), String> {
    crate::storage::nfe_faturas_settings::save_nfe_faturas_settings(&app, &settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn process_nfe_faturas_selection(
    paths: Vec<String>,
    settings: NfeFaturasSettings,
    app: AppHandle,
) -> Result<NfeFaturasProcessResult, String> {
    let mut logs = Vec::<NfeFaturasLogItem>::new();
    let collected = collect_input_files(&paths, &mut logs).map_err(|e| e.to_string())?;
    let counts = count_input_files(&collected);

    let mut virtual_entries = Vec::<VirtualEntry>::new();
    for path in &collected {
        let kind = classify_kind_from_name(
            path.file_name()
                .and_then(|v| v.to_str())
                .unwrap_or_default(),
        );
        if kind == "zip" {
            match build_virtual_entries_from_zip(path) {
                Ok(mut entries) => virtual_entries.append(&mut entries),
                Err(error) => log_push(
                    &mut logs,
                    "error",
                    "Falha ao abrir ZIP",
                    Some(format!("{} | {}", path.display(), error)),
                ),
            }
        } else {
            match fs::read(path) {
                Ok(bytes) => {
                    let text = decode_text(&bytes);
                    let name = path.to_string_lossy().to_string();
                    virtual_entries.push(VirtualEntry { name, kind, text });
                }
                Err(error) => log_push(
                    &mut logs,
                    "error",
                    "Falha ao ler arquivo",
                    Some(format!("{} | {}", path.display(), error)),
                ),
            }
        }
    }

    let mut sped_files = Vec::<NamedText>::new();
    let mut xml_entries = Vec::<VirtualEntry>::new();

    for entry in virtual_entries {
        let mut kind = entry.kind.clone();
        if kind == "unknown" {
            if looks_like_xml(&entry.text) {
                kind = "xml".into();
            } else if looks_like_sped(&entry.text) {
                kind = "sped".into();
            }
        }

        if kind == "xml" {
            xml_entries.push(entry);
        } else if kind == "sped" {
            sped_files.push(NamedText {
                name: entry.name,
                text: entry.text,
            });
        } else {
            log_push(
                &mut logs,
                "warn",
                "Arquivo ignorado (tipo não reconhecido)",
                Some(entry.name),
            );
        }
    }

    let mut sped_by_chave = HashMap::<String, SpedC100Item>::new();
    let mut sped_0150_by_doc = HashMap::<String, String>::new();
    for sped in &sped_files {
        let c100 = parse_sped_c100(&sped.text);
        for (chave, item) in c100 {
            sped_by_chave.entry(chave).or_insert(item);
        }
        let map_0150 = parse_sped_0150(&sped.text);
        for (doc, ie) in map_0150 {
            sped_0150_by_doc.insert(doc, ie);
        }
    }
    if !sped_by_chave.is_empty() {
        log_push(
            &mut logs,
            "info",
            "SPED (C100) indexado",
            Some(format!("chaves: {}", sped_by_chave.len())),
        );
    }
    if !sped_0150_by_doc.is_empty() {
        log_push(
            &mut logs,
            "info",
            "SPED (0150) indexado",
            Some(format!("docs: {}", sped_0150_by_doc.len())),
        );
    }

    let mut rows = Vec::<NfeFaturasRow>::new();
    let mut nfe_metas = Vec::<NfeMeta>::new();

    for entry in &xml_entries {
        match parse_nfe_meta_and_rows(&entry.text, &entry.name, &settings) {
            Ok((meta, mut parsed_rows)) => {
                nfe_metas.push(meta);
                if parsed_rows.is_empty() {
                    log_push(
                        &mut logs,
                        "warn",
                        "XML sem duplicatas (ignorado, a menos que opção esteja marcada)",
                        Some(entry.name.clone()),
                    );
                } else {
                    rows.append(&mut parsed_rows);
                }
            }
            Err(error) => log_push(
                &mut logs,
                "error",
                "Erro ao processar XML",
                Some(format!("{} | {}", entry.name, error)),
            ),
        }
    }

    apply_sped_c100_to_rows(&mut rows, &sped_by_chave, &settings);
    apply_sped_0150_ie_to_rows(&mut rows, &sped_0150_by_doc, &settings, &mut logs);

    crate::storage::logs::append_log(
        &app,
        &format!(
            "NFe/Faturas: {} registro(s), {} XML, {} SPED, {} ZIP.",
            rows.len(),
            counts.xml,
            counts.sped,
            counts.zip
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(NfeFaturasProcessResult {
        rows,
        counts,
        logs,
        sped_files,
        nfe_metas,
    })
}

#[tauri::command]
pub fn import_nfe_faturas_legacy(
    file_path: String,
    cnpj_filial: Option<String>,
    conferir: bool,
    current_rows: Vec<NfeFaturasRow>,
    sped_files: Vec<NamedText>,
    nfe_metas: Vec<NfeMeta>,
) -> Result<NfeFaturasLegacyResult, String> {
    let mut logs = Vec::<NfeFaturasLogItem>::new();
    let content = fs::read(&file_path).map_err(|e| e.to_string())?;
    let text = decode_text(&content);
    let auto_cnpj = if let Some(cnpj) = cnpj_filial.as_deref() {
        only_digits(cnpj)
    } else {
        guess_cnpj_filial(&current_rows, &sped_files).unwrap_or_default()
    };

    let stats = parse_legacy_pipe_file(
        &text,
        Path::new(&file_path)
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("legado.txt"),
        conferir,
        if auto_cnpj.is_empty() {
            None
        } else {
            Some(auto_cnpj.as_str())
        },
        &sped_files,
        &nfe_metas,
        &mut logs,
    )
    .map_err(|e| e.to_string())?;

    Ok(NfeFaturasLegacyResult {
        rows: stats.rows,
        preview: NfeFaturasLegacyPreview {
            notas: stats.notes,
            parcelas: stats.parcelas,
            invalid_count: stats.invalid_count,
            invalid_lines: stats.invalid_lines,
            warning_count: stats.warning_count,
            warnings: stats.warnings,
            divergences: stats.divergences,
            divergence_count: stats.divergence_count,
        },
        logs,
    })
}

#[tauri::command]
pub fn guess_nfe_faturas_cnpj_filial(
    rows: Vec<NfeFaturasRow>,
    sped_files: Vec<NamedText>,
) -> Result<Option<String>, String> {
    Ok(guess_cnpj_filial(&rows, &sped_files))
}

#[tauri::command]
pub fn export_nfe_faturas_txt(
    rows: Vec<NfeFaturasRow>,
    settings: NfeFaturasSettings,
    output_path: String,
    app: AppHandle,
) -> Result<NfeFaturasExportResult, String> {
    let export_rows = compute_export_rows_base(&rows, &settings);
    if export_rows.is_empty() {
        return Err("Nada para exportar (verifique SPED / filtros).".into());
    }
    let lines = compute_export_lines(&export_rows, &settings)
        .into_iter()
        .map(|item| build_txt_line(&item.row, &item.item3, &settings))
        .collect::<Vec<_>>();
    let content = format!("{}\r\n", lines.join("\r\n"));
    write_text_file(Path::new(&output_path), &content)?;
    crate::storage::logs::append_log(&app, &format!("TXT exportado: {}", output_path))
        .map_err(|e| e.to_string())?;
    Ok(NfeFaturasExportResult {
        output_paths: vec![output_path.clone()],
        lines: lines.len(),
        records: export_rows.len(),
        message: format!("TXT exportado: {}", output_path),
    })
}

#[tauri::command]
pub fn export_nfe_faturas_csv(
    rows: Vec<NfeFaturasRow>,
    settings: NfeFaturasSettings,
    output_path: String,
    app: AppHandle,
) -> Result<NfeFaturasExportResult, String> {
    let export_rows = compute_export_rows_base(&rows, &settings);
    if export_rows.is_empty() {
        return Err("Nada para exportar (verifique SPED / filtros).".into());
    }
    let line_items = compute_export_lines(&export_rows, &settings);
    let mut content = String::new();
    content.push_str("ITEM;CHAVE;DESDOB;CNPJFILIAL;CNPJCPF;UF;IE;NFSERIE;NFNUMERO;DATAEMISSAO;DATAENTRADA;NUMFATURA;DATAVENCIMENTO;VALORBRUTOFAT;FONTE;SPED\r\n");
    for item in &line_items {
        let sped = if item.row.sped_matched {
            "MATCH"
        } else {
            "MISSING"
        };
        let values = [
            item.item3.clone(),
            item.row.chave.clone(),
            item.row.desdob.to_string(),
            item.row.cnpj_filial.clone(),
            item.row.cnpj_cpf.clone(),
            item.row.uf.clone(),
            item.row.ie.clone(),
            item.row.nf_serie.clone(),
            item.row.nf_numero.clone(),
            item.row.data_emissao.clone(),
            item.row.data_entrada.clone(),
            item.row.num_fatura.clone(),
            item.row.data_vencimento.clone(),
            to_money2(&item.row.valor_bruto_fat),
            item.row.source.clone(),
            sped.into(),
        ];
        let line = values
            .iter()
            .map(|v| csv_escape(v))
            .collect::<Vec<_>>()
            .join(";");
        content.push_str(&line);
        content.push_str("\r\n");
    }
    write_text_file(Path::new(&output_path), &content)?;
    crate::storage::logs::append_log(&app, &format!("CSV exportado: {}", output_path))
        .map_err(|e| e.to_string())?;
    Ok(NfeFaturasExportResult {
        output_paths: vec![output_path.clone()],
        lines: line_items.len(),
        records: export_rows.len(),
        message: format!("CSV exportado: {}", output_path),
    })
}

#[tauri::command]
pub fn export_nfe_faturas_sped(
    rows: Vec<NfeFaturasRow>,
    settings: NfeFaturasSettings,
    sped_files: Vec<NamedText>,
    nfe_metas: Vec<NfeMeta>,
    output_dir: String,
    app: AppHandle,
) -> Result<NfeFaturasExportResult, String> {
    if settings.ctx_nota != "entrada" {
        return Err("Exportação SPED só disponível para Entrada/Compra.".into());
    }
    if !settings.chk_incluir_faturas_sped {
        return Err("Marque 'Incluir faturas no SPED' para gerar C140/C141.".into());
    }
    if sped_files.is_empty() {
        return Err("Nenhum SPED carregado.".into());
    }
    if rows.is_empty() {
        return Err("Nenhum dado de XML processado.".into());
    }

    let meta_map = nfe_metas
        .into_iter()
        .filter(|item| !item.chave.is_empty())
        .map(|item| (item.chave.clone(), item))
        .collect::<HashMap<_, _>>();

    let opts = SpedExportOptions::from_settings(&settings);
    let mode = settings.sel_multi_sped_modo.clone();
    let output_root = PathBuf::from(output_dir);
    fs::create_dir_all(&output_root).map_err(|e| e.to_string())?;

    let mut outputs = Vec::<(String, String, UpdateSpedStats)>::new();
    for sped in &sped_files {
        let updated = update_sped_text_with_c140_c141(&sped.text, &sped.name, &opts, &meta_map)?;
        let name = format!("{}_C140C141.txt", file_stem(&sped.name));
        outputs.push((name, updated.text, updated.stats));
    }

    let mut final_outputs = Vec::<(String, String)>::new();
    let can_consolidate = if mode == "atualizar_individual" {
        false
    } else {
        can_consolidate_sped_files(&sped_files)
    };

    if (mode == "consolidar" || mode == "misto") && can_consolidate && outputs.len() >= 2 {
        if let Some((name, text)) = consolidate_sped_outputs(&outputs) {
            if mode == "consolidar" {
                final_outputs.push((name, text));
            } else {
                for (n, t, _) in &outputs {
                    final_outputs.push((n.clone(), t.clone()));
                }
                final_outputs.push((name, text));
            }
        } else {
            for (n, t, _) in &outputs {
                final_outputs.push((n.clone(), t.clone()));
            }
        }
    } else {
        for (n, t, _) in &outputs {
            final_outputs.push((n.clone(), t.clone()));
        }
    }

    let mut written = Vec::<String>::new();
    for (name, text) in final_outputs {
        let file = output_root.join(&name);
        write_text_file(&file, &text)?;
        written.push(file.to_string_lossy().to_string());
    }

    crate::storage::logs::append_log(
        &app,
        &format!("SPED atualizado exportado: {} arquivo(s)", written.len()),
    )
    .map_err(|e| e.to_string())?;

    Ok(NfeFaturasExportResult {
        output_paths: written.clone(),
        lines: 0,
        records: rows.len(),
        message: format!("SPED atualizado exportado: {} arquivo(s).", written.len()),
    })
}

fn now_iso() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn log_push(logs: &mut Vec<NfeFaturasLogItem>, level: &str, msg: &str, meta: Option<String>) {
    logs.insert(
        0,
        NfeFaturasLogItem {
            ts: now_iso(),
            level: level.to_string(),
            msg: msg.to_string(),
            meta,
        },
    );
    if logs.len() > 500 {
        logs.truncate(500);
    }
}

fn collect_input_files(
    paths: &[String],
    logs: &mut Vec<NfeFaturasLogItem>,
) -> Result<Vec<PathBuf>> {
    let mut out = Vec::<PathBuf>::new();
    let mut seen = HashSet::<String>::new();
    for raw in paths {
        let path = PathBuf::from(raw);
        if !path.exists() {
            log_push(
                logs,
                "warn",
                "Caminho ignorado (não encontrado)",
                Some(raw.clone()),
            );
            continue;
        }
        if path.is_dir() {
            collect_dir_recursively(&path, &mut out, &mut seen)?;
        } else {
            let key = path.to_string_lossy().to_string();
            if seen.insert(key) {
                out.push(path);
            }
        }
    }
    Ok(out)
}

fn collect_dir_recursively(
    path: &Path,
    out: &mut Vec<PathBuf>,
    seen: &mut HashSet<String>,
) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        if child.is_dir() {
            collect_dir_recursively(&child, out, seen)?;
        } else {
            let key = child.to_string_lossy().to_string();
            if seen.insert(key) {
                out.push(child);
            }
        }
    }
    Ok(())
}

fn count_input_files(paths: &[PathBuf]) -> NfeFaturasCounts {
    let mut counts = NfeFaturasCounts::default();
    for path in paths {
        match classify_kind_from_name(
            path.file_name()
                .and_then(|v| v.to_str())
                .unwrap_or_default(),
        )
        .as_str()
        {
            "xml" => counts.xml += 1,
            "zip" => counts.zip += 1,
            "sped" => counts.sped += 1,
            _ => {}
        }
    }
    counts
}

fn build_virtual_entries_from_zip(path: &Path) -> Result<Vec<VirtualEntry>> {
    let bytes = fs::read(path)?;
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)?;
    let mut entries = Vec::<VirtualEntry>::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        if entry.is_dir() {
            continue;
        }
        let mut data = Vec::<u8>::new();
        entry.read_to_end(&mut data)?;
        let text = decode_text(&data);
        let name = format!("{}::{}", path.to_string_lossy(), entry.name());
        let kind = classify_kind_from_name(entry.name());
        entries.push(VirtualEntry { name, kind, text });
    }
    Ok(entries)
}

fn classify_kind_from_name(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".xml") {
        "xml".into()
    } else if lower.ends_with(".zip") {
        "zip".into()
    } else if lower.ends_with(".sped") || lower.ends_with(".txt") {
        "sped".into()
    } else if lower.ends_with(".csv") {
        "csv".into()
    } else {
        "unknown".into()
    }
}

fn decode_text(bytes: &[u8]) -> String {
    match String::from_utf8(bytes.to_vec()) {
        Ok(s) => s,
        Err(_) => bytes.iter().map(|b| *b as char).collect(),
    }
}

fn looks_like_xml(text: &str) -> bool {
    let t = text.to_ascii_lowercase();
    t.contains("<nfe") || t.contains("<infnfe") || t.contains("<nfeproc")
}

fn looks_like_sped(text: &str) -> bool {
    text.contains("|C100|") || text.contains("|0150|") || text.contains("|0000|")
}

fn only_digits(value: &str) -> String {
    value.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn only_alphanum_upper(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

fn parse_money(value: &str) -> Option<f64> {
    let original = value.trim();
    if original.is_empty() {
        return None;
    }
    let mut s = original
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    if s.contains(',') && s.contains('.') {
        s = s.replace('.', "").replace(',', ".");
    } else if s.contains(',') {
        s = s.replace(',', ".");
    } else if s.matches('.').count() > 1 {
        let mut parts = s.split('.').collect::<Vec<_>>();
        let last = parts.pop().unwrap_or_default();
        s = format!("{}.{}", parts.join(""), last);
    }
    let filtered = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect::<String>();
    filtered.parse::<f64>().ok()
}

fn to_money2(value: &str) -> String {
    format!("{:.2}", parse_money(value).unwrap_or(0.0))
}

fn to_sped_money(value: f64) -> String {
    format!("{:.2}", value).replace('.', ",")
}

fn ddmmyyyy_to_digits(value: &str) -> String {
    let s = value.trim();
    if s.is_empty() {
        return String::new();
    }
    if let Some(rest) = s.get(0..10) {
        let bytes = rest.as_bytes();
        if bytes.len() == 10 && bytes[4] == b'-' && bytes[7] == b'-' {
            return format!("{}{}{}", &rest[8..10], &rest[5..7], &rest[0..4]);
        }
    }
    let digits = only_digits(s);
    if digits.len() >= 8 {
        digits[0..8].to_string()
    } else {
        digits
    }
}

fn digits_to_br_date(value: &str) -> String {
    let digits = only_digits(value);
    if digits.len() != 8 {
        return String::new();
    }
    format!("{}/{}/{}", &digits[0..2], &digits[2..4], &digits[4..8])
}

fn br_date_to_time(value: &str) -> Option<i64> {
    let digits = only_digits(value);
    if digits.len() != 8 {
        return None;
    }
    let day = digits[0..2].parse::<u32>().ok()?;
    let month = digits[2..4].parse::<u32>().ok()?;
    let year = digits[4..8].parse::<i32>().ok()?;
    let date = chrono::NaiveDate::from_ymd_opt(year, month, day)?;
    let dt = date.and_hms_opt(0, 0, 0)?;
    Some(dt.and_utc().timestamp())
}

fn add_days_to_dig_date(value: &str, days: i64) -> String {
    let digits = only_digits(value);
    if digits.len() != 8 {
        return String::new();
    }
    let day = digits[0..2].parse::<u32>().ok();
    let month = digits[2..4].parse::<u32>().ok();
    let year = digits[4..8].parse::<i32>().ok();
    let Some(date) = year.and_then(|y| {
        month.and_then(|m| day.and_then(|d| chrono::NaiveDate::from_ymd_opt(y, m, d)))
    }) else {
        return String::new();
    };
    let date = date + chrono::Duration::days(days);
    date.format("%d%m%Y").to_string()
}

fn is_valid_dig_date(value: &str) -> bool {
    only_digits(value).len() == 8
}

fn file_stem(name: &str) -> String {
    Path::new(name)
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or("arquivo")
        .to_string()
}

fn write_text_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, content).map_err(|e| e.to_string())
}

fn csv_escape(value: &str) -> String {
    if value.contains('"') || value.contains(';') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn find_first_descendant<'a>(node: Node<'a, 'a>, tag: &str) -> Option<Node<'a, 'a>> {
    node.descendants()
        .find(|n| n.is_element() && n.tag_name().name().eq_ignore_ascii_case(tag))
}

fn find_text_in_node(node: Option<Node<'_, '_>>, tags: &[&str]) -> String {
    let Some(node) = node else {
        return String::new();
    };
    for tag in tags {
        if let Some(found) = find_first_descendant(node, tag) {
            if let Some(text) = found.text() {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }
    String::new()
}

fn find_chave_nfe(doc: &Document<'_>) -> String {
    for node in doc.descendants().filter(|n| n.is_element()) {
        if node.tag_name().name().eq_ignore_ascii_case("chNFe") {
            let digits = only_digits(node.text().unwrap_or_default());
            if digits.len() == 44 {
                return digits;
            }
        }
    }
    for node in doc.descendants().filter(|n| n.is_element()) {
        if node.tag_name().name().eq_ignore_ascii_case("infNFe") {
            let id = node
                .attribute("Id")
                .or_else(|| node.attribute("id"))
                .unwrap_or_default();
            let digits = only_digits(id);
            if digits.len() >= 44 {
                return digits[digits.len() - 44..].to_string();
            }
        }
    }
    String::new()
}

fn parse_nfe_meta_and_rows(
    xml: &str,
    file_name: &str,
    settings: &NfeFaturasSettings,
) -> Result<(NfeMeta, Vec<NfeFaturasRow>)> {
    let doc = Document::parse(xml)?;
    let nfe = doc
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name().eq_ignore_ascii_case("NFe"))
        .ok_or_else(|| anyhow!("Não encontrei tag <NFe>"))?;
    let inf_nfe = find_first_descendant(nfe, "infNFe").unwrap_or(nfe);
    let ide = find_first_descendant(inf_nfe, "ide");
    let emit = find_first_descendant(inf_nfe, "emit");
    let dest = find_first_descendant(inf_nfe, "dest");
    let cobr = find_first_descendant(inf_nfe, "cobr");
    let total = find_first_descendant(inf_nfe, "total");
    let icms_tot = total.and_then(|node| find_first_descendant(node, "ICMSTot"));

    let chave = find_chave_nfe(&doc);
    let ctx = settings.ctx_nota.to_ascii_lowercase();
    let terceiro = if ctx == "saida" { dest } else { emit };
    let filial = if ctx == "saida" { emit } else { dest };

    let cnpj_filial = only_digits(&find_text_in_node(filial, &["CNPJ", "CPF"]));
    let cnpj_cpf_terceiro = only_digits(&find_text_in_node(terceiro, &["CNPJ", "CPF"]));
    let ender_terceiro = terceiro
        .and_then(|node| {
            if ctx == "saida" {
                find_first_descendant(node, "enderDest")
            } else {
                find_first_descendant(node, "enderEmit")
            }
        })
        .or_else(|| terceiro.and_then(|node| find_first_descendant(node, "enderDest")))
        .or_else(|| terceiro.and_then(|node| find_first_descendant(node, "enderEmit")));

    let uf = find_text_in_node(ender_terceiro, &["UF"]).to_ascii_uppercase();
    let ie = only_alphanum_upper(&find_text_in_node(terceiro, &["IE"]));
    let serie = only_digits(&find_text_in_node(ide, &["serie"]));
    let nf_numero = only_digits(&find_text_in_node(ide, &["nNF"]));
    let dh_emi = {
        let dh = find_text_in_node(ide, &["dhEmi"]);
        if dh.is_empty() {
            find_text_in_node(ide, &["dEmi"])
        } else {
            dh
        }
    };
    let dh_sai_ent = {
        let dh = find_text_in_node(ide, &["dhSaiEnt"]);
        if dh.is_empty() {
            let d = find_text_in_node(ide, &["dSaiEnt"]);
            if d.is_empty() {
                dh_emi.clone()
            } else {
                d
            }
        } else {
            dh
        }
    };

    let data_emissao_br = digits_to_br_date(&ddmmyyyy_to_digits(&dh_emi));
    let data_entrada_br = digits_to_br_date(&ddmmyyyy_to_digits(&dh_sai_ent));
    let v_nf = find_text_in_node(icms_tot, &["vNF"]);

    let fat = cobr.and_then(|node| find_first_descendant(node, "fat"));
    let n_fat = find_text_in_node(fat, &["nFat"]);
    let v_orig = find_text_in_node(fat, &["vOrig"]);
    let v_liq = find_text_in_node(fat, &["vLiq"]);

    let mut dup_list = Vec::<NfeMetaDup>::new();
    let mut rows = Vec::<NfeFaturasRow>::new();
    if let Some(cobr_node) = cobr {
        for dup in cobr_node
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name().eq_ignore_ascii_case("dup"))
        {
            let n_dup = find_text_in_node(Some(dup), &["nDup"]);
            let d_venc = find_text_in_node(Some(dup), &["dVenc"]);
            let v_dup = find_text_in_node(Some(dup), &["vDup"]);
            dup_list.push(NfeMetaDup {
                n_dup: if n_dup.is_empty() {
                    format!("{}/{}", nf_numero, serie)
                } else {
                    n_dup.clone()
                },
                d_venc: d_venc.clone(),
                v_dup: v_dup.clone(),
            });
            rows.push(NfeFaturasRow {
                chave: chave.clone(),
                desdob: 0,
                cnpj_filial: cnpj_filial.clone(),
                cnpj_cpf: cnpj_cpf_terceiro.clone(),
                uf: uf.clone(),
                ie: ie.clone(),
                nf_serie: serie.clone(),
                nf_numero: nf_numero.clone(),
                data_emissao: data_emissao_br.clone(),
                data_entrada: data_entrada_br.clone(),
                num_fatura: if n_dup.is_empty() {
                    format!("{}/{}", nf_numero, serie)
                } else {
                    n_dup
                },
                data_vencimento: digits_to_br_date(&ddmmyyyy_to_digits(&d_venc)),
                valor_bruto_fat: to_money2(if v_dup.is_empty() { &v_nf } else { &v_dup }),
                source: file_name.to_string(),
                sped_matched: false,
                legado: false,
                consolidated: false,
            });
        }
    }

    if rows.is_empty() && settings.chk_incluir_sem_dup {
        rows.push(NfeFaturasRow {
            chave: chave.clone(),
            desdob: 0,
            cnpj_filial: cnpj_filial.clone(),
            cnpj_cpf: cnpj_cpf_terceiro.clone(),
            uf: uf.clone(),
            ie: ie.clone(),
            nf_serie: serie.clone(),
            nf_numero: nf_numero.clone(),
            data_emissao: data_emissao_br.clone(),
            data_entrada: data_entrada_br.clone(),
            num_fatura: if n_fat.is_empty() {
                format!("{}/{}", nf_numero, serie)
            } else {
                n_fat.clone()
            },
            data_vencimento: String::new(),
            valor_bruto_fat: to_money2(if v_orig.is_empty() {
                if v_liq.is_empty() {
                    &v_nf
                } else {
                    &v_liq
                }
            } else {
                &v_orig
            }),
            source: file_name.to_string(),
            sped_matched: false,
            legado: false,
            consolidated: false,
        });
    }

    let meta = NfeMeta {
        chave,
        n_fat,
        v_orig,
        v_liq,
        v_nf,
        dup_list,
        nf_numero,
        serie,
        cnpj_terceiro: cnpj_cpf_terceiro,
        dt_emi: ddmmyyyy_to_digits(&dh_emi),
        source: file_name.to_string(),
    };

    Ok((meta, rows))
}

fn parse_sped_c100(content: &str) -> HashMap<String, SpedC100Item> {
    let mut map = HashMap::<String, SpedC100Item>::new();
    for raw in content.lines() {
        let line = raw.trim();
        if !line.starts_with("|C100|") {
            continue;
        }
        let fields = line
            .trim_matches('|')
            .split('|')
            .map(|v| v.trim())
            .collect::<Vec<_>>();
        if fields.is_empty() || fields[0] != "C100" {
            continue;
        }
        let chave = only_digits(fields.get(8).copied().unwrap_or_default());
        if chave.len() != 44 {
            continue;
        }
        let dt_escr = only_digits(fields.get(10).copied().unwrap_or_default());
        if dt_escr.len() != 8 {
            continue;
        }
        let ind_pgto = fields.get(12).copied().unwrap_or_default();
        let desdob = if ind_pgto == "1" { 1 } else { 0 };
        map.entry(chave).or_insert(SpedC100Item { dt_escr, desdob });
    }
    map
}

fn parse_sped_0150(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::<String, String>::new();
    for raw in content.lines() {
        let line = raw.trim();
        if !line.starts_with("|0150|") {
            continue;
        }
        let fields = line
            .trim_matches('|')
            .split('|')
            .map(|v| v.trim())
            .collect::<Vec<_>>();
        if fields.is_empty() || fields[0] != "0150" {
            continue;
        }
        let doc = only_digits(fields.get(4).copied().unwrap_or_default());
        let doc = if doc.is_empty() {
            only_digits(fields.get(5).copied().unwrap_or_default())
        } else {
            doc
        };
        if doc.len() != 11 && doc.len() != 14 {
            continue;
        }
        let ie = only_alphanum_upper(fields.get(6).copied().unwrap_or_default());
        if ie.is_empty() {
            continue;
        }
        map.insert(doc, ie);
    }
    map
}

fn apply_sped_c100_to_rows(
    rows: &mut [NfeFaturasRow],
    sped_by_chave: &HashMap<String, SpedC100Item>,
    settings: &NfeFaturasSettings,
) {
    if !settings.chk_usar_sped {
        for row in rows {
            row.sped_matched = false;
        }
        return;
    }
    for row in rows {
        let chave = only_digits(&row.chave);
        if let Some(item) = sped_by_chave.get(&chave) {
            row.data_entrada = digits_to_br_date(&item.dt_escr);
            row.desdob = item.desdob;
            row.sped_matched = true;
        } else {
            row.sped_matched = false;
        }
    }
}

fn apply_sped_0150_ie_to_rows(
    rows: &mut [NfeFaturasRow],
    map: &HashMap<String, String>,
    settings: &NfeFaturasSettings,
    logs: &mut Vec<NfeFaturasLogItem>,
) {
    if map.is_empty() {
        return;
    }
    for row in rows {
        let doc = only_digits(&row.cnpj_cpf);
        if doc.len() != 11 && doc.len() != 14 {
            continue;
        }
        let Some(ie_sped) = map.get(&doc) else {
            continue;
        };
        let ie_xml = only_alphanum_upper(&row.ie);
        let forced = settings.chk_ie_sped_padrao;
        let should_replace = if forced {
            true
        } else if ie_xml.is_empty() {
            true
        } else {
            only_digits(&ie_xml).len() != only_digits(ie_sped).len()
        };
        if should_replace && ie_xml != *ie_sped {
            row.ie = ie_sped.clone();
            log_push(
                logs,
                "info",
                "IE ajustada via SPED 0150",
                Some(format!(
                    "chave {} | NF {} | CNPJ {} | IE xml {} -> IE sped {}",
                    row.chave,
                    row.nf_numero,
                    doc,
                    if ie_xml.is_empty() {
                        "(vazio)"
                    } else {
                        ie_xml.as_str()
                    },
                    ie_sped
                )),
            );
        }
    }
}

fn guess_cnpj_filial(rows: &[NfeFaturasRow], sped_files: &[NamedText]) -> Option<String> {
    for row in rows {
        if !row.legado {
            let dig = only_digits(&row.cnpj_filial);
            if dig.len() == 14 {
                return Some(dig);
            }
        }
    }
    for sped in sped_files {
        for raw in sped.text.lines() {
            let line = raw.trim();
            if !line.starts_with("|0000|") {
                continue;
            }
            let fields = line
                .trim_matches('|')
                .split('|')
                .map(|v| v.trim())
                .collect::<Vec<_>>();
            if fields.first().copied() != Some("0000") {
                continue;
            }
            let cnpj = only_digits(fields.get(5).copied().unwrap_or_default());
            if cnpj.len() == 14 {
                return Some(cnpj);
            }
        }
    }
    None
}

fn parse_legacy_pipe_file(
    text: &str,
    file_name: &str,
    conferir: bool,
    cnpj_filial: Option<&str>,
    sped_files: &[NamedText],
    nfe_metas: &[NfeMeta],
    logs: &mut Vec<NfeFaturasLogItem>,
) -> Result<LegacyStats> {
    let clean = text.trim_start_matches('\u{feff}');
    let mut groups = BTreeMap::<String, LegacyGroup>::new();
    let mut invalid_lines = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    let mut invalid_count = 0usize;
    let mut warning_count = 0usize;
    let mut header_skipped = false;

    for (idx, raw_line) in clean.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let fields = trimmed
            .split('|')
            .map(|v| v.trim().to_string())
            .collect::<Vec<_>>();
        if !header_skipped && is_legacy_header_line(&fields, idx) {
            header_skipped = true;
            continue;
        }
        if fields.len() < 11 {
            invalid_count += 1;
            push_bounded(
                &mut invalid_lines,
                format!("Linha {}: colunas insuficientes.", line_no),
            );
            continue;
        }
        let cnpj = only_digits(fields.get(0).map(|v| v.as_str()).unwrap_or_default());
        if cnpj.len() != 14 {
            invalid_count += 1;
            push_bounded(
                &mut invalid_lines,
                format!("Linha {}: CNPJ inválido.", line_no),
            );
            continue;
        }
        let uf = fields
            .get(1)
            .cloned()
            .unwrap_or_default()
            .to_ascii_uppercase();
        if uf.len() != 2 {
            invalid_count += 1;
            push_bounded(
                &mut invalid_lines,
                format!("Linha {}: UF inválida.", line_no),
            );
            continue;
        }
        let ie = only_alphanum_upper(fields.get(2).map(|v| v.as_str()).unwrap_or_default());
        let serie = fields.get(3).cloned().unwrap_or_default();
        let nf_numero = only_digits(fields.get(5).map(|v| v.as_str()).unwrap_or_default());
        if nf_numero.is_empty() {
            invalid_count += 1;
            push_bounded(
                &mut invalid_lines,
                format!("Linha {}: NF Num ausente.", line_no),
            );
            continue;
        }
        let Some((dt_escrit_br, dt_escrit_dig)) =
            parse_legacy_date(fields.get(6).map(|v| v.as_str()).unwrap_or_default())
        else {
            invalid_count += 1;
            push_bounded(
                &mut invalid_lines,
                format!("Linha {}: DT Escrit inválida.", line_no),
            );
            continue;
        };
        let parcela = fields.get(7).cloned().unwrap_or_default();
        let Some((dt_venc_br, _)) =
            parse_legacy_date(fields.get(8).map(|v| v.as_str()).unwrap_or_default())
        else {
            invalid_count += 1;
            push_bounded(
                &mut invalid_lines,
                format!("Linha {}: DT VENC inválida.", line_no),
            );
            continue;
        };
        let Some(vl_nf) = parse_money(fields.get(9).map(|v| v.as_str()).unwrap_or_default()) else {
            invalid_count += 1;
            push_bounded(
                &mut invalid_lines,
                format!("Linha {}: VL NF inválido.", line_no),
            );
            continue;
        };
        let Some(vl_fat) = parse_money(fields.get(10).map(|v| v.as_str()).unwrap_or_default())
        else {
            invalid_count += 1;
            push_bounded(
                &mut invalid_lines,
                format!("Linha {}: VL Fatura inválido.", line_no),
            );
            continue;
        };
        if vl_fat <= 0.0 {
            warning_count += 1;
            push_bounded(&mut warnings, format!("Linha {}: VL Fatura <= 0.", line_no));
        }
        if parcela.trim().is_empty() {
            warning_count += 1;
            push_bounded(
                &mut warnings,
                format!("Linha {}: Parcela vazia (fallback aplicado).", line_no),
            );
        }
        let key = format!("{}|{}|{}|{}", cnpj, serie, nf_numero, dt_escrit_dig);
        let group = groups.entry(key).or_insert(LegacyGroup {
            cnpj: cnpj.clone(),
            uf: uf.clone(),
            ie: ie.clone(),
            serie: serie.clone(),
            nf_numero: nf_numero.clone(),
            dt_escrit: dt_escrit_br.clone(),
            total_nf: vl_nf,
            duplicatas: Vec::new(),
        });
        if (group.total_nf - vl_nf).abs() > 0.01 {
            warning_count += 1;
            push_bounded(
                &mut warnings,
                format!("Linha {}: VL NF divergente dentro do grupo.", line_no),
            );
        }
        group.duplicatas.push(LegacyDup {
            n_dup: if parcela.trim().is_empty() {
                format!("{}/{}", nf_numero, serie)
            } else {
                parcela
            },
            d_venc: dt_venc_br,
            v_dup: vl_fat,
        });
    }

    let guessed_cnpj = cnpj_filial.map(only_digits).filter(|v| v.len() == 14);
    let mut rows = Vec::<NfeFaturasRow>::new();
    let mut parcelas = 0usize;
    for group in groups.values() {
        for dup in &group.duplicatas {
            parcelas += 1;
            rows.push(NfeFaturasRow {
                chave: String::new(),
                desdob: 0,
                cnpj_filial: guessed_cnpj.clone().unwrap_or_default(),
                cnpj_cpf: group.cnpj.clone(),
                uf: group.uf.clone(),
                ie: group.ie.clone(),
                nf_serie: group.serie.clone(),
                nf_numero: group.nf_numero.clone(),
                data_emissao: group.dt_escrit.clone(),
                data_entrada: group.dt_escrit.clone(),
                num_fatura: dup.n_dup.clone(),
                data_vencimento: dup.d_venc.clone(),
                valor_bruto_fat: format!("{:.2}", dup.v_dup),
                source: format!("LEGADO:{}", file_name),
                sped_matched: false,
                legado: true,
                consolidated: false,
            });
        }
    }
    if guessed_cnpj.is_none() && !rows.is_empty() {
        warning_count += 1;
        push_bounded(
            &mut warnings,
            "CNPJ da filial não definido (coluna CNPJFILIAL ficará vazia).".into(),
        );
    }

    let mut divergences = Vec::<String>::new();
    let mut divergence_count = 0usize;
    if conferir {
        let xml_index = build_xml_index(nfe_metas);
        let sped_index = build_sped_index(sped_files);
        if xml_index.is_empty() && sped_index.is_empty() {
            divergence_count += 1;
            push_bounded(
                &mut divergences,
                "XML/SPED não processados. Carregue e processe antes para conferir.".into(),
            );
        } else {
            for group in groups.values() {
                let key = format!(
                    "{}|{}|{}",
                    group.cnpj,
                    only_digits(&group.serie),
                    only_digits(&group.nf_numero)
                );
                let total_dup = group.duplicatas.iter().map(|d| d.v_dup).sum::<f64>();
                if let Some(meta) = xml_index.get(&key) {
                    let xml_total = compute_total_from_meta(meta);
                    if (xml_total - total_dup).abs() > 0.05 {
                        divergence_count += 1;
                        push_bounded(
                            &mut divergences,
                            format!(
                                "XML: total divergente para NF {}/{} (legado {:.2} x XML {:.2}).",
                                group.nf_numero, group.serie, total_dup, xml_total
                            ),
                        );
                    }
                    let mut legacy_venc = group
                        .duplicatas
                        .iter()
                        .map(|d| d.d_venc.clone())
                        .collect::<Vec<_>>();
                    legacy_venc.sort();
                    let mut xml_venc = meta
                        .dup_list
                        .iter()
                        .map(|d| digits_to_br_date(&ddmmyyyy_to_digits(&d.d_venc)))
                        .filter(|d| !d.is_empty())
                        .collect::<Vec<_>>();
                    xml_venc.sort();
                    if !xml_venc.is_empty() && !legacy_venc.is_empty() && xml_venc != legacy_venc {
                        divergence_count += 1;
                        push_bounded(
                            &mut divergences,
                            format!(
                                "XML: vencimentos divergentes na NF {}/{}.",
                                group.nf_numero, group.serie
                            ),
                        );
                    }
                } else if !xml_index.is_empty() {
                    divergence_count += 1;
                    push_bounded(
                        &mut divergences,
                        format!(
                            "XML: NF {}/{} não encontrada para CNPJ {}.",
                            group.nf_numero, group.serie, group.cnpj
                        ),
                    );
                }
                if let Some(sped) = sped_index.get(&key) {
                    if !sped.0.is_empty() && sped.0 != group.dt_escrit {
                        divergence_count += 1;
                        push_bounded(
                            &mut divergences,
                            format!(
                                "SPED: DT_E_S divergente na NF {}/{} (legado {} x SPED {}).",
                                group.nf_numero, group.serie, group.dt_escrit, sped.0
                            ),
                        );
                    }
                    if let Some(vl_doc) = sped.1 {
                        if (vl_doc - group.total_nf).abs() > 0.05 {
                            divergence_count += 1;
                            push_bounded(
                                &mut divergences,
                                format!("SPED: VL_DOC divergente na NF {}/{} (legado {:.2} x SPED {:.2}).", group.nf_numero, group.serie, group.total_nf, vl_doc),
                            );
                        }
                    }
                } else if !sped_index.is_empty() {
                    divergence_count += 1;
                    push_bounded(
                        &mut divergences,
                        format!(
                            "SPED: NF {}/{} não encontrada para CNPJ {}.",
                            group.nf_numero, group.serie, group.cnpj
                        ),
                    );
                }
            }
        }
    }

    if !rows.is_empty() {
        log_push(
            logs,
            "info",
            "LEGADO importado",
            Some(format!(
                "{} | notas {} | parcelas {}",
                file_name,
                groups.len(),
                parcelas
            )),
        );
    }

    Ok(LegacyStats {
        rows,
        notes: groups.len(),
        parcelas,
        invalid_count,
        invalid_lines,
        warning_count,
        warnings,
        divergences,
        divergence_count,
    })
}

fn push_bounded(list: &mut Vec<String>, value: String) {
    if list.len() < 30 {
        list.push(value);
    }
}

fn is_legacy_header_line(fields: &[String], line_index: usize) -> bool {
    let joined = fields.join("|").to_ascii_uppercase();
    if line_index == 0
        && (joined.contains("CNPJ")
            || joined.contains("EMITENTE")
            || joined.contains("SERIE")
            || joined.contains("ESCRIT"))
    {
        return true;
    }
    let first = fields
        .first()
        .cloned()
        .unwrap_or_default()
        .to_ascii_uppercase();
    if first.contains("CNPJ") || first.contains("EMITENTE") {
        return true;
    }
    line_index < 3
        && joined.chars().any(|c| c.is_ascii_alphabetic())
        && only_digits(&joined).len() < 11
}

fn parse_legacy_date(value: &str) -> Option<(String, String)> {
    let clean = value.trim();
    let parts = clean
        .split(|c| c == '/' || c == '-' || c == '.')
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();
    if parts.len() != 3 {
        return None;
    }
    let day = parts[0].parse::<u32>().ok()?;
    let month = parts[1].parse::<u32>().ok()?;
    let mut year = parts[2].parse::<i32>().ok()?;
    if parts[2].len() == 2 {
        year += 2000;
    }
    let date = chrono::NaiveDate::from_ymd_opt(year, month, day)?;
    Some((
        date.format("%d/%m/%Y").to_string(),
        date.format("%d%m%Y").to_string(),
    ))
}

fn build_xml_index(nfe_metas: &[NfeMeta]) -> HashMap<String, NfeMeta> {
    let mut map = HashMap::<String, NfeMeta>::new();
    for meta in nfe_metas {
        let key = format!(
            "{}|{}|{}",
            only_digits(&meta.cnpj_terceiro),
            only_digits(&meta.serie),
            only_digits(&meta.nf_numero)
        );
        if !key.starts_with("||") && !map.contains_key(&key) {
            map.insert(key, meta.clone());
        }
    }
    map
}

fn build_sped_index(sped_files: &[NamedText]) -> HashMap<String, (String, Option<f64>)> {
    let mut index = HashMap::<String, (String, Option<f64>)>::new();
    for sped in sped_files {
        let mut part_doc = HashMap::<String, String>::new();
        for raw in sped.text.lines() {
            let line = raw.trim();
            if !line.starts_with("|0150|") {
                continue;
            }
            let fields = line
                .trim_matches('|')
                .split('|')
                .map(|v| v.trim())
                .collect::<Vec<_>>();
            let cod = fields.get(1).copied().unwrap_or_default().to_string();
            let mut doc = only_digits(fields.get(4).copied().unwrap_or_default());
            if doc.is_empty() {
                doc = only_digits(fields.get(5).copied().unwrap_or_default());
            }
            if !cod.is_empty() && (doc.len() == 11 || doc.len() == 14) {
                part_doc.insert(cod, doc);
            }
        }
        for raw in sped.text.lines() {
            let line = raw.trim();
            if !line.starts_with("|C100|") {
                continue;
            }
            let fields = line
                .trim_matches('|')
                .split('|')
                .map(|v| v.trim())
                .collect::<Vec<_>>();
            let cod_part = fields.get(3).copied().unwrap_or_default();
            let doc = part_doc.get(cod_part).cloned().unwrap_or_default();
            let serie = only_digits(fields.get(6).copied().unwrap_or_default());
            let num = only_digits(fields.get(7).copied().unwrap_or_default());
            if doc.is_empty() || num.is_empty() {
                continue;
            }
            let dt_es =
                digits_to_br_date(&only_digits(fields.get(10).copied().unwrap_or_default()));
            let vl_doc = parse_money(fields.get(11).copied().unwrap_or_default());
            let key = format!("{}|{}|{}", doc, serie, num);
            index.entry(key).or_insert((dt_es, vl_doc));
        }
    }
    index
}

fn compute_total_from_meta(meta: &NfeMeta) -> f64 {
    if !meta.dup_list.is_empty() {
        let sum = meta
            .dup_list
            .iter()
            .map(|d| parse_money(&d.v_dup).unwrap_or(0.0))
            .sum::<f64>();
        if sum > 0.0 {
            return sum;
        }
    }
    parse_money(&meta.v_liq)
        .or_else(|| parse_money(&meta.v_orig))
        .or_else(|| parse_money(&meta.v_nf))
        .unwrap_or(0.0)
}

fn parse_cnpj_set(raw: &str) -> HashSet<String> {
    raw.split(|c: char| c.is_whitespace() || c == ',' || c == ';')
        .filter_map(|part| {
            let dig = only_digits(part);
            if dig.len() == 11 || dig.len() == 14 {
                Some(dig)
            } else {
                None
            }
        })
        .collect()
}

fn apply_consolidacao_por_terceiro(
    rows: &[NfeFaturasRow],
    settings: &NfeFaturasSettings,
) -> Vec<NfeFaturasRow> {
    if !settings.chk_consolidar_cnpj {
        return rows.to_vec();
    }
    let set = parse_cnpj_set(&settings.txt_cnpjs_consolidar);
    if set.is_empty() {
        return rows.to_vec();
    }

    let mut groups = BTreeMap::<String, Vec<NfeFaturasRow>>::new();
    let mut order = Vec::<String>::new();
    for row in rows {
        let key = format!("{}|{}", only_digits(&row.chave), only_digits(&row.cnpj_cpf));
        if !groups.contains_key(&key) {
            order.push(key.clone());
        }
        groups.entry(key).or_default().push(row.clone());
    }

    let mut out = Vec::<NfeFaturasRow>::new();
    for key in order {
        let arr = groups.remove(&key).unwrap_or_default();
        let cnpj = arr
            .first()
            .map(|r| only_digits(&r.cnpj_cpf))
            .unwrap_or_default();
        if set.contains(&cnpj) && arr.len() > 1 {
            let mut base = arr[0].clone();
            let total = arr
                .iter()
                .map(|r| parse_money(&r.valor_bruto_fat).unwrap_or(0.0))
                .sum::<f64>();
            let min_venc = arr
                .iter()
                .filter_map(|r| {
                    br_date_to_time(&r.data_vencimento).map(|t| (t, r.data_vencimento.clone()))
                })
                .min_by_key(|item| item.0)
                .map(|item| item.1)
                .unwrap_or_else(|| base.data_vencimento.clone());
            base.valor_bruto_fat = format!("{:.2}", total);
            base.data_vencimento = min_venc;
            base.desdob = 0;
            base.consolidated = true;
            out.push(base);
        } else {
            out.extend(arr);
        }
    }
    out
}

fn compute_export_rows_base(
    rows: &[NfeFaturasRow],
    settings: &NfeFaturasSettings,
) -> Vec<NfeFaturasRow> {
    let filtered = if settings.chk_usar_sped && settings.chk_somente_com_sped {
        rows.iter()
            .filter(|r| r.sped_matched)
            .cloned()
            .collect::<Vec<_>>()
    } else {
        rows.to_vec()
    };
    apply_consolidacao_por_terceiro(&filtered, settings)
}

fn compute_export_lines(
    rows: &[NfeFaturasRow],
    settings: &NfeFaturasSettings,
) -> Vec<ExportLineItem> {
    let mut lines = Vec::<ExportLineItem>::new();
    for row in rows {
        if settings.chk_forcar_duas_linhas {
            lines.push(ExportLineItem {
                row: row.clone(),
                item3: "000".into(),
            });
            lines.push(ExportLineItem {
                row: row.clone(),
                item3: "001".into(),
            });
        } else {
            lines.push(ExportLineItem {
                row: row.clone(),
                item3: "000".into(),
            });
            if row.desdob == 1 {
                lines.push(ExportLineItem {
                    row: row.clone(),
                    item3: "001".into(),
                });
            }
        }
    }
    lines
}

fn pad_serie_with_zeros(serie: &str, settings: &NfeFaturasSettings) -> String {
    let digits = settings.serie_digits.parse::<usize>().unwrap_or(0);
    let s = only_digits(serie);
    if s.is_empty() || digits == 0 {
        return s;
    }
    format!("{:0>width$}", s, width = digits)
}

fn format_str_size(source: &str, ch: char, size: usize, left_align: bool) -> String {
    let source = source.to_string();
    if left_align {
        let mut result = source;
        while result.len() < size {
            result.push(ch);
        }
        result.chars().take(size).collect()
    } else {
        let mut result = String::new();
        while result.len() + source.len() < size {
            result.push(ch);
        }
        result.push_str(&source);
        let len = result.chars().count();
        result.chars().skip(len.saturating_sub(size)).collect()
    }
}

fn build_txt_line(
    row: &NfeFaturasRow,
    item_desdob3: &str,
    settings: &NfeFaturasSettings,
) -> String {
    let origem = settings.origem.chars().next().unwrap_or('0').to_string();
    let tipo = settings.tipo.chars().next().unwrap_or('1').to_string();
    let cnpjcpf = only_digits(&row.cnpj_cpf);
    let uf = row.uf.to_ascii_uppercase();
    let ie = only_alphanum_upper(&row.ie);
    let serie_fmt = pad_serie_with_zeros(&row.nf_serie, settings);
    let nfnumero = row.nf_numero.clone();
    let data_escr = ddmmyyyy_to_digits(if row.data_entrada.is_empty() {
        &row.data_emissao
    } else {
        &row.data_entrada
    });
    let num_fatura = format!("{}/{}", row.nf_numero, row.num_fatura);
    let mut data_venc = ddmmyyyy_to_digits(&row.data_vencimento);
    if settings.chk_venc30 {
        let base = ddmmyyyy_to_digits(if row.data_entrada.is_empty() {
            &row.data_emissao
        } else {
            &row.data_entrada
        });
        if base.len() == 8 {
            data_venc = add_days_to_dig_date(&base, 30);
        }
    }
    let valor = to_money2(&row.valor_bruto_fat);

    [
        format_str_size(&origem, ' ', 1, false),
        format_str_size(&tipo, ' ', 1, false),
        format_str_size(&cnpjcpf, '0', 14, false),
        format_str_size(&uf, ' ', 2, false),
        format_str_size(&ie, ' ', 20, false),
        format_str_size(&serie_fmt, ' ', 6, false),
        format_str_size("", ' ', 6, false),
        format_str_size(&nfnumero, '0', 10, false),
        format_str_size(item_desdob3, ' ', 3, false),
        format_str_size(&data_escr, ' ', 8, true),
        format_str_size(&num_fatura, ' ', 20, false),
        format_str_size(&data_venc, ' ', 8, false),
        format_str_size(&valor, ' ', 14, false),
        format_str_size("", ' ', 14, false),
        format_str_size("", ' ', 14, false),
        format_str_size("", ' ', 14, false),
        format_str_size("", ' ', 14, false),
        format_str_size(&valor, ' ', 14, false),
        format_str_size("", ' ', 8, false),
        format_str_size("", ' ', 11, false),
        format_str_size("", ' ', 8, false),
        format_str_size("", ' ', 8, false),
        format_str_size("", ' ', 14, false),
        format_str_size("", ' ', 11, false),
        format_str_size("", ' ', 14, false),
        format_str_size("", ' ', 11, false),
        format_str_size("", ' ', 14, false),
        format_str_size("", ' ', 11, false),
        format_str_size("", ' ', 6, false),
        format_str_size("N", ' ', 1, false),
        format_str_size("", ' ', 14, true),
        format_str_size("2", ' ', 1, false),
        format_str_size("", ' ', 30, false),
        format_str_size("9", ' ', 1, false),
        format_str_size("03", ' ', 2, false),
        format_str_size("", ' ', 40, false),
        format_str_size("", ' ', 3, false),
    ]
    .join("")
}

#[derive(Debug, Clone)]
struct SpedExportOptions {
    recriar: bool,
    modo_parcelas: String,
    qtd_parcelas_geral: usize,
    regras_fornecedor: HashMap<String, usize>,
    intervalo_dias: i32,
    dias_por_parcela: Vec<i32>,
    regra_sem_dup: String,
    consolidacao_interna: String,
}

impl SpedExportOptions {
    fn from_settings(settings: &NfeFaturasSettings) -> Self {
        Self {
            recriar: settings.chk_recriar_c140_c141,
            modo_parcelas: settings.sel_modo_parcelas.clone(),
            qtd_parcelas_geral: settings.num_qtd_parcelas_geral.max(1) as usize,
            regras_fornecedor: parse_parcelas_por_fornecedor(
                &settings.txt_regras_fornecedor_parcelas,
            ),
            intervalo_dias: if settings.chk_venc30 {
                30
            } else {
                settings.num_venc_intervalo_dias
            },
            dias_por_parcela: parse_dias_por_parcela(&settings.txt_venc_dias_por_parcela),
            regra_sem_dup: if settings.chk_incluir_sem_dup {
                "gerar".into()
            } else {
                "nao_gerar".into()
            },
            consolidacao_interna: settings.sel_consolidacao_interna_nfe.clone(),
        }
    }
}

fn parse_parcelas_por_fornecedor(raw: &str) -> HashMap<String, usize> {
    let mut map = HashMap::<String, usize>::new();
    for line in raw.lines() {
        let parts = line.split('=').collect::<Vec<_>>();
        if parts.len() < 2 {
            continue;
        }
        let cnpj = only_digits(parts[0]);
        let qtd = only_digits(parts[1]).parse::<usize>().unwrap_or(0);
        if (cnpj.len() == 11 || cnpj.len() == 14) && qtd > 0 {
            map.insert(cnpj, qtd);
        }
    }
    map
}

fn parse_dias_por_parcela(raw: &str) -> Vec<i32> {
    raw.split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .filter_map(|v| v.trim().parse::<i32>().ok())
        .filter(|v| *v >= 0)
        .collect()
}

fn split_total_into_parcelas(total: f64, qtd: usize) -> Vec<f64> {
    let qtd = qtd.max(1);
    let cents = (total * 100.0).round() as i64;
    let base = cents / qtd as i64;
    let rem = cents - (base * qtd as i64);
    let mut out = Vec::<f64>::new();
    for i in 0..qtd {
        let mut cur = base;
        if i + 1 == qtd {
            cur += rem;
        }
        out.push(cur as f64 / 100.0);
    }
    out
}

#[derive(Debug, Clone)]
struct ParcelaData {
    n_dup: String,
    venc: String,
    valor: f64,
}

fn build_parcelas_for_c100(
    meta: &NfeMeta,
    opts: &SpedExportOptions,
    dt_doc: &str,
    dt_es: &str,
) -> Vec<ParcelaData> {
    let total = compute_total_from_meta(meta);
    let mut base = Vec::<ParcelaData>::new();

    if opts.modo_parcelas == "respeitar_xml" {
        if !meta.dup_list.is_empty() {
            base = meta
                .dup_list
                .iter()
                .map(|d| ParcelaData {
                    n_dup: d.n_dup.clone(),
                    venc: digits_to_br_date(&ddmmyyyy_to_digits(&d.d_venc)),
                    valor: parse_money(&d.v_dup).unwrap_or(0.0),
                })
                .collect();
            if !base.iter().any(|p| p.valor > 0.0) && total > 0.0 {
                let split = split_total_into_parcelas(total, base.len());
                for (idx, item) in base.iter_mut().enumerate() {
                    item.valor = split[idx];
                }
            }
        } else if opts.regra_sem_dup == "gerar" {
            base.push(ParcelaData {
                n_dup: meta.n_fat.clone(),
                venc: String::new(),
                valor: total,
            });
        }
    } else {
        let mut qtd = opts.qtd_parcelas_geral;
        if opts.modo_parcelas == "forcar_por_fornecedor" {
            if let Some(found) = opts.regras_fornecedor.get(&meta.cnpj_terceiro) {
                qtd = *found;
            }
        }
        let split = split_total_into_parcelas(total, qtd.max(1));
        base = split
            .into_iter()
            .map(|v| ParcelaData {
                n_dup: String::new(),
                venc: String::new(),
                valor: v,
            })
            .collect();
    }

    let mut parcelas = Vec::<ParcelaData>::new();
    for (idx, item) in base.into_iter().enumerate() {
        let venc = if !item.venc.is_empty() {
            ddmmyyyy_to_digits(&item.venc)
        } else {
            let base_date = if is_valid_dig_date(dt_es) {
                dt_es.to_string()
            } else {
                dt_doc.to_string()
            };
            if !is_valid_dig_date(&base_date) {
                String::new()
            } else {
                let days = opts
                    .dias_por_parcela
                    .get(idx)
                    .copied()
                    .unwrap_or(opts.intervalo_dias * (idx as i32 + 1));
                add_days_to_dig_date(&base_date, days as i64)
            }
        };
        if !is_valid_dig_date(&venc) {
            continue;
        }
        parcelas.push(ParcelaData {
            n_dup: item.n_dup,
            venc,
            valor: item.valor,
        });
    }

    normalize_parcelas(parcelas, &opts.consolidacao_interna)
}

fn normalize_parcelas(parcelas: Vec<ParcelaData>, mode: &str) -> Vec<ParcelaData> {
    if parcelas.is_empty() {
        return parcelas;
    }
    if mode == "reduzir_para_1_parcela" {
        let total = parcelas.iter().map(|p| p.valor).sum::<f64>();
        let first = parcelas
            .iter()
            .find(|p| !p.venc.is_empty())
            .cloned()
            .unwrap_or_else(|| parcelas[0].clone());
        return vec![ParcelaData {
            n_dup: first.n_dup,
            venc: first.venc,
            valor: total,
        }];
    }
    if mode == "agrupar_por_data" {
        let mut map = BTreeMap::<String, f64>::new();
        for parcela in parcelas {
            *map.entry(parcela.venc).or_insert(0.0) += parcela.valor;
        }
        return map
            .into_iter()
            .map(|(venc, valor)| ParcelaData {
                n_dup: String::new(),
                venc,
                valor,
            })
            .collect();
    }
    parcelas
}

fn build_c140_c141_lines(meta: &NfeMeta, parcelas: &[ParcelaData], dt_doc: &str) -> Vec<String> {
    if parcelas.is_empty() {
        return Vec::new();
    }
    let num_tit = parcelas
        .iter()
        .find(|p| !p.n_dup.trim().is_empty())
        .map(|p| p.n_dup.clone())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| {
            if !meta.n_fat.is_empty() {
                meta.n_fat.clone()
            } else if !meta.nf_numero.is_empty() {
                format!("{}/{}", meta.nf_numero, meta.serie)
            } else if meta.chave.len() >= 6 {
                meta.chave[meta.chave.len() - 6..].to_string()
            } else {
                "TITULO".into()
            }
        });
    let total = parcelas.iter().map(|p| p.valor).sum::<f64>();
    let mut out = Vec::<String>::new();
    out.push(format!(
        "|C140|1|00||{}|{}|{}||",
        num_tit,
        parcelas.len(),
        to_sped_money(total)
    ));
    for (idx, parcela) in parcelas.iter().enumerate() {
        let mut venc = parcela.venc.clone();
        if let (Some(t_doc), Some(t_venc)) = (
            br_date_to_time(&digits_to_br_date(dt_doc)),
            br_date_to_time(&digits_to_br_date(&venc)),
        ) {
            if t_venc < t_doc {
                venc = dt_doc.to_string();
            }
        }
        out.push(format!(
            "|C141|{:02}|{}|{}||",
            idx + 1,
            venc,
            to_sped_money(parcela.valor)
        ));
    }
    out
}

#[derive(Debug, Clone, Default)]
struct UpdateSpedStats {
    total: usize,
    updated: usize,
    ignored: usize,
    missing_xml: usize,
    existing: usize,
}

#[derive(Debug, Clone)]
struct UpdatedSpedOutput {
    text: String,
    stats: UpdateSpedStats,
}

fn update_sped_text_with_c140_c141(
    sped_text: &str,
    _source_name: &str,
    opts: &SpedExportOptions,
    meta_map: &HashMap<String, NfeMeta>,
) -> Result<UpdatedSpedOutput, String> {
    let raw_lines = sped_text
        .lines()
        .map(|v| v.trim_end_matches('\r').to_string())
        .filter(|v| !v.trim().is_empty())
        .collect::<Vec<_>>();
    let lines = strip_block9(&raw_lines);
    let mut out = Vec::<String>::new();
    let mut stats = UpdateSpedStats::default();
    let mut i = 0usize;

    while i < lines.len() {
        let line = lines[i].trim().to_string();
        if !line.starts_with("|C100|") {
            out.push(lines[i].clone());
            i += 1;
            continue;
        }
        stats.total += 1;
        let start = i;
        let mut end = i + 1;
        while end < lines.len() {
            let current = lines[end].trim();
            if current.starts_with("|C100|") || current.starts_with("|C990|") {
                break;
            }
            end += 1;
        }
        let mut group = lines[start..end].to_vec();
        let fields = line
            .trim_matches('|')
            .split('|')
            .map(|v| v.trim())
            .collect::<Vec<_>>();
        let chave = only_digits(fields.get(8).copied().unwrap_or_default());
        let dt_doc = only_digits(fields.get(9).copied().unwrap_or_default());
        let dt_es = only_digits(fields.get(10).copied().unwrap_or_default());
        let has_c140 = group.iter().any(|l| {
            let trim = l.trim();
            trim.starts_with("|C140|") || trim.starts_with("|C141|")
        });
        if has_c140 {
            stats.existing += 1;
        }
        let Some(meta) = meta_map.get(&chave) else {
            stats.missing_xml += 1;
            out.extend(group);
            i = end;
            continue;
        };
        if has_c140 && !opts.recriar {
            stats.ignored += 1;
            out.extend(group);
            i = end;
            continue;
        }
        if has_c140 && opts.recriar {
            group.retain(|l| {
                let trim = l.trim();
                !trim.starts_with("|C140|") && !trim.starts_with("|C141|")
            });
        }
        let parcelas = build_parcelas_for_c100(meta, opts, &dt_doc, &dt_es);
        if parcelas.is_empty() {
            stats.ignored += 1;
            out.extend(group);
            i = end;
            continue;
        }
        let c_lines = build_c140_c141_lines(meta, &parcelas, &dt_doc);
        let insert_at = group
            .iter()
            .position(|l| l.trim().starts_with("|C170|"))
            .unwrap_or(group.len());
        let new_group = [
            group[..insert_at].to_vec(),
            c_lines,
            group[insert_at..].to_vec(),
        ]
        .concat();
        stats.updated += 1;
        out.extend(new_group);
        i = end;
    }

    let out = update_c990(&out);
    let out = rebuild_block9(&out);
    Ok(UpdatedSpedOutput {
        text: format!("{}\r\n", out.join("\r\n")),
        stats,
    })
}

fn strip_block9(lines: &[String]) -> Vec<String> {
    let Some(start) = lines.iter().position(|l| l.trim().starts_with("|9001|")) else {
        return lines.to_vec();
    };
    let end = lines
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, l)| l.trim().starts_with("|9999|"))
        .map(|(idx, _)| idx)
        .unwrap_or(lines.len().saturating_sub(1));
    let mut out = Vec::<String>::new();
    out.extend_from_slice(&lines[..start]);
    if end + 1 < lines.len() {
        out.extend_from_slice(&lines[end + 1..]);
    }
    out
}

fn update_c990(lines: &[String]) -> Vec<String> {
    let mut out = lines.to_vec();
    let idx_c001 = out.iter().position(|l| l.trim().starts_with("|C001|"));
    let idx_c990 = out.iter().position(|l| l.trim().starts_with("|C990|"));
    if let (Some(start), Some(end)) = (idx_c001, idx_c990) {
        if end >= start {
            out[end] = format!("|C990|{}|", end - start + 1);
        }
    }
    out
}

fn rebuild_block9(lines: &[String]) -> Vec<String> {
    let mut counts = BTreeMap::<String, usize>::new();
    for line in lines {
        let trim = line.trim();
        if !trim.starts_with('|') {
            continue;
        }
        let reg = trim
            .trim_matches('|')
            .split('|')
            .next()
            .unwrap_or_default()
            .to_string();
        if !reg.is_empty() {
            *counts.entry(reg).or_insert(0) += 1;
        }
    }
    counts.insert("9001".into(), 1);
    counts.insert("9990".into(), 1);
    counts.insert("9999".into(), 1);
    let reg_count = counts.len() + 1;
    counts.insert("9900".into(), reg_count);

    let mut block9 = Vec::<String>::new();
    block9.push("|9001|0|".into());
    for (reg, count) in &counts {
        block9.push(format!("|9900|{}|{}|", reg, count));
    }
    let qtd_lin9 = block9.len() + 2;
    let qtd_lin = lines.len() + qtd_lin9;
    block9.push(format!("|9990|{}|", qtd_lin9));
    block9.push(format!("|9999|{}|", qtd_lin));

    let mut out = lines.to_vec();
    out.extend(block9);
    out
}

fn parse_sped_header(
    sped_text: &str,
) -> Option<(String, String, String, String, String, String, usize)> {
    let line = sped_text.lines().find(|l| l.trim().starts_with("|0000|"))?;
    let fields = line
        .trim()
        .trim_matches('|')
        .split('|')
        .map(|v| v.trim().to_string())
        .collect::<Vec<_>>();
    Some((
        fields.get(1).cloned().unwrap_or_default(),
        only_digits(fields.get(3).map(|v| v.as_str()).unwrap_or_default()),
        only_digits(fields.get(4).map(|v| v.as_str()).unwrap_or_default()),
        only_digits(fields.get(6).map(|v| v.as_str()).unwrap_or_default()),
        fields
            .get(8)
            .cloned()
            .unwrap_or_default()
            .to_ascii_uppercase(),
        only_alphanum_upper(fields.get(9).map(|v| v.as_str()).unwrap_or_default()),
        fields.len(),
    ))
}

fn can_consolidate_sped_files(sped_files: &[NamedText]) -> bool {
    if sped_files.len() < 2 {
        return true;
    }
    let headers = sped_files
        .iter()
        .filter_map(|s| parse_sped_header(&s.text))
        .collect::<Vec<_>>();
    if headers.len() != sped_files.len() {
        return false;
    }
    let base = headers.first().cloned().unwrap();
    headers.iter().all(|h| h == &base)
}

fn split_sped_by_c100(lines: &[String]) -> (Vec<String>, Vec<Vec<String>>, Vec<String>) {
    let indexes = lines
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            if l.trim().starts_with("|C100|") {
                Some(i)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if indexes.is_empty() {
        return (lines.to_vec(), Vec::new(), Vec::new());
    }
    let suffix_start = lines
        .iter()
        .enumerate()
        .skip(*indexes.last().unwrap() + 1)
        .find(|(_, l)| l.trim().starts_with("|C990|"))
        .map(|(i, _)| i)
        .unwrap_or(lines.len());
    let prefix = lines[..indexes[0]].to_vec();
    let mut groups = Vec::<Vec<String>>::new();
    for (pos, start) in indexes.iter().enumerate() {
        let end = if pos + 1 < indexes.len() {
            indexes[pos + 1]
        } else {
            suffix_start
        };
        groups.push(lines[*start..end].to_vec());
    }
    let suffix = if suffix_start < lines.len() {
        lines[suffix_start..].to_vec()
    } else {
        Vec::new()
    };
    (prefix, groups, suffix)
}

fn consolidate_sped_outputs(
    outputs: &[(String, String, UpdateSpedStats)],
) -> Option<(String, String)> {
    let updated_lines = outputs
        .iter()
        .map(|(_, text, _)| {
            strip_block9(
                &text
                    .lines()
                    .map(|v| v.to_string())
                    .filter(|v| !v.trim().is_empty())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    let (base_prefix, _, base_suffix) = split_sped_by_c100(&updated_lines[0]);
    let base_suffix_no_c990 = base_suffix
        .into_iter()
        .filter(|l| !l.trim().starts_with("|C990|"))
        .collect::<Vec<_>>();
    let mut all_groups = Vec::<Vec<String>>::new();
    for lines in &updated_lines {
        let (prefix, groups, suffix) = split_sped_by_c100(lines);
        let suffix_no_c990 = suffix
            .into_iter()
            .filter(|l| !l.trim().starts_with("|C990|"))
            .collect::<Vec<_>>();
        if prefix != base_prefix || suffix_no_c990 != base_suffix_no_c990 {
            return None;
        }
        all_groups.extend(groups);
    }
    let mut merged = Vec::<String>::new();
    merged.extend(base_prefix);
    for group in all_groups {
        merged.extend(group);
    }
    merged.extend(base_suffix_no_c990);
    let merged = update_c990(&merged);
    let merged = rebuild_block9(&strip_block9(&merged));
    Some((
        "SPED_CONSOLIDADO_C140C141.txt".into(),
        format!("{}\r\n", merged.join("\r\n")),
    ))
}
