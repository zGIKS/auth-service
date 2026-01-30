use async_trait::async_trait;
use auth_service::iam::identity::domain::error::DomainError;
/// Shared mocks for identity tests
use auth_service::iam::identity::domain::model::aggregates::identity::Identity;
use auth_service::iam::identity::domain::model::value_objects::{
    email::Email, pending_identity::PendingIdentity,
};
use auth_service::iam::identity::domain::repositories::{
    identity_repository::IdentityRepository,
    password_reset_token_repository::PasswordResetTokenRepository,
    pending_identity_repository::PendingIdentityRepository,
};
use auth_service::iam::identity::domain::services::notification_service::NotificationService;
use auth_service::iam::identity::domain::services::session_invalidation_service::SessionInvalidationService;
use mockall::mock;
use std::error::Error;
use std::future::Future;
use std::time::Duration;
use uuid::Uuid;

mock! {
    pub IdentityRepository {}

    impl IdentityRepository for IdentityRepository {
        fn save(&self, identity: Identity) -> impl Future<Output = Result<Identity, Box<dyn Error + Send + Sync>>> + Send;
        fn find_by_email(&self, email: &Email) -> impl Future<Output = Result<Option<Identity>, Box<dyn Error + Send + Sync>>> + Send;
    }
}

mock! {
    pub PendingIdentityRepository {}

    #[async_trait]
    impl PendingIdentityRepository for PendingIdentityRepository {
        async fn save(&self, pending_identity: PendingIdentity, token_hash: String, ttl: Duration) -> Result<(), DomainError>;
        async fn find(&self, token_hash: &str) -> Result<Option<PendingIdentity>, DomainError>;
        async fn delete(&self, token_hash: &str) -> Result<(), DomainError>;
        async fn find_token_by_email(&self, email: &str) -> Result<Option<String>, DomainError>;
    }
}

mock! {
    pub PasswordResetTokenRepository {}

    #[async_trait]
    impl PasswordResetTokenRepository for PasswordResetTokenRepository {
        async fn save(&self, email: String, token_hash: String, ttl: Duration) -> Result<(), DomainError>;
        async fn find_email_by_token(&self, token_hash: &str) -> Result<Option<String>, DomainError>;
        async fn delete(&self, token_hash: &str) -> Result<(), DomainError>;
    }
}

mock! {
    pub NotificationService {}

    #[async_trait]
    impl NotificationService for NotificationService {
        async fn send_verification_email(&self, to: &str, token: &str) -> Result<(), DomainError>;
        async fn send_password_reset_email(&self, to: &str, reset_link: &str) -> Result<(), DomainError>;
    }
}

mock! {
    pub SessionInvalidationService {}

    #[async_trait]
    impl SessionInvalidationService for SessionInvalidationService {
        async fn invalidate_all_sessions(&self, user_id: Uuid) -> Result<(), Box<dyn Error + Send + Sync>>;
    }
}
