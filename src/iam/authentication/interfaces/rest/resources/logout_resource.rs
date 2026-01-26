use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct LogoutResource {
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}
