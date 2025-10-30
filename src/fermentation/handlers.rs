use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tower_sessions::Session;

use crate::fermentation::models::{CreateFermentationRequest, FermentationResponse};
use crate::fermentation::repository::FermentationRepository;
use crate::users::UserSession;
use crate::AppState;

#[derive(Debug)]
pub enum ApiError {
    ValidationError(String),
    DatabaseError(String),
    InternalError(String),
    Unauthorized,
    NotFound(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "User not authenticated".to_string(),
            ),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub async fn create_fermentation(
    session: Session,
    State(state): State<AppState>,
    Json(request): Json<CreateFermentationRequest>,
) -> Result<(StatusCode, Json<FermentationResponse>), ApiError> {
    // Get user from session
    let user_session: Option<UserSession> = session
        .get("user")
        .await
        .map_err(|e| ApiError::InternalError(format!("Session error: {}", e)))?;

    let user = user_session.ok_or(ApiError::Unauthorized)?;

    // Validate request
    if request.name.trim().is_empty() {
        return Err(ApiError::ValidationError(
            "Fermentation name cannot be empty".to_string(),
        ));
    }

    if request.name.len() > 255 {
        return Err(ApiError::ValidationError(
            "Fermentation name is too long (max 255 characters)".to_string(),
        ));
    }

    // Validate date format
    if chrono::DateTime::parse_from_rfc3339(&request.start_date).is_err() {
        return Err(ApiError::ValidationError(
            "Invalid start_date format. Use ISO 8601 format (e.g., 2024-01-01T00:00:00Z)".to_string(),
        ));
    }

    if let Some(ref target_date) = request.target_end_date {
        if chrono::DateTime::parse_from_rfc3339(target_date).is_err() {
            return Err(ApiError::ValidationError(
                "Invalid target_end_date format. Use ISO 8601 format".to_string(),
            ));
        }
    }

    let fermentation_repo = FermentationRepository::new(state.db.clone());

    // Verify profile exists
    let profile = fermentation_repo
        .get_profile_by_id(request.profile_id)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to verify profile: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Fermentation profile not found".to_string()))?;

    // Create the fermentation
    let fermentation = fermentation_repo
        .create_fermentation(user.user_id, request)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to create fermentation: {}", e)))?;

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
) -> Result<Json<Vec<crate::fermentation::models::FermentationProfile>>, ApiError> {
    let fermentation_repo = FermentationRepository::new(state.db.clone());

    let profiles = fermentation_repo
        .get_all_profiles()
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to fetch profiles: {}", e)))?;

    Ok(Json(profiles))
}
