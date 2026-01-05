use async_trait::async_trait;
use crate::iam::identity::domain::error::DomainError;

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_verification_email(&self, to: &str, token: &str) -> Result<(), DomainError>;
}
