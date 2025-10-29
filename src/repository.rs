use crate::auth::hash_password;
use crate::database::Database;
use crate::models::{CreateUserRequest, ExperienceLevel, User, UserRole};
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

        let user_id = tokio::task::spawn_blocking(move || -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
            let conn = db.get_connection().lock().unwrap();
            
            conn.execute(
                "INSERT INTO users (email, password_hash, role, experience_level) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![&email, &password_hash, "user", &experience_level_str],
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
                "SELECT id, email, password_hash, role, experience_level, created_at, updated_at 
                 FROM users WHERE email = ?1"
            )?;

            let user = stmt.query_row([&email], |row| {
                Ok(User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    password_hash: row.get(2)?,
                    role: UserRole::from(row.get::<_, String>(3)?),
                    experience_level: ExperienceLevel::from(row.get::<_, String>(4)?),
                    created_at: parse_datetime(row.get::<_, String>(5)?),
                    updated_at: parse_datetime(row.get::<_, String>(6)?),
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
                "SELECT id, email, password_hash, role, experience_level, created_at, updated_at 
                 FROM users WHERE id = ?1"
            )?;

            let user = stmt.query_row([id], |row| {
                Ok(User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    password_hash: row.get(2)?,
                    role: UserRole::from(row.get::<_, String>(3)?),
                    experience_level: ExperienceLevel::from(row.get::<_, String>(4)?),
                    created_at: parse_datetime(row.get::<_, String>(5)?),
                    updated_at: parse_datetime(row.get::<_, String>(6)?),
                })
            })?;

            Ok(user)
        })
        .await?
    }
}

fn parse_datetime(s: String) -> DateTime<Utc> {
    // SQLite stores timestamps as strings, parse them
    // Format: YYYY-MM-DD HH:MM:SS
    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
        .ok()
        .and_then(|dt| dt.and_utc().into())
        .unwrap_or_else(Utc::now)
}
