use crate::core::domain::document::NfseDocument;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StandardizeXmlOptions {
    pub target: String,
    pub remove_iss_aliquota: Option<bool>,
    pub remove_iss_value: Option<bool>,
    pub keep_only_iss_retido: Option<bool>,
    pub remove_incompatible_tags: Option<bool>,
}

pub fn export_document_to_standard_xml(document: &NfseDocument, options: &StandardizeXmlOptions) -> String {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    match options.target.as_str() {
        "abrasf_v1" => xml.push_str("<CompNfse xmlns=\"http://www.abrasf.org.br/nfse.xsd\">\n"),
        "salvador_like" => xml.push_str("<CompNfse xmlns=\"http://www.abrasf.org.br/nfse.xsd\" versao=\"salvador-like\">\n"),
        _ => xml.push_str("<CompNfse xmlns=\"http://www.abrasf.org.br/nfse.xsd\" versao=\"2.04\">\n"),
    }
    xml.push_str("  <Nfse>\n");
    xml.push_str(&format!("    <InfNfse Id=\"{}\">\n", escape_attr(&document.id)));
    write_tag(&mut xml, 6, "Numero", &document.numero);
    write_tag(&mut xml, 6, "CodigoVerificacao", &document.chave);
    write_tag(&mut xml, 6, "DataEmissao", &document.emissao);
    write_tag(&mut xml, 6, "Competencia", &document.competencia);
    xml.push_str("      <Servico>\n");
    xml.push_str("        <Valores>\n");
    write_tag(&mut xml, 10, "ValorServicos", &format_decimal(document.taxes.valor_servicos));
    write_tag(&mut xml, 10, "BaseCalculo", &format_decimal(document.taxes.base_calculo));
    if !options.keep_only_iss_retido.unwrap_or(false) && !options.remove_iss_value.unwrap_or(false) {
        write_tag(&mut xml, 10, "ValorIss", &format_decimal(document.taxes.valor_iss));
    }
    if !options.keep_only_iss_retido.unwrap_or(false) && !options.remove_iss_aliquota.unwrap_or(false) {
        write_tag(&mut xml, 10, "Aliquota", &format_decimal6(document.taxes.aliquota_iss / 100.0));
    }
    write_tag(&mut xml, 10, "ValorLiquidoNfse", &format_decimal(document.taxes.valor_liquido));
    write_tag(&mut xml, 10, "ValorPis", &format_decimal(document.taxes.valor_pis));
    write_tag(&mut xml, 10, "ValorCofins", &format_decimal(document.taxes.valor_cofins));
    write_tag(&mut xml, 10, "ValorInss", &format_decimal(document.taxes.valor_inss));
    write_tag(&mut xml, 10, "ValorIr", &format_decimal(document.taxes.valor_irrf));
    write_tag(&mut xml, 10, "IssRetido", if document.taxes.iss_retido { "1" } else { "2" });
    xml.push_str("        </Valores>\n");
    write_tag(&mut xml, 8, "ItemListaServico", &document.item_lista_servico);
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

fn format_decimal(value: f64) -> String { format!("{value:.2}") }
fn format_decimal6(value: f64) -> String { format!("{value:.6}") }

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn escape_attr(value: &str) -> String { escape_xml(value) }
