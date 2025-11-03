use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use time::Duration;
use tower_sessions::{Expiry, Session};

use crate::users::auth::{hash_password, verify_password};
use crate::users::models::{
    ChangePasswordRequest, CreateUserRequest, ExperienceLevel, LoginRequest, LoginResponse,
    TemperatureUnit, UpdateProfileRequest, UserResponse, UserSession,
};
use crate::users::repository::UserRepository;
use crate::AppState;

#[derive(Debug)]
pub enum ApiError {
    UserAlreadyExists,
    ValidationError(String),
    DatabaseError(String),
    InternalError(String),
    Unauthorized,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::UserAlreadyExists => (
                StatusCode::CONFLICT,
                "User with this email already exists".to_string(),
            ),
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub async fn register_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError> {
    // Validate email format
    if !is_valid_email(&request.email) {
        return Err(ApiError::ValidationError(
            "Invalid email format".to_string(),
        ));
    }

    // Validate password strength
    if request.password.len() < 8 {
        return Err(ApiError::ValidationError(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    let user_repo = UserRepository::new(state.db.clone());

    // Check if user already exists
    match user_repo.find_by_email(&request.email).await {
        Ok(Some(_)) => return Err(ApiError::UserAlreadyExists),
        Ok(None) => {}
        Err(e) => {
            return Err(ApiError::DatabaseError(format!(
                "Failed to check existing user: {}",
                e
            )))
        }
    }

    // Create the user
    let user = user_repo
        .create_user(request)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to create user: {}", e)))?;

    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

pub async fn logout_user(session: Session) -> Result<Json<serde_json::Value>, ApiError> {
    // Clear the session
    session
        .flush()
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to clear session: {}", e)))?;

    Ok(Json(json!({
        "success": true,
        "message": "Logout successful"
    })))
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

pub async fn login_user(
    session: Session,
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Validate email format
    if !is_valid_email(&request.email) {
        return Ok(Json(LoginResponse {
            success: false,
            user: None,
            message: "Invalid email format".to_string(),
        }));
    }

    let user_repo = UserRepository::new(state.db.clone());

    // Find user by email
    let user = match user_repo.find_by_email(&request.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(Json(LoginResponse {
                success: false,
                user: None,
                message: "Invalid email or password".to_string(),
            }));
        }
        Err(e) => {
            return Err(ApiError::DatabaseError(format!(
                "Failed to find user: {}",
                e
            )));
        }
    };

    // Check if user is locked
    if user.is_locked {
        return Ok(Json(LoginResponse {
            success: false,
            user: None,
            message: "Account is locked. Please contact an administrator.".to_string(),
        }));
    }

    // Verify password
    match verify_password(&request.password, &user.password_hash) {
        Ok(true) => {
            // Create server-side session
            let user_session = UserSession {
                user_id: user.id,
                email: user.email.clone(),
                role: user.role.clone(),
            };
            // Set session expiry based on remember_me flag
            // Default: 24 hours, Remember me: 5 days (120 hours)
            let expiry_duration = if request.remember_me {
                Duration::hours(120) // 5 days
            } else {
                Duration::hours(24) // 1 day (default)
            };

            session.set_expiry(Some(Expiry::OnInactivity(expiry_duration)));

            // Store session data
            session
                .insert("user", user_session)
                .await
                .map_err(|e| ApiError::InternalError(format!("Failed to create session: {}", e)))?;

            Ok(Json(LoginResponse {
                success: true,
                user: Some(UserResponse::from(user)),
                message: "Login successful".to_string(),
            }))
        }
        Ok(false) => Ok(Json(LoginResponse {
            success: false,
            user: None,
            message: "Invalid email or password".to_string(),
        })),
        Err(e) => Err(ApiError::InternalError(format!(
            "Failed to verify password: {}",
            e
        ))),
    }
}

pub async fn update_profile(
    session: Session,
    State(state): State<AppState>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    // Check if user is authenticated
    let user_session: UserSession = session
        .get("user")
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to get session: {}", e)))?
        .ok_or(ApiError::Unauthorized)?;

    // Validate experience level
    if !ExperienceLevel::is_valid(&request.experience_level) {
        return Err(ApiError::ValidationError(
            "Invalid experience level. Must be 'beginner', 'intermediate', or 'advanced'"
                .to_string(),
        ));
    }

    // Validate temperature unit
    if !TemperatureUnit::is_valid(&request.preferred_temp_unit) {
        return Err(ApiError::ValidationError(
            "Invalid temperature unit. Must be 'fahrenheit' or 'celsius'".to_string(),
        ));
    }

    let experience_level = ExperienceLevel::from(request.experience_level);
    let preferred_temp_unit = TemperatureUnit::from(request.preferred_temp_unit);
    let user_repo = UserRepository::new(state.db.clone());

    // Update the user's profile (experience level, temperature unit, first name, and last name)
    let updated_user = user_repo
        .update_profile(
            user_session.user_id,
            experience_level,
            preferred_temp_unit,
            request.first_name,
            request.last_name,
        )
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to update profile: {}", e)))?;

    Ok(Json(UserResponse::from(updated_user)))
}

pub async fn change_password(
    session: Session,
    State(state): State<AppState>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check if user is authenticated
    let user_session: UserSession = session
        .get("user")
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to get session: {}", e)))?
        .ok_or(ApiError::Unauthorized)?;

    // Validate new password strength
    if request.new_password.len() < 8 {
        return Err(ApiError::ValidationError(
            "New password must be at least 8 characters long".to_string(),
        ));
    }

    let user_repo = UserRepository::new(state.db.clone());

    // Get current user to verify current password
    let user = user_repo
        .find_by_id(user_session.user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to find user: {}", e)))?;

    // Verify current password
    match verify_password(&request.current_password, &user.password_hash) {
        Ok(true) => {
            // Hash new password
            let new_password_hash = hash_password(&request.new_password)
                .map_err(|e| ApiError::InternalError(format!("Failed to hash password: {}", e)))?;

            // Update password in database
            user_repo
                .update_password(user_session.user_id, new_password_hash)
                .await
                .map_err(|e| {
                    ApiError::DatabaseError(format!("Failed to update password: {}", e))
                })?;

            Ok(Json(json!({
                "success": true,
                "message": "Password changed successfully"
            })))
        }
        Ok(false) => Err(ApiError::ValidationError(
            "Current password is incorrect".to_string(),
        )),
        Err(e) => Err(ApiError::InternalError(format!(
            "Failed to verify password: {}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("test.user+tag@domain.co.uk"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("user@"));
    }
}
