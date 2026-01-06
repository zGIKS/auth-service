use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessagingError {
    #[error("Failed to send email: {0}")]
    SendError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
