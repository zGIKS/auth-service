use crate::iam::authentication::domain::{
    model::value_objects::claims::Claims,
    services::authentication_command_service::{
        AuthenticationQueryService, SessionRepository, TokenService,
    },
};
use std::error::Error;

pub struct AuthenticationQueryServiceImpl<T, S>
where
    T: TokenService,
    S: SessionRepository,
{
    token_service: T,
    session_repository: S,
}

impl<T, S> AuthenticationQueryServiceImpl<T, S>
where
    T: TokenService,
    S: SessionRepository,
{
    pub fn new(token_service: T, session_repository: S) -> Self {
        Self {
            token_service,
            session_repository,
        }
    }
}

#[async_trait::async_trait]
impl<T, S> AuthenticationQueryService for AuthenticationQueryServiceImpl<T, S>
where
    T: TokenService,
    S: SessionRepository,
{
    async fn verify_token(&self, token: &str) -> Result<Claims, Box<dyn Error + Send + Sync>> {
        // 1. Validate signature and expiration
        let claims = self.token_service.validate_token(token)?;

        // 2. Check if JTI is blacklisted
        if self
            .session_repository
            .is_jti_blacklisted(&claims.jti)
            .await?
        {
            return Err("Token has been revoked".into());
        }

        // 3. Check if token was issued before the global invalidation timestamp
        if let Some(invalidation_timestamp) = self
            .session_repository
            .get_user_invalidation_timestamp(claims.sub)
            .await?
        {
            // claims.iat is usize (seconds), invalidation_timestamp is u64 (seconds)
            if (claims.iat as u64) < invalidation_timestamp {
                return Err("Token invalidated by password reset or logout".into());
            }
        }

        // 4. Check if session exists in Redis and matches JTI
        // This ensures that if the session is deleted (e.g. forced logout), the token is invalid immediately
        let active_jti = self.session_repository.get_session_jti(claims.sub).await?;

        match active_jti {
            Some(jti) => {
                if jti != claims.jti {
                    return Err("Token does not match active session".into());
                }
            }
            None => {
                return Err("Session has expired or does not exist".into());
            }
        }

        Ok(claims)
    }
}
