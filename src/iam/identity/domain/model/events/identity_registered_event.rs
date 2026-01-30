use crate::iam::identity::domain::model::value_objects::identity_id::IdentityId;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct IdentityRegisteredEvent {
    pub identity_id: IdentityId,
    pub occurred_on: DateTime<Utc>,
}

impl IdentityRegisteredEvent {
    pub fn new(identity_id: IdentityId) -> Self {
        Self {
            identity_id,
            occurred_on: Utc::now(),
        }
    }
}
