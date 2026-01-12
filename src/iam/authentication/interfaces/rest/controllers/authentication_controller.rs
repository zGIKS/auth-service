use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use crate::shared::interfaces::rest::app_state::AppState;
use crate::shared::interfaces::rest::error_response::ErrorResponse;
use crate::iam::authentication::{
    domain::model::commands::{signin_command::SigninCommand, refresh_token_command::RefreshTokenCommand},
    infrastructure::{
        services::jwt_token_service::JwtTokenService,
        persistence::redis::redis_session_repository::RedisSessionRepository,
    },
    application::command_services::authentication_command_service_impl::AuthenticationCommandServiceImpl,
    domain::services::authentication_command_service::AuthenticationCommandService,
    interfaces::rest::resources::{signin_resource::{SigninResource, TokenResponse}, refresh_token_resource::RefreshTokenResource},
};
use crate::iam::identity::{
    infrastructure::persistence::postgres::repositories::identity_repository_impl::IdentityRepositoryImpl,
    application::acl::identity_facade_impl::IdentityFacadeImpl,
};
use validator::Validate;

#[utoipa::path(
    post,
    path = "/api/v1/auth/sign-in",
    tag = "auth",
    request_body = SigninResource,
    responses(
        (status = 200, description = "Sign in successful", body = TokenResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn signin(
    State(state): State<AppState>,
    Json(resource): Json<SigninResource>,
) -> impl IntoResponse {
    if let Err(e) = resource.validate() {
        return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
    }

    let identity_repo = IdentityRepositoryImpl::new(state.db.clone());
    let identity_facade = IdentityFacadeImpl::new(identity_repo);
    let token_service = JwtTokenService::new(state.jwt_secret.clone(), state.session_duration_seconds);
    let session_repo = RedisSessionRepository::new(state.redis.clone(), state.session_duration_seconds);
    
    let service = AuthenticationCommandServiceImpl::new(identity_facade, token_service, session_repo);

    let command = SigninCommand::new(resource.email, resource.password);
    
    match service.signin(command).await {
        Ok((token, refresh_token)) => (StatusCode::OK, Json(TokenResponse { 
            token: token.value().to_string(),
            refresh_token: refresh_token.value().to_string()
        })).into_response(),
        Err(e) => {
            tracing::error!("Authentication error: {}", e);
            ErrorResponse::new("Invalid credentials")
                .with_code(401)
                .into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh-token",
    tag = "auth",
    request_body = RefreshTokenResource,
    responses(
        (status = 200, description = "Token refreshed", body = TokenResponse),
        (status = 401, description = "Invalid or expired refresh token"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(resource): Json<RefreshTokenResource>,
) -> impl IntoResponse {
    if let Err(e) = resource.validate() {
        return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
    }

    let identity_repo = IdentityRepositoryImpl::new(state.db.clone());
    let identity_facade = IdentityFacadeImpl::new(identity_repo);
    let token_service = JwtTokenService::new(state.jwt_secret.clone(), state.session_duration_seconds);
    let session_repo = RedisSessionRepository::new(state.redis.clone(), state.session_duration_seconds);
    
    let service = AuthenticationCommandServiceImpl::new(identity_facade, token_service, session_repo);

    let command = RefreshTokenCommand::new(resource.refresh_token);
    
    match service.refresh_token(command).await {
        Ok((token, refresh_token)) => (StatusCode::OK, Json(TokenResponse { 
            token: token.value().to_string(),
            refresh_token: refresh_token.value().to_string()
        })).into_response(),
        Err(e) => {
            tracing::error!("Refresh token error: {}", e);
            ErrorResponse::new("Invalid or expired refresh token")
                .with_code(401)
                .into_response()
        }
    }
}
