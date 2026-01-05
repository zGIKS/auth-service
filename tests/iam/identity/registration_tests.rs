use auth_service::iam::identity::application::command_services::identity_command_service_impl::IdentityCommandServiceImpl;
use auth_service::iam::identity::domain::model::commands::register_identity_command::RegisterIdentityCommand;
use auth_service::iam::identity::domain::model::commands::confirm_registration_command::ConfirmRegistrationCommand;
use auth_service::iam::identity::domain::model::value_objects::{
    auth_provider::AuthProvider, email::Email, password::Password, pending_identity::PendingIdentity,
};
use auth_service::iam::identity::domain::repositories::{
    identity_repository::IdentityRepository, pending_identity_repository::PendingIdentityRepository,
};
use auth_service::iam::identity::domain::services::identity_command_service::IdentityCommandService;
use auth_service::iam::identity::domain::error::DomainError;
use auth_service::iam::identity::domain::model::aggregates::identity::Identity;
use mockall::mock;
use std::error::Error;
use std::future::Future;
use std::time::Duration;
use auth_service::shared::domain::model::entities::auditable_model::AuditableModel;
use auth_service::iam::identity::domain::model::value_objects::identity_id::IdentityId;
use async_trait::async_trait;

// Mock the repository specifically for this test file
mock! {
    pub IdentityRepository {}
    
    impl IdentityRepository for IdentityRepository {
        fn save(&self, identity: Identity) -> impl Future<Output = Result<Identity, Box<dyn Error + Send + Sync>>> + Send;
        fn find_by_email(&self, email: &Email) -> impl Future<Output = Result<Option<Identity>, Box<dyn Error + Send + Sync>>> + Send;
    }
}

mock! {
    pub PendingIdentityRepository {}

    #[async_trait]
    impl PendingIdentityRepository for PendingIdentityRepository {
        async fn save(&self, pending_identity: PendingIdentity, token_hash: String, ttl: Duration) -> Result<(), DomainError>;
        async fn find(&self, token_hash: &str) -> Result<Option<PendingIdentity>, DomainError>;
        async fn delete(&self, token_hash: &str) -> Result<(), DomainError>;
        async fn find_token_by_email(&self, email: &str) -> Result<Option<String>, DomainError>;
    }
}

#[tokio::test]
async fn test_register_identity_success() {
    let mut mock_repo = MockIdentityRepository::new();
    let mut mock_pending_repo = MockPendingIdentityRepository::new();
    let ttl = Duration::from_secs(900);

    mock_repo
        .expect_find_by_email()
        .returning(|_| Box::pin(async { Ok(None) }));

    mock_pending_repo
        .expect_find_token_by_email()
        .times(1)
        .returning(|_| Ok(None));

    mock_pending_repo
        .expect_save()
        .times(1)
        .returning(|_, _, _| Ok(()));

    let service = IdentityCommandServiceImpl::new(mock_repo, mock_pending_repo, ttl);

    let email = Email::new("test@gmail.com".to_string()).unwrap(); 
    let password = Password::new("SecurePass123!".to_string()).unwrap();
    // Default provider is Email, verified is false
    let command = RegisterIdentityCommand::new(email, password, AuthProvider::Email, false);

    let result = service.handle(command).await;
    assert!(result.is_ok());
    let (identity, _token) = result.unwrap();
    assert_eq!(identity.is_verified(), false); 
}

#[tokio::test]
async fn test_register_identity_invalid_mx() {
    let mock_repo = MockIdentityRepository::new(); 
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let ttl = Duration::from_secs(900);
    
    let service = IdentityCommandServiceImpl::new(mock_repo, mock_pending_repo, ttl);

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
async fn test_password_is_hashed_before_saving_pending() {
    let mut mock_repo = MockIdentityRepository::new();
    let mut mock_pending_repo = MockPendingIdentityRepository::new();
    let ttl = Duration::from_secs(900);
    
    let plain_password = "SecretPassword123!";

    mock_repo
        .expect_find_by_email()
        .returning(|_| Box::pin(async { Ok(None) }));

    mock_pending_repo
        .expect_find_token_by_email()
        .times(1)
        .returning(|_| Ok(None));

    // Verify that the password sent to pending repo save is hashed
    mock_pending_repo
        .expect_save()
        .withf(move |pending_identity, _, _| {
            let stored_pass = &pending_identity.password_hash;
            // Bcrypt hash always starts with $2
            stored_pass.starts_with("$2") && stored_pass != plain_password
        })
        .times(1)
        .returning(|_, _, _| Ok(()));

    let service = IdentityCommandServiceImpl::new(mock_repo, mock_pending_repo, ttl);
    let email = Email::new("hash_test@gmail.com".to_string()).unwrap();
    let password = Password::new(plain_password.to_string()).unwrap();
    let command = RegisterIdentityCommand::new(email, password, AuthProvider::Email, false);

    let result = service.handle(command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_identity_overwrites_existing_pending() {
    let mut mock_repo = MockIdentityRepository::new();
    let mut mock_pending_repo = MockPendingIdentityRepository::new();
    let ttl = Duration::from_secs(900);
    
    let old_token_hash = "old_token_hash_123";

    mock_repo
        .expect_find_by_email()
        .returning(|_| Box::pin(async { Ok(None) }));

    // 1. Should check for existing pending
    mock_pending_repo
        .expect_find_token_by_email()
        .times(1)
        .returning(move |_| Ok(Some(old_token_hash.to_string())));

    // 2. Should delete the old one
    mock_pending_repo
        .expect_delete()
        .with(mockall::predicate::eq(old_token_hash))
        .times(1)
        .returning(|_| Ok(()));

    // 3. Should save the new one
    mock_pending_repo
        .expect_save()
        .times(1)
        .returning(|_, _, _| Ok(()));

    let service = IdentityCommandServiceImpl::new(mock_repo, mock_pending_repo, ttl);
    let email = Email::new("overwrite@gmail.com".to_string()).unwrap();
    let password = Password::new("SecurePass123!".to_string()).unwrap();
    let command = RegisterIdentityCommand::new(email, password, AuthProvider::Email, false);

    let result = service.handle(command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_identity_duplicate_email() {
    let mut mock_repo = MockIdentityRepository::new();
    let mock_pending_repo = MockPendingIdentityRepository::new();
    let ttl = Duration::from_secs(900);

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

    let service = IdentityCommandServiceImpl::new(mock_repo, mock_pending_repo, ttl);

    let email = Email::new("duplicate@gmail.com".to_string()).unwrap();
    let password = Password::new("SecurePass123!".to_string()).unwrap();
    let command = RegisterIdentityCommand::new(email, password, AuthProvider::Email, false);

    let result = service.handle(command).await;

    match result {
        Err(DomainError::EmailAlreadyExists) => assert!(true),
        _ => panic!("Expected EmailAlreadyExists error, got {:?}", result),
    }
}

#[tokio::test]
async fn test_confirm_registration_success() {
    let mut mock_repo = MockIdentityRepository::new();
    let mut mock_pending_repo = MockPendingIdentityRepository::new();
    let ttl = Duration::from_secs(900);

    let token_str = "some-uuid-token";
    // In real code we use VerificationToken logic, but here we can just mock what 'find' returns
    // The service computes hash(token) -> finds in pending repo
    
    // We expect a call to find with SOME hash.
    mock_pending_repo
        .expect_find()
        .times(1)
        .returning(|_| {
            let pending = PendingIdentity {
                email: "test@gmail.com".to_string(),
                password_hash: "$2a$12$somehash".to_string(),
                provider: "Email".to_string(),
            };
            Ok(Some(pending))
        });

    // We expect a call to save in IdentityRepository (the final persistence)
    mock_repo
        .expect_save()
        .times(1)
        .returning(|identity| Box::pin(async { Ok(identity) }));

    // We expect a call to delete the pending identity
    mock_pending_repo
        .expect_delete()
        .times(1)
        .returning(|_| Ok(()));

    let service = IdentityCommandServiceImpl::new(mock_repo, mock_pending_repo, ttl);
    
    let command = ConfirmRegistrationCommand {
        token: token_str.to_string(),
    };

    let result = service.confirm_registration(command).await;
    
    assert!(result.is_ok());
    let identity = result.unwrap();
    assert_eq!(identity.email().value(), "test@gmail.com");
    assert!(identity.is_verified());
}

#[tokio::test]
async fn test_confirm_registration_invalid_token() {
    let mock_repo = MockIdentityRepository::new();
    let mut mock_pending_repo = MockPendingIdentityRepository::new();
    let ttl = Duration::from_secs(900);

    let token_str = "invalid-uuid-token";
    
    // Simulate token not found (returns None)
    mock_pending_repo
        .expect_find()
        .times(1)
        .returning(|_| Ok(None));

    let service = IdentityCommandServiceImpl::new(mock_repo, mock_pending_repo, ttl);
    
    let command = ConfirmRegistrationCommand {
        token: token_str.to_string(),
    };

    let result = service.confirm_registration(command).await;
    
    match result {
        Err(DomainError::InvalidToken) => assert!(true),
        _ => panic!("Expected InvalidToken error, got {:?}", result),
    }
}