use anyhow::Result;

use crate::core::domain::document::{ConversionProfile, NfseDocument};
use crate::core::mappers::prosoft_servicos_prestados::map_document_to_ba_prestados_line;

pub fn export_documents_to_txt(documents: &[NfseDocument], profile: &ConversionProfile) -> Result<String> {
    let mut lines = Vec::new();
    for document in documents {
        lines.push(map_document_to_ba_prestados_line(document, profile)?);
    }
    Ok(lines.join("\r\n"))
}
