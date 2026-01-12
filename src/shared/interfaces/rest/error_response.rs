use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Standard error response for API endpoints
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// User-friendly error message
    pub message: String,
    /// HTTP status code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u16>,
}

impl ErrorResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
        }
    }

    pub fn with_code(mut self, code: u16) -> Self {
        self.code = Some(code);
        self
    }

    /// Create response for generic internal error without exposing details
    pub fn internal_error() -> Self {
        Self::new("An internal error occurred. Please try again later.")
    }

    /// Create response for service unavailable
    pub fn service_unavailable() -> Self {
        Self::new("Service temporarily unavailable. Please try again later.")
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let status = self
            .code
            .and_then(|c| StatusCode::from_u16(c).ok())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (status, Json(self)).into_response()
    }
}
