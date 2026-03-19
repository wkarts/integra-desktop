use std::fs;
use std::path::{Path, PathBuf};

pub fn collect_xml_files_from_dir(path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    if !path.exists() {
        return Err("Diretório não encontrado".into());
    }

    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_xml_files_from_dir(&path)?);
        } else if path
            .extension()
            .map(|ext| ext.eq_ignore_ascii_case("xml"))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }

    Ok(files)
}
