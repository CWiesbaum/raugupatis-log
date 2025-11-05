use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TemperatureUnit {
    Fahrenheit,
    Celsius,
}

impl TemperatureUnit {
    pub fn as_str(&self) -> &str {
        match self {
            TemperatureUnit::Fahrenheit => "fahrenheit",
            TemperatureUnit::Celsius => "celsius",
        }
    }

    pub fn is_valid(s: &str) -> bool {
        matches!(s, "fahrenheit" | "celsius")
    }
}

impl std::fmt::Display for TemperatureUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq<&str> for TemperatureUnit {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl From<String> for TemperatureUnit {
    fn from(s: String) -> Self {
        match s.as_str() {
            "celsius" => TemperatureUnit::Celsius,
            _ => TemperatureUnit::Fahrenheit,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
}

impl UserRole {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            UserRole::User => "user",
            UserRole::Admin => "admin",
        }
    }

    /// Returns a display-friendly version of the role with proper capitalization
    pub fn display_name(&self) -> &str {
        match self {
            UserRole::User => "User",
            UserRole::Admin => "Admin",
        }
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<String> for UserRole {
    fn from(s: String) -> Self {
        match s.as_str() {
            "admin" => UserRole::Admin,
            _ => UserRole::User,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExperienceLevel {
    Beginner,
    Intermediate,
    Advanced,
}

impl ExperienceLevel {
    pub fn as_str(&self) -> &str {
        match self {
            ExperienceLevel::Beginner => "beginner",
            ExperienceLevel::Intermediate => "intermediate",
            ExperienceLevel::Advanced => "advanced",
        }
    }

    pub fn is_valid(s: &str) -> bool {
        matches!(s, "beginner" | "intermediate" | "advanced")
    }
}

impl std::fmt::Display for ExperienceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq<&str> for ExperienceLevel {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl From<String> for ExperienceLevel {
    fn from(s: String) -> Self {
        match s.as_str() {
            "intermediate" => ExperienceLevel::Intermediate,
            "advanced" => ExperienceLevel::Advanced,
            _ => ExperienceLevel::Beginner,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
    pub experience_level: ExperienceLevel,
    pub preferred_temp_unit: TemperatureUnit,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_locked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub experience_level: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub email: String,
    pub role: UserRole,
    pub experience_level: ExperienceLevel,
    pub preferred_temp_unit: TemperatureUnit,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            role: user.role,
            experience_level: user.experience_level,
            preferred_temp_unit: user.preferred_temp_unit,
            first_name: user.first_name,
            last_name: user.last_name,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub remember_me: bool,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub user: Option<UserResponse>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: i64,
    pub email: String,
    pub role: UserRole,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub experience_level: String,
    pub preferred_temp_unit: String,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_display_name() {
        assert_eq!(UserRole::User.display_name(), "User");
        assert_eq!(UserRole::Admin.display_name(), "Admin");
    }

    #[test]
    fn test_user_role_as_str_unchanged() {
        // Ensure as_str() still returns lowercase for database operations
        assert_eq!(UserRole::User.as_str(), "user");
        assert_eq!(UserRole::Admin.as_str(), "admin");
    }
}
