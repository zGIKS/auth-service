use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct Email {
    #[validate(email)]
    value: String,
}

impl Email {
    pub fn new(value: String) -> Result<Self, validator::ValidationErrors> {
        let email = Self { value };
        email.validate()?;
        Ok(email)
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
