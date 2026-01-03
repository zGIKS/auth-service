use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Auth Service API",
        version = "1.0.0",
        description = "API documentation for the Auth Service"
    ),
    servers(
        (url = "http://localhost:3000", description = "Local server")
    )
    // Aquí se agregarían componentes, schemas y paths de otros módulos
)]
pub struct OpenApiConfiguration;
