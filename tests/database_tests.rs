use eprice::database::*;
use eprice::error::{ServiceError, ServiceResult};
use eprice::models::*;
use std::sync::Arc;
use tokio;

#[tokio::test]
async fn test_database_connection() {
    let db = Database::new(":memory:").await;
    assert!(db.is_ok());
}

#[tokio::test]
async fn test_database_migration() {
    let db = Database::new(":memory:")
        .await
        .expect("Failed to create database");
    let result = db.migrate().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_database_transaction() {
    let db = Database::new(":memory:")
        .await
        .expect("Failed to create database");
    db.migrate().await.expect("Failed to migrate");

    let result = db.begin_transaction().await;
    assert!(result.is_ok());

    let mut tx = result.unwrap();

    // Perform some operations within transaction
    let query = "INSERT INTO products (name, category, created_at, updated_at) VALUES (?, ?, ?, ?)";
    let now = chrono::Utc::now();
    let result = sqlx::query(query)
        .bind("Test Product")
        .bind("Food")
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await;

    assert!(result.is_ok());

    // Commit transaction
    let commit_result = tx.commit().await;
    assert!(commit_result.is_ok());
}

#[tokio::test]
async fn test_database_rollback() {
    let db = Database::new(":memory:")
        .await
        .expect("Failed to create database");
    db.migrate().await.expect("Failed to migrate");

    let mut tx = db
        .begin_transaction()
        .await
        .expect("Failed to begin transaction");

    // Insert some data
    let query = "INSERT INTO products (name, category, created_at, updated_at) VALUES (?, ?, ?, ?)";
    let now = chrono::Utc::now();
    sqlx::query(query)
        .bind("Test Product")
        .bind("Food")
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await
        .expect("Failed to insert");

    // Rollback
    let rollback_result = tx.rollback().await;
    assert!(rollback_result.is_ok());

    // Verify data was not persisted
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM products")
        .fetch_one(&db.pool)
        .await
        .expect("Failed to count products");

    assert_eq!(count.0, 0);
}

#[tokio::test]
async fn test_product_repository_crud() {
    let db = Database::new(":memory:")
        .await
        .expect("Failed to create database");
    db.migrate().await.expect("Failed to migrate");

    let repo = ProductRepository::new(Arc::new(db));

    // Create
    let product = Product {
        id: 0,
        name: "Test Product".to_string(),
        barcode: Some("1234567890123".to_string()),
        category: ProductCategory::Food,
        brand: Some("Test Brand".to_string()),
        description: Some("Test Description".to_string()),
        image_url: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = repo
        .create(&product)
        .await
        .expect("Failed to create product");
    assert!(created.id > 0);

    // Read
    let found = repo
        .find_by_id(created.id)
        .await
        .expect("Failed to find product");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Test Product");

    // Update
    let mut updated_product = created.clone();
    updated_product.name = "Updated Product".to_string();
    let update_result = repo.update(&updated_product).await;
    assert!(update_result.is_ok());

    // Verify update
    let found_updated = repo
        .find_by_id(created.id)
        .await
        .expect("Failed to find updated product");
    assert_eq!(found_updated.unwrap().name, "Updated Product");

    // Delete
    let delete_result = repo.delete(created.id).await;
    assert!(delete_result.is_ok());

    // Verify deletion
    let found_deleted = repo
        .find_by_id(created.id)
        .await
        .expect("Failed to check deleted product");
    assert!(found_deleted.is_none());
}

#[tokio::test]
async fn test_store_repository_crud() {
    let db = Database::new(":memory:")
        .await
        .expect("Failed to create database");
    db.migrate().await.expect("Failed to migrate");

    let repo = StoreRepository::new(Arc::new(db));

    // Create
    let store = Store {
        id: 0,
        name: "Test Store".to_string(),
        chain: Some("Test Chain".to_string()),
        address: "123 Test St".to_string(),
        latitude: 35.6762,
        longitude: 139.6503,
        phone: Some("+81-3-1234-5678".to_string()),
        hours: Some("9:00-21:00".to_string()),
        is_verified: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = repo.create(&store).await.expect("Failed to create store");
    assert!(created.id > 0);

    // Test distance-based queries
    let nearby = repo
        .find_nearby(35.6762, 139.6503, 1.0)
        .await
        .expect("Failed to find nearby stores");
    assert_eq!(nearby.len(), 1);
    assert_eq!(nearby[0].id, created.id);

    // Test outside radius
    let far_away = repo
        .find_nearby(36.0, 140.0, 1.0)
        .await
        .expect("Failed to search far location");
    assert_eq!(far_away.len(), 0);
}

#[tokio::test]
async fn test_user_repository_crud() {
    let db = Database::new(":memory:")
        .await
        .expect("Failed to create database");
    db.migrate().await.expect("Failed to migrate");

    let repo = UserRepository::new(Arc::new(db));

    // Create
    let user = User {
        id: 0,
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        display_name: Some("Test User".to_string()),
        avatar_url: None,
        reputation_score: 100,
        is_verified: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = repo.create(&user).await.expect("Failed to create user");
    assert!(created.id > 0);

    // Test find by username
    let found_by_username = repo
        .find_by_username("testuser")
        .await
        .expect("Failed to find by username");
    assert!(found_by_username.is_some());
    assert_eq!(found_by_username.unwrap().email, "test@example.com");

    // Test find by email
    let found_by_email = repo
        .find_by_email("test@example.com")
        .await
        .expect("Failed to find by email");
    assert!(found_by_email.is_some());
    assert_eq!(found_by_email.unwrap().username, "testuser");

    // Test unique constraint violations
    let duplicate_user = User {
        id: 0,
        username: "testuser".to_string(), // Same username
        email: "different@example.com".to_string(),
        password_hash: "different_hash".to_string(),
        display_name: Some("Different User".to_string()),
        avatar_url: None,
        reputation_score: 0,
        is_verified: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let duplicate_result = repo.create(&duplicate_user).await;
    assert!(duplicate_result.is_err()); // Should fail due to unique constraint
}

#[tokio::test]
async fn test_price_record_repository_crud() {
    let db = Database::new(":memory:")
        .await
        .expect("Failed to create database");
    db.migrate().await.expect("Failed to migrate");

    let repo = PriceRecordRepository::new(Arc::new(db));

    // Create
    let price_record = PriceRecord {
        id: 0,
        product_id: 1,
        store_id: 1,
        price: 1500,
        date_recorded: chrono::Utc::now(),
        user_id: Some(1),
        verification_status: VerificationStatus::Pending,
        verification_count: 0,
        notes: Some("Test price".to_string()),
    };

    let created = repo
        .create(&price_record)
        .await
        .expect("Failed to create price record");
    assert!(created.id > 0);

    // Test find by product
    let product_prices = repo
        .find_by_product_id(1)
        .await
        .expect("Failed to find by product");
    assert_eq!(product_prices.len(), 1);
    assert_eq!(product_prices[0].price, 1500);

    // Test find by store
    let store_prices = repo
        .find_by_store_id(1)
        .await
        .expect("Failed to find by store");
    assert_eq!(store_prices.len(), 1);

    // Test verification status filtering
    let pending_prices = repo
        .find_by_verification_status(VerificationStatus::Pending)
        .await
        .expect("Failed to find pending");
    assert_eq!(pending_prices.len(), 1);

    let verified_prices = repo
        .find_by_verification_status(VerificationStatus::Verified)
        .await
        .expect("Failed to find verified");
    assert_eq!(verified_prices.len(), 0);
}

#[tokio::test]
async fn test_review_repository_crud() {
    let db = Database::new(":memory:")
        .await
        .expect("Failed to create database");
    db.migrate().await.expect("Failed to migrate");

    let repo = ReviewRepository::new(Arc::new(db));

    // Create
    let review = UserReview {
        id: 0,
        user_id: 1,
        product_id: 1,
        store_id: 1,
        rating: 4,
        comment: Some("Great product!".to_string()),
        is_verified: false,
        helpful_count: 0,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = repo.create(&review).await.expect("Failed to create review");
    assert!(created.id > 0);

    // Test find by product
    let product_reviews = repo
        .find_by_product_id(1)
        .await
        .expect("Failed to find by product");
    assert_eq!(product_reviews.len(), 1);
    assert_eq!(product_reviews[0].rating, 4);

    // Test find by user
    let user_reviews = repo
        .find_by_user_id(1)
        .await
        .expect("Failed to find by user");
    assert_eq!(user_reviews.len(), 1);

    // Test average rating calculation
    let avg_rating = repo
        .get_average_rating(1)
        .await
        .expect("Failed to get average rating");
    assert_eq!(avg_rating, Some(4.0));
}

#[tokio::test]
async fn test_database_connection_pool() {
    let db = Database::new(":memory:")
        .await
        .expect("Failed to create database");

    // Test multiple concurrent connections
    let handles = (0..5)
        .map(|i| {
            let pool = db.pool.clone();
            tokio::spawn(async move {
                let result = sqlx::query("SELECT 1").fetch_one(&pool).await;
                (i, result.is_ok())
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        let (_, success) = handle.await.expect("Task failed");
        assert!(success);
    }
}

#[tokio::test]
async fn test_database_error_handling() {
    let db = Database::new(":memory:")
        .await
        .expect("Failed to create database");

    // Test query with invalid SQL
    let result = sqlx::query("INVALID SQL STATEMENT")
        .fetch_one(&db.pool)
        .await;

    assert!(result.is_err());

    // Test accessing non-existent table before migration
    let result = sqlx::query("SELECT * FROM products")
        .fetch_all(&db.pool)
        .await;

    assert!(result.is_err());
}
