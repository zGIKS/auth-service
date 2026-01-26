use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};

pub struct WebConfiguration;

impl WebConfiguration {
    pub fn cors() -> CorsLayer {
        CorsLayer::new()
            .allow_origin(Any) // En producción especificar orígenes
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
            ])
            .allow_headers(Any)
    }
}
