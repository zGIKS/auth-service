use async_trait::async_trait;

use crate::iam::federation::domain::{
    error::FederationError, model::value_objects::google_user::GoogleUser,
};

#[async_trait]
pub trait GoogleOAuthService: Send + Sync {
    async fn exchange_code(&self, code: String) -> Result<GoogleUser, FederationError>;
}
