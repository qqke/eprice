use once_cell::sync::Lazy;
use regex::Regex;

// Email validation regex
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").expect("Invalid email regex")
});

/// Validate email format
pub fn validate_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email)
}

/// Validate password strength
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 6 {
        return Err("Password must be at least 6 characters long".to_string());
    }

    if password.len() > 128 {
        return Err("Password must be less than 128 characters".to_string());
    }

    // Check for at least one letter and one number
    let has_letter = password.chars().any(|c| c.is_alphabetic());
    let has_number = password.chars().any(|c| c.is_numeric());

    if !has_letter {
        return Err("Password must contain at least one letter".to_string());
    }

    if !has_number {
        return Err("Password must contain at least one number".to_string());
    }

    Ok(())
}

/// Validate username
pub fn validate_username(username: &str) -> Result<(), String> {
    if username.trim().is_empty() {
        return Err("Username cannot be empty".to_string());
    }

    if username.len() < 3 {
        return Err("Username must be at least 3 characters long".to_string());
    }

    if username.len() > 50 {
        return Err("Username must be less than 50 characters".to_string());
    }

    // Check for valid characters (alphanumeric and underscore)
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Username can only contain letters, numbers, and underscores".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com"));
        assert!(validate_email("user.name@domain.co.jp"));
        assert!(!validate_email("invalid.email"));
        assert!(!validate_email("@domain.com"));
        assert!(!validate_email("user@"));
    }

    #[test]
    fn test_password_validation() {
        assert!(validate_password("password123").is_ok());
        assert!(validate_password("abc123").is_ok());
        assert!(validate_password("short").is_err()); // Too short
        assert!(validate_password("nouppercase123").is_ok());
        assert!(validate_password("nonumbers").is_err()); // No numbers
        assert!(validate_password("123456").is_err()); // No letters
    }

    #[test]
    fn test_username_validation() {
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("test_user").is_ok());
        assert!(validate_username("ab").is_err()); // Too short
        assert!(validate_username("").is_err()); // Empty
        assert!(validate_username("user-name").is_err()); // Invalid character
    }
}
