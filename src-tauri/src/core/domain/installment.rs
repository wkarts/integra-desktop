use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Installment {
    pub numero: String,
    pub vencimento: String,
    pub valor: f64,
}
