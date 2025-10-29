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
    raugupatis_log::create_router(app_state)
}

async fn create_test_app_state() -> AppState {
    // Create a test database in memory or temporary location
    let config = Arc::new(AppConfig {
        server_address: "0.0.0.0:3000".to_string(),
        database_url: "sqlite::memory:".to_string(),
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
    let app1 = raugupatis_log::create_router(app_state.clone());
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
    let app2 = raugupatis_log::create_router(app_state);
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

    let app1 = raugupatis_log::create_router(app_state.clone());
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

    let app2 = raugupatis_log::create_router(app_state);
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

    let app1 = raugupatis_log::create_router(app_state.clone());
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

    let app2 = raugupatis_log::create_router(app_state);
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

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_home_page_shows_login_button_when_not_logged_in() {
    let app = create_test_app().await;

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should show Login button when not logged in
    assert!(body_str.contains("Login"));
    assert!(body_str.contains("/login"));
}

#[tokio::test]
async fn test_home_page_hides_login_button_when_logged_in() {
    use axum::http::header;
    use tower::ServiceExt;
    
    let app_state = create_test_app_state().await;
    let app = raugupatis_log::create_router(app_state);
    
    // First register a user
    let register_body = json!({
        "email": "hometest@example.com",
        "password": "securepassword123"
    });

    let response = app
        .clone()
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

    // Now login with correct credentials
    let login_body = json!({
        "email": "hometest@example.com",
        "password": "securepassword123"
    });

    let login_response = app
        .clone()
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

    assert_eq!(login_response.status(), StatusCode::OK);

    // Extract the session cookie from the response
    let cookies = login_response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .collect::<Vec<_>>();

    assert!(!cookies.is_empty(), "Should have session cookie");

    // Now request the home page with the session cookie
    let mut home_request = Request::builder().uri("/").body(Body::empty()).unwrap();
    
    // Add session cookies to the request
    let cookie_header = cookies
        .iter()
        .map(|c| c.split(';').next().unwrap_or(c))
        .collect::<Vec<_>>()
        .join("; ");
    home_request.headers_mut().insert(
        header::COOKIE,
        cookie_header.parse().unwrap(),
    );

    let home_response = app
        .oneshot(home_request)
        .await
        .unwrap();

    assert_eq!(home_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(home_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Should NOT show Login button link when logged in
    assert!(!body_str.contains(r#"href="/login""#));
    // Should show Dashboard button instead
    assert!(body_str.contains("Go to Dashboard"));
    assert!(body_str.contains("/dashboard"));
}

