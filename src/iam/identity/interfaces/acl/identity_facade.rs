use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

#[async_trait]
pub trait IdentityFacade: Send + Sync {
    async fn verify_credentials(
        &self,
        email: String,
        password: String,
    ) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>>;

    async fn user_exists(&self, email: String) -> Result<bool, Box<dyn Error + Send + Sync>>;
}
