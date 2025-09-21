use crate::models::{PriceRecord, Product, Store, User};
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Row, Sqlite};

/// Generic repository trait for common database operations
#[allow(async_fn_in_trait)]
pub trait Repository<T> {
    async fn create(&self, entity: &T) -> Result<()>;
    async fn find_by_id(&self, id: &str) -> Result<Option<T>>;
    async fn update(&self, entity: &T) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn find_all(&self) -> Result<Vec<T>>;
}

/// User repository for user-related database operations
pub struct UserRepository {
    pool: Pool<Sqlite>,
}

impl UserRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, last_login, reputation_score 
             FROM users WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                created_at: DateTime::from_timestamp(row.get::<i64, _>("created_at"), 0)
                    .unwrap_or(Utc::now()),
                last_login: row
                    .get::<Option<i64>, _>("last_login")
                    .and_then(|ts| DateTime::from_timestamp(ts, 0)),
                reputation_score: row.get("reputation_score"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Find user by username
    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, last_login, reputation_score 
             FROM users WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                created_at: DateTime::from_timestamp(row.get::<i64, _>("created_at"), 0)
                    .unwrap_or(Utc::now()),
                last_login: row
                    .get::<Option<i64>, _>("last_login")
                    .and_then(|ts| DateTime::from_timestamp(ts, 0)),
                reputation_score: row.get("reputation_score"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Update user's last login timestamp
    pub async fn update_last_login(&self, user_id: &str) -> Result<()> {
        sqlx::query("UPDATE users SET last_login = ? WHERE id = ?")
            .bind(Utc::now().timestamp())
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

impl Repository<User> for UserRepository {
    async fn create(&self, user: &User) -> Result<()> {
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, created_at, last_login, reputation_score) 
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.created_at.timestamp())
        .bind(user.last_login.map(|dt| dt.timestamp()))
        .bind(user.reputation_score)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, last_login, reputation_score 
             FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                created_at: DateTime::from_timestamp(row.get::<i64, _>("created_at"), 0)
                    .unwrap_or(Utc::now()),
                last_login: row
                    .get::<Option<i64>, _>("last_login")
                    .and_then(|ts| DateTime::from_timestamp(ts, 0)),
                reputation_score: row.get("reputation_score"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn update(&self, user: &User) -> Result<()> {
        sqlx::query(
            "UPDATE users SET username = ?, email = ?, password_hash = ?, last_login = ?, reputation_score = ? 
             WHERE id = ?"
        )
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.last_login.map(|dt| dt.timestamp()))
        .bind(user.reputation_score)
        .bind(&user.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<User>> {
        let rows = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, last_login, reputation_score 
             FROM users ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let users = rows
            .into_iter()
            .map(|row| User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                created_at: DateTime::from_timestamp(row.get::<i64, _>("created_at"), 0)
                    .unwrap_or(Utc::now()),
                last_login: row
                    .get::<Option<i64>, _>("last_login")
                    .and_then(|ts| DateTime::from_timestamp(ts, 0)),
                reputation_score: row.get("reputation_score"),
            })
            .collect();

        Ok(users)
    }
}

/// Product repository for product-related database operations
pub struct ProductRepository {
    pool: Pool<Sqlite>,
}

impl ProductRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Find products by category
    pub async fn find_by_category(&self, category: &str) -> Result<Vec<Product>> {
        let rows = sqlx::query(
            "SELECT id, name, category, description, barcode, images, tags, created_at 
             FROM products WHERE category = ? ORDER BY name",
        )
        .bind(category)
        .fetch_all(&self.pool)
        .await?;

        let products = rows
            .into_iter()
            .map(|row| Product {
                id: row.get("id"),
                name: row.get("name"),
                category: row.get("category"),
                description: row.get("description"),
                barcode: row.get("barcode"),
                images: serde_json::from_str(&row.get::<String, _>("images")).unwrap_or_default(),
                prices: Vec::new(), // Will be loaded separately
                tags: serde_json::from_str(&row.get::<String, _>("tags")).unwrap_or_default(),
                created_at: DateTime::from_timestamp(row.get::<i64, _>("created_at"), 0)
                    .unwrap_or(Utc::now()),
            })
            .collect();

        Ok(products)
    }

    /// Find product by barcode
    pub async fn find_by_barcode(&self, barcode: &str) -> Result<Option<Product>> {
        let row = sqlx::query(
            "SELECT id, name, category, description, barcode, images, tags, created_at 
             FROM products WHERE barcode = ?",
        )
        .bind(barcode)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Product {
                id: row.get("id"),
                name: row.get("name"),
                category: row.get("category"),
                description: row.get("description"),
                barcode: row.get("barcode"),
                images: serde_json::from_str(&row.get::<String, _>("images")).unwrap_or_default(),
                prices: Vec::new(),
                tags: serde_json::from_str(&row.get::<String, _>("tags")).unwrap_or_default(),
                created_at: DateTime::from_timestamp(row.get::<i64, _>("created_at"), 0)
                    .unwrap_or(Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    /// Search products by name
    pub async fn search_by_name(&self, name: &str) -> Result<Vec<Product>> {
        let search_term = format!("%{}%", name);
        let rows = sqlx::query(
            "SELECT id, name, category, description, barcode, images, tags, created_at 
             FROM products WHERE name LIKE ? ORDER BY name",
        )
        .bind(search_term)
        .fetch_all(&self.pool)
        .await?;

        let products = rows
            .into_iter()
            .map(|row| Product {
                id: row.get("id"),
                name: row.get("name"),
                category: row.get("category"),
                description: row.get("description"),
                barcode: row.get("barcode"),
                images: serde_json::from_str(&row.get::<String, _>("images")).unwrap_or_default(),
                prices: Vec::new(),
                tags: serde_json::from_str(&row.get::<String, _>("tags")).unwrap_or_default(),
                created_at: DateTime::from_timestamp(row.get::<i64, _>("created_at"), 0)
                    .unwrap_or(Utc::now()),
            })
            .collect();

        Ok(products)
    }
}

impl Repository<Product> for ProductRepository {
    async fn create(&self, product: &Product) -> Result<()> {
        let images_json = serde_json::to_string(&product.images)?;
        let tags_json = serde_json::to_string(&product.tags)?;

        sqlx::query(
            "INSERT INTO products (id, name, category, description, barcode, images, tags, created_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&product.id)
        .bind(&product.name)
        .bind(&product.category)
        .bind(&product.description)
        .bind(&product.barcode)
        .bind(images_json)
        .bind(tags_json)
        .bind(product.created_at.timestamp())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Product>> {
        let row = sqlx::query(
            "SELECT id, name, category, description, barcode, images, tags, created_at 
             FROM products WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Product {
                id: row.get("id"),
                name: row.get("name"),
                category: row.get("category"),
                description: row.get("description"),
                barcode: row.get("barcode"),
                images: serde_json::from_str(&row.get::<String, _>("images")).unwrap_or_default(),
                prices: Vec::new(),
                tags: serde_json::from_str(&row.get::<String, _>("tags")).unwrap_or_default(),
                created_at: DateTime::from_timestamp(row.get::<i64, _>("created_at"), 0)
                    .unwrap_or(Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    async fn update(&self, product: &Product) -> Result<()> {
        let images_json = serde_json::to_string(&product.images)?;
        let tags_json = serde_json::to_string(&product.tags)?;

        sqlx::query(
            "UPDATE products SET name = ?, category = ?, description = ?, barcode = ?, images = ?, tags = ? 
             WHERE id = ?"
        )
        .bind(&product.name)
        .bind(&product.category)
        .bind(&product.description)
        .bind(&product.barcode)
        .bind(images_json)
        .bind(tags_json)
        .bind(&product.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM products WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Product>> {
        let rows = sqlx::query(
            "SELECT id, name, category, description, barcode, images, tags, created_at 
             FROM products ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        let products = rows
            .into_iter()
            .map(|row| Product {
                id: row.get("id"),
                name: row.get("name"),
                category: row.get("category"),
                description: row.get("description"),
                barcode: row.get("barcode"),
                images: serde_json::from_str(&row.get::<String, _>("images")).unwrap_or_default(),
                prices: Vec::new(),
                tags: serde_json::from_str(&row.get::<String, _>("tags")).unwrap_or_default(),
                created_at: DateTime::from_timestamp(row.get::<i64, _>("created_at"), 0)
                    .unwrap_or(Utc::now()),
            })
            .collect();

        Ok(products)
    }
}

/// Store repository for store-related database operations
pub struct StoreRepository {
    pool: Pool<Sqlite>,
}

impl StoreRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Find stores within a radius of given coordinates
    pub async fn find_nearby(
        &self,
        latitude: f64,
        longitude: f64,
        radius_km: f64,
    ) -> Result<Vec<Store>> {
        // Simple implementation - in a real app you'd use spatial indexing
        let stores = self.find_all().await?;
        let nearby_stores = stores
            .into_iter()
            .filter(|store| store.distance_to(latitude, longitude) <= radius_km)
            .collect();
        Ok(nearby_stores)
    }
}

impl Repository<Store> for StoreRepository {
    async fn create(&self, store: &Store) -> Result<()> {
        let tags_json = serde_json::to_string(&store.tags)?;

        sqlx::query(
            "INSERT INTO stores (id, name, address, latitude, longitude, rating, opening_hours, phone, tags, symbol, created_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&store.id)
        .bind(&store.name)
        .bind(&store.address)
        .bind(store.latitude)
        .bind(store.longitude)
        .bind(store.rating)
        .bind(&store.opening_hours)
        .bind(&store.phone)
        .bind(tags_json)
        .bind(store.symbol.to_string())
        .bind(store.created_at.timestamp())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Store>> {
        let row = sqlx::query(
            "SELECT id, name, address, latitude, longitude, rating, opening_hours, phone, tags, symbol, created_at 
             FROM stores WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let symbol_str: String = row.get("symbol");
            let symbol = symbol_str.chars().next().unwrap_or('🏪');

            Ok(Some(Store {
                id: row.get("id"),
                name: row.get("name"),
                address: row.get("address"),
                latitude: row.get("latitude"),
                longitude: row.get("longitude"),
                rating: row.get("rating"),
                opening_hours: row.get("opening_hours"),
                phone: row.get("phone"),
                tags: serde_json::from_str(&row.get::<String, _>("tags")).unwrap_or_default(),
                symbol,
                created_at: DateTime::from_timestamp(row.get::<i64, _>("created_at"), 0)
                    .unwrap_or(Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    async fn update(&self, store: &Store) -> Result<()> {
        let tags_json = serde_json::to_string(&store.tags)?;

        sqlx::query(
            "UPDATE stores SET name = ?, address = ?, latitude = ?, longitude = ?, rating = ?, 
             opening_hours = ?, phone = ?, tags = ?, symbol = ? WHERE id = ?",
        )
        .bind(&store.name)
        .bind(&store.address)
        .bind(store.latitude)
        .bind(store.longitude)
        .bind(store.rating)
        .bind(&store.opening_hours)
        .bind(&store.phone)
        .bind(tags_json)
        .bind(store.symbol.to_string())
        .bind(&store.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM stores WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Store>> {
        let rows = sqlx::query(
            "SELECT id, name, address, latitude, longitude, rating, opening_hours, phone, tags, symbol, created_at 
             FROM stores ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        let stores = rows
            .into_iter()
            .map(|row| {
                let symbol_str: String = row.get("symbol");
                let symbol = symbol_str.chars().next().unwrap_or('🏪');

                Store {
                    id: row.get("id"),
                    name: row.get("name"),
                    address: row.get("address"),
                    latitude: row.get("latitude"),
                    longitude: row.get("longitude"),
                    rating: row.get("rating"),
                    opening_hours: row.get("opening_hours"),
                    phone: row.get("phone"),
                    tags: serde_json::from_str(&row.get::<String, _>("tags")).unwrap_or_default(),
                    symbol,
                    created_at: DateTime::from_timestamp(row.get::<i64, _>("created_at"), 0)
                        .unwrap_or(Utc::now()),
                }
            })
            .collect();

        Ok(stores)
    }
}

/// Price repository for price-related database operations
pub struct PriceRepository {
    pool: Pool<Sqlite>,
}

impl PriceRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Find prices for a specific product
    pub async fn find_by_product_id(&self, product_id: &str) -> Result<Vec<PriceRecord>> {
        let rows = sqlx::query(
            "SELECT id, product_id, store_id, user_id, price, timestamp, is_on_sale, receipt_image, verification_status 
             FROM price_records WHERE product_id = ? ORDER BY timestamp DESC"
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await?;

        let price_records = rows
            .into_iter()
            .map(|row| PriceRecord {
                id: row.get("id"),
                product_id: row.get("product_id"),
                store_id: row.get("store_id"),
                user_id: row.get("user_id"),
                price: row.get("price"),
                timestamp: DateTime::from_timestamp(row.get::<i64, _>("timestamp"), 0)
                    .unwrap_or(Utc::now()),
                is_on_sale: row.get("is_on_sale"),
                receipt_image: row.get("receipt_image"),
                verification_status: row.get("verification_status"),
            })
            .collect();

        Ok(price_records)
    }

    /// Find latest verified prices for a product
    pub async fn find_latest_verified_prices(
        &self,
        product_id: &str,
        limit: i32,
    ) -> Result<Vec<PriceRecord>> {
        let rows = sqlx::query(
            "SELECT id, product_id, store_id, user_id, price, timestamp, is_on_sale, receipt_image, verification_status 
             FROM price_records WHERE product_id = ? AND verification_status = 'verified' 
             ORDER BY timestamp DESC LIMIT ?"
        )
        .bind(product_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let price_records = rows
            .into_iter()
            .map(|row| PriceRecord {
                id: row.get("id"),
                product_id: row.get("product_id"),
                store_id: row.get("store_id"),
                user_id: row.get("user_id"),
                price: row.get("price"),
                timestamp: DateTime::from_timestamp(row.get::<i64, _>("timestamp"), 0)
                    .unwrap_or(Utc::now()),
                is_on_sale: row.get("is_on_sale"),
                receipt_image: row.get("receipt_image"),
                verification_status: row.get("verification_status"),
            })
            .collect();

        Ok(price_records)
    }

    /// Create a new price record
    pub async fn create_price_record(&self, price_record: &PriceRecord) -> Result<()> {
        sqlx::query(
            "INSERT INTO price_records (id, product_id, store_id, user_id, price, timestamp, is_on_sale, receipt_image, verification_status) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&price_record.id)
        .bind(&price_record.product_id)
        .bind(&price_record.store_id)
        .bind(&price_record.user_id)
        .bind(price_record.price)
        .bind(price_record.timestamp.timestamp())
        .bind(price_record.is_on_sale)
        .bind(&price_record.receipt_image)
        .bind(&price_record.verification_status)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
