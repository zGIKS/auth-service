use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshToken {
    value: String,
}

impl RefreshToken {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
