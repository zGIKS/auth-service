/// Tests for email confirmation flow
use super::test_mocks::*;
use auth_service::iam::identity::application::command_services::identity_command_service_impl::IdentityCommandServiceImpl;
use auth_service::iam::identity::domain::error::DomainError;
use auth_service::iam::identity::domain::model::aggregates::identity::Identity;
use auth_service::iam::identity::domain::model::commands::confirm_registration_command::ConfirmRegistrationCommand;
use auth_service::iam::identity::domain::model::commands::register_identity_command::RegisterIdentityCommand;
use auth_service::iam::identity::domain::model::queries::confirm_email_query::ConfirmEmailQuery;
use auth_service::iam::identity::domain::model::value_objects::{
    auth_provider::AuthProvider, email::Email, password::Password,
    pending_identity::PendingIdentity,
};
use auth_service::iam::identity::domain::services::identity_command_service::IdentityCommandService;
use std::time::Duration;

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

    mock_pending_repo.expect_find().times(1).returning(|_| {
        let pending = PendingIdentity {
            email: "test@gmail.com".to_string(),
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
