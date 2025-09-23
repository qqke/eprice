use eframe::egui;
use eprice::models::*;
use eprice::ui::components::*;
use eprice::ui::state::*;

#[test]
fn test_app_state_initialization() {
    let state = AppState::new();

    assert!(state.current_user.is_none());
    assert_eq!(state.current_view, ViewType::Login);
    assert!(state.search_results.is_empty());
    assert!(state.price_history.is_empty());
    assert!(state.nearby_stores.is_empty());
}

#[test]
fn test_app_state_user_login() {
    let mut state = AppState::new();

    let user = User {
        id: 1,
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password_hash: "hash".to_string(),
        display_name: Some("Test User".to_string()),
        avatar_url: None,
        reputation_score: 100,
        is_verified: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state.set_current_user(Some(user.clone()));

    assert!(state.current_user.is_some());
    assert_eq!(state.current_user.unwrap().username, "testuser");
}

#[test]
fn test_app_state_view_navigation() {
    let mut state = AppState::new();

    assert_eq!(state.current_view, ViewType::Login);

    state.set_view(ViewType::Dashboard);
    assert_eq!(state.current_view, ViewType::Dashboard);

    state.set_view(ViewType::ProductSearch);
    assert_eq!(state.current_view, ViewType::ProductSearch);

    state.set_view(ViewType::Settings);
    assert_eq!(state.current_view, ViewType::Settings);
}

#[test]
fn test_search_state_management() {
    let mut state = AppState::new();

    // Test search query
    state.set_search_query("iPhone".to_string());
    assert_eq!(state.search_query, "iPhone");

    // Test search results
    let product = Product {
        id: 1,
        name: "iPhone 14".to_string(),
        barcode: Some("123456789".to_string()),
        category: ProductCategory::Electronics,
        brand: Some("Apple".to_string()),
        description: Some("Latest iPhone".to_string()),
        image_url: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state.set_search_results(vec![product.clone()]);
    assert_eq!(state.search_results.len(), 1);
    assert_eq!(state.search_results[0].name, "iPhone 14");

    // Test selected product
    state.set_selected_product(Some(product));
    assert!(state.selected_product.is_some());
    assert_eq!(state.selected_product.as_ref().unwrap().name, "iPhone 14");
}

#[test]
fn test_price_history_management() {
    let mut state = AppState::new();

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

    state.set_price_history(vec![price1, price2]);
    assert_eq!(state.price_history.len(), 2);

    // Test that history is sorted by date
    assert!(state.price_history[0].date_recorded > state.price_history[1].date_recorded);
}

#[test]
fn test_store_management() {
    let mut state = AppState::new();

    let store1 = Store {
        id: 1,
        name: "Store A".to_string(),
        chain: Some("Chain A".to_string()),
        address: "123 Main St".to_string(),
        latitude: 35.6762,
        longitude: 139.6503,
        phone: Some("+81-3-1234-5678".to_string()),
        hours: Some("9:00-21:00".to_string()),
        is_verified: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let store2 = Store {
        id: 2,
        name: "Store B".to_string(),
        chain: Some("Chain B".to_string()),
        address: "456 Second St".to_string(),
        latitude: 35.6863,
        longitude: 139.6604,
        phone: Some("+81-3-2345-6789".to_string()),
        hours: Some("8:00-22:00".to_string()),
        is_verified: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state.set_nearby_stores(vec![store1, store2]);
    assert_eq!(state.nearby_stores.len(), 2);

    state.set_selected_store(Some(state.nearby_stores[0].clone()));
    assert!(state.selected_store.is_some());
    assert_eq!(state.selected_store.as_ref().unwrap().name, "Store A");
}

#[test]
fn test_loading_state_management() {
    let mut state = AppState::new();

    // Test loading states
    assert!(!state.is_loading_search);
    assert!(!state.is_loading_prices);
    assert!(!state.is_loading_stores);

    state.set_loading_search(true);
    assert!(state.is_loading_search);

    state.set_loading_prices(true);
    assert!(state.is_loading_prices);

    state.set_loading_stores(true);
    assert!(state.is_loading_stores);

    // Test clearing loading states
    state.set_loading_search(false);
    state.set_loading_prices(false);
    state.set_loading_stores(false);

    assert!(!state.is_loading_search);
    assert!(!state.is_loading_prices);
    assert!(!state.is_loading_stores);
}

#[test]
fn test_error_state_management() {
    let mut state = AppState::new();

    assert!(state.last_error.is_none());

    state.set_error("Test error message".to_string());
    assert!(state.last_error.is_some());
    assert_eq!(state.last_error.as_ref().unwrap(), "Test error message");

    state.clear_error();
    assert!(state.last_error.is_none());
}

#[test]
fn test_settings_state() {
    let mut state = AppState::new();

    // Test default settings
    assert_eq!(state.settings.language, Language::Japanese);
    assert_eq!(state.settings.currency, Currency::JPY);
    assert_eq!(state.settings.distance_unit, DistanceUnit::Kilometers);
    assert!(state.settings.enable_notifications);
    assert!(state.settings.auto_verify_receipts);

    // Test updating settings
    state.settings.language = Language::English;
    state.settings.currency = Currency::USD;
    state.settings.distance_unit = DistanceUnit::Miles;
    state.settings.enable_notifications = false;
    state.settings.auto_verify_receipts = false;

    assert_eq!(state.settings.language, Language::English);
    assert_eq!(state.settings.currency, Currency::USD);
    assert_eq!(state.settings.distance_unit, DistanceUnit::Miles);
    assert!(!state.settings.enable_notifications);
    assert!(!state.settings.auto_verify_receipts);
}

#[test]
fn test_camera_state() {
    let mut state = AppState::new();

    // Test default camera state
    assert!(!state.camera_active);
    assert!(state.captured_image.is_none());
    assert!(state.scanned_barcode.is_none());

    // Test camera activation
    state.set_camera_active(true);
    assert!(state.camera_active);

    // Test barcode scanning
    state.set_scanned_barcode(Some("1234567890123".to_string()));
    assert!(state.scanned_barcode.is_some());
    assert_eq!(state.scanned_barcode.as_ref().unwrap(), "1234567890123");

    // Test image capture
    let test_image = vec![1, 2, 3, 4, 5]; // Mock image data
    state.set_captured_image(Some(test_image.clone()));
    assert!(state.captured_image.is_some());
    assert_eq!(state.captured_image.as_ref().unwrap(), &test_image);
}

#[test]
fn test_notification_state() {
    let mut state = AppState::new();

    assert!(state.notifications.is_empty());

    // Test adding notifications
    state.add_notification("Info message".to_string(), NotificationType::Info);
    assert_eq!(state.notifications.len(), 1);

    state.add_notification("Warning message".to_string(), NotificationType::Warning);
    assert_eq!(state.notifications.len(), 2);

    state.add_notification("Error message".to_string(), NotificationType::Error);
    assert_eq!(state.notifications.len(), 3);

    // Test notification types
    assert_eq!(
        state.notifications[0].notification_type,
        NotificationType::Info
    );
    assert_eq!(
        state.notifications[1].notification_type,
        NotificationType::Warning
    );
    assert_eq!(
        state.notifications[2].notification_type,
        NotificationType::Error
    );

    // Test clearing notifications
    state.clear_notifications();
    assert!(state.notifications.is_empty());
}

#[test]
fn test_form_validation_state() {
    let mut state = AppState::new();

    // Test login form
    assert!(state.login_username.is_empty());
    assert!(state.login_password.is_empty());
    assert!(state.login_error.is_none());

    state.login_username = "testuser".to_string();
    state.login_password = "password123".to_string();

    assert_eq!(state.login_username, "testuser");
    assert_eq!(state.login_password, "password123");

    // Test registration form
    assert!(state.register_username.is_empty());
    assert!(state.register_email.is_empty());
    assert!(state.register_password.is_empty());
    assert!(state.register_confirm_password.is_empty());

    state.register_username = "newuser".to_string();
    state.register_email = "new@example.com".to_string();
    state.register_password = "newpassword".to_string();
    state.register_confirm_password = "newpassword".to_string();

    assert_eq!(state.register_username, "newuser");
    assert_eq!(state.register_email, "new@example.com");
    assert_eq!(state.register_password, "newpassword");
    assert_eq!(state.register_confirm_password, "newpassword");
}

#[test]
fn test_price_alert_state() {
    let mut state = AppState::new();

    assert!(state.price_alerts.is_empty());

    let alert = PriceAlert {
        id: 1,
        user_id: 1,
        product_id: 1,
        store_id: Some(1),
        target_price: 1000,
        is_active: true,
        notification_sent: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state.add_price_alert(alert.clone());
    assert_eq!(state.price_alerts.len(), 1);
    assert_eq!(state.price_alerts[0].target_price, 1000);

    // Test updating alert
    let mut updated_alert = alert;
    updated_alert.target_price = 900;
    state.update_price_alert(updated_alert);
    assert_eq!(state.price_alerts[0].target_price, 900);

    // Test removing alert
    state.remove_price_alert(1);
    assert!(state.price_alerts.is_empty());
}

#[test]
fn test_review_state() {
    let mut state = AppState::new();

    assert!(state.product_reviews.is_empty());
    assert!(state.current_review.is_none());

    let review = UserReview {
        id: 1,
        user_id: 1,
        product_id: 1,
        store_id: 1,
        rating: 4,
        comment: Some("Great product!".to_string()),
        is_verified: true,
        helpful_count: 5,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state.set_product_reviews(vec![review.clone()]);
    assert_eq!(state.product_reviews.len(), 1);
    assert_eq!(state.product_reviews[0].rating, 4);

    state.set_current_review(Some(review));
    assert!(state.current_review.is_some());
    assert_eq!(state.current_review.as_ref().unwrap().rating, 4);
}
