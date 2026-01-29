use auth_service::shared::infrastructure::services::account_lockout::{AccountLockoutVerifier, LockoutError};

pub struct MockAccountLockoutShim;

impl MockAccountLockoutShim {
    pub fn new() -> Self {
        Self
    }
}

pub struct InnerMockAccountLockout {
    check_locked: Box<dyn Fn(&str) -> Result<(), LockoutError> + Send + Sync>,
    register_failure: Box<dyn Fn(&str, u64, u64) -> Result<bool, LockoutError> + Send + Sync>,
    reset_failure: Box<dyn Fn(&str) -> Result<(), LockoutError> + Send + Sync>,
}
