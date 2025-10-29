use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
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
        // Read migration file
        let migration_sql = include_str!("../migrations/001_initial_schema.sql");
        
        let migrations = Migrations::new(vec![
            M::up(migration_sql),
        ]);

        // Apply migrations - need to move the migrations into the closure
        let mut conn = self.connection.lock().unwrap();
        migrations.to_latest(&mut conn)?;

        Ok(())
    }

    pub async fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.connection.lock().unwrap();
        // Simple health check - verify we can query the database
        let _result: i32 = conn.query_row("SELECT 1", [], |row| row.get(0))?;
        Ok(())
    }

    pub fn get_connection(&self) -> &Mutex<Connection> {
        &self.connection
    }
}