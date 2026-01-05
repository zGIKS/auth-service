use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use validator::Validate;

use crate::iam::identity::application::command_services::identity_command_service_impl::IdentityCommandServiceImpl;
use crate::iam::identity::domain::services::identity_command_service::IdentityCommandService;
use crate::iam::identity::domain::model::commands::register_identity_command::RegisterIdentityCommand;
use crate::iam::identity::domain::model::value_objects::{
    email::Email, password::Password, auth_provider::AuthProvider
};
use crate::iam::identity::domain::error::DomainError;
use crate::iam::identity::interfaces::rest::resources::register_identity_resource::{
    RegisterIdentityRequest, RegisterIdentityResponse
};
use crate::iam::identity::infrastructure::persistence::postgres::repositories::identity_repository_impl::IdentityRepositoryImpl;
use crate::iam::identity::infrastructure::persistence::redis::pending_identity_repository_impl::PendingIdentityRepositoryImpl;
use crate::shared::interfaces::rest::app_state::AppState;

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
    let pending_repo = PendingIdentityRepositoryImpl::new(state.redis);
    let ttl = std::time::Duration::from_secs(state.pending_registration_ttl_seconds);
    let service = IdentityCommandServiceImpl::new(identity_repo, pending_repo, ttl);

    match service.handle(command).await {
        Ok((_identity, _token)) => {
            // TODO: Send event to Messaging BC with the token
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
