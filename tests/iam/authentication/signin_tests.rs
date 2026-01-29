use auth_service::iam::authentication::application::command_services::authentication_command_service_impl::AuthenticationCommandServiceImpl;
use auth_service::iam::authentication::domain::model::commands::signin_command::SigninCommand;
use auth_service::iam::authentication::domain::model::value_objects::{token::Token, refresh_token::RefreshToken};
use auth_service::iam::authentication::domain::services::authentication_command_service::AuthenticationCommandService;
use uuid::Uuid;
use crate::iam::authentication::test_mocks::{MockIdentityFacadeShim, MockTokenServiceShim, MockSessionRepositoryShim, MockAccountLockoutVerifierShim};

#[tokio::test]
async fn test_signin_success() {
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mut mock_token_service = MockTokenServiceShim::new();
    let mut mock_session_repository = MockSessionRepositoryShim::new();
    let mut mock_account_lockout = MockAccountLockoutVerifierShim::new();

    let user_id = Uuid::new_v4();
    let email = "test@example.com".to_string();
    let password = "password123".to_string();
    let token_string = "generated_token_123".to_string();
    let jti_string = "unique-jti-123".to_string();
    let token = Token::new(token_string.clone());
    let refresh_token_string = "generated_refresh_token_123".to_string();
    let refresh_token = RefreshToken::new(refresh_token_string.clone());

    // Setup IdentityFacade mock
    mock_identity_facade
        .expect_verify_credentials()
        .with(
            mockall::predicate::eq(email.clone()),
            mockall::predicate::eq(password.clone()),
        )
        .returning(move |_, _| Ok(Some(user_id)));

    // Setup TokenService mock
    let token_clone = token.clone();
    let jti_clone = jti_string.clone();
    mock_token_service
        .expect_generate_token()
        .with(mockall::predicate::eq(user_id))
        .returning(move |_| Ok((token_clone.clone(), jti_clone.clone())));

    let refresh_token_clone = refresh_token.clone();
    mock_token_service
        .expect_generate_refresh_token()
        .returning(move || Ok(refresh_token_clone.clone()));

    // Setup SessionRepository mock
    let jti_clone_2 = jti_string.clone();
    mock_session_repository
        .expect_create_session()
        .withf(move |uid: &Uuid, jti: &str| *uid == user_id && jti == jti_clone_2)
        .returning(|_, _| Ok(()));

    let refresh_token_clone_2 = refresh_token.clone();
    mock_session_repository
        .expect_save_refresh_token()
        .withf(move |uid: &Uuid, rt: &RefreshToken, ttl: &u64| {
            *uid == user_id && rt.value() == refresh_token_clone_2.value() && *ttl == 2592000
        })
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
    let (res_token, res_refresh_token) = result.unwrap();
    assert_eq!(res_token.value(), token_string);
    assert_eq!(res_refresh_token.value(), refresh_token_string);
}

#[tokio::test]
async fn test_signin_invalid_credentials() {
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mock_token_service = MockTokenServiceShim::new();
    let mock_session_repository = MockSessionRepositoryShim::new();
    let mut mock_account_lockout = MockAccountLockoutVerifierShim::new();

    let email = "wrong@example.com".to_string();
    let password = "wrongpassword".to_string();

    // Setup IdentityFacade mock to return None
    mock_identity_facade
        .expect_verify_credentials()
        .returning(|_, _| Ok(None));

    // Fix: Expect existence check
    mock_identity_facade
        .expect_user_exists()
        .returning(|_| Ok(true));

    // Lockout mocks
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
    let err = result.unwrap_err();
    assert_eq!(err.to_string(), "Invalid credentials");
}
