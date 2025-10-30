mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_home_endpoint() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_dashboard_page_loads() {
    let app = common::create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/dashboard")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Dashboard should redirect to login when not authenticated
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_dashboard_displays_user_names() {
    use axum::http::header;
    use serde_json::json;

    let app_state = common::create_test_app_state().await;

    // Create a user with first and last name
    let register_body = json!({
        "email": "john.doe@example.com",
        "password": "securepassword123",
        "first_name": "John",
        "last_name": "Doe"
    });

    let app1 = raugupatis_log::create_router(app_state.clone()).await;
    let register_response = app1
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

    assert_eq!(register_response.status(), StatusCode::CREATED);

    // Login to get session
    let login_body = json!({
        "email": "john.doe@example.com",
        "password": "securepassword123"
    });

    let app2 = raugupatis_log::create_router(app_state.clone()).await;
    let login_response = app2
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

    // Extract session cookie from login response
    let session_cookie = login_response
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|h| h.to_str().ok())
        .expect("Session cookie should be set");

    // Access dashboard with session cookie
    let app3 = raugupatis_log::create_router(app_state).await;
    let dashboard_response = app3
        .oneshot(
            Request::builder()
                .uri("/dashboard")
                .header(header::COOKIE, session_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(dashboard_response.status(), StatusCode::OK);

    // Check that the response body contains the first and last name
    let body = axum::body::to_bytes(dashboard_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify that first name and last name are displayed
    assert!(
        body_str.contains("John Doe"),
        "Dashboard should display full name 'John Doe'"
    );
    assert!(
        body_str.contains("john.doe@example.com"),
        "Dashboard should display email"
    );
}

#[tokio::test]
async fn test_dashboard_without_user_names() {
    use axum::http::header;
    use serde_json::json;

    let app_state = common::create_test_app_state().await;

    // Create a user without first and last name
    let register_body = json!({
        "email": "noname@example.com",
        "password": "securepassword123"
    });

    let app1 = raugupatis_log::create_router(app_state.clone()).await;
    let register_response = app1
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

    assert_eq!(register_response.status(), StatusCode::CREATED);

    // Login to get session
    let login_body = json!({
        "email": "noname@example.com",
        "password": "securepassword123"
    });

    let app2 = raugupatis_log::create_router(app_state.clone()).await;
    let login_response = app2
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

    // Extract session cookie from login response
    let session_cookie = login_response
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|h| h.to_str().ok())
        .expect("Session cookie should be set");

    // Access dashboard with session cookie
    let app3 = raugupatis_log::create_router(app_state).await;
    let dashboard_response = app3
        .oneshot(
            Request::builder()
                .uri("/dashboard")
                .header(header::COOKIE, session_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(dashboard_response.status(), StatusCode::OK);

    // Check that the response body does not contain name labels when names are not provided
    let body = axum::body::to_bytes(dashboard_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify that email is displayed but name labels are not shown
    assert!(
        body_str.contains("noname@example.com"),
        "Dashboard should display email"
    );
    assert!(
        !body_str.contains("<strong>Name:</strong>"),
        "Dashboard should not display Name label when names are not provided"
    );
    assert!(
        !body_str.contains("<strong>First Name:</strong>"),
        "Dashboard should not display First Name label when not provided"
    );
    assert!(
        !body_str.contains("<strong>Last Name:</strong>"),
        "Dashboard should not display Last Name label when not provided"
    );
}
