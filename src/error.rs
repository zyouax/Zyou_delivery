use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShippingError {
    #[error("Environment variable missing: {0}")]
    EnvVarMissing(String),
    #[error("API request failed: {0}")]
    ApiRequestFailed(String),
    #[error("JSON parsing error: {0}")]
    JsonParsingError(String),
    #[error("Feature not implemented for carrier: {0}")]
    NotImplemented(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}