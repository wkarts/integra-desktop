use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Party {
    pub nome: String,
    pub documento: String,
    pub inscricao_municipal: Option<String>,
    pub endereco: Option<String>,
    pub municipio_codigo: Option<String>,
    pub municipio_nome: Option<String>,
    pub uf: Option<String>,
    pub cep: Option<String>,
}
