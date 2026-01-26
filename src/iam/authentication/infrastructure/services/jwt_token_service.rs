use crate::iam::authentication::domain::model::value_objects::{
    claims::Claims, refresh_token::RefreshToken, token::Token,
};
use crate::iam::authentication::domain::services::authentication_command_service::TokenService;
use chrono::{Duration, Utc};
use hex;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    sub: String,
    exp: usize,
    jti: String,
    iat: usize,
}

pub struct JwtTokenService {
    secret: String,
    duration_seconds: u64,
}

impl JwtTokenService {
    pub fn new(secret: String, duration_seconds: u64) -> Self {
        Self {
            secret,
            duration_seconds,
        }
    }
}

impl TokenService for JwtTokenService {
    fn generate_token(
        &self,
        user_id: Uuid,
    ) -> Result<(Token, String), Box<dyn Error + Send + Sync>> {
        let now = Utc::now();
        let expiration = now
            .checked_add_signed(Duration::seconds(self.duration_seconds as i64))
            .expect("valid timestamp")
            .timestamp();

        let iat = now.timestamp() as usize;

        let jti = Uuid::new_v4().to_string();

        let claims = JwtClaims {
            sub: user_id.to_string(),
            exp: expiration as usize,
            jti: jti.clone(),
            iat,
        };

        let token_str = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok((Token::new(token_str), jti))
    }

    fn generate_refresh_token(&self) -> Result<RefreshToken, Box<dyn Error + Send + Sync>> {
        let mut key = [0u8; 32];
        rand::rng().fill_bytes(&mut key);
        let token = hex::encode(key);
        Ok(RefreshToken::new(token))
    }

    fn validate_token(&self, token: &str) -> Result<Claims, Box<dyn Error + Send + Sync>> {
        let decoding_key = DecodingKey::from_secret(self.secret.as_bytes());
        let validation = Validation::default();

        let token_data = decode::<JwtClaims>(token, &decoding_key, &validation)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let sub = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok(Claims {
            sub,
            exp: token_data.claims.exp,
            jti: token_data.claims.jti,
            iat: token_data.claims.iat,
        })
    }
}
