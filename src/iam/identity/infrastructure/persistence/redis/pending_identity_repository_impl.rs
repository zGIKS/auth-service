use crate::iam::identity::domain::{
    error::DomainError, model::value_objects::pending_identity::PendingIdentity,
    repositories::pending_identity_repository::PendingIdentityRepository,
};
use async_trait::async_trait;
use redis::{AsyncCommands, Client};
use std::time::Duration;

pub struct PendingIdentityRepositoryImpl {
    client: Client,
}

impl PendingIdentityRepositoryImpl {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]

impl PendingIdentityRepository for PendingIdentityRepositoryImpl {
    async fn save(
        &self,
        pending_identity: PendingIdentity,
        token_hash: String,
        ttl: Duration,
    ) -> Result<(), DomainError> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let key = format!("pending_identity:{}", token_hash);
        let email_key = format!("pending_email:{}", pending_identity.email);

        let value = serde_json::to_string(&pending_identity)
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let _: () = con
            .set_ex(&key, value, ttl.as_secs())
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let _: () = con
            .set_ex(&email_key, &token_hash, ttl.as_secs())
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn find(&self, token_hash: &str) -> Result<Option<PendingIdentity>, DomainError> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let key = format!("pending_identity:{}", token_hash);
        let value: Option<String> = con
            .get(key)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        match value {
            Some(v) => {
                let pending: PendingIdentity = serde_json::from_str(&v)
                    .map_err(|e| DomainError::InternalError(e.to_string()))?;
                Ok(Some(pending))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, token_hash: &str) -> Result<(), DomainError> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let key = format!("pending_identity:{}", token_hash);

        // Try to find the identity first to get the email for cleanup
        let value: Option<String> = con
            .get(&key)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        if let Some(v) = value
            && let Ok(pending) = serde_json::from_str::<PendingIdentity>(&v)
        {
            let email_key = format!("pending_email:{}", pending.email);
            let _: () = con
                .del(email_key)
                .await
                .map_err(|e| DomainError::InternalError(e.to_string()))?;
        }

        let _: () = con
            .del(key)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn find_token_by_email(&self, email: &str) -> Result<Option<String>, DomainError> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        let key = format!("pending_email:{}", email);
        let value: Option<String> = con
            .get(key)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(value)
    }
}
