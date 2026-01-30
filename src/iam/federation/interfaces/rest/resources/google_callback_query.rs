use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct GoogleCallbackQuery {
    /// Authorization code returned by Google OAuth
    #[param(example = "authorization_code_from_google")]
    pub code: String,
    /// Optional state parameter sent during authorization
    #[param(example = "optional-state")]
    pub state: Option<String>,
}
