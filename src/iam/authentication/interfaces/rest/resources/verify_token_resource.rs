use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, IntoParams, Validate, ToSchema)]
pub struct VerifyTokenResource {
    #[validate(length(min = 1))]
    #[schema(example = "eyJhbGciOiJIUzI1Ni...")]
    pub token: String,
}

#[derive(Serialize, ToSchema)]
pub struct VerifyTokenResponse {
    pub is_valid: bool,
    pub sub: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
