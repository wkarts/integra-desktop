use anyhow::{anyhow, Result};
use roxmltree::Document;
use uuid::Uuid;

use super::abrasf_v2::{
    extract_chave, fallback_text, find_first, find_first_from, first_non_empty,
    normalize_service_code, only_digits, option_string, optional_digits, parse_decimal,
    resolve_municipio, sanitize, text_from_descendant,
};
use crate::core::domain::{document::NfseDocument, party::Party, taxes::Taxes};

pub fn parse_ubaira(xml: &str, file_name: &str) -> Result<NfseDocument> {
    let document = Document::parse(xml)?;
    let inf = find_first(&document, "InfNfse").ok_or_else(|| anyhow!("InfNfse não encontrado"))?;
    let prestador_servico = find_first_from(inf, "PrestadorServico");
    let tomador = find_first_from(inf, "Tomador");
    let info_adic = first_non_empty(vec![
        text_from_descendant(inf, "OutrasInformacoes"),
        text_from_descendant(inf, "CodigoVerificacao"),
    ]);
    let municipio_raw = first_non_empty(vec![
        text_from_descendant(inf, "MunicipioIncidencia"),
        text_from_descendant(inf, "CodigoMunicipio"),
    ]);
    let (municipio_codigo, municipio_nome) = resolve_municipio(&municipio_raw);

    Ok(NfseDocument {
        id: Uuid::new_v4().to_string(),
        file_name: file_name.to_string(),
        provider: "ubaira_custom".into(),
        provider_friendly: "Ubaíra custom".into(),
        layout: "ba_prestados".into(),
        numero: text_from_descendant(inf, "Numero"),
        serie: fallback_text(text_from_descendant(inf, "Serie"), "UNICA"),
        emissao: text_from_descendant(inf, "DataEmissao"),
        competencia: text_from_descendant(inf, "Competencia"),
        chave: extract_chave(&info_adic),
        municipio_codigo,
        municipio_nome,
        item_lista_servico: normalize_service_code(&text_from_descendant(inf, "ItemListaServico")),
        codigo_cnae: optional_digits(text_from_descendant(inf, "CodigoCnae")),
        discriminacao: sanitize(&text_from_descendant(inf, "Discriminacao")),
        info_adic: sanitize(&info_adic),
        prestador: Party {
            nome: text_from_descendant(prestador_servico, "RazaoSocial"),
            documento: only_digits(&first_non_empty(vec![
                text_from_descendant(prestador_servico, "Cnpj"),
                text_from_descendant(prestador_servico, "Cpf"),
            ])),
            inscricao_municipal: option_string(text_from_descendant(
                prestador_servico,
                "InscricaoMunicipal",
            )),
            endereco: option_string(text_from_descendant(prestador_servico, "Endereco")),
            municipio_codigo: option_string(text_from_descendant(
                prestador_servico,
                "CodigoMunicipio",
            )),
            municipio_nome: None,
            uf: option_string(text_from_descendant(prestador_servico, "Uf")),
            cep: option_string(only_digits(&text_from_descendant(prestador_servico, "Cep"))),
        },
        tomador: Party {
            nome: text_from_descendant(tomador, "RazaoSocial"),
            documento: only_digits(&first_non_empty(vec![
                text_from_descendant(tomador, "Cnpj"),
                text_from_descendant(tomador, "Cpf"),
            ])),
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
