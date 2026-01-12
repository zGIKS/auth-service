use async_trait::async_trait;
use uuid::Uuid;
use std::error::Error;

#[async_trait]
pub trait SessionInvalidationService: Send + Sync {
    async fn invalidate_all_sessions(&self, user_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>>;
}
