use askama::Template;
use axum::response::Html;

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
