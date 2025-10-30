use crate::database::Database;
use crate::fermentation::models::{
    CreateFermentationRequest, Fermentation, FermentationProfile, FermentationStatus,
};
use chrono::{DateTime, Utc};
use rusqlite::OptionalExtension;
use std::sync::Arc;

pub struct FermentationRepository {
    db: Arc<Database>,
}

impl FermentationRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn create_fermentation(
        &self,
        user_id: i64,
        request: CreateFermentationRequest,
    ) -> Result<Fermentation, Box<dyn std::error::Error + Send + Sync>> {
        // Parse dates
        let start_date = DateTime::parse_from_rfc3339(&request.start_date)
            .map_err(|e| format!("Invalid start_date format: {}", e))?
            .with_timezone(&Utc);

        let target_end_date = if let Some(ref date_str) = request.target_end_date {
            Some(
                DateTime::parse_from_rfc3339(date_str)
                    .map_err(|e| format!("Invalid target_end_date format: {}", e))?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        let db = self.db.clone();
        let name = request.name.clone();
        let notes = request.notes.clone();
        let ingredients_json = request.ingredients.clone();
        let profile_id = request.profile_id;

        let fermentation_id = tokio::task::spawn_blocking(move || -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();
            
            let start_date_str = start_date.format("%Y-%m-%d %H:%M:%S").to_string();
            let target_end_date_str = target_end_date
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());
            
            conn.execute(
                "INSERT INTO fermentations (user_id, profile_id, name, start_date, target_end_date, status, notes, ingredients_json) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    user_id,
                    profile_id,
                    &name,
                    &start_date_str,
                    target_end_date_str,
                    "active",
                    notes,
                    ingredients_json,
                ],
            )?;
            
            let fermentation_id = conn.last_insert_rowid();
            Ok(fermentation_id)
        })
        .await??;

        self.find_by_id(fermentation_id).await
    }

    pub async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Fermentation, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(move || -> Result<Fermentation, Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT id, user_id, profile_id, name, start_date, target_end_date, actual_end_date, 
                        status, success_rating, notes, ingredients_json, created_at, updated_at
                 FROM fermentations WHERE id = ?1"
            )?;

            let fermentation = stmt.query_row([id], |row| {
                Ok(Fermentation {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    profile_id: row.get(2)?,
                    name: row.get(3)?,
                    start_date: parse_datetime(row.get::<_, String>(4)?),
                    target_end_date: row.get::<_, Option<String>>(5)?.map(parse_datetime),
                    actual_end_date: row.get::<_, Option<String>>(6)?.map(parse_datetime),
                    status: FermentationStatus::from(row.get::<_, String>(7)?),
                    success_rating: row.get(8)?,
                    notes: row.get(9)?,
                    ingredients_json: row.get(10)?,
                    created_at: parse_datetime(row.get::<_, String>(11)?),
                    updated_at: parse_datetime(row.get::<_, String>(12)?),
                })
            })?;

            Ok(fermentation)
        })
        .await?
    }

    pub async fn get_profile_by_id(
        &self,
        id: i64,
    ) -> Result<Option<FermentationProfile>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(move || -> Result<Option<FermentationProfile>, Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT id, name, type, min_days, max_days, temp_min, temp_max, description
                 FROM fermentation_profiles WHERE id = ?1"
            )?;

            let profile = stmt.query_row([id], |row| {
                Ok(FermentationProfile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    type_name: row.get(2)?,
                    min_days: row.get(3)?,
                    max_days: row.get(4)?,
                    temp_min: row.get(5)?,
                    temp_max: row.get(6)?,
                    description: row.get(7)?,
                })
            }).optional()?;

            Ok(profile)
        })
        .await?
    }

    pub async fn get_all_profiles(
        &self,
    ) -> Result<Vec<FermentationProfile>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(move || -> Result<Vec<FermentationProfile>, Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT id, name, type, min_days, max_days, temp_min, temp_max, description
                 FROM fermentation_profiles ORDER BY name"
            )?;

            let profiles = stmt.query_map([], |row| {
                Ok(FermentationProfile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    type_name: row.get(2)?,
                    min_days: row.get(3)?,
                    max_days: row.get(4)?,
                    temp_min: row.get(5)?,
                    temp_max: row.get(6)?,
                    description: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

            Ok(profiles)
        })
        .await?
    }
}

fn parse_datetime(s: String) -> DateTime<Utc> {
    // SQLite stores timestamps as strings, parse them
    // Format: YYYY-MM-DD HH:MM:SS
    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|dt| dt.and_utc())
        .unwrap_or_else(Utc::now)
}
