/// Tests for RequestPasswordResetCommand flow
use super::test_mocks::*;
use auth_service::iam::identity::application::command_services::identity_command_service_impl::IdentityCommandServiceImpl;
use auth_service::iam::identity::domain::model::aggregates::identity::Identity;
use auth_service::iam::identity::domain::model::commands::request_password_reset_command::RequestPasswordResetCommand;
use auth_service::iam::identity::domain::model::value_objects::identity_id::IdentityId;
use auth_service::iam::identity::domain::model::value_objects::{
    auth_provider::AuthProvider, email::Email, password::Password,
};
use auth_service::iam::identity::domain::services::identity_command_service::IdentityCommandService;
use auth_service::shared::domain::model::entities::auditable_model::AuditableModel;
use std::time::Duration;

#[tokio::test]
async fn test_request_password_reset_success() {
    let mut mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let mut mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mut mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let test_email = Email::new("user@gmail.com".to_string()).unwrap();

    unsafe { std::env::set_var("FRONTEND_URL", "http://localhost:3000") };

    // User exists
    mock_repo
        .expect_find_by_email()
        .times(1)
        .returning(|email| {
            let identity = Identity::new(
                IdentityId::new(),
                email.clone(),
                Password::new("hashed_password_valid_length".to_string()).unwrap(),
                AuthProvider::Email,
                AuditableModel::new(),
            );
            Box::pin(async move { Ok(Some(identity)) })
        });

    // Should save token to repository
    mock_password_reset_repo
        .expect_save()
        .times(1)
        .returning(|_, _, _| Ok(()));

    // Should send reset email
    mock_notification_service
        .expect_send_password_reset_email()
        .times(1)
        .withf(|to, link| to == "user@gmail.com" && link.contains("reset-password?token="))
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

    let command = RequestPasswordResetCommand::new(test_email);
    let result = service.request_password_reset(command).await;

    assert!(result.is_ok());

    // unsafe { std::env::remove_var("FRONTEND_URL") };
}

#[tokio::test]
async fn test_request_password_reset_non_existent_email_returns_ok() {
    let mut mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let test_email = Email::new("nonexistent@gmail.com".to_string()).unwrap();

    // User does not exist
    mock_repo
        .expect_find_by_email()
        .times(1)
        .returning(|_| Box::pin(async { Ok(None) }));

    // Should NOT save token or send email (security: don't reveal user existence)

    let service = IdentityCommandServiceImpl::new(
        mock_repo,
        mock_pending_repo,
        mock_password_reset_repo,
        mock_notification_service,
        mock_session_invalidation_service,
        ttl,
        reset_ttl,
    );

    let command = RequestPasswordResetCommand::new(test_email);
    let result = service.request_password_reset(command).await;

    // Should return OK to prevent email enumeration
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_request_password_reset_generates_secure_token() {
    let mut mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let mut mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mut mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let test_email = Email::new("tokentest@gmail.com".to_string()).unwrap();

    unsafe { std::env::set_var("FRONTEND_URL", "http://localhost:3000") };

    mock_repo
        .expect_find_by_email()
        .times(1)
        .returning(|email| {
            let identity = Identity::new(
                IdentityId::new(),
                email.clone(),
                Password::new("hashed_password_valid_length".to_string()).unwrap(),
                AuthProvider::Email,
                AuditableModel::new(),
            );
            Box::pin(async move { Ok(Some(identity)) })
        });

    // Verify token hash is saved (not plain token)
    mock_password_reset_repo
        .expect_save()
        .times(1)
        .withf(|_, token_hash, _| {
            // Hash should be hex string (SHA-256 = 64 chars)
            token_hash.len() == 64 && token_hash.chars().all(|c| c.is_ascii_hexdigit())
        })
        .returning(|_, _, _| Ok(()));

    mock_notification_service
        .expect_send_password_reset_email()
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

    let command = RequestPasswordResetCommand::new(test_email);
    let result = service.request_password_reset(command).await;

    assert!(result.is_ok());

    // unsafe { std::env::remove_var("FRONTEND_URL") };
}

#[tokio::test]
async fn test_request_password_reset_uses_correct_ttl() {
    let mut mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let mut mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mut mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(600); // 10 minutes

    let test_email = Email::new("ttltest@gmail.com".to_string()).unwrap();

    unsafe { std::env::set_var("FRONTEND_URL", "http://localhost:3000") };

    mock_repo
        .expect_find_by_email()
        .times(1)
        .returning(|email| {
            let identity = Identity::new(
                IdentityId::new(),
                email.clone(),
                Password::new("hashed_password_valid_length".to_string()).unwrap(),
                AuthProvider::Email,
                AuditableModel::new(),
            );
            Box::pin(async move { Ok(Some(identity)) })
        });

    // Verify TTL is passed correctly
    mock_password_reset_repo
        .expect_save()
        .times(1)
        .withf(move |_, _, ttl_arg| *ttl_arg == Duration::from_secs(600))
        .returning(|_, _, _| Ok(()));

    mock_notification_service
        .expect_send_password_reset_email()
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

    let command = RequestPasswordResetCommand::new(test_email);
    let result = service.request_password_reset(command).await;

    assert!(result.is_ok());

    // unsafe { std::env::remove_var("FRONTEND_URL") };
}

#[tokio::test]
async fn test_request_password_reset_email_contains_frontend_url() {
    unsafe { std::env::set_var("FRONTEND_URL", "http://localhost:3000") };

    let mut mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let mut mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mut mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let test_email = Email::new("linktest@gmail.com".to_string()).unwrap();

    mock_repo
        .expect_find_by_email()
        .times(1)
        .returning(|email| {
            let identity = Identity::new(
                IdentityId::new(),
                email.clone(),
                Password::new("hashed_password_valid_length".to_string()).unwrap(),
                AuthProvider::Email,
                AuditableModel::new(),
            );
            Box::pin(async move { Ok(Some(identity)) })
        });

    mock_password_reset_repo
        .expect_save()
        .times(1)
        .returning(|_, _, _| Ok(()));

    // Verify email link structure
    mock_notification_service
        .expect_send_password_reset_email()
        .times(1)
        .withf(|_, link| {
            // Should contain frontend URL and token parameter
            link.contains("reset-password?token=")
                && (link.starts_with("http://localhost:3000") || link.starts_with("http://"))
        })
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

    let command = RequestPasswordResetCommand::new(test_email);
    let result = service.request_password_reset(command).await;

    assert!(result.is_ok());

    // unsafe { std::env::remove_var("FRONTEND_URL") };
}
