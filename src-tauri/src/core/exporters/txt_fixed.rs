use anyhow::Result;

use crate::core::domain::document::{ConversionProfile, NfseDocument};
use crate::core::mappers::prosoft_faturas::map_document_to_fatura_line;
use crate::core::mappers::prosoft_servicos_prestados::map_document_to_ba_prestados_line;
use crate::core::mappers::prosoft_servicos_tomados::map_document_to_ba_tomados_line;

pub fn export_documents_to_txt(
    documents: &[NfseDocument],
    profile: &ConversionProfile,
) -> Result<String> {
    let mut lines = Vec::new();
    for document in documents {
        let line = match profile.output_layout.as_str() {
            "ba_tomados" => map_document_to_ba_tomados_line(document, profile)?,
            "prosoft_faturas" => map_document_to_fatura_line(document, profile)?,
            _ => map_document_to_ba_prestados_line(document, profile)?,
        };
        lines.push(line);
    }
    Ok(lines.join("\r\n"))
}
