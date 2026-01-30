use crate::messaging::domain::error::MessagingError;
use async_trait::async_trait;

#[async_trait]
pub trait MessagingFacade: Send + Sync {
    async fn send_email(
        &self,
        to: String,
        subject: String,
        body: String,
    ) -> Result<(), MessagingError>;
}
