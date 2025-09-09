use anyhow::Result;
use sqlx::{migrate::Migrator, Pool, Sqlite};

/// Run database migrations
pub async fn run_migrations(pool: &Pool<Sqlite>) -> Result<()> {
    // Create tables if they don't exist
    create_users_table(pool).await?;
    create_stores_table(pool).await?;
    create_products_table(pool).await?;
    create_price_records_table(pool).await?;
    create_user_reviews_table(pool).await?;
    create_price_alerts_table(pool).await?;
    create_ocr_results_table(pool).await?;

    log::info!("Database migrations completed successfully");
    Ok(())
}

/// Create users table
async fn create_users_table(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY NOT NULL,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            last_login INTEGER,
            reputation_score INTEGER NOT NULL DEFAULT 0
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Create stores table
async fn create_stores_table(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS stores (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            address TEXT NOT NULL,
            latitude REAL NOT NULL,
            longitude REAL NOT NULL,
            rating REAL NOT NULL DEFAULT 0.0,
            opening_hours TEXT NOT NULL,
            phone TEXT NOT NULL,
            tags TEXT NOT NULL, -- JSON array
            symbol TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Create products table
async fn create_products_table(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS products (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            description TEXT NOT NULL,
            barcode TEXT,
            images TEXT NOT NULL, -- JSON array
            tags TEXT NOT NULL, -- JSON array
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Create price_records table
async fn create_price_records_table(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS price_records (
            id TEXT PRIMARY KEY NOT NULL,
            product_id TEXT NOT NULL,
            store_id TEXT NOT NULL,
            user_id TEXT,
            price REAL NOT NULL,
            timestamp INTEGER NOT NULL,
            is_on_sale BOOLEAN NOT NULL DEFAULT FALSE,
            receipt_image TEXT,
            verification_status TEXT NOT NULL DEFAULT 'pending',
            FOREIGN KEY (product_id) REFERENCES products (id),
            FOREIGN KEY (store_id) REFERENCES stores (id),
            FOREIGN KEY (user_id) REFERENCES users (id)
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Create user_reviews table
async fn create_user_reviews_table(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_reviews (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            store_id TEXT,
            product_id TEXT,
            rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
            comment TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id),
            FOREIGN KEY (store_id) REFERENCES stores (id),
            FOREIGN KEY (product_id) REFERENCES products (id)
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Create price_alerts table
async fn create_price_alerts_table(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS price_alerts (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            product_id TEXT NOT NULL,
            target_price REAL NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id),
            FOREIGN KEY (product_id) REFERENCES products (id)
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Create ocr_results table
async fn create_ocr_results_table(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ocr_results (
            id TEXT PRIMARY KEY NOT NULL,
            image_path TEXT NOT NULL,
            extracted_text TEXT NOT NULL,
            parsed_items TEXT NOT NULL, -- JSON array
            confidence_score REAL NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Create indexes for better performance
pub async fn create_indexes(pool: &Pool<Sqlite>) -> Result<()> {
    // Index for price lookups
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_price_records_product_id ON price_records(product_id)",
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_price_records_store_id ON price_records(store_id)")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_price_records_timestamp ON price_records(timestamp)",
    )
    .execute(pool)
    .await?;

    // Index for user lookups
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
        .execute(pool)
        .await?;

    // Index for product searches
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_products_category ON products(category)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_products_barcode ON products(barcode)")
        .execute(pool)
        .await?;

    // Index for store location searches
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_stores_location ON stores(latitude, longitude)")
        .execute(pool)
        .await?;

    log::info!("Database indexes created successfully");
    Ok(())
}
