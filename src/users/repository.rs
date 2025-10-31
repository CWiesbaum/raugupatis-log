use crate::database::Database;
use crate::users::auth::hash_password;
use crate::users::models::{CreateUserRequest, ExperienceLevel, User, UserRole};
use chrono::{DateTime, Utc};
use rusqlite::OptionalExtension;
use std::sync::Arc;

pub struct UserRepository {
    db: Arc<Database>,
}

impl UserRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn create_user(
        &self,
        request: CreateUserRequest,
    ) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        let password_hash = hash_password(&request.password)?;
        let experience_level = request
            .experience_level
            .map(ExperienceLevel::from)
            .unwrap_or(ExperienceLevel::Beginner);

        let db = self.db.clone();
        let email = request.email.clone();
        let experience_level_str = experience_level.as_str().to_string();
        let first_name = request.first_name.clone();
        let last_name = request.last_name.clone();

        let user_id = tokio::task::spawn_blocking(move || -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            conn.execute(
                "INSERT INTO users (email, password_hash, role, experience_level, first_name, last_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![&email, &password_hash, "user", &experience_level_str, &first_name, &last_name],
            )?;

            let user_id = conn.last_insert_rowid();
            Ok(user_id)
        })
        .await??;

        self.find_by_id(user_id).await
    }

    pub async fn find_by_email(
        &self,
        email: &str,
    ) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();
        let email = email.to_string();

        tokio::task::spawn_blocking(move || -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            let mut stmt = conn.prepare(
                "SELECT id, email, password_hash, role, experience_level, first_name, last_name, is_locked, created_at, updated_at
                 FROM users WHERE email = ?1"
            )?;

            let user = stmt.query_row([&email], |row| {
                Ok(User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    password_hash: row.get(2)?,
                    role: UserRole::from(row.get::<_, String>(3)?),
                    experience_level: ExperienceLevel::from(row.get::<_, String>(4)?),
                    first_name: row.get(5)?,
                    last_name: row.get(6)?,
                    is_locked: row.get::<_, i64>(7)? != 0,
                    created_at: parse_datetime(row.get::<_, String>(8)?),
                    updated_at: parse_datetime(row.get::<_, String>(9)?),
                })
            }).optional()?;

            Ok(user)
        })
        .await?
    }

    pub async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(move || -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            let mut stmt = conn.prepare(
                "SELECT id, email, password_hash, role, experience_level, first_name, last_name, is_locked, created_at, updated_at
                 FROM users WHERE id = ?1"
            )?;

            let user = stmt.query_row([id], |row| {
                Ok(User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    password_hash: row.get(2)?,
                    role: UserRole::from(row.get::<_, String>(3)?),
                    experience_level: ExperienceLevel::from(row.get::<_, String>(4)?),
                    first_name: row.get(5)?,
                    last_name: row.get(6)?,
                    is_locked: row.get::<_, i64>(7)? != 0,
                    created_at: parse_datetime(row.get::<_, String>(8)?),
                    updated_at: parse_datetime(row.get::<_, String>(9)?),
                })
            })?;

            Ok(user)
        })
        .await?
    }

    pub async fn update_experience_level(
        &self,
        user_id: i64,
        experience_level: ExperienceLevel,
    ) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();
        let experience_level_str = experience_level.as_str().to_string();

        tokio::task::spawn_blocking(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            conn.execute(
                "UPDATE users SET experience_level = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                rusqlite::params![&experience_level_str, user_id],
            )?;

            Ok(())
        })
        .await??;

        self.find_by_id(user_id).await
    }

    pub async fn update_profile(
        &self,
        user_id: i64,
        experience_level: ExperienceLevel,
        first_name: Option<String>,
        last_name: Option<String>,
    ) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();
        let experience_level_str = experience_level.as_str().to_string();

        tokio::task::spawn_blocking(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            conn.execute(
                "UPDATE users SET experience_level = ?1, first_name = ?2, last_name = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4",
                rusqlite::params![&experience_level_str, &first_name, &last_name, user_id],
            )?;

            Ok(())
        })
        .await??;

        self.find_by_id(user_id).await
    }

    pub async fn update_password(
        &self,
        user_id: i64,
        new_password_hash: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();

            conn.execute(
                "UPDATE users SET password_hash = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                rusqlite::params![&new_password_hash, user_id],
            )?;

            Ok(())
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
