use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use raugupatis_log::{config::AppConfig, database::Database, AppState};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

async fn create_test_app() -> Router {
    // Create a test database in memory or temporary location
    let config = Arc::new(AppConfig {
        server_address: "0.0.0.0:3000".to_string(),
        database_url: "sqlite::memory:".to_string(),
        environment: "test".to_string(),
        session_secret: "test-secret".to_string(),
    });

    let db = Arc::new(Database::new(&config.database_url).await.unwrap());
    db.migrate().await.unwrap();

    let app_state = AppState {
        db,
        config: config.clone(),
    };

    raugupatis_log::create_router(app_state)
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_home_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_register_user_success() {
    let app = create_test_app().await;

    let request_body = json!({
        "email": "test@example.com",
        "password": "securepassword123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["email"], "test@example.com");
    assert!(body_json["id"].as_i64().is_some());
}

#[tokio::test]
async fn test_register_user_duplicate_email() {
    let app = create_test_app().await;

    let request_body = json!({
        "email": "duplicate@example.com",
        "password": "securepassword123"
    });

    // First registration should succeed
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Second registration should fail with conflict
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_register_user_invalid_email() {
    let app = create_test_app().await;

    let request_body = json!({
        "email": "not-an-email",
        "password": "securepassword123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_user_short_password() {
    let app = create_test_app().await;

    let request_body = json!({
        "email": "test@example.com",
        "password": "short"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}