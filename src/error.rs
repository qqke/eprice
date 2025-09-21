use anyhow::Result;
use std::fmt;
use thiserror::Error;

/// Centralized error types for the eprice application
#[derive(Error, Debug, Clone)]
pub enum AppError {
    /// Authentication related errors
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Alert system errors
    #[error("Alert error: {0}")]
    Alert(String),

    /// Database related errors
    #[error("Database error: {0}")]
    Database(String),

    /// Service layer errors
    #[error("Service error: {0}")]
    Service(String),

    /// File system errors
    #[error("File system error: {0}")]
    FileSystem(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Network/HTTP errors
    #[error("Network error: {0}")]
    Network(String),

    /// OCR processing errors
    #[error("OCR error: {0}")]
    Ocr(String),

    /// Camera/Scanner errors
    #[error("Scanner error: {0}")]
    Scanner(String),

    /// Serialization/Deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Generic internal errors
    #[error("Internal error: {0}")]
    Internal(String),

    /// Business logic errors
    #[error("Business logic error: {0}")]
    BusinessLogic(String),

    /// External service errors
    #[error("External service error: {0}")]
    ExternalService(String),
}

/// Conversion implementations for error types
impl From<crate::auth::AuthError> for AppError {
    fn from(error: crate::auth::AuthError) -> Self {
        AppError::Authentication(error.to_string())
    }
}

impl From<crate::alerts::AlertError> for AppError {
    fn from(error: crate::alerts::AlertError) -> Self {
        AppError::Alert(error.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::Database(error.to_string())
    }
}

impl From<crate::services::ServiceError> for AppError {
    fn from(error: crate::services::ServiceError) -> Self {
        AppError::Service(error.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::FileSystem(error.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::Serialization(error.to_string())
    }
}

/// Result type using AppError
pub type AppResult<T> = Result<T, AppError>;

/// Error context for better error tracking
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub user_id: Option<String>,
    pub request_id: Option<String>,
    pub additional_info: Option<String>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            user_id: None,
            request_id: None,
            additional_info: None,
        }
    }

    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_info(mut self, info: impl Into<String>) -> Self {
        self.additional_info = Some(info.into());
        self
    }
}

/// Enhanced error with context
#[derive(Debug, Clone)]
pub struct ContextualError {
    pub error: AppError,
    pub context: ErrorContext,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ContextualError {
    pub fn new(error: AppError, context: ErrorContext) -> Self {
        Self {
            error,
            context,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Log the error with full context
    pub fn log(&self) {
        log::error!(
            "Error in operation '{}': {} | User: {:?} | Request: {:?} | Info: {:?} | Time: {}",
            self.context.operation,
            self.error,
            self.context.user_id,
            self.context.request_id,
            self.context.additional_info,
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        );
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match &self.error {
            AppError::Authentication(_) => {
                "Authentication failed. Please check your credentials and try again.".to_string()
            }
            AppError::Database(_) => {
                "A database error occurred. Please try again later.".to_string()
            }
            AppError::Validation(msg) => format!("Validation failed: {}", msg),
            AppError::Network(_) => {
                "Network error. Please check your connection and try again.".to_string()
            }
            AppError::FileSystem(_) => {
                "File operation failed. Please check permissions and try again.".to_string()
            }
            AppError::Configuration(_) => {
                "Configuration error. Please contact support.".to_string()
            }
            AppError::Ocr(_) => {
                "OCR processing failed. Please try with a clearer image.".to_string()
            }
            AppError::Scanner(_) => {
                "Scanner error. Please check camera permissions and try again.".to_string()
            }
            AppError::BusinessLogic(msg) => msg.clone(),
            AppError::ExternalService(_) => {
                "External service unavailable. Please try again later.".to_string()
            }
            _ => "An unexpected error occurred. Please try again.".to_string(),
        }
    }
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (in {})", self.error, self.context.operation)
    }
}

impl std::error::Error for ContextualError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Trait for adding context to errors
pub trait ErrorExt<T> {
    fn with_context(self, context: ErrorContext) -> Result<T, ContextualError>;
    fn with_operation(self, operation: impl Into<String>) -> Result<T, ContextualError>;
}

impl<T, E> ErrorExt<T> for Result<T, E>
where
    E: Into<AppError>,
{
    fn with_context(self, context: ErrorContext) -> Result<T, ContextualError> {
        self.map_err(|e| ContextualError::new(e.into(), context))
    }

    fn with_operation(self, operation: impl Into<String>) -> Result<T, ContextualError> {
        self.with_context(ErrorContext::new(operation))
    }
}

/// Error handler for UI components
pub struct ErrorHandler {
    current_error: Option<ContextualError>,
    error_history: Vec<ContextualError>,
    max_history_size: usize,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self {
            current_error: None,
            error_history: Vec::new(),
            max_history_size: 50,
        }
    }

    /// Handle a new error
    pub fn handle_error(&mut self, error: ContextualError) {
        error.log();

        // Add to history
        self.error_history.push(error.clone());
        if self.error_history.len() > self.max_history_size {
            self.error_history.remove(0);
        }

        // Set as current error
        self.current_error = Some(error);
    }

    /// Get the current error
    pub fn current_error(&self) -> Option<&ContextualError> {
        self.current_error.as_ref()
    }

    /// Get user-friendly message for current error
    pub fn current_error_message(&self) -> Option<String> {
        self.current_error.as_ref().map(|e| e.user_message())
    }

    /// Clear the current error
    pub fn clear_current_error(&mut self) {
        self.current_error = None;
    }

    /// Get error history
    pub fn error_history(&self) -> &[ContextualError] {
        &self.error_history
    }

    /// Check if there are any errors
    pub fn has_error(&self) -> bool {
        self.current_error.is_some()
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience macros for error handling
#[macro_export]
macro_rules! app_error {
    ($kind:ident, $msg:expr) => {
        $crate::error::AppError::$kind($msg.to_string())
    };
    ($kind:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::error::AppError::$kind(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! with_context {
    ($result:expr, $op:expr) => {
        $result.with_operation($op)
    };
    ($result:expr, $op:expr, $user:expr) => {
        $result.with_context($crate::error::ErrorContext::new($op).with_user($user))
    };
    ($result:expr, $op:expr, $user:expr, $info:expr) => {
        $result.with_context(
            $crate::error::ErrorContext::new($op)
                .with_user($user)
                .with_info($info),
        )
    };
}

/// Recovery strategies for different error types
pub enum RecoveryStrategy {
    Retry,
    RetryWithDelay(std::time::Duration),
    Fallback,
    UserIntervention,
    Ignore,
}

impl AppError {
    /// Get the recommended recovery strategy for this error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            AppError::Network(_) => {
                RecoveryStrategy::RetryWithDelay(std::time::Duration::from_secs(2))
            }
            AppError::Database(_) => {
                RecoveryStrategy::RetryWithDelay(std::time::Duration::from_secs(1))
            }
            AppError::Authentication(_) => RecoveryStrategy::UserIntervention,
            AppError::Validation(_) => RecoveryStrategy::UserIntervention,
            AppError::Configuration(_) => RecoveryStrategy::UserIntervention,
            AppError::FileSystem(_) => RecoveryStrategy::Retry,
            AppError::ExternalService(_) => {
                RecoveryStrategy::RetryWithDelay(std::time::Duration::from_secs(5))
            }
            _ => RecoveryStrategy::UserIntervention,
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self.recovery_strategy(),
            RecoveryStrategy::Retry
                | RecoveryStrategy::RetryWithDelay(_)
                | RecoveryStrategy::Fallback
        )
    }
}
