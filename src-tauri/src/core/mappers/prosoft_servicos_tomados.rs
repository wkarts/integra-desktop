use anyhow::Result;

use crate::core::domain::document::{ConversionProfile, NfseDocument};
use crate::core::mappers::prosoft_servicos_prestados::map_document_to_ba_prestados_line;

pub fn map_document_to_ba_tomados_line(
    document: &NfseDocument,
    profile: &ConversionProfile,
) -> Result<String> {
    let mut line = map_document_to_ba_prestados_line(document, profile)?;
    if !line.is_empty() {
        line.replace_range(0..1, "2");
    }
    Ok(line)
}

#[cfg(test)]
mod tests {
    use super::map_document_to_ba_tomados_line;
    use crate::core::domain::document::{ConversionProfile, NfseDocument};
    use crate::core::domain::party::Party;
    use crate::core::domain::taxes::Taxes;

    #[test]
    fn define_tipo_registro_tomados() {
        let profile = ConversionProfile::default();
        let document = NfseDocument {
            id: "1".into(),
            file_name: "a.xml".into(),
            provider: "x".into(),
            provider_friendly: "x".into(),
            layout: "ba_tomados".into(),
            numero: "1".into(),
            serie: "1".into(),
            emissao: "2026-03-19".into(),
            competencia: "2026-03".into(),
            chave: "x".into(),
            municipio_codigo: "123".into(),
            municipio_nome: "Cidade".into(),
            item_lista_servico: "0101".into(),
            codigo_cnae: None,
            discriminacao: "abc".into(),
            info_adic: String::new(),
            prestador: Party::default(),
            tomador: Party::default(),
            taxes: Taxes::default(),
            warnings: vec![],
        };
        let line = map_document_to_ba_tomados_line(&document, &profile).expect("ok");
        assert!(line.starts_with('2'));
    }
}
