use crate::models::PriceRecord;
use crate::services::{ServiceError, ServiceResult};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Price service for managing price operations and business logic
pub struct PriceService {
    /// In-memory price cache (in real app would use database)
    price_records: HashMap<String, PriceRecord>,
}

impl PriceService {
    pub fn new() -> Self {
        Self {
            price_records: HashMap::new(),
        }
    }

    /// Submit a new price record
    pub fn submit_price(
        &mut self,
        product_id: String,
        store_id: String,
        user_id: Option<String>,
        price: f64,
        is_on_sale: bool,
        receipt_image: Option<String>,
    ) -> ServiceResult<PriceRecord> {
        // Validate input
        self.validate_price_submission(price)?;

        // Create price record
        let price_record = PriceRecord::new(
            Some(product_id),
            store_id,
            user_id,
            price,
            is_on_sale,
            receipt_image,
        );

        // Store price record
        if let Some(ref id) = price_record.id {
            self.price_records.insert(id.clone(), price_record.clone());
        }

        log::info!(
            "Price submitted: ¥{:.2} for product {}",
            price,
            price_record
                .product_id
                .as_ref()
                .unwrap_or(&"unknown".to_string())
        );
        Ok(price_record)
    }

    /// Get price record by ID
    pub fn get_price_record(&self, price_id: &str) -> ServiceResult<PriceRecord> {
        self.price_records
            .get(price_id)
            .cloned()
            .ok_or_else(|| ServiceError::NotFound(format!("Price record {} not found", price_id)))
    }

    /// Verify a price record
    pub fn verify_price(&mut self, price_id: &str, verified: bool) -> ServiceResult<PriceRecord> {
        let price_record = self.price_records.get_mut(price_id).ok_or_else(|| {
            ServiceError::NotFound(format!("Price record {} not found", price_id))
        })?;

        if verified {
            price_record.verify();
        } else {
            price_record.reject();
        }

        log::info!(
            "Price record {} {}",
            price_id,
            if verified { "verified" } else { "rejected" }
        );
        Ok(price_record.clone())
    }

    /// Get price records for a product
    pub fn get_product_prices(&self, product_id: &str) -> ServiceResult<Vec<PriceRecord>> {
        let prices: Vec<PriceRecord> = self
            .price_records
            .values()
            .filter(|p| p.product_id.as_ref() == Some(&product_id.to_string()))
            .cloned()
            .collect();

        Ok(prices)
    }

    /// Get verified price records for a product
    pub fn get_verified_product_prices(&self, product_id: &str) -> ServiceResult<Vec<PriceRecord>> {
        let prices: Vec<PriceRecord> = self
            .price_records
            .values()
            .filter(|p| {
                p.product_id.as_ref() == Some(&product_id.to_string())
                    && p.verification_status == "verified"
            })
            .cloned()
            .collect();

        Ok(prices)
    }

    /// Get price records for a store
    pub fn get_store_prices(&self, store_id: &str) -> ServiceResult<Vec<PriceRecord>> {
        let prices: Vec<PriceRecord> = self
            .price_records
            .values()
            .filter(|p| p.store_id == store_id)
            .cloned()
            .collect();

        Ok(prices)
    }

    /// Get price records by user
    pub fn get_user_prices(&self, user_id: &str) -> ServiceResult<Vec<PriceRecord>> {
        let prices: Vec<PriceRecord> = self
            .price_records
            .values()
            .filter(|p| p.user_id.as_ref() == Some(&user_id.to_string()))
            .cloned()
            .collect();

        Ok(prices)
    }

    /// Get current lowest price for a product
    pub fn get_current_lowest_price(&self, product_id: &str) -> ServiceResult<Option<PriceRecord>> {
        let verified_prices = self.get_verified_product_prices(product_id)?;

        let lowest_price = verified_prices.into_iter().min_by(|a, b| {
            a.price
                .partial_cmp(&b.price)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(lowest_price)
    }

    /// Get price comparison across stores for a product
    pub fn get_price_comparison(
        &self,
        product_id: &str,
    ) -> ServiceResult<Vec<StorePriceComparison>> {
        let verified_prices = self.get_verified_product_prices(product_id)?;

        // Group by store and find latest price for each store
        let mut store_prices: HashMap<String, PriceRecord> = HashMap::new();

        for price in verified_prices {
            let store_id = price.store_id.clone();

            if let Some(existing) = store_prices.get(&store_id) {
                if price.timestamp > existing.timestamp {
                    store_prices.insert(store_id, price);
                }
            } else {
                store_prices.insert(store_id, price);
            }
        }

        let comparison: Vec<StorePriceComparison> = store_prices
            .into_iter()
            .map(|(store_id, price_record)| StorePriceComparison {
                store_id,
                price: price_record.price,
                is_on_sale: price_record.is_on_sale,
                timestamp: price_record.timestamp,
            })
            .collect();

        Ok(comparison)
    }

    /// Get price history for a product over time
    pub fn get_price_history(
        &self,
        product_id: &str,
        days: i64,
    ) -> ServiceResult<Vec<PriceHistoryPoint>> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days);

        let verified_prices = self.get_verified_product_prices(product_id)?;

        let history: Vec<PriceHistoryPoint> = verified_prices
            .into_iter()
            .filter(|p| p.timestamp > cutoff_date)
            .map(|p| PriceHistoryPoint {
                price: p.price,
                timestamp: p.timestamp,
                store_id: p.store_id,
                is_on_sale: p.is_on_sale,
            })
            .collect();

        Ok(history)
    }

    /// Calculate price statistics for a product
    pub fn get_price_statistics(&self, product_id: &str) -> ServiceResult<PriceStatistics> {
        let verified_prices = self.get_verified_product_prices(product_id)?;

        if verified_prices.is_empty() {
            return Ok(PriceStatistics {
                min_price: 0.0,
                max_price: 0.0,
                avg_price: 0.0,
                median_price: 0.0,
                total_records: 0,
                stores_count: 0,
                sale_percentage: 0.0,
            });
        }

        let prices: Vec<f64> = verified_prices.iter().map(|p| p.price).collect();
        let min_price = prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_price = prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

        let mut sorted_prices = prices.clone();
        sorted_prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_price = if sorted_prices.len() % 2 == 0 {
            let mid = sorted_prices.len() / 2;
            (sorted_prices[mid - 1] + sorted_prices[mid]) / 2.0
        } else {
            sorted_prices[sorted_prices.len() / 2]
        };

        let unique_stores: std::collections::HashSet<String> =
            verified_prices.iter().map(|p| p.store_id.clone()).collect();

        let sale_count = verified_prices.iter().filter(|p| p.is_on_sale).count();
        let sale_percentage = (sale_count as f64 / verified_prices.len() as f64) * 100.0;

        Ok(PriceStatistics {
            min_price,
            max_price,
            avg_price,
            median_price,
            total_records: verified_prices.len(),
            stores_count: unique_stores.len(),
            sale_percentage,
        })
    }

    /// Get trending prices (products with recent price changes)
    pub fn get_trending_prices(&self, limit: usize) -> ServiceResult<Vec<TrendingPrice>> {
        let recent_cutoff = Utc::now() - chrono::Duration::hours(24);

        // Group by product and count recent price updates
        let mut product_activity: HashMap<String, usize> = HashMap::new();
        let mut product_latest_price: HashMap<String, PriceRecord> = HashMap::new();

        for price in self.price_records.values() {
            if let Some(ref product_id) = price.product_id {
                if price.timestamp > recent_cutoff && price.verification_status == "verified" {
                    *product_activity.entry(product_id.clone()).or_insert(0) += 1;

                    if let Some(existing) = product_latest_price.get(product_id) {
                        if price.timestamp > existing.timestamp {
                            product_latest_price.insert(product_id.clone(), price.clone());
                        }
                    } else {
                        product_latest_price.insert(product_id.clone(), price.clone());
                    }
                }
            }
        }

        let mut trending: Vec<TrendingPrice> = product_activity
            .into_iter()
            .filter_map(|(product_id, activity_count)| {
                product_latest_price
                    .get(&product_id)
                    .map(|price| TrendingPrice {
                        product_id: product_id.clone(),
                        latest_price: price.price,
                        activity_count,
                        timestamp: price.timestamp,
                    })
            })
            .collect();

        trending.sort_by(|a, b| b.activity_count.cmp(&a.activity_count));
        trending.truncate(limit);

        Ok(trending)
    }

    /// Get price alerts that should trigger
    pub fn check_price_alerts(
        &self,
        target_prices: &[(String, f64)],
    ) -> ServiceResult<Vec<PriceAlert>> {
        let mut triggered_alerts = Vec::new();

        for (product_id, target_price) in target_prices {
            if let Ok(Some(current_lowest)) = self.get_current_lowest_price(product_id) {
                if current_lowest.price <= *target_price {
                    triggered_alerts.push(PriceAlert {
                        product_id: product_id.clone(),
                        target_price: *target_price,
                        current_price: current_lowest.price,
                        store_id: current_lowest.store_id,
                        timestamp: current_lowest.timestamp,
                    });
                }
            }
        }

        Ok(triggered_alerts)
    }

    /// Get price submission statistics
    pub fn get_submission_stats(&self) -> ServiceResult<SubmissionStats> {
        let total_submissions = self.price_records.len();

        let verified_count = self
            .price_records
            .values()
            .filter(|p| p.verification_status == "verified")
            .count();

        let pending_count = self
            .price_records
            .values()
            .filter(|p| p.verification_status == "pending")
            .count();

        let rejected_count = self
            .price_records
            .values()
            .filter(|p| p.verification_status == "rejected")
            .count();

        let unique_products: std::collections::HashSet<String> = self
            .price_records
            .values()
            .filter_map(|p| p.product_id.clone())
            .collect();

        let unique_stores: std::collections::HashSet<String> = self
            .price_records
            .values()
            .map(|p| p.store_id.clone())
            .collect();

        Ok(SubmissionStats {
            total_submissions,
            verified_count,
            pending_count,
            rejected_count,
            unique_products: unique_products.len(),
            unique_stores: unique_stores.len(),
        })
    }

    // Helper methods

    fn validate_price_submission(&self, price: f64) -> ServiceResult<()> {
        if price <= 0.0 {
            return Err(ServiceError::ValidationError(
                "Price must be positive".to_string(),
            ));
        }

        if price > 1_000_000.0 {
            return Err(ServiceError::ValidationError("Price too high".to_string()));
        }

        Ok(())
    }
}

impl Default for PriceService {
    fn default() -> Self {
        Self::new()
    }
}

/// Store price comparison entry
#[derive(Debug, Clone)]
pub struct StorePriceComparison {
    pub store_id: String,
    pub price: f64,
    pub is_on_sale: bool,
    pub timestamp: DateTime<Utc>,
}

/// Price history point
#[derive(Debug, Clone)]
pub struct PriceHistoryPoint {
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    pub store_id: String,
    pub is_on_sale: bool,
}

/// Price statistics
#[derive(Debug, Clone)]
pub struct PriceStatistics {
    pub min_price: f64,
    pub max_price: f64,
    pub avg_price: f64,
    pub median_price: f64,
    pub total_records: usize,
    pub stores_count: usize,
    pub sale_percentage: f64,
}

/// Trending price information
#[derive(Debug, Clone)]
pub struct TrendingPrice {
    pub product_id: String,
    pub latest_price: f64,
    pub activity_count: usize,
    pub timestamp: DateTime<Utc>,
}

/// Price alert information
#[derive(Debug, Clone)]
pub struct PriceAlert {
    pub product_id: String,
    pub target_price: f64,
    pub current_price: f64,
    pub store_id: String,
    pub timestamp: DateTime<Utc>,
}

/// Price submission statistics
#[derive(Debug, Clone)]
pub struct SubmissionStats {
    pub total_submissions: usize,
    pub verified_count: usize,
    pub pending_count: usize,
    pub rejected_count: usize,
    pub unique_products: usize,
    pub unique_stores: usize,
}
