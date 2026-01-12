use thiserror::Error;

#[derive(Debug, Error)]
pub enum FederationError {
    #[error("Invalid or missing authorization code")]
    InvalidAuthorizationCode,
    #[error("Email provided by Google is invalid")]
    InvalidEmail,
    #[error("Email provided by Google is not verified")]
    EmailNotVerified,
    #[error("Email already registered with another provider")]
    ProviderMismatch,
    #[error("Failed to exchange authorization code: {0}")]
    TokenExchange(String),
    #[error("Failed to obtain Google user info: {0}")]
    UserInfo(String),
    #[error("Internal error: {0}")]
    Internal(String),
}
