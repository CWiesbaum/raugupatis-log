use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, Redirect};
use tower_sessions::Session;

use crate::fermentation::models::Fermentation;
use crate::fermentation::repository::FermentationRepository;
use crate::users::UserSession;
use crate::AppState;

#[derive(Template)]
#[template(path = "fermentation/list.html")]
pub struct FermentationListTemplate {
    pub title: String,
    pub user_email: String,
    pub fermentations: Vec<Fermentation>,
}

pub async fn fermentation_list_handler(
    State(state): State<AppState>,
    session: Session,
) -> Result<Html<String>, Redirect> {
    // Get user from session
    let user_session: Option<UserSession> = session.get("user").await.unwrap_or(None);

    if let Some(user) = user_session {
        let repo = FermentationRepository::new(state.db.clone());

        let fermentations = repo
            .find_all_by_user(user.user_id)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("Error fetching fermentations: {}", e);
                Vec::new()
            });

        let template = FermentationListTemplate {
            title: "My Fermentations - Raugupatis Log".to_string(),
            user_email: user.email,
            fermentations,
        };

        Ok(Html(template.render().unwrap_or_else(|e| {
            tracing::error!("Failed to render fermentation list template: {}", e);
            "Error rendering fermentation list".to_string()
        })))
    } else {
        // Redirect to login if not authenticated
        Err(Redirect::to("/login"))
    }
}

#[derive(Template)]
#[template(path = "fermentation/new.html")]
pub struct NewFermentationTemplate {
    pub title: String,
}

pub async fn new_fermentation_handler(session: Session) -> Result<Html<String>, Redirect> {
    // Get user from session
    let user_session: Option<UserSession> = session.get("user").await.unwrap_or(None);

    if user_session.is_some() {
        let template = NewFermentationTemplate {
            title: "New Fermentation - Raugupatis Log".to_string(),
        };

        Ok(Html(
            template
                .render()
                .unwrap_or_else(|_| "Template render error".to_string()),
        ))
    } else {
        // Redirect to login if not authenticated
        Err(Redirect::to("/login"))
    }
}

#[derive(Template)]
#[template(path = "fermentation/detail.html")]
pub struct FermentationDetailTemplate {
    pub title: String,
    pub fermentation: Fermentation,
}

pub async fn fermentation_detail_handler(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Result<Html<String>, Redirect> {
    // Get user from session
    let user_session: Option<UserSession> = session.get("user").await.unwrap_or(None);

    if let Some(user) = user_session {
        let repo = FermentationRepository::new(state.db.clone());

        // Fetch the fermentation by ID, ensuring it belongs to the current user
        match repo.find_by_id(id, user.user_id).await {
            Ok(Some(fermentation)) => {
                let template = FermentationDetailTemplate {
                    title: format!("{} - Raugupatis Log", fermentation.name),
                    fermentation,
                };

                Ok(Html(template.render().unwrap_or_else(|e| {
                    tracing::error!("Failed to render fermentation detail template: {}", e);
                    "Error rendering fermentation detail".to_string()
                })))
            }
            Ok(None) => {
                // Fermentation not found or doesn't belong to this user
                Err(Redirect::to("/fermentations"))
            }
            Err(e) => {
                tracing::error!("Error fetching fermentation: {}", e);
                Err(Redirect::to("/fermentations"))
            }
        }
    } else {
        // Redirect to login if not authenticated
        Err(Redirect::to("/login"))
    }
}

#[derive(Template)]
#[template(path = "fermentation/edit.html")]
pub struct EditFermentationTemplate {
    pub title: String,
    pub fermentation: Fermentation,
}

pub async fn edit_fermentation_handler(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Result<Html<String>, Redirect> {
    // Get user from session
    let user_session: Option<UserSession> = session.get("user").await.unwrap_or(None);

    if let Some(user) = user_session {
        let repo = FermentationRepository::new(state.db.clone());

        // Fetch the fermentation by ID, ensuring it belongs to the current user
        match repo.find_by_id(id, user.user_id).await {
            Ok(Some(fermentation)) => {
                let template = EditFermentationTemplate {
                    title: format!("Edit {} - Raugupatis Log", fermentation.name),
                    fermentation,
                };

                Ok(Html(template.render().unwrap_or_else(|e| {
                    tracing::error!("Failed to render edit fermentation template: {}", e);
                    "Error rendering edit fermentation".to_string()
                })))
            }
            Ok(None) => {
                // Fermentation not found or doesn't belong to this user
                Err(Redirect::to("/fermentations"))
            }
            Err(e) => {
                tracing::error!("Error fetching fermentation: {}", e);
                Err(Redirect::to("/fermentations"))
            }
        }
    } else {
        // Redirect to login if not authenticated
        Err(Redirect::to("/login"))
    }
}
