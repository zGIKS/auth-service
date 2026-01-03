use crate::iam::authentication::domain::services::authentication_command_service::TokenService;
use crate::iam::authentication::domain::model::value_objects::token::Token;
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
}

impl JwtTokenService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

impl TokenService for JwtTokenService {
    fn generate_token(&self, user_id: Uuid) -> Result<Token, Box<dyn Error + Send + Sync>> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(1))
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
}
