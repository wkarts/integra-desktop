use crate::core::domain::document::NfseDocument;

pub fn has_minimum_required_fields(document: &NfseDocument) -> bool {
    !document.numero.trim().is_empty()
        && !document.emissao.trim().is_empty()
        && !document.tomador.documento.trim().is_empty()
        && document.taxes.valor_servicos >= 0.0
}
