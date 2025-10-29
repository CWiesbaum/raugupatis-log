use askama::Template;
use axum::response::{Html, Redirect};
use tower_sessions::Session;
use crate::models::UserSession;

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
    
    Html(template.render().unwrap_or_else(|_| "Template render error".to_string()))
}

#[derive(Template)]
#[template(path = "login.html")]
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
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub title: String,
    pub user_email: String,
}

pub async fn dashboard_handler(session: Session) -> Result<Html<String>, Redirect> {
    // Get user from session
    let user_session: Option<UserSession> = session
        .get("user")
        .await
        .unwrap_or(None);
    
    if let Some(user) = user_session {
        let template = DashboardTemplate {
            title: "Dashboard - Raugupatis Log".to_string(),
            user_email: user.email,
        };
        
        Ok(Html(template.render().unwrap_or_else(|_| "Template render error".to_string())))
    } else {
        // Redirect to login if not authenticated
        Err(Redirect::to("/login"))
    }
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    pub title: String,
}

pub async fn register_handler() -> Html<String> {
    let template = RegisterTemplate {
        title: "Register - Raugupatis Log".to_string(),
    };
    
    Html(template.render().unwrap_or_else(|_| "Template render error".to_string()))
}
