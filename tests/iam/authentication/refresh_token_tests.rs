use auth_service::iam::authentication::application::command_services::authentication_command_service_impl::AuthenticationCommandServiceImpl;
use auth_service::iam::authentication::domain::model::commands::refresh_token_command::RefreshTokenCommand;
use auth_service::iam::authentication::domain::model::value_objects::{token::Token, refresh_token::RefreshToken};
use auth_service::iam::authentication::domain::services::authentication_command_service::AuthenticationCommandService;
use uuid::Uuid;
use crate::iam::authentication::test_mocks::{MockIdentityFacadeShim, MockTokenServiceShim, MockSessionRepositoryShim, MockAccountLockoutVerifierShim};

#[tokio::test]
async fn test_refresh_token_success() {
    let mock_identity_facade = MockIdentityFacadeShim::new();
    let mut mock_token_service = MockTokenServiceShim::new();
    let mut mock_session_repository = MockSessionRepositoryShim::new();
    let mock_account_lockout = MockAccountLockoutVerifierShim::new();

    let user_id = Uuid::new_v4();
    let old_refresh_token_str = "old_refresh_token".to_string();
    let old_refresh_token = RefreshToken::new(old_refresh_token_str.clone());

    let new_token_str = "new_access_token".to_string();
    let new_jti_str = "new_jti".to_string();
    let new_token = Token::new(new_token_str.clone());

    let new_refresh_token_str = "new_refresh_token".to_string();
    let new_refresh_token = RefreshToken::new(new_refresh_token_str.clone());

    // Mock: Get User by Refresh Token (Token exists and is valid)
    let old_rt_clone = old_refresh_token.clone();
    mock_session_repository
        .expect_get_user_by_refresh_token()
        .with(mockall::predicate::eq(old_rt_clone))
        .returning(move |_| Ok(Some(user_id)));

    // Mock: Delete old Refresh Token (Rotation)
    let old_rt_clone_2 = old_refresh_token.clone();
    mock_session_repository
        .expect_delete_refresh_token()
        .with(mockall::predicate::eq(old_rt_clone_2))
        .returning(|_| Ok(()));

    // Mock: Generate New Token
    let new_token_clone = new_token.clone();
    let new_jti_clone = new_jti_str.clone();
    mock_token_service
        .expect_generate_token()
        .with(mockall::predicate::eq(user_id))
        .returning(move |_| Ok((new_token_clone.clone(), new_jti_clone.clone())));

    // Mock: Generate New Refresh Token
    let new_refresh_token_clone = new_refresh_token.clone();
    mock_token_service
        .expect_generate_refresh_token()
        .returning(move || Ok(new_refresh_token_clone.clone()));

    // Mock: Save New Session
    let new_jti_clone_2 = new_jti_str.clone();
    mock_session_repository
        .expect_create_session()
        .withf(move |uid: &Uuid, jti: &str| *uid == user_id && jti == new_jti_clone_2)
        .returning(|_, _| Ok(()));

    // Mock: Save New Refresh Token
    let new_refresh_token_clone_2 = new_refresh_token.clone();
    mock_session_repository
        .expect_save_refresh_token()
        .withf(move |uid: &Uuid, rt: &RefreshToken, ttl: &u64| {
            *uid == user_id && rt.value() == new_refresh_token_clone_2.value() && *ttl == 2592000
        })
        .returning(|_, _, _| Ok(()));

    let service = AuthenticationCommandServiceImpl::new(
        mock_identity_facade,
        mock_token_service,
        mock_session_repository,
        mock_account_lockout,
        2592000,
    );

    let command = RefreshTokenCommand::new(old_refresh_token_str);
    let result = service.refresh_token(command).await;

    assert!(result.is_ok());
    let (res_token, res_refresh_token) = result.unwrap();
    assert_eq!(res_token.value(), new_token_str);
    assert_eq!(res_refresh_token.value(), new_refresh_token_str);
}

#[tokio::test]
async fn test_refresh_token_invalid() {
    let mock_identity_facade = MockIdentityFacadeShim::new();
    let mock_token_service = MockTokenServiceShim::new();
    let mock_account_lockout = MockAccountLockoutVerifierShim::new();
    let mut mock_session_repository = MockSessionRepositoryShim::new();

    let invalid_refresh_token_str = "invalid_token".to_string();
    let invalid_refresh_token = RefreshToken::new(invalid_refresh_token_str.clone());

    // Mock: Get User by Refresh Token returns None (Invalid/Expired)
    mock_session_repository
        .expect_get_user_by_refresh_token()
        .with(mockall::predicate::eq(invalid_refresh_token))
        .returning(|_| Ok(None));

    let service = AuthenticationCommandServiceImpl::new(
        mock_identity_facade,
        mock_token_service,
        mock_session_repository,
        mock_account_lockout,
        2592000,
    );

    let command = RefreshTokenCommand::new(invalid_refresh_token_str);
    let result = service.refresh_token(command).await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid or expired refresh token"
    );
}
