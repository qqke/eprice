use eprice::models::*;
use eprice::utils::*;

#[test]
fn test_distance_calculation() {
    // Test distance between Tokyo Station and Shibuya
    let tokyo_lat = 35.6762;
    let tokyo_lon = 139.6503;
    let shibuya_lat = 35.6598;
    let shibuya_lon = 139.7006;

    let distance = calculate_distance(tokyo_lat, tokyo_lon, shibuya_lat, shibuya_lon);

    // Distance should be approximately 4.3 km
    assert!(distance > 4.0 && distance < 5.0);
}

#[test]
fn test_distance_calculation_same_point() {
    let lat = 35.6762;
    let lon = 139.6503;

    let distance = calculate_distance(lat, lon, lat, lon);
    assert_eq!(distance, 0.0);
}

#[test]
fn test_distance_calculation_far_points() {
    // Test distance between Tokyo and New York
    let tokyo_lat = 35.6762;
    let tokyo_lon = 139.6503;
    let ny_lat = 40.7128;
    let ny_lon = -74.0060;

    let distance = calculate_distance(tokyo_lat, tokyo_lon, ny_lat, ny_lon);

    // Distance should be approximately 10,800 km
    assert!(distance > 10000.0 && distance < 12000.0);
}

#[test]
fn test_format_price_jpy() {
    assert_eq!(format_price(1000, Currency::JPY), "¥1,000");
    assert_eq!(format_price(1234567, Currency::JPY), "¥1,234,567");
    assert_eq!(format_price(0, Currency::JPY), "¥0");
    assert_eq!(format_price(99, Currency::JPY), "¥99");
}

#[test]
fn test_format_price_usd() {
    assert_eq!(format_price(1000, Currency::USD), "$10.00");
    assert_eq!(format_price(1234567, Currency::USD), "$12,345.67");
    assert_eq!(format_price(0, Currency::USD), "$0.00");
    assert_eq!(format_price(99, Currency::USD), "$0.99");
    assert_eq!(format_price(1, Currency::USD), "$0.01");
}

#[test]
fn test_format_price_eur() {
    assert_eq!(format_price(1000, Currency::EUR), "€10.00");
    assert_eq!(format_price(1234567, Currency::EUR), "€12,345.67");
    assert_eq!(format_price(0, Currency::EUR), "€0.00");
    assert_eq!(format_price(99, Currency::EUR), "€0.99");
}

#[test]
fn test_format_distance_kilometers() {
    assert_eq!(format_distance(1000.0, DistanceUnit::Kilometers), "1.0 km");
    assert_eq!(format_distance(1500.0, DistanceUnit::Kilometers), "1.5 km");
    assert_eq!(format_distance(0.5, DistanceUnit::Kilometers), "0.5 km");
    assert_eq!(format_distance(0.0, DistanceUnit::Kilometers), "0.0 km");
    assert_eq!(format_distance(10.123, DistanceUnit::Kilometers), "10.1 km");
}

#[test]
fn test_format_distance_miles() {
    assert_eq!(format_distance(1000.0, DistanceUnit::Miles), "0.6 miles");
    assert_eq!(format_distance(1609.34, DistanceUnit::Miles), "1.0 miles");
    assert_eq!(format_distance(804.67, DistanceUnit::Miles), "0.5 miles");
    assert_eq!(format_distance(0.0, DistanceUnit::Miles), "0.0 miles");
}

#[test]
fn test_validate_email() {
    // Valid emails
    assert!(validate_email("test@example.com"));
    assert!(validate_email("user.name@domain.co.jp"));
    assert!(validate_email("user+tag@domain.org"));
    assert!(validate_email("user123@subdomain.domain.com"));

    // Invalid emails
    assert!(!validate_email("invalid"));
    assert!(!validate_email("@domain.com"));
    assert!(!validate_email("user@"));
    assert!(!validate_email("user@domain"));
    assert!(!validate_email("user..name@domain.com"));
    assert!(!validate_email(""));
}

#[test]
fn test_validate_password() {
    // Valid passwords
    assert!(validate_password("Password123!"));
    assert!(validate_password("MySecureP@ss1"));
    assert!(validate_password("Abcdef123!"));

    // Invalid passwords - too short
    assert!(!validate_password("Pass1!"));
    assert!(!validate_password(""));

    // Invalid passwords - missing uppercase
    assert!(!validate_password("password123!"));

    // Invalid passwords - missing lowercase
    assert!(!validate_password("PASSWORD123!"));

    // Invalid passwords - missing digit
    assert!(!validate_password("Password!"));

    // Invalid passwords - missing special character
    assert!(!validate_password("Password123"));
}

#[test]
fn test_validate_barcode() {
    // Valid barcodes
    assert!(validate_barcode("1234567890123")); // 13 digits
    assert!(validate_barcode("123456789012")); // 12 digits
    assert!(validate_barcode("12345678")); // 8 digits

    // Invalid barcodes
    assert!(!validate_barcode("12345")); // Too short
    assert!(!validate_barcode("12345678901234")); // Too long
    assert!(!validate_barcode("1234567890abc")); // Contains letters
    assert!(!validate_barcode("1234-5678-9012")); // Contains dashes
    assert!(!validate_barcode(""));
}

#[test]
fn test_sanitize_search_query() {
    assert_eq!(sanitize_search_query("  Normal Query  "), "Normal Query");
    assert_eq!(
        sanitize_search_query("Query\nWith\nNewlines"),
        "Query With Newlines"
    );
    assert_eq!(
        sanitize_search_query("Query\tWith\tTabs"),
        "Query With Tabs"
    );
    assert_eq!(
        sanitize_search_query("Query\r\nWith\r\nCRLF"),
        "Query With CRLF"
    );
    assert_eq!(
        sanitize_search_query("  Multiple   Spaces  "),
        "Multiple Spaces"
    );
    assert_eq!(sanitize_search_query(""), "");
    assert_eq!(sanitize_search_query("   "), "");
}

#[test]
fn test_format_datetime() {
    let datetime = chrono::Utc
        .with_ymd_and_hms(2023, 12, 25, 15, 30, 45)
        .unwrap();

    // Test different formats
    assert_eq!(
        format_datetime(&datetime, DateTimeFormat::Short),
        "12/25/23"
    );
    assert_eq!(
        format_datetime(&datetime, DateTimeFormat::Medium),
        "Dec 25, 2023"
    );
    assert_eq!(
        format_datetime(&datetime, DateTimeFormat::Long),
        "December 25, 2023 15:30"
    );
    assert_eq!(
        format_datetime(&datetime, DateTimeFormat::Full),
        "December 25, 2023 15:30:45 UTC"
    );
}

#[test]
fn test_parse_price() {
    // Valid price strings
    assert_eq!(parse_price("1000"), Ok(1000));
    assert_eq!(parse_price("10.50"), Ok(1050));
    assert_eq!(parse_price("1,234"), Ok(1234));
    assert_eq!(parse_price("1,234.56"), Ok(123456));
    assert_eq!(parse_price("0"), Ok(0));
    assert_eq!(parse_price("0.01"), Ok(1));

    // Invalid price strings
    assert!(parse_price("").is_err());
    assert!(parse_price("abc").is_err());
    assert!(parse_price("-10").is_err());
    assert!(parse_price("10.999").is_err());
}

#[test]
fn test_generate_barcode_checksum() {
    // Test EAN-13 checksum
    assert_eq!(generate_barcode_checksum("123456789012"), Some(8));
    assert_eq!(generate_barcode_checksum("400330101393"), Some(7));

    // Test UPC-A checksum
    assert_eq!(generate_barcode_checksum("03600029145"), Some(2));

    // Invalid inputs
    assert_eq!(generate_barcode_checksum("123"), None); // Too short
    assert_eq!(generate_barcode_checksum("12345678901234"), None); // Too long
    assert_eq!(generate_barcode_checksum("12345678901a"), None); // Contains letter
}

#[test]
fn test_truncate_string() {
    assert_eq!(truncate_string("Hello World", 5), "Hello");
    assert_eq!(truncate_string("Hello World", 20), "Hello World");
    assert_eq!(truncate_string("Hello World", 11), "Hello World");
    assert_eq!(truncate_string("", 5), "");
    assert_eq!(truncate_string("Hello", 0), "");
}

#[test]
fn test_slug_from_string() {
    assert_eq!(slug_from_string("Hello World"), "hello-world");
    assert_eq!(slug_from_string("Product Name 123"), "product-name-123");
    assert_eq!(
        slug_from_string("Special Characters!@#$%"),
        "special-characters"
    );
    assert_eq!(slug_from_string("  Multiple   Spaces  "), "multiple-spaces");
    assert_eq!(slug_from_string("Ünïcödé Tëxt"), "unicode-text");
    assert_eq!(slug_from_string(""), "");
}

#[test]
fn test_calculate_price_trend() {
    let prices = vec![
        (chrono::Utc::now() - chrono::Duration::days(7), 1000),
        (chrono::Utc::now() - chrono::Duration::days(5), 1100),
        (chrono::Utc::now() - chrono::Duration::days(3), 1050),
        (chrono::Utc::now() - chrono::Duration::days(1), 1200),
    ];

    let trend = calculate_price_trend(&prices);

    match trend {
        PriceTrend::Increasing => assert!(true), // Price increased from 1000 to 1200
        _ => assert!(false, "Expected increasing trend"),
    }
}

#[test]
fn test_calculate_price_trend_decreasing() {
    let prices = vec![
        (chrono::Utc::now() - chrono::Duration::days(7), 1200),
        (chrono::Utc::now() - chrono::Duration::days(5), 1100),
        (chrono::Utc::now() - chrono::Duration::days(3), 1050),
        (chrono::Utc::now() - chrono::Duration::days(1), 1000),
    ];

    let trend = calculate_price_trend(&prices);

    match trend {
        PriceTrend::Decreasing => assert!(true),
        _ => assert!(false, "Expected decreasing trend"),
    }
}

#[test]
fn test_calculate_price_trend_stable() {
    let prices = vec![
        (chrono::Utc::now() - chrono::Duration::days(7), 1000),
        (chrono::Utc::now() - chrono::Duration::days(5), 1005),
        (chrono::Utc::now() - chrono::Duration::days(3), 995),
        (chrono::Utc::now() - chrono::Duration::days(1), 1000),
    ];

    let trend = calculate_price_trend(&prices);

    match trend {
        PriceTrend::Stable => assert!(true),
        _ => assert!(false, "Expected stable trend"),
    }
}

#[test]
fn test_hash_password() {
    let password = "testpassword123";
    let hash = hash_password(password).expect("Failed to hash password");

    // Verify the hash is not the same as the original password
    assert_ne!(hash, password);

    // Verify the hash starts with bcrypt identifier
    assert!(hash.starts_with("$2b$"));

    // Verify we can verify the password
    assert!(verify_password(password, &hash).expect("Failed to verify password"));

    // Verify wrong password fails
    assert!(!verify_password("wrongpassword", &hash).expect("Failed to verify wrong password"));
}

#[test]
fn test_generate_user_token() {
    let user_id = 123;
    let token = generate_user_token(user_id);

    // Token should be non-empty
    assert!(!token.is_empty());

    // Token should be different each time
    let token2 = generate_user_token(user_id);
    assert_ne!(token, token2);

    // Token should contain user ID information
    assert!(verify_user_token(&token, user_id));
    assert!(!verify_user_token(&token, 456));
}
