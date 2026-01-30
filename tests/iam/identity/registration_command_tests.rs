/// Tests for registration command and validation
use super::test_mocks::*;
use auth_service::iam::identity::application::command_services::identity_command_service_impl::IdentityCommandServiceImpl;
use auth_service::iam::identity::domain::error::DomainError;
use auth_service::iam::identity::domain::model::aggregates::identity::Identity;
use auth_service::iam::identity::domain::model::commands::register_identity_command::RegisterIdentityCommand;
use auth_service::iam::identity::domain::model::value_objects::identity_id::IdentityId;
use auth_service::iam::identity::domain::model::value_objects::{
    auth_provider::AuthProvider, email::Email, password::Password,
};
use auth_service::iam::identity::domain::services::identity_command_service::IdentityCommandService;
use auth_service::shared::domain::model::entities::auditable_model::AuditableModel;
use std::time::Duration;

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
