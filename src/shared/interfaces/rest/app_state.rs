use sea_orm::DatabaseConnection;
use redis::Client;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: Client,
    pub jwt_secret: String,
}

impl axum::extract::FromRef<AppState> for DatabaseConnection {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}
