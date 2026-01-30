use crate::messaging::domain::error::MessagingError;
use crate::messaging::domain::model::commands::send_email_command::SendEmailCommand;
use async_trait::async_trait;

#[async_trait]
pub trait MessagingCommandService: Send + Sync {
    async fn send_email(&self, command: SendEmailCommand) -> Result<(), MessagingError>;
}
