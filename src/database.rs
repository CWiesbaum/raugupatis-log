use rusqlite::Connection;
use std::sync::Mutex;
use tokio::task;

pub struct Database {
    connection: Mutex<Connection>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let db_path = database_url.strip_prefix("sqlite:").unwrap_or(database_url).to_string();
        
        // Ensure the data directory exists
        if let Some(parent) = std::path::Path::new(&db_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let connection = task::spawn_blocking(move || -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
            let conn = Connection::open(&db_path)?;
            Ok(conn)
        }).await??;

        Ok(Database {
            connection: Mutex::new(connection),
        })
    }

    pub async fn migrate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For now, we'll skip the actual migration and just ensure the database file exists
        // In a proper implementation, this would run the migrations using rusqlite_migration
        task::spawn_blocking(|| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            // Placeholder for migration logic - would implement actual migrations here
            Ok(())
        }).await??;

        Ok(())
    }

    pub async fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        task::spawn_blocking(|| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            // Simple health check - in a real implementation, would test database connectivity
            Ok(())
        }).await??;

        Ok(())
    }
}