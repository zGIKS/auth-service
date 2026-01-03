use utoipa::OpenApi;

pub mod shared;
pub mod iam;

#[derive(OpenApi)]
#[openapi(
    paths(
        iam::identity::interfaces::rest::controllers::identity_controller::register_identity,
        iam::authentication::interfaces::rest::controllers::authentication_controller::signin
    ),
    components(
        schemas(
            iam::identity::interfaces::rest::resources::register_identity_resource::RegisterIdentityRequest,
            iam::identity::interfaces::rest::resources::register_identity_resource::RegisterIdentityResponse,
            iam::authentication::interfaces::rest::resources::signin_resource::SigninResource,
            iam::authentication::interfaces::rest::resources::signin_resource::TokenResponse
        )
    ),
    tags(
        (name = "identity", description = "Identity management"),
        (name = "auth", description = "Authentication")
    )
)]
pub struct ApiDoc;
