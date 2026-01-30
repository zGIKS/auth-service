use crate::messaging::domain::error::MessagingError;
use crate::messaging::domain::model::value_objects::{
    body::Body, email_address::EmailAddress, subject::Subject,
};
use async_trait::async_trait;

#[async_trait]
pub trait EmailSenderService: Send + Sync {
    async fn send(
        &self,
        to: &EmailAddress,
        subject: &Subject,
        body: &Body,
    ) -> Result<(), MessagingError>;
}
