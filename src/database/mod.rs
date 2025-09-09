pub mod connection;
pub mod migrations;
pub mod repository;

pub use connection::DatabaseManager;
pub use repository::{PriceRepository, ProductRepository, StoreRepository, UserRepository};

use anyhow::Result;
use sqlx::sqlite::SqlitePool;

/// Database module provides centralized database access and management
/// for the eprice application using SQLite with SQLx.
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database instance with the given connection pool
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Initialize the database with migrations
    pub async fn initialize(&self) -> Result<()> {
        migrations::run_migrations(&self.pool).await?;
        Ok(())
    }
}
