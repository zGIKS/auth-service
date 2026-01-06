use crate::iam::identity::domain::model::value_objects::email::Email;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct RequestPasswordResetCommand {
    #[validate(nested)]
    pub email: Email,
}

impl RequestPasswordResetCommand {
    pub fn new(email: Email) -> Self {
        Self { email }
    }
}
