use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use validator::Validate;

use crate::iam::identity::application::command_services::identity_command_service_impl::IdentityCommandServiceImpl;
use crate::iam::identity::domain::services::identity_command_service::IdentityCommandService;
use crate::iam::identity::domain::model::commands::{
    register_identity_command::RegisterIdentityCommand,
    confirm_registration_command::ConfirmRegistrationCommand,
    request_password_reset_command::RequestPasswordResetCommand,
    reset_password_command::ResetPasswordCommand,
};
use crate::iam::identity::domain::model::value_objects::{
    email::Email, password::Password, auth_provider::AuthProvider
};
use crate::iam::identity::domain::error::DomainError;
use crate::iam::identity::interfaces::rest::resources::register_identity_resource::{
    RegisterIdentityRequest, RegisterIdentityResponse
};
use crate::iam::identity::interfaces::rest::resources::request_password_reset_resource::{
    RequestPasswordResetRequest, RequestPasswordResetResponse
};
use crate::iam::identity::interfaces::rest::resources::reset_password_resource::{
    ResetPasswordRequest, ResetPasswordResponse
};
use crate::iam::identity::infrastructure::persistence::postgres::repositories::identity_repository_impl::IdentityRepositoryImpl;
use crate::iam::identity::infrastructure::persistence::redis::pending_identity_repository_impl::PendingIdentityRepositoryImpl;
use crate::iam::identity::infrastructure::persistence::redis::password_reset_token_repository_impl::PasswordResetTokenRepositoryImpl;
use crate::shared::interfaces::rest::app_state::AppState;
use crate::messaging::infrastructure::services::smtp_email_sender::SmtpEmailSender;
use crate::messaging::application::command_services::messaging_command_service_impl::MessagingCommandServiceImpl;
use crate::messaging::application::acl::messaging_facade_impl::MessagingFacadeImpl;
use crate::iam::identity::application::outbound::acl::email_service::EmailService;

#[utoipa::path(
    post,
    path = "/api/v1/auth/sign-up",
    tag = "identity",
    request_body = RegisterIdentityRequest,
    responses(
        (status = 201, description = "Identity registered successfully", body = RegisterIdentityResponse),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn register_identity(
    State(state): State<AppState>,
    Json(payload): Json<RegisterIdentityRequest>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
         return (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)).into_response();
    }

    let email = match Email::new(payload.email) {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid email: {}", e)).into_response(),
    };

    let password = match Password::new(payload.password) {
        Ok(p) => p,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    // Default values: Provider = Email, is_verified = false
    let provider = AuthProvider::Email;
    let is_verified = false;

    let command = RegisterIdentityCommand::new(
        email,
        password,
        provider,
        is_verified,
    );

    let identity_repo = IdentityRepositoryImpl::new(state.db);
    let pending_repo = PendingIdentityRepositoryImpl::new(state.redis.clone());
    let password_reset_repo = PasswordResetTokenRepositoryImpl::new(state.redis);
    
    // Messaging / Email Service Construction
    let smtp_sender = match SmtpEmailSender::new() {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to initialize email sender: {}", e)).into_response(),
    };
    let messaging_service = MessagingCommandServiceImpl::new(smtp_sender);
    let messaging_facade = MessagingFacadeImpl::new(messaging_service);
    let email_service = EmailService::new(messaging_facade);

    let ttl = std::time::Duration::from_secs(state.pending_registration_ttl_seconds);
    let reset_ttl = std::time::Duration::from_secs(state.password_reset_ttl_seconds);
    let service = IdentityCommandServiceImpl::new(identity_repo, pending_repo, password_reset_repo, email_service, ttl, reset_ttl);

    match service.handle(command).await {
        Ok((_identity, _token)) => {
            let resource = RegisterIdentityResponse {
                message: "Identity registered successfully. Please check your email to verify your account.".to_string(),
            };
            (StatusCode::CREATED, Json(resource)).into_response()
        },
        Err(e) => match e {
            DomainError::EmailAlreadyExists => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            DomainError::InvalidEmailDomain(_) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            DomainError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            DomainError::InvalidToken => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/identity/confirm-registration",
    tag = "identity",
    request_body = ConfirmRegistrationCommand,
    responses(
        (status = 200, description = "Identity verified successfully"),
        (status = 400, description = "Invalid Token or Request"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn confirm_registration(
    State(state): State<AppState>,
    Json(payload): Json<ConfirmRegistrationCommand>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
        return (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)).into_response();
    }

    let identity_repo = IdentityRepositoryImpl::new(state.db);
    let pending_repo = PendingIdentityRepositoryImpl::new(state.redis.clone());
    let password_reset_repo = PasswordResetTokenRepositoryImpl::new(state.redis);

    let smtp_sender = match SmtpEmailSender::new() {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to initialize email sender: {}", e)).into_response(),
    };
    let messaging_service = MessagingCommandServiceImpl::new(smtp_sender);
    let messaging_facade = MessagingFacadeImpl::new(messaging_service);
    let email_service = EmailService::new(messaging_facade);
    
    let ttl = std::time::Duration::from_secs(state.pending_registration_ttl_seconds);
    let reset_ttl = std::time::Duration::from_secs(state.password_reset_ttl_seconds);
    let service = IdentityCommandServiceImpl::new(identity_repo, pending_repo, password_reset_repo, email_service, ttl, reset_ttl);

    match service.confirm_registration(payload).await {
        Ok(_) => (StatusCode::OK, "Account verified successfully. You can now sign in.").into_response(),
        Err(e) => match e {
             DomainError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid or expired verification token.").into_response(),
             DomainError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
             _ => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/identity/forgot-password",
    tag = "identity",
    request_body = RequestPasswordResetRequest,
    responses(
        (status = 200, description = "Password reset email sent if account exists", body = RequestPasswordResetResponse),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn request_password_reset(
    State(state): State<AppState>,
    Json(payload): Json<RequestPasswordResetRequest>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
         return (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)).into_response();
    }

    let email = match Email::new(payload.email) {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid email: {}", e)).into_response(),
    };

    let command = RequestPasswordResetCommand::new(email);

    let identity_repo = IdentityRepositoryImpl::new(state.db);
    let pending_repo = PendingIdentityRepositoryImpl::new(state.redis.clone());
    let password_reset_repo = PasswordResetTokenRepositoryImpl::new(state.redis);
    
    let smtp_sender = match SmtpEmailSender::new() {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to initialize email sender: {}", e)).into_response(),
    };
    let messaging_service = MessagingCommandServiceImpl::new(smtp_sender);
    let messaging_facade = MessagingFacadeImpl::new(messaging_service);
    let email_service = EmailService::new(messaging_facade);

    let ttl = std::time::Duration::from_secs(state.pending_registration_ttl_seconds);
    let reset_ttl = std::time::Duration::from_secs(state.password_reset_ttl_seconds);
    let service = IdentityCommandServiceImpl::new(identity_repo, pending_repo, password_reset_repo, email_service, ttl, reset_ttl);

    match service.request_password_reset(command).await {
        Ok(_) => {
            let resource = RequestPasswordResetResponse {
                message: "If an account with that email exists, we sent you a password reset link.".to_string(),
            };
            (StatusCode::OK, Json(resource)).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/identity/reset-password",
    tag = "identity",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successfully", body = ResetPasswordResponse),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
         return (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)).into_response();
    }

    let new_password = match Password::new(payload.new_password) {
        Ok(p) => p,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    let command = ResetPasswordCommand::new(payload.token, new_password);

    let identity_repo = IdentityRepositoryImpl::new(state.db);
    let pending_repo = PendingIdentityRepositoryImpl::new(state.redis.clone());
    let password_reset_repo = PasswordResetTokenRepositoryImpl::new(state.redis);
    
    let smtp_sender = match SmtpEmailSender::new() {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to initialize email sender: {}", e)).into_response(),
    };
    let messaging_service = MessagingCommandServiceImpl::new(smtp_sender);
    let messaging_facade = MessagingFacadeImpl::new(messaging_service);
    let email_service = EmailService::new(messaging_facade);

    let ttl = std::time::Duration::from_secs(state.pending_registration_ttl_seconds);
    let reset_ttl = std::time::Duration::from_secs(state.password_reset_ttl_seconds);
    let service = IdentityCommandServiceImpl::new(identity_repo, pending_repo, password_reset_repo, email_service, ttl, reset_ttl);

    match service.reset_password(command).await {
        Ok(_) => {
             let resource = ResetPasswordResponse {
                message: "Password has been reset successfully.".to_string(),
            };
            (StatusCode::OK, Json(resource)).into_response()
        },
        Err(e) => match e {
             DomainError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid or expired reset token.".to_string()).into_response(),
             _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}
