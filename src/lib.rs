#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
pub mod models;
// pub mod database; // Disabled for now
pub mod alerts;
pub mod auth;
pub mod ocr;
pub mod services;
pub mod utils;

// Scanner module is only available for native targets (not WASM)
#[cfg(not(target_arch = "wasm32"))]
pub mod scanner;

pub use app::TemplateApp;

// Re-export commonly used types
pub use models::{PriceRecord, Product, Store};
// pub use auth::{User, AuthManager}; // Disabled for now
// pub use database::Database; // Disabled for now
