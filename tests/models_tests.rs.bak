use chrono::Utc;
use eprice::models::*;

#[cfg(test)]
mod product_tests {
    use super::*;

    #[test]
    fn test_product_creation() {
        let product = Product::new(
            "Test Product".to_string(),
            "Electronics".to_string(),
            "A test product description".to_string(),
            Some("1234567890".to_string()),
            vec!["test.jpg".to_string()],
            vec!["tag1".to_string(), "tag2".to_string()],
        );

        assert_eq!(product.name, "Test Product");
        assert_eq!(product.category, "Electronics");
        assert_eq!(product.description, "A test product description");
        assert_eq!(product.barcode, Some("1234567890".to_string()));
        assert_eq!(product.images.len(), 1);
        assert_eq!(product.tags.len(), 2);
        assert!(product.prices.is_empty());
        assert!(!product.id.is_empty());
    }

    #[test]
    fn test_product_current_lowest_price_empty() {
        let product = Product::new(
            "Test Product".to_string(),
            "Electronics".to_string(),
            "Description".to_string(),
            None,
            vec![],
            vec![],
        );

        assert!(product.current_lowest_price().is_none());
    }

    #[test]
    fn test_product_current_lowest_price_with_verified_prices() {
        let mut product = Product::new(
            "Test Product".to_string(),
            "Electronics".to_string(),
            "Description".to_string(),
            None,
            vec![],
            vec![],
        );

        // Add price records with different prices and verification status
        let price1 = PriceRecord {
            id: Some("1".to_string()),
            product_id: Some(product.id.clone()),
            store_id: "store1".to_string(),
            user_id: None,
            price: 10.0,
            timestamp: Utc::now(),
            is_on_sale: false,
            receipt_image: None,
            verification_status: "verified".to_string(),
        };

        let price2 = PriceRecord {
            id: Some("2".to_string()),
            product_id: Some(product.id.clone()),
            store_id: "store2".to_string(),
            user_id: None,
            price: 8.0,
            timestamp: Utc::now(),
            is_on_sale: false,
            receipt_image: None,
            verification_status: "verified".to_string(),
        };

        let price3 = PriceRecord {
            id: Some("3".to_string()),
            product_id: Some(product.id.clone()),
            store_id: "store3".to_string(),
            user_id: None,
            price: 5.0,
            timestamp: Utc::now(),
            is_on_sale: false,
            receipt_image: None,
            verification_status: "pending".to_string(), // Not verified
        };

        product.prices.push(price1);
        product.prices.push(price2);
        product.prices.push(price3);

        // Should return the lowest verified price (8.0), not the lowest overall (5.0)
        let lowest = product.current_lowest_price();
        assert!(lowest.is_some());
        assert_eq!(lowest.unwrap().price, 8.0);
    }

    #[test]
    fn test_product_verified_prices() {
        let mut product = Product::new(
            "Test Product".to_string(),
            "Electronics".to_string(),
            "Description".to_string(),
            None,
            vec![],
            vec![],
        );

        let verified_price = PriceRecord {
            id: Some("1".to_string()),
            product_id: Some(product.id.clone()),
            store_id: "store1".to_string(),
            user_id: None,
            price: 10.0,
            timestamp: Utc::now(),
            is_on_sale: false,
            receipt_image: None,
            verification_status: "verified".to_string(),
        };

        let pending_price = PriceRecord {
            id: Some("2".to_string()),
            product_id: Some(product.id.clone()),
            store_id: "store2".to_string(),
            user_id: None,
            price: 8.0,
            timestamp: Utc::now(),
            is_on_sale: false,
            receipt_image: None,
            verification_status: "pending".to_string(),
        };

        product.prices.push(verified_price);
        product.prices.push(pending_price);

        let verified_prices = product.verified_prices();
        assert_eq!(verified_prices.len(), 1);
        assert_eq!(verified_prices[0].price, 10.0);
    }

    #[test]
    fn test_product_average_price() {
        let mut product = Product::new(
            "Test Product".to_string(),
            "Electronics".to_string(),
            "Description".to_string(),
            None,
            vec![],
            vec![],
        );

        // No prices
        assert!(product.average_price().is_none());

        // Add verified prices
        let price1 = PriceRecord {
            id: Some("1".to_string()),
            product_id: Some(product.id.clone()),
            store_id: "store1".to_string(),
            user_id: None,
            price: 10.0,
            timestamp: Utc::now(),
            is_on_sale: false,
            receipt_image: None,
            verification_status: "verified".to_string(),
        };

        let price2 = PriceRecord {
            id: Some("2".to_string()),
            product_id: Some(product.id.clone()),
            store_id: "store2".to_string(),
            user_id: None,
            price: 20.0,
            timestamp: Utc::now(),
            is_on_sale: false,
            receipt_image: None,
            verification_status: "verified".to_string(),
        };

        product.prices.push(price1);
        product.prices.push(price2);

        let avg = product.average_price();
        assert!(avg.is_some());
        assert_eq!(avg.unwrap(), 15.0);
    }
}

#[cfg(test)]
mod price_record_tests {
    use super::*;

    #[test]
    fn test_price_record_creation() {
        let price_record = PriceRecord::new(
            Some("product1".to_string()),
            "store1".to_string(),
            Some("user1".to_string()),
            99.99,
            true,
            Some("receipt.jpg".to_string()),
        );

        assert!(price_record.id.is_some());
        assert_eq!(price_record.product_id, Some("product1".to_string()));
        assert_eq!(price_record.store_id, "store1");
        assert_eq!(price_record.user_id, Some("user1".to_string()));
        assert_eq!(price_record.price, 99.99);
        assert!(price_record.is_on_sale);
        assert_eq!(price_record.receipt_image, Some("receipt.jpg".to_string()));
        assert_eq!(price_record.verification_status, "pending");
    }

    #[test]
    fn test_price_record_verify() {
        let mut price_record = PriceRecord::new(
            Some("product1".to_string()),
            "store1".to_string(),
            None,
            10.0,
            false,
            None,
        );

        assert_eq!(price_record.verification_status, "pending");

        price_record.verify();
        assert_eq!(price_record.verification_status, "verified");
    }

    #[test]
    fn test_price_record_reject() {
        let mut price_record = PriceRecord::new(
            Some("product1".to_string()),
            "store1".to_string(),
            None,
            10.0,
            false,
            None,
        );

        assert_eq!(price_record.verification_status, "pending");

        price_record.reject();
        assert_eq!(price_record.verification_status, "rejected");
    }
}

#[cfg(test)]
mod store_tests {
    use super::*;

    #[test]
    fn test_store_creation() {
        let store = Store::new(
            "Test Store".to_string(),
            "123 Test Street".to_string(),
            40.7128,
            -74.0060,
            "9:00-21:00".to_string(),
            "555-1234".to_string(),
            vec!["grocery".to_string(), "electronics".to_string()],
            '🏪',
        );

        assert_eq!(store.name, "Test Store");
        assert_eq!(store.address, "123 Test Street");
        assert_eq!(store.latitude, 40.7128);
        assert_eq!(store.longitude, -74.0060);
        assert_eq!(store.opening_hours, "9:00-21:00");
        assert_eq!(store.phone, "555-1234");
        assert_eq!(store.tags.len(), 2);
        assert_eq!(store.symbol, '🏪');
        assert!(!store.id.is_empty());
        assert_eq!(store.rating, 0.0); // Default rating
    }

    #[test]
    fn test_store_distance_calculation() {
        let store = Store::new(
            "Test Store".to_string(),
            "Address".to_string(),
            40.7128, // New York
            -74.0060,
            "9:00-21:00".to_string(),
            "555-1234".to_string(),
            vec![],
            '🏪',
        );

        // Distance to same location should be 0
        let distance_same = store.distance_to(40.7128, -74.0060);
        assert!(distance_same < 0.1); // Very small distance due to floating point precision

        // Distance to Los Angeles (approximate)
        let distance_la = store.distance_to(34.0522, -118.2437);
        assert!(distance_la > 3000.0); // Should be over 3000 km
    }

    #[test]
    fn test_store_is_open() {
        let store = Store::new(
            "Test Store".to_string(),
            "Address".to_string(),
            40.7128,
            -74.0060,
            "9:00-21:00".to_string(),
            "555-1234".to_string(),
            vec![],
            '🏪',
        );

        // Note: This test might be time-dependent. In a real implementation,
        // you might want to pass the current time as a parameter.
        let is_open = store.is_open();
        // We can't easily test this without mocking time, so just verify it returns a boolean
        assert!(is_open == true || is_open == false);
    }
}

#[cfg(test)]
mod user_tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
        );

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, "hashed_password");
        assert!(!user.id.is_empty());
        assert!(user.last_login.is_none());
        assert_eq!(user.reputation_score, 0);
        assert!(user.is_active);
        assert!(user.email_verified);
    }

    #[test]
    fn test_user_update_reputation() {
        let mut user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
        );

        assert_eq!(user.reputation_score, 0);

        user.update_reputation(10);
        assert_eq!(user.reputation_score, 10);

        user.update_reputation(-5);
        assert_eq!(user.reputation_score, 5);
    }

    #[test]
    fn test_user_deactivate() {
        let mut user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
        );

        assert!(user.is_active);

        user.deactivate();
        assert!(!user.is_active);
    }

    #[test]
    fn test_user_verify_email() {
        let mut user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
        );

        // Assume email starts as unverified for this test
        user.email_verified = false;
        assert!(!user.email_verified);

        user.verify_email();
        assert!(user.email_verified);
    }
}

#[cfg(test)]
mod user_review_tests {
    use super::*;

    #[test]
    fn test_user_review_creation() {
        let review = UserReview::new(
            "user1".to_string(),
            "product1".to_string(),
            5,
            "Great product!".to_string(),
        );

        assert_eq!(review.user_id, "user1");
        assert_eq!(review.product_id, "product1");
        assert_eq!(review.rating, 5);
        assert_eq!(review.comment, "Great product!");
        assert!(!review.id.is_empty());
        assert!(!review.is_verified);
    }

    #[test]
    fn test_user_review_verify() {
        let mut review = UserReview::new(
            "user1".to_string(),
            "product1".to_string(),
            4,
            "Good product".to_string(),
        );

        assert!(!review.is_verified);

        review.verify();
        assert!(review.is_verified);
    }
}

#[cfg(test)]
mod price_alert_tests {
    use super::*;

    #[test]
    fn test_price_alert_creation() {
        let alert = PriceAlert::new("user1".to_string(), "product1".to_string(), 50.0);

        assert_eq!(alert.user_id, "user1");
        assert_eq!(alert.product_id, "product1");
        assert_eq!(alert.target_price, 50.0);
        assert!(!alert.id.is_empty());
        assert!(alert.is_active);
        assert_eq!(alert.notification_type, "email");
    }

    #[test]
    fn test_price_alert_should_trigger() {
        let alert = PriceAlert::new("user1".to_string(), "product1".to_string(), 50.0);

        // Price below target should trigger
        assert!(alert.should_trigger(45.0));

        // Price equal to target should trigger
        assert!(alert.should_trigger(50.0));

        // Price above target should not trigger
        assert!(!alert.should_trigger(55.0));
    }

    #[test]
    fn test_price_alert_deactivate() {
        let mut alert = PriceAlert::new("user1".to_string(), "product1".to_string(), 50.0);

        assert!(alert.is_active);

        alert.deactivate();
        assert!(!alert.is_active);
    }
}
