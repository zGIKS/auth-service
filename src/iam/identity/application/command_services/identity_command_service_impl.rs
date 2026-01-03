use crate::iam::identity::domain::{
    model::{
        aggregates::identity::Identity,
        commands::register_identity_command::RegisterIdentityCommand,
    },
    repositories::identity_repository::IdentityRepository,
    services::identity_command_service::IdentityCommandService,
};
use std::error::Error;

pub struct IdentityCommandServiceImpl<R: IdentityRepository> {
    repository: R,
}

impl<R: IdentityRepository> IdentityCommandServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: IdentityRepository> IdentityCommandService for IdentityCommandServiceImpl<R> {
    async fn handle(
        &self,
        command: RegisterIdentityCommand,
    ) -> Result<Identity, Box<dyn Error + Send + Sync>> {
        if let Some(_) = self.repository.find_by_email(&command.email).await? {
            return Err("User already exists".into());
        }

        let identity = Identity::register(command);
        self.repository.save(identity).await
    }
}