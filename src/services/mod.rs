pub mod price_service;
pub mod product_service;
pub mod review_service;
pub mod store_service;
pub mod user_service;

pub use price_service::PriceService;
pub use product_service::ProductService;
pub use review_service::ReviewService;
pub use store_service::StoreService;
pub use user_service::UserService;

use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),
    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

pub type ServiceResult<T> = Result<T, ServiceError>;

/// Application services aggregator
pub struct AppServices {
    pub user_service: UserService,
    pub product_service: ProductService,
    pub store_service: StoreService,
    pub price_service: PriceService,
    pub review_service: ReviewService,
}

impl AppServices {
    pub fn new() -> Self {
        Self {
            user_service: UserService::new(),
            product_service: ProductService::new(),
            store_service: StoreService::new(),
            price_service: PriceService::new(),
            review_service: ReviewService::new(),
        }
    }
}

impl Default for AppServices {
    fn default() -> Self {
        Self::new()
    }
}
