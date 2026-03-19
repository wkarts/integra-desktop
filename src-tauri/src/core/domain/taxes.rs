use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Taxes {
    pub valor_servicos: f64,
    pub base_calculo: f64,
    pub valor_iss: f64,
    pub aliquota_iss: f64,
    pub valor_liquido: f64,
    pub valor_irrf: f64,
    pub valor_pis: f64,
    pub valor_cofins: f64,
    pub valor_csll: f64,
    pub valor_inss: f64,
    pub iss_retido: bool,
}
