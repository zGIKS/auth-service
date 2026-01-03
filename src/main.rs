use axum::{routing::get, Router};
use utoipa::{OpenApi};
use utoipa_swagger_ui::SwaggerUi;
use dotenvy::dotenv;

#[derive(OpenApi)]
#[openapi(
    paths(hello),
    tags(
        (name = "hello", description = "Hello World endpoint")
    )
)]
struct ApiDoc;

/// Hello World endpoint
#[utoipa::path(
    get,
    path = "/hello",
    tag = "hello",
    responses(
        (status = 200, description = "Hello World response", body = String)
    )
)]
async fn hello() -> String {
    "Hello World".to_string()
}

mod shared;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    let app = Router::new()
        .route("/hello", get(hello))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap();

    println!("Servidor corriendo en http://localhost:{}", port);
    println!("Swagger UI disponible en http://localhost:{}/swagger-ui", port);

    axum::serve(listener, app).await.unwrap();
}
