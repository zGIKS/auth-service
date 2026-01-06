use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct Body {
    #[validate(length(min = 1))]
    value: String,
}

impl Body {
    pub fn new(value: String) -> Result<Self, ValidationErrors> {
        let body = Self { value };
        body.validate()?;
        Ok(body)
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
