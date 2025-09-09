use serde::{Deserialize, Serialize};

/// Login request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub remember_me: bool,
}

/// Registration request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub password_confirm: String,
}

impl RegisterRequest {
    /// Validate the registration request
    pub fn validate(&self) -> Result<(), String> {
        if self.username.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }

        if self.username.len() < 3 {
            return Err("Username must be at least 3 characters long".to_string());
        }

        if !self.email.contains('@') {
            return Err("Invalid email format".to_string());
        }

        if self.password.len() < 6 {
            return Err("Password must be at least 6 characters long".to_string());
        }

        if self.password != self.password_confirm {
            return Err("Passwords do not match".to_string());
        }

        Ok(())
    }
}

// Re-export the User model from the main models module
pub use crate::models::User;
