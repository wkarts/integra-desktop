use crate::core::parsers::sped::SpedRecord;

pub fn upsert_c140_c141(records: &[SpedRecord], payload_lines: &[String]) -> Vec<SpedRecord> {
    let mut next = records
        .iter()
        .filter(|record| record.registro != "C140" && record.registro != "C141")
        .cloned()
        .collect::<Vec<_>>();

    next.extend(payload_lines.iter().map(|line| {
        let campos = line
            .trim_matches('|')
            .split('|')
            .map(|item| item.trim().to_string())
            .collect::<Vec<_>>();
        let registro = campos.first().cloned().unwrap_or_default();
        SpedRecord { registro, campos }
    }));

    next
}

#[cfg(test)]
mod tests {
    use super::upsert_c140_c141;
    use crate::core::parsers::sped::SpedRecord;

    #[test]
    fn substitui_registros_existentes() {
        let base = vec![
            SpedRecord {
                registro: "C100".into(),
                campos: vec!["C100".into()],
            },
            SpedRecord {
                registro: "C140".into(),
                campos: vec!["C140".into(), "old".into()],
            },
        ];
        let payload = vec!["|C140|new|".to_string(), "|C141|x|".to_string()];
        let updated = upsert_c140_c141(&base, &payload);
        assert_eq!(updated.len(), 3);
        assert!(updated
            .iter()
            .any(|record| record.campos.contains(&"new".to_string())));
    }
}
