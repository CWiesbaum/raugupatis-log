use crate::users::repository::UserRepository;
use crate::users::{UserResponse, UserSession};
use crate::AppState;
use askama::Template;
use axum::extract::State;
use axum::response::{Html, Redirect};
use tower_sessions::Session;

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    pub title: String,
    pub message: String,
}

pub async fn home_handler() -> Html<String> {
    let template = HomeTemplate {
        title: "Raugupatis Log".to_string(),
        message: "Welcome to Raugupatis Log - Your Fermentation Tracking Companion!".to_string(),
    };

    Html(
        template
            .render()
            .unwrap_or_else(|_| "Template render error".to_string()),
    )
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub title: String,
    pub user: UserResponse,
}

pub async fn dashboard_handler(
    session: Session,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    // Check if user is authenticated
    let user_session: UserSession = session
        .get("user")
        .await
        .ok()
        .flatten()
        .ok_or_else(|| Redirect::to("/login"))?;

    // Fetch user details from database
    let user_repo = UserRepository::new(state.db.clone());
    let user = user_repo
        .find_by_id(user_session.user_id)
        .await
        .map_err(|_| Redirect::to("/login"))?;

    let template = DashboardTemplate {
        title: "Dashboard - Raugupatis Log".to_string(),
        user: UserResponse::from(user),
    };

    Ok(Html(
        template
            .render()
            .unwrap_or_else(|_| "Template render error".to_string()),
    ))
}
