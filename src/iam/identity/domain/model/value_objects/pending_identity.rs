use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingIdentity {
    pub email: String,
    pub password_hash: String,
    pub provider: String,
}
