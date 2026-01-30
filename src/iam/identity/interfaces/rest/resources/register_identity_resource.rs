use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Serialize, Validate, ToSchema)]
pub struct RegisterIdentityRequest {
    #[validate(email, length(max = 254))]
    pub email: String,
    #[validate(length(
        min = 6,
        max = 72,
        message = "Password must be between 6 and 72 characters"
    ))]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterIdentityResponse {
    pub message: String,
}
