use crate::core::domain::document::NfseDocument;

pub fn normalize_nfse_document(mut document: NfseDocument) -> NfseDocument {
    document.numero = document.numero.trim().to_string();
    document.serie = document.serie.trim().to_string();
    document.municipio_codigo = document.municipio_codigo.trim().to_string();
    document.municipio_nome = document.municipio_nome.trim().to_uppercase();
    document.discriminacao = document.discriminacao.trim().to_string();
    document.info_adic = document.info_adic.trim().to_string();
    document
}
