use anyhow::Result;

use crate::core::domain::document::{ConversionProfile, NfseDocument};

pub fn export_documents_to_csv(
    documents: &[NfseDocument],
    _profile: &ConversionProfile,
) -> Result<String> {
    let mut rows = Vec::new();
    rows.push(vec![
        "provider".into(),
        "numero".into(),
        "serie".into(),
        "emissao".into(),
        "competencia".into(),
        "municipio_nome".into(),
        "municipio_codigo".into(),
        "prestador_razao".into(),
        "prestador_doc".into(),
        "tomador_razao".into(),
        "tomador_doc".into(),
        "codigo_servico".into(),
        "valor_servicos".into(),
        "base_calculo".into(),
        "valor_iss".into(),
        "aliquota_iss".into(),
        "iss_retido".into(),
        "irrf".into(),
        "pis".into(),
        "cofins".into(),
        "csll".into(),
        "inss".into(),
        "valor_liquido".into(),
        "chave".into(),
        "observacao".into(),
    ]);

    for item in documents {
        rows.push(vec![
            item.provider_friendly.clone(),
            item.numero.clone(),
            item.serie.clone(),
            item.emissao.clone(),
            item.competencia.clone(),
            item.municipio_nome.clone(),
            item.municipio_codigo.clone(),
            item.prestador.nome.clone(),
            item.prestador.documento.clone(),
            item.tomador.nome.clone(),
            item.tomador.documento.clone(),
            item.item_lista_servico.clone(),
            item.taxes.valor_servicos.to_string(),
            item.taxes.base_calculo.to_string(),
            item.taxes.valor_iss.to_string(),
            item.taxes.aliquota_iss.to_string(),
            if item.taxes.iss_retido {
                "1".into()
            } else {
                "0".into()
            },
            item.taxes.valor_irrf.to_string(),
            item.taxes.valor_pis.to_string(),
            item.taxes.valor_cofins.to_string(),
            item.taxes.valor_csll.to_string(),
            item.taxes.valor_inss.to_string(),
            item.taxes.valor_liquido.to_string(),
            item.chave.clone(),
            item.discriminacao.clone(),
        ]);
    }

    Ok(rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(escape_csv)
                .collect::<Vec<_>>()
                .join(";")
        })
        .collect::<Vec<_>>()
        .join("\r\n"))
}

fn escape_csv(value: String) -> String {
    if value.contains(';') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value
    }
}
