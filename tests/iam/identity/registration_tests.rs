use super::test_mocks::*;
use async_trait::async_trait;
use auth_service::iam::identity::application::command_services::identity_command_service_impl::IdentityCommandServiceImpl;
use auth_service::iam::identity::domain::error::DomainError;
use auth_service::iam::identity::domain::model::aggregates::identity::Identity;
use auth_service::iam::identity::domain::model::commands::confirm_registration_command::ConfirmRegistrationCommand;
use auth_service::iam::identity::domain::model::commands::register_identity_command::RegisterIdentityCommand;
use auth_service::iam::identity::domain::model::queries::confirm_email_query::ConfirmEmailQuery;
use auth_service::iam::identity::domain::model::value_objects::identity_id::IdentityId;
use auth_service::iam::identity::domain::model::value_objects::{
    auth_provider::AuthProvider, email::Email, password::Password,
    pending_identity::PendingIdentity,
};
use auth_service::iam::identity::domain::repositories::{
    identity_repository::IdentityRepository,
    password_reset_token_repository::PasswordResetTokenRepository,
    pending_identity_repository::PendingIdentityRepository,
};
use auth_service::iam::identity::domain::services::identity_command_service::IdentityCommandService;
use auth_service::iam::identity::domain::services::notification_service::NotificationService;
use auth_service::shared::domain::model::entities::auditable_model::AuditableModel;
use mockall::mock;
use std::error::Error;
use std::future::Future;
use std::time::Duration;

// Mock the repository specifically for this test file
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

#[tokio::test]
async fn test_register_identity_success() {
    let mut mock_repo = MockIdentityRepository::new();
    let mut mock_pending_repo = MockPendingIdentityRepository::new();
    let mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mut mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    mock_repo
        .expect_find_by_email()
        .returning(|_| Box::pin(async { Ok(None) }));

    mock_pending_repo
        .expect_find_token_by_email()
        .times(1)
        .returning(|_| Ok(None));

    mock_pending_repo
        .expect_save()
        .times(1)
        .returning(|_, _, _| Ok(()));

    mock_notification_service
        .expect_send_verification_email()
        .times(1)
        .returning(|_, _| Ok(()));

    let service = IdentityCommandServiceImpl::new(
        mock_repo,
        mock_pending_repo,
        mock_password_reset_repo,
        mock_notification_service,
        mock_session_invalidation_service,
        ttl,
        reset_ttl,
    );

    let email = Email::new("test@gmail.com".to_string()).unwrap();
    let password = Password::new("SecurePass123!".to_string()).unwrap();
    // Default provider is Email
    let command = RegisterIdentityCommand::new(email, password, AuthProvider::Email);

    let result: Result<(Identity, String), DomainError> = service.handle(command).await;
    assert!(result.is_ok());
}

// Test removed: MX validation was removed from registration flow
// Email validation now relies on email confirmation (more reliable)
// MX validation caused: DNS failures, latency, false negatives

#[tokio::test]
async fn test_password_is_hashed_before_saving_pending() {
    let mut mock_repo = MockIdentityRepository::new();
    let mut mock_pending_repo = MockPendingIdentityRepository::new();
    let mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mut mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let plain_password = "SecretPassword123!";

    mock_repo
        .expect_find_by_email()
        .returning(|_| Box::pin(async { Ok(None) }));

    mock_pending_repo
        .expect_find_token_by_email()
        .times(1)
        .returning(|_| Ok(None));

    // Verify that the password sent to pending repo save is hashed
    mock_pending_repo
        .expect_save()
        .withf(move |pending_identity, _, _| {
            let stored_pass = &pending_identity.password_hash;
            // Bcrypt hash always starts with $2
            stored_pass.starts_with("$2") && stored_pass != plain_password
        })
        .times(1)
        .returning(|_, _, _| Ok(()));

    mock_notification_service
        .expect_send_verification_email()
        .times(1)
        .returning(|_, _| Ok(()));

    let service = IdentityCommandServiceImpl::new(
        mock_repo,
        mock_pending_repo,
        mock_password_reset_repo,
        mock_notification_service,
        mock_session_invalidation_service,
        ttl,
        reset_ttl,
    );
    let email = Email::new("hash_test@gmail.com".to_string()).unwrap();
    let password = Password::new(plain_password.to_string()).unwrap();
    let command = RegisterIdentityCommand::new(email, password, AuthProvider::Email);

    let result: Result<(Identity, String), DomainError> = service.handle(command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_identity_overwrites_existing_pending() {
    let mut mock_repo = MockIdentityRepository::new();
    let mut mock_pending_repo = MockPendingIdentityRepository::new();
    let mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mut mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let old_token_hash = "old_token_hash_123";

    mock_repo
        .expect_find_by_email()
        .returning(|_| Box::pin(async { Ok(None) }));

    // 1. Should check for existing pending
    mock_pending_repo
        .expect_find_token_by_email()
        .times(1)
        .returning(move |_| Ok(Some(old_token_hash.to_string())));

    // 2. Should delete the old one
    mock_pending_repo
        .expect_delete()
        .with(mockall::predicate::eq(old_token_hash))
        .times(1)
        .returning(|_| Ok(()));

    // 3. Should save the new one
    mock_pending_repo
        .expect_save()
        .times(1)
        .returning(|_, _, _| Ok(()));

    mock_notification_service
        .expect_send_verification_email()
        .times(1)
        .returning(|_, _| Ok(()));

    let service = IdentityCommandServiceImpl::new(
        mock_repo,
        mock_pending_repo,
        mock_password_reset_repo,
        mock_notification_service,
        mock_session_invalidation_service,
        ttl,
        reset_ttl,
    );
    let email = Email::new("overwrite@gmail.com".to_string()).unwrap();
    let password = Password::new("SecurePass123!".to_string()).unwrap();
    let command = RegisterIdentityCommand::new(email, password, AuthProvider::Email);

    let result: Result<(Identity, String), DomainError> = service.handle(command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_identity_duplicate_email() {
    let mut mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    // Simulate existing user found
    mock_repo.expect_find_by_email().returning(|email| {
        let existing_identity = Identity::new(
            IdentityId::new(),
            email.clone(),
            Password::new("hashed_password_valid_length".to_string()).unwrap(),
            AuthProvider::Email,
            AuditableModel::new(),
        );
        Box::pin(async move { Ok(Some(existing_identity)) })
    });

    let service = IdentityCommandServiceImpl::new(
        mock_repo,
        mock_pending_repo,
        mock_password_reset_repo,
        mock_notification_service,
        mock_session_invalidation_service,
        ttl,
        reset_ttl,
    );

    let email = Email::new("duplicate@gmail.com".to_string()).unwrap();
    let password = Password::new("SecurePass123!".to_string()).unwrap();
    let command = RegisterIdentityCommand::new(email, password, AuthProvider::Email);

    let result: Result<(Identity, String), DomainError> = service.handle(command).await;

    match result {
        Err(DomainError::EmailAlreadyExists) => {} // Expected
        _ => panic!("Expected EmailAlreadyExists error, got {:?}", result),
    }
}

#[tokio::test]
async fn test_confirm_registration_success() {
    let mut mock_repo = MockIdentityRepository::new();
    let mut mock_pending_repo = MockPendingIdentityRepository::new();
    let mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let token_str = "some-uuid-token";
    // In real code we use VerificationToken logic, but here we can just mock what 'find' returns
    // The service computes hash(token) -> finds in pending repo

    // We expect a call to find with SOME hash.
    mock_pending_repo.expect_find().times(1).returning(|_| {
        let pending = PendingIdentity {
            email: "test@gmail.com".to_string(),
            password_hash: "$2a$12$somehash".to_string(),
            provider: "Email".to_string(),
        };
        Ok(Some(pending))
    });

    // We expect a call to save in IdentityRepository (the final persistence)
    mock_repo
        .expect_save()
        .times(1)
        .returning(|identity| Box::pin(async { Ok(identity) }));

    // We expect a call to delete the pending identity
    mock_pending_repo
        .expect_delete()
        .times(1)
        .returning(|_| Ok(()));

    let service = IdentityCommandServiceImpl::new(
        mock_repo,
        mock_pending_repo,
        mock_password_reset_repo,
        mock_notification_service,
        mock_session_invalidation_service,
        ttl,
        reset_ttl,
    );

    let command = ConfirmRegistrationCommand {
        token: token_str.to_string(),
    };

    let result: Result<Identity, DomainError> = service.confirm_registration(command).await;

    assert!(result.is_ok());
    let identity = result.unwrap();
    assert_eq!(identity.email().value(), "test@gmail.com");
}

#[tokio::test]
async fn test_confirm_registration_invalid_token() {
    let mock_repo = MockIdentityRepository::new();
    let mut mock_pending_repo = MockPendingIdentityRepository::new();
    let mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let token_str = "invalid-uuid-token";

    // Simulate token not found (returns None)
    mock_pending_repo
        .expect_find()
        .times(1)
        .returning(|_| Ok(None));

    let service = IdentityCommandServiceImpl::new(
        mock_repo,
        mock_pending_repo,
        mock_password_reset_repo,
        mock_notification_service,
        mock_session_invalidation_service,
        ttl,
        reset_ttl,
    );

    let command = ConfirmRegistrationCommand {
        token: token_str.to_string(),
    };

    let result: Result<Identity, DomainError> = service.confirm_registration(command).await;

    match result {
        Err(DomainError::InvalidToken) => {} // Expected
        _ => panic!("Expected InvalidToken error, got {:?}", result),
    }
}

#[test]
fn test_confirm_email_query_validation_success() {
    // Valid token (32+ characters)
    let token = "a".repeat(32);
    let query = ConfirmEmailQuery::new(token.clone());
    assert!(query.is_ok());
    assert_eq!(query.unwrap().token, token);
}

#[test]
fn test_confirm_email_query_validation_too_short() {
    // Token too short (less than 32 characters)
    let token = "short_token_123".to_string();
    let query = ConfirmEmailQuery::new(token);
    assert!(query.is_err());
}

#[test]
fn test_confirm_email_query_validation_exactly_32() {
    // Exactly 32 characters (boundary test)
    let token = "a".repeat(32);
    let query = ConfirmEmailQuery::new(token);
    assert!(query.is_ok());
}

#[test]
fn test_confirm_email_query_validation_31_chars_fails() {
    // 31 characters (just below limit)
    let token = "a".repeat(31);
    let query = ConfirmEmailQuery::new(token);
    assert!(query.is_err());
}

#[test]
fn test_confirm_email_query_validation_empty_fails() {
    // Empty token
    let token = String::new();
    let query = ConfirmEmailQuery::new(token);
    assert!(query.is_err());
}

#[test]
fn test_confirm_email_query_realistic_token() {
    // Realistic base64-encoded token
    let token = "K9j2mX_pqR8wT4vL3nZ5bH8mP2xQ7wY9A1B2C3D4E5F6G7H8".to_string();
    let query = ConfirmEmailQuery::new(token.clone());
    assert!(query.is_ok());
    assert_eq!(query.unwrap().token, token);
}

// ============================================
// INTEGRATION-STYLE TESTS
// ============================================

#[tokio::test]
async fn test_confirm_registration_with_query_object() {
    let mut mock_repo = MockIdentityRepository::new();
    let mut mock_pending_repo = MockPendingIdentityRepository::new();
    let mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let token_str = "a".repeat(32);

    mock_pending_repo.expect_find().times(1).returning(|_| {
        let pending = PendingIdentity {
            email: "querytest@gmail.com".to_string(),
            password_hash: "$2a$12$somehash".to_string(),
            provider: "Email".to_string(),
        };
        Ok(Some(pending))
    });

    mock_repo
        .expect_save()
        .times(1)
        .returning(|identity| Box::pin(async { Ok(identity) }));

    mock_pending_repo
        .expect_delete()
        .times(1)
        .returning(|_| Ok(()));

    let service = IdentityCommandServiceImpl::new(
        mock_repo,
        mock_pending_repo,
        mock_password_reset_repo,
        mock_notification_service,
        mock_session_invalidation_service,
        ttl,
        reset_ttl,
    );

    // Create query object first (validates token)
    let query = ConfirmEmailQuery::new(token_str.clone());
    assert!(query.is_ok());

    // Then create command from validated query
    let command = ConfirmRegistrationCommand::new(query.unwrap().token);

    let result: Result<Identity, DomainError> = service.confirm_registration(command).await;

    assert!(result.is_ok());
    let identity = result.unwrap();
    assert_eq!(identity.email().value(), "querytest@gmail.com");
}

#[tokio::test]
async fn test_end_to_end_registration_flow() {
    let mut mock_repo = MockIdentityRepository::new();
    let mut mock_pending_repo = MockPendingIdentityRepository::new();
    let mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mut mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    // Step 1: Register
    mock_repo
        .expect_find_by_email()
        .times(1)
        .returning(|_| Box::pin(async { Ok(None) }));

    mock_pending_repo
        .expect_find_token_by_email()
        .times(1)
        .returning(|_| Ok(None));

    mock_pending_repo
        .expect_save()
        .times(1)
        .returning(|_, _, _| Ok(()));

    mock_notification_service
        .expect_send_verification_email()
        .times(1)
        .returning(|_, _| Ok(()));

    // Step 2: Confirm (will be called later)
    mock_pending_repo.expect_find().times(1).returning(|_| {
        let pending = PendingIdentity {
            email: "endtoend@gmail.com".to_string(),
            password_hash: "$2a$12$somehash".to_string(),
            provider: "Email".to_string(),
        };
        Ok(Some(pending))
    });

    mock_repo
        .expect_save()
        .times(1)
        .returning(|identity| Box::pin(async { Ok(identity) }));

    mock_pending_repo
        .expect_delete()
        .times(1)
        .returning(|_| Ok(()));

    let service = IdentityCommandServiceImpl::new(
        mock_repo,
        mock_pending_repo,
        mock_password_reset_repo,
        mock_notification_service,
        mock_session_invalidation_service,
        ttl,
        reset_ttl,
    );

    // Execute Step 1: Register
    let email = Email::new("endtoend@gmail.com".to_string()).unwrap();
    let password = Password::new("SecurePass123!".to_string()).unwrap();
    let register_cmd = RegisterIdentityCommand::new(email, password, AuthProvider::Email);

    let register_result = service.handle(register_cmd).await;
    assert!(register_result.is_ok());
    let (_identity, token) = register_result.unwrap();

    // Execute Step 2: Confirm with token from registration
    let confirm_cmd = ConfirmRegistrationCommand::new(token);
    let confirm_result = service.confirm_registration(confirm_cmd).await;

    assert!(confirm_result.is_ok());
    let final_identity = confirm_result.unwrap();
    assert_eq!(final_identity.email().value(), "endtoend@gmail.com");
}
