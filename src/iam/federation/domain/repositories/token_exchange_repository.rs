use crate::iam::federation::domain::error::FederationError;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct ExchangeTokens {
    pub access_token: String,
    pub refresh_token: String,
}

#[async_trait]
pub trait TokenExchangeRepository: Send + Sync {
    async fn save(&self, tokens: ExchangeTokens) -> Result<String, FederationError>;
    async fn claim(&self, code: String) -> Result<Option<ExchangeTokens>, FederationError>;
}
