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

#[tokio::test]
async fn test_fermentation_detail_page_redirects_when_not_logged_in() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/fermentation/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should redirect to login when not authenticated
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_fermentation_detail_page_redirects_when_not_found() {
    let app_state = common::create_test_app_state().await;

    // First register and login a user
    let register_body = json!({
        "email": "detailviewer@example.com",
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
        "email": "detailviewer@example.com",
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

    // Try to access a non-existent fermentation
    let app3 = raugupatis_log::create_router(app_state).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/fermentation/999999")
                .header("Cookie", cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should redirect to fermentations list when not found
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_fermentation_detail_page_displays_correctly() {
    let app_state = common::create_test_app_state().await;

    // First register and login a user
    let register_body = json!({
        "email": "detailtest@example.com",
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
        "email": "detailtest@example.com",
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

    // Create a fermentation
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "Test Pickles for Detail View",
        "start_date": "2024-01-15T10:00:00Z",
        "target_end_date": "2024-01-22T10:00:00Z",
        "notes": "Testing the detail view",
        "ingredients": "cucumbers, salt, water"
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

    // Parse response to get fermentation ID
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let fermentation: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let fermentation_id = fermentation["id"].as_i64().unwrap();

    // Now access the detail page
    let app4 = raugupatis_log::create_router(app_state).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri(format!("/fermentation/{}", fermentation_id))
                .header("Cookie", cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should successfully display the detail page
    assert_eq!(response.status(), StatusCode::OK);

    // Verify the page contains expected content
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    // Check for key elements in the HTML
    assert!(html.contains("Test Pickles for Detail View"));
    assert!(html.contains("Testing the detail view"));
    assert!(html.contains("cucumbers, salt, water"));
    assert!(html.contains("Timeline"));
}

#[tokio::test]
async fn test_fermentation_detail_cannot_access_other_users_fermentation() {
    let app_state = common::create_test_app_state().await;

    // Register and login first user
    let register_body = json!({
        "email": "user1@example.com",
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
        "email": "user1@example.com",
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
        "name": "User1 Pickles",
        "start_date": "2024-01-15T10:00:00Z",
        "notes": "Private to user1"
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
        "email": "user2@example.com",
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
        "email": "user2@example.com",
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

    // Try to access user1's fermentation as user2
    let app6 = raugupatis_log::create_router(app_state).await;
    let response = app6
        .oneshot(
            Request::builder()
                .uri(format!("/fermentation/{}", fermentation_id))
                .header("Cookie", user2_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should redirect to fermentations list when trying to access another user's fermentation
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_update_fermentation_success() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "updater@example.com",
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
        "email": "updater@example.com",
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
        "name": "Original Pickles",
        "start_date": "2024-01-15T10:00:00Z",
        "notes": "Original notes"
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

    // Update the fermentation
    let update_body = json!({
        "name": "Updated Pickles",
        "status": "completed",
        "success_rating": 5,
        "notes": "Updated notes - turned out great!"
    });

    let app4 = raugupatis_log::create_router(app_state).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}", fermentation_id))
                .method("PUT")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(updated["name"], "Updated Pickles");
    assert_eq!(updated["status"], "completed");
}

#[tokio::test]
async fn test_update_fermentation_unauthorized() {
    let app = common::create_test_app().await;

    let update_body = json!({
        "name": "Should Not Work",
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/fermentation/1")
                .method("PUT")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_update_fermentation_not_found() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "notfound@example.com",
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
        "email": "notfound@example.com",
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

    // Try to update a non-existent fermentation
    let update_body = json!({
        "name": "Should Not Work",
    });

    let app3 = raugupatis_log::create_router(app_state).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/fermentation/999999")
                .method("PUT")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_fermentation_invalid_name() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "invalidname@example.com",
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
        "email": "invalidname@example.com",
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
        "name": "Valid Name",
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

    // Try to update with empty name
    let update_body = json!({
        "name": "   ",
    });

    let app4 = raugupatis_log::create_router(app_state).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}", fermentation_id))
                .method("PUT")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_fermentation_invalid_status() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "invalidstatus@example.com",
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
        "email": "invalidstatus@example.com",
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
        "name": "Valid Name",
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

    // Try to update with invalid status
    let update_body = json!({
        "status": "invalid_status",
    });

    let app4 = raugupatis_log::create_router(app_state).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}", fermentation_id))
                .method("PUT")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_fermentation_invalid_success_rating() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "invalidrating@example.com",
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
        "email": "invalidrating@example.com",
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
        "name": "Valid Name",
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

    // Try to update with invalid success rating (out of 1-5 range)
    let update_body = json!({
        "success_rating": 10,
    });

    let app4 = raugupatis_log::create_router(app_state).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}", fermentation_id))
                .method("PUT")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_fermentation_cannot_update_other_users_fermentation() {
    let app_state = common::create_test_app_state().await;

    // Register and login first user
    let register_body1 = json!({
        "email": "owner@example.com",
        "password": "securepassword123"
    });

    let app1 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app1
        .oneshot(
            Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&register_body1).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let login_body1 = json!({
        "email": "owner@example.com",
        "password": "securepassword123"
    });

    let app2 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/users/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&login_body1).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let owner_cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("Login should set a session cookie");

    // Create a fermentation as first user
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "Owner's Pickles",
        "start_date": "2024-01-15T10:00:00Z",
    });

    let app3 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/fermentation")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", owner_cookie)
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
        "email": "hacker@example.com",
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
        "email": "hacker@example.com",
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

    let hacker_cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .expect("Login should set a session cookie");

    // Try to update first user's fermentation as second user
    let update_body = json!({
        "name": "Hacked!",
    });

    let app6 = raugupatis_log::create_router(app_state).await;
    let response = app6
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}", fermentation_id))
                .method("PUT")
                .header("Content-Type", "application/json")
                .header("Cookie", hacker_cookie)
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return NOT_FOUND (not revealing that it exists but belongs to another user)
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_edit_fermentation_page_requires_auth() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/fermentation/1/edit")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should redirect to login when not authenticated
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_edit_fermentation_page_redirects_when_not_found() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "editviewer@example.com",
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
        "email": "editviewer@example.com",
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

    // Try to access edit page for non-existent fermentation
    let app3 = raugupatis_log::create_router(app_state).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/fermentation/999999/edit")
                .header("Cookie", cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should redirect to fermentations list when not found
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_fermentation_list_includes_thumbnails() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "thumbnails@example.com",
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
        "email": "thumbnails@example.com",
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

    // Create an active fermentation
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "Active Fermentation with Photos",
        "start_date": "2024-01-15T10:00:00Z",
        "target_end_date": "2024-01-25T10:00:00Z",
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

    // Add a "start" stage photo
    use raugupatis_log::photos::{PhotoRepository, PhotoStage};
    use chrono::Utc;
    
    let photo_repo = PhotoRepository::new(app_state.db.clone());
    let _photo = photo_repo
        .create_photo(
            fermentation_id,
            "/uploads/test_start.jpg".to_string(),
            Some("Start photo".to_string()),
            Utc::now(),
            PhotoStage::Start,
        )
        .await
        .unwrap();

    // Add a "progress" stage photo (should not be used as thumbnail)
    let _photo2 = photo_repo
        .create_photo(
            fermentation_id,
            "/uploads/test_progress.jpg".to_string(),
            Some("Progress photo".to_string()),
            Utc::now(),
            PhotoStage::Progress,
        )
        .await
        .unwrap();

    // Fetch fermentations via API
    let app4 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri("/api/fermentations")
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
    let fermentations: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    // Should have the fermentation with thumbnail_path
    assert_eq!(fermentations.len(), 1);
    let ferm = &fermentations[0];
    assert_eq!(ferm["thumbnail_path"], "/uploads/test_start.jpg");

    // Now create a completed fermentation
    let fermentation_body2 = json!({
        "profile_id": 1,
        "name": "Completed Fermentation with Photos",
        "start_date": "2024-01-01T10:00:00Z",
        "target_end_date": "2024-01-10T10:00:00Z",
    });

    let app5 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app5
        .oneshot(
            Request::builder()
                .uri("/api/fermentation")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(
                    serde_json::to_string(&fermentation_body2).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let fermentation2: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let fermentation_id2 = fermentation2["id"].as_i64().unwrap();

    // Add "start" and "end" photos to completed fermentation
    let _photo3 = photo_repo
        .create_photo(
            fermentation_id2,
            "/uploads/test_completed_start.jpg".to_string(),
            Some("Completed start photo".to_string()),
            Utc::now(),
            PhotoStage::Start,
        )
        .await
        .unwrap();

    let _photo4 = photo_repo
        .create_photo(
            fermentation_id2,
            "/uploads/test_completed_end.jpg".to_string(),
            Some("Completed end photo".to_string()),
            Utc::now(),
            PhotoStage::End,
        )
        .await
        .unwrap();

    // Mark as completed
    let update_body = json!({
        "status": "completed",
        "actual_end_date": "2024-01-10T10:00:00Z",
    });

    let app6 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app6
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}", fermentation_id2))
                .method("PUT")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Fetch fermentations again
    let app7 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app7
        .oneshot(
            Request::builder()
                .uri("/api/fermentations")
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
    let fermentations: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    // Should have 2 fermentations
    assert_eq!(fermentations.len(), 2);

    // Find the completed fermentation (should be first due to created_at DESC order)
    let completed_ferm = fermentations
        .iter()
        .find(|f| f["status"] == "completed")
        .expect("Should have a completed fermentation");

    // Completed fermentation should use "end" photo as thumbnail
    assert_eq!(
        completed_ferm["thumbnail_path"],
        "/uploads/test_completed_end.jpg"
    );
}

#[tokio::test]
async fn test_fermentation_list_search_by_name() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "search_test@example.com",
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
        "email": "search_test@example.com",
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

    // Create multiple fermentations with different names
    let fermentation1 = json!({
        "profile_id": 1,
        "name": "Spicy Pickles",
        "start_date": "2024-01-15T10:00:00Z",
    });

    let fermentation2 = json!({
        "profile_id": 1,
        "name": "Sweet Pickles",
        "start_date": "2024-01-16T10:00:00Z",
    });

    let fermentation3 = json!({
        "profile_id": 1,
        "name": "Kimchi Batch",
        "start_date": "2024-01-17T10:00:00Z",
    });

    for ferm_body in [fermentation1, fermentation2, fermentation3] {
        let app = raugupatis_log::create_router(app_state.clone()).await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/fermentation")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .header("Cookie", cookie_header)
                    .body(Body::from(serde_json::to_string(&ferm_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Search for "Pickles"
    let app4 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri("/api/fermentations?search=Pickles")
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
    let fermentations: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    // Should only return the two fermentations with "Pickles" in the name
    assert_eq!(fermentations.len(), 2);
    assert!(fermentations.iter().all(|f| f["name"]
        .as_str()
        .unwrap()
        .contains("Pickles")));
}

#[tokio::test]
async fn test_fermentation_list_filter_by_status() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "filter_status@example.com",
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
        "email": "filter_status@example.com",
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

    // Create fermentations with different statuses
    let fermentation1 = json!({
        "profile_id": 1,
        "name": "Active Fermentation",
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
                .body(Body::from(serde_json::to_string(&fermentation1).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let ferm1: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let ferm1_id = ferm1["id"].as_i64().unwrap();

    // Create another fermentation and mark it as completed
    let fermentation2 = json!({
        "profile_id": 1,
        "name": "Completed Fermentation",
        "start_date": "2024-01-10T10:00:00Z",
    });

    let app4 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri("/api/fermentation")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(serde_json::to_string(&fermentation2).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let ferm2: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let ferm2_id = ferm2["id"].as_i64().unwrap();

    // Update second fermentation to completed
    let update_body = json!({
        "status": "completed",
        "actual_end_date": "2024-01-17T10:00:00Z",
    });

    let app5 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app5
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}", ferm2_id))
                .method("PUT")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Filter by active status
    let app6 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app6
        .oneshot(
            Request::builder()
                .uri("/api/fermentations?status=active")
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
    let fermentations: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    // Should only return the active fermentation
    assert_eq!(fermentations.len(), 1);
    assert_eq!(fermentations[0]["status"], "active");
    assert_eq!(fermentations[0]["id"], ferm1_id);

    // Filter by completed status
    let app7 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app7
        .oneshot(
            Request::builder()
                .uri("/api/fermentations?status=completed")
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
    let fermentations: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    // Should only return the completed fermentation
    assert_eq!(fermentations.len(), 1);
    assert_eq!(fermentations[0]["status"], "completed");
    assert_eq!(fermentations[0]["id"], ferm2_id);
}

#[tokio::test]
async fn test_fermentation_list_sort_by_name() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "sort_test@example.com",
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
        "email": "sort_test@example.com",
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

    // Create fermentations with different names
    let names = ["Zebra Pickles", "Apple Kimchi", "Mango Kombucha"];

    for name in names {
        let fermentation = json!({
            "profile_id": 1,
            "name": name,
            "start_date": "2024-01-15T10:00:00Z",
        });

        let app = raugupatis_log::create_router(app_state.clone()).await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/fermentation")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .header("Cookie", cookie_header)
                    .body(Body::from(serde_json::to_string(&fermentation).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Sort by name ascending
    let app3 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/fermentations?sort_by=name&sort_order=asc")
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
    let fermentations: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert_eq!(fermentations.len(), 3);
    assert_eq!(fermentations[0]["name"], "Apple Kimchi");
    assert_eq!(fermentations[1]["name"], "Mango Kombucha");
    assert_eq!(fermentations[2]["name"], "Zebra Pickles");

    // Sort by name descending
    let app4 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri("/api/fermentations?sort_by=name&sort_order=desc")
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
    let fermentations: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert_eq!(fermentations.len(), 3);
    assert_eq!(fermentations[0]["name"], "Zebra Pickles");
    assert_eq!(fermentations[1]["name"], "Mango Kombucha");
    assert_eq!(fermentations[2]["name"], "Apple Kimchi");
}

#[tokio::test]
async fn test_fermentation_list_combined_search_and_filter() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "combined_test@example.com",
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
        "email": "combined_test@example.com",
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

    // Create multiple fermentations
    let fermentations = [
        ("Active Pickles", "active"),
        ("Active Kimchi", "active"),
        ("Completed Pickles", "active"),
    ];

    let mut created_ids = Vec::new();

    for (name, _status) in fermentations {
        let fermentation = json!({
            "profile_id": 1,
            "name": name,
            "start_date": "2024-01-15T10:00:00Z",
        });

        let app = raugupatis_log::create_router(app_state.clone()).await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/fermentation")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .header("Cookie", cookie_header)
                    .body(Body::from(serde_json::to_string(&fermentation).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let ferm: serde_json::Value = serde_json::from_slice(&body).unwrap();
        created_ids.push(ferm["id"].as_i64().unwrap());
    }

    // Update the third fermentation to completed
    let update_body = json!({
        "status": "completed",
        "actual_end_date": "2024-01-22T10:00:00Z",
    });

    let app3 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}", created_ids[2]))
                .method("PUT")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Search for "Pickles" AND filter by status "active"
    let app4 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri("/api/fermentations?search=Pickles&status=active")
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
    let fermentations: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    // Should only return "Active Pickles" (not "Completed Pickles" or "Active Kimchi")
    assert_eq!(fermentations.len(), 1);
    assert_eq!(fermentations[0]["name"], "Active Pickles");
    assert_eq!(fermentations[0]["status"], "active");
}
#[tokio::test]
async fn test_create_temperature_log_success() {
    let app_state = common::create_test_app_state().await;

    // First register and login a user
    let register_body = json!({
        "email": "temp_logger@example.com",
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
        "email": "temp_logger@example.com",
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

    // Create fermentation
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "Test Fermentation for Temperature",
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

    // Add temperature log
    let temp_log_body = json!({
        "temperature": 72.5,
        "notes": "Temperature looking good"
    });

    let app4 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/temperature", fermentation_id))
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(serde_json::to_string(&temp_log_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let temp_log: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(temp_log["temperature"], 72.5);
    assert_eq!(temp_log["notes"], "Temperature looking good");
    assert!(temp_log["id"].as_i64().is_some());
}

#[tokio::test]
async fn test_create_temperature_log_unauthorized() {
    let app = common::create_test_app().await;

    let temp_log_body = json!({
        "temperature": 72.5,
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/fermentation/1/temperature")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&temp_log_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_temperature_logs_success() {
    let app_state = common::create_test_app_state().await;

    // First register and login a user
    let register_body = json!({
        "email": "temp_list@example.com",
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
        "email": "temp_list@example.com",
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

    // Create fermentation
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "Test Fermentation for Temperature List",
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

    // Add multiple temperature logs
    for (temp, note) in &[(70.0, "First reading"), (72.5, "Second reading"), (75.0, "Third reading")] {
        let temp_log_body = json!({
            "temperature": temp,
            "notes": note
        });

        let app = raugupatis_log::create_router(app_state.clone()).await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!("/api/fermentation/{}/temperature", fermentation_id))
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .header("Cookie", cookie_header)
                    .body(Body::from(serde_json::to_string(&temp_log_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // List temperature logs
    let app4 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/temperature", fermentation_id))
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
    let logs: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    // Should have 3 temperature logs
    assert_eq!(logs.len(), 3);
    
    // Logs should be ordered by recorded_at DESC (newest first)
    assert_eq!(logs[0]["temperature"], 75.0);
    assert_eq!(logs[1]["temperature"], 72.5);
    assert_eq!(logs[2]["temperature"], 70.0);
}

#[tokio::test]
async fn test_list_temperature_logs_unauthorized() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/fermentation/1/temperature")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_cannot_add_temperature_to_other_users_fermentation() {
    let app_state = common::create_test_app_state().await;

    // Register and login first user
    let register_body1 = json!({
        "email": "user1@example.com",
        "password": "password123"
    });

    let app1 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app1
        .oneshot(
            Request::builder()
                .uri("/api/users/register")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&register_body1).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let login_body1 = json!({
        "email": "user1@example.com",
        "password": "password123"
    });

    let app2 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app2
        .oneshot(
            Request::builder()
                .uri("/api/users/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&login_body1).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let cookie_user1 = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap();

    // Create fermentation as user1
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "User1's Fermentation",
        "start_date": "2024-01-15T10:00:00Z",
    });

    let app3 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/fermentation")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_user1)
                .body(Body::from(serde_json::to_string(&fermentation_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let fermentation: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let fermentation_id = fermentation["id"].as_i64().unwrap();

    // Register and login second user
    let register_body2 = json!({
        "email": "user2@example.com",
        "password": "password123"
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

    let login_body2 = json!({
        "email": "user2@example.com",
        "password": "password123"
    });

    let app5 = raugupatis_log::create_router(app_state.clone()).await;
    let _response = app5
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

    let cookie_user2 = _response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap();

    // Try to add temperature log to user1's fermentation as user2
    let temp_log_body = json!({
        "temperature": 72.5,
    });

    let app6 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app6
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/temperature", fermentation_id))
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_user2)
                .body(Body::from(serde_json::to_string(&temp_log_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return not found (fermentation doesn't exist for user2)
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_temperature_log_invalid_temperature() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "tempvalidation@example.com",
        "password": "password123"
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
        "email": "tempvalidation@example.com",
        "password": "password123"
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

    let cookie_header = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap();

    // Create fermentation
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "Test Fermentation",
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
                .body(Body::from(serde_json::to_string(&fermentation_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let fermentation: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let fermentation_id = fermentation["id"].as_i64().unwrap();

    // Try to add temperature log with negative temperature
    let temp_log_body = json!({
        "temperature": -10.0,
    });

    let app4 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/temperature", fermentation_id))
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(serde_json::to_string(&temp_log_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Try to add temperature log with temperature above max
    let temp_log_body = json!({
        "temperature": 200.0,
    });

    let app5 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app5
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/temperature", fermentation_id))
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(serde_json::to_string(&temp_log_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_temperature_log_with_celsius() {
    let app_state = common::create_test_app_state().await;

    // Register and login a user
    let register_body = json!({
        "email": "celsius_user@example.com",
        "password": "password123"
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
        "email": "celsius_user@example.com",
        "password": "password123"
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

    let cookie_header = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap();

    // Create fermentation
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "Test Fermentation",
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
                .body(Body::from(serde_json::to_string(&fermentation_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let fermentation: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let fermentation_id = fermentation["id"].as_i64().unwrap();

    // Add temperature log in Celsius (20°C should convert to ~68°F)
    let temp_log_body = json!({
        "temperature": 20.0,
        "temp_unit": "celsius",
        "notes": "Room temperature in Celsius"
    });

    let app4 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/temperature", fermentation_id))
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie_header)
                .body(Body::from(serde_json::to_string(&temp_log_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let temp_log: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify that the temperature was stored in Fahrenheit (20°C = 68°F)
    let stored_temp = temp_log["temperature"].as_f64().unwrap();
    assert!((stored_temp - 68.0).abs() < 0.1);
}

#[tokio::test]
async fn test_finish_fermentation_success() {
    let app_state = common::create_test_app_state().await;

    // Register and login
    let register_body = json!({
        "email": "finisher@example.com",
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
        "email": "finisher@example.com",
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

    let cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap();

    // Create a fermentation
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "Test Fermentation",
        "start_date": "2024-01-15T10:00:00Z",
    });

    let app3 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/fermentation")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&fermentation_body).unwrap()))
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

    // Finish the fermentation
    let finish_body = json!({
        "success_rating": 4,
        "taste_profile": "Crispy and tangy with a nice fermented flavor",
        "lessons_learned": "Next time, use less salt for a milder taste"
    });

    let app4 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/finish", fermentation_id))
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&finish_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let finished_fermentation: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify the fermentation is completed
    assert_eq!(finished_fermentation["status"], "completed");
}

#[tokio::test]
async fn test_finish_fermentation_unauthorized() {
    let app = common::create_test_app().await;

    let finish_body = json!({
        "success_rating": 4
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/fermentation/1/finish")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&finish_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_finish_fermentation_invalid_rating() {
    let app_state = common::create_test_app_state().await;

    // Register and login
    let register_body = json!({
        "email": "invalid@example.com",
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
        "email": "invalid@example.com",
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

    let cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap();

    // Create a fermentation
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "Test Fermentation",
        "start_date": "2024-01-15T10:00:00Z",
    });

    let app3 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/fermentation")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&fermentation_body).unwrap()))
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

    // Try to finish with invalid rating (> 5)
    let finish_body = json!({
        "success_rating": 10
    });

    let app4 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/finish", fermentation_id))
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&finish_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_taste_profile_success() {
    let app_state = common::create_test_app_state().await;

    // Register and login
    let register_body = json!({
        "email": "taster@example.com",
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
        "email": "taster@example.com",
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

    let cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap();

    // Create and finish a fermentation
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "Test Fermentation",
        "start_date": "2024-01-15T10:00:00Z",
    });

    let app3 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/fermentation")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&fermentation_body).unwrap()))
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

    // Finish the fermentation first
    let finish_body = json!({
        "success_rating": 4
    });

    let app4 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/finish", fermentation_id))
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&finish_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Add a taste profile
    let taste_profile_body = json!({
        "profile_text": "Tangy and crisp, with a hint of garlic"
    });

    let app5 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app5
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/taste-profiles", fermentation_id))
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&taste_profile_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let taste_profile: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(taste_profile["profile_text"], "Tangy and crisp, with a hint of garlic");
    assert_eq!(taste_profile["fermentation_id"], fermentation_id);
}

#[tokio::test]
async fn test_list_taste_profiles_success() {
    let app_state = common::create_test_app_state().await;

    // Register and login
    let register_body = json!({
        "email": "profilelister@example.com",
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
        "email": "profilelister@example.com",
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

    let cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap();

    // Create and finish a fermentation
    let fermentation_body = json!({
        "profile_id": 1,
        "name": "Test Fermentation",
        "start_date": "2024-01-15T10:00:00Z",
    });

    let app3 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app3
        .oneshot(
            Request::builder()
                .uri("/api/fermentation")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&fermentation_body).unwrap()))
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

    // Finish the fermentation with initial taste profile
    let finish_body = json!({
        "success_rating": 4,
        "taste_profile": "Initial taste: crispy and tangy"
    });

    let app4 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app4
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/finish", fermentation_id))
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&finish_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Add another taste profile
    let taste_profile_body = json!({
        "profile_text": "After a week: more complex flavor with deeper fermentation notes"
    });

    let app5 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app5
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/taste-profiles", fermentation_id))
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&taste_profile_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // List taste profiles
    let app6 = raugupatis_log::create_router(app_state.clone()).await;
    let response = app6
        .oneshot(
            Request::builder()
                .uri(&format!("/api/fermentation/{}/taste-profiles", fermentation_id))
                .header("Cookie", cookie)
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

    // Should have 2 taste profiles
    assert_eq!(profiles.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_taste_profile_unauthorized() {
    let app = common::create_test_app().await;

    let taste_profile_body = json!({
        "profile_text": "Test profile"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/fermentation/1/taste-profiles")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&taste_profile_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
