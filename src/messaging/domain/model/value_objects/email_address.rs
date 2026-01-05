use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct EmailAddress {
    #[validate(email)]
    value: String,
}

impl EmailAddress {
    pub fn new(value: String) -> Result<Self, ValidationErrors> {
        let email = Self { value };
        email.validate()?;
        Ok(email)
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
