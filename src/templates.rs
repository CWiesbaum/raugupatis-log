use askama::Template;
use axum::response::Html;

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

pub async fn dashboard_handler() -> Html<String> {
    // For now, just show a generic dashboard
    // In the future, this will get the user from session
    let template = DashboardTemplate {
        title: "Dashboard - Raugupatis Log".to_string(),
        user_email: "User".to_string(),
    };
    
    Html(template.render().unwrap_or_else(|_| "Template render error".to_string()))
}
