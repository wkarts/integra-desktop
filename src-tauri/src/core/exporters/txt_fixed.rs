use anyhow::Result;

use crate::core::domain::document::{ConversionProfile, NfseDocument};
use crate::core::mappers::prosoft_faturas::map_document_to_fatura_line;
use crate::core::mappers::prosoft_servicos_prestados::{
    map_document_to_ba_prestados_line, map_document_to_ba_prestados_record,
};
use crate::core::mappers::prosoft_servicos_tomados::map_document_to_ba_tomados_line;

pub fn export_documents_to_txt(
    documents: &[NfseDocument],
    profile: &ConversionProfile,
) -> Result<String> {
    if profile.output_layout == "ba_prestados" {
        return export_ba_prestados(documents, profile);
    }

    let mut lines = Vec::new();
    for document in documents {
        let line = match profile.output_layout.as_str() {
            "ba_tomados" => map_document_to_ba_tomados_line(document, profile)?,
            "prosoft_faturas" => map_document_to_fatura_line(document, profile)?,
            _ => map_document_to_ba_prestados_line(document, profile)?,
        };
        lines.extend(
            line.split("\r\n")
                .filter(|item| !item.trim().is_empty())
                .map(|item| item.to_string()),
        );
    }
    Ok(lines.join("\r\n"))
}

fn export_ba_prestados(documents: &[NfseDocument], profile: &ConversionProfile) -> Result<String> {
    let mut ordered = documents.to_vec();
    ordered.sort_by_key(|item| {
        (
            emissao_sort_key(&item.emissao),
            nota_sort_key(&item.numero),
            item.id.clone(),
        )
    });

    let mut lines = Vec::new();
    for document in &ordered {
        let record = map_document_to_ba_prestados_record(document, profile)?;
        validate_main_line(&record.main_line, document)?;
        validate_obs_rule(&record.main_line, record.obs_line.as_deref(), document)?;
        lines.push(record.main_line);
        if let Some(obs) = record.obs_line {
            lines.push(obs);
        }
    }

    if lines.iter().any(|item| item.trim().is_empty()) {
        anyhow::bail!("arquivo final contém linhas vazias no layout ba_prestados");
    }

    Ok(lines.join("\r\n"))
}

fn validate_main_line(line: &str, document: &NfseDocument) -> Result<()> {
    if line.contains('\n') || line.contains('\r') {
        anyhow::bail!(
            "NF {} contém quebra de linha indevida no registro principal",
            document.numero
        );
    }
    if line.chars().count() != 1172 {
        anyhow::bail!(
            "NF {} possui linha principal com tamanho inválido: esperado 1172, obtido {}",
            document.numero,
            line.chars().count()
        );
    }
    Ok(())
}

fn validate_obs_rule(
    main_line: &str,
    obs_line: Option<&str>,
    document: &NfseDocument,
) -> Result<()> {
    let obs_flag = main_line.chars().nth(909).unwrap_or('0');
    let has_obs = obs_line.is_some();

    if has_obs && obs_flag != '1' {
        anyhow::bail!(
            "NF {} possui *OBS, mas campo 910 não está marcado com 1",
            document.numero
        );
    }
    if !has_obs && obs_flag == '1' {
        anyhow::bail!(
            "NF {} está com campo 910=1 sem linha *OBS correspondente",
            document.numero
        );
    }

    if let Some(obs) = obs_line {
        if !obs.starts_with("*OBS") {
            anyhow::bail!("NF {} possui linha de observação inválida", document.numero);
        }
        if obs.contains('\n') || obs.contains('\r') {
            anyhow::bail!(
                "NF {} possui quebra de linha indevida na observação estendida",
                document.numero
            );
        }
    }
    Ok(())
}

fn emissao_sort_key(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some((y, m, d)) = parse_ymd(trimmed) {
        return format!("{y:04}{m:02}{d:02}");
    }
    if let Some((d, m, y)) = parse_dmy(trimmed) {
        return format!("{y:04}{m:02}{d:02}");
    }
    only_digits(trimmed)
}

fn parse_ymd(raw: &str) -> Option<(u32, u32, u32)> {
    let parts = raw.split('-').collect::<Vec<_>>();
    if parts.len() != 3 || parts[0].len() != 4 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

fn parse_dmy(raw: &str) -> Option<(u32, u32, u32)> {
    let parts = raw.split('/').collect::<Vec<_>>();
    if parts.len() != 3 || parts[0].len() != 2 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

fn nota_sort_key(raw: &str) -> u64 {
    only_digits(raw).parse::<u64>().unwrap_or_default()
}

fn only_digits(value: &str) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

#[cfg(test)]
mod tests {
    use super::export_documents_to_txt;
    use crate::core::domain::document::{ConversionProfile, NfseDocument};
    use crate::core::domain::party::Party;
    use crate::core::domain::taxes::Taxes;

    fn sample_document() -> NfseDocument {
        NfseDocument {
            id: "doc-1".into(),
            file_name: "a.xml".into(),
            provider: "x".into(),
            provider_friendly: "x".into(),
            layout: "ba_prestados".into(),
            numero: "123".into(),
            serie: "UNICA".into(),
            emissao: "2026-03-25".into(),
            competencia: "2026-03".into(),
            chave: "chave".into(),
            municipio_codigo: "2927408".into(),
            municipio_nome: "SALVADOR".into(),
            item_lista_servico: "0101".into(),
            codigo_cnae: None,
            discriminacao: "Servico de teste com observacao longa para gerar linha estendida"
                .into(),
            info_adic: String::new(),
            prestador: Party::default(),
            tomador: Party::default(),
            taxes: Taxes::default(),
            warnings: vec![],
        }
    }

    #[test]
    fn exporta_linha_principal_com_1172_posicoes() {
        let mut profile = ConversionProfile::default();
        profile.output_layout = "ba_prestados".into();
        profile.obs_extended = "never".into();
        let output = export_documents_to_txt(&[sample_document()], &profile).expect("ok");
        let first_line = output.lines().next().unwrap_or_default();
        assert_eq!(first_line.chars().count(), 1172);
    }

    #[test]
    fn exporta_obs_imediatamente_abaixo_da_nf() {
        let mut profile = ConversionProfile::default();
        profile.output_layout = "ba_prestados".into();
        profile.obs_extended = "always".into();
        let output = export_documents_to_txt(&[sample_document()], &profile).expect("ok");
        let lines = output.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        assert!(lines[1].starts_with("*OBS"));
    }
}
