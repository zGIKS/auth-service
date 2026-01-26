use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct ConfirmRegistrationCommand {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
}

impl ConfirmRegistrationCommand {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}
