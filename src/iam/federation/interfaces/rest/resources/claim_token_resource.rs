use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct ClaimTokenRequest {
    #[validate(length(min = 1, message = "Code is required"))]
    pub code: String,
}

#[derive(Serialize, ToSchema)]
pub struct ClaimTokenResponse {
    pub token: String,
    pub refresh_token: String,
}
