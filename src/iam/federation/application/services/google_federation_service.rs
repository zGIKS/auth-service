use bcrypt::{DEFAULT_COST, hash};
use rand::RngCore;

use crate::iam::authentication::domain::{
    model::value_objects::{refresh_token::RefreshToken, token::Token},
    services::authentication_command_service::{SessionRepository, TokenService},
};
use crate::iam::federation::domain::{
    error::FederationError, services::google_oauth_service::GoogleOAuthService,
};
use crate::iam::identity::domain::{
    model::{
        aggregates::identity::Identity,
        value_objects::{
            auth_provider::AuthProvider, email::Email, identity_id::IdentityId, password::Password,
        },
    },
    repositories::identity_repository::IdentityRepository,
};
use crate::shared::domain::model::entities::auditable_model::AuditableModel;

pub struct GoogleFederationService<R, T, S, O>
where
    R: IdentityRepository,
    T: TokenService,
    S: SessionRepository,
    O: GoogleOAuthService,
{
    identity_repository: R,
    token_service: T,
    session_repository: S,
    oauth_service: O,
    refresh_token_duration_seconds: u64,
}

impl<R, T, S, O> GoogleFederationService<R, T, S, O>
where
    R: IdentityRepository,
    T: TokenService,
    S: SessionRepository,
    O: GoogleOAuthService,
{
    pub fn new(
        identity_repository: R,
        token_service: T,
        session_repository: S,
        oauth_service: O,
        refresh_token_duration_seconds: u64,
    ) -> Self {
        Self {
            identity_repository,
            token_service,
            session_repository,
            oauth_service,
            refresh_token_duration_seconds,
        }
    }

    pub async fn authenticate(
        &self,
        code: String,
    ) -> Result<(Token, RefreshToken), FederationError> {
        if code.trim().is_empty() {
            return Err(FederationError::InvalidAuthorizationCode);
        }

        let google_user = self.oauth_service.exchange_code(code).await?;

        if !google_user.email_verified {
            return Err(FederationError::EmailNotVerified);
        }

        let email =
            Email::new(google_user.email.clone()).map_err(|_| FederationError::InvalidEmail)?;

        let existing_identity = self
            .identity_repository
            .find_by_email(&email)
            .await
            .map_err(|e| FederationError::Internal(e.to_string()))?;

        let user_id = match existing_identity {
            Some(identity) => {
                if identity.provider() != &AuthProvider::Google {
                    return Err(FederationError::ProviderMismatch);
                }
                identity.id().value()
            }
            None => {
                let mut random_bytes = [0u8; 16];
                rand::rng().fill_bytes(&mut random_bytes);
                let placeholder_password = hex::encode(random_bytes);

                let hashed = hash(placeholder_password, DEFAULT_COST)
                    .map_err(|e| FederationError::Internal(e.to_string()))?;
                let password = Password::new(hashed).map_err(FederationError::Internal)?;

                let identity = Identity::new(
                    IdentityId::new(),
                    email.clone(),
                    password,
                    AuthProvider::Google,
                    AuditableModel::new(),
                );

                let persisted = self
                    .identity_repository
                    .save(identity)
                    .await
                    .map_err(|e| FederationError::Internal(e.to_string()))?;

                persisted.id().value()
            }
        };

        let (token, jti) = self
            .token_service
            .generate_token(user_id)
            .map_err(|e| FederationError::Internal(e.to_string()))?;

        let refresh_token = self
            .token_service
            .generate_refresh_token()
            .map_err(|e| FederationError::Internal(e.to_string()))?;

        self.session_repository
            .create_session(user_id, &jti)
            .await
            .map_err(|e| FederationError::Internal(e.to_string()))?;

        self.session_repository
            .save_refresh_token(user_id, &refresh_token, self.refresh_token_duration_seconds)
            .await
            .map_err(|e| FederationError::Internal(e.to_string()))?;

        Ok((token, refresh_token))
    }
}
