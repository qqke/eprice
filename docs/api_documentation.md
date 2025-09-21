# EPrice API Documentation

## Overview

The EPrice application provides a comprehensive price comparison system with the following core APIs:

## Service APIs

### User Service

#### Authentication
```rust
// Create a new user account
async fn create_user(user: &User) -> ServiceResult<User>

// Authenticate user with username/password
async fn authenticate(username: &str, password: &str) -> ServiceResult<User>

// Get user by ID
async fn get_user_by_id(user_id: i32) -> ServiceResult<User>

// Update user reputation score
async fn update_reputation(user_id: i32, new_score: i32) -> ServiceResult<()>
```

#### User Management
```rust
// Get user statistics
async fn get_user_stats() -> ServiceResult<UserStats>

// Find user by username
async fn find_by_username(username: &str) -> ServiceResult<Option<User>>

// Find user by email
async fn find_by_email(email: &str) -> ServiceResult<Option<User>>
```

### Product Service

#### Product Management
```rust
// Create new product
async fn create_product(product: &Product) -> ServiceResult<Product>

// Search products with filters
async fn search_products(
    query: &str,
    category: Option<ProductCategory>,
    brand: Option<String>,
    limit: i32,
    offset: i32
) -> ServiceResult<Vec<Product>>

// Get product by ID
async fn get_product_by_id(product_id: i32) -> ServiceResult<Product>

// Get product by barcode
async fn get_product_by_barcode(barcode: &str) -> ServiceResult<Option<Product>>
```

#### Product Statistics
```rust
// Get product statistics
async fn get_product_stats() -> ServiceResult<ProductStats>

// Get trending products
async fn get_trending_products(limit: i32) -> ServiceResult<Vec<Product>>
```

### Store Service

#### Store Management
```rust
// Create new store
async fn create_store(store: &Store) -> ServiceResult<Store>

// Find stores near location
async fn find_nearby_stores(latitude: f64, longitude: f64, radius_km: f64) -> ServiceResult<Vec<Store>>

// Get store by ID
async fn get_store_by_id(store_id: i32) -> ServiceResult<Store>

// Update store information
async fn update_store(store: &Store) -> ServiceResult<()>
```

#### Store Statistics
```rust
// Get store statistics
async fn get_store_stats() -> ServiceResult<StoreStats>

// Get store verification status
async fn get_verification_status(store_id: i32) -> ServiceResult<bool>
```

### Price Service

#### Price Management
```rust
// Add new price record
async fn add_price_record(price_record: &PriceRecord) -> ServiceResult<PriceRecord>

// Get price history for product
async fn get_price_history(product_id: i32, days: Option<i32>) -> ServiceResult<Vec<PriceRecord>>

// Verify price record
async fn verify_price(price_id: i32, user_id: i32, is_correct: bool) -> ServiceResult<()>

// Get current lowest price for product
async fn get_current_lowest_price(product_id: i32) -> ServiceResult<Option<PriceRecord>>
```

#### Price Analytics
```rust
// Get trending prices
async fn get_trending_prices(limit: i32) -> ServiceResult<Vec<TrendingPrice>>

// Get price statistics for product
async fn get_price_statistics(product_id: i32) -> ServiceResult<PriceStatistics>

// Get price alerts for user
async fn get_price_alerts(user_id: i32) -> ServiceResult<Vec<PriceAlert>>
```

### Review Service

#### Review Management
```rust
// Create new review
async fn create_review(review: &UserReview) -> ServiceResult<UserReview>

// Get reviews for product
async fn get_reviews_for_product(product_id: i32, limit: i32, offset: i32) -> ServiceResult<Vec<UserReview>>

// Get reviews by user
async fn get_reviews_by_user(user_id: i32, limit: i32, offset: i32) -> ServiceResult<Vec<UserReview>>

// Mark review as helpful
async fn mark_helpful(review_id: i32, user_id: i32) -> ServiceResult<()>

// Verify review
async fn verify_review(review_id: i32) -> ServiceResult<()>
```

#### Review Analytics
```rust
// Get recent reviews
async fn get_recent_reviews(limit: i32, offset: i32) -> ServiceResult<Vec<UserReview>>

// Get average rating for product
async fn get_average_rating(product_id: i32) -> ServiceResult<Option<f64>>
```

## Data Models

### User
```rust
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub reputation_score: i32,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Product
```rust
pub struct Product {
    pub id: i32,
    pub name: String,
    pub barcode: Option<String>,
    pub category: ProductCategory,
    pub brand: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum ProductCategory {
    Food,
    Electronics,
    Clothing,
    Home,
    Health,
    Other,
}
```

### Store
```rust
pub struct Store {
    pub id: i32,
    pub name: String,
    pub chain: Option<String>,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub phone: Option<String>,
    pub hours: Option<String>,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### PriceRecord
```rust
pub struct PriceRecord {
    pub id: i32,
    pub product_id: i32,
    pub store_id: i32,
    pub price: i32, // Price in cents
    pub date_recorded: DateTime<Utc>,
    pub user_id: Option<i32>,
    pub verification_status: VerificationStatus,
    pub verification_count: i32,
    pub notes: Option<String>,
}

pub enum VerificationStatus {
    Pending,
    Verified,
    Disputed,
    Rejected,
}
```

### UserReview
```rust
pub struct UserReview {
    pub id: i32,
    pub user_id: i32,
    pub product_id: i32,
    pub store_id: i32,
    pub rating: i32, // 1-5 stars
    pub comment: Option<String>,
    pub is_verified: bool,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### PriceAlert
```rust
pub struct PriceAlert {
    pub id: i32,
    pub user_id: i32,
    pub product_id: i32,
    pub store_id: Option<i32>,
    pub target_price: i32, // Price in cents
    pub is_active: bool,
    pub notification_sent: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## Error Handling

### ServiceError Types
```rust
pub enum ServiceError {
    DatabaseError(String),
    ValidationError(String),
    NotFound(String),
    Unauthorized(String),
    Conflict(String),
    InternalError(String),
}

pub type ServiceResult<T> = Result<T, ServiceError>;
```

### Error Responses
- `DatabaseError`: Database operation failures
- `ValidationError`: Input validation failures
- `NotFound`: Requested resource not found
- `Unauthorized`: Authentication/authorization failures
- `Conflict`: Resource conflicts (e.g., duplicate entries)
- `InternalError`: Unexpected system errors

## Async Operations

### AsyncOperation Types
```rust
pub enum AsyncOperationType {
    PriceUpdate,
    ImageProcessing,
    DataSync,
    Notification,
    Backup,
}

pub struct AsyncOperation {
    pub id: String,
    pub operation_type: AsyncOperationType,
    pub status: OperationStatus,
    pub progress: f32, // 0.0 to 1.0
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum OperationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}
```

### Async Manager API
```rust
// Submit new async operation
pub fn submit_operation(operation: AsyncOperation) -> OperationHandle

// Get operation status
pub fn get_operation_status(operation_id: &str) -> Option<OperationStatus>

// Cancel operation
pub fn cancel_operation(operation_id: &str) -> Result<(), String>

// List operations for user
pub fn list_user_operations(user_id: i32) -> Vec<AsyncOperation>
```

## Search Engine

### Search Types
```rust
pub struct SearchQuery {
    pub text: String,
    pub filters: SearchFilters,
    pub sort_by: SortBy,
    pub limit: usize,
    pub offset: usize,
}

pub struct SearchFilters {
    pub category: Option<ProductCategory>,
    pub brand: Option<String>,
    pub price_range: Option<(i32, i32)>,
    pub store_id: Option<i32>,
    pub location: Option<(f64, f64, f64)>, // lat, lon, radius
}

pub enum SortBy {
    Relevance,
    Price,
    Date,
    Rating,
    Distance,
}
```

### Search API
```rust
// Search products with advanced filtering
pub fn search(query: SearchQuery) -> ServiceResult<SearchResult>

// Get search suggestions
pub fn get_suggestions(partial_query: &str) -> Vec<String>

// Index new product for search
pub fn index_product(product: &Product) -> Result<(), String>
```

## Utilities

### Distance Calculation
```rust
// Calculate distance between two coordinates
pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64
```

### Price Formatting
```rust
// Format price for display
pub fn format_price(price_cents: i32, currency: Currency) -> String

// Parse price from string
pub fn parse_price(price_str: &str) -> Result<i32, String>
```

### Validation
```rust
// Validate email address
pub fn validate_email(email: &str) -> bool

// Validate password strength
pub fn validate_password(password: &str) -> bool

// Validate barcode format
pub fn validate_barcode(barcode: &str) -> bool
```

### Security
```rust
// Hash password with bcrypt
pub fn hash_password(password: &str) -> Result<String, String>

// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String>

// Generate user authentication token
pub fn generate_user_token(user_id: i32) -> String

// Verify user token
pub fn verify_user_token(token: &str, user_id: i32) -> bool
```

## Configuration

### Database Configuration
```rust
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub connection_timeout: Duration,
}
```

### Application Settings
```rust
pub struct AppSettings {
    pub language: Language,
    pub currency: Currency,
    pub distance_unit: DistanceUnit,
    pub enable_notifications: bool,
    pub auto_verify_receipts: bool,
}

pub enum Language {
    Japanese,
    English,
    Chinese,
}

pub enum Currency {
    JPY,
    USD,
    EUR,
}

pub enum DistanceUnit {
    Kilometers,
    Miles,
}
```

## Usage Examples

### Creating a User
```rust
let user = User {
    id: 0, // Will be assigned by database
    username: "john_doe".to_string(),
    email: "john@example.com".to_string(),
    password_hash: hash_password("secure_password")?,
    display_name: Some("John Doe".to_string()),
    avatar_url: None,
    reputation_score: 0,
    is_verified: false,
    created_at: Utc::now(),
    updated_at: Utc::now(),
};

let created_user = user_service.create_user(&user).await?;
```

### Adding a Price Record
```rust
let price_record = PriceRecord {
    id: 0,
    product_id: 123,
    store_id: 456,
    price: 1500, // ¥15.00
    date_recorded: Utc::now(),
    user_id: Some(created_user.id),
    verification_status: VerificationStatus::Pending,
    verification_count: 0,
    notes: Some("Found on sale".to_string()),
};

let created_price = price_service.add_price_record(&price_record).await?;
```

### Searching Products
```rust
let query = SearchQuery {
    text: "iPhone 14".to_string(),
    filters: SearchFilters {
        category: Some(ProductCategory::Electronics),
        brand: Some("Apple".to_string()),
        price_range: Some((100000, 150000)), // ¥1000-¥1500
        store_id: None,
        location: Some((35.6762, 139.6503, 5.0)), // Tokyo, 5km radius
    },
    sort_by: SortBy::Price,
    limit: 20,
    offset: 0,
};

let results = search_engine.search(query)?;
```

## Rate Limiting

API calls are subject to rate limiting:
- Anonymous users: 100 requests/hour
- Authenticated users: 1000 requests/hour
- Premium users: 5000 requests/hour

## Versioning

API version is specified in the service interface. Current version: v1.0.0

Breaking changes will result in a new major version. Backward compatibility is maintained within major versions.

## Performance Considerations

- Database queries are optimized with proper indexing
- Search results are cached for common queries
- Async operations handle long-running tasks
- Connection pooling minimizes database overhead
- Geographic queries use spatial indexing for efficiency