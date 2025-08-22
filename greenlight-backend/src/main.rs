mod config;
mod error;
mod models;
mod services;
mod handlers;
mod middleware;

use axum::{
    middleware::{from_fn_with_state},
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use services::JwtService;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub jwt_service: Arc<JwtService>,
    pub config: Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "greenlight_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded successfully");

    // Setup database connection
    let db_pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    tracing::info!("Database migrations completed");

    // Initialize services
    let jwt_service = Arc::new(JwtService::new(&config.jwt_secret));

    // Create application state
    let state = AppState {
        db_pool,
        jwt_service,
        config: config.clone(),
    };

    // Build our application with routes
    let app = Router::new()
        // Public routes (no authentication required)
        .route("/auth/register", post(handlers::auth::signup))
        .route("/auth/login", post(handlers::auth::login))
        .route("/health", get(health_check))
        
        // Protected routes (authentication required)
        .nest("/", Router::new()
            .route("/node/register", post(handlers::node::register_node))
            .route("/node/recover", post(handlers::node::recover_node))
            .route("/node/info", get(handlers::node::get_node_info))
            .route("/node/balance", get(handlers::node::get_balance))
            .route("/node/offer", post(handlers::node::create_offer))
            .route("/ws", get(handlers::websocket::websocket_handler))
            .route_layer(from_fn_with_state(
                state.clone(),
                middleware::auth::auth_middleware,
            ))
        )
        
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.server_host, config.server_port)).await?;
    tracing::info!("Server starting on {}:{}", config.server_host, config.server_port);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
