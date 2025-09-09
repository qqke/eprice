use crate::models::{PriceRecord, Product};
use crate::services::{ServiceError, ServiceResult};
use chrono::Utc;
use std::collections::HashMap;

/// Product service for managing product operations and business logic
pub struct ProductService {
    /// In-memory product cache (in real app would use database)
    products: HashMap<String, Product>,
    /// Category mappings
    categories: Vec<String>,
}

impl ProductService {
    pub fn new() -> Self {
        let mut service = Self {
            products: HashMap::new(),
            categories: vec![
                "Beverages".to_string(),
                "Snacks".to_string(),
                "Food".to_string(),
                "Personal Care".to_string(),
                "Household".to_string(),
                "Electronics".to_string(),
                "Clothing".to_string(),
                "Books".to_string(),
                "Other".to_string(),
            ],
        };

        // Initialize with some sample products
        service.init_sample_products();
        service
    }

    /// Create a new product
    pub fn create_product(
        &mut self,
        name: String,
        category: String,
        description: String,
        barcode: Option<String>,
        tags: Vec<String>,
    ) -> ServiceResult<Product> {
        // Validate input
        self.validate_product_data(&name, &category, &description)?;

        // Check if barcode already exists
        if let Some(ref bc) = barcode {
            if self
                .products
                .values()
                .any(|p| p.barcode.as_ref() == Some(bc))
            {
                return Err(ServiceError::ValidationError(
                    "Barcode already exists".to_string(),
                ));
            }
        }

        // Create product
        let product = Product::new(name, category, description, barcode, vec![], tags);

        // Store product
        self.products.insert(product.id.clone(), product.clone());

        log::info!("Product created: {}", product.name);
        Ok(product)
    }

    /// Get product by ID
    pub fn get_product(&self, product_id: &str) -> ServiceResult<Product> {
        self.products
            .get(product_id)
            .cloned()
            .ok_or_else(|| ServiceError::NotFound(format!("Product {} not found", product_id)))
    }

    /// Get product by barcode
    pub fn get_product_by_barcode(&self, barcode: &str) -> ServiceResult<Option<Product>> {
        let product = self
            .products
            .values()
            .find(|p| p.barcode.as_ref() == Some(&barcode.to_string()))
            .cloned();

        Ok(product)
    }

    /// Update product information
    pub fn update_product(
        &mut self,
        product_id: &str,
        name: Option<String>,
        category: Option<String>,
        description: Option<String>,
        tags: Option<Vec<String>>,
    ) -> ServiceResult<Product> {
        // Validate inputs first
        if let Some(ref new_name) = name {
            self.validate_product_name(new_name)?;
        }

        if let Some(ref new_category) = category {
            self.validate_category(new_category)?;
        }

        if let Some(ref new_description) = description {
            self.validate_description(new_description)?;
        }

        // Now get mutable reference
        let product = self
            .products
            .get_mut(product_id)
            .ok_or_else(|| ServiceError::NotFound(format!("Product {} not found", product_id)))?;

        // Update fields if provided
        if let Some(new_name) = name {
            product.name = new_name;
        }

        if let Some(new_category) = category {
            product.category = new_category;
        }

        if let Some(new_description) = description {
            product.description = new_description;
        }

        if let Some(new_tags) = tags {
            product.tags = new_tags;
        }

        log::info!("Product updated: {}", product.name);
        Ok(product.clone())
    }

    /// Delete product
    pub fn delete_product(&mut self, product_id: &str) -> ServiceResult<()> {
        let product = self
            .products
            .remove(product_id)
            .ok_or_else(|| ServiceError::NotFound(format!("Product {} not found", product_id)))?;

        log::info!("Product deleted: {}", product.name);
        Ok(())
    }

    /// Search products
    pub fn search_products(
        &self,
        query: &str,
        category: Option<&str>,
    ) -> ServiceResult<Vec<Product>> {
        let query_lower = query.to_lowercase();

        let products: Vec<Product> = self
            .products
            .values()
            .filter(|p| {
                // Text search
                let matches_query = query.is_empty()
                    || p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
                    || p.tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower));

                // Category filter
                let matches_category = category.map_or(true, |cat| p.category == cat);

                matches_query && matches_category
            })
            .cloned()
            .collect();

        Ok(products)
    }

    /// Get products by category
    pub fn get_products_by_category(&self, category: &str) -> ServiceResult<Vec<Product>> {
        let products: Vec<Product> = self
            .products
            .values()
            .filter(|p| p.category == category)
            .cloned()
            .collect();

        Ok(products)
    }

    /// Get all categories
    pub fn get_categories(&self) -> Vec<String> {
        self.categories.clone()
    }

    /// Add category
    pub fn add_category(&mut self, category: String) -> ServiceResult<()> {
        if self.categories.contains(&category) {
            return Err(ServiceError::ValidationError(
                "Category already exists".to_string(),
            ));
        }

        self.categories.push(category.clone());
        log::info!("Category added: {}", category);
        Ok(())
    }

    /// Get products with pagination
    pub fn list_products(&self, offset: usize, limit: usize) -> ServiceResult<Vec<Product>> {
        let products: Vec<Product> = self
            .products
            .values()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();

        Ok(products)
    }

    /// Add price record to product
    pub fn add_price_record(
        &mut self,
        product_id: &str,
        price_record: PriceRecord,
    ) -> ServiceResult<()> {
        let product = self
            .products
            .get_mut(product_id)
            .ok_or_else(|| ServiceError::NotFound(format!("Product {} not found", product_id)))?;

        // Validate price record
        if price_record.price <= 0.0 {
            return Err(ServiceError::ValidationError(
                "Price must be positive".to_string(),
            ));
        }

        product.prices.push(price_record);

        log::info!("Price record added to product: {}", product.name);
        Ok(())
    }

    /// Get current lowest price for product
    pub fn get_current_lowest_price(&self, product_id: &str) -> ServiceResult<Option<f64>> {
        let product = self.get_product(product_id)?;

        let lowest_price = product
            .verified_prices()
            .iter()
            .map(|p| p.price)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        Ok(lowest_price)
    }

    /// Get price history for product
    pub fn get_price_history(
        &self,
        product_id: &str,
        days: i64,
    ) -> ServiceResult<Vec<PriceRecord>> {
        let product = self.get_product(product_id)?;

        let cutoff_date = Utc::now() - chrono::Duration::days(days);

        let price_history: Vec<PriceRecord> = product
            .prices
            .iter()
            .filter(|p| p.timestamp > cutoff_date && p.verification_status == "verified")
            .cloned()
            .collect();

        Ok(price_history)
    }

    /// Get trending products (most price updates recently)
    pub fn get_trending_products(&self, limit: usize) -> ServiceResult<Vec<Product>> {
        let mut products_with_activity: Vec<(Product, usize)> = self
            .products
            .values()
            .map(|p| {
                let recent_activity = p
                    .prices
                    .iter()
                    .filter(|price| {
                        let one_week_ago = Utc::now() - chrono::Duration::days(7);
                        price.timestamp > one_week_ago
                    })
                    .count();
                (p.clone(), recent_activity)
            })
            .collect();

        // Sort by activity level
        products_with_activity.sort_by(|a, b| b.1.cmp(&a.1));

        let trending_products: Vec<Product> = products_with_activity
            .into_iter()
            .take(limit)
            .map(|(product, _)| product)
            .collect();

        Ok(trending_products)
    }

    /// Get product statistics
    pub fn get_product_stats(&self) -> ServiceResult<ProductStats> {
        let total_products = self.products.len();
        let total_prices = self.products.values().map(|p| p.prices.len()).sum();

        let category_counts: HashMap<String, usize> =
            self.products
                .values()
                .fold(HashMap::new(), |mut acc, product| {
                    *acc.entry(product.category.clone()).or_insert(0) += 1;
                    acc
                });

        let verified_prices = self
            .products
            .values()
            .flat_map(|p| &p.prices)
            .filter(|price| price.verification_status == "verified")
            .count();

        Ok(ProductStats {
            total_products,
            total_prices,
            verified_prices,
            category_counts,
            categories: self.categories.clone(),
        })
    }

    // Helper methods

    fn validate_product_data(
        &self,
        name: &str,
        category: &str,
        description: &str,
    ) -> ServiceResult<()> {
        self.validate_product_name(name)?;
        self.validate_category(category)?;
        self.validate_description(description)?;
        Ok(())
    }

    fn validate_product_name(&self, name: &str) -> ServiceResult<()> {
        if name.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                "Product name cannot be empty".to_string(),
            ));
        }

        if name.len() > 200 {
            return Err(ServiceError::ValidationError(
                "Product name too long".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_category(&self, category: &str) -> ServiceResult<()> {
        if !self.categories.contains(&category.to_string()) {
            return Err(ServiceError::ValidationError(format!(
                "Invalid category: {}",
                category
            )));
        }

        Ok(())
    }

    fn validate_description(&self, description: &str) -> ServiceResult<()> {
        if description.len() > 1000 {
            return Err(ServiceError::ValidationError(
                "Description too long".to_string(),
            ));
        }

        Ok(())
    }

    fn init_sample_products(&mut self) {
        let sample_products = vec![
            Product::new(
                "Coca Cola 500ml".to_string(),
                "Beverages".to_string(),
                "Classic Coca Cola in 500ml bottle".to_string(),
                Some("4901234567890".to_string()),
                vec![],
                vec!["cola".to_string(), "soft drink".to_string()],
            ),
            Product::new(
                "Potato Chips Original".to_string(),
                "Snacks".to_string(),
                "Crispy original flavor potato chips".to_string(),
                Some("4901234567891".to_string()),
                vec![],
                vec!["chips".to_string(), "snack".to_string()],
            ),
            Product::new(
                "Mineral Water 330ml".to_string(),
                "Beverages".to_string(),
                "Natural mineral water in 330ml bottle".to_string(),
                Some("12345678".to_string()),
                vec![],
                vec!["water".to_string(), "drink".to_string()],
            ),
        ];

        for product in sample_products {
            self.products.insert(product.id.clone(), product);
        }
    }
}

impl Default for ProductService {
    fn default() -> Self {
        Self::new()
    }
}

/// Product statistics
#[derive(Debug, Clone)]
pub struct ProductStats {
    pub total_products: usize,
    pub total_prices: usize,
    pub verified_prices: usize,
    pub category_counts: HashMap<String, usize>,
    pub categories: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PriceRecord;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_product_creation_success() {
        let mut service = ProductService::new();

        let result = service.create_product(
            "Test Product".to_string(),
            "Electronics".to_string(),
            "A test product".to_string(),
            Some("1234567890123".to_string()),
            vec!["test".to_string()],
        );

        assert!(result.is_ok());
        let product = result.unwrap();
        assert_eq!(product.name, "Test Product");
        assert_eq!(product.category, "Electronics");
        assert_eq!(product.barcode, Some("1234567890123".to_string()));
    }

    #[test]
    fn test_product_creation_invalid_category() {
        let mut service = ProductService::new();

        let result = service.create_product(
            "Test Product".to_string(),
            "InvalidCategory".to_string(),
            "A test product".to_string(),
            None,
            vec![],
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_product_creation_duplicate_barcode() {
        let mut service = ProductService::new();

        // Create first product
        service
            .create_product(
                "Product 1".to_string(),
                "Electronics".to_string(),
                "First product".to_string(),
                Some("1234567890123".to_string()),
                vec![],
            )
            .unwrap();

        // Try to create second product with same barcode
        let result = service.create_product(
            "Product 2".to_string(),
            "Electronics".to_string(),
            "Second product".to_string(),
            Some("1234567890123".to_string()),
            vec![],
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_get_product_by_id() {
        let mut service = ProductService::new();

        let product = service
            .create_product(
                "Test Product".to_string(),
                "Electronics".to_string(),
                "A test product".to_string(),
                None,
                vec![],
            )
            .unwrap();

        let result = service.get_product(&product.id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Test Product");
    }

    #[test]
    fn test_get_product_by_barcode() {
        let mut service = ProductService::new();

        service
            .create_product(
                "Test Product".to_string(),
                "Electronics".to_string(),
                "A test product".to_string(),
                Some("1234567890123".to_string()),
                vec![],
            )
            .unwrap();

        let result = service.get_product_by_barcode("1234567890123");
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        let result = service.get_product_by_barcode("nonexistent");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_update_product() {
        let mut service = ProductService::new();

        let product = service
            .create_product(
                "Test Product".to_string(),
                "Electronics".to_string(),
                "A test product".to_string(),
                None,
                vec![],
            )
            .unwrap();

        let result = service.update_product(
            &product.id,
            Some("Updated Product".to_string()),
            None,
            Some("Updated description".to_string()),
            Some(vec!["updated".to_string()]),
        );

        assert!(result.is_ok());
        let updated_product = result.unwrap();
        assert_eq!(updated_product.name, "Updated Product");
        assert_eq!(updated_product.description, "Updated description");
        assert_eq!(updated_product.tags, vec!["updated".to_string()]);
    }

    #[test]
    fn test_delete_product() {
        let mut service = ProductService::new();

        let product = service
            .create_product(
                "Test Product".to_string(),
                "Electronics".to_string(),
                "A test product".to_string(),
                None,
                vec![],
            )
            .unwrap();

        let result = service.delete_product(&product.id);
        assert!(result.is_ok());

        let result = service.get_product(&product.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_products() {
        let mut service = ProductService::new();

        service
            .create_product(
                "iPhone 15".to_string(),
                "Electronics".to_string(),
                "Latest iPhone model".to_string(),
                None,
                vec!["apple".to_string(), "smartphone".to_string()],
            )
            .unwrap();

        service
            .create_product(
                "Samsung Galaxy".to_string(),
                "Electronics".to_string(),
                "Android smartphone".to_string(),
                None,
                vec!["samsung".to_string(), "android".to_string()],
            )
            .unwrap();

        // Search by name
        let results = service.search_products("iPhone", None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "iPhone 15");

        // Search by tag
        let results = service.search_products("smartphone", None).unwrap();
        assert_eq!(results.len(), 2);

        // Search with category filter
        let results = service.search_products("", Some("Electronics")).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_get_products_by_category() {
        let mut service = ProductService::new();

        service
            .create_product(
                "iPhone".to_string(),
                "Electronics".to_string(),
                "Smartphone".to_string(),
                None,
                vec![],
            )
            .unwrap();

        service
            .create_product(
                "Chips".to_string(),
                "Food".to_string(),
                "Snack".to_string(),
                None,
                vec![],
            )
            .unwrap();

        let electronics = service.get_products_by_category("Electronics").unwrap();
        assert_eq!(electronics.len(), 1);

        let food = service.get_products_by_category("Food").unwrap();
        assert_eq!(food.len(), 1);
    }

    #[test]
    fn test_category_management() {
        let mut service = ProductService::new();

        let categories = service.get_categories();
        let initial_count = categories.len();

        let result = service.add_category("NewCategory".to_string());
        assert!(result.is_ok());

        let categories = service.get_categories();
        assert_eq!(categories.len(), initial_count + 1);
        assert!(categories.contains(&"NewCategory".to_string()));

        // Try to add duplicate category
        let result = service.add_category("NewCategory".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_product_stats() {
        let mut service = ProductService::new();

        // Create some products
        service
            .create_product(
                "Product 1".to_string(),
                "Electronics".to_string(),
                "Description 1".to_string(),
                None,
                vec![],
            )
            .unwrap();

        service
            .create_product(
                "Product 2".to_string(),
                "Food".to_string(),
                "Description 2".to_string(),
                None,
                vec![],
            )
            .unwrap();

        let stats = service.get_product_stats().unwrap();
        assert_eq!(stats.total_products, 2);
        assert!(stats.category_counts.contains_key("Electronics"));
        assert!(stats.category_counts.contains_key("Food"));
    }
}
