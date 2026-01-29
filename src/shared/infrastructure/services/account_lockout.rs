use async_trait::async_trait;
use redis::{AsyncCommands, Client, RedisError};

#[async_trait]
pub trait AccountLockoutVerifier: Send + Sync {
    async fn check_locked(&self, identity: &str, ip: Option<&str>) -> Result<(), LockoutError>;
    async fn register_failure(
        &self,
        identity: &str,
        ip: Option<&str>,
        threshold: u64,
        lock_duration_sec: u64,
    ) -> Result<bool, LockoutError>;
    async fn reset_failure(&self, identity: &str, ip: Option<&str>) -> Result<(), LockoutError>;
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
    /// If IP is provided, checks if that specific IP is locked for this identity.
    /// Also checks global identity lock (if any).
    async fn check_locked(&self, identity: &str, ip: Option<&str>) -> Result<(), LockoutError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        // 1. Check Global Lock (lockout:email)
        let global_lock_key = format!("lockout:{}", identity);
        let global_ttl: i64 = conn.ttl(&global_lock_key).await?;
        if global_ttl > 0 {
            return Err(LockoutError::Locked(global_ttl as u64));
        }

        // 2. Check IP-specific Lock (lockout:email:ip)
        if let Some(ip_addr) = ip {
            let ip_lock_key = format!("lockout:{}:{}", identity, ip_addr);
            let ip_ttl: i64 = conn.ttl(&ip_lock_key).await?;
            if ip_ttl > 0 {
                return Err(LockoutError::Locked(ip_ttl as u64));
            }
        }

        Ok(())
    }

    /// Registers a failed attempt.
    /// If IP is provided, registers failure against `identity:ip`.
    /// Otherwise registers against `identity` globally.
    /// If attempts exceed threshold, locks the corresponding scope.
    /// Returns true if locked.
    async fn register_failure(
        &self,
        identity: &str,
        ip: Option<&str>,
        threshold: u64,
        lock_duration_sec: u64,
    ) -> Result<bool, LockoutError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let (attempts_key, lock_key) = if let Some(ip_addr) = ip {
            (
                format!("login_failures:{}:{}", identity, ip_addr),
                format!("lockout:{}:{}", identity, ip_addr),
            )
        } else {
            (
                format!("login_failures:{}", identity),
                format!("lockout:{}", identity),
            )
        };

        // Increment attempts using INCR
        let attempts: u64 = conn.incr(&attempts_key, 1).await?;

        // Set expiry on the attempts key (sliding window for failures check)
        if attempts == 1 {
            let _: () = conn.expire(&attempts_key, 600).await?; // 10 minutes failure window
        }

        if attempts >= threshold {
            // Lock the account (scoped to IP if provided)
            let _: () = conn.set_ex(&lock_key, "locked", lock_duration_sec).await?;
            // Reset attempts so strict lockout period applies
            let _: () = conn.del(&attempts_key).await?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Resets the failure counter (e.g., on successful login).
    /// Clears both global and IP-specific counters for this identity to be safe.
    async fn reset_failure(&self, identity: &str, ip: Option<&str>) -> Result<(), LockoutError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let global_attempts = format!("login_failures:{}", identity);
        let _: () = conn.del(&global_attempts).await?;

        if let Some(ip_addr) = ip {
            let ip_attempts = format!("login_failures:{}:{}", identity, ip_addr);
            let _: () = conn.del(&ip_attempts).await?;
        }

        Ok(())
    }
}

impl AccountLockoutService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}
