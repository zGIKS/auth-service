use async_trait::async_trait;
use redis::{AsyncCommands, cmd};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::iam::federation::domain::{
    error::FederationError,
    repositories::token_exchange_repository::{ExchangeTokens, TokenExchangeRepository},
};

#[derive(Serialize, Deserialize)]
struct RedisExchangeTokens {
    access_token: String,
    refresh_token: String,
}

pub struct TokenExchangeRepositoryImpl {
    client: redis::Client,
    ttl: Duration,
}

impl TokenExchangeRepositoryImpl {
    pub fn new(client: redis::Client) -> Self {
        Self {
            client,
            ttl: Duration::from_secs(60), // Code valid for 60 seconds
        }
    }
}

#[async_trait]
impl TokenExchangeRepository for TokenExchangeRepositoryImpl {
    async fn save(&self, tokens: ExchangeTokens) -> Result<String, FederationError> {
        let code = Uuid::new_v4().to_string();
        let key = format!("google_exchange:{}", code);

        let redis_tokens = RedisExchangeTokens {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        };

        let value = serde_json::to_string(&redis_tokens)
            .map_err(|e| FederationError::Internal(format!("Serialization error: {}", e)))?;

        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| FederationError::Internal(format!("Redis connection error: {}", e)))?;

        let _: () = con
            .set_ex(&key, value, self.ttl.as_secs())
            .await
            .map_err(|e| FederationError::Internal(format!("Redis save error: {}", e)))?;

        Ok(code)
    }

    async fn claim(&self, code: String) -> Result<Option<ExchangeTokens>, FederationError> {
        let key = format!("google_exchange:{}", code);
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| FederationError::Internal(format!("Redis connection error: {}", e)))?;

        // Atomic: get and delete in a single operation (Redis >= 6.2)
        let value: Option<String> = cmd("GETDEL")
            .arg(&key)
            .query_async(&mut con)
            .await
            .map_err(|e| FederationError::Internal(format!("Redis GETDEL error: {}", e)))?;

        if let Some(v) = value {
            let redis_tokens: RedisExchangeTokens = serde_json::from_str(&v)
                .map_err(|e| FederationError::Internal(format!("Deserialization error: {}", e)))?;

            Ok(Some(ExchangeTokens {
                access_token: redis_tokens.access_token,
                refresh_token: redis_tokens.refresh_token,
            }))
        } else {
            Ok(None)
        }
    }
}
