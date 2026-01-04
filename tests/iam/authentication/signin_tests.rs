use auth_service::iam::authentication::application::command_services::authentication_command_service_impl::AuthenticationCommandServiceImpl;
use auth_service::iam::authentication::domain::model::commands::signin_command::SigninCommand;
use auth_service::iam::authentication::domain::model::value_objects::token::Token;
use auth_service::iam::authentication::domain::services::authentication_command_service::{AuthenticationCommandService, SessionRepository, TokenService};
use auth_service::iam::identity::interfaces::acl::identity_facade::IdentityFacade;
use mockall::mock;
use uuid::Uuid;
use std::error::Error;

// Mock IdentityFacade
// We use a shim because the trait is async and mockall has limits with async_trait direct mocking sometimes,
// or we want explicit control over the signatures.
mock! {
    pub IdentityFacadeShim {
        fn verify_credentials(&self, email: String, password: String) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>>;
    }
}

#[async_trait::async_trait]
impl IdentityFacade for MockIdentityFacadeShim {
    async fn verify_credentials(&self, email: String, password: String) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>> {
        self.verify_credentials(email, password)
    }
}

// Mock TokenService
// TokenService is synchronous, so we can mock it directly or via shim. 
// Using shim for consistency here or just a different name to avoid collision if any.
mock! {
    pub TokenServiceShim {}
    
    impl TokenService for TokenServiceShim {
        fn generate_token(&self, user_id: Uuid) -> Result<Token, Box<dyn Error + Send + Sync>>;
    }
}

// Mock SessionRepository
mock! {
    pub SessionRepositoryShim {
        fn create_session(&self, user_id: Uuid, token: Token) -> Result<(), Box<dyn Error + Send + Sync>>;
    }
}

#[async_trait::async_trait]
impl SessionRepository for MockSessionRepositoryShim {
    async fn create_session(&self, user_id: Uuid, token: &Token) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.create_session(user_id, token.clone())
    }
}

#[tokio::test]
async fn test_signin_success() {
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mut mock_token_service = MockTokenServiceShim::new();
    let mut mock_session_repository = MockSessionRepositoryShim::new();

    let user_id = Uuid::new_v4();
    let email = "test@example.com".to_string();
    let password = "password123".to_string();
    let token_string = "generated_token_123".to_string();
    let token = Token::new(token_string.clone());

    // Setup IdentityFacade mock/home/giks/Documents/IAM-service/auth-service-main/tests/iam/authentication/signin_tests.rs
    mock_identity_facade
        .expect_verify_credentials()
        .with(mockall::predicate::eq(email.clone()), mockall::predicate::eq(password.clone()))
        .returning(move |_, _| Ok(Some(user_id)));

    // Setup TokenService mock
    let token_clone = token.clone();
    mock_token_service
        .expect_generate_token()
        .with(mockall::predicate::eq(user_id))
        .returning(move |_| Ok(token_clone.clone()));

    // Setup SessionRepository mock
    let token_clone_2 = token.clone();
    mock_session_repository
        .expect_create_session()
        .withf(move |uid: &Uuid, t: &Token| *uid == user_id && t.value() == token_clone_2.value())
        .returning(|_, _| Ok(()));

    let service = AuthenticationCommandServiceImpl::new(
        mock_identity_facade,
        mock_token_service,
        mock_session_repository
    );

    let command = SigninCommand::new(email, password);
    let result: Result<Token, Box<dyn Error + Send + Sync>> = service.signin(command).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().value(), token_string);
}

#[tokio::test]
async fn test_signin_invalid_credentials() {
    let mut mock_identity_facade = MockIdentityFacadeShim::new();
    let mock_token_service = MockTokenServiceShim::new();
    let mock_session_repository = MockSessionRepositoryShim::new();

    let email = "wrong@example.com".to_string();
    let password = "wrongpassword".to_string();

    // Setup IdentityFacade mock to return None
    mock_identity_facade
        .expect_verify_credentials()
        .returning(|_, _| Ok(None));

    let service = AuthenticationCommandServiceImpl::new(
        mock_identity_facade,
        mock_token_service,
        mock_session_repository
    );

    let command = SigninCommand::new(email, password);
    let result: Result<Token, Box<dyn Error + Send + Sync>> = service.signin(command).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.to_string(), "Invalid credentials");
}
