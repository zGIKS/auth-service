use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct Subject {
    #[validate(length(min = 1, max = 255))]
    value: String,
}

impl Subject {
    pub fn new(value: String) -> Result<Self, ValidationErrors> {
        let subject = Self { value };
        subject.validate()?;
        Ok(subject)
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
