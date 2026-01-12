use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

#[async_trait]
pub trait SessionInvalidationService: Send + Sync {
    async fn invalidate_all_sessions(
        &self,
        user_id: Uuid,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}
