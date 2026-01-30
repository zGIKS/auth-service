use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct RefreshTokenResource {
    #[validate(length(min = 1))]
    #[schema(example = "string")]
    pub refresh_token: String,
}
