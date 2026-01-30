use crate::iam::authentication::domain::model::{
    commands::{
        logout_command::LogoutCommand, refresh_token_command::RefreshTokenCommand,
        signin_command::SigninCommand,
    },
    value_objects::{claims::Claims, refresh_token::RefreshToken, token::Token},
};
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

#[async_trait]
pub trait TokenService: Send + Sync {
    fn generate_token(
        &self,
        user_id: Uuid,
    ) -> Result<(Token, String), Box<dyn Error + Send + Sync>>;
    fn generate_refresh_token(&self) -> Result<RefreshToken, Box<dyn Error + Send + Sync>>;
    // New validation method
    fn validate_token(&self, token: &str) -> Result<Claims, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create_session(
        &self,
        user_id: Uuid,
        jti: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn get_session_jti(
        &self,
        user_id: Uuid,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>>;

    // Refresh Token Management
    async fn save_refresh_token(
        &self,
        user_id: Uuid,
        refresh_token: &RefreshToken,
        ttl_seconds: u64,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn get_user_by_refresh_token(
        &self,
        refresh_token: &RefreshToken,
    ) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>>;
    async fn delete_refresh_token(
        &self,
        refresh_token: &RefreshToken,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;

    // Session Management
    async fn revoke_all_user_sessions(
        &self,
        user_id: Uuid,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn is_jti_blacklisted(&self, jti: &str) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn get_user_invalidation_timestamp(
        &self,
        user_id: Uuid,
    ) -> Result<Option<u64>, Box<dyn Error + Send + Sync>>;
    async fn delete_session(&self, user_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait AuthenticationCommandService: Send + Sync {
    async fn signin(
        &self,
        command: SigninCommand,
    ) -> Result<(Token, RefreshToken), Box<dyn Error + Send + Sync>>;
    async fn refresh_token(
        &self,
        command: RefreshTokenCommand,
    ) -> Result<(Token, RefreshToken), Box<dyn Error + Send + Sync>>;
    async fn logout(&self, command: LogoutCommand) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait AuthenticationQueryService: Send + Sync {
    async fn verify_token(&self, token: &str) -> Result<Claims, Box<dyn Error + Send + Sync>>;
}
