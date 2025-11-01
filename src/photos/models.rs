use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PhotoStage {
    Start,
    Progress,
    End,
}

impl PhotoStage {
    pub fn as_str(&self) -> &str {
        match self {
            PhotoStage::Start => "start",
            PhotoStage::Progress => "progress",
            PhotoStage::End => "end",
        }
    }
}

impl From<String> for PhotoStage {
    fn from(s: String) -> Self {
        match s.as_str() {
            "start" => PhotoStage::Start,
            "end" => PhotoStage::End,
            _ => PhotoStage::Progress,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FermentationPhoto {
    pub id: i64,
    pub fermentation_id: i64,
    pub file_path: String,
    pub caption: Option<String>,
    pub taken_at: DateTime<Utc>,
    pub stage: PhotoStage,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PhotoResponse {
    pub id: i64,
    pub fermentation_id: i64,
    pub file_path: String,
    pub caption: Option<String>,
    pub taken_at: DateTime<Utc>,
    pub stage: String,
}

impl From<FermentationPhoto> for PhotoResponse {
    fn from(photo: FermentationPhoto) -> Self {
        Self {
            id: photo.id,
            fermentation_id: photo.fermentation_id,
            file_path: photo.file_path,
            caption: photo.caption,
            taken_at: photo.taken_at,
            stage: photo.stage.as_str().to_string(),
        }
    }
}
