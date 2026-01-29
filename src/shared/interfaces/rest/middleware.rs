use crate::shared::infrastructure::services::rate_limiter::RedisRateLimiter;
use crate::shared::interfaces::rest::app_state::AppState;
use axum::{
    extract::{ConnectInfo, Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let client = state.redis.clone();
    let limiter = RedisRateLimiter::new(client);

    // Determine client IP
    // Priority:
    // 1. ConnectInfo (Real Remote Address)
    // 2. X-Forwarded-For (Only if we decide to trust it - usually depends on config)
    // For this implementation, we prioritize ConnectInfo logic as requested.

    // We try to get ConnectInfo extension which Axum provides if Router is properly set up with .into_make_service_with_connect_info::<SocketAddr>()
    // If not available, we fall back to "unknown".
    // Note: The user prompt implementation uses `x-forwarded-for` naively.

    let remote_ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string());

    let x_forwarded = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("unknown").trim().to_string());

    // Logic: Use remote_ip if valid. Only use x_forwarded if configured or if remote_ip looks like a local proxy (not implemented here per strict request, but commonly done).
    // The request specifically asks: "ensure the limiter reuses the real IP (remote_addr) and validate x-forwarded-for only when coming from trusted proxies"

    // For now, we will prefer remote_ip. If the app is behind a reverse proxy (like Nginx on localhost), remote_ip will be 127.0.0.1.
    // In that specific case, we might trust XFF.

    let ip = match remote_ip {
        Some(ip_str) => {
            // Check if trusted proxy (e.g. localhost)
            if ip_str == "127.0.0.1" || ip_str == "::1" {
                x_forwarded.unwrap_or(ip_str)
            } else {
                ip_str
            }
        }
        None => {
            // Fallback if ConnectInfo is missing.
            // Do NOT trust X-Forwarded-For blindly as it can be spoofed.
            // Since we can't verify the source, we treat it as unknown.
            "unknown".to_string()
        }
    };

    let path = req.uri().path().to_string();
    let method = req.method().clone();

    // Global IP Limit: 20 req/sec
    let global_key = format!("rl:ip:{}", ip);
    // limit=20, rate=20.0 (20 tokens/sec)
    if (limiter.check(&global_key, 20, 20.0, 1).await).is_err() {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Sign-in limit: 5 req/min per IP
    if path.contains("/auth/sign-in") && method == Method::POST {
        let path_key = format!("rl:signin:ip:{}", ip);
        if (limiter.check(&path_key, 5, 0.0833, 1).await).is_err() {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }

    // Forgot Password limit: 3 req/min per IP
    if path.contains("/identity/forgot-password") && method == Method::POST {
        let path_key = format!("rl:forgot:ip:{}", ip);
        if (limiter.check(&path_key, 3, 0.05, 1).await).is_err() {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }

    Ok(next.run(req).await)
}
