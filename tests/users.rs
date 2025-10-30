mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_register_user_success() {
    let app = common::create_test_app().await;

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
    let app_state = common::create_test_app_state().await;
    
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
    let app = common::create_test_app().await;

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
    let app = common::create_test_app().await;

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
    let app_state = common::create_test_app_state().await;
    
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
    let app_state = common::create_test_app_state().await;
    
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
    let app = common::create_test_app().await;

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
    let app = common::create_test_app().await;

    let response = app
        .oneshot(Request::builder().uri("/register").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_login_invalid_email() {
    let app = common::create_test_app().await;

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
    let app = common::create_test_app().await;

    let response = app
        .oneshot(Request::builder().uri("/login").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_logout() {
    let app_state = common::create_test_app_state().await;
    
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
async fn test_register_user_default_experience_level() {
    let app = common::create_test_app().await;

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
    let app_state = common::create_test_app_state().await;
    
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

#[tokio::test]
async fn test_profile_page_redirect_when_not_authenticated() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(Request::builder().uri("/profile").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Profile page should redirect to login when not authenticated
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    
    // Verify redirect location
    let location = response.headers().get("location");
    assert!(location.is_some());
    assert_eq!(location.unwrap(), "/login");
}

#[tokio::test]
async fn test_update_profile_success() {
    let app_state = common::create_test_app_state().await;
    
    // First register a user
    let register_body = json!({
        "email": "profiletest@example.com",
        "password": "securepassword123",
        "experience_level": "beginner"
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

    // Login to get session
    let login_body = json!({
        "email": "profiletest@example.com",
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
    let cookie_value = cookies.unwrap().to_str().unwrap();

    // Update profile with session cookie
    let update_body = json!({
        "experience_level": "advanced"
    });

    let app3 = raugupatis_log::create_router(app_state).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/users/profile")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_value)
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["email"], "profiletest@example.com");
    assert_eq!(body_json["experience_level"], "advanced");
}

#[tokio::test]
async fn test_update_profile_unauthorized() {
    let app = common::create_test_app().await;

    let update_body = json!({
        "experience_level": "advanced"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/users/profile")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return unauthorized when no session
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_update_profile_invalid_experience_level() {
    let app_state = common::create_test_app_state().await;
    
    // Register and login
    let register_body = json!({
        "email": "invalidexp@example.com",
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

    let login_body = json!({
        "email": "invalidexp@example.com",
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
    
    let cookies = response.headers().get("set-cookie");
    assert!(cookies.is_some());
    let cookie_value = cookies.unwrap().to_str().unwrap();

    // Try to update with invalid experience level
    let update_body = json!({
        "experience_level": "invalid"
    });

    let app3 = raugupatis_log::create_router(app_state).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/users/profile")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_value)
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_user_with_first_and_last_name() {
    let app = common::create_test_app().await;

    let request_body = json!({
        "email": "nameduser@example.com",
        "password": "securepassword123",
        "first_name": "John",
        "last_name": "Doe"
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

    assert_eq!(body_json["email"], "nameduser@example.com");
    assert_eq!(body_json["first_name"], "John");
    assert_eq!(body_json["last_name"], "Doe");
}

#[tokio::test]
async fn test_register_user_without_names() {
    let app = common::create_test_app().await;

    let request_body = json!({
        "email": "noname@example.com",
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

    assert_eq!(body_json["email"], "noname@example.com");
    // Names should be null when not provided
    assert!(body_json["first_name"].is_null());
    assert!(body_json["last_name"].is_null());
}

#[tokio::test]
async fn test_update_profile_with_names() {
    let app_state = common::create_test_app_state().await;
    
    // Register a user without names
    let register_body = json!({
        "email": "updatenames@example.com",
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

    // Login to get session
    let login_body = json!({
        "email": "updatenames@example.com",
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
    
    let cookies = response.headers().get("set-cookie");
    assert!(cookies.is_some());
    let cookie_value = cookies.unwrap().to_str().unwrap();

    // Update profile with names
    let update_body = json!({
        "experience_level": "intermediate",
        "first_name": "Jane",
        "last_name": "Smith"
    });

    let app3 = raugupatis_log::create_router(app_state).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/users/profile")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_value)
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["email"], "updatenames@example.com");
    assert_eq!(body_json["experience_level"], "intermediate");
    assert_eq!(body_json["first_name"], "Jane");
    assert_eq!(body_json["last_name"], "Smith");
}

#[tokio::test]
async fn test_login_returns_user_names() {
    let app_state = common::create_test_app_state().await;
    
    // Register with names
    let register_body = json!({
        "email": "withnames@example.com",
        "password": "securepassword123",
        "first_name": "Alice",
        "last_name": "Wonder"
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

    // Login and verify names are returned
    let login_body = json!({
        "email": "withnames@example.com",
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
    assert_eq!(body_json["user"]["email"], "withnames@example.com");
    assert_eq!(body_json["user"]["first_name"], "Alice");
    assert_eq!(body_json["user"]["last_name"], "Wonder");
}

#[tokio::test]
async fn test_login_without_remember_me() {
    let app_state = common::create_test_app_state().await;
    
    // Register a user
    let register_body = json!({
        "email": "noremember@example.com",
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

    // Login without remember_me (defaults to false)
    let login_body = json!({
        "email": "noremember@example.com",
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
    assert_eq!(body_json["user"]["email"], "noremember@example.com");
}

#[tokio::test]
async fn test_login_with_remember_me_true() {
    let app_state = common::create_test_app_state().await;
    
    // Register a user
    let register_body = json!({
        "email": "rememberme@example.com",
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

    // Login with remember_me set to true
    let login_body = json!({
        "email": "rememberme@example.com",
        "password": "securepassword123",
        "remember_me": true
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

    // Extract cookie headers before consuming the response
    let cookies = response.headers().get("set-cookie");
    assert!(cookies.is_some(), "Login with remember_me should set a session cookie");

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["success"], true);
    assert_eq!(body_json["user"]["email"], "rememberme@example.com");
}

#[tokio::test]
async fn test_login_with_remember_me_false() {
    let app_state = common::create_test_app_state().await;
    
    // Register a user
    let register_body = json!({
        "email": "dontremember@example.com",
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

    // Login with remember_me explicitly set to false
    let login_body = json!({
        "email": "dontremember@example.com",
        "password": "securepassword123",
        "remember_me": false
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
    assert_eq!(body_json["user"]["email"], "dontremember@example.com");
}
