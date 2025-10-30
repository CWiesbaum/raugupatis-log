mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_fermentation_list_page_redirects_when_not_logged_in() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/fermentations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should redirect to login when not authenticated
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_fermentation_list_api_unauthorized() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/fermentations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // API should return 401 when not authenticated
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_fermentation_profiles() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/fermentation/profiles")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let profiles: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Should have the default profiles from migration
    assert!(!profiles.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_create_fermentation_success() {
    let app_state = common::create_test_app_state().await;

    // First register and login a user
    let register_body = json!({
        "email": "fermenter@example.com",
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
        "email": "fermenter@example.com",
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
    let cookie_header = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("Login should set a session cookie");

    // Create fermentation
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "My First Pickles",
        "start_date": "2024-01-15T10:00:00Z",
        "target_end_date": "2024-01-22T10:00:00Z",
        "notes": "Using grandma's recipe",
        "ingredients": "cucumbers, salt, water, dill"
    });

    let app3 = raugupatis_log::create_router(app_state).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/fermentation")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(
                    serde_json::to_string(&fermentation_body).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body_json["name"], "My First Pickles");
    assert_eq!(body_json["profile_id"], 1);
    assert!(body_json["id"].as_i64().is_some());
}

#[tokio::test]
async fn test_create_fermentation_unauthorized() {
    let app = common::create_test_app().await;

    let fermentation_body = json!({
        "profile_id": 1,
        "name": "My First Pickles",
        "start_date": "2024-01-15T10:00:00Z",
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/fermentation")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&fermentation_body).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should fail without authentication
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_fermentation_invalid_profile() {
    let app_state = common::create_test_app_state().await;

    // First register and login a user
    let register_body = json!({
        "email": "fermenter2@example.com",
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
        "email": "fermenter2@example.com",
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
    let cookie_header = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("Login should set a session cookie");

    // Try to create fermentation with invalid profile
    let fermentation_body = json!({
        "profile_id": 99999,
        "name": "Invalid Profile Test",
        "start_date": "2024-01-15T10:00:00Z",
    });

    let app3 = raugupatis_log::create_router(app_state).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/fermentation")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(
                    serde_json::to_string(&fermentation_body).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_fermentation_empty_name() {
    let app_state = common::create_test_app_state().await;

    // First register and login a user
    let register_body = json!({
        "email": "fermenter3@example.com",
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
        "email": "fermenter3@example.com",
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
    let cookie_header = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("Login should set a session cookie");

    // Try to create fermentation with empty name
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "   ",
        "start_date": "2024-01-15T10:00:00Z",
    });

    let app3 = raugupatis_log::create_router(app_state).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/fermentation")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(
                    serde_json::to_string(&fermentation_body).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_new_fermentation_page_requires_auth() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/fermentation/new")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should redirect to login when not authenticated
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}
