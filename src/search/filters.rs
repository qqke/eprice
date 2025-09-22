use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Advanced search filters for product and price discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub price_range: Option<PriceRange>,
    pub categories: Vec<CategoryFilter>,
    pub stores: Vec<StoreFilter>,
    pub tags: Vec<String>,
    pub availability: AvailabilityFilter,
    pub time_range: Option<TimeRange>,
    pub rating_filter: Option<RatingFilter>,
    pub promotion_filter: PromotionFilter,
    pub verification_status: VerificationFilter,
    pub sort_options: SortOptions,
}

/// Price range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRange {
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub include_sale_prices: bool,
    pub currency: String,
}

/// Category filter with hierarchy support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryFilter {
    pub category_id: String,
    pub category_name: String,
    pub include_subcategories: bool,
    pub weight: f32, // For relevance scoring
}

/// Store filter with location and preference support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreFilter {
    pub store_id: String,
    pub store_name: String,
    pub max_distance_km: Option<f64>,
    pub preferred: bool,
    pub minimum_rating: Option<f64>,
}

/// Availability filter options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AvailabilityFilter {
    All,
    InStock,
    OutOfStock,
    LimitedStock,
    PreOrder,
}

/// Time range for price history and updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub last_updated_hours: Option<u32>,
}

/// Rating and review filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingFilter {
    pub min_rating: f64,
    pub min_review_count: Option<u32>,
    pub verified_reviews_only: bool,
}

/// Promotion and discount filter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PromotionFilter {
    All,
    OnSale,
    HasCoupon,
    BuyOneGetOne,
    BulkDiscount,
    NoPromotions,
}

/// Verification status filter for price records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationFilter {
    All,
    Verified,
    Pending,
    Rejected,
}

/// Sorting options for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOptions {
    pub primary_sort: SortField,
    pub secondary_sort: Option<SortField>,
    pub sort_direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortField {
    Relevance,
    Price,
    Name,
    Rating,
    Distance,
    LastUpdated,
    PopularityScore,
    PriceChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            price_range: None,
            categories: Vec::new(),
            stores: Vec::new(),
            tags: Vec::new(),
            availability: AvailabilityFilter::All,
            time_range: None,
            rating_filter: None,
            promotion_filter: PromotionFilter::All,
            verification_status: VerificationFilter::Verified,
            sort_options: SortOptions {
                primary_sort: SortField::Relevance,
                secondary_sort: Some(SortField::Price),
                sort_direction: SortDirection::Descending,
            },
        }
    }
}

impl Default for PriceRange {
    fn default() -> Self {
        Self {
            min_price: None,
            max_price: None,
            include_sale_prices: true,
            currency: "CNY".to_string(),
        }
    }
}

impl SearchFilters {
    /// Create a new filter set with basic price range
    pub fn with_price_range(min_price: Option<f64>, max_price: Option<f64>) -> Self {
        Self {
            price_range: Some(PriceRange {
                min_price,
                max_price,
                include_sale_prices: true,
                currency: "CNY".to_string(),
            }),
            ..Self::default()
        }
    }

    /// Add category filter
    pub fn add_category(
        &mut self,
        category_id: String,
        category_name: String,
        include_subcategories: bool,
    ) {
        self.categories.push(CategoryFilter {
            category_id,
            category_name,
            include_subcategories,
            weight: 1.0,
        });
    }

    /// Add store filter
    pub fn add_store(&mut self, store_id: String, store_name: String, max_distance: Option<f64>) {
        self.stores.push(StoreFilter {
            store_id,
            store_name,
            max_distance_km: max_distance,
            preferred: false,
            minimum_rating: None,
        });
    }

    /// Set availability filter
    pub fn set_availability(&mut self, availability: AvailabilityFilter) {
        self.availability = availability;
    }

    /// Set promotion filter
    pub fn set_promotion_filter(&mut self, promotion: PromotionFilter) {
        self.promotion_filter = promotion;
    }

    /// Set sort options
    pub fn set_sort(&mut self, field: SortField, direction: SortDirection) {
        self.sort_options.primary_sort = field;
        self.sort_options.sort_direction = direction;
    }

    /// Add tags for filtering
    pub fn add_tags(&mut self, tags: Vec<String>) {
        self.tags.extend(tags);
    }

    /// Check if filters are empty (no filtering applied)
    pub fn is_empty(&self) -> bool {
        self.price_range.is_none()
            && self.categories.is_empty()
            && self.stores.is_empty()
            && self.tags.is_empty()
            && matches!(self.availability, AvailabilityFilter::All)
            && self.time_range.is_none()
            && self.rating_filter.is_none()
            && matches!(self.promotion_filter, PromotionFilter::All)
    }

    /// Reset all filters to default
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Validate filter consistency
    pub fn validate(&self) -> Result<(), String> {
        // Validate price range
        if let Some(ref price_range) = self.price_range {
            if let (Some(min), Some(max)) = (price_range.min_price, price_range.max_price) {
                if min > max {
                    return Err("Minimum price cannot be greater than maximum price".to_string());
                }
                if min < 0.0 || max < 0.0 {
                    return Err("Prices cannot be negative".to_string());
                }
            }
        }

        // Validate rating filter
        if let Some(ref rating) = self.rating_filter {
            if rating.min_rating < 0.0 || rating.min_rating > 5.0 {
                return Err("Rating must be between 0 and 5".to_string());
            }
        }

        // Validate time range
        if let Some(ref time_range) = self.time_range {
            if let (Some(start), Some(end)) = (time_range.start_date, time_range.end_date) {
                if start > end {
                    return Err("Start date cannot be after end date".to_string());
                }
            }
        }

        Ok(())
    }
}
