use crate::iam::identity::domain::model::value_objects::email::Email;
use crate::iam::identity::domain::repositories::identity_repository::IdentityRepository;
use crate::iam::identity::interfaces::acl::identity_facade::IdentityFacade;
use bcrypt::verify;
use std::error::Error;
use uuid::Uuid;

pub struct IdentityFacadeImpl<R: IdentityRepository> {
    repository: R,
}

impl<R: IdentityRepository> IdentityFacadeImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait::async_trait]
impl<R: IdentityRepository> IdentityFacade for IdentityFacadeImpl<R> {
    async fn verify_credentials(
        &self,
        email: String,
        password: String,
    ) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>> {
        let email_vo = match Email::new(email) {
            Ok(e) => e,
            Err(_) => return Ok(None),
        };

        match self.repository.find_by_email(&email_vo).await {
            Ok(Some(identity)) => {
                let valid = verify(&password, identity.password().value())?;
                if valid {
                    Ok(Some(identity.id().value()))
                } else {
                    Ok(None)
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn user_exists(&self, email: String) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let email_vo = match Email::new(email) {
            Ok(e) => e,
            Err(_) => return Ok(false),
        };

        match self.repository.find_by_email(&email_vo).await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }
}
