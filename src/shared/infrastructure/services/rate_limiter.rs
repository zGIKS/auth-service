use redis::{Client, RedisError, Script};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct RedisRateLimiter {
    client: Client,
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    #[error("Rate limit exceeded. Retry after {0} ms")]
    Exceeded(u64),
}

impl RedisRateLimiter {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Checks if the request is allowed using the Token Bucket algorithm.
    ///
    /// # Arguments
    /// * `key` - The unique key for the limit (e.g., "rate_limit:ip:127.0.0.1")
    /// * `limit` - The burst capacity of the bucket
    /// * `rate_per_sec` - The rate at which tokens are refilled per second
    /// * `cost` - The cost of the current request (usually 1)
    ///
    /// # Returns
    /// * `Ok(())` if allowed
    /// * `Err(RateLimitError::Exceeded(retry_after_ms))` if limited
    pub async fn check(
        &self,
        key: &str,
        limit: u64,
        rate_per_sec: f64,
        cost: u64,
    ) -> Result<(), RateLimitError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(RateLimitError::Redis)?;

        // Lua script for Token Bucket
        // KEYS[1]: tokens_key (stores current token count)
        // KEYS[2]: timestamp_key (stores last refill timestamp)
        // ARGV[1]: limit (max tokens)
        // ARGV[2]: rate (tokens/sec)
        // ARGV[3]: cost (tokens to consume)
        // ARGV[4]: now (current timestamp in seconds)
        // ARGV[5]: ttl (expiration for keys, e.g., 2 times the time to fill bucket)
        let script = Script::new(
            r#"
            local tokens_key = KEYS[1]
            local ts_key = KEYS[2]
            local limit = tonumber(ARGV[1])
            local rate = tonumber(ARGV[2])
            local cost = tonumber(ARGV[3])
            local now = tonumber(ARGV[4])
            local ttl = tonumber(ARGV[5])

            local tokens = tonumber(redis.call("get", tokens_key))
            local last_refill = tonumber(redis.call("get", ts_key))

            if tokens == nil then
                tokens = limit
                last_refill = now
            end

            local delta = math.max(0, now - last_refill)
            local filled_tokens = math.min(limit, tokens + (delta * rate))

            if filled_tokens >= cost then
                redis.call("set", tokens_key, filled_tokens - cost)
                redis.call("set", ts_key, now)
                redis.call("expire", tokens_key, ttl)
                redis.call("expire", ts_key, ttl)
                return {1, 0}
            else
                local missing = cost - filled_tokens
                local retry_seconds = missing / rate
                return {0, retry_seconds}
            end
        "#,
        );

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let ttl = (limit as f64 / rate_per_sec * 2.0).ceil() as u64; // Keep keys long enough to recover
        let ttl = if ttl < 60 { 60 } else { ttl };

        let tokens_key = format!("{}:tokens", key);
        let ts_key = format!("{}:ts", key);

        let result: (i32, f64) = script
            .key(&[tokens_key, ts_key])
            .arg(limit)
            .arg(rate_per_sec)
            .arg(cost)
            .arg(now)
            .arg(ttl)
            .invoke_async(&mut conn)
            .await
            .map_err(RateLimitError::Redis)?;

        if result.0 == 1 {
            Ok(())
        } else {
            Err(RateLimitError::Exceeded((result.1 * 1000.0) as u64))
        }
    }
}
