use redis::Client;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: redis::Client,
    pub jwt_secret: String,
    pub session_duration_seconds: u64,
    pub refresh_token_duration_seconds: u64,
    pub pending_registration_ttl_seconds: u64,
    pub password_reset_ttl_seconds: u64,
    pub frontend_url: Option<String>,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
}

impl axum::extract::FromRef<AppState> for DatabaseConnection {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl axum::extract::FromRef<AppState> for Client {
    fn from_ref(state: &AppState) -> Self {
        state.redis.clone()
    }
}
