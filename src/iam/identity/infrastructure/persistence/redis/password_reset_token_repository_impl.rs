use crate::iam::identity::domain::{
    error::DomainError, repositories::password_reset_token_repository::PasswordResetTokenRepository,
};
use async_trait::async_trait;
use redis::{AsyncCommands, Client};
use std::time::Duration;

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

    fn format_email_key(email: &str) -> String {
        format!("password_reset_email:{}", email)
    }

    fn format_lock_key(email: &str) -> String {
        format!("password_reset_lock:{}", email)
    }
}

#[async_trait]
impl PasswordResetTokenRepository for PasswordResetTokenRepositoryImpl {
    async fn save(
        &self,
        email: String,
        token_hash: String,
        ttl: Duration,
    ) -> Result<(), DomainError> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let lock_key = Self::format_lock_key(&email);
        let token_key = Self::format_key(&token_hash);
        let email_key = Self::format_email_key(&email);
        let ttl_secs = ttl.as_secs();

        // Acquire distributed lock (10 second timeout)
        let lock_acquired: bool = con
            .set_nx(&lock_key, "locked")
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        if !lock_acquired {
            return Err(DomainError::InternalError(
                "Password reset request already in progress for this email".to_string(),
            ));
        }

        // Set lock expiration to prevent deadlock
        let _: () = con
            .expire(&lock_key, 10)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        // Find and delete old token if exists
        let old_token_hash: Option<String> = con
            .get(&email_key)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let mut pipe = redis::pipe();
        let pipe = pipe.atomic();

        // Delete old token if exists
        if let Some(old_hash) = old_token_hash {
            let old_token_key = Self::format_key(&old_hash);
            pipe.del(&old_token_key);
        }

        // Save new token and email mapping
        pipe.set_ex(&token_key, &email, ttl_secs);
        pipe.set_ex(&email_key, &token_hash, ttl_secs);

        let _: () = pipe
            .query_async(&mut con)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        // Release lock
        let _: () = con
            .del(&lock_key)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn find_email_by_token(&self, token_hash: &str) -> Result<Option<String>, DomainError> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let key = Self::format_key(token_hash);
        let email: Option<String> = con
            .get(&key)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        // If token exists, verify it is the latest one for this email
        if let Some(ref e) = email {
            let email_key = Self::format_email_key(e);
            let active_token_hash: Option<String> = con
                .get(&email_key)
                .await
                .map_err(|e| DomainError::InternalError(e.to_string()))?;

            if let Some(active_hash) = active_token_hash {
                if active_hash != token_hash {
                    return Ok(None);
                }
            } else {
                // Inconsistent state: token exists but no mapping from email.
                // Treat as invalid or allow? strict: invalid.
                return Ok(None);
            }
        }

        Ok(email)
    }

    async fn delete(&self, token_hash: &str) -> Result<(), DomainError> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let key = Self::format_key(token_hash);

        // Get email to clean up the secondary index
        let email: Option<String> = con
            .get(&key)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let mut pipe = redis::pipe();
        let pipe = pipe.atomic();

        pipe.del(&key);

        if let Some(e) = email {
            let email_key = Self::format_email_key(&e);
            pipe.del(email_key);
        }

        let _: () = pipe
            .query_async(&mut con)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(())
    }
}
