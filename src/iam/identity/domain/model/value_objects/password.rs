use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Password {
    value: String,
}

impl Password {
    pub fn new(value: String) -> Result<Self, String> {
        if value.len() < 12 || value.len() > 72 {
            return Err("Password must be between 12 and 72 characters".to_string());
        }
        Ok(Self { value })
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
