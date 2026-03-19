use anyhow::{anyhow, Result};
use regex::Regex;
use roxmltree::{Document, Node};
use uuid::Uuid;

use crate::core::domain::{document::NfseDocument, party::Party, taxes::Taxes};

pub fn parse(xml: &str, file_name: &str) -> Result<NfseDocument> {
    let document = Document::parse(xml)?;
    let inf = find_first(&document, "InfNfse").ok_or_else(|| anyhow!("InfNfse não encontrado"))?;
    let info_adic = text_from_descendant(inf, "OutrasInformacoes");
    let provider_friendly = if document.root_element().tag_name().namespace().unwrap_or_default().contains("abrasf") {
        "WebISS/ABRASF"
    } else {
        "CompNfse genérico"
    };

    let prestador_servico = find_first_from(inf, "PrestadorServico");
    let tomador = find_first_from(inf, "Tomador");

    let municipio_raw = first_non_empty(vec![
        text_from_descendant(inf, "MunicipioIncidencia"),
        text_from_descendant(inf, "CodigoMunicipio"),
    ]);

    let (municipio_codigo, municipio_nome) = resolve_municipio(&municipio_raw);

    Ok(NfseDocument {
        id: Uuid::new_v4().to_string(),
        file_name: file_name.to_string(),
        provider: "webiss_abrasf_202".into(),
        provider_friendly: provider_friendly.into(),
        layout: "ba_prestados".into(),
        numero: text_from_descendant(inf, "Numero"),
        serie: fallback_text(text_from_descendant(inf, "Serie"), "A"),
        emissao: text_from_descendant(inf, "DataEmissao"),
        competencia: text_from_descendant(inf, "Competencia"),
        chave: first_non_empty(vec![text_from_descendant(inf, "ChaveAcesso"), extract_chave(&info_adic)]),
        municipio_codigo,
        municipio_nome,
        item_lista_servico: normalize_service_code(&text_from_descendant(inf, "ItemListaServico")),
        codigo_cnae: optional_digits(text_from_descendant(inf, "CodigoCnae")),
        discriminacao: sanitize(&text_from_descendant(inf, "Discriminacao")),
        info_adic: sanitize(&info_adic),
        prestador: Party {
            nome: text_from_descendant(prestador_servico, "RazaoSocial"),
            documento: only_digits(&first_non_empty(vec![text_from_descendant(prestador_servico, "Cnpj"), text_from_descendant(prestador_servico, "Cpf")])),
            inscricao_municipal: option_string(text_from_descendant(prestador_servico, "InscricaoMunicipal")),
            endereco: option_string(text_from_descendant(prestador_servico, "Endereco")),
            municipio_codigo: option_string(text_from_descendant(prestador_servico, "CodigoMunicipio")),
            municipio_nome: None,
            uf: option_string(text_from_descendant(prestador_servico, "Uf")),
            cep: option_string(only_digits(&text_from_descendant(prestador_servico, "Cep"))),
        },
        tomador: Party {
            nome: text_from_descendant(tomador, "RazaoSocial"),
            documento: only_digits(&first_non_empty(vec![text_from_descendant(tomador, "Cnpj"), text_from_descendant(tomador, "Cpf")])),
            inscricao_municipal: option_string(text_from_descendant(tomador, "InscricaoMunicipal")),
            endereco: option_string(text_from_descendant(tomador, "Endereco")),
            municipio_codigo: option_string(text_from_descendant(tomador, "CodigoMunicipio")),
            municipio_nome: None,
            uf: option_string(text_from_descendant(tomador, "Uf")),
            cep: option_string(only_digits(&text_from_descendant(tomador, "Cep"))),
        },
        taxes: Taxes {
            valor_servicos: parse_decimal(&text_from_descendant(inf, "ValorServicos")),
            base_calculo: parse_decimal(&text_from_descendant(inf, "BaseCalculo")),
            valor_iss: parse_decimal(&text_from_descendant(inf, "ValorIss")),
            aliquota_iss: parse_decimal(&text_from_descendant(inf, "Aliquota")),
            valor_liquido: parse_decimal(&text_from_descendant(inf, "ValorLiquidoNfse")),
            valor_irrf: parse_decimal(&text_from_descendant(inf, "ValorIr")),
            valor_pis: parse_decimal(&text_from_descendant(inf, "ValorPis")),
            valor_cofins: parse_decimal(&text_from_descendant(inf, "ValorCofins")),
            valor_csll: parse_decimal(&text_from_descendant(inf, "ValorCsll")),
            valor_inss: parse_decimal(&text_from_descendant(inf, "ValorInss")),
            iss_retido: text_from_descendant(inf, "IssRetido") == "1",
        },
        warnings: Vec::new(),
    })
}

pub(crate) fn find_first<'a>(document: &'a Document<'a>, tag: &str) -> Option<Node<'a, 'a>> {
    document.descendants().find(|node| node.is_element() && node.tag_name().name() == tag)
}

pub(crate) fn find_first_from<'a>(node: Node<'a, 'a>, tag: &str) -> Node<'a, 'a> {
    node.descendants()
        .find(|item| item.is_element() && item.tag_name().name() == tag)
        .unwrap_or(node)
}

pub(crate) fn text_from_descendant<'a>(node: Node<'a, 'a>, tag: &str) -> String {
    node.descendants()
        .find(|item| item.is_element() && item.tag_name().name() == tag)
        .and_then(|item| item.text())
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub(crate) fn parse_decimal(value: &str) -> f64 {
    let mut raw = value.trim().replace(' ', "");
    if raw.contains(',') && raw.contains('.') {
        raw = raw.replace('.', "").replace(',', ".");
    } else if raw.contains(',') {
        raw = raw.replace(',', ".");
    }
    raw.parse::<f64>().unwrap_or(0.0)
}

pub(crate) fn only_digits(value: &str) -> String {
    value.chars().filter(|c| c.is_ascii_digit()).collect()
}

pub(crate) fn optional_digits(value: String) -> Option<String> {
    let digits = only_digits(&value);
    if digits.is_empty() { None } else { Some(digits) }
}

pub(crate) fn option_string(value: String) -> Option<String> {
    if value.trim().is_empty() { None } else { Some(value) }
}

pub(crate) fn extract_chave(text: &str) -> String {
    Regex::new(r"(\d{44})")
        .ok()
        .and_then(|regex| regex.captures(text))
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_default()
}

pub(crate) fn sanitize(value: &str) -> String {
    value
        .replace("<BR />", " ")
        .replace("<BR/>", " ")
        .replace("<br />", " ")
        .replace("<br/>", " ")
        .replace("\\s\\n", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn normalize_service_code(value: &str) -> String {
    let digits = only_digits(value);
    if digits.len() <= 4 && !digits.is_empty() {
        format!("{:0>4}", digits)
    } else {
        digits
    }
}

pub(crate) fn fallback_text(primary: String, fallback: &str) -> String {
    if primary.trim().is_empty() { fallback.to_string() } else { primary }
}

pub(crate) fn first_non_empty(values: Vec<String>) -> String {
    values.into_iter().find(|value| !value.trim().is_empty()).unwrap_or_default()
}

pub(crate) fn resolve_municipio(raw: &str) -> (String, String) {
    let normalized = raw.trim();
    match normalized {
        "2928703" => ("2928703".into(), "SANTO ANTÔNIO DE JESUS".into()),
        "2932101" => ("2932101".into(), "UBAÍRA".into()),
        "2927408" => ("2927408".into(), "SALVADOR".into()),
        "2930709" => ("2930709".into(), "SIMÕES FILHO".into()),
        "2910809" | "2910800" => ("2910809".into(), "FEIRA DE SANTANA".into()),
        "2905700" => ("2905700".into(), "CAMAÇARI".into()),
        "2933307" => ("2933307".into(), "VITÓRIA DA CONQUISTA".into()),
        _ if normalized.chars().all(|c| c.is_ascii_digit()) => (normalized.to_string(), normalized.to_string()),
        _ => {
            let upper = normalized.to_uppercase();
            let code = match upper.as_str() {
                "SANTO ANTONIO DE JESUS" | "SANTO ANTÔNIO DE JESUS" => "2928703",
                "UBAIRA" | "UBAÍRA" => "2932101",
                "FEIRA DE SANTANA" => "2910809",
                _ => "",
            };
            (code.to_string(), upper)
        }
    }
}
