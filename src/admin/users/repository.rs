use crate::database::Database;
use crate::users::auth::hash_password;
use crate::users::models::{ExperienceLevel, TemperatureUnit, User, UserRole};
use chrono::{DateTime, Utc};
use rusqlite::OptionalExtension;
use std::sync::Arc;

pub struct AdminUserRepository {
    db: Arc<Database>,
}

impl AdminUserRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// List all users (admin only)
    pub async fn list_all_users(
        &self,
    ) -> Result<Vec<User>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(move || -> Result<Vec<User>, Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            let mut stmt = conn.prepare(
                "SELECT id, email, password_hash, role, experience_level, preferred_temp_unit, first_name, last_name, is_locked, created_at, updated_at
                 FROM users ORDER BY created_at DESC"
            )?;

            let users = stmt
                .query_map([], |row| {
                    Ok(User {
                        id: row.get(0)?,
                        email: row.get(1)?,
                        password_hash: row.get(2)?,
                        role: UserRole::from(row.get::<_, String>(3)?),
                        experience_level: ExperienceLevel::from(row.get::<_, String>(4)?),
                        preferred_temp_unit: TemperatureUnit::from(row.get::<_, String>(5)?),
                        first_name: row.get(6)?,
                        last_name: row.get(7)?,
                        is_locked: row.get::<_, i64>(8)? != 0,
                        created_at: parse_datetime(row.get::<_, String>(9)?),
                        updated_at: parse_datetime(row.get::<_, String>(10)?),
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(users)
        })
        .await?
    }

    /// Create a new user as admin (can set role)
    pub async fn create_user_as_admin(
        &self,
        email: String,
        password: String,
        role: UserRole,
        experience_level: ExperienceLevel,
        first_name: Option<String>,
        last_name: Option<String>,
    ) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        let password_hash = hash_password(&password)?;
        let db = self.db.clone();
        let role_str = role.as_str().to_string();
        let experience_level_str = experience_level.as_str().to_string();

        let user_id = tokio::task::spawn_blocking(move || -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            conn.execute(
                "INSERT INTO users (email, password_hash, role, experience_level, first_name, last_name)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![&email, &password_hash, &role_str, &experience_level_str, &first_name, &last_name],
            )?;

            let user_id = conn.last_insert_rowid();
            Ok(user_id)
        })
        .await??;

        self.find_by_id(user_id).await
    }

    /// Update user details as admin
    pub async fn update_user_as_admin(
        &self,
        user_id: i64,
        email: String,
        role: UserRole,
        experience_level: ExperienceLevel,
        first_name: Option<String>,
        last_name: Option<String>,
    ) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();
        let role_str = role.as_str().to_string();
        let experience_level_str = experience_level.as_str().to_string();

        tokio::task::spawn_blocking(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            conn.execute(
                "UPDATE users SET email = ?1, role = ?2, experience_level = ?3, first_name = ?4, last_name = ?5, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?6",
                rusqlite::params![&email, &role_str, &experience_level_str, &first_name, &last_name, user_id],
            )?;

            Ok(())
        })
        .await??;

        self.find_by_id(user_id).await
    }

    /// Lock or unlock a user account
    pub async fn lock_user(
        &self,
        user_id: i64,
        locked: bool,
    ) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();
        let is_locked = if locked { 1 } else { 0 };

        tokio::task::spawn_blocking(
            move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                conn.execute(
                    "UPDATE users SET is_locked = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                    rusqlite::params![is_locked, user_id],
                )?;

                Ok(())
            },
        )
        .await??;

        self.find_by_id(user_id).await
    }

    /// Delete a user (admin only)
    pub async fn delete_user(
        &self,
        user_id: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(
            move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                conn.execute(
                    "DELETE FROM users WHERE id = ?1",
                    rusqlite::params![user_id],
                )?;

                Ok(())
            },
        )
        .await?
    }

    /// Find user by ID (helper method)
    async fn find_by_id(&self, id: i64) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(move || -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            let mut stmt = conn.prepare(
                "SELECT id, email, password_hash, role, experience_level, preferred_temp_unit, first_name, last_name, is_locked, created_at, updated_at
                 FROM users WHERE id = ?1"
            )?;

            let user = stmt.query_row([id], |row| {
                Ok(User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    password_hash: row.get(2)?,
                    role: UserRole::from(row.get::<_, String>(3)?),
                    experience_level: ExperienceLevel::from(row.get::<_, String>(4)?),
                    preferred_temp_unit: TemperatureUnit::from(row.get::<_, String>(5)?),
                    first_name: row.get(6)?,
                    last_name: row.get(7)?,
                    is_locked: row.get::<_, i64>(8)? != 0,
                    created_at: parse_datetime(row.get::<_, String>(9)?),
                    updated_at: parse_datetime(row.get::<_, String>(10)?),
                })
            })?;

            Ok(user)
        })
        .await?
    }

    /// Check if email already exists (helper method)
    pub async fn email_exists(
        &self,
        email: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();
        let email = email.to_string();

        tokio::task::spawn_blocking(
            move || -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
                let conn = db.get_connection().lock().unwrap();

                let mut stmt = conn.prepare("SELECT id FROM users WHERE email = ?1")?;
                let result = stmt.query_row([&email], |_row| Ok(())).optional()?;

                Ok(result.is_some())
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
        .unwrap_or_else(Utc::now)
}
