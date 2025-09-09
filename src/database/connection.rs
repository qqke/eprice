use crate::utils;
use anyhow::Result;
use sqlx::{sqlite::SqlitePool, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::Path;

/// Database connection manager for SQLite
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    /// Create a new database manager with the given database URL
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Create a new database manager with default configuration
    pub async fn new_default() -> Result<Self> {
        // Ensure data directory exists
        utils::initialize_directories()?;

        let data_dir = utils::get_data_directory()?;
        let db_path = data_dir.join("eprice.db");
        let database_url = format!("sqlite:{}", db_path.display());

        Self::new(&database_url).await
    }

    /// Get the connection pool
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Close the database connection pool
    pub async fn close(self) {
        self.pool.close().await;
    }

    /// Check if the database connection is healthy
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }
}
