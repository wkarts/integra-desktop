use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpedRecord {
    pub registro: String,
    pub campos: Vec<String>,
}

pub fn parse_sped(content: &str) -> Result<Vec<SpedRecord>> {
    let records = content
        .lines()
        .filter(|line| line.starts_with('|') && line.ends_with('|'))
        .map(|line| {
            let parts = line
                .trim_matches('|')
                .split('|')
                .map(|item| item.trim().to_string())
                .collect::<Vec<_>>();
            let registro = parts.first().cloned().unwrap_or_default();
            SpedRecord {
                registro,
                campos: parts,
            }
        })
        .collect::<Vec<_>>();

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::parse_sped;

    #[test]
    fn parse_registros_sped() {
        let input = "|C100|0|1|\n|C140|001|";
        let records = parse_sped(input).expect("deve parsear");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].registro, "C100");
    }
}
