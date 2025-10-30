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
use tower_sessions::{SessionManagerLayer, Expiry};
use tower_sessions_rusqlite_store::{RusqliteStore, tokio_rusqlite};
use std::sync::Arc;
use time::Duration;

pub mod config;
pub mod database;
pub mod fermentation;
pub mod templates;
pub mod users;

pub use config::AppConfig;
pub use database::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub config: Arc<AppConfig>,
}

pub async fn create_router(app_state: AppState) -> Router {
    // Create tokio-rusqlite connection for session store
    let db_path = app_state.db.get_db_path().to_string();
    let session_conn = tokio_rusqlite::Connection::open(db_path)
        .await
        .expect("Failed to open database for session store");
    
    // Create session store using tokio-rusqlite connection
    let session_store = RusqliteStore::new(session_conn);
    
    // Create session layer with 24 hour expiration
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::hours(24)));

    Router::new()
        .route("/", get(crate::templates::home_handler))
        .route("/register", get(crate::users::register_handler))
        .route("/health", get(health_handler))
        .route("/login", get(crate::users::login_handler))
        .route("/dashboard", get(crate::templates::dashboard_handler))
        .route("/fermentations", get(crate::fermentation::fermentation_list_handler))
        .route("/fermentation/new", get(crate::fermentation::new_fermentation_handler))
        .route("/profile", get(crate::users::profile_handler))
        .route("/api/users/register", post(crate::users::register_user))
        .route("/api/users/login", post(crate::users::login_user))
        .route("/api/users/logout", post(crate::users::logout_user))
        .route("/api/fermentations", get(crate::fermentation::list_fermentations))
        .route("/api/fermentation/profiles", get(crate::fermentation::get_profiles))
        .route("/api/fermentation", post(crate::fermentation::create_fermentation))
        .route("/api/users/profile", post(crate::users::update_profile))
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
