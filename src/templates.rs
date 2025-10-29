use askama::Template;
use axum::response::Html;
use tower_sessions::Session;

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    pub title: String,
    pub message: String,
    pub user_email: Option<String>,
}

pub async fn home_handler(session: Session) -> Html<String> {
    // Check if user is logged in by looking for user_email in session
    let user_email = session.get::<String>("user_email").await.ok().flatten();

    let template = HomeTemplate {
        title: "Raugupatis Log".to_string(),
        message: "Welcome to Raugupatis Log - Your Fermentation Tracking Companion!".to_string(),
        user_email,
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

pub async fn dashboard_handler() -> Html<String> {
    // For now, just show a generic dashboard
    // In the future, this will get the user from session
    let template = DashboardTemplate {
        title: "Dashboard - Raugupatis Log".to_string(),
        user_email: "User".to_string(),
    };
    
    Html(template.render().unwrap_or_else(|_| "Template render error".to_string()))
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
