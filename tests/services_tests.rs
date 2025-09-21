use eprice::database::Database;
use eprice::error::{ServiceError, ServiceResult};
use eprice::models::*;
use eprice::services::*;
use std::sync::Arc;
use tokio;

async fn setup_test_database() -> Arc<Database> {
    let db = Database::new(":memory:")
        .await
        .expect("Failed to create test database");
    db.migrate().await.expect("Failed to run migrations");
    Arc::new(db)
}

#[tokio::test]
async fn test_price_service_add_price_record() {
    let db = setup_test_database().await;
    let price_service = PriceService::new(db.clone());

    let price_record = PriceRecord {
        id: 1,
        product_id: 1,
        store_id: 1,
        price: 1250,
        date_recorded: chrono::Utc::now(),
        user_id: Some(1),
        verification_status: VerificationStatus::Pending,
        verification_count: 0,
        notes: Some("Test price record".to_string()),
    };

    let result = price_service.add_price_record(&price_record).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_price_service_get_price_history() {
    let db = setup_test_database().await;
    let price_service = PriceService::new(db.clone());

    // Add some test data first
    let price1 = PriceRecord {
        id: 1,
        product_id: 1,
        store_id: 1,
        price: 1000,
        date_recorded: chrono::Utc::now() - chrono::Duration::days(7),
        user_id: Some(1),
        verification_status: VerificationStatus::Verified,
        verification_count: 3,
        notes: None,
    };

    let price2 = PriceRecord {
        id: 2,
        product_id: 1,
        store_id: 1,
        price: 1200,
        date_recorded: chrono::Utc::now() - chrono::Duration::days(3),
        user_id: Some(1),
        verification_status: VerificationStatus::Verified,
        verification_count: 2,
        notes: None,
    };

    price_service.add_price_record(&price1).await.unwrap();
    price_service.add_price_record(&price2).await.unwrap();

    let history = price_service.get_price_history(1, Some(30)).await.unwrap();
    assert_eq!(history.len(), 2);

    // Should be sorted by date descending
    assert!(history[0].date_recorded > history[1].date_recorded);
}

#[tokio::test]
async fn test_price_service_verify_price() {
    let db = setup_test_database().await;
    let price_service = PriceService::new(db.clone());

    let price_record = PriceRecord {
        id: 1,
        product_id: 1,
        store_id: 1,
        price: 1500,
        date_recorded: chrono::Utc::now(),
        user_id: Some(1),
        verification_status: VerificationStatus::Pending,
        verification_count: 0,
        notes: None,
    };

    price_service.add_price_record(&price_record).await.unwrap();

    let result = price_service.verify_price(1, 2, true).await;
    assert!(result.is_ok());

    // Check that verification count increased
    let updated_record = price_service.get_price_record(1).await.unwrap();
    assert_eq!(updated_record.verification_count, 1);
}

#[tokio::test]
async fn test_product_service_create_product() {
    let db = setup_test_database().await;
    let product_service = ProductService::new(db.clone());

    let product = Product {
        id: 0, // Will be assigned by database
        name: "Test Product".to_string(),
        barcode: Some("1234567890123".to_string()),
        category: ProductCategory::Food,
        brand: Some("Test Brand".to_string()),
        description: Some("A test product for unit testing".to_string()),
        image_url: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let result = product_service.create_product(&product).await;
    assert!(result.is_ok());

    let created_product = result.unwrap();
    assert!(created_product.id > 0);
    assert_eq!(created_product.name, "Test Product");
    assert_eq!(created_product.barcode, Some("1234567890123".to_string()));
}

#[tokio::test]
async fn test_product_service_search_products() {
    let db = setup_test_database().await;
    let product_service = ProductService::new(db.clone());

    // Create test products
    let product1 = Product {
        id: 0,
        name: "Apple iPhone 14".to_string(),
        barcode: Some("1111111111111".to_string()),
        category: ProductCategory::Electronics,
        brand: Some("Apple".to_string()),
        description: Some("Latest iPhone model".to_string()),
        image_url: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let product2 = Product {
        id: 0,
        name: "Samsung Galaxy S23".to_string(),
        barcode: Some("2222222222222".to_string()),
        category: ProductCategory::Electronics,
        brand: Some("Samsung".to_string()),
        description: Some("Android smartphone".to_string()),
        image_url: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    product_service.create_product(&product1).await.unwrap();
    product_service.create_product(&product2).await.unwrap();

    // Search by name
    let results = product_service
        .search_products("iPhone", None, None, 10, 0)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].name.contains("iPhone"));

    // Search by category
    let results = product_service
        .search_products("", Some(ProductCategory::Electronics), None, 10, 0)
        .await
        .unwrap();
    assert_eq!(results.len(), 2);

    // Search by brand
    let results = product_service
        .search_products("", None, Some("Apple".to_string()), 10, 0)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].brand, Some("Apple".to_string()));
}

#[tokio::test]
async fn test_store_service_create_store() {
    let db = setup_test_database().await;
    let store_service = StoreService::new(db.clone());

    let store = Store {
        id: 0,
        name: "Test Supermarket".to_string(),
        chain: Some("Test Chain".to_string()),
        address: "123 Test Street, Test City".to_string(),
        latitude: 35.6762,
        longitude: 139.6503,
        phone: Some("+81-3-1234-5678".to_string()),
        hours: Some("9:00-21:00".to_string()),
        is_verified: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let result = store_service.create_store(&store).await;
    assert!(result.is_ok());

    let created_store = result.unwrap();
    assert!(created_store.id > 0);
    assert_eq!(created_store.name, "Test Supermarket");
    assert_eq!(created_store.latitude, 35.6762);
}

#[tokio::test]
async fn test_store_service_find_nearby_stores() {
    let db = setup_test_database().await;
    let store_service = StoreService::new(db.clone());

    // Create stores at different locations
    let store1 = Store {
        id: 0,
        name: "Close Store".to_string(),
        chain: None,
        address: "Near location".to_string(),
        latitude: 35.6762, // Tokyo Station
        longitude: 139.6503,
        phone: None,
        hours: None,
        is_verified: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let store2 = Store {
        id: 0,
        name: "Far Store".to_string(),
        chain: None,
        address: "Far location".to_string(),
        latitude: 35.0116, // Much further south
        longitude: 135.7681,
        phone: None,
        hours: None,
        is_verified: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    store_service.create_store(&store1).await.unwrap();
    store_service.create_store(&store2).await.unwrap();

    // Search near Tokyo Station with 5km radius
    let nearby_stores = store_service
        .find_nearby_stores(35.6762, 139.6503, 5.0)
        .await
        .unwrap();

    assert_eq!(nearby_stores.len(), 1);
    assert_eq!(nearby_stores[0].name, "Close Store");
}

#[tokio::test]
async fn test_user_service_create_user() {
    let db = setup_test_database().await;
    let user_service = UserService::new(db.clone());

    let user = User {
        id: 0,
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        display_name: Some("Test User".to_string()),
        avatar_url: None,
        reputation_score: 0,
        is_verified: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let result = user_service.create_user(&user).await;
    assert!(result.is_ok());

    let created_user = result.unwrap();
    assert!(created_user.id > 0);
    assert_eq!(created_user.username, "testuser");
    assert_eq!(created_user.email, "test@example.com");
}

#[tokio::test]
async fn test_user_service_authenticate() {
    let db = setup_test_database().await;
    let user_service = UserService::new(db.clone());

    // First create a user with a known password hash
    let password = "testpassword123";
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();

    let user = User {
        id: 0,
        username: "authuser".to_string(),
        email: "auth@example.com".to_string(),
        password_hash,
        display_name: Some("Auth User".to_string()),
        avatar_url: None,
        reputation_score: 0,
        is_verified: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created_user = user_service.create_user(&user).await.unwrap();

    // Test successful authentication
    let auth_result = user_service.authenticate("authuser", password).await;
    assert!(auth_result.is_ok());
    assert_eq!(auth_result.unwrap().id, created_user.id);

    // Test failed authentication
    let bad_auth = user_service.authenticate("authuser", "wrongpassword").await;
    assert!(bad_auth.is_err());
}

#[tokio::test]
async fn test_user_service_update_reputation() {
    let db = setup_test_database().await;
    let user_service = UserService::new(db.clone());

    let user = User {
        id: 0,
        username: "repuser".to_string(),
        email: "rep@example.com".to_string(),
        password_hash: "hash".to_string(),
        display_name: Some("Rep User".to_string()),
        avatar_url: None,
        reputation_score: 100,
        is_verified: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created_user = user_service.create_user(&user).await.unwrap();

    // Update reputation
    let result = user_service.update_reputation(created_user.id, 150).await;
    assert!(result.is_ok());

    // Verify the update
    let updated_user = user_service.get_user_by_id(created_user.id).await.unwrap();
    assert_eq!(updated_user.reputation_score, 150);
}

#[tokio::test]
async fn test_review_service_create_review() {
    let db = setup_test_database().await;
    let review_service = ReviewService::new(db.clone());

    let review = UserReview {
        id: 0,
        user_id: 1,
        product_id: 1,
        store_id: 1,
        rating: 4,
        comment: Some("Great product, good price".to_string()),
        is_verified: false,
        helpful_count: 0,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let result = review_service.create_review(&review).await;
    assert!(result.is_ok());

    let created_review = result.unwrap();
    assert!(created_review.id > 0);
    assert_eq!(created_review.rating, 4);
    assert_eq!(
        created_review.comment,
        Some("Great product, good price".to_string())
    );
}

#[tokio::test]
async fn test_review_service_get_reviews_for_product() {
    let db = setup_test_database().await;
    let review_service = ReviewService::new(db.clone());

    // Create multiple reviews for the same product
    let review1 = UserReview {
        id: 0,
        user_id: 1,
        product_id: 1,
        store_id: 1,
        rating: 5,
        comment: Some("Excellent!".to_string()),
        is_verified: true,
        helpful_count: 10,
        created_at: chrono::Utc::now() - chrono::Duration::days(2),
        updated_at: chrono::Utc::now() - chrono::Duration::days(2),
    };

    let review2 = UserReview {
        id: 0,
        user_id: 2,
        product_id: 1,
        store_id: 2,
        rating: 3,
        comment: Some("Average quality".to_string()),
        is_verified: false,
        helpful_count: 2,
        created_at: chrono::Utc::now() - chrono::Duration::days(1),
        updated_at: chrono::Utc::now() - chrono::Duration::days(1),
    };

    review_service.create_review(&review1).await.unwrap();
    review_service.create_review(&review2).await.unwrap();

    let reviews = review_service
        .get_reviews_for_product(1, 10, 0)
        .await
        .unwrap();
    assert_eq!(reviews.len(), 2);

    // Should be sorted by creation date descending
    assert!(reviews[0].created_at > reviews[1].created_at);
}

#[tokio::test]
async fn test_review_service_mark_helpful() {
    let db = setup_test_database().await;
    let review_service = ReviewService::new(db.clone());

    let review = UserReview {
        id: 0,
        user_id: 1,
        product_id: 1,
        store_id: 1,
        rating: 4,
        comment: Some("Helpful review".to_string()),
        is_verified: false,
        helpful_count: 0,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created_review = review_service.create_review(&review).await.unwrap();

    // Mark as helpful
    let result = review_service.mark_helpful(created_review.id, 2).await;
    assert!(result.is_ok());

    // Verify helpful count increased
    let updated_review = review_service
        .get_review_by_id(created_review.id)
        .await
        .unwrap();
    assert_eq!(updated_review.helpful_count, 1);
}

#[tokio::test]
async fn test_review_service_verify_review() {
    let db = setup_test_database().await;
    let review_service = ReviewService::new(db.clone());

    let review = UserReview {
        id: 0,
        user_id: 1,
        product_id: 1,
        store_id: 1,
        rating: 5,
        comment: Some("Needs verification".to_string()),
        is_verified: false,
        helpful_count: 0,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created_review = review_service.create_review(&review).await.unwrap();

    // Verify the review
    let result = review_service.verify_review(created_review.id).await;
    assert!(result.is_ok());

    // Check verification status
    let verified_review = review_service
        .get_review_by_id(created_review.id)
        .await
        .unwrap();
    assert!(verified_review.is_verified);
}

#[tokio::test]
async fn test_service_error_handling() {
    let db = setup_test_database().await;
    let user_service = UserService::new(db.clone());

    // Test error when user doesn't exist
    let result = user_service.get_user_by_id(999).await;
    assert!(result.is_err());

    // Test authentication with non-existent user
    let auth_result = user_service.authenticate("nonexistent", "password").await;
    assert!(auth_result.is_err());
}
