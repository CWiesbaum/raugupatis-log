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
