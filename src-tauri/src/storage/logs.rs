use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use tauri::{AppHandle, Manager};

pub fn append_log(app: &AppHandle, message: &str) -> Result<()> {
    let file = logs_file(app)?;
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut handle = OpenOptions::new().create(true).append(true).open(file)?;
    writeln!(handle, "{}", message)?;
    Ok(())
}

pub fn list_logs(app: &AppHandle) -> Result<Vec<String>> {
    let file = logs_file(app)?;
    if !file.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(file)?;
    Ok(content.lines().map(ToString::to_string).collect())
}

fn logs_file(app: &AppHandle) -> Result<PathBuf> {
    let dir = app.path().app_data_dir()?;
    Ok(dir.join("logs").join("runtime.log"))
}
