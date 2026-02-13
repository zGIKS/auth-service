use redis::Client;
use std::env;

pub async fn connect() -> Result<Client, String> {
    let redis_url = env::var("REDIS_URL").map_err(|_| "REDIS_URL must be set".to_string())?;
    Client::open(redis_url).map_err(|e| format!("Failed to create Redis client: {}", e))
}
