use crate::database::Database;
use crate::fermentation::models::{
    CreateFermentationRequest, CreateTemperatureLogRequest, Fermentation, FermentationListQuery,
    FermentationProfile, FermentationStatus, TemperatureLog, UpdateFermentationRequest,
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
        query: &FermentationListQuery,
    ) -> Result<Vec<Fermentation>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();
        let search = query.search.clone();
        let status = query.status.clone();
        let profile_type = query.profile_type.clone();
        let sort_by = query
            .sort_by
            .clone()
            .unwrap_or_else(|| "created_at".to_string());
        let sort_order = query
            .sort_order
            .clone()
            .unwrap_or_else(|| "desc".to_string());

        tokio::task::spawn_blocking(
            move || -> Result<Vec<Fermentation>, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                // Build dynamic WHERE clause
                let mut where_clauses = vec!["f.user_id = ?1".to_string()];
                let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(user_id)];

                // Add search filter
                if let Some(search_term) = search {
                    if !search_term.trim().is_empty() {
                        where_clauses.push(
                            "(f.name LIKE ? OR f.notes LIKE ? OR f.ingredients_json LIKE ?)"
                                .to_string(),
                        );
                        let search_pattern = format!("%{}%", search_term);
                        params.push(Box::new(search_pattern.clone()));
                        params.push(Box::new(search_pattern.clone()));
                        params.push(Box::new(search_pattern));
                    }
                }

                // Add status filter
                if let Some(status_filter) = status {
                    if !status_filter.trim().is_empty() {
                        where_clauses.push("f.status = ?".to_string());
                        params.push(Box::new(status_filter));
                    }
                }

                // Add profile type filter
                if let Some(profile_type_filter) = profile_type {
                    if !profile_type_filter.trim().is_empty() {
                        where_clauses.push("p.type = ?".to_string());
                        params.push(Box::new(profile_type_filter));
                    }
                }

                // Build ORDER BY clause
                let sort_column = match sort_by.as_str() {
                    "name" => "f.name",
                    "start_date" => "f.start_date",
                    "status" => "f.status",
                    _ => "f.created_at",
                };

                let order_direction = if sort_order.to_lowercase() == "asc" {
                    "ASC"
                } else {
                    "DESC"
                };

                let query = format!(
                    "SELECT f.id, f.user_id, f.profile_id, f.name, f.start_date, f.target_end_date,
                        f.actual_end_date, f.status, f.success_rating, f.notes, f.ingredients_json,
                        f.lessons_learned, f.created_at, f.updated_at, p.name as profile_name, p.type as profile_type
                     FROM fermentations f
                     LEFT JOIN fermentation_profiles p ON f.profile_id = p.id
                     WHERE {}
                     ORDER BY {} {}",
                    where_clauses.join(" AND "),
                    sort_column,
                    order_direction
                );

                let mut stmt = conn.prepare(&query)?;
                let params_refs: Vec<&dyn rusqlite::ToSql> =
                    params.iter().map(|p| p.as_ref()).collect();

                let fermentations = stmt
                    .query_map(params_refs.as_slice(), |row| {
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
                            lessons_learned: row.get(11)?,
                            created_at: parse_datetime(row.get::<_, String>(12)?),
                            updated_at: parse_datetime(row.get::<_, String>(13)?),
                            profile_name: row.get(14)?,
                            profile_type: row.get(15)?,
                            thumbnail_path: None,
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
                        f.lessons_learned, f.created_at, f.updated_at, p.name as profile_name, p.type as profile_type
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
                            lessons_learned: row.get(11)?,
                            created_at: parse_datetime(row.get::<_, String>(12)?),
                            updated_at: parse_datetime(row.get::<_, String>(13)?),
                            profile_name: row.get(14)?,
                            profile_type: row.get(15)?,
                            thumbnail_path: None,
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

    pub async fn update_fermentation(
        &self,
        id: i64,
        user_id: i64,
        request: UpdateFermentationRequest,
    ) -> Result<Option<Fermentation>, Box<dyn std::error::Error + Send + Sync>> {
        // First verify the fermentation exists and belongs to the user
        if self.find_by_id(id, user_id).await?.is_none() {
            return Ok(None);
        }

        // Parse dates if provided
        let start_date = if let Some(ref date_str) = request.start_date {
            Some(
                DateTime::parse_from_rfc3339(date_str)
                    .map_err(|e| format!("Invalid start_date format: {}", e))?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        let target_end_date = if let Some(ref date_str) = request.target_end_date {
            Some(
                DateTime::parse_from_rfc3339(date_str)
                    .map_err(|e| format!("Invalid target_end_date format: {}", e))?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        let actual_end_date = if let Some(ref date_str) = request.actual_end_date {
            Some(
                DateTime::parse_from_rfc3339(date_str)
                    .map_err(|e| format!("Invalid actual_end_date format: {}", e))?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        let db = self.db.clone();
        let name = request.name.clone();
        let status = request.status.clone();
        let success_rating = request.success_rating;
        let notes = request.notes.clone();
        let ingredients_json = request.ingredients.clone();

        tokio::task::spawn_blocking(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            // Build dynamic UPDATE query based on provided fields
            let mut updates = Vec::new();
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(n) = name {
                updates.push("name = ?");
                params.push(Box::new(n));
            }

            if let Some(d) = start_date {
                updates.push("start_date = ?");
                params.push(Box::new(d.format("%Y-%m-%d %H:%M:%S").to_string()));
            }

            if request.target_end_date.is_some() {
                if let Some(d) = target_end_date {
                    updates.push("target_end_date = ?");
                    params.push(Box::new(d.format("%Y-%m-%d %H:%M:%S").to_string()));
                } else {
                    updates.push("target_end_date = NULL");
                }
            }

            if request.actual_end_date.is_some() {
                if let Some(d) = actual_end_date {
                    updates.push("actual_end_date = ?");
                    params.push(Box::new(d.format("%Y-%m-%d %H:%M:%S").to_string()));
                } else {
                    updates.push("actual_end_date = NULL");
                }
            }

            if let Some(s) = status {
                updates.push("status = ?");
                params.push(Box::new(s));
            }

            if request.success_rating.is_some() {
                if let Some(r) = success_rating {
                    updates.push("success_rating = ?");
                    params.push(Box::new(r));
                } else {
                    updates.push("success_rating = NULL");
                }
            }

            if request.notes.is_some() {
                if let Some(n) = notes {
                    updates.push("notes = ?");
                    params.push(Box::new(n));
                } else {
                    updates.push("notes = NULL");
                }
            }

            if request.ingredients.is_some() {
                if let Some(i) = ingredients_json {
                    updates.push("ingredients_json = ?");
                    params.push(Box::new(i));
                } else {
                    updates.push("ingredients_json = NULL");
                }
            }

            // Always update the updated_at timestamp
            updates.push("updated_at = CURRENT_TIMESTAMP");

            if updates.is_empty() {
                return Ok(());
            }

            let query = format!(
                "UPDATE fermentations SET {} WHERE id = ? AND user_id = ?",
                updates.join(", ")
            );

            params.push(Box::new(id));
            params.push(Box::new(user_id));

            let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            conn.execute(&query, params_refs.as_slice())?;

            Ok(())
        })
        .await??;

        // Return the updated fermentation
        self.find_by_id(id, user_id).await
    }

    pub async fn create_temperature_log(
        &self,
        fermentation_id: i64,
        user_id: i64,
        request: CreateTemperatureLogRequest,
    ) -> Result<TemperatureLog, Box<dyn std::error::Error + Send + Sync>> {
        // Verify the fermentation exists and belongs to the user
        if self.find_by_id(fermentation_id, user_id).await?.is_none() {
            return Err("Fermentation not found".into());
        }

        // Parse recorded_at date or use current time
        let recorded_at = if let Some(ref date_str) = request.recorded_at {
            DateTime::parse_from_rfc3339(date_str)
                .map_err(|e| format!("Invalid recorded_at format: {}", e))?
                .with_timezone(&Utc)
        } else {
            Utc::now()
        };

        let db = self.db.clone();
        let temperature = request.temperature;
        let notes = request.notes.clone();

        let log_id = tokio::task::spawn_blocking(move || -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            let recorded_at_str = recorded_at.format("%Y-%m-%d %H:%M:%S").to_string();

            conn.execute(
                "INSERT INTO temperature_logs (fermentation_id, recorded_at, temperature, notes)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    fermentation_id,
                    &recorded_at_str,
                    temperature,
                    notes,
                ],
            )?;

            let log_id = conn.last_insert_rowid();
            Ok(log_id)
        })
        .await??;

        // Retrieve the created log
        self.find_temperature_log_by_id(log_id).await?
            .ok_or_else(|| "Failed to retrieve created temperature log".into())
    }

    pub async fn find_temperature_logs_by_fermentation(
        &self,
        fermentation_id: i64,
        user_id: i64,
    ) -> Result<Vec<TemperatureLog>, Box<dyn std::error::Error + Send + Sync>> {
        // Verify the fermentation exists and belongs to the user
        if self.find_by_id(fermentation_id, user_id).await?.is_none() {
            return Err("Fermentation not found".into());
        }

        let db = self.db.clone();

        tokio::task::spawn_blocking(
            move || -> Result<Vec<TemperatureLog>, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                let mut stmt = conn.prepare(
                    "SELECT id, fermentation_id, recorded_at, temperature, notes, created_at
                     FROM temperature_logs
                     WHERE fermentation_id = ?1
                     ORDER BY recorded_at DESC",
                )?;

                let logs = stmt
                    .query_map([fermentation_id], |row| {
                        Ok(TemperatureLog {
                            id: row.get(0)?,
                            fermentation_id: row.get(1)?,
                            recorded_at: parse_datetime(row.get::<_, String>(2)?),
                            temperature: row.get(3)?,
                            notes: row.get(4)?,
                            created_at: parse_datetime(row.get::<_, String>(5)?),
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(logs)
            },
        )
        .await?
    }

    async fn find_temperature_log_by_id(
        &self,
        id: i64,
    ) -> Result<Option<TemperatureLog>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(
            move || -> Result<Option<TemperatureLog>, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                let mut stmt = conn.prepare(
                    "SELECT id, fermentation_id, recorded_at, temperature, notes, created_at
                     FROM temperature_logs
                     WHERE id = ?1",
                )?;

                let log = stmt
                    .query_row([id], |row| {
                        Ok(TemperatureLog {
                            id: row.get(0)?,
                            fermentation_id: row.get(1)?,
                            recorded_at: parse_datetime(row.get::<_, String>(2)?),
                            temperature: row.get(3)?,
                            notes: row.get(4)?,
                            created_at: parse_datetime(row.get::<_, String>(5)?),
                        })
                    })
                    .optional()?;

                Ok(log)
            },
        )
        .await?
    }

    pub async fn finish_fermentation(
        &self,
        fermentation_id: i64,
        user_id: i64,
        request: crate::fermentation::models::FinishFermentationRequest,
    ) -> Result<Option<Fermentation>, Box<dyn std::error::Error + Send + Sync>> {
        // Verify the fermentation exists and belongs to the user
        let fermentation = self.find_by_id(fermentation_id, user_id).await?;
        if fermentation.is_none() {
            return Ok(None);
        }

        let db = self.db.clone();
        let success_rating = request.success_rating;
        let lessons_learned = request.lessons_learned.clone();
        let taste_profile = request.taste_profile.clone();

        tokio::task::spawn_blocking(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            // Update fermentation to completed status
            let now = Utc::now();
            let actual_end_date_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
            params.push(Box::new("completed".to_string()));
            params.push(Box::new(actual_end_date_str));

            if let Some(rating) = success_rating {
                params.push(Box::new(rating));
            }

            if let Some(lessons) = lessons_learned {
                params.push(Box::new(lessons));
            }

            params.push(Box::new(fermentation_id));
            params.push(Box::new(user_id));

            let query = if success_rating.is_some() && request.lessons_learned.is_some() {
                "UPDATE fermentations SET status = ?, actual_end_date = ?, success_rating = ?, lessons_learned = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND user_id = ?"
            } else if success_rating.is_some() {
                "UPDATE fermentations SET status = ?, actual_end_date = ?, success_rating = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND user_id = ?"
            } else if request.lessons_learned.is_some() {
                "UPDATE fermentations SET status = ?, actual_end_date = ?, lessons_learned = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND user_id = ?"
            } else {
                "UPDATE fermentations SET status = ?, actual_end_date = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND user_id = ?"
            };

            let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            conn.execute(query, params_refs.as_slice())?;

            // Add initial taste profile if provided
            if let Some(profile_text) = taste_profile {
                if !profile_text.trim().is_empty() {
                    let tasted_at_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
                    conn.execute(
                        "INSERT INTO taste_profiles (fermentation_id, profile_text, tasted_at)
                         VALUES (?1, ?2, ?3)",
                        rusqlite::params![fermentation_id, profile_text, tasted_at_str],
                    )?;
                }
            }

            Ok(())
        })
        .await??;

        // Return the updated fermentation
        self.find_by_id(fermentation_id, user_id).await
    }

    pub async fn create_taste_profile(
        &self,
        fermentation_id: i64,
        user_id: i64,
        request: crate::fermentation::models::CreateTasteProfileRequest,
    ) -> Result<crate::fermentation::models::TasteProfile, Box<dyn std::error::Error + Send + Sync>> {
        // Verify the fermentation exists and belongs to the user
        if self.find_by_id(fermentation_id, user_id).await?.is_none() {
            return Err("Fermentation not found".into());
        }

        // Parse tasted_at date or use current time
        let tasted_at = if let Some(ref date_str) = request.tasted_at {
            DateTime::parse_from_rfc3339(date_str)
                .map_err(|e| format!("Invalid tasted_at format: {}", e))?
                .with_timezone(&Utc)
        } else {
            Utc::now()
        };

        let db = self.db.clone();
        let profile_text = request.profile_text.clone();

        let profile_id = tokio::task::spawn_blocking(move || -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            let tasted_at_str = tasted_at.format("%Y-%m-%d %H:%M:%S").to_string();

            conn.execute(
                "INSERT INTO taste_profiles (fermentation_id, profile_text, tasted_at)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![fermentation_id, profile_text, tasted_at_str],
            )?;

            let profile_id = conn.last_insert_rowid();
            Ok(profile_id)
        })
        .await??;

        // Retrieve the created taste profile
        self.find_taste_profile_by_id(profile_id).await?
            .ok_or_else(|| "Failed to retrieve created taste profile".into())
    }

    pub async fn find_taste_profiles_by_fermentation(
        &self,
        fermentation_id: i64,
        user_id: i64,
    ) -> Result<Vec<crate::fermentation::models::TasteProfile>, Box<dyn std::error::Error + Send + Sync>> {
        // Verify the fermentation exists and belongs to the user
        if self.find_by_id(fermentation_id, user_id).await?.is_none() {
            return Err("Fermentation not found".into());
        }

        let db = self.db.clone();

        tokio::task::spawn_blocking(
            move || -> Result<Vec<crate::fermentation::models::TasteProfile>, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                let mut stmt = conn.prepare(
                    "SELECT id, fermentation_id, profile_text, tasted_at, created_at
                     FROM taste_profiles
                     WHERE fermentation_id = ?1
                     ORDER BY tasted_at DESC",
                )?;

                let profiles = stmt
                    .query_map([fermentation_id], |row| {
                        Ok(crate::fermentation::models::TasteProfile {
                            id: row.get(0)?,
                            fermentation_id: row.get(1)?,
                            profile_text: row.get(2)?,
                            tasted_at: parse_datetime(row.get::<_, String>(3)?),
                            created_at: parse_datetime(row.get::<_, String>(4)?),
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(profiles)
            },
        )
        .await?
    }

    async fn find_taste_profile_by_id(
        &self,
        id: i64,
    ) -> Result<Option<crate::fermentation::models::TasteProfile>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(
            move || -> Result<Option<crate::fermentation::models::TasteProfile>, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                let mut stmt = conn.prepare(
                    "SELECT id, fermentation_id, profile_text, tasted_at, created_at
                     FROM taste_profiles
                     WHERE id = ?1",
                )?;

                let profile = stmt
                    .query_row([id], |row| {
                        Ok(crate::fermentation::models::TasteProfile {
                            id: row.get(0)?,
                            fermentation_id: row.get(1)?,
                            profile_text: row.get(2)?,
                            tasted_at: parse_datetime(row.get::<_, String>(3)?),
                            created_at: parse_datetime(row.get::<_, String>(4)?),
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
