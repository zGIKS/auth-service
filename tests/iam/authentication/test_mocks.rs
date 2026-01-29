/// Shared mocks for authentication tests
use auth_service::iam::authentication::domain::model::value_objects::{
    claims::Claims, refresh_token::RefreshToken, token::Token,
};
use auth_service::iam::authentication::domain::services::authentication_command_service::{
    SessionRepository, TokenService,
};
use auth_service::iam::identity::interfaces::acl::identity_facade::IdentityFacade;
use mockall::mock;
use std::error::Error;
use uuid::Uuid;

use auth_service::shared::infrastructure::services::account_lockout::{
    AccountLockoutVerifier, LockoutError,
};

// Mock IdentityFacade using shim pattern for async traits
mock! {
    pub IdentityFacadeShim {
        pub fn verify_credentials(&self, email: String, password: String) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>>;
        pub fn user_exists(&self, email: String) -> Result<bool, Box<dyn Error + Send + Sync>>;
    }
}

#[async_trait::async_trait]
impl IdentityFacade for MockIdentityFacadeShim {
    async fn verify_credentials(
        &self,
        email: String,
        password: String,
    ) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>> {
        self.verify_credentials(email, password)
    }

    async fn user_exists(&self, email: String) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.user_exists(email)
    }
}

// Mock TokenService (synchronous, direct mock)
mock! {
    pub TokenServiceShim {}

    impl TokenService for TokenServiceShim {
        fn generate_token(&self, user_id: Uuid) -> Result<(Token, String), Box<dyn Error + Send + Sync>>;
        fn generate_refresh_token(&self) -> Result<RefreshToken, Box<dyn Error + Send + Sync>>;
        fn validate_token(&self, token: &str) -> Result<Claims, Box<dyn Error + Send + Sync>>;
    }
}

// Mock SessionRepository using shim pattern for async traits
mock! {
    pub SessionRepositoryShim {
        pub fn create_session(&self, user_id: Uuid, jti: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
        pub fn get_session_jti(&self, user_id: Uuid) -> Result<Option<String>, Box<dyn Error + Send + Sync>>;
        pub fn save_refresh_token(&self, user_id: Uuid, refresh_token: RefreshToken, ttl_seconds: u64) -> Result<(), Box<dyn Error + Send + Sync>>;
        pub fn get_user_by_refresh_token(&self, refresh_token: RefreshToken) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>>;
        pub fn delete_refresh_token(&self, refresh_token: RefreshToken) -> Result<(), Box<dyn Error + Send + Sync>>;
        pub fn revoke_all_user_sessions(&self, user_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>>;
        pub fn is_jti_blacklisted(&self, jti: &str) -> Result<bool, Box<dyn Error + Send + Sync>>;
        pub fn get_user_invalidation_timestamp(&self, user_id: Uuid) -> Result<Option<u64>, Box<dyn Error + Send + Sync>>;
        pub fn delete_session(&self, user_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>>;
    }
}

#[async_trait::async_trait]
impl SessionRepository for MockSessionRepositoryShim {
    async fn create_session(
        &self,
        user_id: Uuid,
        jti: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.create_session(user_id, jti)
    }

    async fn get_session_jti(
        &self,
        user_id: Uuid,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        self.get_session_jti(user_id)
    }

    async fn save_refresh_token(
        &self,
        user_id: Uuid,
        refresh_token: &RefreshToken,
        ttl_seconds: u64,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.save_refresh_token(user_id, refresh_token.clone(), ttl_seconds)
    }

    async fn get_user_by_refresh_token(
        &self,
        refresh_token: &RefreshToken,
    ) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>> {
        self.get_user_by_refresh_token(refresh_token.clone())
    }

    async fn delete_refresh_token(
        &self,
        refresh_token: &RefreshToken,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.delete_refresh_token(refresh_token.clone())
    }

    async fn revoke_all_user_sessions(
        &self,
        user_id: Uuid,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.revoke_all_user_sessions(user_id)
    }

    async fn is_jti_blacklisted(&self, jti: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.is_jti_blacklisted(jti)
    }

    async fn get_user_invalidation_timestamp(
        &self,
        user_id: Uuid,
    ) -> Result<Option<u64>, Box<dyn Error + Send + Sync>> {
        self.get_user_invalidation_timestamp(user_id)
    }

    async fn delete_session(&self, user_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.delete_session(user_id)
    }
}

// Mock AccountLockoutVerifier
mock! {
    pub AccountLockoutVerifierShim {
        pub fn check_locked(&self, identity: String, ip: Option<String>) -> Result<(), LockoutError>;
        pub fn register_failure(&self, identity: String, ip: Option<String>, threshold: u64, lock_duration_sec: u64) -> Result<bool, LockoutError>;
        pub fn reset_failure(&self, identity: String, ip: Option<String>) -> Result<(), LockoutError>;
    }
}

#[async_trait::async_trait]
impl AccountLockoutVerifier for MockAccountLockoutVerifierShim {
    async fn check_locked(&self, identity: &str, ip: Option<&str>) -> Result<(), LockoutError> {
        self.check_locked(identity.to_string(), ip.map(|s| s.to_string()))
    }

    async fn register_failure(
        &self,
        identity: &str,
        ip: Option<&str>,
        threshold: u64,
        lock_duration_sec: u64,
    ) -> Result<bool, LockoutError> {
        self.register_failure(
            identity.to_string(),
            ip.map(|s| s.to_string()),
            threshold,
            lock_duration_sec,
        )
    }

    async fn reset_failure(&self, identity: &str, ip: Option<&str>) -> Result<(), LockoutError> {
        self.reset_failure(identity.to_string(), ip.map(|s| s.to_string()))
    }
}
