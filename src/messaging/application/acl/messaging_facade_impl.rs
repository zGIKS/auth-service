use crate::messaging::{
    domain::{
        error::MessagingError, model::commands::send_email_command::SendEmailCommand,
        services::messaging_command_service::MessagingCommandService,
    },
    interfaces::acl::messaging_facade::MessagingFacade,
};
use async_trait::async_trait;

pub struct MessagingFacadeImpl<S>
where
    S: MessagingCommandService,
{
    command_service: S,
}

impl<S> MessagingFacadeImpl<S>
where
    S: MessagingCommandService,
{
    pub fn new(command_service: S) -> Self {
        Self { command_service }
    }
}

#[async_trait]
impl<S> MessagingFacade for MessagingFacadeImpl<S>
where
    S: MessagingCommandService,
{
    async fn send_email(
        &self,
        to: String,
        subject: String,
        body: String,
    ) -> Result<(), MessagingError> {
        let command = SendEmailCommand::new(to, subject, body)
            .map_err(|e| MessagingError::SendError(e.to_string()))?;

        self.command_service.send_email(command).await
    }
}
