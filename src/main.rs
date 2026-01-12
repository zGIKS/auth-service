use axum::{routing::{post, get}, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use dotenvy::dotenv;
use sea_orm::{Database, Schema, ConnectionTrait};
use auth_service::{iam, ApiDoc};
use auth_service::shared::interfaces::rest::app_state::AppState;


use auth_service::shared::infrastructure::persistence::redis as redis_infra;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&database_url).await.expect("Failed to connect to DB");

    let redis_client = redis_infra::connect().await;

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let session_duration_seconds: u64 = std::env::var("SESSION_DURATION_SECONDS")
        .unwrap_or_else(|_| "3600".to_string())
        .parse()
        .unwrap_or(3600);

    let pending_registration_ttl_seconds: u64 = std::env::var("PENDING_REGISTRATION_TTL_SECONDS")
        .unwrap_or_else(|_| "900".to_string())
        .parse()
        .unwrap_or(900);

    let password_reset_ttl_seconds: u64 = std::env::var("PASSWORD_RESET_TTL_SECONDS")
        .unwrap_or_else(|_| "900".to_string())
        .parse()
        .unwrap_or(900);

    let frontend_url = std::env::var("FRONTEND_URL").ok();

    // Create table if not exists
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let mut create_table_op = schema.create_table_from_entity(iam::identity::infrastructure::persistence::postgres::model::Entity);
    let stmt = builder.build(create_table_op.if_not_exists());

    match db.execute(stmt).await {
        Ok(_) => println!("Table 'users' checked/created successfully."),
        Err(e) => eprintln!("Error creating table: {}", e),
    }

    let state = AppState {
        db,
        redis: redis_client,
        jwt_secret,
        session_duration_seconds,
        pending_registration_ttl_seconds,
        password_reset_ttl_seconds,
        frontend_url,
    };

    let app = Router::new()
        .route("/api/v1/auth/sign-up", post(iam::identity::interfaces::rest::controllers::identity_controller::register_identity))
        .route("/api/v1/auth/sign-in", post(iam::authentication::interfaces::rest::controllers::authentication_controller::signin))
        .route("/api/v1/auth/refresh-token", post(iam::authentication::interfaces::rest::controllers::authentication_controller::refresh_token))
        .route("/api/v1/identity/confirm-registration", get(iam::identity::interfaces::rest::controllers::identity_controller::confirm_registration))
        .route("/api/v1/identity/forgot-password", post(iam::identity::interfaces::rest::controllers::identity_controller::request_password_reset))
        .route("/api/v1/identity/reset-password", post(iam::identity::interfaces::rest::controllers::identity_controller::reset_password))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap();

    println!("Servidor corriendo en http://localhost:{}", port);
    println!("Swagger UI disponible en http://localhost:{}/swagger-ui", port);

    axum::serve(listener, app).await.unwrap();
}