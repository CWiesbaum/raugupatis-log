use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use askama::Template;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod templates;

use config::AppConfig;
use database::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub config: Arc<AppConfig>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Arc::new(AppConfig::load()?);
    info!("Configuration loaded successfully");

    // Initialize database
    let db = Arc::new(Database::new(&config.database_url).await?);
    info!("Database initialized successfully");

    // Run migrations
    db.migrate().await?;
    info!("Database migrations completed");

    let app_state = AppState { db, config: config.clone() };

    // Build the application with middleware stack
    let app = Router::new()
        .route("/", get(home_handler))
        .route("/health", get(health_handler))
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive()),
        );

    let listener = TcpListener::bind(&config.server_address).await?;
    info!("Server starting on {}", config.server_address);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn home_handler() -> Html<String> {
    let template = templates::HomeTemplate {
        title: "Raugupatis Log".to_string(),
        message: "Welcome to Raugupatis Log - Your Fermentation Tracking Companion!".to_string(),
    };
    
    Html(template.render().unwrap_or_else(|_| "Template render error".to_string()))
}

async fn health_handler(State(state): State<AppState>) -> Result<&'static str, StatusCode> {
    // Simple health check - could be expanded to check database connectivity
    match state.db.health_check().await {
        Ok(_) => Ok("OK"),
        Err(e) => {
            warn!("Health check failed: {}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}