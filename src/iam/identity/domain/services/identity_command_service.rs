use crate::iam::identity::domain::{
    error::DomainError,
    model::{
        aggregates::identity::Identity,
        commands::{
            confirm_registration_command::ConfirmRegistrationCommand,
            register_identity_command::RegisterIdentityCommand,
            request_password_reset_command::RequestPasswordResetCommand,
            reset_password_command::ResetPasswordCommand,
        },
    },
};
use async_trait::async_trait;

#[async_trait]
pub trait IdentityCommandService: Send + Sync {
    async fn handle(
        &self,
        command: RegisterIdentityCommand,
    ) -> Result<(Identity, String), DomainError>;

    async fn confirm_registration(
        &self,
        command: ConfirmRegistrationCommand,
    ) -> Result<Identity, DomainError>;

    async fn request_password_reset(
        &self,
        command: RequestPasswordResetCommand,
    ) -> Result<(), DomainError>;

    async fn reset_password(&self, command: ResetPasswordCommand) -> Result<(), DomainError>;
}
