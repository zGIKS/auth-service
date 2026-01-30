use redis::Client;
use std::env;

pub async fn connect() -> Client {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    Client::open(redis_url).expect("Failed to create Redis client")
}
