use std::path::{Path, PathBuf};

pub fn list_xml_candidates(paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .filter(|path| has_xml_or_zip_extension(path.as_path()))
        .cloned()
        .collect()
}

fn has_xml_or_zip_extension(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|item| item.to_str()).map(|item| item.to_ascii_lowercase()),
        Some(ext) if ext == "xml" || ext == "zip"
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::list_xml_candidates;

    #[test]
    fn filtra_xml_e_zip() {
        let files = vec![
            PathBuf::from("a.xml"),
            PathBuf::from("b.zip"),
            PathBuf::from("c.txt"),
        ];
        let selected = list_xml_candidates(&files);
        assert_eq!(selected.len(), 2);
    }
}
