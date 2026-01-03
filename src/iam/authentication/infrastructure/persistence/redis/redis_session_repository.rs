use crate::iam::authentication::domain::services::authentication_command_service::SessionRepository;
use crate::iam::authentication::domain::model::value_objects::token::Token;
use async_trait::async_trait;
use redis::AsyncCommands;
use uuid::Uuid;
use std::error::Error;

pub struct RedisSessionRepository {
    client: redis::Client,
    session_duration_seconds: u64,
}

impl RedisSessionRepository {
    pub fn new(client: redis::Client, session_duration_seconds: u64) -> Self {
        Self { client, session_duration_seconds }
    }
}

#[async_trait]
impl SessionRepository for RedisSessionRepository {
    async fn create_session(&self, user_id: Uuid, token: &Token) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut con = self.client.get_multiplexed_async_connection().await.map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        let key = format!("session:{}", user_id);
        let _: () = con.set_ex(key, token.value(), self.session_duration_seconds).await.map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(())
    }
}
