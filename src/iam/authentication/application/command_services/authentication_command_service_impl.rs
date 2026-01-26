use crate::iam::authentication::domain::{
    model::{
        commands::{
            logout_command::LogoutCommand, refresh_token_command::RefreshTokenCommand,
            signin_command::SigninCommand,
        },
        value_objects::{refresh_token::RefreshToken, token::Token},
    },
    services::authentication_command_service::{
        AuthenticationCommandService, SessionRepository, TokenService,
    },
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
    async fn signin(
        &self,
        command: SigninCommand,
    ) -> Result<(Token, RefreshToken), Box<dyn Error + Send + Sync>> {
        let user_id = self
            .identity_facade
            .verify_credentials(command.email, command.password)
            .await?;

        match user_id {
            Some(uid) => {
                // Generate token and get its JTI
                let (token, jti) = self.token_service.generate_token(uid)?;
                let refresh_token = self.token_service.generate_refresh_token()?;

                // Pass JTI to create_session
                self.session_repository.create_session(uid, &jti).await?;

                // 30 days = 2592000 seconds. TODO: Configurable
                self.session_repository
                    .save_refresh_token(uid, &refresh_token, 2592000)
                    .await?;

                Ok((token, refresh_token))
            }
            None => Err("Invalid credentials".into()),
        }
    }

    async fn refresh_token(
        &self,
        command: RefreshTokenCommand,
    ) -> Result<(Token, RefreshToken), Box<dyn Error + Send + Sync>> {
        let refresh_token = RefreshToken::new(command.refresh_token);

        let user_id = self
            .session_repository
            .get_user_by_refresh_token(&refresh_token)
            .await?
            .ok_or("Invalid or expired refresh token")?;

        // Rotation: Revoke old token
        self.session_repository
            .delete_refresh_token(&refresh_token)
            .await?;

        // Generate new pair
        let (new_token, new_jti) = self.token_service.generate_token(user_id)?;
        let new_refresh_token = self.token_service.generate_refresh_token()?;

        // Save using JTI
        self.session_repository
            .create_session(user_id, &new_jti)
            .await?;
        self.session_repository
            .save_refresh_token(user_id, &new_refresh_token, 2592000)
            .await?;

        Ok((new_token, new_refresh_token))
    }

    async fn logout(
        &self,
        command: LogoutCommand,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let refresh_token = RefreshToken::new(command.refresh_token);
        
        self.session_repository
            .delete_refresh_token(&refresh_token)
            .await?;

        Ok(())
    }
}
