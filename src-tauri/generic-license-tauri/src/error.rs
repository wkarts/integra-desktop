use thiserror::Error;

#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("configuração inválida: {0}")]
    Config(String),

    #[error("erro HTTP: {0}")]
    Http(String),

    #[error("licença inválida: {0}")]
    Invalid(String),

    #[error("erro de serialização: {0}")]
    Serde(String),

    #[error("erro de IO: {0}")]
    Io(String),
}
