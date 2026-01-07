/// REST Resource for email confirmation endpoint.
/// Used for query parameter extraction in GET request.
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

/// Query parameters for GET /api/v1/identity/confirm-registration
#[derive(Debug, Deserialize, IntoParams, Validate)]
pub struct ConfirmEmailQueryParams {
    /// Verification token from email link
    #[validate(length(min = 32, message = "Invalid token format"))]
    pub token: String,
}

/// Success response (used internally, actual response is HTTP redirect)
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ConfirmEmailResponse {
    pub message: String,
}
