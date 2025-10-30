mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

/// Helper to create an admin user and login
async fn create_and_login_admin(app_state: raugupatis_log::AppState) -> String {
    use raugupatis_log::users::CreateUserRequest;
    use raugupatis_log::users::UserRepository;

    let user_repo = UserRepository::new(app_state.db.clone());
    let admin_request = CreateUserRequest {
        email: "admin@example.com".to_string(),
        password: "adminpass123".to_string(),
        experience_level: Some("intermediate".to_string()),
        first_name: Some("Admin".to_string()),
        last_name: Some("User".to_string()),
    };

    let admin_user = user_repo.create_user(admin_request).await.unwrap();

    // Manually set admin role in database
    let db = app_state.db.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db.get_connection().lock().unwrap();
        conn.execute(
            "UPDATE users SET role = 'admin' WHERE id = ?1",
            rusqlite::params![admin_user.id],
        )
        .unwrap();
    })
    .await
    .unwrap();

    // Login to get session cookie
    let app = raugupatis_log::create_router(app_state).await;
    let login_body = json!({
        "email": "admin@example.com",
        "password": "adminpass123"
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
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    cookie_header.to_string()
}

/// Helper to create a regular user and login
async fn create_and_login_user(app_state: raugupatis_log::AppState) -> String {
    use raugupatis_log::users::CreateUserRequest;
    use raugupatis_log::users::UserRepository;

    let user_repo = UserRepository::new(app_state.db.clone());
    let user_request = CreateUserRequest {
        email: "user@example.com".to_string(),
        password: "userpass123".to_string(),
        experience_level: Some("beginner".to_string()),
        first_name: None,
        last_name: None,
    };

    user_repo.create_user(user_request).await.unwrap();

    // Login to get session cookie
    let app = raugupatis_log::create_router(app_state).await;
    let login_body = json!({
        "email": "user@example.com",
        "password": "userpass123"
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
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    cookie_header.to_string()
}

#[tokio::test]
async fn test_admin_list_users() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(app_state.clone()).await;

    let app = raugupatis_log::create_router(app_state).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/users")
                .method("GET")
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
    let users: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    // Should have at least the admin user
    assert!(!users.is_empty());
}

#[tokio::test]
async fn test_non_admin_cannot_list_users() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_user(app_state.clone()).await;

    let app = raugupatis_log::create_router(app_state).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/users")
                .method("GET")
                .header("Cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_create_user() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(app_state.clone()).await;

    let app = raugupatis_log::create_router(app_state).await;

    let request_body = json!({
        "email": "newuser@example.com",
        "password": "newpass123",
        "role": "user",
        "experience_level": "beginner",
        "first_name": "New",
        "last_name": "User"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/users")
                .method("POST")
                .header("Cookie", cookie)
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
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(user["email"], "newuser@example.com");
    assert_eq!(user["role"], "user");
    assert_eq!(user["first_name"], "New");
    assert_eq!(user["last_name"], "User");
}

#[tokio::test]
async fn test_admin_create_admin_user() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(app_state.clone()).await;

    let app = raugupatis_log::create_router(app_state).await;

    let request_body = json!({
        "email": "anotheradmin@example.com",
        "password": "adminpass123",
        "role": "admin",
        "experience_level": "advanced"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/users")
                .method("POST")
                .header("Cookie", cookie)
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
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(user["email"], "anotheradmin@example.com");
    assert_eq!(user["role"], "admin");
}

#[tokio::test]
async fn test_non_admin_cannot_create_user() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_user(app_state.clone()).await;

    let app = raugupatis_log::create_router(app_state).await;

    let request_body = json!({
        "email": "hacker@example.com",
        "password": "hackpass123",
        "role": "admin"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/users")
                .method("POST")
                .header("Cookie", cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_update_user() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(app_state.clone()).await;

    // Create a regular user first
    use raugupatis_log::users::CreateUserRequest;
    use raugupatis_log::users::UserRepository;

    let user_repo = UserRepository::new(app_state.db.clone());
    let user_request = CreateUserRequest {
        email: "toupdate@example.com".to_string(),
        password: "password123".to_string(),
        experience_level: Some("beginner".to_string()),
        first_name: None,
        last_name: None,
    };

    let created_user = user_repo.create_user(user_request).await.unwrap();

    let app = raugupatis_log::create_router(app_state).await;

    let update_body = json!({
        "email": "updated@example.com",
        "role": "user",
        "experience_level": "advanced",
        "first_name": "Updated",
        "last_name": "Name"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/admin/users/{}", created_user.id))
                .method("PUT")
                .header("Cookie", cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(user["email"], "updated@example.com");
    assert_eq!(user["experience_level"], "advanced");
    assert_eq!(user["first_name"], "Updated");
    assert_eq!(user["last_name"], "Name");
}

#[tokio::test]
async fn test_admin_lock_user() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(app_state.clone()).await;

    // Create a regular user first
    use raugupatis_log::users::CreateUserRequest;
    use raugupatis_log::users::UserRepository;

    let user_repo = UserRepository::new(app_state.db.clone());
    let user_request = CreateUserRequest {
        email: "tolock@example.com".to_string(),
        password: "password123".to_string(),
        experience_level: Some("beginner".to_string()),
        first_name: None,
        last_name: None,
    };

    let created_user = user_repo.create_user(user_request).await.unwrap();

    let app = raugupatis_log::create_router(app_state).await;

    let lock_body = json!({
        "locked": true
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/admin/users/{}/lock", created_user.id))
                .method("POST")
                .header("Cookie", cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&lock_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(user["is_locked"], true);
}

#[tokio::test]
async fn test_locked_user_cannot_login() {
    let app_state = common::create_test_app_state().await;

    // Create a user
    use raugupatis_log::users::CreateUserRequest;
    use raugupatis_log::users::UserRepository;

    let user_repo = UserRepository::new(app_state.db.clone());
    let user_request = CreateUserRequest {
        email: "locked@example.com".to_string(),
        password: "password123".to_string(),
        experience_level: Some("beginner".to_string()),
        first_name: None,
        last_name: None,
    };

    let created_user = user_repo.create_user(user_request).await.unwrap();

    // Lock the user
    let db = app_state.db.clone();
    let user_id = created_user.id;
    tokio::task::spawn_blocking(move || {
        let conn = db.get_connection().lock().unwrap();
        conn.execute(
            "UPDATE users SET is_locked = 1 WHERE id = ?1",
            rusqlite::params![user_id],
        )
        .unwrap();
    })
    .await
    .unwrap();

    // Try to login
    let app = raugupatis_log::create_router(app_state).await;

    let login_body = json!({
        "email": "locked@example.com",
        "password": "password123"
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
    let login_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(login_response["success"], false);
    assert!(login_response["message"]
        .as_str()
        .unwrap()
        .contains("locked"));
}

#[tokio::test]
async fn test_admin_delete_user() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(app_state.clone()).await;

    // Create a regular user first
    use raugupatis_log::users::CreateUserRequest;
    use raugupatis_log::users::UserRepository;

    let user_repo = UserRepository::new(app_state.db.clone());
    let user_request = CreateUserRequest {
        email: "todelete@example.com".to_string(),
        password: "password123".to_string(),
        experience_level: Some("beginner".to_string()),
        first_name: None,
        last_name: None,
    };

    let created_user = user_repo.create_user(user_request).await.unwrap();

    let app = raugupatis_log::create_router(app_state).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/admin/users/{}", created_user.id))
                .method("DELETE")
                .header("Cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_admin_cannot_lock_themselves() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(app_state.clone()).await;

    // Get the admin user ID
    use raugupatis_log::users::UserRepository;

    let user_repo = UserRepository::new(app_state.db.clone());
    let admin = user_repo
        .find_by_email("admin@example.com")
        .await
        .unwrap()
        .unwrap();

    let app = raugupatis_log::create_router(app_state).await;

    let lock_body = json!({
        "locked": true
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/admin/users/{}/lock", admin.id))
                .method("POST")
                .header("Cookie", cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&lock_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_admin_cannot_delete_themselves() {
    let app_state = common::create_test_app_state().await;

    // Create admin user and get their ID
    use raugupatis_log::users::CreateUserRequest;
    use raugupatis_log::users::UserRepository;

    let user_repo = UserRepository::new(app_state.db.clone());
    let admin_request = CreateUserRequest {
        email: "admin@example.com".to_string(),
        password: "adminpass123".to_string(),
        experience_level: Some("intermediate".to_string()),
        first_name: Some("Admin".to_string()),
        last_name: Some("User".to_string()),
    };

    let admin_user = user_repo.create_user(admin_request).await.unwrap();

    // Manually set admin role in database
    let db = app_state.db.clone();
    let admin_id = admin_user.id;
    tokio::task::spawn_blocking(move || {
        let conn = db.get_connection().lock().unwrap();
        conn.execute(
            "UPDATE users SET role = 'admin' WHERE id = ?1",
            rusqlite::params![admin_id],
        )
        .unwrap();
    })
    .await
    .unwrap();

    // Login to get session cookie
    let app = raugupatis_log::create_router(app_state).await;
    let login_body = json!({
        "email": "admin@example.com",
        "password": "adminpass123"
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
    let _cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();

    // Create another router with the same app_state to test delete
    let app_state2 = common::create_test_app_state().await;

    // Recreate admin in new state
    let user_repo2 = UserRepository::new(app_state2.db.clone());
    let admin_request2 = CreateUserRequest {
        email: "admin2@example.com".to_string(),
        password: "adminpass123".to_string(),
        experience_level: Some("intermediate".to_string()),
        first_name: Some("Admin".to_string()),
        last_name: Some("User".to_string()),
    };

    let admin_user2 = user_repo2.create_user(admin_request2).await.unwrap();

    // Manually set admin role in database
    let db2 = app_state2.db.clone();
    let admin_id2 = admin_user2.id;
    tokio::task::spawn_blocking(move || {
        let conn = db2.get_connection().lock().unwrap();
        conn.execute(
            "UPDATE users SET role = 'admin' WHERE id = ?1",
            rusqlite::params![admin_id2],
        )
        .unwrap();
    })
    .await
    .unwrap();

    // Login with new admin
    let app2 = raugupatis_log::create_router(app_state2.clone()).await;
    let login_body2 = json!({
        "email": "admin2@example.com",
        "password": "adminpass123"
    });

    let response2 = app2
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

    // Extract session cookie
    let cookie2 = response2
        .headers()
        .get("set-cookie")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();

    // Try to delete themselves
    let app3 = raugupatis_log::create_router(app_state2).await;

    let response3 = app3
        .oneshot(
            Request::builder()
                .uri(format!("/api/admin/users/{}", admin_id2))
                .method("DELETE")
                .header("Cookie", cookie2)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response3.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_admin_users_page_requires_admin() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_user(app_state.clone()).await;

    let app = raugupatis_log::create_router(app_state).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/users")
                .method("GET")
                .header("Cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should redirect to dashboard
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_admin_users_page_loads_for_admin() {
    let app_state = common::create_test_app_state().await;
    let cookie = create_and_login_admin(app_state.clone()).await;

    let app = raugupatis_log::create_router(app_state).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/users")
                .method("GET")
                .header("Cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
