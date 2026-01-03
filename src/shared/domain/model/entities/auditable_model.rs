use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditableModel {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AuditableModel {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl Default for AuditableModel {
    fn default() -> Self {
        Self::new()
    }
}
