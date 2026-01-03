use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Validate, ToSchema)]
pub struct RegisterIdentityRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterIdentityResponse {
    pub message: String,
}
