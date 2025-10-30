use askama::Template;
use axum::{
    response::{Html, Redirect},
    extract::State,
};
use tower_sessions::Session;

use crate::users::models::{UserSession, UserResponse};
use crate::users::repository::UserRepository;
use crate::AppState;

#[derive(Template)]
#[template(path = "users/login.html")]
pub struct LoginTemplate {
    pub title: String,
}

pub async fn login_handler() -> Html<String> {
    let template = LoginTemplate {
        title: "Login - Raugupatis Log".to_string(),
    };
    
    Html(template.render().unwrap_or_else(|_| "Template render error".to_string()))
}

#[derive(Template)]
#[template(path = "users/register.html")]
pub struct RegisterTemplate {
    pub title: String,
}

pub async fn register_handler() -> Html<String> {
    let template = RegisterTemplate {
        title: "Register - Raugupatis Log".to_string(),
    };
    
    Html(template.render().unwrap_or_else(|_| "Template render error".to_string()))
}

#[derive(Template)]
#[template(path = "users/profile.html")]
pub struct ProfileTemplate {
    pub title: String,
    pub user: UserResponse,
}

pub async fn profile_handler(
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

    let template = ProfileTemplate {
        title: "Edit Profile - Raugupatis Log".to_string(),
        user: UserResponse::from(user),
    };
    
    Ok(Html(template.render().unwrap_or_else(|_| "Template render error".to_string())))
}
