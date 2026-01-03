use async_trait::async_trait;
use uuid::Uuid;
use std::error::Error;

#[async_trait]
pub trait IdentityFacade: Send + Sync {
    async fn verify_credentials(&self, email: String, password: String) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>>;
}
