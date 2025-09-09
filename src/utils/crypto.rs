use anyhow::Result;
// use bcrypt::{hash, verify, DEFAULT_COST}; // Disabled for now

/// Hash a password using bcrypt (stub implementation)
pub fn hash_password(password: &str) -> Result<String> {
    // Stub implementation - would use bcrypt in real app
    Ok(format!("hashed_{}", password))
}

/// Verify a password against a hash (stub implementation)
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    // Stub implementation - would use bcrypt in real app
    Ok(hash == &format!("hashed_{}", password))
}

/// Generate a random salt (not needed for bcrypt as it includes salt)
pub fn generate_salt() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}
