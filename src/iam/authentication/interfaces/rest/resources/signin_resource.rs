use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct SigninResource {
    #[validate(email)]
    #[schema(example = "string")]
    pub email: String,
    #[validate(length(min = 6))]
    #[schema(example = "string")]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct TokenResponse {
    pub token: String,
    pub refresh_token: String,
}
