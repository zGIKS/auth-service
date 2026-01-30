/// Query for email confirmation via GET request.
/// Represents a read-intent operation to validate a verification token.
use validator::Validate;

#[derive(Debug, Clone, Validate)]
pub struct ConfirmEmailQuery {
    #[validate(length(min = 32, message = "Token must be at least 32 characters"))]
    pub token: String,
}

impl ConfirmEmailQuery {
    pub fn new(token: String) -> Result<Self, validator::ValidationErrors> {
        let query = Self { token };
        query.validate()?;
        Ok(query)
    }
}
