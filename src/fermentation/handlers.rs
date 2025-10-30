use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use tower_sessions::Session;

use crate::fermentation::models::Fermentation;
use crate::fermentation::repository::FermentationRepository;
use crate::users::UserSession;
use crate::AppState;

pub async fn list_fermentations(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<Vec<Fermentation>>, StatusCode> {
    // Get user from session
    let user_session: Option<UserSession> = session
        .get("user")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = user_session.ok_or(StatusCode::UNAUTHORIZED)?;

    let repo = FermentationRepository::new(state.db.clone());
    
    match repo.find_all_by_user(user.user_id).await {
        Ok(fermentations) => Ok(Json(fermentations)),
        Err(e) => {
            tracing::error!("Error fetching fermentations: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
