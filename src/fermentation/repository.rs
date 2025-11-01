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

        // Use the find_by_id from main branch which returns Option<Fermentation>
        self.find_by_id(fermentation_id, user_id)
            .await?
            .ok_or_else(|| "Failed to retrieve created fermentation".into())
    }

    pub async fn find_all_by_user(
        &self,
        user_id: i64,
    ) -> Result<Vec<Fermentation>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(
            move || -> Result<Vec<Fermentation>, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                let mut stmt = conn.prepare(
                    "SELECT f.id, f.user_id, f.profile_id, f.name, f.start_date, f.target_end_date,
                        f.actual_end_date, f.status, f.success_rating, f.notes, f.ingredients_json,
                        f.created_at, f.updated_at, p.name as profile_name, p.type as profile_type
                 FROM fermentations f
                 LEFT JOIN fermentation_profiles p ON f.profile_id = p.id
                 WHERE f.user_id = ?1
                 ORDER BY f.created_at DESC",
                )?;

                let fermentations = stmt
                    .query_map([user_id], |row| {
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
                            profile_name: row.get(13)?,
                            profile_type: row.get(14)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(fermentations)
            },
        )
        .await?
    }

    pub async fn find_by_id(
        &self,
        id: i64,
        user_id: i64,
    ) -> Result<Option<Fermentation>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(
            move || -> Result<Option<Fermentation>, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                let mut stmt = conn.prepare(
                    "SELECT f.id, f.user_id, f.profile_id, f.name, f.start_date, f.target_end_date,
                        f.actual_end_date, f.status, f.success_rating, f.notes, f.ingredients_json,
                        f.created_at, f.updated_at, p.name as profile_name, p.type as profile_type
                 FROM fermentations f
                 LEFT JOIN fermentation_profiles p ON f.profile_id = p.id
                 WHERE f.id = ?1 AND f.user_id = ?2",
                )?;

                let fermentation = stmt
                    .query_row([id, user_id], |row| {
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
                            profile_name: row.get(13)?,
                            profile_type: row.get(14)?,
                        })
                    })
                    .optional()?;

                Ok(fermentation)
            },
        )
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
                "SELECT id, name, type, min_days, max_days, temp_min, temp_max, description, is_active, created_at
                 FROM fermentation_profiles WHERE id = ?1"
            )?;

            let profile = stmt.query_row([id], |row| {
                Ok(FermentationProfile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    r#type: row.get(2)?,
                    min_days: row.get(3)?,
                    max_days: row.get(4)?,
                    temp_min: row.get(5)?,
                    temp_max: row.get(6)?,
                    description: row.get(7)?,
                    is_active: row.get::<_, i32>(8)? != 0,
                    created_at: parse_datetime(row.get::<_, String>(9)?),
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
                "SELECT id, name, type, min_days, max_days, temp_min, temp_max, description, is_active, created_at
                 FROM fermentation_profiles WHERE is_active = 1 ORDER BY name"
            )?;

            let profiles = stmt.query_map([], |row| {
                Ok(FermentationProfile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    r#type: row.get(2)?,
                    min_days: row.get(3)?,
                    max_days: row.get(4)?,
                    temp_min: row.get(5)?,
                    temp_max: row.get(6)?,
                    description: row.get(7)?,
                    is_active: row.get::<_, i32>(8)? != 0,
                    created_at: parse_datetime(row.get::<_, String>(9)?),
                })
            })?.collect::<Result<Vec<_>, _>>()?;

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
        .unwrap_or_else(|| {
            tracing::warn!(
                "Failed to parse datetime '{}', falling back to current time",
                s
            );
            Utc::now()
        })
}
