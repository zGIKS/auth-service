use crate::iam::identity::domain::{
    model::{
        aggregates::identity::Identity,
        commands::{
            register_identity_command::RegisterIdentityCommand,
            confirm_registration_command::ConfirmRegistrationCommand,
        },
        value_objects::{
            pending_identity::PendingIdentity,
            verification_token::VerificationToken,
            password::Password,
            email::Email,
            auth_provider::AuthProvider,
        },
    },
    repositories::{
        identity_repository::IdentityRepository,
        pending_identity_repository::PendingIdentityRepository,
    },
    services::identity_command_service::IdentityCommandService,
    error::DomainError,
};
use bcrypt::{hash, DEFAULT_COST};
use std::time::Duration;
use std::str::FromStr;

pub struct IdentityCommandServiceImpl<R, P>
where
    R: IdentityRepository,
    P: PendingIdentityRepository,
{
    identity_repository: R,
    pending_repository: P,
    pending_ttl: Duration,
}

impl<R, P> IdentityCommandServiceImpl<R, P>
where
    R: IdentityRepository,
    P: PendingIdentityRepository,
{
    pub fn new(identity_repository: R, pending_repository: P, pending_ttl: Duration) -> Self {
        Self {
            identity_repository,
            pending_repository,
            pending_ttl,
        }
    }
}

impl<R, P> IdentityCommandService for IdentityCommandServiceImpl<R, P>
where
    R: IdentityRepository,
    P: PendingIdentityRepository,
{
    async fn handle(
        &self,
        mut command: RegisterIdentityCommand,
    ) -> Result<(Identity, String), DomainError> {
        // Validate MX records
        if let Err(e) = command.email.validate_mx().await {
            return Err(DomainError::InvalidEmailDomain(e.to_string()));
        }

        match self.identity_repository.find_by_email(&command.email).await {
            Ok(Some(_)) => return Err(DomainError::EmailAlreadyExists),
            Ok(None) => {}
            Err(e) => return Err(DomainError::InternalError(e.to_string())),
        }

        // Security: Hash password before domain/persistence interaction
        let hashed = hash(command.password.value(), DEFAULT_COST)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        // Replace plain password with hash in the command
        command.password = Password::new(hashed.clone())
            .map_err(DomainError::InternalError)?;

        // Generate Verification Token
        let token = VerificationToken::new();
        let token_hash = token.hash();

        // Create PendingIdentity
        let pending = PendingIdentity {
            email: command.email.value().to_string(),
            password_hash: hashed,
            provider: command.provider.to_string(),
        };

        // Save to Redis with configured TTL
        self.pending_repository
            .save(pending, token_hash, self.pending_ttl)
            .await?;

        let identity = Identity::register(command);

        Ok((identity, token.value().to_string()))
    }

    async fn confirm_registration(
        &self,
        command: ConfirmRegistrationCommand,
    ) -> Result<Identity, DomainError> {
        let token = VerificationToken::from_string(command.token);
        let token_hash = token.hash();

        let pending = self.pending_repository.find(&token_hash).await?
            .ok_or(DomainError::InvalidToken)?;

        let email = Email::new(pending.email)
            .map_err(|e| DomainError::InternalError(format!("Invalid pending email: {}", e)))?;

        let password = Password::new(pending.password_hash)
            .map_err(DomainError::InternalError)?;

        let provider = AuthProvider::from_str(&pending.provider)
            .map_err(|e| DomainError::InternalError(format!("Invalid pending provider: {}", e)))?;

        let register_command = RegisterIdentityCommand {
            email,
            password,
            provider,
            is_verified: true,
        };

        let identity = Identity::register(register_command);

        self.identity_repository.save(identity.clone()).await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        self.pending_repository.delete(&token_hash).await?;

        Ok(identity)
    }
}
