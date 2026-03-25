use anyhow::Result;

use crate::core::domain::document::{ConversionProfile, FieldAction, FieldRule, NfseDocument};

#[derive(Debug, Clone)]
pub struct ProsoftSpPrestadoRecord {
    pub main_line: String,
    pub obs_line: Option<String>,
}

pub fn map_document_to_sp_prestados_record(
    document: &NfseDocument,
    profile: &ConversionProfile,
) -> Result<ProsoftSpPrestadoRecord> {
    let (main_line, observacao) = build_main_line_and_observation(document, profile)?;
    let emit_obs = should_emit_obs(&observacao, &profile.obs_extended);
    let obs_line = if emit_obs {
        Some(format!(
            "*OBS{}",
            sanitize_obs_line_ascii(&observacao)
                .chars()
                .take(7000)
                .collect::<String>()
        ))
    } else {
        None
    };

    Ok(ProsoftSpPrestadoRecord {
        main_line,
        obs_line,
    })
}

pub fn map_document_to_sp_prestados_line(
    document: &NfseDocument,
    profile: &ConversionProfile,
) -> Result<String> {
    let record = map_document_to_sp_prestados_record(document, profile)?;
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
    let len = 1674usize;
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
        service_code.chars().take(12).collect::<String>()
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
    let observacao = sanitize_obs_line_ascii(&apply_string_rule(
        format!("{} {}", observacao_base, competencia).trim(),
        &profile.field_rules.observacao,
    ));
    let observacao_curta = observacao.chars().take(40).collect::<String>();
    let municipio_nome = sanitize_text_ascii(&apply_string_rule(
        &document.municipio_nome,
        &profile.field_rules.municipio,
    ))
    .chars()
    .take(20)
    .collect::<String>();
    let municipio_codigo = only_digits(&document.municipio_codigo);
    let tomador_doc = only_digits(&document.tomador.documento);
    let tomador_uf = sanitize_text_ascii(document.tomador.uf.as_deref().unwrap_or_default())
        .chars()
        .take(2)
        .collect::<String>();
    let tomador_ie = sanitize_text_ascii(
        document
            .tomador
            .inscricao_municipal
            .as_deref()
            .unwrap_or_default(),
    )
    .chars()
    .take(18)
    .collect::<String>();

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
    let iss_retido = apply_bool_rule(document.taxes.iss_retido, &profile.field_rules.iss_retido);
    let modelo_iss = if profile.modelo_iss.trim() == "53" {
        "53"
    } else {
        "51"
    };
    let tipo_livro = if modelo_iss == "53" { "2" } else { "1" };
    let tipo_recolhimento = if profile.tipo_recolhimento.trim().is_empty() {
        if iss_retido {
            "11"
        } else {
            "10"
        }
    } else {
        profile.tipo_recolhimento.trim()
    };
    let motivo_retencao = if profile.motivo_retencao.trim().is_empty() {
        if iss_retido {
            "T"
        } else {
            ""
        }
    } else {
        profile.motivo_retencao.trim()
    };
    let operacao_nota = if profile.operacao_nota.trim().is_empty() {
        "A"
    } else {
        profile.operacao_nota.trim()
    };
    let tipo_documento =
        apply_string_rule(&profile.tipo_documento, &profile.field_rules.tipo_documento);
    let especie_documento = apply_string_rule(
        &profile.especie_documento,
        &profile.field_rules.especie_documento,
    );
    let situacao_documento = sanitize_text_ascii(&profile.situacao_documento)
        .chars()
        .take(10)
        .collect::<String>();
    let modelo_nf = if profile.modelo_nf.trim().is_empty() {
        "OU000"
    } else {
        profile.modelo_nf.trim()
    };
    let situacao_documento_first = profile
        .situacao_documento
        .chars()
        .take(1)
        .collect::<String>();
    let valor_iss_retido_fmt = if iss_retido {
        format_money(valor_iss)
    } else {
        String::new()
    };
    let emissao_retencao_fmt = if iss_retido {
        ddmmaa(&emissao)
    } else {
        String::new()
    };
    let base_calculo_retido_fmt = if iss_retido {
        format_money(base_calculo)
    } else {
        String::new()
    };
    let aliquota_retido_fmt = if iss_retido {
        format_money_short(aliquota)
    } else {
        String::new()
    };
    let base_calculo_inss_small_fmt = if valor_inss > 0.0 {
        format_money_small(base_calculo)
    } else {
        String::new()
    };
    let aliquota_inss_fmt = if valor_inss > 0.0 {
        format_rate_from_tax(valor_inss, base_calculo)
    } else {
        String::new()
    };
    let valor_inss_small_fmt = if valor_inss > 0.0 {
        format_money_small(valor_inss)
    } else {
        String::new()
    };

    put(&mut chars, 1, 1, tipo_livro, false, ' ');
    put(&mut chars, 2, 6, &ddmmaa(&emissao), true, ' ');
    put(&mut chars, 8, 3, &serie, false, ' ');
    put(&mut chars, 11, 6, "", true, ' ');
    put(&mut chars, 17, 6, "", true, ' ');
    put(&mut chars, 23, 4, &code_4, true, ' ');
    put(&mut chars, 27, 1, "0", false, ' ');
    put(
        &mut chars,
        28,
        14,
        &format_money(document.taxes.valor_servicos),
        true,
        ' ',
    );
    put(&mut chars, 42, 5, &format_money_short(aliquota), true, ' ');
    put(&mut chars, 47, 14, &format_money(valor_iss), true, ' ');
    put(&mut chars, 61, 40, &observacao_curta, false, ' ');
    put(&mut chars, 101, 5, "", false, ' ');
    put(&mut chars, 106, 3, "", false, ' ');

    if modelo_iss == "51" {
        put(&mut chars, 109, 14, &format_money(base_calculo), true, ' ');
        put(&mut chars, 123, 14, "", true, ' ');
        put(&mut chars, 137, 14, "", true, ' ');
        put(&mut chars, 151, 14, "", true, ' ');
        put(&mut chars, 165, 14, "", true, ' ');
        put(&mut chars, 179, 14, "", true, ' ');
        put(&mut chars, 193, 14, "", true, ' ');
        put(&mut chars, 207, 14, "", true, ' ');
    } else {
        put(&mut chars, 109, 14, "", true, ' ');
        put(&mut chars, 123, 14, "", true, ' ');
        put(&mut chars, 137, 14, "", true, ' ');
        put(
            &mut chars,
            151,
            14,
            &format_money(document.taxes.valor_servicos),
            true,
            ' ',
        );
        put(&mut chars, 165, 14, &format_money(base_calculo), true, ' ');
        put(&mut chars, 179, 14, "", true, ' ');
        put(&mut chars, 193, 14, "", true, ' ');
        put(&mut chars, 207, 14, "", true, ' ');
    }

    put(&mut chars, 221, 20, &municipio_nome, false, ' ');
    put(&mut chars, 241, 14, &tomador_doc, true, '0');
    put(&mut chars, 255, 14, &format_money(valor_irrf), true, ' ');
    put(&mut chars, 269, 14, &format_money(valor_inss), true, ' ');
    put(&mut chars, 283, 14, "", false, ' ');
    put(&mut chars, 297, 14, &valor_iss_retido_fmt, true, ' ');
    put(&mut chars, 311, 5, &especie_documento, false, ' ');
    put(&mut chars, 316, 1, &situacao_documento_first, false, ' ');
    put(&mut chars, 317, 5, "", false, ' ');
    put(&mut chars, 322, 200, "", false, ' ');
    put(&mut chars, 522, 100, "", false, ' ');
    put(&mut chars, 622, 5, &serie, false, ' ');
    put(&mut chars, 627, 8, &fit_left(&numero, 8), true, '0');
    put(&mut chars, 635, 8, &fit_left(&numero, 8), true, '0');
    put(&mut chars, 643, 6, "", false, ' ');
    put(&mut chars, 649, 4, "", false, ' ');
    put(&mut chars, 653, 6, "", false, ' ');
    put(
        &mut chars,
        659,
        1,
        &profile.responsavel_retencao,
        false,
        ' ',
    );
    put(&mut chars, 660, 10, "", true, '0');
    put(&mut chars, 670, 10, "", true, '0');
    put(&mut chars, 680, 3, "", false, ' ');
    put(&mut chars, 683, 2, "", false, ' ');
    put(&mut chars, 685, 18, &tomador_ie, false, ' ');
    put(&mut chars, 703, 2, &tomador_uf, false, ' ');
    put(&mut chars, 705, 6, &emissao_retencao_fmt, false, ' ');
    put(&mut chars, 711, 14, &base_calculo_retido_fmt, true, ' ');
    put(&mut chars, 725, 5, &aliquota_retido_fmt, true, ' ');
    put(&mut chars, 730, 12, &code_12, false, ' ');
    put(&mut chars, 742, 5, modelo_nf, false, ' ');
    put(&mut chars, 747, 2, motivo_retencao, false, ' ');
    put(&mut chars, 749, 2, operacao_nota, false, ' ');
    put(&mut chars, 751, 3, tipo_recolhimento, false, ' ');
    put(&mut chars, 754, 1, "0", false, ' ');
    put(&mut chars, 755, 3, &tipo_documento, false, ' ');
    put(
        &mut chars,
        758,
        1,
        if valor_irrf > 0.0 { "1" } else { "0" },
        false,
        ' ',
    );
    put(&mut chars, 759, 5, &ibge5(&municipio_codigo), false, ' ');
    put(&mut chars, 764, 10, &situacao_documento, false, ' ');
    put(&mut chars, 774, 2, "", false, ' ');
    put(&mut chars, 776, 2, &profile.cst_pis, false, ' ');
    put(&mut chars, 778, 2, &profile.cst_cofins, false, ' ');
    put(&mut chars, 780, 14, &format_money(base_calculo), true, ' ');
    put(
        &mut chars,
        794,
        8,
        &format_rate_from_tax(valor_pis, base_calculo),
        true,
        ' ',
    );
    put(&mut chars, 802, 14, &format_money(valor_pis), true, ' ');
    put(
        &mut chars,
        816,
        8,
        &format_rate_from_tax(valor_cofins, base_calculo),
        true,
        ' ',
    );
    put(&mut chars, 824, 14, &format_money(valor_cofins), true, ' ');
    put(&mut chars, 838, 12, "", false, ' ');
    put(&mut chars, 850, 12, "", false, ' ');
    put(&mut chars, 862, 48, "", false, ' ');
    put(&mut chars, 910, 1, "9", false, ' ');
    put(&mut chars, 911, 8, &ddmmaaaa(&emissao), false, ' ');
    put(&mut chars, 919, 8, "", false, ' ');
    put(&mut chars, 927, 4, &profile.cod_rec_pis, false, ' ');
    put(&mut chars, 931, 4, &profile.cod_rec_cofins, false, ' ');
    put(&mut chars, 935, 1, "N", false, ' ');
    put(&mut chars, 936, 20, "", false, ' ');
    put(&mut chars, 956, 1, "", false, ' ');
    put(&mut chars, 957, 3, "", false, ' ');
    put(&mut chars, 960, 1, "", false, ' ');
    put(&mut chars, 961, 10, &fit_left(&numero, 10), true, '0');
    put(&mut chars, 971, 10, &fit_left(&numero, 10), true, '0');
    put(&mut chars, 981, 2, "", false, ' ');
    put(&mut chars, 983, 2, "", false, ' ');
    put(&mut chars, 985, 2, "", false, ' ');
    put(&mut chars, 987, 10, "", false, ' ');
    put(&mut chars, 997, 8, &format_money_4(aliquota), true, ' ');
    put(&mut chars, 1005, 3, &profile.cst_iss, false, ' ');
    put(
        &mut chars,
        1008,
        8,
        &municipality_export_code(profile, document),
        false,
        ' ',
    );
    put(&mut chars, 1016, 4, &profile.cfps, false, ' ');
    put(&mut chars, 1020, 4, &profile.cod_receita_irrf, false, ' ');
    put(
        &mut chars,
        1024,
        1,
        if should_emit_obs(&observacao, &profile.obs_extended) {
            "1"
        } else {
            "0"
        },
        false,
        ' ',
    );
    put(&mut chars, 1025, 12, "", false, ' ');
    put(
        &mut chars,
        1037,
        12,
        &base_calculo_inss_small_fmt,
        true,
        ' ',
    );
    put(&mut chars, 1049, 8, &aliquota_inss_fmt, true, ' ');
    put(&mut chars, 1057, 12, &valor_inss_small_fmt, true, ' ');
    put(&mut chars, 1069, 12, &valor_inss_small_fmt, true, ' ');
    put(&mut chars, 1081, 12, "", true, ' ');
    put(&mut chars, 1093, 12, "", true, ' ');
    put(&mut chars, 1105, 12, "", true, ' ');
    put(&mut chars, 1117, 12, "", true, ' ');
    put(&mut chars, 1129, 12, "", true, ' ');
    put(&mut chars, 1141, 12, "", true, ' ');
    put(&mut chars, 1153, 21, "", false, ' ');
    put(&mut chars, 1174, 2, "", false, ' ');
    put(&mut chars, 1176, 12, "", true, ' ');
    put(&mut chars, 1188, 21, "", false, ' ');
    put(&mut chars, 1209, 2, "", false, ' ');
    put(&mut chars, 1211, 12, "", true, ' ');
    put(&mut chars, 1223, 12, "", true, ' ');
    put(&mut chars, 1235, 12, "", true, ' ');
    put(&mut chars, 1247, 12, "", true, ' ');
    put(&mut chars, 1259, 14, "", false, ' ');
    put(&mut chars, 1273, 14, "", false, ' ');
    put(&mut chars, 1287, 1, "", false, ' ');
    put(&mut chars, 1288, 6, "", false, ' ');
    put(&mut chars, 1294, 1, "", false, ' ');
    put(&mut chars, 1295, 8, "", false, ' ');
    put(&mut chars, 1303, 1, "", false, ' ');
    put(&mut chars, 1304, 8, "", false, ' ');
    put(&mut chars, 1312, 50, "", false, ' ');
    put(&mut chars, 1362, 1, "", false, ' ');
    put(&mut chars, 1363, 50, "", false, ' ');
    put(&mut chars, 1413, 12, "", true, ' ');
    put(&mut chars, 1425, 2, "", false, ' ');
    put(&mut chars, 1427, 3, "", false, ' ');
    put(&mut chars, 1430, 6, "", false, ' ');
    put(&mut chars, 1436, 12, "", true, ' ');
    put(&mut chars, 1448, 5, "", true, ' ');
    put(&mut chars, 1453, 6, "", true, ' ');
    put(&mut chars, 1459, 5, "", true, ' ');
    put(&mut chars, 1464, 6, "", true, ' ');
    put(&mut chars, 1470, 12, "", true, ' ');
    put(&mut chars, 1482, 5, "", true, ' ');
    put(&mut chars, 1487, 12, "", true, ' ');
    put(&mut chars, 1499, 12, "", true, ' ');
    put(&mut chars, 1511, 5, "", true, ' ');
    put(&mut chars, 1516, 6, "", true, ' ');
    put(&mut chars, 1522, 5, "", true, ' ');
    put(&mut chars, 1527, 6, "", true, ' ');
    put(&mut chars, 1533, 12, "", true, ' ');
    put(&mut chars, 1545, 5, "", true, ' ');
    put(&mut chars, 1550, 12, "", true, ' ');
    put(&mut chars, 1562, 12, "", true, ' ');
    put(&mut chars, 1574, 2, "", false, ' ');
    put(&mut chars, 1576, 5, "", true, ' ');
    put(&mut chars, 1581, 12, "", true, ' ');
    put(&mut chars, 1593, 5, "", true, ' ');
    put(&mut chars, 1598, 6, "", true, ' ');
    put(&mut chars, 1604, 5, "", true, ' ');
    put(&mut chars, 1609, 6, "", true, ' ');
    put(&mut chars, 1615, 12, "", true, ' ');
    put(&mut chars, 1627, 5, "", true, ' ');
    put(&mut chars, 1632, 12, "", true, ' ');
    put(&mut chars, 1644, 12, "", true, ' ');
    put(&mut chars, 1656, 2, "", false, ' ');
    put(&mut chars, 1658, 5, "", true, ' ');
    put(&mut chars, 1663, 12, "", true, ' ');

    Ok((chars.into_iter().collect(), observacao))
}

fn municipality_export_code(profile: &ConversionProfile, document: &NfseDocument) -> String {
    let profile_code = only_digits(&profile.company_municipio_codigo);
    if profile_code.len() == 8 {
        return profile_code;
    }
    let raw = only_digits(&document.municipio_codigo);
    if raw.len() >= 8 {
        return raw.chars().take(8).collect();
    }
    raw
}

fn ibge5(value: &str) -> String {
    let digits = only_digits(value);
    if digits.len() >= 5 {
        digits[digits.len() - 5..].to_string()
    } else {
        digits
    }
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

fn sanitize_obs_line_ascii(value: &str) -> String {
    sanitize_ascii(value)
        .replace("\r\n", " ")
        .replace(['\n', '\r'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn sanitize_text_ascii(value: &str) -> String {
    sanitize_ascii(value)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase()
}

fn sanitize_ascii(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        let mapped = match ch {
            'á' | 'à' | 'ã' | 'â' | 'ä' | 'Á' | 'À' | 'Ã' | 'Â' | 'Ä' => 'A',
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'E',
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
            'ó' | 'ò' | 'õ' | 'ô' | 'ö' | 'Ó' | 'Ò' | 'Õ' | 'Ô' | 'Ö' => 'O',
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
            'ç' | 'Ç' => 'C',
            'ñ' | 'Ñ' => 'N',
            _ if ch.is_ascii() => ch,
            _ => ' ',
        };
        out.push(mapped);
    }
    out
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
fn format_money_small(value: f64) -> String {
    format!("{value:.2}")
}

fn format_rate_from_tax(tax_value: f64, base_value: f64) -> String {
    if base_value <= 0.0 || tax_value <= 0.0 {
        return format!("{:.4}", 0.0);
    }
    format!("{:.4}", (tax_value / base_value) * 100.0)
}

fn sanitize_serie(value: &str) -> String {
    sanitize_ascii(value)
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
