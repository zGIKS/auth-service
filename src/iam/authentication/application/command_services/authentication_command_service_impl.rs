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
use crate::shared::infrastructure::services::account_lockout::AccountLockoutVerifier;
use std::error::Error;

pub struct AuthenticationCommandServiceImpl<F, T, S, L>
where
    F: IdentityFacade,
    T: TokenService,
    S: SessionRepository,
    L: AccountLockoutVerifier,
{
    identity_facade: F,
    token_service: T,
    session_repository: S,
    account_lockout_service: L,
    refresh_token_duration_seconds: u64,
}

impl<F, T, S, L> AuthenticationCommandServiceImpl<F, T, S, L>
where
    F: IdentityFacade,
    T: TokenService,
    S: SessionRepository,
    L: AccountLockoutVerifier,
{
    pub fn new(
        identity_facade: F,
        token_service: T,
        session_repository: S,
        account_lockout_service: L,
        refresh_token_duration_seconds: u64,
    ) -> Self {
        Self {
            identity_facade,
            token_service,
            session_repository,
            account_lockout_service,
            refresh_token_duration_seconds,
        }
    }
}

#[async_trait::async_trait]
impl<F, T, S, L> AuthenticationCommandService for AuthenticationCommandServiceImpl<F, T, S, L>
where
    F: IdentityFacade,
    T: TokenService,
    S: SessionRepository,
    L: AccountLockoutVerifier,
{
    async fn signin(
        &self,
        command: SigninCommand,
    ) -> Result<(Token, RefreshToken), Box<dyn Error + Send + Sync>> {
        // Check if the account is locked (globally or for this IP)
        if let Err(e) = self.account_lockout_service.check_locked(&command.email, command.ip_address.as_deref()).await {
            return Err(Box::new(e));
        }

        let email = command.email.clone();
        let user_id = self
            .identity_facade
            .verify_credentials(command.email, command.password)
            .await?;

        match user_id {
            Some(uid) => {
                // Reset failure counter on successful login
                self.account_lockout_service.reset_failure(&email, command.ip_address.as_deref()).await?;

                // Generate token and get its JTI
                let (token, jti) = self.token_service.generate_token(uid)?;
                let refresh_token = self.token_service.generate_refresh_token()?;

                // Pass JTI to create_session
                self.session_repository.create_session(uid, &jti).await?;

                self.session_repository
                    .save_refresh_token(
                        uid,
                        &refresh_token,
                        self.refresh_token_duration_seconds,
                    )
                    .await?;

                Ok((token, refresh_token))
            }
            None => {
                // Check if user exists before registering failure to prevent DoS on non-existent accounts
                if self.identity_facade.user_exists(email.clone()).await? {
                    self.account_lockout_service.register_failure(&email, command.ip_address.as_deref(), 5, 300).await?;
                }
                Err("Invalid credentials".into())
            },
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
            .save_refresh_token(
                user_id,
                &new_refresh_token,
                self.refresh_token_duration_seconds,
            )
            .await?;

        Ok((new_token, new_refresh_token))
    }

    async fn logout(
        &self,
        command: LogoutCommand,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let refresh_token = RefreshToken::new(command.refresh_token);
        
        let user_id = self
            .session_repository
            .get_user_by_refresh_token(&refresh_token)
            .await?;

        self.session_repository
            .delete_refresh_token(&refresh_token)
            .await?;

        if let Some(uid) = user_id {
            self.session_repository.delete_session(uid).await?;
        }

        Ok(())
    }
}
