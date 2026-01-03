use crate::iam::identity::domain::{
    model::{
        aggregates::identity::Identity,
        commands::register_identity_command::RegisterIdentityCommand,
    },
    repositories::identity_repository::IdentityRepository,
    services::identity_command_service::IdentityCommandService,
    error::DomainError,
    model::value_objects::password::Password,
};
use bcrypt::{hash, DEFAULT_COST};

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
        mut command: RegisterIdentityCommand,
    ) -> Result<Identity, DomainError> {
        // Validate MX records
        if let Err(e) = command.email.validate_mx().await {
            return Err(DomainError::InvalidEmailDomain(e.to_string()));
        }

        match self.repository.find_by_email(&command.email).await {
            Ok(Some(_)) => return Err(DomainError::EmailAlreadyExists),
            Ok(None) => {},
            Err(e) => return Err(DomainError::InternalError(e.to_string())),
        }

        // Security: Hash password before domain/persistence interaction
        let hashed = hash(command.password.value(), DEFAULT_COST)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        
        // Replace plain password with hash in the command (or create new Password VO)
        // Since Password::new validates length (12-72), and bcrypt hash is 60, it fits perfectly.
        command.password = Password::new(hashed)
            .map_err(|e| DomainError::InternalError(e))?;

        let identity = Identity::register(command);
        
        self.repository.save(identity).await.map_err(|e| DomainError::InternalError(e.to_string()))
    }
}
