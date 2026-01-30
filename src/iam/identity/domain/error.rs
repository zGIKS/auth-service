use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Email already exists")]
    EmailAlreadyExists,
    #[error("Invalid email domain: {0}")]
    InvalidEmailDomain(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Invalid or expired token")]
    InvalidToken,
}
