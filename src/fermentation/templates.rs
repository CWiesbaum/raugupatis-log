use askama::Template;
use axum::extract::{Path, Query, State};
use axum::response::{Html, Redirect};
use tower_sessions::Session;

use crate::fermentation::models::{Fermentation, FermentationListQuery};
use crate::fermentation::repository::FermentationRepository;
use crate::users::UserSession;
use crate::AppState;

#[derive(Template)]
#[template(path = "fermentation/list.html")]
pub struct FermentationListTemplate {
    pub title: String,
    pub user_email: String,
    pub fermentations: Vec<Fermentation>,
    pub search: String,
    pub status_filter: String,
    pub profile_type_filter: String,
    pub sort_by: String,
    pub sort_order: String,
}

pub async fn fermentation_list_handler(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<FermentationListQuery>,
) -> Result<Html<String>, Redirect> {
    // Get user from session
    let user_session: Option<UserSession> = session.get("user").await.unwrap_or(None);

    if let Some(user) = user_session {
        let repo = FermentationRepository::new(state.db.clone());
        let photo_repo = crate::photos::PhotoRepository::new(state.db.clone());

        let mut fermentations = repo
            .find_all_by_user(user.user_id, &query)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("Error fetching fermentations: {}", e);
                Vec::new()
            });

        // Populate thumbnail_path for each fermentation
        for fermentation in &mut fermentations {
            fermentation.thumbnail_path = photo_repo
                .get_thumbnail_for_fermentation(fermentation.id, fermentation.status.as_str())
                .await
                .unwrap_or_else(|e| {
                    tracing::error!(
                        "Error fetching thumbnail for fermentation {}: {}",
                        fermentation.id, e
                    );
                    None
                });
        }

        let template = FermentationListTemplate {
            title: "My Fermentations - Raugupatis Log".to_string(),
            user_email: user.email,
            fermentations,
            search: query.search.clone().unwrap_or_default(),
            status_filter: query.status.clone().unwrap_or_default(),
            profile_type_filter: query.profile_type.clone().unwrap_or_default(),
            sort_by: query
                .sort_by
                .clone()
                .unwrap_or_else(|| "created_at".to_string()),
            sort_order: query.sort_order.clone().unwrap_or_else(|| "desc".to_string()),
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
    pub temp_unit: String,
}

pub async fn new_fermentation_handler(
    State(state): State<AppState>,
    session: Session,
) -> Result<Html<String>, Redirect> {
    // Get user from session
    let user_session: Option<UserSession> = session.get("user").await.unwrap_or(None);

    if let Some(user) = user_session {
        // Fetch user details to get temperature preference
        // Default to Fahrenheit if user not found (should never happen in normal flow)
        let user_repo = crate::users::UserRepository::new(state.db.clone());
        let temp_unit = user_repo
            .find_by_id(user.user_id)
            .await
            .map(|u| u.preferred_temp_unit.as_str().to_string())
            .unwrap_or_else(|e| {
                tracing::warn!("Could not fetch user temperature preference: {}", e);
                "fahrenheit".to_string()
            });

        let template = NewFermentationTemplate {
            title: "New Fermentation - Raugupatis Log".to_string(),
            temp_unit,
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
    pub photos: Vec<crate::photos::FermentationPhoto>,
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
        let photo_repo = crate::photos::PhotoRepository::new(state.db.clone());

        // Fetch the fermentation by ID, ensuring it belongs to the current user
        match repo.find_by_id(id, user.user_id).await {
            Ok(Some(fermentation)) => {
                // Fetch photos for this fermentation
                let photos = photo_repo
                    .find_by_fermentation(id)
                    .await
                    .unwrap_or_else(|e| {
                        tracing::error!("Error fetching photos: {}", e);
                        Vec::new()
                    });

                let template = FermentationDetailTemplate {
                    title: format!("{} - Raugupatis Log", fermentation.name),
                    fermentation,
                    photos,
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
