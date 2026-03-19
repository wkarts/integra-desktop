use crate::core::domain::document::NfseDocument;

pub fn collect_document_warnings(document: &NfseDocument) -> Vec<String> {
    let mut warnings = Vec::new();

    if document.chave.trim().is_empty() {
        warnings.push("Chave NFS-e não identificada automaticamente.".into());
    }
    if document.municipio_codigo.trim().is_empty() {
        warnings.push("Município da prestação sem código IBGE resolvido.".into());
    }
    if document.taxes.base_calculo <= 0.0 {
        warnings.push("Base de cálculo zerada ou ausente no XML.".into());
    }
    if document.discriminacao.trim().is_empty() && document.info_adic.trim().is_empty() {
        warnings.push("Documento sem observação/discriminação.".into());
    }

    warnings
}
