use crate::core::domain::document::NfseDocument;
use anyhow::{anyhow, Result};

pub fn parse(_xml: &str, _file_name: &str) -> Result<NfseDocument> {
    Err(anyhow!(
        "Parser Betha ainda não implementado nesta entrega."
    ))
}
