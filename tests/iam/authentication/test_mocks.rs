/// Shared mocks for authentication tests
use auth_service::iam::authentication::domain::model::value_objects::token::Token;
use auth_service::iam::authentication::domain::services::authentication_command_service::{SessionRepository, TokenService};
use auth_service::iam::identity::interfaces::acl::identity_facade::IdentityFacade;
use mockall::mock;
use uuid::Uuid;
use std::error::Error;

// Mock IdentityFacade using shim pattern for async traits
mock! {
    pub IdentityFacadeShim {
        pub fn verify_credentials(&self, email: String, password: String) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>>;
    }
}

#[async_trait::async_trait]
impl IdentityFacade for MockIdentityFacadeShim {
    async fn verify_credentials(&self, email: String, password: String) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>> {
        self.verify_credentials(email, password)
    }
}

// Mock TokenService (synchronous, direct mock)
mock! {
    pub TokenServiceShim {}
    
    impl TokenService for TokenServiceShim {
        fn generate_token(&self, user_id: Uuid) -> Result<Token, Box<dyn Error + Send + Sync>>;
    }
}

// Mock SessionRepository using shim pattern for async traits
mock! {
    pub SessionRepositoryShim {
        pub fn create_session(&self, user_id: Uuid, token: Token) -> Result<(), Box<dyn Error + Send + Sync>>;
    }
}

#[async_trait::async_trait]
impl SessionRepository for MockSessionRepositoryShim {
    async fn create_session(&self, user_id: Uuid, token: &Token) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.create_session(user_id, token.clone())
    }
}
