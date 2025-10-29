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
use handlers::register_user;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub config: Arc<AppConfig>,
}

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(crate::templates::home_handler))
        .route("/register", get(crate::templates::register_handler))
        .route("/health", get(health_handler))
        .route("/api/users/register", post(register_user))
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
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
