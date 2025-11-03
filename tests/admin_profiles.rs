mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

// Helper function to create an admin user and login
async fn create_and_login_admin(app_state: &raugupatis_log::AppState) -> String {
    use raugupatis_log::admin::AdminUserRepository;
    use raugupatis_log::users::models::{ExperienceLevel, UserRole};

    let repo = AdminUserRepository::new(app_state.db.clone());
    let _admin = repo
        .create_user_as_admin(
            "admin@example.com".to_string(),
            "adminpassword123".to_string(),
            UserRole::Admin,
            ExperienceLevel::Advanced,
            Some("Admin".to_string()),
            Some("User".to_string()),
        )
        .await
        .unwrap();

    // Login to get session cookie
    let app = raugupatis_log::create_router(app_state.clone()).await;
    let login_body = json!({
        "email": "admin@example.com",
        "password": "adminpassword123"
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

    // Extract session cookie
    let cookie_header = response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap();

    // Parse cookie to get just the session part
    cookie_header.split(';').next().unwrap().to_string()
}

// Helper function to create a regular user and login
async fn create_and_login_user(app_state: &raugupatis_log::AppState) -> String {
    use raugupatis_log::admin::AdminUserRepository;
    use raugupatis_log::users::models::{ExperienceLevel, UserRole};

    let repo = AdminUserRepository::new(app_state.db.clone());
    let _user = repo
        .create_user_as_admin(
            "user@example.com".to_string(),
            "userpassword123".to_string(),
            UserRole::User,
            ExperienceLevel::Beginner,
            None,
            None,
        )
        .await
        .unwrap();

    // Login to get session cookie
    let app = raugupatis_log::create_router(app_state.clone()).await;
    let login_body = json!({
        "email": "user@example.com",
        "password": "userpassword123"
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

    // Extract session cookie
    let cookie_header = response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap();

    cookie_header.split(';').next().unwrap().to_string()
}

#[tokio::test]
async fn test_admin_profiles_page_redirect_when_not_authenticated() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/profiles")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should redirect to login when not authenticated
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_admin_profiles_page_forbidden_for_regular_user() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_user(&app_state).await;

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/profiles")
                .header("Cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should redirect to dashboard when not admin
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_admin_profiles_page_accessible_for_admin() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(&app_state).await;

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/profiles")
                .header("Cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_all_profiles_requires_admin() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_user(&app_state).await;

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles")
                .header("Cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_list_all_profiles_includes_inactive() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(&app_state).await;

    // Deactivate one profile first
    use raugupatis_log::admin::AdminProfileRepository;
    let repo = AdminProfileRepository::new(app_state.db.clone());
    repo.set_profile_active_status(1, false).await.unwrap();

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles")
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
    let profiles_array = profiles.as_array().unwrap();

    // Should include all 7 default profiles
    assert_eq!(profiles_array.len(), 7);

    // Check that at least one is inactive
    let has_inactive = profiles_array
        .iter()
        .any(|p| p["is_active"].as_bool() == Some(false));
    assert!(has_inactive);
}

#[tokio::test]
async fn test_create_profile_success() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(&app_state).await;

    let create_body = json!({
        "name": "Test Ferment",
        "type": "test",
        "min_days": 3,
        "max_days": 7,
        "temp_min": 65.0,
        "temp_max": 75.0,
        "description": "A test fermentation profile"
    });

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let profile: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(profile["name"], "Test Ferment");
    assert_eq!(profile["type"], "test");
    assert_eq!(profile["is_active"], true);
}

#[tokio::test]
async fn test_create_profile_validation_errors() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(&app_state).await;

    // Test empty name
    let invalid_body = json!({
        "name": "",
        "type": "test",
        "min_days": 3,
        "max_days": 7,
        "temp_min": 65.0,
        "temp_max": 75.0
    });

    let app = raugupatis_log::create_router(app_state.clone()).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", &cookie)
                .body(Body::from(serde_json::to_string(&invalid_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test min_days > max_days
    let invalid_body = json!({
        "name": "Test",
        "type": "test",
        "min_days": 10,
        "max_days": 5,
        "temp_min": 65.0,
        "temp_max": 75.0
    });

    let app = raugupatis_log::create_router(app_state.clone()).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", &cookie)
                .body(Body::from(serde_json::to_string(&invalid_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test temp_min >= temp_max
    let invalid_body = json!({
        "name": "Test",
        "type": "test",
        "min_days": 3,
        "max_days": 7,
        "temp_min": 75.0,
        "temp_max": 65.0
    });

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&invalid_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_profile_duplicate_name() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(&app_state).await;

    // Try to create a profile with the same name as an existing one (Pickles)
    let duplicate_body = json!({
        "name": "Pickles",
        "type": "vegetable",
        "min_days": 3,
        "max_days": 7,
        "temp_min": 65.0,
        "temp_max": 75.0
    });

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&duplicate_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_profile_requires_admin() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_user(&app_state).await;

    let create_body = json!({
        "name": "Test Ferment",
        "type": "test",
        "min_days": 3,
        "max_days": 7,
        "temp_min": 65.0,
        "temp_max": 75.0
    });

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_copy_profile_success() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(&app_state).await;

    // Copy the Pickles profile (id=1)
    let copy_body = json!({
        "new_name": "Pickles Copy"
    });

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles/1/copy")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&copy_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let profile: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(profile["name"], "Pickles Copy");
    assert_eq!(profile["type"], "vegetable");
    assert_eq!(profile["min_days"], 3);
    assert_eq!(profile["max_days"], 7);
    assert_eq!(profile["is_active"], true);
}

#[tokio::test]
async fn test_copy_profile_nonexistent() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(&app_state).await;

    let copy_body = json!({
        "new_name": "Copy of Nonexistent"
    });

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles/9999/copy")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&copy_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_copy_profile_duplicate_name() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(&app_state).await;

    // Try to copy with a name that already exists
    let copy_body = json!({
        "new_name": "Kombucha"
    });

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles/1/copy")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&copy_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_copy_profile_requires_admin() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_user(&app_state).await;

    let copy_body = json!({
        "new_name": "Pickles Copy"
    });

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles/1/copy")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&copy_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_deactivate_profile_success() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(&app_state).await;

    // Deactivate profile
    let deactivate_body = json!({
        "is_active": false
    });

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles/1/status")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&deactivate_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let profile: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(profile["is_active"], false);
}

#[tokio::test]
async fn test_reactivate_profile_success() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(&app_state).await;

    // First deactivate
    use raugupatis_log::admin::AdminProfileRepository;
    let repo = AdminProfileRepository::new(app_state.db.clone());
    repo.set_profile_active_status(1, false).await.unwrap();

    // Now reactivate
    let activate_body = json!({
        "is_active": true
    });

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles/1/status")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&activate_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let profile: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(profile["is_active"], true);
}

#[tokio::test]
async fn test_deactivate_profile_requires_admin() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_user(&app_state).await;

    let deactivate_body = json!({
        "is_active": false
    });

    let app = raugupatis_log::create_router(app_state).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles/1/status")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", cookie)
                .body(Body::from(serde_json::to_string(&deactivate_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_deactivated_profile_not_in_user_list() {
    let app_state = common::create_test_app_state().await;
    let admin_cookie = create_and_login_admin(&app_state).await;

    // Deactivate a profile
    let deactivate_body = json!({
        "is_active": false
    });

    let app = raugupatis_log::create_router(app_state.clone()).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/profiles/1/status")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Cookie", admin_cookie)
                .body(Body::from(serde_json::to_string(&deactivate_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Now check the user-facing profiles endpoint (no auth required)
    let app = raugupatis_log::create_router(app_state).await;
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
    let profiles_array = profiles.as_array().unwrap();

    // Should have 6 profiles (7 default - 1 deactivated)
    assert_eq!(profiles_array.len(), 6);

    // Make sure the deactivated profile is not in the list
    let has_deactivated_profile = profiles_array.iter().any(|p| p["id"] == 1);
    assert!(!has_deactivated_profile);
}
