use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Request to create a new fermentation profile
#[derive(Debug, Deserialize)]
pub struct CreateProfileRequest {
    pub name: String,
    pub r#type: String,
    pub min_days: i32,
    pub max_days: i32,
    pub temp_min: f64,
    pub temp_max: f64,
    pub description: Option<String>,
}

/// Request to copy an existing profile
#[derive(Debug, Deserialize)]
pub struct CopyProfileRequest {
    pub new_name: String,
}

/// Request to deactivate a profile
#[derive(Debug, Deserialize)]
pub struct DeactivateProfileRequest {
    pub is_active: bool,
}

/// Response for profile management (includes is_active status)
#[derive(Debug, Serialize)]
pub struct AdminProfileResponse {
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub min_days: i32,
    pub max_days: i32,
    pub temp_min: f64,
    pub temp_max: f64,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<crate::fermentation::models::FermentationProfile> for AdminProfileResponse {
    fn from(profile: crate::fermentation::models::FermentationProfile) -> Self {
        Self {
            id: profile.id,
            name: profile.name,
            r#type: profile.r#type,
            min_days: profile.min_days,
            max_days: profile.max_days,
            temp_min: profile.temp_min,
            temp_max: profile.temp_max,
            description: profile.description,
            is_active: profile.is_active,
            created_at: profile.created_at,
        }
    }
}
