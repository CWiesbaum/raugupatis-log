use askama::Template;
use axum::{
    extract::State,
    response::{Html, Redirect},
};
use tower_sessions::Session;

use crate::admin::users::models::AdminUserResponse;
use crate::admin::users::repository::AdminUserRepository;
use crate::users::models::{UserRole, UserSession};
use crate::AppState;

#[derive(Template)]
#[template(path = "admin/users/list.html")]
pub struct AdminUsersListTemplate {
    pub title: String,
    pub users: Vec<AdminUserResponse>,
    pub admin_email: String,
}

/// Handler for admin users list page
pub async fn admin_users_list_handler(
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

    // Fetch all users
    let repo = AdminUserRepository::new(state.db.clone());
    let users = repo
        .list_all_users()
        .await
        .map_err(|_| Redirect::to("/dashboard"))?;

    let admin_responses: Vec<AdminUserResponse> =
        users.into_iter().map(AdminUserResponse::from).collect();

    let template = AdminUsersListTemplate {
        title: "User Administration - Raugupatis Log".to_string(),
        users: admin_responses,
        admin_email: user_session.email,
    };

    Ok(Html(
        template
            .render()
            .unwrap_or_else(|_| "Template render error".to_string()),
    ))
}
