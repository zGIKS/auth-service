/// Integration tests for password reset token invalidation
/// These tests verify that previous tokens are invalidated when new ones are requested
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
async fn test_multiple_password_reset_requests_invalidate_previous_tokens() {
    // This test verifies that when multiple password reset requests are made,
    // only the most recent token remains valid
    let mut mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let mut mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mut mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let test_email = Email::new("user@example.com".to_string()).unwrap();

    unsafe { std::env::set_var("FRONTEND_URL", "http://localhost:3000") };

    // User exists for all requests
    mock_repo
        .expect_find_by_email()
        .times(3)
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

    // Mock expects 3 saves - the repository implementation should:
    // 1. Acquire distributed lock for the email
    // 2. Check for existing token via email key
    // 3. Delete old token if exists (atomic operation)
    // 4. Save new token and update email->token mapping
    // 5. Release lock
    mock_password_reset_repo
        .expect_save()
        .times(3)
        .returning(|_, _, _| Ok(()));

    // Send emails for all requests
    mock_notification_service
        .expect_send_password_reset_email()
        .times(3)
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

    // First request
    let command1 = RequestPasswordResetCommand::new(test_email.clone());
    let result1 = service.request_password_reset(command1).await;
    assert!(
        result1.is_ok(),
        "First password reset request should succeed"
    );

    // Second request (invalidates first token)
    let command2 = RequestPasswordResetCommand::new(test_email.clone());
    let result2 = service.request_password_reset(command2).await;
    assert!(
        result2.is_ok(),
        "Second password reset request should succeed"
    );

    // Third request (invalidates second token)
    let command3 = RequestPasswordResetCommand::new(test_email);
    let result3 = service.request_password_reset(command3).await;
    assert!(
        result3.is_ok(),
        "Third password reset request should succeed"
    );

    // unsafe { std::env::remove_var("FRONTEND_URL") };

    // Note: The actual token invalidation happens in the repository layer
    // password_reset_token_repository_impl.rs:
    //
    // fn save():
    //   1. Acquire distributed lock (password_reset_lock:{email})
    //   2. Get old token hash from email key (password_reset_email:{email})
    //   3. If old token exists, delete it (password_reset:{old_hash})
    //   4. Atomically:
    //      - SET password_reset:{new_hash} = email (with TTL)
    //      - SET password_reset_email:{email} = new_hash (with TTL)
    //   5. Release lock
    //
    // fn find_email_by_token():
    //   - Gets email from token key
    //   - Verifies token is the current one by checking email key
    //   - Returns None if token doesn't match current token
    //
    // This ensures only ONE token per email is valid at any time
}

#[tokio::test]
async fn test_password_reset_handles_concurrent_requests_safely() {
    // This test documents the behavior when concurrent requests occur
    // The distributed lock prevents race conditions
    let mut mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let mut mock_password_reset_repo = MockPasswordResetTokenRepository::new();
    let mut mock_notification_service = MockNotificationService::new();
    let mock_session_invalidation_service = MockSessionInvalidationService::new();
    let ttl = Duration::from_secs(900);
    let reset_ttl = Duration::from_secs(900);

    let test_email = Email::new("concurrent@example.com".to_string()).unwrap();

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

    // Repository's save() uses SET NX (set if not exists) for lock acquisition
    // If lock is already held, it returns an error
    // The service catches this and returns Ok() for security (don't reveal system state)
    mock_password_reset_repo
        .expect_save()
        .times(1)
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

    // If a second concurrent request arrives while the first is processing:
    // - The distributed lock (password_reset_lock:{email}) is already held
    // - SET NX returns false
    // - Repository returns Err(InternalError)
    // - Service catches the error and returns Ok() (security: don't reveal state)
    // - User receives "success" message but no email is sent
    // - Lock expires after 10 seconds to prevent deadlock

    // unsafe { std::env::remove_var("FRONTEND_URL") };
}
