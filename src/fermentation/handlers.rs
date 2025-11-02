use axum::{extract::{Path, State}, http::StatusCode, Json};
use tower_sessions::Session;

use crate::fermentation::models::{
    CreateFermentationRequest, CreateTemperatureLogRequest, Fermentation, FermentationResponse,
    TemperatureLog, UpdateFermentationRequest,
};
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
    let photo_repo = crate::photos::PhotoRepository::new(state.db.clone());

    match repo.find_all_by_user(user.user_id).await {
        Ok(mut fermentations) => {
            // Populate thumbnail_path for each fermentation
            for fermentation in &mut fermentations {
                fermentation.thumbnail_path = photo_repo
                    .get_thumbnail_for_fermentation(fermentation.id, fermentation.status.as_str())
                    .await
                    .unwrap_or_else(|e| {
                        tracing::error!("Error fetching thumbnail for fermentation {}: {}", fermentation.id, e);
                        None
                    });
            }
            Ok(Json(fermentations))
        }
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

pub async fn update_fermentation(
    session: Session,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateFermentationRequest>,
) -> Result<Json<FermentationResponse>, StatusCode> {
    // Get user from session
    let user_session: Option<UserSession> = session
        .get("user")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = user_session.ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate request fields
    if let Some(ref name) = request.name {
        if name.trim().is_empty() || name.len() > 255 {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // Validate date formats
    if let Some(ref start_date) = request.start_date {
        if chrono::DateTime::parse_from_rfc3339(start_date).is_err() {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    if let Some(ref target_date) = request.target_end_date {
        if !target_date.is_empty() && chrono::DateTime::parse_from_rfc3339(target_date).is_err() {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    if let Some(ref actual_date) = request.actual_end_date {
        if !actual_date.is_empty() && chrono::DateTime::parse_from_rfc3339(actual_date).is_err() {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // Validate status
    if let Some(ref status) = request.status {
        if !matches!(status.as_str(), "active" | "paused" | "completed" | "failed") {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // Validate success rating
    if let Some(rating) = request.success_rating {
        if !(1..=5).contains(&rating) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let fermentation_repo = FermentationRepository::new(state.db.clone());

    // Update the fermentation
    let fermentation = fermentation_repo
        .update_fermentation(id, user.user_id, request)
        .await
        .map_err(|e| {
            tracing::error!("Error updating fermentation: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get the profile for the response
    let profile = fermentation_repo
        .get_profile_by_id(fermentation.profile_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(FermentationResponse::from_fermentation_and_profile(
        fermentation,
        profile,
    )))
}

pub async fn create_temperature_log(
    session: Session,
    State(state): State<AppState>,
    Path(fermentation_id): Path<i64>,
    Json(request): Json<CreateTemperatureLogRequest>,
) -> Result<(StatusCode, Json<TemperatureLog>), StatusCode> {
    // Get user from session
    let user_session: Option<UserSession> = session
        .get("user")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = user_session.ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate temperature value
    if request.temperature.is_nan() || request.temperature.is_infinite() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate recorded_at format if provided
    if let Some(ref recorded_at) = request.recorded_at {
        if chrono::DateTime::parse_from_rfc3339(recorded_at).is_err() {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let fermentation_repo = FermentationRepository::new(state.db.clone());

    // Create the temperature log
    let temperature_log = fermentation_repo
        .create_temperature_log(fermentation_id, user.user_id, request)
        .await
        .map_err(|e| {
            tracing::error!("Error creating temperature log: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(temperature_log)))
}

pub async fn list_temperature_logs(
    session: Session,
    State(state): State<AppState>,
    Path(fermentation_id): Path<i64>,
) -> Result<Json<Vec<TemperatureLog>>, StatusCode> {
    // Get user from session
    let user_session: Option<UserSession> = session
        .get("user")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = user_session.ok_or(StatusCode::UNAUTHORIZED)?;

    let fermentation_repo = FermentationRepository::new(state.db.clone());

    let logs = fermentation_repo
        .find_temperature_logs_by_fermentation(fermentation_id, user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching temperature logs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(logs))
}
