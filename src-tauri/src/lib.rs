#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod commands;
pub mod core;
pub mod storage;

use commands::{
    append_runtime_log, check_license_status, clipboard_write_text,
    convert_nfse_mixed_batch_to_standard, convert_nfse_path_batch_to_standard,
    convert_nfse_upload_batch_to_standard, convert_nfse_xml_to_standard, dialog_confirm,
    dialog_message_error, dialog_message_info, dialog_message_warning,
    dialog_pick_nfe_faturas_directory, dialog_pick_nfe_faturas_files,
    dialog_pick_nfe_faturas_legacy_file, dialog_pick_nfe_faturas_output_dir,
    dialog_pick_nfse_converter_directory, dialog_pick_nfse_converter_files,
    dialog_save_nfe_faturas_file, export_nfe_faturas_csv, export_nfe_faturas_legacy_txt,
    export_nfe_faturas_sped, export_nfe_faturas_txt, export_nfse_csv, export_nfse_txt,
    generate_local_license, get_app_meta, get_default_station_name, get_machine_fingerprint,
    get_registration_device_info, get_startup_licensing_context, guess_nfe_faturas_cnpj_filial,
    import_nfe_faturas_legacy, list_runtime_logs, load_conversion_profile, load_license_settings,
    load_nfe_faturas_settings, load_profile_bundle, process_nfe_faturas_selection,
    process_nfse_upload_batch, process_nfse_xml_batch, save_conversion_profile,
    save_license_settings, save_nfe_faturas_settings, save_profile_bundle, validate_local_license,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            process_nfse_xml_batch,
            process_nfse_upload_batch,
            process_nfe_faturas_selection,
            import_nfe_faturas_legacy,
            guess_nfe_faturas_cnpj_filial,
            dialog_pick_nfe_faturas_files,
            dialog_pick_nfe_faturas_directory,
            dialog_pick_nfe_faturas_legacy_file,
            dialog_pick_nfe_faturas_output_dir,
            dialog_save_nfe_faturas_file,
            dialog_message_info,
            dialog_message_warning,
            dialog_message_error,
            dialog_confirm,
            clipboard_write_text,
            export_nfse_txt,
            export_nfse_csv,
            convert_nfse_xml_to_standard,
            convert_nfse_upload_batch_to_standard,
            convert_nfse_mixed_batch_to_standard,
            convert_nfse_path_batch_to_standard,
            dialog_pick_nfse_converter_files,
            dialog_pick_nfse_converter_directory,
            export_nfe_faturas_txt,
            export_nfe_faturas_csv,
            export_nfe_faturas_legacy_txt,
            export_nfe_faturas_sped,
            save_conversion_profile,
            load_conversion_profile,
            save_profile_bundle,
            load_profile_bundle,
            save_nfe_faturas_settings,
            load_nfe_faturas_settings,
            save_license_settings,
            load_license_settings,
            check_license_status,
            get_machine_fingerprint,
            get_default_station_name,
            get_registration_device_info,
            get_startup_licensing_context,
            generate_local_license,
            validate_local_license,
            get_app_meta,
            append_runtime_log,
            list_runtime_logs,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar Integra Desktop");
}
