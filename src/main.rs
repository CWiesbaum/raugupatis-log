use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use raugupatis_log::{config::AppConfig, create_router, database::Database, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
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

    let app_state = AppState {
        db,
        config: config.clone(),
    };

    let app = create_router(app_state).await;

    let listener = TcpListener::bind(&config.server_address).await?;
    info!("Server starting on {}", config.server_address);

    axum::serve(listener, app).await?;

    Ok(())
}
