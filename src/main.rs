use axum::{routing::post, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use dotenvy::dotenv;
use sea_orm::{Database, Schema, ConnectionTrait};
use auth_service::{iam, ApiDoc};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&database_url).await.expect("Failed to connect to DB");

    // Create table if not exists
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let mut create_table_op = schema.create_table_from_entity(iam::identity::infrastructure::persistence::postgres::model::Entity);
    let stmt = builder.build(create_table_op.if_not_exists());

    match db.execute(stmt).await {
        Ok(_) => println!("Table 'users' checked/created successfully."),
        Err(e) => eprintln!("Error creating table: {}", e),
    }

    let app = Router::new()
        .route("/api/v1/auth/sign-up", post(iam::identity::interfaces::rest::controllers::identity_controller::register_identity))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(db);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap();

    println!("Servidor corriendo en http://localhost:{}", port);
    println!("Swagger UI disponible en http://localhost:{}/swagger-ui", port);

    axum::serve(listener, app).await.unwrap();
}