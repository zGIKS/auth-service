use crate::iam::authentication::domain::model::value_objects::refresh_token::RefreshToken;
use crate::iam::authentication::domain::services::authentication_command_service::SessionRepository;
use async_trait::async_trait;
use chrono::Utc;
use hex;
use redis::AsyncCommands;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::str::FromStr;
use uuid::Uuid;

pub struct RedisSessionRepository {
    client: redis::Client,
    session_duration_seconds: u64,
}

impl RedisSessionRepository {
    pub fn new(client: redis::Client, session_duration_seconds: u64) -> Self {
        Self {
            client,
            session_duration_seconds,
        }
    }

    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }
}

#[async_trait]
impl SessionRepository for RedisSessionRepository {
    async fn create_session(
        &self,
        user_id: Uuid,
        jti: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        // Store session ID -> JTI (not the full token)
        let key = format!("session:{}", user_id);
        let _: () = con
            .set_ex(key, jti, self.session_duration_seconds)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok(())
    }

    async fn get_session_jti(
        &self,
        user_id: Uuid,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let key = format!("session:{}", user_id);
        let jti: Option<String> = con
            .get(key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok(jti)
    }

    async fn save_refresh_token(
        &self,
        user_id: Uuid,
        refresh_token: &RefreshToken,
        ttl_seconds: u64,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        // Hash the token before using it as a key
        let hashed_token = self.hash_token(refresh_token.value());
        let key = format!("refresh_token:{}", hashed_token);

        // Store UserID -> we can look up UserID by Token Hash
        let _: () = con
            .set_ex(key.clone(), user_id.to_string(), ttl_seconds)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        // Also store a mapping of UserID -> Set of Refresh Token Hashes
        let user_tokens_key = format!("user_tokens:{}", user_id);
        let _: () = con
            .sadd(user_tokens_key.clone(), hashed_token)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        let _: () = con
            .expire(user_tokens_key, ttl_seconds as i64)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok(())
    }

    async fn get_user_by_refresh_token(
        &self,
        refresh_token: &RefreshToken,
    ) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let hashed_token = self.hash_token(refresh_token.value());
        let key = format!("refresh_token:{}", hashed_token);

        let user_id_str: Option<String> = con
            .get(key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        match user_id_str {
            Some(s) => Ok(Some(Uuid::from_str(&s)?)),
            None => Ok(None),
        }
    }

    async fn delete_refresh_token(
        &self,
        refresh_token: &RefreshToken,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let hashed_token = self.hash_token(refresh_token.value());
        let key = format!("refresh_token:{}", hashed_token);

        // Before deleting, try to get the user_id to clean up the set
        let user_id_str: Option<String> = con.get(key.clone()).await.ok().flatten();

        let _: () = con
            .del(key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        if let Some(uid) = user_id_str {
            let user_tokens_key = format!("user_tokens:{}", uid);
            let _: () = con
                .srem(user_tokens_key, hashed_token)
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        }

        Ok(())
    }

    async fn revoke_all_user_sessions(
        &self,
        user_id: Uuid,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        // 1. Blacklist the current active JTI
        let session_key = format!("session:{}", user_id);
        let current_jti: Option<String> = con
            .get(session_key.clone())
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        if let Some(jti) = current_jti {
            let blacklist_key = format!("blacklist:{}", jti);
            // Blacklist for the remaining session duration
            let _: () = con
                .set_ex(blacklist_key, "revoked", self.session_duration_seconds)
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        }

        // 2. Get all refresh token hashes for the user
        let user_tokens_key = format!("user_tokens:{}", user_id);
        let token_hashes: Vec<String> = con
            .smembers(user_tokens_key.clone())
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        // 3. Delete each refresh token key
        for hash in token_hashes {
            let key = format!("refresh_token:{}", hash);
            let _: () = con
                .del(key)
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        }

        // 4. Delete the user's token set
        let _: () = con
            .del(user_tokens_key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        // 5. Delete the active session key
        let _: () = con
            .del(session_key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        // 6. Set token invalidation timestamp
        // Any token issued BEFORE this timestamp will be considered invalid
        let invalidation_key = format!("invalidation_timestamp:{}", user_id);
        let now = Utc::now().timestamp();
        // Keep this record for longer than max session duration (e.g. 30 days) to match max refresh token life
        let retention_ttl = 30 * 24 * 60 * 60;
        let _: () = con
            .set_ex(invalidation_key, now, retention_ttl)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok(())
    }

    async fn is_jti_blacklisted(&self, jti: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        let key = format!("blacklist:{}", jti);
        let exists: bool = con
            .exists(key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        Ok(exists)
    }

    async fn get_user_invalidation_timestamp(
        &self,
        user_id: Uuid,
    ) -> Result<Option<u64>, Box<dyn Error + Send + Sync>> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
        let key = format!("invalidation_timestamp:{}", user_id);
        let timestamp_str: Option<String> = con
            .get(key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        match timestamp_str {
            Some(s) => {
                Ok(Some(s.parse::<u64>().map_err(|e| {
                    Box::new(e) as Box<dyn Error + Send + Sync>
                })?))
            }
            None => Ok(None),
        }
    }

    async fn delete_session(&self, user_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let key = format!("session:{}", user_id);
        let _: () = con
            .del(key)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok(())
    }
}
