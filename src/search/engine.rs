use crate::models::{PriceRecord, Product, Store};
use crate::search::filters::{SearchFilters, SortDirection, SortField};
use crate::services::ServiceResult;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Advanced search engine for intelligent product and price discovery
pub struct SearchEngine {
    // Search indices for fast lookups
    product_index: HashMap<String, Vec<String>>, // term -> product_ids
    store_index: HashMap<String, Vec<String>>,   // term -> store_ids
    category_index: HashMap<String, Vec<String>>, // category -> product_ids
    tag_index: HashMap<String, Vec<String>>,     // tag -> product_ids

    // Cache for search results
    search_cache: HashMap<String, (SearchResult, DateTime<Utc>)>,
    cache_ttl_minutes: u32,
}

/// Search query with natural language processing
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: String,
    pub filters: SearchFilters,
    pub fuzzy_matching: bool,
    pub include_suggestions: bool,
    pub max_results: usize,
    pub min_relevance_score: f32,
}

/// Comprehensive search results with metadata
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub items: Vec<SearchResultItem>,
    pub total_count: usize,
    pub search_time_ms: u64,
    pub suggestions: Vec<String>,
    pub filters_applied: SearchFilters,
    pub facets: SearchFacets,
}

/// Individual search result item with relevance scoring
#[derive(Debug, Clone)]
pub struct SearchResultItem {
    pub product: Product,
    pub best_price: Option<PriceRecord>,
    pub store_info: Option<Store>,
    pub relevance_score: f32,
    pub match_reasons: Vec<MatchReason>,
    pub price_trend: PriceTrend,
    pub availability_info: AvailabilityInfo,
}

/// Explanation of why item matched the search
#[derive(Debug, Clone)]
pub enum MatchReason {
    NameMatch(f32),        // score
    CategoryMatch(f32),    // score
    TagMatch(String, f32), // tag, score
    DescriptionMatch(f32), // score
    BarcodeMatch,
    StoreMatch(String), // store name
}

/// Price trend information for search results
#[derive(Debug, Clone)]
pub struct PriceTrend {
    pub current_price: f64,
    pub price_change_24h: Option<f64>,
    pub price_change_7d: Option<f64>,
    pub lowest_price_30d: Option<f64>,
    pub is_trending_up: bool,
    pub confidence: f32,
}

/// Availability information
#[derive(Debug, Clone)]
pub struct AvailabilityInfo {
    pub in_stock: bool,
    pub stock_level: StockLevel,
    pub last_seen: DateTime<Utc>,
    pub store_count: usize,
}

#[derive(Debug, Clone)]
pub enum StockLevel {
    High,
    Medium,
    Low,
    OutOfStock,
    Unknown,
}

/// Search facets for filtering UI
#[derive(Debug, Clone)]
pub struct SearchFacets {
    pub categories: Vec<FacetItem>,
    pub stores: Vec<FacetItem>,
    pub price_ranges: Vec<PriceRangeFacet>,
    pub brands: Vec<FacetItem>,
    pub tags: Vec<FacetItem>,
}

#[derive(Debug, Clone)]
pub struct FacetItem {
    pub name: String,
    pub count: usize,
    pub selected: bool,
}

#[derive(Debug, Clone)]
pub struct PriceRangeFacet {
    pub min_price: f64,
    pub max_price: f64,
    pub count: usize,
    pub label: String,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            product_index: HashMap::new(),
            store_index: HashMap::new(),
            category_index: HashMap::new(),
            tag_index: HashMap::new(),
            search_cache: HashMap::new(),
            cache_ttl_minutes: 15,
        }
    }

    /// Build search indices from data
    pub fn build_indices(&mut self, products: &[Product], stores: &[Store]) -> ServiceResult<()> {
        self.clear_indices();

        // Build product index
        for product in products {
            self.index_product(product)?;
        }

        // Build store index
        for store in stores {
            self.index_store(store)?;
        }

        Ok(())
    }

    /// Perform advanced search with intelligent ranking
    pub fn search(&mut self, query: SearchQuery) -> ServiceResult<SearchResult> {
        let start_time = std::time::Instant::now();

        // Check cache first
        let cache_key = self.generate_cache_key(&query);
        if let Some((cached_result, cached_time)) = self.search_cache.get(&cache_key) {
            if self.is_cache_valid(cached_time) {
                return Ok(cached_result.clone());
            }
        }

        // Perform search
        let mut items = Vec::new();
        let query_terms = self.tokenize_query(&query.text);

        // Find matching products
        let matching_products = self.find_matching_products(&query_terms, &query.filters)?;

        // Score and rank results
        for (product, base_score) in matching_products {
            if let Some(result_item) =
                self.create_search_result_item(product, base_score, &query)?
            {
                if result_item.relevance_score >= query.min_relevance_score {
                    items.push(result_item);
                }
            }
        }

        // Sort results
        self.sort_results(&mut items, &query.filters.sort_options);

        // Limit results
        items.truncate(query.max_results);

        // Generate suggestions
        let suggestions = if query.include_suggestions {
            self.generate_suggestions(&query.text, &items)
        } else {
            Vec::new()
        };

        // Generate facets
        let facets = self.generate_facets(&items);

        let result = SearchResult {
            total_count: items.len(),
            items,
            search_time_ms: start_time.elapsed().as_millis() as u64,
            suggestions,
            filters_applied: query.filters.clone(),
            facets,
        };

        // Cache result
        self.search_cache
            .insert(cache_key, (result.clone(), Utc::now()));

        Ok(result)
    }

    /// Auto-complete suggestions
    pub fn get_suggestions(&self, partial_query: &str, limit: usize) -> Vec<String> {
        let partial_lower = partial_query.to_lowercase();
        let mut suggestions = Vec::new();

        // Search in product names
        for term in self.product_index.keys() {
            if term.to_lowercase().starts_with(&partial_lower) {
                suggestions.push(term.clone());
            }
        }

        // Search in categories
        for category in self.category_index.keys() {
            if category.to_lowercase().starts_with(&partial_lower) {
                suggestions.push(category.clone());
            }
        }

        suggestions.sort();
        suggestions.dedup();
        suggestions.truncate(limit);
        suggestions
    }

    /// Get trending searches
    pub fn get_trending_searches(&self, limit: usize) -> Vec<String> {
        // In a real implementation, this would track search frequency
        vec![
            "可口可乐".to_string(),
            "iPhone".to_string(),
            "薯片".to_string(),
            "洗发水".to_string(),
            "牛奶".to_string(),
        ]
        .into_iter()
        .take(limit)
        .collect()
    }

    // Private helper methods

    fn clear_indices(&mut self) {
        self.product_index.clear();
        self.store_index.clear();
        self.category_index.clear();
        self.tag_index.clear();
    }

    fn index_product(&mut self, product: &Product) -> ServiceResult<()> {
        // Index product name
        let name_terms = self.tokenize(&product.name);
        for term in name_terms {
            self.product_index
                .entry(term)
                .or_default()
                .push(product.id.clone());
        }

        // Index category
        let category_terms = self.tokenize(&product.category);
        for term in category_terms {
            self.category_index
                .entry(term)
                .or_default()
                .push(product.id.clone());
        }

        // Index tags
        for tag in &product.tags {
            self.tag_index
                .entry(tag.clone())
                .or_default()
                .push(product.id.clone());
        }

        // Index description
        let desc_terms = self.tokenize(&product.description);
        for term in desc_terms {
            self.product_index
                .entry(term)
                .or_default()
                .push(product.id.clone());
        }

        Ok(())
    }

    fn index_store(&mut self, store: &Store) -> ServiceResult<()> {
        let name_terms = self.tokenize(&store.name);
        for term in name_terms {
            self.store_index
                .entry(term)
                .or_default()
                .push(store.id.clone());
        }

        Ok(())
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .filter(|s| s.len() > 1)
            .map(|s| s.to_string())
            .collect()
    }

    fn tokenize_query(&self, query: &str) -> Vec<String> {
        self.tokenize(query)
    }

    fn find_matching_products(
        &self,
        query_terms: &[String],
        _filters: &SearchFilters,
    ) -> ServiceResult<Vec<(Product, f32)>> {
        // This is a simplified implementation
        // In a real system, this would use more sophisticated matching
        let mut product_scores: HashMap<String, f32> = HashMap::new();

        // Score products based on query terms
        for term in query_terms {
            if let Some(product_ids) = self.product_index.get(term) {
                for product_id in product_ids {
                    *product_scores.entry(product_id.clone()).or_insert(0.0) += 1.0;
                }
            }
        }

        // Convert to products (this would use actual product service in real implementation)
        let products: Vec<(Product, f32)> = product_scores
            .into_iter()
            .filter_map(|(id, score)| {
                // This is placeholder - would fetch actual products
                Some((self.create_mock_product(&id), score))
            })
            .collect();

        Ok(products)
    }

    fn create_search_result_item(
        &self,
        product: Product,
        base_score: f32,
        _query: &SearchQuery,
    ) -> ServiceResult<Option<SearchResultItem>> {
        // Create match reasons
        let match_reasons = vec![MatchReason::NameMatch(base_score)];

        // Create price trend (mock data)
        let price_trend = PriceTrend {
            current_price: 10.0,
            price_change_24h: Some(-0.5),
            price_change_7d: Some(-2.0),
            lowest_price_30d: Some(8.5),
            is_trending_up: false,
            confidence: 0.85,
        };

        // Create availability info
        let availability_info = AvailabilityInfo {
            in_stock: true,
            stock_level: StockLevel::High,
            last_seen: Utc::now(),
            store_count: 5,
        };

        Ok(Some(SearchResultItem {
            product,
            best_price: None,
            store_info: None,
            relevance_score: base_score,
            match_reasons,
            price_trend,
            availability_info,
        }))
    }

    fn sort_results(
        &self,
        items: &mut [SearchResultItem],
        sort_options: &crate::search::filters::SortOptions,
    ) {
        items.sort_by(|a, b| {
            use std::cmp::Ordering;

            let primary_cmp = match sort_options.primary_sort {
                SortField::Relevance => a
                    .relevance_score
                    .partial_cmp(&b.relevance_score)
                    .unwrap_or(Ordering::Equal),
                SortField::Price => a
                    .price_trend
                    .current_price
                    .partial_cmp(&b.price_trend.current_price)
                    .unwrap_or(Ordering::Equal),
                SortField::Name => a.product.name.cmp(&b.product.name),
                _ => Ordering::Equal,
            };

            match sort_options.sort_direction {
                SortDirection::Ascending => primary_cmp,
                SortDirection::Descending => primary_cmp.reverse(),
            }
        });
    }

    fn generate_suggestions(&self, query: &str, results: &[SearchResultItem]) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Add category suggestions based on results
        for item in results.iter().take(3) {
            suggestions.push(format!("{} in {}", query, item.product.category));
        }

        suggestions.truncate(5);
        suggestions
    }

    fn generate_facets(&self, results: &[SearchResultItem]) -> SearchFacets {
        let mut categories = HashMap::new();
        let stores = HashMap::new();

        for item in results {
            *categories.entry(item.product.category.clone()).or_insert(0) += 1;
        }

        SearchFacets {
            categories: categories
                .into_iter()
                .map(|(name, count)| FacetItem {
                    name,
                    count,
                    selected: false,
                })
                .collect(),
            stores: stores
                .into_iter()
                .map(|(name, count)| FacetItem {
                    name,
                    count,
                    selected: false,
                })
                .collect(),
            price_ranges: vec![PriceRangeFacet {
                min_price: 0.0,
                max_price: 10.0,
                count: 5,
                label: "¥0 - ¥10".to_string(),
            }],
            brands: Vec::new(),
            tags: Vec::new(),
        }
    }

    fn generate_cache_key(&self, query: &SearchQuery) -> String {
        format!("{}_{}", query.text, query.max_results)
    }

    fn is_cache_valid(&self, cached_time: &DateTime<Utc>) -> bool {
        let now = Utc::now();
        let duration = now.signed_duration_since(*cached_time);
        duration.num_minutes() < self.cache_ttl_minutes as i64
    }

    fn create_mock_product(&self, id: &str) -> Product {
        Product {
            id: id.to_string(),
            name: format!("Product {}", id),
            category: "Test Category".to_string(),
            description: "Test product".to_string(),
            barcode: None,
            images: Vec::new(),
            prices: Vec::new(),
            tags: Vec::new(),
            created_at: Utc::now(),
        }
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: String::new(),
            filters: SearchFilters::default(),
            fuzzy_matching: true,
            include_suggestions: true,
            max_results: 50,
            min_relevance_score: 0.1,
        }
    }
}
