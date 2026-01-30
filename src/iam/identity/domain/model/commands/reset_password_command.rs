use crate::iam::identity::domain::model::value_objects::password::Password;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct ResetPasswordCommand {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
    #[validate(nested)]
    pub new_password: Password,
}

impl ResetPasswordCommand {
    pub fn new(token: String, new_password: Password) -> Self {
        Self {
            token,
            new_password,
        }
    }
}
