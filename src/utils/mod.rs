pub mod crypto;
pub mod file_utils;
pub mod notification;
pub mod validation;

pub use crypto::hash_password;
pub use file_utils::{ensure_directory_exists, get_app_data_dir};
pub use notification::NotificationService;
pub use validation::{validate_email, validate_password};

use anyhow::Result;
use std::path::PathBuf;

/// Get the application's data directory
pub fn get_data_directory() -> Result<PathBuf> {
    get_app_data_dir()
}

/// Initialize required directories for the application
pub fn initialize_directories() -> Result<()> {
    let data_dir = get_data_directory()?;
    ensure_directory_exists(&data_dir)?;
    ensure_directory_exists(&data_dir.join("images"))?;
    ensure_directory_exists(&data_dir.join("receipts"))?;
    ensure_directory_exists(&data_dir.join("cache"))?;
    Ok(())
}
