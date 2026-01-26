use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};

use crate::iam::authentication::{
    infrastructure::{
        persistence::redis::redis_session_repository::RedisSessionRepository,
        services::jwt_token_service::JwtTokenService,
    },
    interfaces::rest::resources::signin_resource::TokenResponse,
};
use crate::iam::federation::{
    application::services::google_federation_service::GoogleFederationService,
    domain::error::FederationError,
    infrastructure::services::google_oauth_client::GoogleOAuthClient,
    interfaces::rest::resources::google_callback_query::GoogleCallbackQuery,
};
use crate::iam::identity::infrastructure::persistence::postgres::repositories::identity_repository_impl::IdentityRepositoryImpl;
use crate::shared::interfaces::rest::{app_state::AppState, error_response::ErrorResponse};

#[utoipa::path(
    get,
    path = "/api/v1/auth/google",
    tag = "auth",
    responses((status = 302, description = "Redirect to Google OAuth"))
)]
pub async fn redirect_to_google(State(state): State<AppState>) -> impl IntoResponse {
    let scope = urlencoding::encode("openid email profile");
    let redirect_uri = urlencoding::encode(&state.google_redirect_uri);

    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent",
        state.google_client_id, redirect_uri, scope
    );

    Redirect::to(&url)
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/google/callback",
    tag = "auth",
    params(GoogleCallbackQuery),
    responses(
        (status = 200, description = "Federated login successful", body = TokenResponse),
        (status = 400, description = "Invalid authorization code", body = ErrorResponse),
        (status = 401, description = "Email not verified", body = ErrorResponse),
        (status = 409, description = "Email registered with different provider", body = ErrorResponse),
        (status = 502, description = "Google provider error", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn google_callback(
    State(state): State<AppState>,
    Query(query): Query<GoogleCallbackQuery>,
) -> impl IntoResponse {
    let oauth_client = GoogleOAuthClient::new(
        state.google_client_id.clone(),
        state.google_client_secret.clone(),
        state.google_redirect_uri.clone(),
    );

    let identity_repo = IdentityRepositoryImpl::new(state.db.clone());
    let token_service =
        JwtTokenService::new(state.jwt_secret.clone(), state.session_duration_seconds);
    let session_repo =
        RedisSessionRepository::new(state.redis.clone(), state.session_duration_seconds);

    let service =
        GoogleFederationService::new(identity_repo, token_service, session_repo, oauth_client);

    match service.authenticate(query.code.clone()).await {
        Ok((token, refresh_token)) => (
            StatusCode::OK,
            Json(TokenResponse {
                token: token.value().to_string(),
                refresh_token: refresh_token.value().to_string(),
            }),
        )
            .into_response(),
        Err(err) => map_error(err).into_response(),
    }
}

fn map_error(error: FederationError) -> impl IntoResponse {
    match error {
        FederationError::InvalidAuthorizationCode => {
            ErrorResponse::new("Invalid authorization code")
                .with_code(StatusCode::BAD_REQUEST.as_u16())
                .into_response()
        }
        FederationError::InvalidEmail => ErrorResponse::new("Invalid email returned by Google")
            .with_code(StatusCode::BAD_REQUEST.as_u16())
            .into_response(),
        FederationError::EmailNotVerified => ErrorResponse::new("Google email is not verified")
            .with_code(StatusCode::UNAUTHORIZED.as_u16())
            .into_response(),
        FederationError::ProviderMismatch => {
            ErrorResponse::new("Email already registered with a different provider")
                .with_code(StatusCode::CONFLICT.as_u16())
                .into_response()
        }
        FederationError::TokenExchange(details) => {
            ErrorResponse::new(format!("Failed to exchange code with Google: {}", details))
                .with_code(StatusCode::BAD_GATEWAY.as_u16())
                .into_response()
        }
        FederationError::UserInfo(details) => {
            ErrorResponse::new(format!("Failed to retrieve Google user info: {}", details))
                .with_code(StatusCode::BAD_GATEWAY.as_u16())
                .into_response()
        }
        FederationError::Internal(details) => {
            ErrorResponse::new(format!("Internal error: {}", details))
                .with_code(StatusCode::INTERNAL_SERVER_ERROR.as_u16())
                .into_response()
        }
    }
}
