#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod commands;
pub mod core;
pub mod storage;

use commands::{
    append_runtime_log, check_license_status, export_nfse_csv, export_nfse_txt, get_app_meta,
    get_machine_fingerprint, list_runtime_logs, load_conversion_profile, load_license_settings,
    load_profile_bundle, process_nfse_upload_batch, process_nfse_xml_batch,
    save_conversion_profile, save_license_settings, save_profile_bundle,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            process_nfse_xml_batch,
            process_nfse_upload_batch,
            export_nfse_txt,
            export_nfse_csv,
            save_conversion_profile,
            load_conversion_profile,
            save_profile_bundle,
            load_profile_bundle,
            save_license_settings,
            load_license_settings,
            check_license_status,
            get_machine_fingerprint,
            get_app_meta,
            append_runtime_log,
            list_runtime_logs,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar Integra Desktop");
}
