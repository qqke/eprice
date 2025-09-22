#![warn(clippy::all, rust_2018_idioms)]

pub mod alerts;
pub mod app;
pub mod async_ops;
pub mod auth;
#[cfg(not(target_arch = "wasm32"))]
pub mod database;
pub mod error;
pub mod models;
pub mod ocr;
pub mod search;
pub mod services;
pub mod settings;
pub mod utils;
pub mod verification;

// Scanner module is only available for native targets (not WASM)
#[cfg(not(target_arch = "wasm32"))]
pub mod scanner;

pub use app::TemplateApp;

// Re-export commonly used types
pub use error::{AppError, AppResult, ContextualError, ErrorHandler};
pub use models::{PriceRecord, Product, Store};
// pub use auth::{User, AuthManager}; // Disabled for now
// pub use database::Database; // Disabled for now
