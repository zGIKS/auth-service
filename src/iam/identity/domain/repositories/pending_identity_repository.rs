use crate::iam::identity::domain::{
    error::DomainError, model::value_objects::pending_identity::PendingIdentity,
};
use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait PendingIdentityRepository: Send + Sync {
    async fn save(
        &self,
        pending_identity: PendingIdentity,
        token_hash: String,
        ttl: Duration,
    ) -> Result<(), DomainError>;
    async fn find(&self, token_hash: &str) -> Result<Option<PendingIdentity>, DomainError>;
    async fn delete(&self, token_hash: &str) -> Result<(), DomainError>;
    async fn find_token_by_email(&self, email: &str) -> Result<Option<String>, DomainError>;
}
