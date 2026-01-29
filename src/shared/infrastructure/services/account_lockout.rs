use redis::{Client, AsyncCommands, RedisError};
use async_trait::async_trait;

#[async_trait]
pub trait AccountLockoutVerifier: Send + Sync {
    async fn check_locked(&self, identity: &str) -> Result<(), LockoutError>;
    async fn register_failure(&self, identity: &str, threshold: u64, lock_duration_sec: u64) -> Result<bool, LockoutError>;
    async fn reset_failure(&self, identity: &str) -> Result<(), LockoutError>;
}

#[derive(Clone)]
pub struct AccountLockoutService {
    client: Client,
}

#[derive(Debug, thiserror::Error)]
pub enum LockoutError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    #[error("Account is locked. Try again later.")]
    Locked(u64), // TTL remaining
}

#[async_trait]
impl AccountLockoutVerifier for AccountLockoutService {
    /// Checks if the identity is locked out.
    async fn check_locked(&self, identity: &str) -> Result<(), LockoutError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let lock_key = format!("lockout:{}", identity);

        let ttl: i64 = conn.ttl(&lock_key).await?;

        if ttl > 0 {
            return Err(LockoutError::Locked(ttl as u64));
        }

        Ok(())
    }

    /// Registers a failed attempt. 
    /// If attempts exceed threshold, locks the account.
    /// Returns true if the account is now locked.
    async fn register_failure(&self, identity: &str, threshold: u64, lock_duration_sec: u64) -> Result<bool, LockoutError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        
        // This key tracks the number of failures
        let attempts_key = format!("login_failures:{}", identity);
        // This key indicates the lockout status
        let lock_key = format!("lockout:{}", identity);

        // Increment attempts using INCR
        let attempts: u64 = conn.incr(&attempts_key, 1).await?;
        
        // Set expiry on the attempts key (sliding window for failures check)
        // e.g., if you fail 5 times in 10 minutes.
        if attempts == 1 {
            let _: () = conn.expire(&attempts_key, 600).await?; // 10 minutes failure window
        }

        if attempts >= threshold {
            // Lock the account
            let _: () = conn.set_ex(&lock_key, "locked", lock_duration_sec).await?;
            // Reset attempts so strict lockout period applies
            let _: () = conn.del(&attempts_key).await?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Resets the failure counter (e.g., on successful login).
    async fn reset_failure(&self, identity: &str) -> Result<(), LockoutError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let attempts_key = format!("login_failures:{}", identity);
        let _: () = conn.del(&attempts_key).await?;
        Ok(())
    }
}

impl AccountLockoutService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}
