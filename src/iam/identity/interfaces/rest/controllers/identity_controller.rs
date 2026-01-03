use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::DatabaseConnection;
use validator::Validate;

use crate::iam::identity::application::command_services::identity_command_service_impl::IdentityCommandServiceImpl;
use crate::iam::identity::domain::services::identity_command_service::IdentityCommandService;
use crate::iam::identity::domain::model::commands::register_identity_command::RegisterIdentityCommand;
use crate::iam::identity::domain::model::value_objects::{
    email::Email, password::Password, auth_provider::AuthProvider
};
use crate::iam::identity::interfaces::rest::resources::register_identity_resource::{
    RegisterIdentityRequest, RegisterIdentityResponse
};
use crate::iam::identity::infrastructure::persistence::postgres::repositories::identity_repository_impl::IdentityRepositoryImpl;

#[utoipa::path(
    post,
    path = "/api/v1/auth/sign-up",
    tag = "identity",
    request_body = RegisterIdentityRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterIdentityResponse),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn register_identity(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<RegisterIdentityRequest>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
         return (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)).into_response();
    }

    let email = match Email::new(payload.email) {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Invalid email: {}", e)).into_response(),
    };

    let password = Password::new(payload.password);

    // Default values: Provider = Email, is_verified = false
    let provider = AuthProvider::Email;
    let is_verified = false;

    let command = RegisterIdentityCommand::new(
        email,
        password,
        provider,
        is_verified,
    );

    let repo = IdentityRepositoryImpl::new(db);
    let service = IdentityCommandServiceImpl::new(repo);

    match service.handle(command).await {
        Ok(_identity) => {
            let resource = RegisterIdentityResponse {
                message: "User registered successfully".to_string(),
            };
            (StatusCode::CREATED, Json(resource)).into_response()
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}