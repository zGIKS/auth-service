use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use uuid::Uuid;
use validator::Validate;

use crate::iam::authentication::{
    infrastructure::{
        persistence::redis::redis_session_repository::RedisSessionRepository,
        services::jwt_token_service::JwtTokenService,
    },
};
use crate::iam::federation::{
    application::services::google_federation_service::GoogleFederationService,
    domain::{
        error::FederationError,
        repositories::token_exchange_repository::{TokenExchangeRepository, ExchangeTokens},
    },
    infrastructure::{
        services::google_oauth_client::GoogleOAuthClient,
        persistence::redis::token_exchange_repository_impl::TokenExchangeRepositoryImpl,
    },
    interfaces::rest::resources::{
        google_callback_query::GoogleCallbackQuery,
        claim_token_resource::{ClaimTokenRequest, ClaimTokenResponse},
    },
};
use crate::iam::identity::infrastructure::persistence::postgres::repositories::identity_repository_impl::IdentityRepositoryImpl;
use crate::shared::interfaces::rest::{app_state::AppState, error_response::ErrorResponse};

#[utoipa::path(
    get,
    path = "/api/v1/auth/google",
    tag = "auth",
    responses((status = 302, description = "Redirect to Google OAuth"))
)]
pub async fn redirect_to_google(
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let csrf_state = Uuid::new_v4().to_string();
    let scope = urlencoding::encode("openid email profile");
    let redirect_uri = urlencoding::encode(&state.google_redirect_uri);

    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent&state={}",
        state.google_client_id, redirect_uri, scope, csrf_state
    );

    let mut cookie = Cookie::new("oauth_state", csrf_state);
    cookie.set_http_only(true);
    cookie.set_secure(true); // Ensure HTTPS is used
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    // Expire in 10 minutes
    cookie.set_max_age(time::Duration::minutes(10));

    (jar.add(cookie), Redirect::to(&url))
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
    jar: CookieJar,
    Query(query): Query<GoogleCallbackQuery>,
) -> impl IntoResponse {
    let frontend_url = match state.frontend_url.as_deref() {
        Some(url) => url,
        None => {
            return ErrorResponse::new("Frontend URL not configured")
                .with_code(StatusCode::INTERNAL_SERVER_ERROR.as_u16())
                .into_response();
        }
    };

    // CSRF Validation
    let cookie_state = jar.get("oauth_state").map(|c| c.value().to_string());
    if query.state.is_none() || cookie_state.is_none() || query.state != cookie_state {
        let redirect_url = format!(
            "{}/login?error=csrf_error&message={}",
            frontend_url,
            urlencoding::encode("Invalid CSRF state")
        );
        return (
            jar.remove(Cookie::from("oauth_state")),
            Redirect::to(&redirect_url),
        )
            .into_response();
    }

    // Clean up cookie
    let jar = jar.remove(Cookie::from("oauth_state"));

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
            // Secure Handoff: Save tokens to Redis and get a one-time code
            let token_exchange_repo = TokenExchangeRepositoryImpl::new(state.redis.clone());
            let exchange_tokens = ExchangeTokens {
                access_token: token.value().to_string(),
                refresh_token: refresh_token.value().to_string(),
            };

            match token_exchange_repo.save(exchange_tokens).await {
                Ok(code) => {
                    // Redirect with the code only
                    let redirect_url = format!(
                        "{}/auth/google/callback?code={}",
                        frontend_url,
                        urlencoding::encode(&code)
                    );
                    (jar, Redirect::to(&redirect_url)).into_response()
                }
                Err(e) => {
                    tracing::error!("Failed to save exchange tokens: {:?}", e);
                    let redirect_url = format!(
                        "{}/login?error=internal_error&message={}",
                        frontend_url,
                        urlencoding::encode("Failed to secure login session")
                    );
                    (jar, Redirect::to(&redirect_url)).into_response()
                }
            }
        }
        Err(err) => {
            let error_msg = match err {
                FederationError::InvalidAuthorizationCode => "Invalid authorization code",
                FederationError::InvalidEmail => "Invalid email returned by Google",
                FederationError::EmailNotVerified => "Google email is not verified",
                FederationError::ProviderMismatch => {
                    "Email already registered with a different provider"
                }
                FederationError::TokenExchange(_) => "Failed to exchange code with Google",
                FederationError::UserInfo(_) => "Failed to retrieve Google user info",
                FederationError::Internal(_) => "Internal error",
            };

            let redirect_url = format!(
                "{}/login?error=google_auth_failed&message={}",
                frontend_url,
                urlencoding::encode(error_msg)
            );
            (jar, Redirect::to(&redirect_url)).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/google/claim",
    tag = "auth",
    request_body = ClaimTokenRequest,
    responses(
        (status = 200, description = "Tokens claimed successfully", body = ClaimTokenResponse),
        (status = 400, description = "Invalid or expired code"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn claim_token(
    State(state): State<AppState>,
    Json(payload): Json<ClaimTokenRequest>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
        return ErrorResponse::new(e.to_string())
            .with_code(StatusCode::BAD_REQUEST.as_u16())
            .into_response();
    }

    let token_exchange_repo = TokenExchangeRepositoryImpl::new(state.redis.clone());

    match token_exchange_repo.claim(payload.code).await {
        Ok(Some(tokens)) => (
            StatusCode::OK,
            Json(ClaimTokenResponse {
                token: tokens.access_token,
                refresh_token: tokens.refresh_token,
            }),
        )
            .into_response(),
        Ok(None) => ErrorResponse::new("Invalid or expired code")
            .with_code(StatusCode::BAD_REQUEST.as_u16())
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to claim exchange token: {:?}", e);
            ErrorResponse::internal_error().into_response()
        }
    }
}
