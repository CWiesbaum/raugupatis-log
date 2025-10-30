use askama::Template;
use axum::response::{Html, Redirect};
use tower_sessions::Session;
use crate::users::UserSession;

#[derive(Template)]
#[template(path = "fermentation/new.html")]
pub struct NewFermentationTemplate {
    pub title: String,
}

pub async fn new_fermentation_handler(session: Session) -> Result<Html<String>, Redirect> {
    // Get user from session
    let user_session: Option<UserSession> = session
        .get("user")
        .await
        .unwrap_or(None);
    
    if user_session.is_some() {
        let template = NewFermentationTemplate {
            title: "New Fermentation - Raugupatis Log".to_string(),
        };
        
        Ok(Html(template.render().unwrap_or_else(|_| "Template render error".to_string())))
    } else {
        // Redirect to login if not authenticated
        Err(Redirect::to("/login"))
    }
}
