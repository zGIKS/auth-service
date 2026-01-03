use async_trait::async_trait;
use uuid::Uuid;
use crate::iam::authentication::domain::model::value_objects::token::Token;
use std::error::Error;

#[async_trait]
pub trait TokenService: Send + Sync {
    fn generate_token(&self, user_id: Uuid) -> Result<Token, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create_session(&self, user_id: Uuid, token: &Token) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait AuthenticationCommandService: Send + Sync {
    async fn login(&self, command: crate::iam::authentication::domain::model::commands::login_command::LoginCommand) -> Result<Token, Box<dyn Error + Send + Sync>>;
}
