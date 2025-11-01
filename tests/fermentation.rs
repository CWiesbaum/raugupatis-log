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
