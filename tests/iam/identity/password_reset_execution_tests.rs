/// Tests for ResetPasswordCommand execution
use super::test_mocks::*;
use auth_service::iam::identity::application::command_services::identity_command_service_impl::IdentityCommandServiceImpl;
use auth_service::iam::identity::domain::error::DomainError;
use auth_service::iam::identity::domain::model::aggregates::identity::Identity;
use auth_service::iam::identity::domain::model::commands::reset_password_command::ResetPasswordCommand;
use auth_service::iam::identity::domain::model::value_objects::identity_id::IdentityId;
use auth_service::iam::identity::domain::model::value_objects::{
    auth_provider::AuthProvider, password::Password,
};
use auth_service::iam::identity::domain::services::identity_command_service::IdentityCommandService;
use auth_service::shared::domain::model::entities::auditable_model::AuditableModel;
use std::time::Duration;

#[tokio::test]
async fn test_reset_password_success() {
    let mut mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let mut mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mock_notification_service = MockNotificationService::new();
    let mut mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let test_token = "valid-reset-token-12345678901234567890";
    let test_email = "reset@gmail.com";
    let identity_id = IdentityId::new();

    // 1. Token exists and returns email
    mock_password_reset_repo
        .expect_find_email_by_token()
        .times(1)
        .returning(move |_| Ok(Some(test_email.to_string())));

    // 2. Find user by email
    let identity_id_copy = identity_id;
    mock_repo
        .expect_find_by_email()
        .times(1)
        .returning(move |email| {
            let identity = Identity::new(
                identity_id_copy,
                email.clone(),
                Password::new("old_hashed_password_valid_length".to_string()).unwrap(),
                AuthProvider::Email,
                AuditableModel::new(),
            );
            Box::pin(async move { Ok(Some(identity)) })
        });

    // 3. Save updated identity with new password
    mock_repo
        .expect_save()
        .times(1)
        .withf(|identity| {
            // Verify password was changed (should start with $2 for bcrypt)
            identity.password().value().starts_with("$2")
        })
        .returning(|identity| Box::pin(async { Ok(identity) }));

    // 4. Delete token after successful reset
    mock_password_reset_repo
        .expect_delete()
        .times(1)
        .returning(|_| Ok(()));

    // 5. Invalidate sessions
    mock_session_invalidation_service
        .expect_invalidate_all_sessions()
        .times(1)
        .with(mockall::predicate::eq(identity_id.value()))
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

    let new_password = Password::new("NewSecurePass123!".to_string()).unwrap();
    let command = ResetPasswordCommand::new(test_token.to_string(), new_password);
    let result = service.reset_password(command).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_reset_password_invalid_token() {
    let mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let mut mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let invalid_token = "invalid-token-does-not-exist";

    // Token not found
    mock_password_reset_repo
        .expect_find_email_by_token()
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

    let new_password = Password::new("NewSecurePass123!".to_string()).unwrap();
    let command = ResetPasswordCommand::new(invalid_token.to_string(), new_password);
    let result = service.reset_password(command).await;

    match result {
        Err(DomainError::InvalidToken) => {} // Expected
        _ => panic!("Expected InvalidToken error, got {:?}", result),
    }
}

#[tokio::test]
async fn test_reset_password_hashes_new_password() {
    let mut mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let mut mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mock_notification_service = MockNotificationService::new();
    let mut mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let test_token = "hash-test-token-12345678901234567890";
    let test_email = "hashtest@gmail.com";
    let plain_password = "PlainPassword123!";

    mock_password_reset_repo
        .expect_find_email_by_token()
        .times(1)
        .returning(move |_| Ok(Some(test_email.to_string())));

    mock_repo
        .expect_find_by_email()
        .times(1)
        .returning(|email| {
            let identity = Identity::new(
                IdentityId::new(),
                email.clone(),
                Password::new("old_password_hash_valid_length".to_string()).unwrap(),
                AuthProvider::Email,
                AuditableModel::new(),
            );
            Box::pin(async move { Ok(Some(identity)) })
        });

    // Verify that saved password is hashed (bcrypt format)
    mock_repo
        .expect_save()
        .times(1)
        .withf(move |identity| {
            let saved_password = identity.password().value();
            // Bcrypt hash starts with $2 and is NOT the plain password
            saved_password.starts_with("$2") && saved_password != plain_password
        })
        .returning(|identity| Box::pin(async { Ok(identity) }));

    mock_password_reset_repo
        .expect_delete()
        .times(1)
        .returning(|_| Ok(()));

    // Session invalidation should be called
    mock_session_invalidation_service
        .expect_invalidate_all_sessions()
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

    let new_password = Password::new(plain_password.to_string()).unwrap();
    let command = ResetPasswordCommand::new(test_token.to_string(), new_password);
    let result = service.reset_password(command).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_reset_password_deletes_token_after_use() {
    let mut mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let mut mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mock_notification_service = MockNotificationService::new();
    let mut mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let test_token = "delete-token-test-12345678901234567890";
    let test_email = "deletetest@gmail.com";

    mock_password_reset_repo
        .expect_find_email_by_token()
        .times(1)
        .returning(move |_| Ok(Some(test_email.to_string())));

    mock_repo
        .expect_find_by_email()
        .times(1)
        .returning(|email| {
            let identity = Identity::new(
                IdentityId::new(),
                email.clone(),
                Password::new("old_password_valid_length".to_string()).unwrap(),
                AuthProvider::Email,
                AuditableModel::new(),
            );
            Box::pin(async move { Ok(Some(identity)) })
        });

    mock_repo
        .expect_save()
        .times(1)
        .returning(|identity| Box::pin(async { Ok(identity) }));

    // Verify token is deleted (one-time use)
    let test_token_clone = test_token.to_string();
    mock_password_reset_repo
        .expect_delete()
        .times(1)
        .withf(move |token_hash| {
            // Should delete the hashed version
            token_hash.len() == 64 && token_hash.chars().all(|c| c.is_ascii_hexdigit())
        })
        .returning(|_| Ok(()));

    // Session invalidation should be called
    mock_session_invalidation_service
        .expect_invalidate_all_sessions()
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

    let new_password = Password::new("NewPassword123!".to_string()).unwrap();
    let command = ResetPasswordCommand::new(test_token_clone, new_password);
    let result = service.reset_password(command).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_reset_password_user_not_found_for_valid_token() {
    let mut mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let mut mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let test_token = "orphan-token-12345678901234567890";
    let test_email = "deleted@gmail.com";

    // Token exists
    mock_password_reset_repo
        .expect_find_email_by_token()
        .times(1)
        .returning(move |_| Ok(Some(test_email.to_string())));

    // But user was deleted
    mock_repo
        .expect_find_by_email()
        .times(1)
        .returning(|_| Box::pin(async { Ok(None) }));

    let service = IdentityCommandServiceImpl::new(
        mock_repo,
        mock_pending_repo,
        mock_password_reset_repo,
        mock_notification_service,
        mock_session_invalidation_service,
        ttl,
        reset_ttl,
    );

    let new_password = Password::new("NewPassword123!".to_string()).unwrap();
    let command = ResetPasswordCommand::new(test_token.to_string(), new_password);
    let result = service.reset_password(command).await;

    match result {
        Err(DomainError::InternalError(msg)) if msg.contains("Identity not found") => {} // Expected
        _ => panic!(
            "Expected InternalError for missing identity, got {:?}",
            result
        ),
    }
}

#[tokio::test]
async fn test_reset_password_validates_password_requirements() {
    // This test verifies that Password value object validation happens
    // before the service is even called

    let weak_password = "weak";
    let result = Password::new(weak_password.to_string());

    // Password validation should fail
    assert!(result.is_err());
}

#[tokio::test]
async fn test_reset_password_accepts_strong_password() {
    let mut mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let mut mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mock_notification_service = MockNotificationService::new();
    let mut mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let test_token = "strong-pass-token-12345678901234567890";
    let test_email = "strongpass@gmail.com";

    mock_password_reset_repo
        .expect_find_email_by_token()
        .times(1)
        .returning(move |_| Ok(Some(test_email.to_string())));

    mock_repo
        .expect_find_by_email()
        .times(1)
        .returning(|email| {
            let identity = Identity::new(
                IdentityId::new(),
                email.clone(),
                Password::new("old_password_valid_length".to_string()).unwrap(),
                AuthProvider::Email,
                AuditableModel::new(),
            );
            Box::pin(async move { Ok(Some(identity)) })
        });

    mock_repo
        .expect_save()
        .times(1)
        .returning(|identity| Box::pin(async { Ok(identity) }));

    mock_password_reset_repo
        .expect_delete()
        .times(1)
        .returning(|_| Ok(()));

    // Session invalidation should be called
    mock_session_invalidation_service
        .expect_invalidate_all_sessions()
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

    // Strong password with all requirements
    let strong_password = Password::new("V3ry$tr0ng&C0mpl3xP@ssw0rd!".to_string()).unwrap();
    let command = ResetPasswordCommand::new(test_token.to_string(), strong_password);
    let result = service.reset_password(command).await;

    assert!(result.is_ok());
}
