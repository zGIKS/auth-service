use async_trait::async_trait;
use redis::{Client, AsyncCommands};
use std::time::Duration;
use crate::iam::identity::domain::{
    error::DomainError,
    repositories::password_reset_token_repository::PasswordResetTokenRepository,
};

pub struct PasswordResetTokenRepositoryImpl {
    client: Client,
}

impl PasswordResetTokenRepositoryImpl {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn format_key(token_hash: &str) -> String {
        format!("password_reset:{}", token_hash)
    }
}

#[async_trait]
impl PasswordResetTokenRepository for PasswordResetTokenRepositoryImpl {
    async fn save(&self, email: String, token_hash: String, ttl: Duration) -> Result<(), DomainError> {
        let mut con = self.client.get_multiplexed_async_connection().await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let key = Self::format_key(&token_hash);
        let ttl_secs = ttl.as_secs();

        // Use SETEX for atomic set with expiration
        let _: () = con.set_ex(&key, email, ttl_secs).await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn find_email_by_token(&self, token_hash: &str) -> Result<Option<String>, DomainError> {
        let mut con = self.client.get_multiplexed_async_connection().await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let key = Self::format_key(token_hash);
        let email: Option<String> = con.get(&key).await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(email)
    }

    async fn delete(&self, token_hash: &str) -> Result<(), DomainError> {
        let mut con = self.client.get_multiplexed_async_connection().await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let key = Self::format_key(token_hash);
        let _: () = con.del(&key).await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(())
    }
}