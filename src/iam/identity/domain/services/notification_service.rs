use crate::iam::identity::domain::error::DomainError;
use async_trait::async_trait;

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_verification_email(&self, to: &str, token: &str) -> Result<(), DomainError>;
    async fn send_password_reset_email(
        &self,
        to: &str,
        reset_link: &str,
    ) -> Result<(), DomainError>;
}
