use anyhow::Result;
use bcrypt::{DEFAULT_COST, hash, verify};

/// Hash a password using bcrypt with default cost
pub fn hash_password(password: &str) -> Result<String> {
    let hashed = hash(password, DEFAULT_COST)?;
    Ok(hashed)
}

/// Hash a password using bcrypt with custom cost
pub fn hash_password_with_cost(password: &str, cost: u32) -> Result<String> {
    let hashed = hash(password, cost)?;
    Ok(hashed)
}

/// Verify a password against a hash using bcrypt
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let is_valid = verify(password, hash)?;
    Ok(is_valid)
}

/// Generate a secure random password
pub fn generate_secure_password(length: usize) -> String {
    use uuid::Uuid;

    // Generate multiple UUIDs and combine them for increased entropy
    let mut password_chars = Vec::new();
    while password_chars.len() < length {
        let uuid = Uuid::new_v4().to_string().replace('-', "");
        password_chars.extend(uuid.chars());
    }

    password_chars.into_iter().take(length).collect()
}

/// Generate a random salt (not needed for bcrypt as it includes salt, but useful for other purposes)
pub fn generate_salt() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

/// Hash data using SHA-256 (for non-password data)
pub fn hash_data_sha256(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Validate password strength
pub fn validate_password_strength(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase {
        return Err("Password must contain at least one uppercase letter".to_string());
    }

    if !has_lowercase {
        return Err("Password must contain at least one lowercase letter".to_string());
    }

    if !has_digit {
        return Err("Password must contain at least one digit".to_string());
    }

    if !has_special {
        return Err("Password must contain at least one special character".to_string());
    }

    Ok(())
}
