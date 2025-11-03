use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tower_sessions::Session;

use crate::admin::profiles::models::{
    AdminProfileResponse, CopyProfileRequest, CreateProfileRequest, DeactivateProfileRequest,
};
use crate::admin::profiles::repository::AdminProfileRepository;
use crate::users::models::{UserRole, UserSession};
use crate::AppState;

#[derive(Debug)]
pub enum AdminProfileApiError {
    Unauthorized,
    Forbidden,
    NotFound,
    ValidationError(String),
    DatabaseError(String),
    InternalError(String),
    Conflict(String),
}

impl IntoResponse for AdminProfileApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AdminProfileApiError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
            }
            AdminProfileApiError::Forbidden => {
                (StatusCode::FORBIDDEN, "Admin access required".to_string())
            }
            AdminProfileApiError::NotFound => {
                (StatusCode::NOT_FOUND, "Profile not found".to_string())
            }
            AdminProfileApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AdminProfileApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AdminProfileApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AdminProfileApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// Helper function to check if the current user is an admin
async fn require_admin(session: &Session) -> Result<UserSession, AdminProfileApiError> {
    let user_session: UserSession = session
        .get("user")
        .await
        .map_err(|e| AdminProfileApiError::InternalError(format!("Failed to get session: {}", e)))?
        .ok_or(AdminProfileApiError::Unauthorized)?;

    match user_session.role {
        UserRole::Admin => Ok(user_session),
        _ => Err(AdminProfileApiError::Forbidden),
    }
}

/// List all fermentation profiles (admin only, includes inactive)
pub async fn list_all_profiles(
    session: Session,
    State(state): State<AppState>,
) -> Result<Json<Vec<AdminProfileResponse>>, AdminProfileApiError> {
    // Check admin authorization
    require_admin(&session).await?;

    let repo = AdminProfileRepository::new(state.db.clone());
    let profiles = repo.list_all_profiles().await.map_err(|e| {
        AdminProfileApiError::DatabaseError(format!("Failed to list profiles: {}", e))
    })?;

    let responses: Vec<AdminProfileResponse> = profiles
        .into_iter()
        .map(AdminProfileResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Create a new fermentation profile (admin only)
pub async fn create_profile(
    session: Session,
    State(state): State<AppState>,
    Json(request): Json<CreateProfileRequest>,
) -> Result<(StatusCode, Json<AdminProfileResponse>), AdminProfileApiError> {
    // Check admin authorization
    require_admin(&session).await?;

    // Validate input
    if request.name.trim().is_empty() {
        return Err(AdminProfileApiError::ValidationError(
            "Profile name cannot be empty".to_string(),
        ));
    }

    if request.r#type.trim().is_empty() {
        return Err(AdminProfileApiError::ValidationError(
            "Profile type cannot be empty".to_string(),
        ));
    }

    if request.min_days <= 0 || request.max_days <= 0 {
        return Err(AdminProfileApiError::ValidationError(
            "Days must be positive".to_string(),
        ));
    }

    if request.min_days > request.max_days {
        return Err(AdminProfileApiError::ValidationError(
            "Minimum days cannot exceed maximum days".to_string(),
        ));
    }

    if request.temp_min >= request.temp_max {
        return Err(AdminProfileApiError::ValidationError(
            "Minimum temperature must be less than maximum temperature".to_string(),
        ));
    }

    let repo = AdminProfileRepository::new(state.db.clone());

    // Check if profile name already exists
    if repo.name_exists(&request.name).await.unwrap_or(false) {
        return Err(AdminProfileApiError::Conflict(
            "Profile with this name already exists".to_string(),
        ));
    }

    // Create profile
    let profile = repo.create_profile(request).await.map_err(|e| {
        AdminProfileApiError::InternalError(format!("Failed to create profile: {}", e))
    })?;

    Ok((
        StatusCode::CREATED,
        Json(AdminProfileResponse::from(profile)),
    ))
}

/// Copy an existing profile (admin only)
pub async fn copy_profile(
    session: Session,
    State(state): State<AppState>,
    Path(profile_id): Path<i64>,
    Json(request): Json<CopyProfileRequest>,
) -> Result<(StatusCode, Json<AdminProfileResponse>), AdminProfileApiError> {
    // Check admin authorization
    require_admin(&session).await?;

    // Validate input
    if request.new_name.trim().is_empty() {
        return Err(AdminProfileApiError::ValidationError(
            "New profile name cannot be empty".to_string(),
        ));
    }

    let repo = AdminProfileRepository::new(state.db.clone());

    // Check if new profile name already exists
    if repo.name_exists(&request.new_name).await.unwrap_or(false) {
        return Err(AdminProfileApiError::Conflict(
            "Profile with this name already exists".to_string(),
        ));
    }

    // Copy profile
    let profile = repo
        .copy_profile(profile_id, request.new_name)
        .await
        .map_err(|e| {
            if e.to_string().contains("no rows") {
                AdminProfileApiError::NotFound
            } else {
                AdminProfileApiError::InternalError(format!("Failed to copy profile: {}", e))
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(AdminProfileResponse::from(profile)),
    ))
}

/// Deactivate or reactivate a profile (admin only)
pub async fn set_profile_active_status(
    session: Session,
    State(state): State<AppState>,
    Path(profile_id): Path<i64>,
    Json(request): Json<DeactivateProfileRequest>,
) -> Result<Json<AdminProfileResponse>, AdminProfileApiError> {
    // Check admin authorization
    require_admin(&session).await?;

    let repo = AdminProfileRepository::new(state.db.clone());

    // Update profile status
    let profile = repo
        .set_profile_active_status(profile_id, request.is_active)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                AdminProfileApiError::NotFound
            } else {
                AdminProfileApiError::DatabaseError(format!(
                    "Failed to update profile status: {}",
                    e
                ))
            }
        })?;

    Ok(Json(AdminProfileResponse::from(profile)))
}
