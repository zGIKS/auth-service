use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};

use crate::iam::authentication::{
    infrastructure::{
        persistence::redis::redis_session_repository::RedisSessionRepository,
        services::jwt_token_service::JwtTokenService,
    },
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
        (status = 302, description = "Redirect to frontend with tokens"),
        (status = 302, description = "Redirect to frontend with error")
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
        state.circuit_breaker.clone(),
    );

    let identity_repo = IdentityRepositoryImpl::new(state.db.clone());
    let token_service =
        JwtTokenService::new(state.jwt_secret.clone(), state.session_duration_seconds);
    let session_repo =
        RedisSessionRepository::new(state.redis.clone(), state.session_duration_seconds);

    let service = GoogleFederationService::new(
        identity_repo,
        token_service,
        session_repo,
        oauth_client,
        state.refresh_token_duration_seconds,
    );

    match service.authenticate(query.code.clone()).await {
        Ok((token, refresh_token)) => {
            let frontend_url = match state.frontend_url.as_deref() {
                Some(url) => url,
                None => {
                    return ErrorResponse::new("Frontend URL not configured")
                        .with_code(StatusCode::INTERNAL_SERVER_ERROR.as_u16())
                        .into_response()
                }
            };

            let redirect_url = format!(
                "{}/auth/google/callback?token={}&refresh_token={}",
                frontend_url,
                urlencoding::encode(token.value()),
                urlencoding::encode(refresh_token.value())
            );
            Redirect::to(&redirect_url).into_response()
        }
        Err(err) => {
            let frontend_url = match state.frontend_url.as_deref() {
                Some(url) => url,
                None => {
                    return ErrorResponse::new("Frontend URL not configured")
                        .with_code(StatusCode::INTERNAL_SERVER_ERROR.as_u16())
                        .into_response()
                }
            };

            let error_msg = match err {
                FederationError::InvalidAuthorizationCode => "Invalid authorization code",
                FederationError::InvalidEmail => "Invalid email returned by Google",
                FederationError::EmailNotVerified => "Google email is not verified",
                FederationError::ProviderMismatch => "Email already registered with a different provider",
                FederationError::TokenExchange(_) => "Failed to exchange code with Google",
                FederationError::UserInfo(_) => "Failed to retrieve Google user info",
                FederationError::Internal(_) => "Internal error",
            };
            
            let redirect_url = format!(
                "{}/login?error=google_auth_failed&message={}",
                frontend_url,
                urlencoding::encode(error_msg)
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}
