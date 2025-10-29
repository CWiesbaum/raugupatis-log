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
