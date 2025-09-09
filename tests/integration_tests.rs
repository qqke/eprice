use eprice::services::AppServices;

#[test]
fn test_basic_service_integration() {
    let mut app_services = AppServices::new();

    // Test user registration
    let user = app_services
        .user_service
        .register_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "password123".to_string(),
        )
        .unwrap();

    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "test@example.com");

    // Test product creation
    let product = app_services
        .product_service
        .create_product(
            "Test Product".to_string(),
            "Electronics".to_string(),
            "A test product".to_string(),
            Some("1234567890123".to_string()),
            vec!["test".to_string()],
        )
        .unwrap();

    assert_eq!(product.name, "Test Product");
    assert_eq!(product.category, "Electronics");

    // Test store creation
    let store = app_services
        .store_service
        .create_store(
            "Test Store".to_string(),
            "123 Test St".to_string(),
            35.6762,
            139.6503,
            "09:00-21:00".to_string(),
            "123-456-7890".to_string(),
            vec!["grocery".to_string()],
            'S',
        )
        .unwrap();

    assert_eq!(store.name, "Test Store");

    // Test price submission
    let price = app_services
        .price_service
        .submit_price(
            product.id.clone(),
            store.id.clone(),
            Some(user.id.clone()),
            99.99,
            false,
            None,
        )
        .unwrap();

    assert_eq!(price.price, 99.99);

    // Test review submission
    let review = app_services
        .review_service
        .submit_review(
            user.id.clone(),
            Some(store.id.clone()),
            None,
            5,
            "Great store!".to_string(),
        )
        .unwrap();

    assert_eq!(review.rating, 5);
}

#[test]
fn test_service_error_handling() {
    let app_services = AppServices::new();

    // Test non-existent user
    let result = app_services.user_service.get_user("nonexistent");
    assert!(result.is_err());

    // Test non-existent product
    let result = app_services.product_service.get_product("nonexistent");
    assert!(result.is_err());

    // Test non-existent store
    let result = app_services.store_service.get_store("nonexistent");
    assert!(result.is_err());
}
