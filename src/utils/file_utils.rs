use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Ensure a directory exists, creating it if necessary
pub fn ensure_directory_exists<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        fs::create_dir_all(path)?;
        log::info!("Created directory: {}", path.display());
    }
    Ok(())
}

/// Get the application's data directory
pub fn get_app_data_dir() -> Result<PathBuf> {
    // For this example, we'll use a simple data directory in the current working directory
    // In a real application, you might want to use platform-specific directories
    let current_dir = std::env::current_dir()?;
    Ok(current_dir.join("data"))
}

/// Get the data directory for the application
pub fn get_data_directory() -> Result<PathBuf> {
    get_app_data_dir()
}

/// Initialize required directories for the application
pub fn initialize_directories() -> Result<()> {
    let data_dir = get_data_directory()?;
    ensure_directory_exists(&data_dir)?;

    // Create subdirectories
    ensure_directory_exists(data_dir.join("images"))?;
    ensure_directory_exists(data_dir.join("receipts"))?;
    ensure_directory_exists(data_dir.join("cache"))?;

    Ok(())
}

/// Save data to a file
pub fn save_to_file<P: AsRef<Path>>(path: P, data: &[u8]) -> Result<()> {
    let path = path.as_ref();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_directory_exists(parent)?;
    }

    fs::write(path, data)?;
    Ok(())
}

/// Load data from a file
pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let path = path.as_ref();
    let data = fs::read(path)?;
    Ok(data)
}

/// Check if a file exists
pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// Get file extension
pub fn get_file_extension<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Generate a unique filename with timestamp
pub fn generate_unique_filename(prefix: &str, extension: &str) -> String {
    let timestamp = chrono::Utc::now().timestamp();
    format!("{}_{}.{}", prefix, timestamp, extension)
}
