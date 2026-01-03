use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Validate, ToSchema)]
pub struct RegisterIdentityRequest {
    #[validate(email, length(max = 254))]
    pub email: String,
    #[validate(length(min = 12, max = 72, message = "Password must be between 12 and 72 characters"))]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterIdentityResponse {
    pub message: String,
}
