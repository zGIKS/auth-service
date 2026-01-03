use utoipa::OpenApi;

pub mod shared;
pub mod iam;

#[derive(OpenApi)]
#[openapi(
    paths(
        iam::identity::interfaces::rest::controllers::identity_controller::register_identity
    ),
    components(
        schemas(
            iam::identity::interfaces::rest::resources::register_identity_resource::RegisterIdentityRequest,
            iam::identity::interfaces::rest::resources::register_identity_resource::RegisterIdentityResponse
        )
    ),
    tags(
        (name = "identity", description = "Identity management")
    )
)]
pub struct ApiDoc;
