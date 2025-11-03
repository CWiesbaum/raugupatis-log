mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_upload_photo_unauthorized() {
    let app = common::create_test_app().await;

    // Create a simple multipart body
    let boundary = "----boundary";
    let body_content = format!(
        "--{}\r\nContent-Disposition: form-data; name=\"photo\"; filename=\"test.jpg\"\r\nContent-Type: image/jpeg\r\n\r\nfake-image-data\r\n--{}--\r\n",
        boundary, boundary
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/fermentation/1/photos")
                .method("POST")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body_content))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 when not authenticated
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_photos_unauthorized() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/fermentation/1/photos")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 when not authenticated
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_upload_photo_to_nonexistent_fermentation() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "photouploader@example.com",
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
        "email": "photouploader@example.com",
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

    let cookie_header = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("Login should set a session cookie");

    // Try to upload photo to non-existent fermentation
    let boundary = "----boundary";
    let body_content = format!(
        "--{}\r\nContent-Disposition: form-data; name=\"photo\"; filename=\"test.jpg\"\r\nContent-Type: image/jpeg\r\n\r\nfake-image-data\r\n--{}--\r\n",
        boundary, boundary
    );

    let app3 = raugupatis_log::create_router(app_state).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/fermentation/99999/photos")
                .method("POST")
                .header("Cookie", cookie_header)
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(body_content))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 404 for non-existent fermentation
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_photos_empty() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "photoviewer@example.com",
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
        "email": "photoviewer@example.com",
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

    let cookie_header = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("Login should set a session cookie");

    // Create a fermentation
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "Test Fermentation for Photos",
        "start_date": "2024-01-15T10:00:00Z",
    });

    let app3 = raugupatis_log::create_router(app_state.clone()).await;
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
    let fermentation: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let fermentation_id = fermentation["id"].as_i64().unwrap();

    // List photos for fermentation (should be empty)
    let app4 = raugupatis_log::create_router(app_state).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/photos", fermentation_id))
                .header("Cookie", cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let photos: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    // Should be empty
    assert_eq!(photos.len(), 0);
}

#[tokio::test]
async fn test_cannot_access_other_users_fermentation_photos() {
    let app_state = common::create_test_app_state().await;

    // Register and login first user
    let register_body = json!({
        "email": "user1photos@example.com",
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
        "email": "user1photos@example.com",
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

    let user1_cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("Login should set a session cookie");

    // Create a fermentation as user1
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "User1 Fermentation",
        "start_date": "2024-01-15T10:00:00Z",
    });

    let app3 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/fermentation")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", user1_cookie)
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
    let fermentation: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let fermentation_id = fermentation["id"].as_i64().unwrap();

    // Register and login second user
    let register_body2 = json!({
        "email": "user2photos@example.com",
        "password": "securepassword123"
    });

    let app4 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&register_body2).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let login_body2 = json!({
        "email": "user2photos@example.com",
        "password": "securepassword123"
    });

    let app5 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app5
        .oneshot(
            Request::builder()
                .uri("/api/users/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&login_body2).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let user2_cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("Login should set a session cookie");

    // Try to list photos from user1's fermentation as user2
    let app6 = raugupatis_log::create_router(app_state).await;
    let response = app6
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/photos", fermentation_id))
                .header("Cookie", user2_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 404 when trying to access another user's fermentation
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
