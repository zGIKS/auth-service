use utoipa::OpenApi;

pub mod iam;
pub mod messaging;
pub mod shared;

#[derive(OpenApi)]
#[openapi(
    paths(
        iam::identity::interfaces::rest::controllers::identity_controller::register_identity,
        iam::identity::interfaces::rest::controllers::identity_controller::confirm_registration,
        iam::identity::interfaces::rest::controllers::identity_controller::request_password_reset,
        iam::identity::interfaces::rest::controllers::identity_controller::reset_password,
        iam::authentication::interfaces::rest::controllers::authentication_controller::signin,
        iam::authentication::interfaces::rest::controllers::authentication_controller::logout,
        iam::authentication::interfaces::rest::controllers::authentication_controller::refresh_token,
        iam::authentication::interfaces::rest::controllers::authentication_controller::verify_token,
        iam::federation::interfaces::rest::controllers::google_controller::redirect_to_google,
        iam::federation::interfaces::rest::controllers::google_controller::google_callback
    ),
    components(
        schemas(
            iam::identity::interfaces::rest::resources::register_identity_resource::RegisterIdentityRequest,
            iam::identity::interfaces::rest::resources::register_identity_resource::RegisterIdentityResponse,
            iam::identity::domain::model::commands::confirm_registration_command::ConfirmRegistrationCommand,
            iam::identity::interfaces::rest::resources::request_password_reset_resource::RequestPasswordResetRequest,
            iam::identity::interfaces::rest::resources::request_password_reset_resource::RequestPasswordResetResponse,
            iam::identity::interfaces::rest::resources::reset_password_resource::ResetPasswordRequest,
            iam::identity::interfaces::rest::resources::reset_password_resource::ResetPasswordResponse,
            iam::authentication::interfaces::rest::resources::signin_resource::SigninResource,
            iam::authentication::interfaces::rest::resources::signin_resource::TokenResponse,
            iam::authentication::interfaces::rest::resources::logout_resource::LogoutResource,
            iam::authentication::interfaces::rest::resources::refresh_token_resource::RefreshTokenResource,
            iam::authentication::interfaces::rest::resources::verify_token_resource::VerifyTokenResource,
            iam::authentication::interfaces::rest::resources::verify_token_resource::VerifyTokenResponse,
            iam::federation::interfaces::rest::resources::google_callback_query::GoogleCallbackQuery,
            shared::interfaces::rest::error_response::ErrorResponse
        )
    ),
    tags(
        (name = "identity", description = "Identity management"),
        (name = "auth", description = "Authentication")
    )
)]
pub struct ApiDoc;
