use auth_service::iam::identity::application::command_services::identity_command_service_impl::IdentityCommandServiceImpl;
use auth_service::iam::identity::domain::model::commands::register_identity_command::RegisterIdentityCommand;
use auth_service::iam::identity::domain::model::value_objects::{auth_provider::AuthProvider, email::Email, password::Password};
use auth_service::iam::identity::domain::repositories::identity_repository::IdentityRepository;
use auth_service::iam::identity::domain::services::identity_command_service::IdentityCommandService;
use auth_service::iam::identity::domain::error::DomainError;
use auth_service::iam::identity::domain::model::aggregates::identity::Identity;
use mockall::mock;
use std::error::Error;
use std::future::Future;
use auth_service::shared::domain::model::entities::auditable_model::AuditableModel;
use auth_service::iam::identity::domain::model::value_objects::identity_id::IdentityId;

// Mock the repository specifically for this test file
mock! {
    pub IdentityRepository {}
    
    impl IdentityRepository for IdentityRepository {
        fn save(&self, identity: Identity) -> impl Future<Output = Result<Identity, Box<dyn Error + Send + Sync>>> + Send;
        fn find_by_email(&self, email: &Email) -> impl Future<Output = Result<Option<Identity>, Box<dyn Error + Send + Sync>>> + Send;
    }
}

#[tokio::test]
async fn test_register_identity_success() {
    let mut mock_repo = MockIdentityRepository::new();

    mock_repo
        .expect_find_by_email()
        .returning(|_| Box::pin(async { Ok(None) }));

    mock_repo
        .expect_save()
        .returning(|identity| Box::pin(async { Ok(identity) }));

    let service = IdentityCommandServiceImpl::new(mock_repo);

    let email = Email::new("test@gmail.com".to_string()).unwrap(); 
    let password = Password::new("SecurePass123!".to_string()).unwrap();
    // Default provider is Email, verified is false
    let command = RegisterIdentityCommand::new(email, password, AuthProvider::Email, false);

    let result = service.handle(command).await;
    assert!(result.is_ok());
    let identity = result.unwrap();
    assert_eq!(identity.is_verified(), false); 
}

#[tokio::test]
async fn test_register_identity_invalid_mx() {
    let mock_repo = MockIdentityRepository::new(); 
    let service = IdentityCommandServiceImpl::new(mock_repo);

    // This domain definitely doesn't exist
    let email = Email::new("user@thisdomaindefinitelydoesnotexist12345.com".to_string()).unwrap(); 
    
    let password = Password::new("SecurePass123!".to_string()).unwrap();
    let command = RegisterIdentityCommand::new(email, password, AuthProvider::Email, false);

    let result = service.handle(command).await;
    
    match result {
        Err(DomainError::InvalidEmailDomain(_)) => assert!(true),
        _ => panic!("Expected InvalidEmailDomain error, got {:?}", result),
    }
}

#[tokio::test]
async fn test_password_is_hashed_before_saving() {
    let mut mock_repo = MockIdentityRepository::new();
    let plain_password = "SecretPassword123!";

    mock_repo
        .expect_find_by_email()
        .returning(|_| Box::pin(async { Ok(None) }));

    // Verificamos que el password que llega al save NO sea el plano
    mock_repo
        .expect_save()
        .withf(move |identity| {
            let stored_pass = identity.password().value();
            // El hash de bcrypt siempre empieza con $2
            stored_pass.starts_with("$2") && stored_pass != plain_password
        })
        .returning(|identity| Box::pin(async { Ok(identity) }));

    let service = IdentityCommandServiceImpl::new(mock_repo);
    let email = Email::new("hash_test@gmail.com".to_string()).unwrap();
    let password = Password::new(plain_password.to_string()).unwrap();
    let command = RegisterIdentityCommand::new(email, password, AuthProvider::Email, false);

    let result = service.handle(command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_identity_duplicate_email() {
    let mut mock_repo = MockIdentityRepository::new();

    // Simulate existing user found
    mock_repo
        .expect_find_by_email()
        .returning(|email| {
            let existing_identity = Identity::new(
                IdentityId::new(),
                email.clone(),
                Password::new("hashed_password_valid_length".to_string()).unwrap(),
                AuthProvider::Email,
                false,
                AuditableModel::new(),
            );
            Box::pin(async move { Ok(Some(existing_identity)) })
        });

    let service = IdentityCommandServiceImpl::new(mock_repo);

    let email = Email::new("duplicate@gmail.com".to_string()).unwrap();
    let password = Password::new("SecurePass123!".to_string()).unwrap();
    let command = RegisterIdentityCommand::new(email, password, AuthProvider::Email, false);

    let result = service.handle(command).await;

    match result {
        Err(DomainError::EmailAlreadyExists) => assert!(true),
        _ => panic!("Expected EmailAlreadyExists error, got {:?}", result),
    }
}