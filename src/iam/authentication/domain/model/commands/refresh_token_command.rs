use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct RefreshTokenCommand {
    #[validate(length(min = 1))]
    pub refresh_token: String,
}

impl RefreshTokenCommand {
    pub fn new(refresh_token: String) -> Self {
        Self { refresh_token }
    }
}
