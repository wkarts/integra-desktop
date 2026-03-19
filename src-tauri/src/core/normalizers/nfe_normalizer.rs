use crate::core::parsers::nfe::ParsedNfe;

pub fn normalize_nfe(mut nfe: ParsedNfe) -> ParsedNfe {
    nfe.chave = nfe.chave.chars().filter(|c| c.is_ascii_digit()).collect();
    nfe.numero = nfe.numero.trim().to_string();
    nfe.serie = nfe.serie.trim().to_string();
    nfe.emitente_documento = nfe
        .emitente_documento
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    nfe.destinatario_documento = nfe
        .destinatario_documento
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    nfe
}

#[cfg(test)]
mod tests {
    use super::normalize_nfe;
    use crate::core::parsers::nfe::ParsedNfe;

    #[test]
    fn normaliza_documentos() {
        let parsed = ParsedNfe {
            chave: "NFe 35-11".into(),
            numero: " 12 ".into(),
            serie: " 001".into(),
            emissao: "2026-03-19".into(),
            emitente_documento: "12.345.678/0001-90".into(),
            destinatario_documento: "123.456.789-09".into(),
            valor_total: 10.0,
        };
        let n = normalize_nfe(parsed);
        assert_eq!(n.chave, "3511");
        assert_eq!(n.emitente_documento, "12345678000190");
        assert_eq!(n.destinatario_documento, "12345678909");
    }
}
