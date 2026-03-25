use anyhow::Result;

use crate::core::domain::document::{ConversionProfile, FieldAction, FieldRule, NfseDocument};

#[derive(Debug, Clone)]
pub struct ProsoftPrestadoRecord {
    pub main_line: String,
    pub obs_line: Option<String>,
}

pub fn map_document_to_ba_prestados_record(
    document: &NfseDocument,
    profile: &ConversionProfile,
) -> Result<ProsoftPrestadoRecord> {
    let (main_line, observacao) = build_main_line_and_observation(document, profile)?;
    let emit_obs = should_emit_obs(&observacao, &profile.obs_extended);
    let obs_line = if emit_obs {
        Some(format!(
            "*OBS{}",
            sanitize_obs_line(&observacao)
                .chars()
                .take(7000)
                .collect::<String>()
        ))
    } else {
        None
    };

    Ok(ProsoftPrestadoRecord {
        main_line,
        obs_line,
    })
}

pub fn map_document_to_ba_prestados_line(
    document: &NfseDocument,
    profile: &ConversionProfile,
) -> Result<String> {
    let record = map_document_to_ba_prestados_record(document, profile)?;
    let mut line = record.main_line;
    if let Some(obs_line) = record.obs_line {
        line.push_str("\r\n");
        line.push_str(&obs_line);
    }
    Ok(line)
}

fn build_main_line_and_observation(
    document: &NfseDocument,
    profile: &ConversionProfile,
) -> Result<(String, String)> {
    let len = 1172usize;
    let mut chars = vec![' '; len];

    let serie = sanitize_serie(&apply_string_rule(
        &document.serie,
        &profile.field_rules.serie,
    ));
    let numero = only_digits(&apply_string_rule(
        &document.numero,
        &profile.field_rules.numero,
    ));
    let service_code = apply_string_rule(
        &document.item_lista_servico,
        &profile.field_rules.codigo_servico,
    );
    let code_4 = if service_code.len() > 4 {
        service_code[..4].to_string()
    } else {
        format!("{:0>4}", service_code)
    };
    let code_12 = if service_code.len() > 4 {
        service_code.clone()
    } else {
        String::new()
    };
    let emissao = apply_string_rule(&document.emissao, &profile.field_rules.data_emissao);
    let competencia =
        apply_string_rule(&document.competencia, &profile.field_rules.data_competencia);
    let observacao_base = first_non_empty(vec![
        document.discriminacao.clone(),
        document.info_adic.clone(),
    ]);
    let observacao = apply_string_rule(
        format!("{} {}", observacao_base, competencia).trim(),
        &profile.field_rules.observacao,
    );
    let observacao_curta = observacao.chars().take(40).collect::<String>();
    let municipio_nome = sanitize_text(&apply_string_rule(
        &document.municipio_nome,
        &profile.field_rules.municipio,
    ))
    .chars()
    .take(20)
    .collect::<String>();
    let municipio_codigo = only_digits(&document.municipio_codigo);
    let tomador_doc = only_digits(&document.tomador.documento);

    let base_calculo = apply_number_rule(
        document.taxes.base_calculo,
        &profile.field_rules.base_calculo,
    );
    let aliquota = apply_number_rule(
        document.taxes.aliquota_iss,
        &profile.field_rules.iss_aliquota,
    );
    let valor_iss = apply_number_rule(document.taxes.valor_iss, &profile.field_rules.valor_iss);
    let valor_irrf = apply_number_rule(document.taxes.valor_irrf, &profile.field_rules.valor_irrf);
    let valor_inss = apply_number_rule(document.taxes.valor_inss, &profile.field_rules.valor_inss);
    let valor_pis = apply_number_rule(document.taxes.valor_pis, &profile.field_rules.valor_pis);
    let valor_cofins = apply_number_rule(
        document.taxes.valor_cofins,
        &profile.field_rules.valor_cofins,
    );
    let _valor_csll = apply_number_rule(document.taxes.valor_csll, &profile.field_rules.valor_csll);
    let _valor_liquido = apply_number_rule(
        document.taxes.valor_liquido,
        &profile.field_rules.valor_liquido,
    );
    let iss_retido = apply_bool_rule(document.taxes.iss_retido, &profile.field_rules.iss_retido);

    put(&mut chars, 1, 1, "1", false, ' ');
    put(&mut chars, 2, 6, &ddmmaa(&emissao), true, ' ');
    put(&mut chars, 8, 5, &serie, false, ' ');
    put(&mut chars, 31, 4, &code_4, true, ' ');
    put(&mut chars, 35, 1, "0", false, ' ');
    put(
        &mut chars,
        36,
        14,
        &format_money(document.taxes.valor_servicos),
        true,
        ' ',
    );
    put(&mut chars, 50, 5, &format_money_short(aliquota), true, ' ');
    put(&mut chars, 55, 14, &format_money(valor_iss), true, ' ');
    put(&mut chars, 69, 40, &observacao_curta, false, ' ');
    put(&mut chars, 117, 14, &format_money(base_calculo), true, ' ');
    put(&mut chars, 173, 14, &format_money(base_calculo), true, ' ');
    put(&mut chars, 229, 20, &municipio_nome, false, ' ');
    put(&mut chars, 249, 14, &tomador_doc, true, '0');
    put(&mut chars, 263, 14, &format_money(valor_irrf), true, ' ');
    put(&mut chars, 277, 14, &format_money(valor_inss), true, ' ');
    put(&mut chars, 294, 14, &format_money(valor_iss), true, ' ');
    put(
        &mut chars,
        308,
        1,
        if iss_retido { "R" } else { "P" },
        false,
        ' ',
    );
    put(
        &mut chars,
        538,
        1,
        &profile.responsavel_retencao,
        false,
        ' ',
    );
    put(&mut chars, 582, 2, "BA", false, ' ');

    if iss_retido {
        put(&mut chars, 584, 14, &format_money(base_calculo), true, ' ');
        put(&mut chars, 598, 5, &format_money_short(aliquota), true, ' ');
        put(&mut chars, 603, 14, &format_money(valor_iss), true, ' ');
        put(&mut chars, 617, 8, &ddmmaaaa(&emissao), false, ' ');
    }

    put(&mut chars, 625, 12, &code_12, false, ' ');
    put(&mut chars, 637, 5, &profile.modelo_nf, false, ' ');
    put(&mut chars, 642, 2, &profile.motivo_retencao, false, ' ');
    let tipo_documento =
        apply_string_rule(&profile.tipo_documento, &profile.field_rules.tipo_documento);
    let especie_documento = apply_string_rule(
        &profile.especie_documento,
        &profile.field_rules.especie_documento,
    );
    let natureza_operacao = apply_string_rule(
        &profile.operacao_nota,
        &profile.field_rules.natureza_operacao,
    );
    put(&mut chars, 644, 2, &natureza_operacao, false, ' ');
    put(&mut chars, 646, 3, &profile.tipo_recolhimento, false, ' ');
    put(&mut chars, 650, 3, &tipo_documento, false, ' ');
    put(
        &mut chars,
        654,
        5,
        &municipio_codigo.chars().take(5).collect::<String>(),
        false,
        ' ',
    );
    put(&mut chars, 659, 10, &profile.situacao_documento, false, ' ');
    put(&mut chars, 669, 5, &especie_documento, false, ' ');
    put(&mut chars, 676, 2, &profile.cst_pis, false, ' ');
    put(&mut chars, 678, 2, &profile.cst_cofins, false, ' ');
    put(&mut chars, 680, 14, &format_money(base_calculo), true, ' ');
    put(
        &mut chars,
        694,
        8,
        &format_rate_from_tax(valor_pis, base_calculo),
        true,
        ' ',
    );
    put(&mut chars, 702, 14, &format_money(valor_pis), true, ' ');
    put(
        &mut chars,
        716,
        8,
        &format_rate_from_tax(valor_cofins, base_calculo),
        true,
        ' ',
    );
    put(&mut chars, 724, 14, &format_money(valor_cofins), true, ' ');
    put(&mut chars, 810, 1, "9", false, ' ');
    put(&mut chars, 827, 4, &profile.cod_rec_pis, false, ' ');
    put(&mut chars, 831, 4, &profile.cod_rec_cofins, false, ' ');
    put(&mut chars, 835, 1, "N", false, ' ');
    put(&mut chars, 861, 10, &fit_left(&numero, 10), true, '0');
    put(&mut chars, 871, 10, &fit_left(&numero, 10), true, '0');
    put(&mut chars, 891, 8, &format_money_4(aliquota), true, ' ');
    put(&mut chars, 899, 3, &profile.cst_iss, false, ' ');
    put(&mut chars, 902, 4, &profile.cfps, false, ' ');
    put(&mut chars, 906, 4, &profile.cod_receita_irrf, false, ' ');
    put(
        &mut chars,
        910,
        1,
        if should_emit_obs(&observacao, &profile.obs_extended) {
            "1"
        } else {
            "0"
        },
        false,
        ' ',
    );

    Ok((chars.into_iter().collect(), observacao))
}

fn apply_number_rule(source: f64, rule: &FieldRule) -> f64 {
    match rule.action {
        FieldAction::Source => source,
        FieldAction::Zero => 0.0,
        FieldAction::Empty | FieldAction::Ignore => 0.0,
        FieldAction::Constant => rule.value.as_deref().map(parse_decimal).unwrap_or(0.0),
    }
}

fn apply_bool_rule(source: bool, rule: &FieldRule) -> bool {
    match rule.action {
        FieldAction::Source => source,
        FieldAction::Zero | FieldAction::Empty | FieldAction::Ignore => false,
        FieldAction::Constant => matches!(
            rule.value.as_deref(),
            Some("1") | Some("true") | Some("TRUE") | Some("S") | Some("s")
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

fn should_emit_obs(obs: &str, mode: &str) -> bool {
    if obs.trim().is_empty() {
        return false;
    }
    match mode {
        "always" => true,
        "never" => false,
        _ => obs.chars().count() > 40,
    }
}

fn sanitize_obs_line(value: &str) -> String {
    value
        .replace("\r\n", " ")
        .replace('\n', " ")
        .replace('\r', " ")
        .trim()
        .to_string()
}

fn put(buffer: &mut [char], position: usize, size: usize, value: &str, right: bool, pad: char) {
    let start = position.saturating_sub(1);
    let fitted = fit(value, size, right, pad);
    for (index, ch) in fitted.chars().enumerate().take(size) {
        if start + index < buffer.len() {
            buffer[start + index] = ch;
        }
    }
}

fn fit(value: &str, size: usize, right: bool, pad: char) -> String {
    let cleaned = value.chars().take(size).collect::<String>();
    if cleaned.len() == size {
        return cleaned;
    }
    let fill = pad.to_string().repeat(size.saturating_sub(cleaned.len()));
    if right {
        format!("{}{}", fill, cleaned)
    } else {
        format!("{}{}", cleaned, fill)
    }
}

fn fit_left(value: &str, size: usize) -> String {
    let digits = only_digits(value);
    if digits.len() >= size {
        digits[digits.len() - size..].to_string()
    } else {
        digits
    }
}

fn parse_decimal(value: &str) -> f64 {
    let mut raw = value.trim().replace(' ', "");
    if raw.contains(',') && raw.contains('.') {
        raw = raw.replace('.', "").replace(',', ".");
    } else if raw.contains(',') {
        raw = raw.replace(',', ".");
    }
    raw.parse::<f64>().unwrap_or(0.0)
}

fn format_money(value: f64) -> String {
    format!("{value:.2}")
}
fn format_money_short(value: f64) -> String {
    format!("{value:.2}")
}
fn format_money_4(value: f64) -> String {
    format!("{value:.4}")
}

fn format_rate_from_tax(tax_value: f64, base_value: f64) -> String {
    if base_value <= 0.0 || tax_value <= 0.0 {
        return format!("{:.4}", 0.0);
    }
    format!("{:.4}", (tax_value / base_value) * 100.0)
}

fn sanitize_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase()
}

fn sanitize_serie(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(5)
        .collect::<String>()
        .to_uppercase()
}

fn only_digits(value: &str) -> String {
    value.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn ddmmaa(value: &str) -> String {
    let digits = only_digits(value);
    if digits.len() >= 8 {
        if value.contains('-') {
            return format!("{}{}{}", &digits[6..8], &digits[4..6], &digits[2..4]);
        }
        return format!("{}{}{}", &digits[0..2], &digits[2..4], &digits[6..8]);
    }
    String::new()
}

fn ddmmaaaa(value: &str) -> String {
    let digits = only_digits(value);
    if digits.len() >= 8 {
        if value.contains('-') {
            return format!("{}{}{}", &digits[6..8], &digits[4..6], &digits[0..4]);
        }
        return format!("{}{}{}", &digits[0..2], &digits[2..4], &digits[4..8]);
    }
    String::new()
}

fn first_non_empty(values: Vec<String>) -> String {
    values
        .into_iter()
        .find(|value| !value.trim().is_empty())
        .unwrap_or_default()
}
