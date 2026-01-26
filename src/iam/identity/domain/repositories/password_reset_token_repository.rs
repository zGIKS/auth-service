use crate::iam::identity::domain::error::DomainError;
use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait PasswordResetTokenRepository: Send + Sync {
    async fn save(
        &self,
        email: String,
        token_hash: String,
        ttl: Duration,
    ) -> Result<(), DomainError>;
    async fn find_email_by_token(&self, token_hash: &str) -> Result<Option<String>, DomainError>;
    async fn delete(&self, token_hash: &str) -> Result<(), DomainError>;
}
