use crate::async_ops::progress::ProgressTracker;
use crate::models::{PriceRecord, Product, Store, User};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Represents different types of async operations in the application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperationType {
    // Data operations
    DataSync,
    DataBackup,
    DataImport,
    DataExport,

    // Price operations
    PriceMonitoring,
    PriceAnalysis,
    PriceComparison,
    PriceHistoryUpdate,

    // Search operations
    ProductSearch,
    StoreSearch,
    AdvancedSearch,
    IndexBuilding,

    // Camera operations
    CameraCapture,
    BarcodeScanning,
    ImageProcessing,
    OCRProcessing,

    // Network operations
    ApiRequest,
    FileDownload,
    FileUpload,
    CloudSync,

    // Database operations
    DatabaseQuery,
    DatabaseUpdate,
    DatabaseMigration,
    DatabaseOptimization,

    // Notification operations
    NotificationSend,
    AlertProcessing,
    EmailSend,
    PushNotification,
}

/// Result of an async operation
#[derive(Debug, Clone)]
pub enum OperationResult {
    Success(OperationData),
    Failure(OperationError),
    Cancelled,
}

/// Data returned by successful operations
#[derive(Debug, Clone)]
pub enum OperationData {
    Products(Vec<Product>),
    Stores(Vec<Store>),
    PriceRecords(Vec<PriceRecord>),
    Users(Vec<User>),
    Text(String),
    Bytes(Vec<u8>),
    Json(serde_json::Value),
    Count(usize),
    Bool(bool),
    None,
}

/// Errors that can occur during async operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationError {
    NetworkError(String),
    DatabaseError(String),
    PermissionDenied(String),
    InvalidInput(String),
    ProcessingError(String),
    TimeoutError(String),
    CancellationError(String),
    ResourceUnavailable(String),
    InternalError(String),
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            OperationError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            OperationError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            OperationError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            OperationError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
            OperationError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            OperationError::CancellationError(msg) => write!(f, "Cancellation error: {}", msg),
            OperationError::ResourceUnavailable(msg) => write!(f, "Resource unavailable: {}", msg),
            OperationError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for OperationError {}

/// Represents an async operation with metadata and progress tracking
#[derive(Clone)]
pub struct AsyncOperation {
    pub id: String,
    pub operation_type: OperationType,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress_tracker: ProgressTracker,
    pub dependencies: Vec<String>, // IDs of operations this depends on
    pub priority: OperationPriority,
    pub retry_count: u32,
    pub max_retries: u32,
    pub timeout_seconds: Option<u64>,
    pub cancellable: bool,
    pub metadata: OperationMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum OperationPriority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OperationMetadata {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub tags: Vec<String>,
    pub context: std::collections::HashMap<String, String>,
}

impl AsyncOperation {
    /// Create a new async operation
    pub fn new(
        operation_type: OperationType,
        description: String,
        priority: OperationPriority,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            operation_type,
            description,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            progress_tracker: ProgressTracker::new(),
            dependencies: Vec::new(),
            priority,
            retry_count: 0,
            max_retries: 3,
            timeout_seconds: Some(300), // 5 minutes default
            cancellable: true,
            metadata: OperationMetadata::default(),
        }
    }

    /// Create a data sync operation
    pub fn data_sync(description: String) -> Self {
        Self::new(
            OperationType::DataSync,
            description,
            OperationPriority::Normal,
        )
    }

    /// Create a price monitoring operation
    pub fn price_monitoring(description: String) -> Self {
        Self::new(
            OperationType::PriceMonitoring,
            description,
            OperationPriority::High,
        )
    }

    /// Create a search operation
    pub fn search_operation(description: String) -> Self {
        Self::new(
            OperationType::ProductSearch,
            description,
            OperationPriority::Normal,
        )
    }

    /// Create a camera capture operation
    pub fn camera_capture(description: String) -> Self {
        Self::new(
            OperationType::CameraCapture,
            description,
            OperationPriority::High,
        )
    }

    /// Create a database operation
    pub fn database_operation(description: String) -> Self {
        Self::new(
            OperationType::DatabaseQuery,
            description,
            OperationPriority::Normal,
        )
    }

    /// Add a dependency to this operation
    pub fn add_dependency(&mut self, operation_id: String) {
        if !self.dependencies.contains(&operation_id) {
            self.dependencies.push(operation_id);
        }
    }

    /// Set metadata for the operation
    pub fn with_metadata(mut self, metadata: OperationMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set timeout for the operation
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }

    /// Set retry configuration
    pub fn with_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Mark operation as non-cancellable
    pub fn non_cancellable(mut self) -> Self {
        self.cancellable = false;
        self
    }

    /// Add tags to the operation
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.metadata.tags = tags;
        self
    }

    /// Add context information
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.metadata.context.insert(key, value);
        self
    }

    /// Check if operation has timed out
    pub fn is_timed_out(&self) -> bool {
        if let (Some(started), Some(timeout)) = (self.started_at, self.timeout_seconds) {
            let elapsed = Utc::now().signed_duration_since(started);
            elapsed.num_seconds() > timeout as i64
        } else {
            false
        }
    }

    /// Check if operation can be retried
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Mark operation as started
    pub fn mark_started(&mut self) {
        self.started_at = Some(Utc::now());
    }

    /// Mark operation as completed
    pub fn mark_completed(&mut self) {
        self.completed_at = Some(Utc::now());
        self.progress_tracker.complete();
    }

    /// Get operation duration in seconds
    pub fn duration_seconds(&self) -> Option<i64> {
        if let (Some(started), Some(completed)) = (self.started_at, self.completed_at) {
            Some(completed.signed_duration_since(started).num_seconds())
        } else {
            self.started_at
                .map(|started| Utc::now().signed_duration_since(started).num_seconds())
        }
    }

    /// Check if dependencies are satisfied
    pub fn dependencies_satisfied(&self, completed_operations: &[String]) -> bool {
        self.dependencies
            .iter()
            .all(|dep| completed_operations.contains(dep))
    }

    /// Get human-readable status
    pub fn status_text(&self) -> String {
        if self.completed_at.is_some() {
            "Completed".to_string()
        } else if self.started_at.is_some() {
            format!(
                "Running ({}%)",
                (self.progress_tracker.progress() * 100.0) as u32
            )
        } else {
            "Pending".to_string()
        }
    }

    /// Estimate remaining time based on progress
    pub fn estimated_remaining_seconds(&self) -> Option<u64> {
        if let Some(started) = self.started_at {
            let elapsed = Utc::now().signed_duration_since(started).num_seconds() as f64;
            let progress = self.progress_tracker.progress();

            if progress > 0.0 && progress < 1.0 {
                let total_estimated = elapsed / progress as f64;
                let remaining = total_estimated - elapsed;
                Some(remaining.max(0.0) as u64)
            } else {
                None
            }
        } else {
            None
        }
    }
}

// (removed duplicate OperationMetadata and OperationPriority definitions)

/// Factory for creating common operation types
pub struct OperationFactory;

impl OperationFactory {
    /// Create a batch data sync operation
    pub fn create_data_sync_batch(items: Vec<String>) -> Vec<AsyncOperation> {
        items
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                AsyncOperation::data_sync(format!("Sync {}", item))
                    .with_context("batch_index".to_string(), i.to_string())
                    .with_tags(vec!["batch".to_string(), "sync".to_string()])
            })
            .collect()
    }

    /// Create a price monitoring chain
    pub fn create_price_monitoring_chain(product_ids: Vec<String>) -> Vec<AsyncOperation> {
        let mut operations = Vec::new();

        // First, fetch current prices
        let fetch_op = AsyncOperation::new(
            OperationType::PriceComparison,
            "Fetch current prices".to_string(),
            OperationPriority::High,
        )
        .with_tags(vec!["price".to_string(), "fetch".to_string()]);

        operations.push(fetch_op.clone());

        // Then analyze each product
        for product_id in product_ids {
            let mut analysis_op = AsyncOperation::new(
                OperationType::PriceAnalysis,
                format!("Analyze price for product {}", product_id),
                OperationPriority::Normal,
            );
            analysis_op.add_dependency(fetch_op.id.clone());
            analysis_op = analysis_op.with_context("product_id".to_string(), product_id);

            operations.push(analysis_op);
        }

        operations
    }

    /// Create a search indexing operation
    pub fn create_search_indexing() -> AsyncOperation {
        AsyncOperation::new(
            OperationType::IndexBuilding,
            "Rebuild search index".to_string(),
            OperationPriority::Low,
        )
        .with_timeout(1800) // 30 minutes
        .with_retries(1)
        .with_tags(vec!["search".to_string(), "index".to_string()])
    }

    /// Create a backup operation
    pub fn create_backup_operation(backup_type: &str) -> AsyncOperation {
        AsyncOperation::new(
            OperationType::DataBackup,
            format!("Create {} backup", backup_type),
            OperationPriority::Low,
        )
        .with_timeout(3600) // 1 hour
        .with_retries(2)
        .non_cancellable()
        .with_context("backup_type".to_string(), backup_type.to_string())
    }
}
