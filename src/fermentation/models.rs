use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FermentationStatus {
    Active,
    Paused,
    Completed,
    Failed,
}

impl FermentationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            FermentationStatus::Active => "active",
            FermentationStatus::Paused => "paused",
            FermentationStatus::Completed => "completed",
            FermentationStatus::Failed => "failed",
        }
    }
}

impl From<String> for FermentationStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "paused" => FermentationStatus::Paused,
            "completed" => FermentationStatus::Completed,
            "failed" => FermentationStatus::Failed,
            _ => FermentationStatus::Active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FermentationProfile {
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub min_days: i32,
    pub max_days: i32,
    pub temp_min: f64,
    pub temp_max: f64,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fermentation {
    pub id: i64,
    pub user_id: i64,
    pub profile_id: i64,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub target_end_date: Option<DateTime<Utc>>,
    pub actual_end_date: Option<DateTime<Utc>>,
    pub status: FermentationStatus,
    pub success_rating: Option<i32>,
    pub notes: Option<String>,
    pub ingredients_json: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Joined from profile
    pub profile_name: Option<String>,
    pub profile_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FermentationWithProfile {
    pub fermentation: Fermentation,
    pub profile: FermentationProfile,
}
