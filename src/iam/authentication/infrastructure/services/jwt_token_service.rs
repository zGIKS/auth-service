use crate::iam::authentication::domain::services::authentication_command_service::TokenService;
use crate::iam::authentication::domain::model::value_objects::{token::Token, refresh_token::RefreshToken};
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::error::Error;
use chrono::{Utc, Duration};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub struct JwtTokenService {
    secret: String,
    duration_seconds: u64,
}

impl JwtTokenService {
    pub fn new(secret: String, duration_seconds: u64) -> Self {
        Self { secret, duration_seconds }
    }
}

impl TokenService for JwtTokenService {
    fn generate_token(&self, user_id: Uuid) -> Result<Token, Box<dyn Error + Send + Sync>> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::seconds(self.duration_seconds as i64))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration as usize,
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_bytes()))
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok(Token::new(token))
    }

    fn generate_refresh_token(&self) -> Result<RefreshToken, Box<dyn Error + Send + Sync>> {
        // Simple opaque token using UUID
        let token = Uuid::new_v4().to_string();
        Ok(RefreshToken::new(token))
    }
}