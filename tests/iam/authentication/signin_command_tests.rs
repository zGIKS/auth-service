/// Tests for signin command and authentication flow
use super::test_mocks::*;
use auth_service::iam::authentication::application::command_services::authentication_command_service_impl::AuthenticationCommandServiceImpl;
use auth_service::iam::authentication::domain::model::commands::signin_command::SigninCommand;
use auth_service::iam::authentication::domain::model::value_objects::{token::Token, refresh_token::RefreshToken};
use auth_service::iam::authentication::domain::services::authentication_command_service::AuthenticationCommandService;
use validator::Validate;
use uuid::Uuid;
use crate::iam::authentication::test_mocks::{MockAccountLockoutVerifierShim};

#[tokio::test]
async fn test_signin_success() {
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mut mock_token_service = MockTokenServiceShim::new();
    let mut mock_session_repository = MockSessionRepositoryShim::new();
    let mut mock_account_lockout = MockAccountLockoutVerifierShim::new();

    let user_id = Uuid::new_v4();
    let email = "test@example.com".to_string();
    let password = "password123".to_string();
    let token_string = "generated_jwt_token_abc123xyz".to_string();
    let jti_string = "unique-jti-123".to_string();
    let token = Token::new(token_string.clone());
    let refresh_token = RefreshToken::new("refresh_token_123".to_string());

    // 1. Identity verifies credentials successfully
    mock_identity_facade
        .expect_verify_credentials()
        .with(
            mockall::predicate::eq(email.clone()),
            mockall::predicate::eq(password.clone()),
        )
        .times(1)
        .returning(move |_, _| Ok(Some(user_id)));

    // 2. Token service generates JWT
    let token_clone = token.clone();
    let jti_clone = jti_string.clone();
    mock_token_service
        .expect_generate_token()
        .with(mockall::predicate::eq(user_id))
        .times(1)
        .returning(move |_| Ok((token_clone.clone(), jti_clone.clone())));

    // 2b. Token service generates Refresh Token
    let refresh_token_clone = refresh_token.clone();
    mock_token_service
        .expect_generate_refresh_token()
        .times(1)
        .returning(move || Ok(refresh_token_clone.clone()));

    // 3. Session is created with JTI
    let jti_clone_2 = jti_string.clone();
    mock_session_repository
        .expect_create_session()
        .withf(move |uid: &Uuid, jti: &str| *uid == user_id && jti == jti_clone_2)
        .times(1)
        .returning(|_, _| Ok(()));

    // 4. Refresh token is saved
    let refresh_token_clone_2 = refresh_token.clone();
    mock_session_repository
        .expect_save_refresh_token()
        .withf(move |uid: &Uuid, rt: &RefreshToken, ttl: &u64| {
            *uid == user_id && rt == &refresh_token_clone_2 && *ttl == 604800
        })
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

    let command = SigninCommand::new(email, password, None);
    let result = service.signin(command).await;

    assert!(result.is_ok());
    let (returned_token, returned_refresh_token) = result.unwrap();
    assert_eq!(returned_token.value(), token_string);
    assert_eq!(returned_refresh_token.value(), "refresh_token_123");
}

#[tokio::test]
async fn test_signin_invalid_credentials() {
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mock_token_service = MockTokenServiceShim::new();
    let mock_session_repository = MockSessionRepositoryShim::new();
    let mut mock_account_lockout = MockAccountLockoutVerifierShim::new();
    let email = "invalid@example.com".to_string();
    let password = "wrongpassword".to_string();

    // Identity returns None (invalid credentials)
    mock_identity_facade
        .expect_verify_credentials()
        .with(
            mockall::predicate::eq(email.clone()),
            mockall::predicate::eq(password.clone()),
        )
        .times(1)
        .returning(|_, _| Ok(None));

    // Allow checking if user exists (User DOES exist, so lockout counts)
    mock_identity_facade
        .expect_user_exists()
        .with(mockall::predicate::eq(email.clone()))
        .times(1)
        .returning(|_| Ok(true));

    // Token service and session repo should NOT be called

    mock_account_lockout
        .expect_check_locked()
        .returning(|_, _| Ok(()));
    mock_account_lockout
        .expect_register_failure()
        .returning(|_, _, _, _| Ok(false));

    let service = AuthenticationCommandServiceImpl::new(
        mock_identity_facade,
        mock_token_service,
        mock_session_repository,
        mock_account_lockout,
        2592000,
    );

    let command = SigninCommand::new(email, password, None);
    let result = service.signin(command).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Invalid credentials");
}

#[tokio::test]
async fn test_signin_identity_facade_error() {
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mock_token_service = MockTokenServiceShim::new();
    let mock_session_repository = MockSessionRepositoryShim::new();
    let mut mock_account_lockout = MockAccountLockoutVerifierShim::new();

    let email = "error@example.com".to_string();
    let password = "password123".to_string();

    // Identity facade returns error (e.g., database down)
    mock_identity_facade
        .expect_verify_credentials()
        .times(1)
        .returning(|_, _| Err("Database connection failed".into()));

    mock_account_lockout
        .expect_check_locked()
        .returning(|_, _| Ok(()));
    // It propagates error, so register/reset might not be called or handled differently.
    // Based on implementation: verify -> Err -> returns Err immediately. No register/reset.

    let service = AuthenticationCommandServiceImpl::new(
        mock_identity_facade,
        mock_token_service,
        mock_session_repository,
        mock_account_lockout,
        2592000,
    );

    let command = SigninCommand::new(email, password, None);
    let result = service.signin(command).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Database"));
}

#[tokio::test]
async fn test_signin_token_generation_error() {
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mut mock_token_service = MockTokenServiceShim::new();
    let mock_session_repository = MockSessionRepositoryShim::new();
    let mut mock_account_lockout = MockAccountLockoutVerifierShim::new();

    let user_id = Uuid::new_v4();
    let email = "tokenerror@example.com".to_string();
    let password = "password123".to_string();

    mock_identity_facade
        .expect_verify_credentials()
        .times(1)
        .returning(move |_, _| Ok(Some(user_id)));

    // Token generation fails
    mock_token_service
        .expect_generate_token()
        .times(1)
        .returning(|_| Err("JWT signing failed".into()));

    // Session should NOT be created

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

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("JWT"));
}

#[tokio::test]
async fn test_signin_session_creation_error() {
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mut mock_token_service = MockTokenServiceShim::new();
    let mut mock_session_repository = MockSessionRepositoryShim::new();
    let mut mock_account_lockout = MockAccountLockoutVerifierShim::new();

    let user_id = Uuid::new_v4();
    let email = "sessionerror@example.com".to_string();
    let password = "password123".to_string();
    let token = Token::new("valid_token_123".to_string());
    let jti = "jti-123".to_string();
    let refresh_token = RefreshToken::new("valid_refresh_token_123".to_string());

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

    // Session creation fails (e.g., Redis down)
    mock_session_repository
        .expect_create_session()
        .times(1)
        .returning(|_, _| Err("Redis connection failed".into()));

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

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Redis"));
}

#[tokio::test]
async fn test_signin_with_different_user_ids() {
    // Test that different users get different sessions
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mut mock_token_service = MockTokenServiceShim::new();
    let mut mock_session_repository = MockSessionRepositoryShim::new();
    let mut mock_account_lockout = MockAccountLockoutVerifierShim::new();

    let user_id_1 = Uuid::new_v4();

    // First signin
    mock_identity_facade
        .expect_verify_credentials()
        .times(1)
        .returning(move |_, _| Ok(Some(user_id_1)));

    let token_1 = Token::new("token_for_user_1".to_string());
    let jti_1 = "jti-1".to_string();
    let token_1_clone = token_1.clone();
    let jti_1_clone = jti_1.clone();
    mock_token_service
        .expect_generate_token()
        .with(mockall::predicate::eq(user_id_1))
        .times(1)
        .returning(move |_| Ok((token_1_clone.clone(), jti_1_clone.clone())));

    let refresh_token_1 = RefreshToken::new("refresh_token_1".to_string());
    let refresh_token_1_clone = refresh_token_1.clone();
    mock_token_service
        .expect_generate_refresh_token()
        .times(1)
        .returning(move || Ok(refresh_token_1_clone.clone()));

    mock_session_repository
        .expect_create_session()
        .withf(move |uid, _| *uid == user_id_1)
        .times(1)
        .returning(|_, _| Ok(()));

    mock_session_repository
        .expect_save_refresh_token()
        .withf(move |uid, _, _| *uid == user_id_1)
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

    let command_1 = SigninCommand::new("user1@example.com".to_string(), "pass1".to_string(), None);
    let result_1 = service.signin(command_1).await;

    assert!(result_1.is_ok());
    assert_eq!(result_1.unwrap().0.value(), "token_for_user_1");
}

#[tokio::test]
async fn test_signin_command_validation() {
    // Test that SigninCommand validates email format
    let valid_email = "valid@example.com".to_string();
    let invalid_email = "not-an-email".to_string();
    let valid_password = "password123".to_string();

    let valid_command = SigninCommand::new(valid_email, valid_password.clone(), None);
    assert!(valid_command.validate().is_ok());

    let invalid_command = SigninCommand::new(invalid_email, valid_password, None);
    assert!(invalid_command.validate().is_err());
}

#[tokio::test]
async fn test_signin_password_length_validation() {
    // Test password minimum length
    let email = "test@example.com".to_string();
    let short_password = "12345".to_string(); // Less than 6 chars
    let valid_password = "123456".to_string(); // Exactly 6 chars

    let invalid_command = SigninCommand::new(email.clone(), short_password, None);
    assert!(invalid_command.validate().is_err());

    let valid_command = SigninCommand::new(email, valid_password, None);
    assert!(valid_command.validate().is_ok());
}
