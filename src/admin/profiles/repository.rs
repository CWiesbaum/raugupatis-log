use crate::database::Database;
use crate::fermentation::models::FermentationProfile;
use chrono::{DateTime, Utc};
use rusqlite::OptionalExtension;
use std::sync::Arc;

pub struct AdminProfileRepository {
    db: Arc<Database>,
}

impl AdminProfileRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// List all profiles (including inactive ones) for admin view
    pub async fn list_all_profiles(
        &self,
    ) -> Result<Vec<FermentationProfile>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(
            move || -> Result<Vec<FermentationProfile>, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                let mut stmt = conn.prepare(
                    "SELECT id, name, type, min_days, max_days, temp_min, temp_max, description, is_active, created_at
                     FROM fermentation_profiles ORDER BY name",
                )?;

                let profiles = stmt
                    .query_map([], |row| {
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
                    })?
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(profiles)
            },
        )
        .await?
    }

    /// Create a new profile
    pub async fn create_profile(
        &self,
        request: crate::admin::profiles::models::CreateProfileRequest,
    ) -> Result<FermentationProfile, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();
        let name = request.name;
        let r#type = request.r#type;
        let min_days = request.min_days;
        let max_days = request.max_days;
        let temp_min = request.temp_min;
        let temp_max = request.temp_max;
        let description = request.description;

        let profile_id = tokio::task::spawn_blocking(
            move || -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                conn.execute(
                    "INSERT INTO fermentation_profiles (name, type, min_days, max_days, temp_min, temp_max, description, is_active)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
                    rusqlite::params![name, r#type, min_days, max_days, temp_min, temp_max, description],
                )?;

                let profile_id = conn.last_insert_rowid();
                Ok(profile_id)
            },
        )
        .await??;

        self.get_profile_by_id(profile_id)
            .await?
            .ok_or_else(|| "Failed to retrieve created profile".into())
    }

    /// Copy an existing profile with a new name
    pub async fn copy_profile(
        &self,
        profile_id: i64,
        new_name: String,
    ) -> Result<FermentationProfile, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        let new_profile_id = tokio::task::spawn_blocking(
            move || -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                // Get the source profile
                let mut stmt = conn.prepare(
                    "SELECT type, min_days, max_days, temp_min, temp_max, description
                     FROM fermentation_profiles WHERE id = ?1",
                )?;

                let (profile_type, min_days, max_days, temp_min, temp_max, description): (
                    String,
                    i32,
                    i32,
                    f64,
                    f64,
                    Option<String>,
                ) = stmt.query_row([profile_id], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                })?;

                // Insert the copy with new name
                conn.execute(
                    "INSERT INTO fermentation_profiles (name, type, min_days, max_days, temp_min, temp_max, description, is_active)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
                    rusqlite::params![
                        new_name,
                        profile_type,
                        min_days,
                        max_days,
                        temp_min,
                        temp_max,
                        description
                    ],
                )?;

                let new_profile_id = conn.last_insert_rowid();
                Ok(new_profile_id)
            },
        )
        .await??;

        self.get_profile_by_id(new_profile_id)
            .await?
            .ok_or_else(|| "Failed to retrieve copied profile".into())
    }

    /// Deactivate or reactivate a profile
    pub async fn set_profile_active_status(
        &self,
        profile_id: i64,
        is_active: bool,
    ) -> Result<FermentationProfile, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(
            move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                conn.execute(
                    "UPDATE fermentation_profiles SET is_active = ?1 WHERE id = ?2",
                    rusqlite::params![if is_active { 1 } else { 0 }, profile_id],
                )?;

                Ok(())
            },
        )
        .await??;

        self.get_profile_by_id(profile_id)
            .await?
            .ok_or_else(|| "Profile not found".into())
    }

    /// Check if a profile name already exists
    pub async fn name_exists(
        &self,
        name: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();
        let name = name.to_string();

        tokio::task::spawn_blocking(
            move || -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM fermentation_profiles WHERE name = ?1",
                    [&name],
                    |row| row.get(0),
                )?;

                Ok(count > 0)
            },
        )
        .await?
    }

    /// Get a profile by ID (admin version, includes inactive profiles)
    async fn get_profile_by_id(
        &self,
        id: i64,
    ) -> Result<Option<FermentationProfile>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(
            move || -> Result<Option<FermentationProfile>, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                let mut stmt = conn.prepare(
                    "SELECT id, name, type, min_days, max_days, temp_min, temp_max, description, is_active, created_at
                     FROM fermentation_profiles WHERE id = ?1",
                )?;

                let profile = stmt
                    .query_row([id], |row| {
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
                    })
                    .optional()?;

                Ok(profile)
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
