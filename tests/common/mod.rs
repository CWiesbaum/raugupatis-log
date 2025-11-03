use axum::Router;
use raugupatis_log::{config::AppConfig, database::Database, AppState};
use std::sync::Arc;

/// Creates a test app with a fresh database for integration testing
#[allow(dead_code)]
pub async fn create_test_app() -> Router {
    let app_state = create_test_app_state().await;
    raugupatis_log::create_router(app_state).await
}

/// Creates a test app state with a unique temporary database
pub async fn create_test_app_state() -> AppState {
    // Create a test database with a unique temporary file
    let temp_dir = std::env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let test_db_path = temp_dir
        .join(format!("test_raugupatis_{}.db", timestamp))
        .to_string_lossy()
        .to_string();

    let test_uploads_dir = temp_dir
        .join(format!("test_uploads_{}", timestamp))
        .to_string_lossy()
        .to_string();

    let config = Arc::new(AppConfig {
        server_address: "0.0.0.0:3000".to_string(),
        database_url: test_db_path,
        environment: "test".to_string(),
        session_secret: "test-secret".to_string(),
        uploads_dir: test_uploads_dir,
    });

    let db = Arc::new(Database::new(&config.database_url).await.unwrap());
    db.migrate().await.unwrap();

    AppState {
        db,
        config: config.clone(),
    }
}
