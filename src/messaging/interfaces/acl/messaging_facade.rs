use async_trait::async_trait;
use crate::messaging::domain::error::MessagingError;

#[async_trait]
pub trait MessagingFacade: Send + Sync {
    async fn send_email(&self, to: String, subject: String, body: String) -> Result<(), MessagingError>;
}
