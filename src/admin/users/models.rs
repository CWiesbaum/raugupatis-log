use crate::users::{ExperienceLevel, User, UserRole};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Response for listing users in admin panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUserResponse {
    pub id: i64,
    pub email: String,
    pub role: UserRole,
    pub experience_level: ExperienceLevel,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_locked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for AdminUserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            role: user.role,
            experience_level: user.experience_level,
            first_name: user.first_name,
            last_name: user.last_name,
            is_locked: user.is_locked,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// Request to create a new user by admin
#[derive(Debug, Deserialize)]
pub struct AdminCreateUserRequest {
    pub email: String,
    pub password: String,
    pub role: String,
    #[serde(default)]
    pub experience_level: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
}

/// Request to update a user by admin
#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub email: String,
    pub role: String,
    pub experience_level: String,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
}

/// Request to lock/unlock a user
#[derive(Debug, Deserialize)]
pub struct LockUserRequest {
    pub locked: bool,
}
