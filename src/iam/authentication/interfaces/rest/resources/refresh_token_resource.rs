use serde::Deserialize;
use validator::Validate;
use utoipa::ToSchema;

#[derive(Deserialize, Validate, ToSchema)]
pub struct RefreshTokenResource {
    #[validate(length(min = 1))]
    #[schema(example = "string")]
    pub refresh_token: String,
}
