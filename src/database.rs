use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::sync::Mutex;
use tokio::task;

pub struct Database {
    connection: Mutex<Connection>,
    db_path: String,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let db_path = database_url
            .strip_prefix("sqlite:")
            .unwrap_or(database_url)
            .to_string();

        // Ensure the data directory exists
        if let Some(parent) = std::path::Path::new(&db_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let db_path_clone = db_path.clone();
        let connection = task::spawn_blocking(
            move || -> Result<Connection, Box<dyn std::error::Error + Send + Sync>> {
                let conn = Connection::open(&db_path_clone)?;
                Ok(conn)
            },
        )
        .await??;

        Ok(Database {
            connection: Mutex::new(connection),
            db_path,
        })
    }

    pub async fn migrate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Read migration files
        let migration_sql_001 = include_str!("../migrations/001_initial_schema.sql");
        let migration_sql_002 = include_str!("../migrations/002_add_sessions_table.sql");
        let migration_sql_003 = include_str!("../migrations/003_add_user_names.sql");
        let migration_sql_004 = include_str!("../migrations/004_add_user_locked_field.sql");
        let migration_sql_005 = include_str!("../migrations/005_add_profile_active_field.sql");

        let migrations = Migrations::new(vec![
            M::up(migration_sql_001),
            M::up(migration_sql_002),
            M::up(migration_sql_003),
            M::up(migration_sql_004),
            M::up(migration_sql_005),
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

    pub fn get_db_path(&self) -> &str {
        &self.db_path
    }
}
