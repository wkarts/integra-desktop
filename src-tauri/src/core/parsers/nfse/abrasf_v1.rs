use anyhow::{anyhow, Result};
use crate::core::domain::document::NfseDocument;

pub fn parse(_xml: &str, _file_name: &str) -> Result<NfseDocument> {
    Err(anyhow!("ABRASF v1 ainda não implementado nesta versão; use o fallback legado quando necessário."))
}
