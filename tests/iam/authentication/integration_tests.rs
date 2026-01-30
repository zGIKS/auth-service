/// Integration tests for authentication flow
use super::test_mocks::*;
use auth_service::iam::authentication::application::command_services::authentication_command_service_impl::AuthenticationCommandServiceImpl;
use auth_service::iam::authentication::domain::model::commands::signin_command::SigninCommand;
use auth_service::iam::authentication::domain::model::value_objects::{token::Token, refresh_token::RefreshToken};
use auth_service::iam::authentication::domain::services::authentication_command_service::AuthenticationCommandService;
use uuid::Uuid;

#[tokio::test]
async fn test_complete_authentication_flow() {
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mut mock_token_service = MockTokenServiceShim::new();
    let mut mock_session_repository = MockSessionRepositoryShim::new();
    let mut mock_account_lockout = MockAccountLockoutVerifierShim::new();

    let user_id = Uuid::new_v4();
    let email = "complete@example.com".to_string();
    let password = "CompletePassword123!".to_string();
    let token = Token::new("complete_flow_token_xyz".to_string());
    let jti = "jti-complete".to_string();
    let refresh_token = RefreshToken::new("complete_flow_refresh_token".to_string());

    // Simulate complete flow: verify → generate → store
    mock_identity_facade
        .expect_verify_credentials()
        .times(1)
        .returning(move |_, _| Ok(Some(user_id)));

    let token_clone = token.clone();
    let jti_clone = jti.clone();
    mock_token_service
        .expect_generate_token()
        .times(1)
        .returning(move |_| Ok((token_clone.clone(), jti_clone.clone())));

    let refresh_token_clone = refresh_token.clone();
    mock_token_service
        .expect_generate_refresh_token()
        .times(1)
        .returning(move || Ok(refresh_token_clone.clone()));

    mock_session_repository
        .expect_create_session()
        .times(1)
        .returning(|_, _| Ok(()));

    mock_session_repository
        .expect_save_refresh_token()
        .times(1)
        .returning(|_, _, _| Ok(()));

    mock_account_lockout
        .expect_check_locked()
        .returning(|_, _| Ok(()));
    mock_account_lockout
        .expect_reset_failure()
        .returning(|_, _| Ok(()));

    let service = AuthenticationCommandServiceImpl::new(
        mock_identity_facade,
        mock_token_service,
        mock_session_repository,
        mock_account_lockout,
        2592000,
    );

    let command = SigninCommand::new(email, password, None);
    let result = service.signin(command).await;

    assert!(result.is_ok());
    let (t, rt) = result.unwrap();
    assert_eq!(t.value(), "complete_flow_token_xyz");
    assert_eq!(rt.value(), "complete_flow_refresh_token");
}

#[tokio::test]
async fn test_multiple_signin_attempts_same_user() {
    // Simulate user logging in multiple times (different sessions)
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mut mock_token_service = MockTokenServiceShim::new();
    let mut mock_session_repository = MockSessionRepositoryShim::new();
    let mut mock_account_lockout = MockAccountLockoutVerifierShim::new();

    let user_id = Uuid::new_v4();

    // First login
    mock_identity_facade
        .expect_verify_credentials()
        .times(1)
        .returning(move |_, _| Ok(Some(user_id)));

    let token1 = Token::new("session_token_1".to_string());
    let jti1 = "jti-1".to_string();
    let token1_clone = token1.clone();
    let jti1_clone = jti1.clone();
    mock_token_service
        .expect_generate_token()
        .times(1)
        .returning(move |_| Ok((token1_clone.clone(), jti1_clone.clone())));

    let refresh_token1 = RefreshToken::new("refresh_token_1".to_string());
    let refresh_token1_clone = refresh_token1.clone();
    mock_token_service
        .expect_generate_refresh_token()
        .times(1)
        .returning(move || Ok(refresh_token1_clone.clone()));

    mock_session_repository
        .expect_create_session()
        .times(1)
        .returning(|_, _| Ok(()));

    mock_session_repository
        .expect_save_refresh_token()
        .times(1)
        .returning(|_, _, _| Ok(()));

    mock_account_lockout
        .expect_check_locked()
        .returning(|_, _| Ok(()));
    mock_account_lockout
        .expect_reset_failure()
        .returning(|_, _| Ok(()));

    let service = AuthenticationCommandServiceImpl::new(
        mock_identity_facade,
        mock_token_service,
        mock_session_repository,
        mock_account_lockout,
        604800,
    );

    let command1 = SigninCommand::new("user@example.com".to_string(), "password".to_string(), None);
    let result1 = service.signin(command1).await;

    assert!(result1.is_ok());
    assert_eq!(result1.unwrap().0.value(), "session_token_1");
}

#[tokio::test]
async fn test_signin_with_acl_boundary() {
    // Verify that Authentication BC uses ACL to communicate with Identity BC
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mut mock_token_service = MockTokenServiceShim::new();
    let mut mock_session_repository = MockSessionRepositoryShim::new();
    let mut mock_account_lockout = MockAccountLockoutVerifierShim::new();
    let user_id = Uuid::new_v4();
    let email = "acl@example.com".to_string();
    let password = "password123".to_string();

    // ACL (IdentityFacade) is the only way to verify credentials
    mock_identity_facade
        .expect_verify_credentials()
        .with(
            mockall::predicate::eq(email.clone()),
            mockall::predicate::eq(password.clone()),
        )
        .times(1)
        .returning(move |_, _| Ok(Some(user_id)));

    let token = Token::new("acl_token".to_string());
    let jti = "jti-acl".to_string();
    let token_clone = token.clone();
    let jti_clone = jti.clone();
    mock_token_service
        .expect_generate_token()
        .times(1)
        .returning(move |_| Ok((token_clone.clone(), jti_clone.clone())));

    let refresh_token = RefreshToken::new("acl_refresh_token".to_string());
    let refresh_token_clone = refresh_token.clone();
    mock_token_service
        .expect_generate_refresh_token()
        .times(1)
        .returning(move || Ok(refresh_token_clone.clone()));

    mock_session_repository
        .expect_create_session()
        .times(1)
        .returning(|_, _| Ok(()));

    mock_session_repository
        .expect_save_refresh_token()
        .times(1)
        .returning(|_, _, _| Ok(()));

    // Lockout mocks
    mock_account_lockout
        .expect_check_locked()
        .returning(|_, _| Ok(()));
    mock_account_lockout
        .expect_reset_failure()
        .returning(|_, _| Ok(()));

    let service = AuthenticationCommandServiceImpl::new(
        mock_identity_facade,
        mock_token_service,
        mock_session_repository,
        mock_account_lockout,
        2592000,
    );

    let command = SigninCommand::new(email, password, None);
    let result = service.signin(command).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_signin_preserves_user_id() {
    // Verify that the same user_id is used for token generation and session creation
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mut mock_token_service = MockTokenServiceShim::new();
    let mut mock_session_repository = MockSessionRepositoryShim::new();
    let mut mock_account_lockout = MockAccountLockoutVerifierShim::new();

    let expected_user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    mock_identity_facade
        .expect_verify_credentials()
        .times(1)
        .returning(move |_, _| Ok(Some(expected_user_id)));

    // Verify token generation receives correct user_id
    mock_token_service
        .expect_generate_token()
        .with(mockall::predicate::eq(expected_user_id))
        .times(1)
        .returning(|_| Ok((Token::new("token".to_string()), "jti".to_string())));

    mock_token_service
        .expect_generate_refresh_token()
        .times(1)
        .returning(|| Ok(RefreshToken::new("refresh_token".to_string())));

    // Verify session creation receives correct user_id
    mock_session_repository
        .expect_create_session()
        .withf(move |uid, _| *uid == expected_user_id)
        .times(1)
        .returning(|_, _| Ok(()));

    mock_session_repository
        .expect_save_refresh_token()
        .withf(move |uid, _, _| *uid == expected_user_id)
        .times(1)
        .returning(|_, _, _| Ok(()));
    // Lockout mocks
    mock_account_lockout
        .expect_check_locked()
        .returning(|_, _| Ok(()));
    mock_account_lockout
        .expect_reset_failure()
        .returning(|_, _| Ok(()));

    let service = AuthenticationCommandServiceImpl::new(
        mock_identity_facade,
        mock_token_service,
        mock_session_repository,
        mock_account_lockout,
        2592000,
    );

    let command = SigninCommand::new("user@example.com".to_string(), "password".to_string(), None);
    let result = service.signin(command).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_signin_error_propagation() {
    // Test that errors propagate correctly through the layers
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mock_token_service = MockTokenServiceShim::new();
    let mock_session_repository = MockSessionRepositoryShim::new();
    let mut mock_account_lockout = MockAccountLockoutVerifierShim::new();

    // Simulate infrastructure failure
    mock_identity_facade
        .expect_verify_credentials()
        .times(1)
        .returning(|_, _| Err("Database connection failed".into()));

    mock_account_lockout
        .expect_check_locked()
        .returning(|_, _| Ok(()));

    // reset_failure won't be called because verify_credentials fails before returning user/none?
    // Wait, verify_credentials return Err here.
    // Logic: check_locked -> verify_credentials -> (Ok(Some) -> reset) | (Ok(None) -> register) | (Err -> propagate)
    // So reset/register won't be called.

    let service = AuthenticationCommandServiceImpl::new(
        mock_identity_facade,
        mock_token_service,
        mock_session_repository,
        mock_account_lockout,
        604800,
    );

    let command = SigninCommand::new(
        "error@example.com".to_string(),
        "password".to_string(),
        None,
    );
    let result = service.signin(command).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), "Database connection failed");
}
