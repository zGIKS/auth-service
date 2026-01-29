use auth_service::shared::interfaces::rest::app_state::AppState;
use auth_service::{ApiDoc, iam};
use axum::{
    Router,
    routing::{get, post},
};
use dotenvy::dotenv;
use sea_orm::{ConnectionTrait, Database, Schema};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use auth_service::shared::infrastructure::circuit_breaker::create_circuit_breaker;
use auth_service::shared::infrastructure::persistence::redis as redis_infra;
use auth_service::shared::interfaces::rest::middleware::rate_limit_middleware;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let port: u16 = std::env::var("PORT")
        .expect("PORT must be set")
        .parse()
        .expect("PORT must be a valid number");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&database_url)
        .await
        .expect("Failed to connect to DB");

    let redis_client = redis_infra::connect().await;

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let session_duration_seconds: u64 = std::env::var("SESSION_DURATION_SECONDS")
        .expect("SESSION_DURATION_SECONDS must be set")
        .parse()
        .expect("SESSION_DURATION_SECONDS must be a number");

    let refresh_token_duration_seconds: u64 = std::env::var("REFRESH_TOKEN_DURATION_SECONDS")
        .expect("REFRESH_TOKEN_DURATION_SECONDS must be set")
        .parse()
        .expect("REFRESH_TOKEN_DURATION_SECONDS must be a number");

    let pending_registration_ttl_seconds: u64 = std::env::var("PENDING_REGISTRATION_TTL_SECONDS")
        .expect("PENDING_REGISTRATION_TTL_SECONDS must be set")
        .parse()
        .expect("PENDING_REGISTRATION_TTL_SECONDS must be a number");

    let password_reset_ttl_seconds: u64 = std::env::var("PASSWORD_RESET_TTL_SECONDS")
        .expect("PASSWORD_RESET_TTL_SECONDS must be set")
        .parse()
        .expect("PASSWORD_RESET_TTL_SECONDS must be a number");

    let lockout_threshold: u64 = std::env::var("LOCKOUT_THRESHOLD")
        .expect("LOCKOUT_THRESHOLD must be set")
        .parse()
        .expect("LOCKOUT_THRESHOLD must be a number");

    let lockout_duration_seconds: u64 = std::env::var("LOCKOUT_DURATION_SECONDS")
        .expect("LOCKOUT_DURATION_SECONDS must be set")
        .parse()
        .expect("LOCKOUT_DURATION_SECONDS must be a number");

    let frontend_url = std::env::var("FRONTEND_URL").ok();

    let google_client_id = std::env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set");
    let google_client_secret =
        std::env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET must be set");
    let google_redirect_uri =
        std::env::var("GOOGLE_REDIRECT_URI").expect("GOOGLE_REDIRECT_URI must be set");

    // Create table if not exists
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let mut create_table_op = schema.create_table_from_entity(
        iam::identity::infrastructure::persistence::postgres::model::Entity,
    );
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
        refresh_token_duration_seconds,
        pending_registration_ttl_seconds,
        password_reset_ttl_seconds,
        frontend_url,
        lockout_threshold,
        lockout_duration_seconds,
        google_client_id,
        google_client_secret,
        google_redirect_uri,
        circuit_breaker: create_circuit_breaker(),
    };

    let app = Router::new()
        .route("/api/v1/auth/sign-up", post(iam::identity::interfaces::rest::controllers::identity_controller::register_identity))
        .route("/api/v1/auth/sign-in", post(iam::authentication::interfaces::rest::controllers::authentication_controller::signin))
        .route("/api/v1/auth/logout", post(iam::authentication::interfaces::rest::controllers::authentication_controller::logout))
        .route("/api/v1/auth/refresh-token", post(iam::authentication::interfaces::rest::controllers::authentication_controller::refresh_token))
        .route("/api/v1/auth/verify", get(iam::authentication::interfaces::rest::controllers::authentication_controller::verify_token))
        .route("/api/v1/auth/google", get(iam::federation::interfaces::rest::controllers::google_controller::redirect_to_google))
        .route("/api/v1/auth/google/callback", get(iam::federation::interfaces::rest::controllers::google_controller::google_callback))
        .route("/api/v1/auth/google/claim", post(iam::federation::interfaces::rest::controllers::google_controller::claim_token))
        .route("/api/v1/identity/confirm-registration", get(iam::identity::interfaces::rest::controllers::identity_controller::confirm_registration))
        .route("/api/v1/identity/forgot-password", post(iam::identity::interfaces::rest::controllers::identity_controller::request_password_reset))
        .route("/api/v1/identity/reset-password", post(iam::identity::interfaces::rest::controllers::identity_controller::reset_password))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(axum::middleware::from_fn_with_state(state.clone(), rate_limit_middleware))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Servidor corriendo en http://localhost:{}", port);
    println!(
        "Swagger UI disponible en http://localhost:{}/swagger-ui",
        port
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}
