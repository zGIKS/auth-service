use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Password {
    value: String,
}

impl Password {
    pub fn new(value: String) -> Self {
        // TODO: Add complexity validation here
        Self { value }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
