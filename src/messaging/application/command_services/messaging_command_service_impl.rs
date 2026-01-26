use crate::messaging::domain::{
    error::MessagingError,
    model::commands::send_email_command::SendEmailCommand,
    services::{
        email_sender_service::EmailSenderService,
        messaging_command_service::MessagingCommandService,
    },
};
use async_trait::async_trait;

pub struct MessagingCommandServiceImpl<S>
where
    S: EmailSenderService,
{
    email_sender_service: S,
}

impl<S> MessagingCommandServiceImpl<S>
where
    S: EmailSenderService,
{
    pub fn new(email_sender_service: S) -> Self {
        Self {
            email_sender_service,
        }
    }
}

#[async_trait]
impl<S> MessagingCommandService for MessagingCommandServiceImpl<S>
where
    S: EmailSenderService,
{
    async fn send_email(&self, command: SendEmailCommand) -> Result<(), MessagingError> {
        self.email_sender_service
            .send(&command.to, &command.subject, &command.body)
            .await
    }
}
