use tauri::AppHandle;

#[tauri::command]
pub fn append_runtime_log(message: String, app: AppHandle) -> Result<(), String> {
    crate::storage::logs::append_log(&app, &message).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_runtime_logs(app: AppHandle) -> Result<Vec<String>, String> {
    crate::storage::logs::list_logs(&app).map_err(|e| e.to_string())
}
