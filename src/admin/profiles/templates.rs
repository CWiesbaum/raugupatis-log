use askama::Template;
use axum::{
    extract::State,
    response::{Html, Redirect},
};
use tower_sessions::Session;

use crate::admin::profiles::models::AdminProfileResponse;
use crate::admin::profiles::repository::AdminProfileRepository;
use crate::users::models::{UserRole, UserSession};
use crate::AppState;

#[derive(Template)]
#[template(path = "admin/profiles/list.html")]
pub struct AdminProfilesListTemplate {
    pub title: String,
    pub profiles: Vec<AdminProfileResponse>,
}

/// Handler for admin profiles list page
pub async fn admin_profiles_list_handler(
    session: Session,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    // Check if user is authenticated and is an admin
    let user_session: UserSession = session
        .get("user")
        .await
        .ok()
        .flatten()
        .ok_or_else(|| Redirect::to("/login"))?;

    // Check if user has admin role
    match user_session.role {
        UserRole::Admin => {}
        _ => return Err(Redirect::to("/dashboard")),
    }

    // Fetch all profiles (including inactive)
    let repo = AdminProfileRepository::new(state.db.clone());
    let profiles = repo
        .list_all_profiles()
        .await
        .map_err(|_| Redirect::to("/dashboard"))?;

    let profile_responses: Vec<AdminProfileResponse> = profiles
        .into_iter()
        .map(AdminProfileResponse::from)
        .collect();

    let template = AdminProfilesListTemplate {
        title: "Fermentation Profile Administration - Raugupatis Log".to_string(),
        profiles: profile_responses,
    };

    Ok(Html(
        template
            .render()
            .unwrap_or_else(|_| "Template render error".to_string()),
    ))
}
