use crate::database::Database;
use crate::photos::models::{FermentationPhoto, PhotoStage};
use chrono::{DateTime, Utc};
use rusqlite::OptionalExtension;
use std::sync::Arc;

pub struct PhotoRepository {
    db: Arc<Database>,
}

impl PhotoRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn create_photo(
        &self,
        fermentation_id: i64,
        file_path: String,
        caption: Option<String>,
        taken_at: DateTime<Utc>,
        stage: PhotoStage,
    ) -> Result<FermentationPhoto, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();
        let stage_str = stage.as_str().to_string();

        let photo_id = tokio::task::spawn_blocking(move || -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            let taken_at_str = taken_at.format("%Y-%m-%d %H:%M:%S").to_string();

            conn.execute(
                "INSERT INTO fermentation_photos (fermentation_id, file_path, caption, taken_at, stage)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    fermentation_id,
                    &file_path,
                    caption,
                    &taken_at_str,
                    &stage_str,
                ],
            )?;

            let photo_id = conn.last_insert_rowid();
            Ok(photo_id)
        })
        .await??;

        self.find_by_id(photo_id)
            .await?
            .ok_or_else(|| "Failed to retrieve created photo".into())
    }

    pub async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<FermentationPhoto>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(
            move || -> Result<Option<FermentationPhoto>, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                let mut stmt = conn.prepare(
                    "SELECT id, fermentation_id, file_path, caption, taken_at, stage, created_at
                     FROM fermentation_photos WHERE id = ?1",
                )?;

                let photo = stmt
                    .query_row([id], |row| {
                        Ok(FermentationPhoto {
                            id: row.get(0)?,
                            fermentation_id: row.get(1)?,
                            file_path: row.get(2)?,
                            caption: row.get(3)?,
                            taken_at: parse_datetime(row.get::<_, String>(4)?),
                            stage: PhotoStage::from(row.get::<_, String>(5)?),
                            created_at: parse_datetime(row.get::<_, String>(6)?),
                        })
                    })
                    .optional()?;

                Ok(photo)
            },
        )
        .await?
    }

    pub async fn find_by_fermentation(
        &self,
        fermentation_id: i64,
    ) -> Result<Vec<FermentationPhoto>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(
            move || -> Result<Vec<FermentationPhoto>, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                let mut stmt = conn.prepare(
                    "SELECT id, fermentation_id, file_path, caption, taken_at, stage, created_at
                     FROM fermentation_photos 
                     WHERE fermentation_id = ?1
                     ORDER BY taken_at ASC, created_at ASC",
                )?;

                let photos = stmt
                    .query_map([fermentation_id], |row| {
                        Ok(FermentationPhoto {
                            id: row.get(0)?,
                            fermentation_id: row.get(1)?,
                            file_path: row.get(2)?,
                            caption: row.get(3)?,
                            taken_at: parse_datetime(row.get::<_, String>(4)?),
                            stage: PhotoStage::from(row.get::<_, String>(5)?),
                            created_at: parse_datetime(row.get::<_, String>(6)?),
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(photos)
            },
        )
        .await?
    }

    /// Get the thumbnail photo for a fermentation based on its status
    /// For active/paused fermentations: returns first "start" stage photo
    /// For completed/failed fermentations: returns first "end" stage photo, falling back to first "start" stage photo
    pub async fn get_thumbnail_for_fermentation(
        &self,
        fermentation_id: i64,
        fermentation_status: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();
        let status = fermentation_status.to_string();

        tokio::task::spawn_blocking(
            move || -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                // Helper function to query photo by stage
                let query_photo_by_stage = |stage: &str| -> rusqlite::Result<Option<String>> {
                    let mut stmt = conn.prepare(
                        "SELECT file_path FROM fermentation_photos 
                         WHERE fermentation_id = ?1 AND stage = ?2
                         ORDER BY taken_at ASC, created_at ASC
                         LIMIT 1",
                    )?;

                    stmt.query_row([fermentation_id.to_string(), stage.to_string()], |row| {
                        row.get::<_, String>(0)
                    })
                    .optional()
                };

                // Determine which stage to prioritize based on fermentation status
                // Using string literals that match FermentationStatus::as_str() values
                let (primary_stage, fallback_stage) = if status == "completed" || status == "failed" {
                    // For finished fermentations, prefer "end" stage, fallback to "start"
                    ("end", Some("start"))
                } else {
                    // For active/paused fermentations, only look for "start" stage
                    ("start", None)
                };

                // Try to get the primary stage photo first
                if let Some(photo) = query_photo_by_stage(primary_stage)? {
                    return Ok(Some(photo));
                }

                // Try fallback stage if specified
                if let Some(fallback) = fallback_stage {
                    if let Some(photo) = query_photo_by_stage(fallback)? {
                        return Ok(Some(photo));
                    }
                }

                Ok(None)
            },
        )
        .await?
    }
}

fn parse_datetime(s: String) -> DateTime<Utc> {
    // SQLite stores timestamps as strings, parse them
    // Format: YYYY-MM-DD HH:MM:SS
    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|| {
            tracing::warn!(
                "Failed to parse datetime '{}', falling back to current time",
                s
            );
            Utc::now()
        })
}
