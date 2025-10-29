use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use raugupatis_log::{config::AppConfig, database::Database, create_router, AppState};

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

    let app = create_router(app_state);

    let listener = TcpListener::bind(&config.server_address).await?;
    info!("Server starting on {}", config.server_address);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn home_handler() -> Html<String> {
    let template = templates::HomeTemplate {
        title: "Raugupatis Log".to_string(),
        message: "Welcome! Master your fermentation batches with digital precision.".to_string(),
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
