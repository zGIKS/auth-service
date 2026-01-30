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
        value_objects::{
            auth_provider::AuthProvider, email::Email, password::Password,
            pending_identity::PendingIdentity, verification_token::VerificationToken,
        },
    },
    repositories::{
        identity_repository::IdentityRepository,
        password_reset_token_repository::PasswordResetTokenRepository,
        pending_identity_repository::PendingIdentityRepository,
    },
    services::{
        identity_command_service::IdentityCommandService,
        notification_service::NotificationService,
        session_invalidation_service::SessionInvalidationService,
    },
};
use async_trait::async_trait;
use bcrypt::{DEFAULT_COST, hash};
use std::str::FromStr;
use std::time::Duration;

pub struct IdentityCommandServiceImpl<R, P, PR, N, S>
where
    R: IdentityRepository,
    P: PendingIdentityRepository,
    PR: PasswordResetTokenRepository,
    N: NotificationService,
    S: SessionInvalidationService,
{
    identity_repository: R,
    pending_repository: P,
    password_reset_repository: PR,
    notification_service: N,
    session_invalidation_service: S,
    pending_ttl: Duration,
    password_reset_ttl: Duration,
}

impl<R, P, PR, N, S> IdentityCommandServiceImpl<R, P, PR, N, S>
where
    R: IdentityRepository,
    P: PendingIdentityRepository,
    PR: PasswordResetTokenRepository,
    N: NotificationService,
    S: SessionInvalidationService,
{
    pub fn new(
        identity_repository: R,
        pending_repository: P,
        password_reset_repository: PR,
        notification_service: N,
        session_invalidation_service: S,
        pending_ttl: Duration,
        password_reset_ttl: Duration,
    ) -> Self {
        Self {
            identity_repository,
            pending_repository,
            password_reset_repository,
            notification_service,
            session_invalidation_service,
            pending_ttl,
            password_reset_ttl,
        }
    }
}

#[async_trait]
impl<R, P, PR, N, S> IdentityCommandService for IdentityCommandServiceImpl<R, P, PR, N, S>
where
    R: IdentityRepository,
    P: PendingIdentityRepository,
    PR: PasswordResetTokenRepository,
    N: NotificationService,
    S: SessionInvalidationService,
{
    async fn handle(
        &self,
        mut command: RegisterIdentityCommand,
    ) -> Result<(Identity, String), DomainError> {
        // Note: MX validation removed - email confirmation is sufficient validation
        // Avoids: DNS failures, latency, false negatives, and external dependencies
        
        match self.identity_repository.find_by_email(&command.email).await {
            Ok(Some(_)) => return Err(DomainError::EmailAlreadyExists),
            Ok(None) => {}
            Err(e) => return Err(DomainError::InternalError(e.to_string())),
        }

        // Security: Hash password before domain/persistence interaction
        let hashed = hash(command.password.value(), DEFAULT_COST)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        // Replace plain password with hash in the command
        command.password = Password::new(hashed.clone()).map_err(DomainError::InternalError)?;

        // Check if there is already a pending registration for this email
        // If so, we invalidate the old one (security/cleanup) to ensure only one active token per email
        if let Ok(Some(old_token_hash)) = self
            .pending_repository
            .find_token_by_email(command.email.value())
            .await
        {
            let _ = self.pending_repository.delete(&old_token_hash).await;
        }

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

        // Send Verification Email
        // Construct the verification link pointing to the FRONTEND
        let frontend_url = std::env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");
        validate_frontend_url(&frontend_url)?;
        let verification_link = format!("{}/verify?token={}", frontend_url, token.value());

        self.notification_service
            .send_verification_email(command.email.value(), &verification_link)
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

        let pending = self
            .pending_repository
            .find(&token_hash)
            .await?
            .ok_or(DomainError::InvalidToken)?;

        let email = Email::new(pending.email)
            .map_err(|e| DomainError::InternalError(format!("Invalid pending email: {}", e)))?;

        let password = Password::new(pending.password_hash).map_err(DomainError::InternalError)?;

        let provider = AuthProvider::from_str(&pending.provider)
            .map_err(|e| DomainError::InternalError(format!("Invalid pending provider: {}", e)))?;

        let register_command = RegisterIdentityCommand {
            email,
            password,
            provider,
        };

        let identity = Identity::register(register_command);

        self.identity_repository
            .save(identity.clone())
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        self.pending_repository.delete(&token_hash).await?;

        Ok(identity)
    }

    async fn request_password_reset(
        &self,
        command: RequestPasswordResetCommand,
    ) -> Result<(), DomainError> {
        // 1. Check if identity exists
        // We must map the error explicitly because ? expects DomainError but repo returns Box<dyn Error>
        let identity_opt = self
            .identity_repository
            .find_by_email(&command.email)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        if identity_opt.is_none() {
            // Security: Don't reveal if email exists. Just return OK.
            return Ok(());
        }

        // 2. Generate Reset Token
        let token = VerificationToken::new();
        let token_hash = token.hash();

        // 3. Save to Redis with distributed lock (invalidates previous tokens)
        // This prevents race conditions and ensures only one valid token exists
        match self
            .password_reset_repository
            .save(
                command.email.value().to_string(),
                token_hash,
                self.password_reset_ttl,
            )
            .await
        {
            Ok(_) => {}
            Err(e) => {
                // If lock acquisition fails, silently return OK for security
                // (don't reveal if request is in progress)
                tracing::warn!("Failed to save password reset token: {}", e);
                return Ok(());
            }
        }

        // 4. Send Email
        let frontend_url = std::env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");
        validate_frontend_url(&frontend_url)?;
        let reset_link = format!("{}/reset-password?token={}", frontend_url, token.value());

        self.notification_service
            .send_password_reset_email(command.email.value(), &reset_link)
            .await?;

        Ok(())
    }

    async fn reset_password(&self, command: ResetPasswordCommand) -> Result<(), DomainError> {
        // 1. Find email by token hash
        let token = VerificationToken::from_string(command.token);
        let token_hash = token.hash();

        let email_str = self
            .password_reset_repository
            .find_email_by_token(&token_hash)
            .await?
            .ok_or(DomainError::InvalidToken)?;

        let email = Email::new(email_str)
            .map_err(|e| DomainError::InternalError(format!("Invalid stored email: {}", e)))?;

        // 2. Find Identity
        let mut identity = self
            .identity_repository
            .find_by_email(&email)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?
            .ok_or(DomainError::InternalError(
                "Identity not found for valid token".to_string(),
            ))?;

        // 3. Hash New Password
        let hashed_password = hash(command.new_password.value(), DEFAULT_COST)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let new_password = Password::new(hashed_password).map_err(DomainError::InternalError)?;

        // 4. Update Identity
        identity.change_password(new_password);

        // 5. Save Identity
        self.identity_repository
            .save(identity.clone())
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        // 6. Delete Token
        self.password_reset_repository.delete(&token_hash).await?;

        // 7. Revoke All Sessions (Security Critical: Invalidate compromised sessions)
        self.session_invalidation_service
            .invalidate_all_sessions(identity.id().value())
            .await
            .map_err(|e| DomainError::InternalError(format!("Failed to revoke sessions: {}", e)))?;

        Ok(())
    }
}

fn validate_frontend_url(url: &str) -> Result<(), DomainError> {
    if url.starts_with("https://") {
        Ok(())
    } else if url.starts_with("http://localhost") || url.starts_with("http://127.0.0.1") {
        // Allow HTTP for local development
        Ok(())
    } else {
        Err(DomainError::InternalError(
            "FRONTEND_URL must use HTTPS for security, or HTTP only for localhost/127.0.0.1 in development".to_string(),
        ))
    }
}
