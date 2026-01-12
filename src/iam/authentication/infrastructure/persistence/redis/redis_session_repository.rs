use crate::iam::authentication::domain::services::authentication_command_service::SessionRepository;
use crate::iam::authentication::domain::model::value_objects::{token::Token, refresh_token::RefreshToken};
use async_trait::async_trait;
use redis::AsyncCommands;
use uuid::Uuid;
use std::error::Error;
use std::str::FromStr;

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

    async fn save_refresh_token(&self, user_id: Uuid, refresh_token: &RefreshToken, ttl_seconds: u64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut con = self.client.get_multiplexed_async_connection().await.map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        let key = format!("refresh_token:{}", refresh_token.value());
        let _: () = con.set_ex(key, user_id.to_string(), ttl_seconds).await.map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(())
    }

    async fn get_user_by_refresh_token(&self, refresh_token: &RefreshToken) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>> {
        let mut con = self.client.get_multiplexed_async_connection().await.map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        let key = format!("refresh_token:{}", refresh_token.value());
        let user_id_str: Option<String> = con.get(key).await.map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        
        match user_id_str {
            Some(s) => Ok(Some(Uuid::from_str(&s)?)),
            None => Ok(None),
        }
    }

    async fn delete_refresh_token(&self, refresh_token: &RefreshToken) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut con = self.client.get_multiplexed_async_connection().await.map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        let key = format!("refresh_token:{}", refresh_token.value());
        let _: () = con.del(key).await.map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(())
    }
}