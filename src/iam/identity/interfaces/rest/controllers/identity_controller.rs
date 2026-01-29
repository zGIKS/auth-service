use crate::shared::interfaces::rest::error_response::ErrorResponse;
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use validator::Validate;

use crate::iam::identity::application::command_services::identity_command_service_impl::IdentityCommandServiceImpl;
use crate::iam::identity::domain::services::identity_command_service::IdentityCommandService;
use crate::iam::identity::domain::model::{
    commands::{
        register_identity_command::RegisterIdentityCommand,
        confirm_registration_command::ConfirmRegistrationCommand,
        request_password_reset_command::RequestPasswordResetCommand,
        reset_password_command::ResetPasswordCommand,
    },
    queries::confirm_email_query::ConfirmEmailQuery,
    value_objects::{
        email::Email, password::Password, auth_provider::AuthProvider
    },
};
use crate::iam::identity::domain::error::DomainError;
use crate::iam::identity::interfaces::rest::resources::register_identity_resource::{
    RegisterIdentityRequest, RegisterIdentityResponse
};
use crate::iam::identity::interfaces::rest::resources::confirm_email_resource::ConfirmEmailQueryParams;
use crate::iam::identity::interfaces::rest::resources::request_password_reset_resource::{
    RequestPasswordResetRequest, RequestPasswordResetResponse
};
use crate::iam::identity::interfaces::rest::resources::reset_password_resource::{
    ResetPasswordRequest, ResetPasswordResponse
};
use crate::iam::identity::infrastructure::persistence::postgres::repositories::identity_repository_impl::IdentityRepositoryImpl;
use crate::iam::identity::infrastructure::persistence::redis::pending_identity_repository_impl::PendingIdentityRepositoryImpl;
use crate::iam::identity::infrastructure::persistence::redis::password_reset_token_repository_impl::PasswordResetTokenRepositoryImpl;
use crate::iam::authentication::infrastructure::persistence::redis::redis_session_repository::RedisSessionRepository;
use crate::iam::authentication::interfaces::acl::session_invalidation_service_impl::SessionInvalidationServiceImpl;
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
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("Invalid email: {}", e)).into_response();
        }
    };

    let password = match Password::new(payload.password) {
        Ok(p) => p,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    // Default values: Provider = Email
    let provider = AuthProvider::Email;

    let command = RegisterIdentityCommand::new(email, password, provider);

    let identity_repo = IdentityRepositoryImpl::new(state.db);
    let pending_repo = PendingIdentityRepositoryImpl::new(state.redis.clone());
    let password_reset_repo = PasswordResetTokenRepositoryImpl::new(state.redis.clone());

    // Messaging / Email Service Construction
    let smtp_sender = match SmtpEmailSender::new(state.circuit_breaker.clone()) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to initialize email sender: {}", e);
            return ErrorResponse::service_unavailable()
                .with_code(503)
                .into_response();
        }
    };
    let messaging_service = MessagingCommandServiceImpl::new(smtp_sender);
    let messaging_facade = MessagingFacadeImpl::new(messaging_service);
    let email_service = EmailService::new(messaging_facade);

    // Session Invalidation Service
    let session_repo =
        RedisSessionRepository::new(state.redis.clone(), state.session_duration_seconds);
    let session_invalidation_service = SessionInvalidationServiceImpl::new(session_repo);

    let ttl = std::time::Duration::from_secs(state.pending_registration_ttl_seconds);
    let reset_ttl = std::time::Duration::from_secs(state.password_reset_ttl_seconds);
    let service = IdentityCommandServiceImpl::new(
        identity_repo,
        pending_repo,
        password_reset_repo,
        email_service,
        session_invalidation_service,
        ttl,
        reset_ttl,
    );

    match service.handle(command).await {
        Ok((_identity, _token)) => {
            let resource = RegisterIdentityResponse {
                message: "Identity registered successfully. Please check your email to verify your account.".to_string(),
            };
            (StatusCode::CREATED, Json(resource)).into_response()
        }
        Err(e) => match e {
            DomainError::EmailAlreadyExists => ErrorResponse::new("Email already registered")
                .with_code(400)
                .into_response(),
            DomainError::InvalidEmailDomain(_) => ErrorResponse::new("Invalid email domain")
                .with_code(400)
                .into_response(),
            DomainError::InternalError(ref msg) => {
                tracing::error!("Registration error: {}", msg);
                ErrorResponse::internal_error()
                    .with_code(500)
                    .into_response()
            }
            DomainError::InvalidToken => ErrorResponse::new("Invalid token")
                .with_code(400)
                .into_response(),
        },
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/identity/confirm-registration",
    tag = "identity",
    params(ConfirmEmailQueryParams),
    responses(
        (status = 302, description = "Redirect to frontend - email verified successfully"),
        (status = 400, description = "Invalid or expired token - Redirect to frontend with error"),
        (status = 500, description = "Internal Server Error - Redirect to frontend with error")
    )
)]
pub async fn confirm_registration(
    State(state): State<AppState>,
    Query(params): Query<ConfirmEmailQueryParams>,
) -> impl IntoResponse {
    // Validate query params
    if let Err(e) = params.validate() {
        let error_url = format!(
            "{}/email-verification-failed?error=invalid_token&message={}",
            state
                .frontend_url
                .as_deref()
                .unwrap_or("http://localhost:3000"),
            urlencoding::encode(&e.to_string())
        );
        return Redirect::to(&error_url).into_response();
    }

    // Create domain query
    let query = match ConfirmEmailQuery::new(params.token.clone()) {
        Ok(q) => q,
        Err(e) => {
            let error_url = format!(
                "{}/email-verification-failed?error=invalid_token&message={}",
                state
                    .frontend_url
                    .as_deref()
                    .unwrap_or("http://localhost:3000"),
                urlencoding::encode(&e.to_string())
            );
            return Redirect::to(&error_url).into_response();
        }
    };

    // Use existing command for backward compatibility
    let command = ConfirmRegistrationCommand::new(query.token);

    let identity_repo = IdentityRepositoryImpl::new(state.db);
    let pending_repo = PendingIdentityRepositoryImpl::new(state.redis.clone());
    let password_reset_repo = PasswordResetTokenRepositoryImpl::new(state.redis.clone());

    let smtp_sender = match SmtpEmailSender::new(state.circuit_breaker.clone()) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to initialize email sender: {}", e);
            let error_url = format!(
                "{}/email-verification-failed?error=service_unavailable&message={}",
                state
                    .frontend_url
                    .as_deref()
                    .unwrap_or("http://localhost:3000"),
                urlencoding::encode("Service temporarily unavailable")
            );
            return Redirect::to(&error_url).into_response();
        }
    };
    let messaging_service = MessagingCommandServiceImpl::new(smtp_sender);
    let messaging_facade = MessagingFacadeImpl::new(messaging_service);
    let email_service = EmailService::new(messaging_facade);

    // Session Invalidation Service
    let session_repo =
        RedisSessionRepository::new(state.redis.clone(), state.session_duration_seconds);
    let session_invalidation_service = SessionInvalidationServiceImpl::new(session_repo);

    let ttl = std::time::Duration::from_secs(state.pending_registration_ttl_seconds);
    let reset_ttl = std::time::Duration::from_secs(state.password_reset_ttl_seconds);
    let service = IdentityCommandServiceImpl::new(
        identity_repo,
        pending_repo,
        password_reset_repo,
        email_service,
        session_invalidation_service,
        ttl,
        reset_ttl,
    );

    match service.confirm_registration(command).await {
        Ok(_) => {
            let success_url = format!(
                "{}/email-verified?success=true",
                state
                    .frontend_url
                    .as_deref()
                    .unwrap_or("http://localhost:3000")
            );
            Redirect::to(&success_url).into_response()
        }
        Err(e) => {
            let error_msg = match e {
                DomainError::InvalidToken => "Invalid or expired verification token",
                DomainError::InternalError(ref msg) => {
                    tracing::error!("Email verification error: {}", msg);
                    "Verification failed. Please try again or request a new verification email"
                }
                _ => "Verification failed",
            };
            let error_url = format!(
                "{}/email-verification-failed?error=verification_failed&message={}",
                state
                    .frontend_url
                    .as_deref()
                    .unwrap_or("http://localhost:3000"),
                urlencoding::encode(error_msg)
            );
            Redirect::to(&error_url).into_response()
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
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("Invalid email: {}", e)).into_response();
        }
    };

    let command = RequestPasswordResetCommand::new(email);

    let identity_repo = IdentityRepositoryImpl::new(state.db);
    let pending_repo = PendingIdentityRepositoryImpl::new(state.redis.clone());
    let password_reset_repo = PasswordResetTokenRepositoryImpl::new(state.redis.clone());

    let smtp_sender = match SmtpEmailSender::new(state.circuit_breaker.clone()) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to initialize email sender: {}", e);
            return ErrorResponse::service_unavailable()
                .with_code(503)
                .into_response();
        }
    };
    let messaging_service = MessagingCommandServiceImpl::new(smtp_sender);
    let messaging_facade = MessagingFacadeImpl::new(messaging_service);
    let email_service = EmailService::new(messaging_facade);

    // Session Invalidation Service
    let session_repo =
        RedisSessionRepository::new(state.redis.clone(), state.session_duration_seconds);
    let session_invalidation_service = SessionInvalidationServiceImpl::new(session_repo);

    let ttl = std::time::Duration::from_secs(state.pending_registration_ttl_seconds);
    let reset_ttl = std::time::Duration::from_secs(state.password_reset_ttl_seconds);
    let service = IdentityCommandServiceImpl::new(
        identity_repo,
        pending_repo,
        password_reset_repo,
        email_service,
        session_invalidation_service,
        ttl,
        reset_ttl,
    );

    match service.request_password_reset(command).await {
        Ok(_) => {
            let resource = RequestPasswordResetResponse {
                message: "If an account with that email exists, we sent you a password reset link."
                    .to_string(),
            };
            (StatusCode::OK, Json(resource)).into_response()
        }
        Err(e) => {
            tracing::error!("Password reset request error: {}", e);
            // Return generic success message for security (don't reveal if email exists)
            let resource = RequestPasswordResetResponse {
                message: "If an account with that email exists, we sent you a password reset link."
                    .to_string(),
            };
            (StatusCode::OK, Json(resource)).into_response()
        }
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
    let password_reset_repo = PasswordResetTokenRepositoryImpl::new(state.redis.clone());

    let smtp_sender = match SmtpEmailSender::new(state.circuit_breaker.clone()) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to initialize email sender: {}", e);
            return ErrorResponse::service_unavailable()
                .with_code(503)
                .into_response();
        }
    };
    let messaging_service = MessagingCommandServiceImpl::new(smtp_sender);
    let messaging_facade = MessagingFacadeImpl::new(messaging_service);
    let email_service = EmailService::new(messaging_facade);

    // Session Invalidation Service
    let session_repo =
        RedisSessionRepository::new(state.redis.clone(), state.session_duration_seconds);
    let session_invalidation_service = SessionInvalidationServiceImpl::new(session_repo);

    let ttl = std::time::Duration::from_secs(state.pending_registration_ttl_seconds);
    let reset_ttl = std::time::Duration::from_secs(state.password_reset_ttl_seconds);
    let service = IdentityCommandServiceImpl::new(
        identity_repo,
        pending_repo,
        password_reset_repo,
        email_service,
        session_invalidation_service,
        ttl,
        reset_ttl,
    );

    match service.reset_password(command).await {
        Ok(_) => {
            let resource = ResetPasswordResponse {
                message: "Password has been reset successfully.".to_string(),
            };
            (StatusCode::OK, Json(resource)).into_response()
        }
        Err(e) => match e {
            DomainError::InvalidToken => ErrorResponse::new("Invalid or expired reset token")
                .with_code(400)
                .into_response(),
            _ => {
                tracing::error!("Password reset error: {}", e);
                ErrorResponse::internal_error()
                    .with_code(500)
                    .into_response()
            }
        },
    }
}
