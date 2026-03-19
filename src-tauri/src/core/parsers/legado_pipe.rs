use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegacyPipeRow {
    pub fields: Vec<String>,
}

pub fn parse_legacy_pipe(content: &str) -> Vec<LegacyPipeRow> {
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| LegacyPipeRow {
            fields: line
                .split('|')
                .map(|item| item.trim().to_string())
                .collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_legacy_pipe;

    #[test]
    fn parse_linhas_pipe() {
        let rows = parse_legacy_pipe("A|B|C\n1|2|3");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].fields[1], "B");
    }
}
