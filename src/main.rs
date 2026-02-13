use auth_service::shared::interfaces::rest::app_state::AppState;
use auth_service::{ApiDoc, iam};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{header, StatusCode},
    routing::{get, post},
    error_handling::HandleErrorLayer,
};
use dotenvy::dotenv;
use sea_orm::{ConnectionTrait, Database, Schema};
use std::time::Duration;
use tower::ServiceBuilder;
use tower::limit::ConcurrencyLimitLayer;
use tower::load_shed::LoadShedLayer;
use tower_http::{
    cors::CorsLayer,
    set_header::SetResponseHeaderLayer,
    timeout::TimeoutLayer,
};
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use auth_service::shared::infrastructure::circuit_breaker::create_circuit_breaker;
use auth_service::shared::infrastructure::persistence::redis as redis_infra;
use auth_service::shared::interfaces::rest::middleware::rate_limit_middleware;

fn parse_app_env() -> Result<String, Box<dyn std::error::Error>> {
    let raw = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
    let env = raw.to_lowercase();

    match env.as_str() {
        "dev" | "prod" => Ok(env),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("APP_ENV must be 'dev' or 'prod', got '{raw}'"),
        )
        .into()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let app_env = parse_app_env()?;

    let port: u16 = std::env::var("PORT")
        .map_err(|_| "PORT must be set")?
        .parse()
        .map_err(|_| "PORT must be a valid number")?;

    let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
    let db = Database::connect(&database_url)
        .await
        .map_err(|e| format!("Failed to connect to DB: {}", e))?;

    let redis_client = redis_infra::connect()
        .await
        .map_err(|e| format!("Redis error: {}", e))?;

    let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| "JWT_SECRET must be set")?;
    let session_duration_seconds: u64 = std::env::var("SESSION_DURATION_SECONDS")
        .map_err(|_| "SESSION_DURATION_SECONDS must be set")?
        .parse()
        .map_err(|_| "SESSION_DURATION_SECONDS must be a number")?;

    let refresh_token_duration_seconds: u64 = std::env::var("REFRESH_TOKEN_DURATION_SECONDS")
        .map_err(|_| "REFRESH_TOKEN_DURATION_SECONDS must be set")?
        .parse()
        .map_err(|_| "REFRESH_TOKEN_DURATION_SECONDS must be a number")?;

    let pending_registration_ttl_seconds: u64 = std::env::var("PENDING_REGISTRATION_TTL_SECONDS")
        .map_err(|_| "PENDING_REGISTRATION_TTL_SECONDS must be set")?
        .parse()
        .map_err(|_| "PENDING_REGISTRATION_TTL_SECONDS must be a number")?;

    let password_reset_ttl_seconds: u64 = std::env::var("PASSWORD_RESET_TTL_SECONDS")
        .map_err(|_| "PASSWORD_RESET_TTL_SECONDS must be set")?
        .parse()
        .map_err(|_| "PASSWORD_RESET_TTL_SECONDS must be a number")?;

    let lockout_threshold: u64 = std::env::var("LOCKOUT_THRESHOLD")
        .map_err(|_| "LOCKOUT_THRESHOLD must be set")?
        .parse()
        .map_err(|_| "LOCKOUT_THRESHOLD must be a number")?;

    let lockout_duration_seconds: u64 = std::env::var("LOCKOUT_DURATION_SECONDS")
        .map_err(|_| "LOCKOUT_DURATION_SECONDS must be set")?
        .parse()
        .map_err(|_| "LOCKOUT_DURATION_SECONDS must be a number")?;

    let frontend_url = std::env::var("FRONTEND_URL").ok();

    let google_client_id =
        std::env::var("GOOGLE_CLIENT_ID").map_err(|_| "GOOGLE_CLIENT_ID must be set")?;
    let google_client_secret =
        std::env::var("GOOGLE_CLIENT_SECRET").map_err(|_| "GOOGLE_CLIENT_SECRET must be set")?;
    let google_redirect_uri =
        std::env::var("GOOGLE_REDIRECT_URI").map_err(|_| "GOOGLE_REDIRECT_URI must be set")?;

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
        .route(
            "/api/v1/identity/sign-up",
            post(iam::identity::interfaces::rest::controllers::identity_controller::register_identity),
        )
        .route(
            "/api/v1/auth/sign-in",
            post(iam::authentication::interfaces::rest::controllers::authentication_controller::signin),
        )
        .route(
            "/api/v1/auth/logout",
            post(iam::authentication::interfaces::rest::controllers::authentication_controller::logout),
        )
        .route(
            "/api/v1/auth/refresh-token",
            post(iam::authentication::interfaces::rest::controllers::authentication_controller::refresh_token),
        )
        .route(
            "/api/v1/auth/verify",
            get(iam::authentication::interfaces::rest::controllers::authentication_controller::verify_token),
        )
        .route(
            "/api/v1/auth/google",
            get(iam::federation::interfaces::rest::controllers::google_controller::redirect_to_google),
        )
        .route(
            "/api/v1/auth/google/callback",
            get(iam::federation::interfaces::rest::controllers::google_controller::google_callback),
        )
        .route(
            "/api/v1/auth/google/claim",
            post(iam::federation::interfaces::rest::controllers::google_controller::claim_token),
        )
        .route(
            "/api/v1/identity/confirm-registration",
            get(iam::identity::interfaces::rest::controllers::identity_controller::confirm_registration),
        )
        .route(
            "/api/v1/identity/forgot-password",
            post(iam::identity::interfaces::rest::controllers::identity_controller::request_password_reset),
        )
        .route(
            "/api/v1/identity/reset-password",
            post(iam::identity::interfaces::rest::controllers::identity_controller::reset_password),
        );

    let app = if app_env == "dev" {
        app.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
    } else {
        app
    };
    
    let app = app
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: Box<dyn std::error::Error + Send + Sync>| async move {
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        format!("Service unavailable: {}", err),
                    )
                }))
                .layer(LoadShedLayer::new())
                .layer(ConcurrencyLimitLayer::new(512))
                .layer(DefaultBodyLimit::max(1024 * 16)) // Limit body to 16KB for security
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    rate_limit_middleware,
                ))
                .layer(CorsLayer::permissive()) // Customize as needed
                // Security Headers
                .layer(SetResponseHeaderLayer::overriding(
                    header::X_CONTENT_TYPE_OPTIONS,
                    header::HeaderValue::from_static("nosniff"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    header::X_FRAME_OPTIONS,
                    header::HeaderValue::from_static("DENY"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    header::STRICT_TRANSPORT_SECURITY,
                    header::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
                ))
        )
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

    println!("Servidor corriendo en http://localhost:{}", port);
    if app_env == "dev" {
        println!("Swagger UI disponible en http://localhost:{}/swagger-ui", port);
    } else {
        println!("Swagger UI deshabilitado en modo prod");
    }

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}
