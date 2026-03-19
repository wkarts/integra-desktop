#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod commands;
pub mod core;
pub mod storage;

use commands::{
    append_runtime_log, export_nfse_csv, export_nfse_txt, list_runtime_logs,
    load_conversion_profile, process_nfse_xml_batch, save_conversion_profile,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            process_nfse_xml_batch,
            export_nfse_txt,
            export_nfse_csv,
            save_conversion_profile,
            load_conversion_profile,
            append_runtime_log,
            list_runtime_logs,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar Integra Desktop");
}
