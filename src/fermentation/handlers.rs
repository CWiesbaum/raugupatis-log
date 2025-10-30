use axum::{extract::State, http::StatusCode, Json};
use tower_sessions::Session;

use crate::fermentation::models::{CreateFermentationRequest, Fermentation, FermentationResponse};
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

pub async fn create_fermentation(
    session: Session,
    State(state): State<AppState>,
    Json(request): Json<CreateFermentationRequest>,
) -> Result<(StatusCode, Json<FermentationResponse>), StatusCode> {
    // Get user from session
    let user_session: Option<UserSession> = session
        .get("user")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = user_session.ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate request
    if request.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    if request.name.len() > 255 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate date format
    if chrono::DateTime::parse_from_rfc3339(&request.start_date).is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }

    if let Some(ref target_date) = request.target_end_date {
        if chrono::DateTime::parse_from_rfc3339(target_date).is_err() {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let fermentation_repo = FermentationRepository::new(state.db.clone());

    // Verify profile exists
    let profile = fermentation_repo
        .get_profile_by_id(request.profile_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Create the fermentation
    let fermentation = fermentation_repo
        .create_fermentation(user.user_id, request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        Json(FermentationResponse::from_fermentation_and_profile(
            fermentation,
            profile,
        )),
    ))
}

pub async fn get_profiles(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::fermentation::models::FermentationProfile>>, StatusCode> {
    let fermentation_repo = FermentationRepository::new(state.db.clone());

    let profiles = fermentation_repo
        .get_all_profiles()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(profiles))
}
