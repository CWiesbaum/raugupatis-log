use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tower_sessions::Session;

use crate::admin::users::models::{
    AdminCreateUserRequest, AdminUpdateUserRequest, AdminUserResponse, LockUserRequest,
};
use crate::admin::users::repository::AdminUserRepository;
use crate::users::models::{ExperienceLevel, UserRole, UserSession};
use crate::AppState;

#[derive(Debug)]
pub enum AdminApiError {
    Unauthorized,
    Forbidden,
    NotFound,
    ValidationError(String),
    DatabaseError(String),
    InternalError(String),
    Conflict(String),
}

impl IntoResponse for AdminApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AdminApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AdminApiError::Forbidden => {
                (StatusCode::FORBIDDEN, "Admin access required".to_string())
            }
            AdminApiError::NotFound => (StatusCode::NOT_FOUND, "User not found".to_string()),
            AdminApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AdminApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AdminApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AdminApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// Helper function to check if the current user is an admin
async fn require_admin(session: &Session) -> Result<UserSession, AdminApiError> {
    let user_session: UserSession = session
        .get("user")
        .await
        .map_err(|e| AdminApiError::InternalError(format!("Failed to get session: {}", e)))?
        .ok_or(AdminApiError::Unauthorized)?;

    match user_session.role {
        UserRole::Admin => Ok(user_session),
        _ => Err(AdminApiError::Forbidden),
    }
}

/// List all users (admin only)
pub async fn list_users(
    session: Session,
    State(state): State<AppState>,
) -> Result<Json<Vec<AdminUserResponse>>, AdminApiError> {
    // Check admin authorization
    require_admin(&session).await?;

    let repo = AdminUserRepository::new(state.db.clone());
    let users = repo
        .list_all_users()
        .await
        .map_err(|e| AdminApiError::DatabaseError(format!("Failed to list users: {}", e)))?;

    let responses: Vec<AdminUserResponse> =
        users.into_iter().map(AdminUserResponse::from).collect();

    Ok(Json(responses))
}

/// Create a new user (admin only)
pub async fn create_user(
    session: Session,
    State(state): State<AppState>,
    Json(request): Json<AdminCreateUserRequest>,
) -> Result<(StatusCode, Json<AdminUserResponse>), AdminApiError> {
    // Check admin authorization
    require_admin(&session).await?;

    // Validate email format
    if !is_valid_email(&request.email) {
        return Err(AdminApiError::ValidationError(
            "Invalid email format".to_string(),
        ));
    }

    // Validate password strength
    if request.password.len() < 8 {
        return Err(AdminApiError::ValidationError(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    // Parse and validate role
    let role = match request.role.as_str() {
        "admin" => UserRole::Admin,
        "user" => UserRole::User,
        _ => {
            return Err(AdminApiError::ValidationError(
                "Invalid role. Must be 'user' or 'admin'".to_string(),
            ));
        }
    };

    // Parse experience level
    let experience_level = request
        .experience_level
        .map(ExperienceLevel::from)
        .unwrap_or(ExperienceLevel::Beginner);

    let repo = AdminUserRepository::new(state.db.clone());

    // Check if email already exists
    if repo.email_exists(&request.email).await.unwrap_or(false) {
        return Err(AdminApiError::Conflict(
            "User with this email already exists".to_string(),
        ));
    }

    // Create user
    let user = repo
        .create_user_as_admin(
            request.email,
            request.password,
            role,
            experience_level,
            request.first_name,
            request.last_name,
        )
        .await
        .map_err(|e| AdminApiError::InternalError(format!("Failed to create user: {}", e)))?;

    Ok((StatusCode::CREATED, Json(AdminUserResponse::from(user))))
}

/// Update a user (admin only)
pub async fn update_user(
    session: Session,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(request): Json<AdminUpdateUserRequest>,
) -> Result<Json<AdminUserResponse>, AdminApiError> {
    // Check admin authorization
    require_admin(&session).await?;

    // Validate email format
    if !is_valid_email(&request.email) {
        return Err(AdminApiError::ValidationError(
            "Invalid email format".to_string(),
        ));
    }

    // Parse and validate role
    let role = match request.role.as_str() {
        "admin" => UserRole::Admin,
        "user" => UserRole::User,
        _ => {
            return Err(AdminApiError::ValidationError(
                "Invalid role. Must be 'user' or 'admin'".to_string(),
            ));
        }
    };

    // Validate experience level
    if !ExperienceLevel::is_valid(&request.experience_level) {
        return Err(AdminApiError::ValidationError(
            "Invalid experience level. Must be 'beginner', 'intermediate', or 'advanced'"
                .to_string(),
        ));
    }

    let experience_level = ExperienceLevel::from(request.experience_level);

    let repo = AdminUserRepository::new(state.db.clone());

    // Update user
    let user = repo
        .update_user_as_admin(
            user_id,
            request.email,
            role,
            experience_level,
            request.first_name,
            request.last_name,
        )
        .await
        .map_err(|e| AdminApiError::DatabaseError(format!("Failed to update user: {}", e)))?;

    Ok(Json(AdminUserResponse::from(user)))
}

/// Lock or unlock a user account (admin only)
pub async fn lock_user(
    session: Session,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(request): Json<LockUserRequest>,
) -> Result<Json<AdminUserResponse>, AdminApiError> {
    // Check admin authorization
    let admin_session = require_admin(&session).await?;

    // Prevent admin from locking themselves
    if admin_session.user_id == user_id {
        return Err(AdminApiError::ValidationError(
            "Cannot lock your own account".to_string(),
        ));
    }

    let repo = AdminUserRepository::new(state.db.clone());

    // Lock/unlock user
    let user = repo
        .lock_user(user_id, request.locked)
        .await
        .map_err(|e| AdminApiError::DatabaseError(format!("Failed to lock/unlock user: {}", e)))?;

    Ok(Json(AdminUserResponse::from(user)))
}

/// Delete a user (admin only)
pub async fn delete_user(
    session: Session,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<StatusCode, AdminApiError> {
    // Check admin authorization
    let admin_session = require_admin(&session).await?;

    // Prevent admin from deleting themselves
    if admin_session.user_id == user_id {
        return Err(AdminApiError::ValidationError(
            "Cannot delete your own account".to_string(),
        ));
    }

    let repo = AdminUserRepository::new(state.db.clone());

    // Delete user
    repo.delete_user(user_id)
        .await
        .map_err(|e| AdminApiError::DatabaseError(format!("Failed to delete user: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

fn is_valid_email(email: &str) -> bool {
    // Basic email validation per RFC 5322 simplified rules
    // Must have exactly one @ separating local and domain parts
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];

    // Local part must not be empty and domain must have at least one dot
    // Domain must have characters before and after the dot
    if local.is_empty() || !domain.contains('.') {
        return false;
    }

    // Check domain has valid structure (at least "x.x" format)
    let domain_parts: Vec<&str> = domain.split('.').collect();
    domain_parts.len() >= 2 && domain_parts.iter().all(|part| !part.is_empty())
}
