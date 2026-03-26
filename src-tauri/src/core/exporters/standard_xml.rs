use crate::core::domain::document::{
    ConversionProfile, FieldAction, FieldRule, NfseDocument,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StandardizeXmlOptions {
    pub target: String,
    pub remove_iss_aliquota: Option<bool>,
    pub remove_iss_value: Option<bool>,
    pub keep_only_iss_retido: Option<bool>,
    pub remove_incompatible_tags: Option<bool>,
    pub apply_profile_rules: Option<bool>,
    pub remove_codigo_verificacao: Option<bool>,
    pub remove_tomador_endereco: Option<bool>,
    pub remove_prestador_im: Option<bool>,
    pub remove_tomador_im: Option<bool>,
    pub remove_cnae: Option<bool>,
    pub remove_discriminacao: Option<bool>,
    pub remove_info_adicional: Option<bool>,
}

pub fn standardize_document_for_xml(
    document: &NfseDocument,
    profile: Option<&ConversionProfile>,
    options: &StandardizeXmlOptions,
) -> NfseDocument {
    let mut normalized = document.clone();

    if options.apply_profile_rules.unwrap_or(true) {
        if let Some(profile) = profile {
            normalized.taxes.base_calculo =
                apply_number_rule(normalized.taxes.base_calculo, &profile.field_rules.base_calculo);
            normalized.taxes.aliquota_iss =
                apply_number_rule(normalized.taxes.aliquota_iss, &profile.field_rules.iss_aliquota);
            normalized.taxes.valor_iss =
                apply_number_rule(normalized.taxes.valor_iss, &profile.field_rules.valor_iss);
            normalized.taxes.valor_liquido =
                apply_number_rule(normalized.taxes.valor_liquido, &profile.field_rules.valor_liquido);
            normalized.taxes.valor_irrf =
                apply_number_rule(normalized.taxes.valor_irrf, &profile.field_rules.valor_irrf);
            normalized.taxes.valor_inss =
                apply_number_rule(normalized.taxes.valor_inss, &profile.field_rules.valor_inss);
            normalized.taxes.valor_pis =
                apply_number_rule(normalized.taxes.valor_pis, &profile.field_rules.valor_pis);
            normalized.taxes.valor_cofins =
                apply_number_rule(normalized.taxes.valor_cofins, &profile.field_rules.valor_cofins);
            normalized.taxes.valor_csll =
                apply_number_rule(normalized.taxes.valor_csll, &profile.field_rules.valor_csll);
            normalized.taxes.iss_retido =
                apply_bool_rule(normalized.taxes.iss_retido, &profile.field_rules.iss_retido);
            normalized.item_lista_servico =
                apply_string_rule(&normalized.item_lista_servico, &profile.field_rules.codigo_servico);
            normalized.municipio_codigo =
                apply_string_rule(&normalized.municipio_codigo, &profile.field_rules.municipio);
            normalized.municipio_nome =
                apply_string_rule(&normalized.municipio_nome, &profile.field_rules.municipio);
            normalized.serie = apply_string_rule(&normalized.serie, &profile.field_rules.serie);
            normalized.numero = apply_string_rule(&normalized.numero, &profile.field_rules.numero);
            normalized.emissao =
                apply_string_rule(&normalized.emissao, &profile.field_rules.data_emissao);
            normalized.competencia =
                apply_string_rule(&normalized.competencia, &profile.field_rules.data_competencia);
            normalized.info_adic =
                apply_string_rule(&normalized.info_adic, &profile.field_rules.campos_complementares);
            normalized.discriminacao =
                apply_string_rule(&normalized.discriminacao, &profile.field_rules.observacao);
        }
    }

    if options.remove_codigo_verificacao.unwrap_or(false) {
        normalized.chave.clear();
    }
    if options.remove_tomador_endereco.unwrap_or(false) {
        normalized.tomador.endereco = None;
        normalized.tomador.cep = None;
        normalized.tomador.uf = None;
        normalized.tomador.municipio_codigo = None;
        normalized.tomador.municipio_nome = None;
    }
    if options.remove_prestador_im.unwrap_or(false) {
        normalized.prestador.inscricao_municipal = None;
    }
    if options.remove_tomador_im.unwrap_or(false) {
        normalized.tomador.inscricao_municipal = None;
    }
    if options.remove_cnae.unwrap_or(false) {
        normalized.codigo_cnae = None;
    }
    if options.remove_discriminacao.unwrap_or(false) {
        normalized.discriminacao.clear();
    }
    if options.remove_info_adicional.unwrap_or(false) {
        normalized.info_adic.clear();
    }

    normalized
}

pub fn export_document_to_standard_xml(
    document: &NfseDocument,
    options: &StandardizeXmlOptions,
) -> String {
    let mut xml = String::new();
    xml.push_str("<?xml version="1.0" encoding="UTF-8"?>\n");
    match options.target.as_str() {
        "abrasf_v1" => xml.push_str("<CompNfse xmlns="http://www.abrasf.org.br/nfse.xsd">\n"),
        "salvador_like" => xml.push_str(
            "<CompNfse xmlns="http://www.abrasf.org.br/nfse.xsd" versao="salvador-like">\n",
        ),
        _ => {
            xml.push_str("<CompNfse xmlns="http://www.abrasf.org.br/nfse.xsd" versao="2.04">\n")
        }
    }
    xml.push_str("  <Nfse>\n");
    xml.push_str(&format!(
        "    <InfNfse Id="{}">\n",
        escape_attr(&document.id)
    ));
    write_tag(&mut xml, 6, "Numero", &document.numero);
    write_tag(&mut xml, 6, "CodigoVerificacao", &document.chave);
    write_tag(&mut xml, 6, "DataEmissao", &document.emissao);
    write_tag(&mut xml, 6, "Competencia", &document.competencia);
    xml.push_str("      <Servico>\n");
    xml.push_str("        <Valores>\n");
    write_tag(
        &mut xml,
        10,
        "ValorServicos",
        &format_decimal(document.taxes.valor_servicos),
    );
    write_tag(
        &mut xml,
        10,
        "BaseCalculo",
        &format_decimal(document.taxes.base_calculo),
    );
    if !options.keep_only_iss_retido.unwrap_or(false) && !options.remove_iss_value.unwrap_or(false)
    {
        write_tag(
            &mut xml,
            10,
            "ValorIss",
            &format_decimal(document.taxes.valor_iss),
        );
    }
    if !options.keep_only_iss_retido.unwrap_or(false)
        && !options.remove_iss_aliquota.unwrap_or(false)
    {
        write_tag(
            &mut xml,
            10,
            "Aliquota",
            &format_decimal6(document.taxes.aliquota_iss / 100.0),
        );
    }
    write_tag(
        &mut xml,
        10,
        "ValorLiquidoNfse",
        &format_decimal(document.taxes.valor_liquido),
    );
    write_tag(
        &mut xml,
        10,
        "ValorPis",
        &format_decimal(document.taxes.valor_pis),
    );
    write_tag(
        &mut xml,
        10,
        "ValorCofins",
        &format_decimal(document.taxes.valor_cofins),
    );
    write_tag(
        &mut xml,
        10,
        "ValorInss",
        &format_decimal(document.taxes.valor_inss),
    );
    write_tag(
        &mut xml,
        10,
        "ValorIr",
        &format_decimal(document.taxes.valor_irrf),
    );
    write_tag(
        &mut xml,
        10,
        "IssRetido",
        if document.taxes.iss_retido { "1" } else { "2" },
    );
    xml.push_str("        </Valores>\n");
    write_tag(
        &mut xml,
        8,
        "ItemListaServico",
        &document.item_lista_servico,
    );
    if let Some(cnae) = &document.codigo_cnae {
        write_tag(&mut xml, 8, "CodigoCnae", cnae);
    }
    write_tag(&mut xml, 8, "Discriminacao", &document.discriminacao);
    write_tag(&mut xml, 8, "CodigoMunicipio", &document.municipio_codigo);
    xml.push_str("      </Servico>\n");
    xml.push_str("      <PrestadorServico>\n");
    if document.prestador.documento.len() <= 11 {
        write_tag(&mut xml, 8, "Cpf", &document.prestador.documento);
    } else {
        write_tag(&mut xml, 8, "Cnpj", &document.prestador.documento);
    }
    if let Some(im) = &document.prestador.inscricao_municipal {
        write_tag(&mut xml, 8, "InscricaoMunicipal", im);
    }
    xml.push_str("      </PrestadorServico>\n");
    xml.push_str("      <TomadorServico>\n");
    xml.push_str("        <IdentificacaoTomador>\n");
    xml.push_str("          <CpfCnpj>\n");
    if document.tomador.documento.len() <= 11 {
        write_tag(&mut xml, 12, "Cpf", &document.tomador.documento);
    } else {
        write_tag(&mut xml, 12, "Cnpj", &document.tomador.documento);
    }
    xml.push_str("          </CpfCnpj>\n");
    if let Some(im) = &document.tomador.inscricao_municipal {
        write_tag(&mut xml, 10, "InscricaoMunicipal", im);
    }
    xml.push_str("        </IdentificacaoTomador>\n");
    write_tag(&mut xml, 8, "RazaoSocial", &document.tomador.nome);
    if !options.remove_incompatible_tags.unwrap_or(true) {
        if let Some(endereco) = &document.tomador.endereco {
            write_tag(&mut xml, 8, "EnderecoLivre", endereco);
        }
    }
    xml.push_str("      </TomadorServico>\n");
    xml.push_str("    </InfNfse>\n");
    xml.push_str("  </Nfse>\n");
    xml.push_str("</CompNfse>\n");
    xml
}

fn apply_number_rule(source: f64, rule: &FieldRule) -> f64 {
    match rule.action {
        FieldAction::Source => source,
        FieldAction::Zero | FieldAction::Empty | FieldAction::Ignore => 0.0,
        FieldAction::Constant => rule.value.as_deref().map(parse_decimal).unwrap_or(0.0),
    }
}

fn apply_bool_rule(source: bool, rule: &FieldRule) -> bool {
    match rule.action {
        FieldAction::Source => source,
        FieldAction::Zero | FieldAction::Empty | FieldAction::Ignore => false,
        FieldAction::Constant => matches!(
            rule.value.as_deref().unwrap_or("").trim().to_ascii_lowercase().as_str(),
            "1" | "s" | "sim" | "true" | "y" | "yes"
        ),
    }
}

fn apply_string_rule(source: &str, rule: &FieldRule) -> String {
    match rule.action {
        FieldAction::Source => source.to_string(),
        FieldAction::Zero => "0".to_string(),
        FieldAction::Empty | FieldAction::Ignore => String::new(),
        FieldAction::Constant => rule.value.clone().unwrap_or_default(),
    }
}

fn parse_decimal(value: &str) -> f64 {
    value.replace('.', "").replace(',', ".").parse::<f64>().unwrap_or(0.0)
}

fn write_tag(xml: &mut String, indent: usize, tag: &str, value: &str) {
    if value.trim().is_empty() {
        return;
    }
    xml.push_str(&" ".repeat(indent));
    xml.push('<');
    xml.push_str(tag);
    xml.push('>');
    xml.push_str(&escape_xml(value));
    xml.push_str("</");
    xml.push_str(tag);
    xml.push_str(">\n");
}

fn format_decimal(value: f64) -> String {
    format!("{value:.2}")
}

fn format_decimal6(value: f64) -> String {
    format!("{value:.6}")
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace(''', "&apos;")
}

fn escape_attr(value: &str) -> String {
    escape_xml(value)
}
