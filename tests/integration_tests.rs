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
    let app_state = create_test_app_state().await;
    raugupatis_log::create_router(app_state).await
}

async fn create_test_app_state() -> AppState {
    // Create a test database with a unique temporary file
    let temp_dir = std::env::temp_dir();
    let test_db_path = temp_dir.join(format!("test_raugupatis_{}.db", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()))
        .to_string_lossy()
        .to_string();
    
    let config = Arc::new(AppConfig {
        server_address: "0.0.0.0:3000".to_string(),
        database_url: test_db_path,
        environment: "test".to_string(),
        session_secret: "test-secret".to_string(),
    });

    let db = Arc::new(Database::new(&config.database_url).await.unwrap());
    db.migrate().await.unwrap();

    AppState {
        db,
        config: config.clone(),
    }
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
    // Use shared app state for both requests to test duplicate detection
    let app_state = create_test_app_state().await;
    
    let request_body = json!({
        "email": "duplicate@example.com",
        "password": "securepassword123"
    });

    // First registration should succeed
    let app1 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app1
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

    // Second registration with same email should fail with conflict
    let app2 = raugupatis_log::create_router(app_state).await;
    let response = app2
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
#[tokio::test]
async fn test_login_success() {
    let app_state = create_test_app_state().await;
    
    // First register a user
    let register_body = json!({
        "email": "logintest@example.com",
        "password": "securepassword123"
    });

    let app1 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app1
        .oneshot(
            Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&register_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Now try to login with correct credentials
    let login_body = json!({
        "email": "logintest@example.com",
        "password": "securepassword123"
    });

    let app2 = raugupatis_log::create_router(app_state).await;
    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/users/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["success"], true);
    assert_eq!(body_json["user"]["email"], "logintest@example.com");
    assert_eq!(body_json["message"], "Login successful");
}

#[tokio::test]
async fn test_login_wrong_password() {
    let app_state = create_test_app_state().await;
    
    // First register a user
    let register_body = json!({
        "email": "wrongpass@example.com",
        "password": "correctpassword123"
    });

    let app1 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app1
        .oneshot(
            Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&register_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Try to login with wrong password
    let login_body = json!({
        "email": "wrongpass@example.com",
        "password": "wrongpassword123"
    });

    let app2 = raugupatis_log::create_router(app_state).await;
    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/users/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["success"], false);
    assert_eq!(body_json["message"], "Invalid email or password");
}

#[tokio::test]
async fn test_login_nonexistent_user() {
    let app = create_test_app().await;

    let login_body = json!({
        "email": "nonexistent@example.com",
        "password": "somepassword123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/users/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["success"], false);
    assert_eq!(body_json["message"], "Invalid email or password");
}

#[tokio::test]
async fn test_register_page_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(Request::builder().uri("/register").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_login_invalid_email() {
    let app = create_test_app().await;

    let login_body = json!({
        "email": "not-an-email",
        "password": "somepassword123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/users/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["success"], false);
    assert_eq!(body_json["message"], "Invalid email format");
}

#[tokio::test]
async fn test_login_page_loads() {
    let app = create_test_app().await;

    let response = app
        .oneshot(Request::builder().uri("/login").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_dashboard_page_loads() {
    let app = create_test_app().await;

    let response = app
        .oneshot(Request::builder().uri("/dashboard").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Dashboard should redirect to login when not authenticated
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_logout() {
    let app_state = create_test_app_state().await;
    
    // First register a user
    let register_body = json!({
        "email": "logouttest@example.com",
        "password": "securepassword123"
    });

    let app1 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app1
        .oneshot(
            Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&register_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Login
    let login_body = json!({
        "email": "logouttest@example.com",
        "password": "securepassword123"
    });

    let app2 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/users/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    // Extract session cookie
    let cookies = response.headers().get("set-cookie");
    assert!(cookies.is_some(), "Login should set a session cookie");

    // Logout
    let app3 = raugupatis_log::create_router(app_state).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/users/logout")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["success"], true);
    assert_eq!(body_json["message"], "Logout successful");
}

#[tokio::test]
async fn test_register_user_with_experience_level() {
    let app = create_test_app().await;

    let request_body = json!({
        "email": "experienced@example.com",
        "password": "securepassword123",
        "experience_level": "advanced"
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

    assert_eq!(body_json["email"], "experienced@example.com");
    assert_eq!(body_json["experience_level"], "advanced");
}

#[tokio::test]
async fn test_register_user_default_experience_level() {
    let app = create_test_app().await;

    // Register without specifying experience level
    let request_body = json!({
        "email": "default@example.com",
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

    assert_eq!(body_json["email"], "default@example.com");
    // Default should be "beginner"
    assert_eq!(body_json["experience_level"], "beginner");
}

#[tokio::test]
async fn test_login_returns_experience_level() {
    let app_state = create_test_app_state().await;
    
    // Register with intermediate experience level
    let register_body = json!({
        "email": "intermediate@example.com",
        "password": "securepassword123",
        "experience_level": "intermediate"
    });

    let app1 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app1
        .oneshot(
            Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&register_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Login and check if experience level is returned
    let login_body = json!({
        "email": "intermediate@example.com",
        "password": "securepassword123"
    });

    let app2 = raugupatis_log::create_router(app_state).await;
    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/users/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["success"], true);
    assert_eq!(body_json["user"]["email"], "intermediate@example.com");
    assert_eq!(body_json["user"]["experience_level"], "intermediate");
}
