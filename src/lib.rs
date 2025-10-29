use axum::{
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::TraceLayer,
};
use tower_sessions::{cookie::time, MemoryStore, SessionManagerLayer};
use std::sync::Arc;

pub mod auth;
pub mod config;
pub mod database;
mod handlers;
mod models;
mod repository;
pub mod templates;

pub use config::AppConfig;
pub use database::Database;
use handlers::{login_user, register_user};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub config: Arc<AppConfig>,
}

pub fn create_router(app_state: AppState) -> Router {
    // Create session store and layer
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_expiry(tower_sessions::Expiry::OnInactivity(time::Duration::days(1))); // 24 hours

    Router::new()
        .route("/", get(crate::templates::home_handler))
        .route("/register", get(crate::templates::register_handler))
        .route("/health", get(health_handler))
        .route("/login", get(crate::templates::login_handler))
        .route("/dashboard", get(crate::templates::dashboard_handler))
        .route("/api/users/register", post(register_user))
        .route("/api/users/login", post(login_user))
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(session_layer)
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive()),
        )
}

use axum::{extract::State, http::StatusCode};
use tracing::warn;

async fn health_handler(State(state): State<AppState>) -> Result<&'static str, StatusCode> {
    match state.db.health_check().await {
        Ok(_) => Ok("OK"),
        Err(e) => {
            warn!("Health check failed: {}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}
