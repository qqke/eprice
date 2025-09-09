use crate::models::Product;
use crate::scanner::models::{BarcodeType, ScanResult};
use crate::scanner::ScannerError;
use anyhow::Result;
use std::collections::HashMap;

/// Product matcher for finding products based on barcode scans
pub struct ProductMatcher {
    /// Cache of barcode to product mappings
    barcode_cache: HashMap<String, Product>,
    /// Similarity threshold for fuzzy matching
    similarity_threshold: f32,
}

impl ProductMatcher {
    pub fn new() -> Self {
        let mut matcher = Self {
            barcode_cache: HashMap::new(),
            similarity_threshold: 0.7,
        };

        // Initialize with some mock products
        matcher.init_mock_products();
        matcher
    }

    pub fn with_similarity_threshold(mut self, threshold: f32) -> Self {
        self.similarity_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Find product by exact barcode match
    pub fn find_product_by_barcode(&self, barcode: &str) -> Result<Option<Product>, ScannerError> {
        log::info!("Looking up product for barcode: {}", barcode);

        // Check cache first
        if let Some(product) = self.barcode_cache.get(barcode) {
            log::info!("Found product in cache: {}", product.name);
            return Ok(Some(product.clone()));
        }

        // In a real implementation, this would query the database
        // For now, we'll generate a mock product based on the barcode
        if self.is_valid_barcode(barcode) {
            let product = self.generate_product_from_barcode(barcode)?;
            log::info!("Generated mock product: {}", product.name);
            Ok(Some(product))
        } else {
            log::warn!("Invalid barcode format: {}", barcode);
            Ok(None)
        }
    }

    /// Find product by scan result
    pub fn find_product_by_scan(
        &self,
        scan_result: &ScanResult,
    ) -> Result<Option<Product>, ScannerError> {
        self.find_product_by_barcode(&scan_result.barcode)
    }

    /// Search for products by partial barcode or name
    pub fn search_products(&self, query: &str) -> Result<Vec<ProductMatch>, ScannerError> {
        let mut matches = Vec::new();

        // Search in cache
        for (barcode, product) in &self.barcode_cache {
            let similarity = self.calculate_similarity(query, &product.name, barcode);

            if similarity >= self.similarity_threshold {
                matches.push(ProductMatch {
                    product: product.clone(),
                    similarity,
                    match_type: if similarity > 0.95 {
                        ProductMatchType::Exact
                    } else if similarity > 0.8 {
                        ProductMatchType::High
                    } else {
                        ProductMatchType::Partial
                    },
                });
            }
        }

        // Sort by similarity
        matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        Ok(matches)
    }

    /// Add product to cache
    pub fn add_product(&mut self, barcode: String, product: Product) {
        self.barcode_cache.insert(barcode, product);
    }

    /// Remove product from cache
    pub fn remove_product(&mut self, barcode: &str) -> Option<Product> {
        self.barcode_cache.remove(barcode)
    }

    /// Clear all cached products
    pub fn clear_cache(&mut self) {
        self.barcode_cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.barcode_cache.len()
    }

    /// Check if barcode is valid format
    fn is_valid_barcode(&self, barcode: &str) -> bool {
        // EAN-13
        if barcode.len() == 13 && barcode.chars().all(|c| c.is_ascii_digit()) {
            return self.validate_ean13(barcode);
        }

        // EAN-8
        if barcode.len() == 8 && barcode.chars().all(|c| c.is_ascii_digit()) {
            return self.validate_ean8(barcode);
        }

        // Code 128 - more flexible
        if barcode.len() >= 3 && barcode.len() <= 20 {
            return true;
        }

        // QR Code - can be any reasonable length
        if barcode.len() >= 1 && barcode.len() <= 1000 {
            return true;
        }

        false
    }

    /// Validate EAN-13 check digit
    fn validate_ean13(&self, barcode: &str) -> bool {
        if barcode.len() != 13 {
            return false;
        }

        let digits: Vec<u32> = barcode.chars().filter_map(|c| c.to_digit(10)).collect();

        if digits.len() != 13 {
            return false;
        }

        let mut sum = 0;
        for i in 0..12 {
            sum += if i % 2 == 0 { digits[i] } else { digits[i] * 3 };
        }

        let check_digit = (10 - (sum % 10)) % 10;
        check_digit == digits[12]
    }

    /// Validate EAN-8 check digit
    fn validate_ean8(&self, barcode: &str) -> bool {
        if barcode.len() != 8 {
            return false;
        }

        let digits: Vec<u32> = barcode.chars().filter_map(|c| c.to_digit(10)).collect();

        if digits.len() != 8 {
            return false;
        }

        let mut sum = 0;
        for i in 0..7 {
            sum += if i % 2 == 0 { digits[i] * 3 } else { digits[i] };
        }

        let check_digit = (10 - (sum % 10)) % 10;
        check_digit == digits[7]
    }

    /// Generate a mock product from barcode
    fn generate_product_from_barcode(&self, barcode: &str) -> Result<Product, ScannerError> {
        let barcode_type = self.determine_barcode_type(barcode);

        let (name, category, _price) = match barcode_type {
            BarcodeType::Ean13 | BarcodeType::Ean8 => {
                // Use barcode prefix to determine product type
                let prefix = &barcode[..std::cmp::min(3, barcode.len())];
                match prefix {
                    "490" | "491" | "492" => (
                        format!(
                            "Japanese Product {}",
                            &barcode[3..std::cmp::min(8, barcode.len())]
                        ),
                        "Food".to_string(),
                        150.0 + (barcode.len() as f64 * 10.0),
                    ),
                    "123" => (
                        format!("Beverage {}", &barcode[3..std::cmp::min(8, barcode.len())]),
                        "Drinks".to_string(),
                        120.0,
                    ),
                    "789" => (
                        format!("Snack {}", &barcode[3..std::cmp::min(8, barcode.len())]),
                        "Snacks".to_string(),
                        200.0,
                    ),
                    _ => (
                        format!("Product {}", &barcode[..std::cmp::min(6, barcode.len())]),
                        "General".to_string(),
                        100.0,
                    ),
                }
            }
            BarcodeType::Code128 => {
                if barcode.starts_with("PROD") {
                    (
                        format!("Product {}", &barcode[4..]),
                        "Manufactured".to_string(),
                        250.0,
                    )
                } else {
                    (format!("Item {}", barcode), "General".to_string(), 180.0)
                }
            }
            BarcodeType::QrCode => {
                if barcode.starts_with("http") {
                    ("Online Product".to_string(), "Digital".to_string(), 99.0)
                } else {
                    (
                        format!("QR Product {}", barcode),
                        "QR Code".to_string(),
                        150.0,
                    )
                }
            }
            BarcodeType::Unknown => (
                format!("Unknown Product {}", barcode),
                "Unknown".to_string(),
                50.0,
            ),
        };

        Ok(Product {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            category,
            description: format!("Auto-generated product for barcode {}", barcode),
            barcode: Some(barcode.to_string()),
            images: vec![],
            prices: vec![],
            tags: vec![],
            created_at: chrono::Utc::now(),
        })
    }

    /// Determine barcode type from string
    fn determine_barcode_type(&self, barcode: &str) -> BarcodeType {
        if barcode.len() == 13 && barcode.chars().all(|c| c.is_ascii_digit()) {
            BarcodeType::Ean13
        } else if barcode.len() == 8 && barcode.chars().all(|c| c.is_ascii_digit()) {
            BarcodeType::Ean8
        } else if barcode.starts_with("http") || barcode.contains("://") {
            BarcodeType::QrCode
        } else if barcode.len() >= 3 && barcode.len() <= 20 {
            BarcodeType::Code128
        } else {
            BarcodeType::Unknown
        }
    }

    /// Calculate similarity between query and product information
    fn calculate_similarity(&self, query: &str, product_name: &str, barcode: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let name_lower = product_name.to_lowercase();

        // Exact barcode match
        if query == barcode {
            return 1.0;
        }

        // Exact name match
        if query_lower == name_lower {
            return 0.95;
        }

        // Substring matches
        if name_lower.contains(&query_lower) {
            return 0.8;
        }

        if query_lower.contains(&name_lower) {
            return 0.75;
        }

        // Partial barcode match
        if barcode.contains(query) || query.contains(barcode) {
            return 0.7;
        }

        // Word-based similarity
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let name_words: Vec<&str> = name_lower.split_whitespace().collect();

        let common_words = query_words
            .iter()
            .filter(|word| name_words.contains(word))
            .count();

        if common_words > 0 {
            let total_words = (query_words.len() + name_words.len()) as f32;
            (common_words as f32 * 2.0) / total_words
        } else {
            0.0
        }
    }

    /// Initialize with mock products for testing
    fn init_mock_products(&mut self) {
        let mock_products = vec![
            (
                "4901234567890",
                Product {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: "Coca Cola 500ml".to_string(),
                    category: "Beverages".to_string(),
                    description: "Classic Coca Cola 500ml bottle".to_string(),
                    barcode: Some("4901234567890".to_string()),
                    images: vec![],
                    prices: vec![],
                    tags: vec!["beverage".to_string(), "cola".to_string()],
                    created_at: chrono::Utc::now(),
                },
            ),
            (
                "4901234567891",
                Product {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: "Potato Chips Original".to_string(),
                    category: "Snacks".to_string(),
                    description: "Original flavor potato chips".to_string(),
                    barcode: Some("4901234567891".to_string()),
                    images: vec![],
                    prices: vec![],
                    tags: vec!["snack".to_string(), "chips".to_string()],
                    created_at: chrono::Utc::now(),
                },
            ),
            (
                "12345678",
                Product {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: "Mineral Water 330ml".to_string(),
                    category: "Beverages".to_string(),
                    description: "Natural mineral water".to_string(),
                    barcode: Some("12345678".to_string()),
                    images: vec![],
                    prices: vec![],
                    tags: vec!["water".to_string(), "beverage".to_string()],
                    created_at: chrono::Utc::now(),
                },
            ),
        ];

        for (barcode, product) in mock_products {
            self.barcode_cache.insert(barcode.to_string(), product);
        }
    }
}

impl Default for ProductMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Product match result
#[derive(Debug, Clone)]
pub struct ProductMatch {
    pub product: Product,
    pub similarity: f32,
    pub match_type: ProductMatchType,
}

/// Type of product match
#[derive(Debug, Clone, PartialEq)]
pub enum ProductMatchType {
    Exact,
    High,
    Partial,
}
