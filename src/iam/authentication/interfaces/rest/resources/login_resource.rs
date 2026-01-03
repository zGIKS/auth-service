use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

#[derive(Deserialize, Validate, ToSchema)]
pub struct LoginResource {
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
}
