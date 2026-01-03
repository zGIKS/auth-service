use crate::iam::authentication::domain::{
    model::{commands::login_command::LoginCommand, value_objects::token::Token},
    services::authentication_command_service::{AuthenticationCommandService, SessionRepository, TokenService},
};
use crate::iam::identity::interfaces::acl::identity_facade::IdentityFacade;
use std::error::Error;

pub struct AuthenticationCommandServiceImpl<F, T, S>
where
    F: IdentityFacade,
    T: TokenService,
    S: SessionRepository,
{
    identity_facade: F,
    token_service: T,
    session_repository: S,
}

impl<F, T, S> AuthenticationCommandServiceImpl<F, T, S>
where
    F: IdentityFacade,
    T: TokenService,
    S: SessionRepository,
{
    pub fn new(identity_facade: F, token_service: T, session_repository: S) -> Self {
        Self {
            identity_facade,
            token_service,
            session_repository,
        }
    }
}

#[async_trait::async_trait]
impl<F, T, S> AuthenticationCommandService for AuthenticationCommandServiceImpl<F, T, S>
where
    F: IdentityFacade,
    T: TokenService,
    S: SessionRepository,
{
    async fn login(&self, command: LoginCommand) -> Result<Token, Box<dyn Error + Send + Sync>> {
        let user_id = self.identity_facade.verify_credentials(command.email, command.password).await?;
        
        match user_id {
            Some(uid) => {
                let token = self.token_service.generate_token(uid)?;
                self.session_repository.create_session(uid, &token).await?;
                Ok(token)
            },
            None => Err("Invalid credentials".into()),
        }
    }
}
