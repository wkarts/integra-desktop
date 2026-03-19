use anyhow::{anyhow, Result};
use roxmltree::Document;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedNfe {
    pub chave: String,
    pub numero: String,
    pub serie: String,
    pub emissao: String,
    pub emitente_documento: String,
    pub destinatario_documento: String,
    pub valor_total: f64,
}

pub fn parse_nfe_xml(xml: &str) -> Result<ParsedNfe> {
    let doc = Document::parse(xml)?;

    let chave = find_text(&doc, &["chNFe", "Id"])
        .map(|v| v.trim_start_matches("NFe").to_string())
        .unwrap_or_default();
    let numero = find_text(&doc, &["nNF"]).unwrap_or_default();
    let serie = find_text(&doc, &["serie"]).unwrap_or_default();
    let emissao = find_text(&doc, &["dhEmi", "dEmi"]).unwrap_or_default();
    let emitente_documento = find_text(&doc, &["CNPJ"]).unwrap_or_default();
    let destinatario_documento = find_destinatario_documento(&doc).unwrap_or_default();
    let valor_total = find_text(&doc, &["vNF"])
        .map(|v| parse_decimal(&v))
        .unwrap_or_default();

    if numero.is_empty() && chave.is_empty() {
        return Err(anyhow!("XML não contém dados mínimos de NFe (nNF/chNFe)."));
    }

    Ok(ParsedNfe {
        chave,
        numero,
        serie,
        emissao,
        emitente_documento,
        destinatario_documento,
        valor_total,
    })
}

fn find_text(doc: &Document<'_>, tags: &[&str]) -> Option<String> {
    tags.iter().find_map(|tag| {
        doc.descendants()
            .find(|node| node.has_tag_name(*tag))
            .and_then(|node| node.text())
            .map(|text| text.trim().to_string())
    })
}

fn find_destinatario_documento(doc: &Document<'_>) -> Option<String> {
    let dest = doc.descendants().find(|node| node.has_tag_name("dest"))?;
    for tag in ["CPF", "CNPJ"] {
        if let Some(value) = dest
            .descendants()
            .find(|node| node.has_tag_name(tag))
            .and_then(|node| node.text())
        {
            return Some(value.trim().to_string());
        }
    }
    None
}

fn parse_decimal(value: &str) -> f64 {
    value
        .replace('.', "")
        .replace(',', ".")
        .parse::<f64>()
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::parse_nfe_xml;

    #[test]
    fn parse_nfe_basica() {
        let xml = r#"<NFe><infNFe Id='NFe123'><ide><nNF>77</nNF><serie>1</serie><dhEmi>2026-03-01</dhEmi></ide><emit><CNPJ>12345678000190</CNPJ></emit><dest><CPF>12345678901</CPF></dest><total><ICMSTot><vNF>100,10</vNF></ICMSTot></total><protNFe><infProt><chNFe>3512345</chNFe></infProt></protNFe></infNFe></NFe>"#;
        let parsed = parse_nfe_xml(xml).expect("deve parsear");
        assert_eq!(parsed.numero, "77");
        assert_eq!(parsed.serie, "1");
        assert_eq!(parsed.emitente_documento, "12345678000190");
        assert_eq!(parsed.destinatario_documento, "12345678901");
    }
}
