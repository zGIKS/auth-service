use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationToken {
    token: String,
}

impl VerificationToken {
    pub fn new() -> Self {
        Self {
            token: Uuid::new_v4().to_string(),
        }
    }

    pub fn from_string(token: String) -> Self {
        Self { token }
    }

    pub fn value(&self) -> &str {
        &self.token
    }

    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.token.as_bytes());
        hex::encode(hasher.finalize())
    }
}

impl Default for VerificationToken {
    fn default() -> Self {
        Self::new()
    }
}
