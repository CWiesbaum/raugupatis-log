use config::{Config, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server_address: String,
    pub database_url: String,
    pub environment: String,
    pub session_secret: String,
    pub uploads_dir: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let env = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            // Start with default configuration
            .add_source(File::with_name("config/default").required(false))
            // Add environment-specific configuration
            .add_source(File::with_name(&format!("config/{}", env)).required(false))
            // Add local configuration (for development overrides)
            .add_source(File::with_name("config/local").required(false))
            // Add environment variables with prefix "RAUGUPATIS_"
            .add_source(Environment::with_prefix("RAUGUPATIS"))
            .build()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(s.try_deserialize()?)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_address: "0.0.0.0:3000".to_string(),
            database_url: "sqlite:data/raugupatis.db".to_string(),
            environment: "development".to_string(),
            session_secret: "your-secret-key-change-in-production".to_string(),
            uploads_dir: "data/uploads".to_string(),
        }
    }
}
